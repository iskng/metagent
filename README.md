# Metagent - Agent Workflow Distribution

Distributable agent workflows for autonomous coding. Install into any repository and keep prompts in sync.

## Quick Start

### Install to current directory

```bash
# Install default agent (code)
/path/to/metagent/install.sh

# Install specific agent
/path/to/metagent/install.sh -a code
```

### Install to specific repo

```bash
/path/to/metagent/install.sh ~/projects/my-app
```

### Configure for your project

```bash
cat .agents/code/BOOTSTRAP_PROMPT.md | claude-code
```

### Sync updates

```bash
# Sync to current directory
/path/to/metagent/sync.sh

# Sync to all tracked repos
/path/to/metagent/sync.sh --all

# Preview changes
/path/to/metagent/sync.sh --dry-run
```

## Directory Structure

```
metagent/
├── install.sh              # Install agent to repo
├── sync.sh                 # Sync updates to repos
├── README.md               # This file
└── code/                   # "code" agent (default)
    ├── prompts/            # Generic prompts (synced)
    │   ├── BOOTSTRAP_PROMPT.md
    │   ├── PLANNING_PROMPT.md
    │   ├── README.md
    │   ├── RECOVERY_PROMPT.md
    │   ├── REFRESH_PROMPT.md
    │   └── SPEC_PROMPT.md
    ├── scripts/            # Generic scripts (synced)
    │   └── spec.sh
    └── templates/          # Project-specific templates (not synced)
        ├── AGENTS.md
        └── TECHNICAL_STANDARDS.md
```

## Adding New Agents

Create a new directory with the same structure:

```
metagent/
└── my-agent/
    ├── prompts/        # Generic prompts
    ├── scripts/        # Generic scripts
    └── templates/      # Project-specific templates
```

Then install with:
```bash
./install.sh -a my-agent /path/to/repo
```

## Commands

### install.sh

```bash
./install.sh [options] [path]

Options:
  -a, --agent NAME    Agent to install (default: code)
  -l, --list          List available agents
  -h, --help          Show help
```

### sync.sh

```bash
./sync.sh [options] [path]

Options:
  -a, --agent NAME    Agent to sync (default: code)
  --all               Sync all tracked repos
  --list              List tracked repos
  --agents            List available agents
  --dry-run           Preview changes
  -h, --help          Show help
```

## File Categories

### Synced (generic)

Files in `prompts/` and `scripts/` are synced. These work across any project.

### Not Synced (project-specific)

Files in `templates/` are only copied on initial install if they don't exist:
- `AGENTS.md` - Build commands, project structure
- `TECHNICAL_STANDARDS.md` - Coding conventions

Task directories (`tasks/*`) are never touched.

## Installed Structure

After installing, your repo will have:

```
your-repo/
└── .agents/
    └── code/                     # Agent name
        ├── AGENTS.md             # Project-specific config
        ├── TECHNICAL_STANDARDS.md
        ├── BOOTSTRAP_PROMPT.md   # Generic prompts
        ├── PLANNING_PROMPT.md
        ├── README.md
        ├── RECOVERY_PROMPT.md
        ├── REFRESH_PROMPT.md
        ├── SPEC_PROMPT.md
        ├── .metagent-source      # Tracking marker
        ├── scripts/
        │   └── spec.sh
        └── tasks/                # Your task directories
```

## Workflow (code agent)

```
1. Spec Phase     → cat .agents/code/SPEC_PROMPT.md | claude-code
2. Planning Phase → cat .agents/code/PLANNING_PROMPT.md | claude-code
3. Build Loop     → while :; do cat .agents/code/tasks/{task}/PROMPT.md | claude-code; done
```

Recovery:
```
cat .agents/code/RECOVERY_PROMPT.md | claude-code
cat .agents/code/REFRESH_PROMPT.md | claude-code
```
