# RALPH DEBUG - Bug Identification and Resolution

## Purpose

Use this prompt when:
- An error or bug is encountered during development
- Tests fail unexpectedly
- Runtime behavior doesn't match expectations
- User reports a defect
- Build succeeds but behavior is wrong

**This prompt systematically identifies, documents, tests, and tracks bugs.**

---

## PHASE ENTRY

When this prompt is invoked, first gather the error context:

> "Describe the error or unexpected behavior you're seeing. Include:
> - Error message (if any)
> - Expected behavior
> - Actual behavior
> - Steps to reproduce (if known)"

---

## CONTEXT LOADING

Study these files first (in order):
- @.agents/code/SPEC.md - **Project specification** (what this project is and does)
- @.agents/code/AGENTS.md - Project build commands and structure
- @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
- @.agents/code/issues/ - Issue files (open + resolved)
- `metagent issues --status open` - Open issue list

---

## PART 1: ERROR ANALYSIS

### 1.1 Collect Error Information

Gather all available context:

```bash
# Get recent build/test output
{BUILD_COMMAND} 2>&1 | tee /tmp/build_output.txt
{TEST_COMMAND} 2>&1 | tee /tmp/test_output.txt

# Recent changes that might have caused the issue
git log --oneline -10
git diff HEAD~5 --stat
```

### 1.2 Classify Error Type

| Type | Symptoms | Priority |
|------|----------|----------|
| Compile/Build | Won't compile, type errors | Critical |
| Runtime Crash | Panics, exceptions, SIGABRT | Critical |
| Logic Error | Wrong output, incorrect behavior | High |
| Performance | Slow, memory leak, timeout | Medium |
| Edge Case | Only fails under specific conditions | Medium |
| Regression | Was working, now broken | High |

---

## PART 2: TASK IDENTIFICATION

### 2.1 Find Related Task/Spec

Spawn subagents (up to 50) to identify which task's spec might be related to this error.

**Search strategy:**
```
For each task in .agents/code/tasks/:
├── Read spec/overview.md - Does this task touch affected area?
├── Read spec/types.md - Are error-related types defined here?
├── Read spec/*.md - Any functions matching the error?
├── Check plan.md - Was related work recently done?
└── Score relevance (0-10)
```

**Search patterns:**
```
From error message, extract:
├── File names mentioned
├── Function names mentioned
├── Type names mentioned
├── Module names mentioned
└── Error codes/types

Search each task's spec/ for matches
```

### 2.2 Report Findings

```markdown
## Task Relevance Analysis

| Task | Relevance | Evidence |
|------|-----------|----------|
| {taskname} | High (8/10) | Spec defines {function} that's failing |
| {taskname2} | Medium (5/10) | Uses types involved in error |
| {taskname3} | Low (2/10) | Unrelated module |

**Primary task identified:** {taskname}
**Confidence:** High/Medium/Low
**Reason:** {why this task is most likely related}
```

If no matching task found:
- Treat as an unassigned bug and create an issue with no task:
  `metagent issue add --title "<short title>" --priority P1 --type bug --source debug --stdin-body`
- Ask the user whether to create a new task or hotfix later, but **do not** create a task unless asked

### 2.3 Load Task Context

Once taskname identified, read in order:
```
.agents/code/tasks/{taskname}/
├── spec/overview.md
├── spec/types.md
├── spec/*.md (all other specs)
├── plan.md
└── (use `metagent issues --task {taskname}` to review open issues)
```

---

## PART 3: ROOT CAUSE ANALYSIS

### 3.1 Trace the Error

Spawn subagents to analyze:

```
Error trace analysis:
├── Where does the error originate? (file:line)
├── What function is it in?
├── What calls that function?
├── What data was passed in?
├── What state was the system in?
└── Where does spec say this should work?
```

### 3.2 Compare to Spec

```markdown
## Spec vs Reality

| Aspect | Spec Says | Code Does | Match? |
|--------|-----------|-----------|--------|
| Input validation | {spec} | {actual} | Yes/No |
| Error handling | {spec} | {actual} | Yes/No |
| Return value | {spec} | {actual} | Yes/No |
| Edge case: {case} | {spec} | {actual} | Yes/No |
```

### 3.3 Identify Root Cause

```markdown
## Root Cause

**Category:** [Spec Gap | Implementation Bug | Test Gap | Integration Issue]

**Description:** {One paragraph explaining exactly why this fails}

**Evidence:**
- {proof point 1}
- {proof point 2}

**Affected Code:**
- `{file}:{line}` - {what's wrong}
- `{file}:{line}` - {what's wrong}
```

---

## PART 4: BUG DOCUMENTATION

### 4.1 Create Issue via CLI

If a task is identified, include `--task {taskname}`. If no task is identified, omit `--task` (unassigned).

Use this template:

```bash
cat <<'EOF' | metagent issue add --title "{Human-Readable Title}" --task {taskname} --priority P1 --type bug --source debug --stdin-body
# Bug: {Human-Readable Title}

## Description
{Clear description of the bug}

## Reproduction
1. {Step 1}
2. {Step 2}
3. {Step 3}

**Expected:** {What should happen}
**Actual:** {What happens instead}

## Error Output
```
{Error message/stack trace}
```

## Root Cause
{Explanation from Part 3.3}

## Affected Files
- `{file}:{line}` - {issue description}

## Related Spec (if known)
- `spec/{file}.md` - {relevant section}

## Fix Strategy
{High-level approach to fix}

## Test Plan
- [ ] Write failing test that reproduces the bug
- [ ] Implement fix
- [ ] Verify test passes
- [ ] Check for regressions
EOF
```

