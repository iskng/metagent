# RALPH PLANNING PHASE - Specification to Plan

## Purpose

This prompt facilitates the **Planning Phase** - the second phase of the Ralph workflow.

Your role:
1. **Load specifications** from the task's spec/ directory
2. **Research codebase** to understand current state
3. **Generate plan.md** with prioritized, actionable tasks
4. **Map dependencies** and identify blockers

**This phase produces plan.md. NO implementation.**

---

## PHASE ENTRY

When this prompt is invoked, first ask:

> "Which task should I plan? (e.g., 'auth-system')"

Then verify:
1. Task directory exists: `.agents/code/tasks/{taskname}/`
2. Specs exist: `.agents/code/tasks/{taskname}/spec/`
3. Spec phase complete: check for `.status` file

If specs missing, direct user to run spec phase first.

---

## CONTEXT LOADING

Study these files in order:
```
0a. @.agents/code/SPEC.md - **Project specification** (what this project is and does)
0b. @.agents/code/AGENTS.md - Build/test commands
0c. @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns
0d. @.agents/code/tasks/{taskname}/spec/overview.md - Goals and architecture
0e. @.agents/code/tasks/{taskname}/spec/types.md - Type definitions
0f. @.agents/code/tasks/{taskname}/spec/*.md - All other spec files
0g. @.agents/code/tasks/{taskname}/plan.md - Current plan (if exists)
```

Understanding the project specification (SPEC.md) is **critical** - it provides the high-level context for how this task fits into the overall project architecture, what abstractions exist, and what patterns to follow.

---

## PART 1: SPECIFICATION ANALYSIS

### 1.1 Extract Requirements

For each spec file, spawn subagents to extract:

```
From overview.md:
├── Goals (what must be achieved)
├── Non-goals (what to avoid)
├── Success criteria (how to verify)
└── Dependencies (what must exist)

From types.md:
├── Types to implement
├── Fields per type
├── Validation rules
└── Type relationships

From {module}.md:
├── Functions to implement
├── Function signatures
├── Error cases to handle
├── Test requirements

From errors.md:
├── Error types to create
├── Error handling patterns
└── Logging requirements
```

### 1.2 Build Requirements Matrix

Create a structured list:

```markdown
## Requirements Extracted

### Types
- [ ] {TypeName} - {field_count} fields - spec/types.md
- [ ] {TypeName2} - {field_count} fields - spec/types.md

### Functions
- [ ] {module}.{function} - spec/{module}.md
- [ ] {module}.{function2} - spec/{module}.md

### Tests
- [ ] {test_case} - spec/{module}.md
- [ ] {test_case2} - spec/{module}.md

### Integrations
- [ ] {integration_point} - spec/overview.md
```

---

## PART 2: CODEBASE STATE RESEARCH

Use subagents (up to 100 parallel) to understand current state.

### 2.1 What Already Exists?

**CRITICAL:** Prevent duplicate implementations.

Spawn subagents to search:
```
For each requirement, search:
├── Exact name matches
├── Partial name matches
├── Synonym matches
├── Concept matches
└── Test files (reveal implementations)

Search locations:
├── src/
├── lib/
├── internal/
├── pkg/
├── test/
└── examples/
```

**Categorize findings:**
- ✅ Fully implemented - needs no work
- 🔶 Partially implemented - needs completion
- ❌ Not implemented - needs creation
- ⚠️ Implemented differently - needs alignment

### 2.2 Find Placeholders and TODOs

Spawn subagents to search for incomplete code:

```
Search patterns:
├── TODO
├── FIXME
├── PLACEHOLDER
├── HACK
├── XXX
├── unimplemented!() (Rust)
├── todo!() (Rust)
├── raise NotImplementedError (Python)
├── pass # (Python empty)
├── throw new Error("not implemented") (JS/TS)
├── panic("not implemented") (Go)
└── Default::default() in suspicious contexts
```

### 2.3 Analyze Patterns to Follow

Spawn subagents to find similar implementations:
```
Search for:
├── Similar modules (precedent)
├── Similar functions (pattern)
├── Similar tests (test structure)
└── Similar error handling
```

### 2.4 Map Impact

Spawn subagents to understand:
```
For code being changed:
├── What calls it? (callers)
├── What does it call? (dependencies)
├── What tests cover it?
└── What might break?

For new code:
├── Where should it live?
├── What patterns to follow?
├── What should test it?
```

---

## PART 3: DEPENDENCY ANALYSIS

### 3.1 Build Dependency Graph

From requirements and research, build:

```
{requirement_A}
├── Depends on: nothing (foundational)
├── Blocks: {B}, {C}
└── Complexity: S

{requirement_B}
├── Depends on: {A}
├── Blocks: {D}
└── Complexity: M

{requirement_C}
├── Depends on: {A}
├── Blocks: {D}
└── Complexity: M

{requirement_D}
├── Depends on: {B}, {C}
├── Blocks: nothing (leaf)
└── Complexity: L
```

### 3.2 Identify Critical Path

Find the longest dependency chain - this determines minimum time to completion.

### 3.3 Detect Circular Dependencies

If A depends on B and B depends on A:
- Flag as blocker
- Note in plan
- Suggest resolution approach

---

## PART 4: GAP ANALYSIS

### 4.1 Spec vs Current State

| Requirement | Spec Says | Current State | Gap |
|-------------|-----------|---------------|-----|
| {type} | {spec} | Not found | Create |
| {function} | {spec} | Stub exists | Complete |
| {test} | {spec} | Missing | Create |
| {integration} | {spec} | Partial | Extend |

