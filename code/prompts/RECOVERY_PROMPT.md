# RECOVERY - When Things Go Wrong

## Purpose

Use this prompt when:
- Codebase is broken and won't compile
- Tests fail in ways Ralph can't fix
- Ralph is going in circles
- Errors overflow context window
- Progress has stalled
- Need to reset to stable state

**This prompt helps diagnose and recover from failure states.**

---

## PHASE 1: DIAGNOSIS

### 1.1 Identify the Problem Type

Ask yourself / the user:

| Symptom | Problem Type | Go To |
|---------|--------------|-------|
| Won't compile, too many errors | Cascading failures | Section 2.1 |
| Tests fail repeatedly | Stuck loop | Section 2.2 |
| Same files modified over and over | Circular fixes | Section 2.3 |
| Duplicate implementations appearing | Search failure | Section 2.4 |
| Placeholders passing tests | Weak back pressure | Section 2.5 |
| Context window full of errors | Overflow | Section 2.6 |
| No progress for many loops | Stuck | Section 2.7 |

### 1.2 Gather Information

Use subagents to collect:

```bash
# Current git state
git status
git log --oneline -20
git describe --tags --abbrev=0

# Build output
{BUILD_COMMAND} 2>&1 | tee /tmp/build_errors.txt

# Test output  
{TEST_COMMAND} 2>&1 | tee /tmp/test_errors.txt

# Error counts
wc -l /tmp/build_errors.txt
wc -l /tmp/test_errors.txt
```

### 1.3 Find Last Stable Point

```bash
# List tags (stable points)
git tag -l --sort=-version:refname | head -10

# Check if tag builds
git stash
git checkout {tag}
{BUILD_COMMAND}
{TEST_COMMAND}
git checkout -
git stash pop
```

---

## PHASE 2: RECOVERY STRATEGIES

### 2.1 Cascading Failures (Too Many Errors)

**Symptom:** Build fails with hundreds of errors, all stemming from one issue.

**Diagnosis:**
```bash
# Find the root cause - often first few errors
{BUILD_COMMAND} 2>&1 | head -50
```

**Recovery:**

Option A - Fix root cause:
1. Identify the first/foundational error
2. Fix ONLY that one issue
3. Rebuild and reassess
4. Repeat until errors are manageable

Option B - Reset to stable:
```bash
git reset --hard {last_stable_tag}
```

Option C - Cherry-pick good changes:
```bash
git reset --hard {last_stable_tag}
git cherry-pick {good_commit_1}
git cherry-pick {good_commit_2}
```

### 2.2 Stuck Loop (Tests Keep Failing)

**Symptom:** Same tests fail repeatedly despite attempts to fix.

**Diagnosis:**
1. Are the tests testing the right thing?
2. Is the spec ambiguous?
3. Is the implementation approach wrong?

**Recovery:**

Option A - Fresh approach:
1. Read the failing test carefully
2. Read the corresponding spec
3. Delete the broken implementation
4. Re-implement from scratch following spec

Option B - Test is wrong:
1. Review spec requirements
2. If test doesn't match spec, fix test
3. Document in plan.md why test changed

Option C - Spec is wrong:
1. Use "the oracle" (think hard)
2. Update spec with correct interpretation
3. Update test to match spec
4. Re-implement

### 2.3 Circular Fixes (Same Files Modified)

**Symptom:** File A is changed, then reverted, then changed again.

**Diagnosis:**
- Conflicting requirements?
- Incomplete understanding?
- Multiple implementations interfering?

**Recovery:**

1. Stop the loop
2. Review git history for the file:
   ```bash
   git log --oneline -20 -- {file}
   git diff {earlier_commit} {later_commit} -- {file}
   ```
3. Identify the conflicting changes
4. Create focused fix in plan.md:
   ```markdown
   - [ ] [P1][M][T{next-id}] RESOLVE: Stabilize {file}
   ```
5. Resume loop with this as top priority

### 2.4 Search Failure (Duplicates)

**Symptom:** Multiple implementations of same concept exist.

**Diagnosis:**
```bash
# Find duplicates
rg -n "function_name" src/
rg -n "similar_pattern" src/
```

**Recovery:**

1. Identify the canonical implementation (matches spec)
2. List all duplicates
3. Update plan.md:
   ```markdown
   - [ ] [P1][M][T{next-id}] DEDUPLICATE: {concept}
   ```
4. Add to PROMPT.md stronger search rules:
   ```markdown
   9999. Before implementing {concept}, search:
   - "{exact_term}"
   - "{alt_term_1}"
   - "{alt_term_2}"
   ```

### 2.5 Weak Back Pressure (Placeholders Pass)

**Symptom:** Code has TODOs/stubs but tests pass.

**Diagnosis:**
```bash
# Find placeholders
rg -n "TODO|FIXME|todo!|unimplemented!|NotImplementedError" src/
```

**Recovery:**

1. Add placeholder detection to CI:
   ```bash
   # Add to test script
   if rg -q "TODO|todo!|unimplemented!" src/; then
     echo "ERROR: Placeholders found"
     exit 1
   fi
   ```

2. Add stricter tests that verify actual behavior

