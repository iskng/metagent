# REFRESH - Regenerate Stale Plans

## Purpose

Use this prompt when:
- plan.md has become stale or inaccurate
- Too many completed items cluttering the plan
- Priorities have shifted
- Ralph is working on wrong things
- Need fresh analysis of current state

**This prompt refreshes plan.md based on current reality.**

---

## WHEN TO REFRESH

Signs plan needs refresh:
- Ralph going in circles
- Tasks no longer make sense
- Completed items cluttering view
- Many "Discovered Issues" accumulated
- Specs have changed significantly
- Priorities shifted since last plan

> "I have deleted the TODO list multiple times... if I throw the TODO list out... 
> You run a Ralph loop with explicit instructions to generate a new TODO list."

---

## PHASE 1: ARCHIVE CURRENT STATE

### 1.1 Save Current Plan

```bash
# Create archive with timestamp
cp .agents/code/tasks/{taskname}/plan.md \
   .agents/code/tasks/{taskname}/plan_archive_$(date +%Y%m%d_%H%M%S).md

# Or if multiple archives exist
mv .agents/code/tasks/{taskname}/plan.md \
   .agents/code/tasks/{taskname}/archived/plan_$(date +%Y%m%d).md
```

### 1.2 Extract Valuable Information

Before discarding, extract:
- Discovered issues (still relevant?)
- Spec clarifications (still needed?)
- Notes and decisions
- Partially completed work

---

## PHASE 2: FRESH ANALYSIS

### 2.1 Spec Compliance Audit

Use subagents (up to 100) to compare code against specs:

For each source file:
1. Identify related spec(s)
2. Compare:
   - Functions: specified vs implemented
   - Signatures: match exactly?
   - Error handling: all cases covered?
   - Edge cases: all handled?
3. Report deviations

Output:
```markdown
## Spec Compliance

### Fully Implemented
- {module}.{function} ✓

### Partially Implemented
- {module}.{function} - Missing: {what}

### Not Implemented
- {module}.{function} - per spec/{file}.md

### Deviates from Spec
- {module}.{function} - Spec says X, code does Y
```

### 2.2 Placeholder Hunt

Use subagents (up to 100) to find incomplete code:

```bash
# Search patterns
TODO
FIXME
PLACEHOLDER
HACK
XXX
TEMP
unimplemented!()
todo!()
raise NotImplementedError
pass  # empty body
throw new Error("not implemented")
panic("not implemented")
Default::default()  # suspicious
return null  # suspicious
return undefined  # suspicious
```

Output:
```markdown
## Placeholders Found

- {file}:{line} - `{placeholder_code}`
  - Function: {function_name}
  - Spec: spec/{file}.md
```

### 2.3 Test Coverage Analysis

Use subagents to map coverage:

1. List all public functions
2. List all test functions
3. Map: function → tests
4. Identify:
   - Untested functions
   - Functions with only happy-path tests
   - Disabled/skipped tests

Output:
```markdown
## Test Coverage

### Untested Functions
- {module}.{function}

### Needs More Tests
- {module}.{function} - Only happy path

### Disabled Tests
- {test_name} - Why: {reason if known}
```

### 2.4 Code Quality Issues

Use subagents to find:
- Commented-out code
- Dead/unreachable code
- Duplicate code
- Missing error handling
- Resource leaks

Output:
```markdown
## Code Quality Issues

- {file}:{line} - {issue_type}: {description}
```

---

## PHASE 3: CURRENT STATE SUMMARY

Compile analysis into summary:

```markdown
## Current State Analysis

### Spec Implementation Status
- Total requirements: {N}
- Fully implemented: {N} ({%})
- Partially implemented: {N} ({%})
- Not implemented: {N} ({%})
- Deviating: {N} ({%})

### Code Quality
- Placeholders: {N}
- Untested functions: {N}
- Quality issues: {N}

### Key Findings
1. {Major finding}
2. {Major finding}
3. {Major finding}
```

---

## PHASE 4: GENERATE NEW PLAN

### 4.1 Prioritize Based on Analysis

**Critical (Blocking):**
- Foundational pieces others need
- Blocking bugs
- Deviations from spec that affect other code

**High Priority:**
- Incomplete spec requirements
- Important placeholders
- Core functionality gaps

**Medium Priority:**
- Test coverage gaps
- Quality issues
- Non-blocking improvements

**Low Priority:**
- Documentation
- Optimization
- Nice-to-haves

### 4.2 Create Fresh plan.md

```markdown
# Implementation Plan - {taskname}

> Refreshed: {date}
> Previous plan archived: plan_archive_{date}.md
> Status: READY

## Current State

{Summary from Phase 3}

## Critical (Blocking)

- [ ] {task} (complexity: S/M/L)
  - Why critical: {reason}
  - Files: {files}

## High Priority (Core)

- [ ] {task} per spec/{file}.md (complexity: M)
  - Status: {not started/partial}
  - Gap: {what's missing}
  - Files: {files}

## Medium Priority

- [ ] {task} (complexity: M)

## Low Priority

- [ ] {task}

## Carried Forward

> From previous plan - still relevant

- [ ] {issue from old plan}

## Resolved

> Issues from previous plan that are now fixed

- [x] {was issue, now fixed}

## Notes

- Refresh reason: {why refreshed}
- Key decisions: {any new decisions}
```

---

## PHASE 5: VALIDATION

### 5.1 Coverage Check

Ensure every spec requirement is covered:
- Either marked as complete
- Or has a task in the plan

### 5.2 Priority Validation

- Critical items truly block others?
- High priority aligns with spec goals?
- No important items buried in Low?

### 5.3 Sanity Check

- Total scope reasonable?
- Complexity estimates realistic?
- Dependencies make sense?

---

## PHASE 6: HANDOFF

```markdown
✅ Plan refreshed for: {taskname}

Changes from previous plan:
- Removed: {N} completed items
- Added: {N} new items from analysis
- Reprioritized: {N} items

Summary:
- Critical: {N}
- High: {N}
- Medium: {N}
- Low: {N}

Previous plan archived at:
  .agents/code/tasks/{taskname}/plan_archive_{date}.md

Resume build loop:
  metagent run {taskname}
```

---

## QUICK REFRESH (Minimal)

For quick cleanup without full analysis:

```bash
# 1. Archive
cp plan.md plan_old.md

# 2. Remove completed items (keep last 5 for context)
# Edit plan.md, move old completed items to archive

# 3. Re-sort priorities based on current understanding

# 4. Resume
```

---

## RULES

1. **Archive before refreshing** - Never lose history
2. **Analyze current state** - Don't guess, check
3. **Carry forward relevant items** - Don't lose discovered issues
4. **Validate coverage** - Every spec requirement covered
5. **Fresh priorities** - Based on current reality

## PRIORITY

9. Archive current plan first
99. Analyze actual codebase state
999. Generate fresh prioritized plan
9999. Ensure spec coverage
99999. Validate before resuming

9999999. **ARCHIVE BEFORE REFRESH - NEVER LOSE HISTORY**