### 4.2 Missing from Specs

If codebase research reveals things specs don't cover:
- Note as "Discovered Requirements"
- Flag for user attention

---

## PART 5: PLAN GENERATION

### 5.1 Priority Classification

**Critical (Blocking):**
- Foundation that everything depends on
- Core types other code needs
- Blockers that prevent all other work
- Usually: S or M complexity

**High Priority (Core):**
- Main functionality from specs
- Primary user-facing features
- Required for success criteria
- Usually: M or L complexity

**Medium Priority (Important):**
- Secondary features
- Quality improvements
- Extended test coverage
- Non-blocking integrations

**Low Priority (Polish):**
- Documentation
- Performance optimization
- Code cleanup
- Nice-to-haves

### 5.2 Task Format

Each task should be actionable:

```markdown
- [ ] {ACTION_VERB} {specific_thing} per spec/{file}.md (complexity: S/M/L)
  - Depends on: {task_ids or "nothing"}
  - Blocks: {task_ids or "nothing"}
  - Files: {files to create or modify}
  - Tests: {test files to create or modify}
  - Notes: {any special considerations}
```

**Action verbs:** Create, Implement, Add, Update, Fix, Complete, Refactor, Test

### 5.3 Complexity Guidelines

| Size | Loops | Characteristics |
|------|-------|-----------------|
| S | 1-2 | Single file, clear requirements, straightforward |
| M | 3-5 | Multiple files, some complexity, clear path |
| L | 6+ | Many files, significant complexity, may need breakdown |

**If L complexity:** Consider breaking into smaller tasks.

---

## PART 6: WRITE PLAN.MD

Create/update `.agents/code/tasks/{taskname}/plan.md`:

```markdown
# Implementation Plan - {taskname}

> Generated: {date}
> Specs: .agents/code/tasks/{taskname}/spec/
> Status: READY

## Overview

{Brief description from spec/overview.md}

## Dependency Graph

```
{Visual representation}
```

## Critical (Blocking)

> Do these first - everything else depends on them

- [ ] {task} (complexity: S)
  - Depends on: nothing
  - Blocks: {items}
  - Files: {files}

- [ ] {task} (complexity: M)
  - Depends on: {items}
  - Blocks: {items}
  - Files: {files}

## High Priority (Core)

> Main functionality from specifications

- [ ] {task} per spec/{module}.md (complexity: M)
  - Depends on: {items}
  - Blocks: {items}
  - Files: {files}
  - Tests: {files}

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

## Spec Clarifications

> Questions for user or to resolve during implementation

- Q: {question}
  - Context: {why it matters}
  - Suggested: {proposed answer}

## Completed

> Move items here when done

## Notes

- Generated: {date}
- Spec files: {count}
- Total tasks: {count}
- Critical path: {description}
```

---

## PART 7: VALIDATION

### 7.1 Coverage Check

Verify every spec requirement maps to a task:

```
Spec Requirements: {count}
Plan Tasks: {count}
Coverage: {percentage}

Unmapped requirements:
- {requirement} - MISSING FROM PLAN (add it!)
```

### 7.2 Dependency Validation

- No circular dependencies
- No orphan tasks (depend on non-existent tasks)
- Critical path makes sense

### 7.3 Complexity Sanity Check

- No single task > L complexity without subtasks
- Complexity estimates are realistic
- Total scope is reasonable

---

## PART 8: COMPLETION

### 8.1 Summary to User

```
Plan complete for: {taskname}

Summary:
  Total tasks: {count}
  Critical: {count}
  High: {count}
  Medium: {count}
  Low: {count}

Critical path: {description}
Estimated loops: {rough estimate}

Issues found:
  Placeholders: {count}
  Spec clarifications: {count}
```

### 8.2 User Review

Ask:
> "Please review the plan. Should I adjust priorities or break down any tasks?"

### 8.3 Completion Marker

```bash
echo "PLAN_COMPLETE=$(date +%Y-%m-%d)" >> .agents/code/tasks/{taskname}/spec/.status
```

### 8.4 Handoff

```
✅ Plan complete for: {taskname}

Next: Run build loop
Command: while :; do cat .agents/code/tasks/{taskname}/PROMPT.md | claude-code; done

The build loop will:
- Pick tasks from plan.md
- Implement per specs
- Test and commit
- Update plan.md
```

---

## SUBAGENT USAGE SUMMARY

| Task | Max Subagents | Purpose |
|------|---------------|---------|
| Spec parsing | 20 | Extract requirements |
| Existing code search | 100 | Find what exists |
| Placeholder search | 50 | Find incomplete code |
| Pattern analysis | 50 | Find precedents |
| Impact mapping | 50 | Understand dependencies |
| Plan writing | 5 | Generate plan sections |

---

## RULES

1. **No implementation** - Plan only
2. **Exhaustive research** - Use subagents liberally
3. **Map everything** - Every spec requirement to a task
4. **Order correctly** - Dependencies before dependents
5. **Be specific** - Actionable tasks, not vague goals
6. **Estimate honestly** - Complexity reflects reality

## PRIORITY

9. Read ALL spec files before planning
99. Research codebase before generating tasks
999. Every requirement must map to a task
9999. Dependencies must be explicit
99999. Tasks must be actionable

9999999. **PLAN ONLY - NO IMPLEMENTATION**
99999999. **EVERY SPEC REQUIREMENT → TASK**