### 4.2 Update Task Plan (if assigned)

If the issue is assigned to a task, add it to `.agents/code/tasks/{taskname}/plan.md` and include the issue ID:

```markdown
## Bugs

> Bugs discovered during implementation - fix before proceeding

- [ ] BUG: {bug-title} (priority: {priority})
  - Issue: {issue-id}
  - Root cause: {brief}
  - Fix approach: {brief}
  - Blocks: {what can't proceed until fixed}
```

---

## PART 5: WRITE FAILING TEST

If the issue is unassigned, pause here and ask whether to create a task or hotfix (or assign it to an existing task) before writing tests or making code changes.
Use `metagent issue assign {issue-id} --task {taskname}` when you have a task.

### 5.1 Identify Test Location

Based on codebase patterns, determine:
```
Test file location:
├── Following project conventions
├── Near related tests
└── Using correct naming pattern
```

### 5.2 Design Test Case

```markdown
## Test Design

**Test Name:** `test_{describes_failure_condition}`

**Setup:**
- {Required state/fixtures}

**Action:**
- {Code that triggers the bug}

**Assert:**
- {What should be true but isn't}
```

### 5.3 Write the Test

Create minimal test that fails:

```{language}
// Test for bug: {bug-title}
// Issue: .agents/code/issues/{issue-id}.md
// Expected: FAIL until bug is fixed

{test implementation}
```

### 5.4 Confirm Test Fails

```bash
# Run specific test
{TEST_COMMAND} --filter {test_name}
```

Expected output: FAIL (confirms bug is reproducible)

### 5.5 Update Issue File

Add to progress log in issue file:

```markdown
### {date} - Failing Test Added
- Test: `{test_file}::{test_name}`
- Confirms: Bug is reproducible
- Test output: {summary}
```

---

## PART 6: FIX TRACKING

### 6.1 Implementation Attempts

Track each fix attempt in the issue file:

```markdown
### {date} - Fix Attempt #{n}
- Approach: {what was tried}
- Files changed: {list}
- Result: {Pass/Fail}
- If failed: {why it didn't work}
```

### 6.2 Fix Verification

When a fix is attempted:

```bash
# Run the failing test
{TEST_COMMAND} --filter {test_name}

# If passes, run full test suite
{TEST_COMMAND}

# Check for regressions
git diff --stat
```

### 6.3 Bug Resolution

When fixed, update issue file:

```markdown
---

## Resolution

**Fixed:** {date}
**Commits:** {commit hash(es)}
**Fix:** {Description of what was changed}

### Final Progress Log Entry

### {date} - RESOLVED
- Fix verified: Test passes
- No regressions: Full suite passes
- Commits: {hashes}
```

Mark the issue resolved:
- `metagent issue resolve {issue-id} --resolution "{what changed}"`

Update `plan.md`:
- Mark bug task as complete
- Remove from blocking items

---

## DECISION TREE

```
Bug Reported
│
├── Gather error information
│   └── Classify error type
│
├── Identify related task
│   ├── Found matching task?
│   │   ├── Yes → Load task context
│   │   └── No → Create unassigned issue, ask for triage
│   │
│   └── Analyze root cause
│       └── Compare to spec
│
├── Document the bug
│   ├── Create issue via CLI
│   └── Update task plan.md (if assigned)
│
├── Write failing test
│   ├── Confirm test fails
│   └── Document in issue file
│
└── Track fix attempts
    ├── Log each attempt
    ├── Verify when fixed
    └── Update all tracking files
```

---

## QUICK COMMANDS

```bash
# List all issues
metagent issues --status all
find .agents/code/issues -name "*.md" | head -20

# Search for specific bug
rg "BUG:" .agents/code/tasks/*/plan.md

# Count open unassigned issues
metagent issues --unassigned

# Run specific failing test
{TEST_COMMAND} --filter {test_name}

# Recent changes
git log --oneline -10 -- {affected_files}
```

---

## SUBAGENT USAGE

| Task | Max Subagents | Purpose |
|------|---------------|---------|
| Task identification | 50 | Find related task/spec |
| Root cause analysis | 30 | Trace error through code |
| Pattern matching | 20 | Find similar bugs/fixes |
| Test discovery | 20 | Find related tests |

---

## RULES

1. **Document first** - Create issue file before fixing
2. **Test first** - Write failing test before implementing fix
3. **Track progress** - Log every attempt in issue file
4. **Update all locations** - Issue file and plan.md
5. **Verify thoroughly** - Full test suite after fix

## PRIORITY

9. Identify which task's spec is related
99. Analyze root cause against spec
999. Create issue file with full documentation
9999. Write failing test that confirms bug
99999. Track progress in issue file

9999999. **DOCUMENT THE BUG BEFORE FIXING IT**
99999999. **WRITE FAILING TEST BEFORE IMPLEMENTING FIX**