3. Update plan.md with each placeholder:
   ```markdown
   - [ ] [P1][M][T{next-id}] PLACEHOLDER: {function} in {file}
   ```

4. Escalate priority in prompt:
   ```markdown
   999999999. SEARCH FOR PLACEHOLDERS BEFORE COMMITTING.
   ```

### 2.6 Context Overflow (Too Much Error Output)

**Symptom:** So many errors that context window fills up.

**Recovery - Cross-AI Strategy:**

1. Export errors to file:
   ```bash
   {BUILD_COMMAND} 2>&1 > errors.txt
   ```

2. Feed to different AI (Gemini, GPT, etc.):
   ```
   These are build errors for a {LANGUAGE} project.
   Please analyze and:
   1. Identify the ROOT CAUSE errors (not cascading)
   2. Prioritize fixes
   3. Create a step-by-step fix plan
   ```

3. Use that plan to create focused fix_plan.md

4. Resume Ralph with the focused plan

### 2.7 Stuck (No Progress)

**Symptom:** Many loops but nothing completed.

**Diagnosis:**
- Task too complex?
- Specs too vague?
- Dependencies not met?
- Blocked on external factor?

**Recovery:**

1. Review plan.md - is it realistic?
2. Break down large tasks:
   ```markdown
   # Instead of:
   - [ ] [P1][L][T9] Implement entire auth system
   
   # Do:
   - [ ] [P1][S][T10] Define auth types
   - [ ] [P1][M][T11] Implement token generation
   - [ ] [P1][M][T12] Implement token validation
   - [ ] [P1][M][T13] Implement login endpoint
   ```

3. Clear blockers:
   - Missing dependencies → add to Critical
   - Missing specs → run spec phase
   - External factors → document and skip

4. Fresh plan:
   ```bash
   # Archive old plan
   mv plan.md plan_old.md
   
   # Regenerate
   cat .agents/code/PLANNING_PROMPT.md | claude --dangerously-skip-permissions
   ```

---

## PHASE 3: PREVENTION

### 3.1 Strengthen Back Pressure

Add to validation chain:
```bash
# Format
{FORMAT_CHECK} || exit 1

# Lint
{LINT_CHECK} || exit 1

# Type check (CRITICAL for dynamic languages)
{TYPE_CHECK} || exit 1

# Placeholder check
rg -q "TODO|PLACEHOLDER|unimplemented" src/ && exit 1

# Build
{BUILD} || exit 1

# Tests
{TEST} || exit 1
```

### 3.2 Improve Search

Add to prompt:
```markdown
Before implementing {COMMON_DUPLICATE}:
1. Search: "{term1}", "{term2}", "{term3}"
2. Check: src/, lib/, test/
3. If found, DO NOT create new implementation
```

### 3.3 Tag More Often

After EVERY successful test run:
```bash
git tag -a "v0.0.{n}" -m "Stable: {description}"
git push --tags
```

More tags = more recovery points.

### 3.4 Keep Plans Fresh

When plan.md becomes cluttered:
```bash
# Archive and regenerate
mv plan.md plan_archive_$(date +%Y%m%d).md
cat .agents/code/PLANNING_PROMPT.md | claude --dangerously-skip-permissions
```

---

## DECISION TREE

```
Problem Detected
│
├── Can Ralph fix with more loops?
│   ├── Yes → Continue with tuned prompt
│   └── No → Manual intervention needed
│       │
│       ├── Is damage localized?
│       │   ├── Yes → Manual fix, document, resume
│       │   └── No → Consider reset
│       │       │
│       │       ├── Good tag to reset to?
│       │       │   ├── Yes → git reset --hard {tag}
│       │       │   └── No → Manual repair or fresh start
│       │       │
│       │       └── Errors overflow context?
│       │           ├── Yes → Cross-AI recovery
│       │           └── No → Feed to Ralph
│       │
│       └── Always:
│           ├── Document what happened
│           ├── Update prompt to prevent recurrence
│           └── Add to AGENTS.md "Common Issues"
```

---

## QUICK COMMANDS

```bash
# Check status
git status && git log --oneline -5

# List stable points
git tag -l --sort=-version:refname | head -10

# Reset to last tag
git reset --hard $(git describe --tags --abbrev=0)

# See recent changes
git diff HEAD~5

# Export errors
{BUILD_COMMAND} 2>&1 > /tmp/errors.txt
wc -l /tmp/errors.txt

# Find placeholders
rg -n "TODO|FIXME|todo!|unimplemented!" src/

# Find duplicates
rg -hn "^fn |^def |^function " src/ | sort | uniq -d
```

---

## RULES

1. **Diagnose before acting** - Understand the problem
2. **Start with least destructive** - Fix before reset
3. **Document everything** - What happened, what fixed it
4. **Prevent recurrence** - Update prompts and checks
5. **Tag after recovery** - Create new stable point

## PRIORITY

9. Diagnose the problem type
99. Try targeted fix first
999. Reset to stable if needed
9999. Document in plan.md and AGENTS.md
99999. Update prompts to prevent recurrence

9999999. **ALWAYS CREATE TAG AFTER RECOVERY**
