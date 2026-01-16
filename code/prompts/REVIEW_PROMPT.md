# REVIEW PHASE - Code Review for Task Implementation

## Purpose

Review all commits from a task implementation to identify issues and determine next steps.

---

## CONTEXT

Task being reviewed: `{task}`

Study these files first:
- @.agents/code/tasks/{task}/spec/ - Task specifications (what was supposed to be built)
- @.agents/code/tasks/{task}/plan.md - Implementation plan

{focus_section}

---

## STEP 1: Find Task Commits

Find all commits related to this task:

```bash
git log --oneline --grep="{task}"
```

For each commit, get the full diff:

```bash
git show <commit-hash>
```

---

## STEP 2: Review Each Commit

For each commit, analyze:

### Spec Compliance
- Does the implementation match the spec?
- Are there missing requirements?
- Are there scope creep or unauthorized changes?
- Were architectural decisions followed?

### Code Quality
- Does the code follow patterns in @.agents/code/TECHNICAL_STANDARDS.md?
- Are there any code smells (duplication, long functions, unclear naming)?
- Is error handling appropriate?

### Correctness
- Are there edge cases not handled?
- Are there potential bugs or race conditions?
- Is the logic correct?

### Security
- Any hardcoded secrets or credentials?
- Input validation issues?
- SQL injection, XSS, or other vulnerabilities?

### Testing
- Are there tests for new functionality?
- Do tests cover edge cases?
- Are tests meaningful (not just coverage padding)?

### Performance
- Any obvious performance issues (N+1 queries, unnecessary loops)?
- Memory leaks or resource cleanup issues?

---

## STEP 3: Document Issues

Create or update `.agents/code/tasks/{task}/issues.md` with findings:

```markdown
# Code Review - {task}

> Reviewed: {date}
> Commits: {list of commit hashes reviewed}
> Verdict: PASS | NEEDS BUILD FIXES | NEEDS SPEC CLARIFICATION

## Spec Issues

Issues that require returning to spec phase for clarification or decisions.

### Issue 1: {title}
- **Problem:** {missing/unclear requirement or architectural issue}
- **Decision needed:** {what needs to be decided}
- **Status:** open

## Build Issues

Issues that require returning to build phase for fixes.

### Issue 1: {title}
- **File:** {path}:{line}
- **Commit:** {hash}
- **Problem:** {bug, code quality issue, missing test}
- **Suggested fix:** {how to fix}
- **Status:** open

## Suggestions

Optional improvements (not blocking).

### Suggestion 1: {title}
- **Description:** {what could be improved}

---

## Review Summary

| Category | Count |
|----------|-------|
| Spec Issues | {n} |
| Build Issues | {n} |
| Suggestions | {n} |

**Verdict:** {PASS | NEEDS BUILD FIXES | NEEDS SPEC CLARIFICATION}
```

---

## STEP 4: Determine Next Stage

Based on your findings, signal the appropriate next stage:

### If SPEC ISSUES exist (any open spec issues):
The task needs spec clarification before proceeding.

```bash
metagent finish review --next spec
```

### If only BUILD ISSUES exist (no spec issues, but build issues):
The task needs code fixes.

```bash
metagent finish review --next build
```

### If PASS (no open spec or build issues):
The task is ready for completion.

```bash
metagent finish review
```

---

## STEP 5: Report to User

After creating/updating issues.md and signaling completion, provide a summary:

```
Review complete for: {task}

Commits reviewed: {count}
Verdict: {PASS | NEEDS BUILD FIXES | NEEDS SPEC CLARIFICATION}

Issues found:
  - Spec issues: {count}
  - Build issues: {count}
  - Suggestions: {count}

Next stage: {spec | build | completed}

Details: .agents/code/tasks/{task}/issues.md
```

---

## RULES

1. Review ALL commits for the task, not just the latest
2. Be specific - include file paths and line numbers
3. Suggest fixes, not just problems
4. Compare implementation against spec
5. Don't nitpick style if it matches project standards
6. Spec issues = requirements/architecture problems → back to spec
7. Build issues = implementation problems → back to build
8. When in doubt, it's a build issue (easier to fix)
