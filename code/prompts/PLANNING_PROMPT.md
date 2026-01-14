# Planning Phase

When this prompt is invoked, if the taskname isn't provided first ask:

> "Which task should I plan? (e.g., 'auth-system')"

Then verify the task directory exists: `.agents/code/tasks/{taskname}/`

---

0a. Study @.agents/code/SPEC.md - **Project specification** (what this project is and does)

0b. Study @.agents/code/AGENTS.md - Build/test commands

0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns

0d. Study @.agents/code/tasks/{taskname}/spec/*.md - All specification files

0e. Study @.agents/code/tasks/{taskname}/plan.md - Current plan (if exists) to determine starting point for research

---

First task is to study the spec/*.md files and plan.md (it may be incorrect or incomplete) and use up to 100 subagents to study existing source code and compare it against the specifications. From that create/update plan.md which is a bullet point list sorted in priority of the items which have yet to be implemented. Think extra hard and use the oracle to plan. Consider searching for TODO, minimal implementations and placeholders. Study plan.md to determine starting point for research and keep it up to date with items considered complete/incomplete using subagents.

Second task is to use up to 100 subagents to study existing tests and examples, then compare against the specifications. From that update plan.md with items that are missing or incomplete. Search for:
- TODO, FIXME, PLACEHOLDER, HACK, XXX
- unimplemented!(), todo!() (Rust)
- raise NotImplementedError, pass (Python)
- throw new Error("not implemented") (JS/TS)
- fatalError("not implemented") (Swift)

Third task is to build a dependency graph. For each requirement determine what it depends on and what it blocks. Order the plan so dependencies come before dependents.

---

For each spec requirement, spawn subagents to search:
- Exact name matches
- Partial name matches
- Synonym matches
- Concept matches
- Test files (reveal implementations)

Categorize findings:
- Fully implemented - needs no work
- Partially implemented - needs completion
- Not implemented - needs creation
- Implemented differently - needs alignment

---

Write plan.md with this structure:

```markdown
# Implementation Plan - {taskname}

> Generated: {date}
> Status: READY

## Overview

{Brief description from spec/overview.md}

## Critical (Blocking)

> Do these first - everything else depends on them

- [ ] {task} (complexity: S/M/L)
  - Depends on: nothing
  - Blocks: {items}
  - Files: {files}

## High Priority (Core)

> Main functionality from specifications

- [ ] {task} per spec/{module}.md (complexity: M)
  - Depends on: {items}
  - Blocks: {items}
  - Files: {files}

## Medium Priority

> Important but not blocking

- [ ] {task} (complexity: M)

## Low Priority

> Polish - defer if needed

- [ ] Documentation
- [ ] Performance optimization

## Discovered Issues

> Found during codebase research

- [ ] EXISTING: {placeholder found} in {file}
- [ ] EXISTING: {inconsistency} in {file}

## Placeholders to Replace

> Existing incomplete implementations

- [ ] PLACEHOLDER: {function} in {file}
  - Current: {what it does now}
  - Needed: {what spec requires}

## Completed

> Move items here when done
```

---

IMPORTANT: If codebase research reveals things specs don't cover, note as "Discovered Requirements" and flag for user attention. If you find something that should be in the specs but isn't, use a subagent to author the specification at spec/{module}.md (search before creating to ensure it doesn't already exist).

---

9. Study ALL spec files and existing plan before researching codebase

99. Research codebase exhaustively before generating tasks - use subagents liberally

999. Every spec requirement must map to a task in plan.md

9999. Keep plan.md up to date with your findings using subagents

99999. Think extra hard and use the oracle when planning complex items

999999. If you find inconsistencies in specs, use a subagent to resolve and update them

9999999. **PLAN ONLY - NO IMPLEMENTATION**

99999999. **EVERY SPEC REQUIREMENT MUST MAP TO A TASK**
