# Ralph Workflow System

A structured, 3-phase autonomous coding workflow based on the Ralph Wiggum technique.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            PLANNING PHASE                                    │
│  ┌─────────────────────────┐         ┌─────────────────────────┐            │
│  │      SPEC PHASE         │         │      TODO PHASE          │            │
│  │    SPEC_PROMPT.md       │────────▶│   PLANNING_PROMPT.md     │            │
│  │                         │         │                          │            │
│  │  • Interview user       │         │  • Read specs            │            │
│  │  • Explore codebase     │         │  • Research codebase     │            │
│  │  • Create spec/*.md     │         │  • Generate plan.md      │            │
│  └─────────────────────────┘         └────────────┬─────────────┘            │
└───────────────────────────────────────────────────┼──────────────────────────┘
                                                    │
                                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          IMPLEMENTATION PHASE                                │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                       INCREMENTAL LOOP                                 │  │
│  │                   {task}/PROMPT.md (per task)                          │  │
│  │                                                                        │  │
│  │   metagent run {task}                                                │  │
│  │                                                                        │  │
│  │   Each loop:                                                           │  │
│  │   1. Load specs & plan                                                 │  │
│  │   2. Pick most important task                                          │  │
│  │   3. Research (subagents)                                              │  │
│  │   4. Implement                                                         │  │
│  │   5. Test (single subagent)                                            │  │
│  │   6. Commit & update plan                                              │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐     │
│  │ recovery_prompt.md │  │ refresh_prompt.md  │  │   BOOTSTRAP_PROMPT │     │
│  │ When things break  │  │ Regenerate plan    │  │   Initial setup    │     │
│  └────────────────────┘  └────────────────────┘  └────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### 1. Install metagent (first time only)

```bash
# From metagent repo
./metagent.sh install
```

### 2. Initialize code agent in your project

```bash
cd ~/my-project
metagent init
# Or: metagent --agent code init
```

### 3. Bootstrap (configure for your project)

```bash
# Run /bootstrap slash command in Claude, or:
metagent start
# It will detect your language/framework and configure AGENTS.md
```

### 4. Start a Task

```bash
# Interactive mode (recommended)
metagent start
# Conducts interview → creates task → specs → planning

# Or manually:
metagent task my-feature       # Create task
# Use /spec slash command      # Write specs
metagent finish spec --session "<session>"           # Advance to planning
# Use /planner slash command   # Create plan
metagent finish planning --session "<session>"       # Advance to ready

# Build Loop
metagent run my-feature
# Monitor output, Ctrl+C to intervene
```

### 5. Recovery

```bash
# When things go wrong, use /debug slash command
```

---

## Directory Structure

### Global (after `metagent install`)

```
~/.metagent/code/
├── BOOTSTRAP_PROMPT.md      # Initial repo setup (/bootstrap)
├── SPEC_PROMPT.md           # Phase 1: Specifications (/spec)
├── PLANNING_PROMPT.md       # Phase 2: Planning (/planner)
├── DEBUG_PROMPT.md          # Bug diagnosis (/debug)
├── RECOVERY_PROMPT.md       # When things break
└── REFRESH_PROMPT.md        # Regenerate stale plans
```

### Per-project (after `metagent init`)

```
project/
├── .agents/code/
│   ├── AGENTS.md                # Build commands & learnings
│   ├── SPEC.md                  # Project specification
│   ├── TECHNICAL_STANDARDS.md   # Coding patterns
│   ├── queue.jsonl              # Task queue
│   └── tasks/                   # All task directories
│       └── {taskname}/          # Per-task directory
│           ├── spec/            # Specifications
│           │   ├── overview.md
│           │   ├── types.md
│           │   └── errors.md
│           ├── plan.md          # Prioritized task list
│           └── PROMPT.md        # Build loop prompt
└── src/                         # Your source code
```

---

## The Prompts

### Core Workflow

| Prompt | Phase | Purpose |
|--------|-------|---------|
| `SPEC_PROMPT.md` | 1 - Spec | Interview user, explore codebase, create specifications |
| `SPEC_EXISTING_TASK_PROMPT.md` | 1 - Spec | Spec for existing tasks (no task creation) |
| `PLANNING_PROMPT.md` | 2 - Plan | Read specs, research codebase, generate plan.md |
| `{task}/PROMPT.md` | 3 - Build | Execute plan incrementally, test, commit |

### Support Prompts

| Prompt | Slash Command | Purpose |
|--------|---------------|---------|
| `BOOTSTRAP_PROMPT.md` | `/bootstrap` | Initial setup for new repositories |
| `DEBUG_PROMPT.md` | `/debug` | Diagnose and fix bugs |
| `SUBMIT_ISSUE_PROMPT.md` | `/submit-issue` | Log a known issue (capture only) |
| `SUBMIT_TASK_PROMPT.md` | `/submit-task` | Capture a task (optionally held) |
| `SUBMIT_HOLD_TASK_PROMPT.md` | `/submit-hold-task` | Create a held task with partial spec from confirmed info |
| `RECOVERY_PROMPT.md` | - | Recover from broken states |
| `REFRESH_PROMPT.md` | - | Regenerate stale or cluttered plans |

### Configuration Files

| File | Purpose |
|------|---------|
| `AGENTS.md` | Build/test commands, project structure, learnings |
| `TECHNICAL_STANDARDS.md` | Coding conventions, patterns, anti-patterns |

---

## Key Concepts

### Subagent Architecture

```
PRIMARY CONTEXT (The Scheduler)
│
├── Loads context (specs, plan, standards)
├── Decides what to do
├── Delegates heavy work:
│   │
│   ├── Search subagents (up to 100 parallel)
│   │   └── Codebase search, file discovery
│   │
│   ├── Read subagents (up to 100 parallel)
│   │   └── File reading, spec analysis
│   │
│   ├── Write subagents (up to 50 parallel)
│   │   └── Code generation, doc updates
│   │
│   └── Build/Test subagent (1 ONLY)
│       └── Validation, prevents race conditions
│
└── Evaluates results, updates state
```

**Critical Rule:** Only 1 subagent for build/test to prevent "back pressure chaos."

### Context Window Management

- ~170k effective context (quality degrades beyond this)
- Load specs fresh each loop ("burn tokens")
- Use subagents to keep primary context clean
- Summarize results, don't store full output

### The "Signs" System

Priority numbers indicate importance:
```
9.     Normal importance
99.    Higher importance
999.   Very important
9999.  Critical
99999. NEVER ignore
```

Add "signs" when Ralph makes repeated mistakes.

### Back Pressure

Fast feedback catches errors quickly:
1. Format check (< 1 second)
2. Lint (1-5 seconds)
3. Type check (5-30 seconds)
4. Build (10-120 seconds)
5. Unit tests (5-30 seconds)
6. Full suite (2-10 minutes)

### Tagging Strategy

- Tag after EVERY stable state
- Start at 0.0.1, increment patch
- Tags are recovery checkpoints
- `git reset --hard {tag}` to recover

---

## Workflow Details

### Phase 1: Specification

**Goal:** Create detailed, unambiguous specifications.

**Process:**
1. Interview user (1-2 questions at a time)
2. Explore existing codebase with subagents
3. Write spec files with exact types, complete signatures
4. Validate with user

**Outputs:**
- `spec/overview.md` - Goals, architecture, constraints
- `spec/types.md` - Type definitions
- `spec/{module}.md` - Per-module specifications
- `spec/errors.md` - Error handling

### Phase 2: Planning

**Goal:** Transform specs into actionable, prioritized plan.

**Process:**
1. Parse all spec files
2. Research codebase for existing implementations
3. Map dependencies between tasks
4. Generate prioritized plan.md

**Outputs:**
- `plan.md` with Critical/High/Medium/Low sections
- Dependency information for each task
- Complexity estimates (S/M/L)

### Phase 3: Build Loop

**Goal:** Execute plan one task at a time.

**Each Loop:**
1. Load context (specs, plan, standards)
2. Choose most important incomplete task
3. Research before implementing (prevent duplicates)
4. Implement fully (no placeholders)
5. Test (unit → related → full)
6. If pass: commit, update plan, tag if stable
7. If fail: fix or document

**Self-Improvement:**
- Build learnings → AGENTS.md
- Bugs → plan.md
- Spec issues → update specs

---

## Common Issues

### Ralph Creates Duplicates

**Cause:** Search didn't find existing code.

**Fix:**
- Strengthen search in prompt (more terms)
- Add specific search instructions
- Check test files (reveal implementations)

### Tests Pass But Code Wrong

**Cause:** Tests don't match specs.

**Fix:**
- Add type checking to validation chain
- Strengthen specs with examples
- Review test coverage against spec requirements

### Ralph Goes in Circles

**Cause:** Conflicting requirements or unclear specs.

**Fix:**
- Break down complex tasks
- Clarify specs
- Use refresh_prompt to regenerate plan

### Context Window Overflow

**Cause:** Too many errors filling context.

**Fix:**
- Export errors to file
- Use different AI to analyze and prioritize
- Reset to last stable tag

---

## Philosophy

### Why "Ralph"?

Named after Ralph Wiggum - "deterministically bad in an undeterministic world."

Ralph WILL make mistakes, but they're:
- Predictable
- Identifiable
- Correctable through iteration

### Core Insight

> "Any problem created by AI can be resolved through a different series of prompts."

### Key Principles

1. **Fresh context each loop** - No accumulated confusion
2. **One task per loop** - Full reasoning on single item
3. **Subagent delegation** - Keep primary context clean
4. **Strong back pressure** - Tests catch errors fast
5. **Self-improvement** - Learnings persist across loops
6. **Eventual consistency** - Enough loops converge on correct

### Monolith Philosophy

Ralph is deliberately monolithic:
- Single process
- Single repository
- Single task per loop
- Sequential at top level
- Parallelism only via subagents

Why: Coordinating non-deterministic agents creates exponential complexity.

---

## Tips

1. **Customize AGENTS.md first** - Real commands are essential
2. **Start with good specs** - Garbage in, garbage out
3. **Watch the loops** - Stream output to catch issues
4. **Trust the process** - Ralph will err then correct
5. **Tag often** - More recovery points
6. **Refresh when stuck** - Fresh plan helps
7. **Senior expertise required** - Guide Ralph, don't replace thinking

---

## Files Included

### Prompts (in ~/.metagent/code/)

| File | Slash Command | Purpose |
|------|---------------|---------|
| BOOTSTRAP_PROMPT.md | `/bootstrap` | Auto-setup for any repo |
| SPEC_PROMPT.md | `/spec` | Specification development |
| SPEC_EXISTING_TASK_PROMPT.md | - | Spec for existing tasks (no task creation) |
| PLANNING_PROMPT.md | `/planner` | Plan generation |
| DEBUG_PROMPT.md | `/debug` | Bug diagnosis |
| SUBMIT_ISSUE_PROMPT.md | `/submit-issue` | Issue capture |
| SUBMIT_TASK_PROMPT.md | `/submit-task` | Task capture |
| SUBMIT_HOLD_TASK_PROMPT.md | `/submit-hold-task` | Held task capture |
| RECOVERY_PROMPT.md | - | Failure recovery |
| REFRESH_PROMPT.md | - | Plan regeneration |
| README.md | - | This documentation |

### Templates (copied to project)

| File | Purpose |
|------|---------|
| AGENTS.md | Build commands & learnings |
| SPEC.md | Project specification |
| TECHNICAL_STANDARDS.md | Coding standards |

---

## Credits

Based on the Ralph Wiggum technique documented by its creator. This implementation structures the methodology into reusable prompts for consistent application.
