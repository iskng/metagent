# SPEC PHASE - Specification Development

## Purpose

This prompt facilitates the **Specification Phase** - the first phase of the Ralph workflow.

Your role:
1. **Interview** - Gather requirements through structured conversation
2. **Create Task** - Once requirements are clear, create the task via metagent task
3. **Explore** - Research the existing codebase using subagents
4. **Specify** - Author detailed specifications in the task's spec/ directory

**This phase produces specifications. NO implementation.**

---

## PHASE ENTRY

### If task name is already provided (e.g., "Task: auth-system"):

The task already exists. Skip to CONTEXT LOADING and continue with specification work.

### If no task name is provided (fresh start):

Start by understanding what the user wants to build:

> "What would you like to build? Describe the feature or functionality you have in mind."

Conduct the requirements interview (PART 1 below) FIRST. Once you understand:
- The problem being solved
- The scope and boundaries
- Key requirements and constraints

THEN create the task:

```bash
metagent task {taskname}
```

Choose a concise, descriptive task name based on what you learned (e.g., 'auth-system', 'api-caching', 'user-onboarding').

---

## CONTEXT LOADING

Study these files first (in order):
- @.agents/code/SPEC.md - **Project specification** (what this project is and does)
- @.agents/code/AGENTS.md - Project build commands and structure
- @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow

Understanding the project specification is **critical** - it tells you what this project is, its architecture, key abstractions, and boundaries. Use this context when interviewing the user and exploring the codebase.

---

## PART 1: REQUIREMENTS INTERVIEW

Conduct a structured interview. Ask **1-2 questions at a time** - don't overwhelm.

### 1.1 Goal Discovery

Ask:
- "What problem does this solve?"
- "What does success look like when this is complete?"
- "What are explicit NON-goals? (Things this should NOT do)"

**Document answers immediately** in working notes.

### 1.2 Scope Definition

Ask:
- "What are the boundaries of this task?"
- "What existing code/modules will this interact with?"
- "Are there external systems or APIs involved?"
- "What data flows in and out?"

### 1.3 Technical Constraints

Ask:
- "Any specific performance requirements? (latency, throughput, memory)"
- "Security requirements? (auth, encryption, validation)"
- "Compatibility requirements? (browsers, OS, versions)"
- "Are there existing patterns this must follow?"

### 1.4 User Stories / Use Cases

Ask:
- "Who or what uses this? (users, other services, cron jobs)"
- "Walk me through the primary workflow step by step"
- "What are the edge cases or error scenarios?"
- "What happens when things go wrong?"

### 1.5 Data & Types

Ask:
- "What are the main data structures involved?"
- "What fields do they have? Are any optional?"
- "What are the validation rules?"
- "Are there relationships between entities?"

### 1.6 Clarification Loop

After gathering initial requirements:
- Summarize your understanding back to the user
- Ask: "What did I miss or misunderstand?"
- Probe any ambiguous areas
- Continue until user confirms understanding is complete

---

## PART 2: CODEBASE EXPLORATION

Use subagents (up to 50 parallel) to research the existing codebase.

### 2.1 Structure Discovery

Spawn subagents to:
```
Analyze:
├── Directory structure (what goes where)
├── Module organization (how code is grouped)
├── Entry points (main files, exports)
├── Configuration (how things are configured)
└── Similar features (precedent for this work)
```

**Report to user:** "Here's how your codebase is organized..."

### 2.2 Integration Point Analysis

Spawn subagents to find:
```
Search for:
├── APIs this will call
├── APIs this will expose
├── Modules this will import
├── Modules that will import this
├── Shared types and interfaces
├── Database tables/collections involved
└── External services touched
```

**Report to user:** "This task will integrate with..."

### 2.3 Pattern Extraction

Spawn subagents to analyze:
```
Extract from existing code:
├── Error handling patterns (how errors are created, thrown, caught)
├── Logging patterns (what/when/how to log)
├── Testing patterns (test structure, mocking, fixtures)
├── Validation patterns (where/how input is validated)
├── Naming conventions (actual examples from code)
└── Documentation style (comment format, README structure)
```

**Report to user:** "Based on your existing code, I should follow these patterns..."

### 2.4 Existing Implementation Check

**CRITICAL:** Before specifying anything, search for existing implementations.

Spawn subagents to search:
```
Search terms (minimum 5 per concept):
├── Exact names user mentioned
├── Partial matches
├── Synonyms
├── Related concepts
└── File naming patterns

Search locations(UPDATE THIS):
├── src/
├── lib/
├── internal/
├── pkg/
├── test/ (reveals what's implemented)
└── examples/

```

