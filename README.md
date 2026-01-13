# Metagent - Agent Workflow Distribution

Distributable agent workflows for autonomous coding. Install into any repository and keep prompts in sync.

## Quick Start

```bash
# First time setup
./metagent.sh link

# Install to a repo
cd ~/projects/my-app
metagent install

# Run bootstrap to configure
/bootstrap
```

## Commands

```bash
metagent link                # Setup globally (first time)
metagent install [path]      # Install agent to repo
metagent sync [path]         # Sync prompts to repo
metagent unlink              # Remove metagent
```

### Options

```bash
metagent install -a code           # Specify agent (default: code)
metagent sync --all                # Sync all tracked repos
metagent sync --dry-run            # Preview changes
metagent --list                    # List available agents
metagent --tracked                 # List tracked repos
```

## What Gets Installed

### Global (`metagent link`)

```
~/.local/bin/metagent              # CLI tool
~/.metagent/                       # Global prompts
~/.claude/commands/                # Slash commands
  ├── bootstrap.md -> ~/.metagent/BOOTSTRAP_PROMPT.md
  ├── spec.md -> ~/.metagent/SPEC_PROMPT.md
  └── plan.md -> ~/.metagent/PLANNING_PROMPT.md
```

### Per-repo (`metagent install`)

```
your-repo/.agents/code/
├── SPEC.md                        # Project specification (generated)
├── AGENTS.md                      # Build commands (configured)
├── TECHNICAL_STANDARDS.md         # Coding patterns (configured)
├── BOOTSTRAP_PROMPT.md
├── SPEC_PROMPT.md
├── PLANNING_PROMPT.md
├── README.md
├── RECOVERY_PROMPT.md
├── REFRESH_PROMPT.md
├── scripts/
│   └── spec.sh
└── tasks/                         # Your task directories
```

## Workflow

```
1. /bootstrap              # Configure for your project (once)
2. /spec my-feature        # Specify what to build
3. /plan my-feature        # Plan the implementation
4. Build loop              # Execute the plan
```

Build loop:
```bash
while :; do cat .agents/code/tasks/my-feature/PROMPT.md | claude-code; done
```

## Syncing Updates

When you update prompts in metagent:

```bash
metagent sync --all      # Update all tracked repos
```

Only generic files (prompts/, scripts/) are synced. Project-specific files (SPEC.md, AGENTS.md, TECHNICAL_STANDARDS.md, tasks/) are never overwritten.

## Adding New Agents

Create a directory with this structure:

```
metagent/
└── my-agent/
    ├── prompts/        # Generic prompts (synced)
    ├── scripts/        # Generic scripts (synced)
    └── templates/      # Project-specific templates (install only)
```

Then: `metagent install -a my-agent`
