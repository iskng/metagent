# Ralph Writing System

A structured, autonomous writing workflow for books, courses, and long-form content.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SETUP PHASE                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         INIT PHASE                                   │    │
│  │                      /writer-init                                    │    │
│  │                                                                      │    │
│  │  • Interview user about project                                      │    │
│  │  • Create outline, style guide, terminology                          │    │
│  │  • Generate editorial_plan.md                                        │    │
│  └──────────────────────────────────┬──────────────────────────────────┘    │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           WRITING PHASE (cycles)                             │
│                                                                              │
│  ┌────────────────────┐         ┌────────────────────┐                      │
│  │    PLAN STAGE      │         │    WRITE STAGE     │                      │
│  │   /writer-plan     │────────▶│     /writer        │◀───┐                 │
│  │                    │         │                    │    │                 │
│  │ • Research section │         │ • Write ONE page   │    │ more pages      │
│  │ • Generate page    │         │ • Editorial checks │    │                 │
│  │   breakdown        │         │ • Update plan      │────┘                 │
│  └────────────────────┘         └─────────┬──────────┘                      │
│           ▲                               │                                  │
│           │         section complete      │                                  │
│           └───────────────────────────────┘                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### 1. Install metagent (first time only)

```bash
# From metagent repo
./metagent.sh install
```

### 2. Initialize writer in your project

```bash
cd ~/my-writing-project
metagent --agent writer init
```

### 3. Start a new writing project

```bash
# Interactive mode
metagent --agent writer start
# Conducts interview → creates task → generates structure

# Or manually with /writer-init slash command in Claude
```

### 4. Run the writing loop

```bash
metagent --agent writer run my-book
# Loops through: plan → write → write → ... → plan → write → ...
```

---

## Workflow Stages

The writer agent cycles between two stages for each section:

| Stage | Prompt | What Happens |
|-------|--------|--------------|
| `init` | INIT_PROMPT.md | Interview, create project structure |
| `plan` | PLANNING_PROMPT.md | Research section, generate page breakdown |
| `write` | PROMPT.md | Write one page (~1000 words) |
| `completed` | - | All sections done |

### Stage Transitions

```
init ──────────────────────▶ plan
                              │
                              ▼
                ┌──────────▶ write ◀──────────┐
                │             │                │
                │             │ more pages     │
                │             └────────────────┘
                │             │
                │             │ section done
                └─────────────┘
                              │
                              │ all sections done
                              ▼
                          completed
```

**Completion signals:**
- `metagent finish write --next write` - more pages in current section
- `metagent finish write --next plan` - section done, plan next one
- `metagent finish write` - all sections complete

---

## Directory Structure

### Global (after `metagent install`)

```
~/.metagent/writer/
├── INIT_PROMPT.md         # Project setup interview
├── PLANNING_PROMPT.md     # Section planning
└── PROMPT.md              # Writing loop
```

### Per-project (after `metagent --agent writer init`)

```
your-project/
└── .agents/writer/
    ├── AGENTS.md          # Project config, learnings
    ├── queue.jsonl        # Task queue
    └── tasks/
        └── {taskname}/
            ├── editorial_plan.md    # Current task list
            ├── content/             # Written content
            │   └── section-NN/
            │       ├── page-01.md
            │       └── ...
            ├── outline/
            │   ├── overview.md      # Goals, audience
            │   └── structure.md     # Full outline
            ├── style/
            │   ├── voice.md         # Voice, tone
            │   └── terminology.md   # Glossary
            └── research/
                └── notes.md         # Research findings
```

---

## The Prompts

### Core Workflow

| Prompt | Stage | Purpose |
|--------|-------|---------|
| `INIT_PROMPT.md` | init | Interview user, create project structure |
| `PLANNING_PROMPT.md` | plan | Research section, generate page breakdown |
| `PROMPT.md` | write | Write one page, run editorial checks |

### Slash Commands

| Command | Maps to |
|---------|---------|
| `/writer-init` | INIT_PROMPT.md |
| `/writer-plan` | PLANNING_PROMPT.md |
| `/writer` | PROMPT.md |

---

## Key Concepts

### One Page Per Loop

Each writing loop produces exactly ONE page (~1000 words):
1. Read editorial_plan.md to find current page
2. Research the topic online (required)
3. Search existing content for consistency
4. Write the page
5. Run editorial checks
6. Update editorial_plan.md
7. Signal completion with appropriate `--next` flag
8. STOP

### Research Before Writing

**CRITICAL:** Every page requires online research before writing:
- Current facts and data
- Technical accuracy verification
- How experts discuss the topic

Research is done via subagents to keep primary context clean.

### Editorial Checks

After writing each page, run checks:
1. **Accuracy** - Facts sourced and verified?
2. **Style** - Voice matches style guide?
3. **Consistency** - No contradictions with existing content?
4. **Quality** - Grammar, clarity, completeness?

### Self-Improvement

Learnings persist across loops:
- Research techniques → AGENTS.md
- Effective patterns → AGENTS.md
- Style clarifications → style/voice.md
- New terminology → style/terminology.md

---

## Commands Reference

```bash
# Global setup (first time)
metagent install

# Initialize writer in a project
metagent --agent writer init

# Create a new task
metagent --agent writer task my-book

# Show task queue
metagent --agent writer queue

# Run the writing loop
metagent --agent writer run my-book

# Signal stage completion
metagent finish write --next write   # more pages
metagent finish write --next plan    # next section
metagent finish write                # all done
```

---

## Source of Truth

| File | Controls |
|------|----------|
| `outline/structure.md` | What sections/topics to cover |
| `editorial_plan.md` | Current page breakdown, progress |
| `style/voice.md` | How to write (tone, person, formality) |
| `style/terminology.md` | Consistent terms and definitions |
| `AGENTS.md` | Project-wide learnings and config |

---

## Common Issues

### Voice Inconsistency

**Cause:** Not reading previous content before writing.

**Fix:**
- Always search existing content before writing
- Re-read style/voice.md each loop
- Add examples to AGENTS.md

### Factual Errors

**Cause:** Skipping research phase.

**Fix:**
- Research is REQUIRED, not optional
- Verify facts against multiple sources
- Add corrections to research/notes.md

### Going Off-Topic

**Cause:** Not following outline structure.

**Fix:**
- outline/structure.md is the spec
- Check page matches planned scope
- Add "stay on topic" reminders to AGENTS.md

### Duplicate Content

**Cause:** Not searching existing content.

**Fix:**
- Search before writing
- Check editorial_plan.md for what's already written
- Cross-reference between sections

---

## Tips

1. **Good outline first** - The outline is your spec; spend time on it
2. **Research every page** - Current information makes better content
3. **One page at a time** - Full focus on single piece
4. **Update AGENTS.md** - Learnings persist across sessions
5. **Trust the loop** - Multiple passes improve quality
6. **Check editorial_plan.md** - Single source of truth for progress

---

## Files Included

| File | Purpose |
|------|---------|
| INIT_PROMPT.md | Project setup interview |
| PLANNING_PROMPT.md | Section planning and research |
| PROMPT.md | Writing loop (one page per loop) |
| README.md | This documentation |

### Templates (copied to project)

| File | Purpose |
|------|---------|
| AGENTS.md | Project config, learnings |