**Report to user:** "I found existing code related to this task..."

If existing code found:
- Discuss with user: extend, replace, or work alongside?
- Note in specs what exists vs what's new

### 2.5 Dependency Mapping

Spawn subagents to map:
```
Understand impact:
├── What depends on code being changed?
├── What will new code depend on?
├── Circular dependency risks?
├── Breaking change potential?
└── Migration needs?
```

---

## PART 3: SPECIFICATION AUTHORING

Create detailed specs in `.agents/code/tasks/{taskname}/spec/`.

### 3.1 Required Specification Files

#### spec/overview.md

```markdown
# {Task Name} - Overview

## Purpose

{One paragraph: What this accomplishes and why it matters}

## Goals

1. {Specific, measurable goal}
2. {Specific, measurable goal}
3. {Specific, measurable goal}

## Non-Goals

> Explicitly out of scope for this task

1. {Thing this will NOT do}
2. {Thing this will NOT do}

## Architecture

{How this fits into the larger system}

## Dependencies

### Requires (must exist before this works)
- {Dependency}: {why needed}

### Provides (what this enables)
- {What other things can use this}

## Success Criteria

> How we know this is complete and correct

- [ ] {Testable criterion}
- [ ] {Testable criterion}
- [ ] {Testable criterion}

## Open Questions

> Resolved during implementation

- Q: {Unresolved question}
  - Context: {Why it matters}
```

#### spec/types.md (if applicable)

```markdown
# Types - {Task Name}

## Core Types

### {TypeName}

```{language}
{Complete type definition with all fields}
```

**Purpose:** {Why this type exists}

**Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| {field} | {type} | Yes/No | {description} |

**Invariants:**
- {What must always be true about this type}

**Example:**
```{language}
{Concrete example with real values}
```

## Type Relationships

```
{Diagram showing how types relate}
```
```

#### spec/{module}.md (one per logical module)

```markdown
# Module: {Name}

## Purpose

{What this module does and why it's separate}

## Dependencies

**Internal:**
- `{module}`: {what it uses from there}

**External:**
- `{package}`: {what it uses}

## Public Interface

### Functions

#### `{function_name}`

```{language}
{Complete function signature}
```

**Purpose:** {What it does}

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| {name} | {type} | {description} |

**Returns:** {type} - {description}

**Errors:**
| Error | When | Recovery |
|-------|------|----------|
| {error} | {condition} | {what caller should do} |

**Example:**
```{language}
// {Describe what this example shows}
{Working code example}
```

**Edge Cases:**
| Input | Behavior |
|-------|----------|
| Empty/null | {what happens} |
| Invalid | {what happens} |
| Maximum | {what happens} |

## Testing Requirements

- [ ] {Specific test case}
- [ ] {Specific test case}
```

#### spec/errors.md (if applicable)

```markdown
# Error Handling - {Task Name}

## Error Types

### {ErrorName}

**When:** {Condition that causes this error}
**Contains:** {Data included in error}
**Recovery:** {How callers should handle}

## Error Propagation

{How errors flow through the system}
```

### 3.2 Specification Quality Checklist

Before completing spec phase, verify:

- [ ] **Types are EXACT** - All fields listed, all constraints specified
- [ ] **Signatures are COMPLETE** - Every parameter, return type, error type
- [ ] **Examples are CONCRETE** - Real values, not placeholders
- [ ] **Edge cases are EXHAUSTIVE** - Empty, null, max, invalid
- [ ] **Errors are SPECIFIC** - When, what, how to handle
- [ ] **Dependencies are MAPPED** - What this needs, what needs this
- [ ] **Success criteria are TESTABLE** - Can write a test for each

---

## PART 4: COMPLETION

### 4.1 Summary to User

```
Specifications complete for: {taskname}

Files created:
  .agents/code/tasks/{taskname}/spec/
  ├── overview.md
  ├── types.md
  ├── {module}.md
  └── errors.md

Key decisions:
  1. {Decision}: {rationale}
```

### 4.2 Signal Completion

After specs are complete and validated, run:

```bash
metagent finish spec
```

This advances the task to the planning phase.

---

## RULES

1. **No implementation** - Specs only
2. **Interview first** - Understand before exploring
3. **Explore thoroughly** - Use subagents liberally
4. **Specify precisely** - Exact types, concrete examples
5. **Check for existing code** - Search before specifying

## PRIORITY

9. Research the code and understand how it works
99. Interview before exploring
999. Confirm understanding with user
9999. Complete specs - no placeholders

9999999. **SPECS ONLY - NO CODE**
