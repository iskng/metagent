# Metagent - Ralph Workflow Distribution

Distributable version of the Ralph autonomous coding workflow. Install into any repository and keep prompts in sync.

## Quick Start

### Install to a new repo

```bash
./install.sh /path/to/your/repo
```

This creates:
```
your-repo/
└── .agents/
    └── code/
        ├── AGENTS.md                 # Template - customize for your project
        ├── BOOTSTRAP_PROMPT.md       # Generic
        ├── PLANNING_PROMPT.md        # Generic
        ├── README.md                 # Generic
        ├── RECOVERY_PROMPT.md        # Generic
        ├── REFRESH_PROMPT.md         # Generic
        ├── SPEC_PROMPT.md            # Generic
        ├── TECHNICAL_STANDARDS.md    # Template - customize for your project
        ├── scripts/
        │   └── spec.sh               # Generic
        └── tasks/                    # Empty - your tasks go here
```

### Configure for your project

```bash
cd /path/to/your/repo
cat .agents/code/BOOTSTRAP_PROMPT.md | claude-code
```

Bootstrap will auto-detect your stack and configure `AGENTS.md` and `TECHNICAL_STANDARDS.md`.

### Sync updates

When you update prompts in metagent:

```bash
# Sync to specific repo
./sync.sh /path/to/your/repo

# Sync to all repos that were installed from metagent
./sync.sh --all

# Preview what would change
./sync.sh --dry-run /path/to/your/repo

# List tracked repos
./sync.sh --list
```

## File Categories

### Generic (synced)

These files work across any project and are kept in sync:

| File | Purpose |
|------|---------|
| `BOOTSTRAP_PROMPT.md` | Initial setup for new repos |
| `SPEC_PROMPT.md` | Phase 1: Specification development |
| `PLANNING_PROMPT.md` | Phase 2: Plan generation |
| `RECOVERY_PROMPT.md` | When things go wrong |
| `REFRESH_PROMPT.md` | Regenerate stale plans |
| `README.md` | Workflow documentation |
| `scripts/spec.sh` | Task directory bootstrapper |

### Project-Specific (not synced)

These files are customized per-project and NOT overwritten by sync:

| File | Purpose |
|------|---------|
| `AGENTS.md` | Build commands, project structure, learnings |
| `TECHNICAL_STANDARDS.md` | Coding conventions, patterns |
| `tasks/*` | All task directories (specs, plans, prompts) |

## Directory Structure

```
metagent/
├── README.md           # This file
├── install.sh          # Install to new repo
├── sync.sh             # Sync updates to existing repos
├── prompts/            # Generic prompt files
│   ├── BOOTSTRAP_PROMPT.md
│   ├── PLANNING_PROMPT.md
│   ├── README.md
│   ├── RECOVERY_PROMPT.md
│   ├── REFRESH_PROMPT.md
│   └── SPEC_PROMPT.md
├── scripts/            # Generic scripts
│   └── spec.sh
└── templates/          # Project-specific templates
    ├── AGENTS.md
    └── TECHNICAL_STANDARDS.md
```

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            PLANNING PHASE                                    │
│  ┌─────────────────────────┐         ┌─────────────────────────┐            │
│  │      SPEC PHASE         │         │      TODO PHASE          │            │
│  │    SPEC_PROMPT.md       │────────▶│   PLANNING_PROMPT.md     │            │
│  └─────────────────────────┘         └────────────┬─────────────┘            │
└───────────────────────────────────────────────────┼──────────────────────────┘
                                                    │
                                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          IMPLEMENTATION PHASE                                │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │   while :; do cat .agents/code/tasks/{task}/PROMPT.md | claude-code; done │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Usage in Target Repo

### Start a task

```bash
# Create task directory
.agents/code/scripts/spec.sh my-feature

# Phase 1: Specification
cat .agents/code/SPEC_PROMPT.md | claude-code
# Tell it: "my-feature"

# Phase 2: Planning
cat .agents/code/PLANNING_PROMPT.md | claude-code
# Tell it: "my-feature"

# Phase 3: Build Loop
while :; do cat .agents/code/tasks/my-feature/PROMPT.md | claude-code; done
```

### Recovery

```bash
# When things go wrong
cat .agents/code/RECOVERY_PROMPT.md | claude-code

# To refresh stale plan
cat .agents/code/REFRESH_PROMPT.md | claude-code
```

## Editing Prompts

1. Edit prompts in `metagent/prompts/`
2. Test changes
3. Run `./sync.sh --all` to push to all tracked repos

Project-specific changes should be made directly in the target repo's `.agents/code/AGENTS.md` and `.agents/code/TECHNICAL_STANDARDS.md`.
