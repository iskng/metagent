# Metagent - Agent Workflow Manager

Centralized agent workflows for autonomous coding and writing.

## Quick Start

```bash
# First time setup
./metagent.sh install

# Initialize code agent in a repo
cd ~/projects/my-app
metagent init
/bootstrap

# Or initialize writer agent
cd ~/projects/my-book
metagent --agent writer init
/writer-init
```

## Agent Types

| Agent | Purpose | Workflow |
|-------|---------|----------|
| `code` (default) | Software development | spec → planning → build loop |
| `writer` | Writing projects | init → plan → write loop |

## Commands

```bash
metagent [--agent TYPE] <command> [args]

metagent install           # Setup globally (first time)
metagent uninstall         # Remove metagent globally
metagent init [path]       # Initialize agent in repo
metagent start             # Start new task interactively
metagent task <name>       # Create task (used by model)
metagent finish <stage>    # Signal stage completion
metagent run <name>        # Run loop for a task
metagent queue             # Show task queue
metagent dequeue <name>    # Remove from queue
metagent run-queue         # Process all queued tasks
```

## What Gets Installed

### Global (`metagent install`)

```
~/.local/bin/metagent              # CLI tool
~/.metagent/
  ├── code/                        # Code agent prompts
  │   ├── BOOTSTRAP_PROMPT.md
  │   ├── SPEC_PROMPT.md
  │   ├── PLANNING_PROMPT.md
  │   └── DEBUG_PROMPT.md
  └── writer/                      # Writer agent prompts
      ├── INIT_PROMPT.md
      ├── PLANNING_PROMPT.md
      └── PROMPT.md
~/.claude/commands/                # Slash commands
  ├── bootstrap.md                 # /bootstrap (code)
  ├── spec.md                      # /spec (code)
  ├── planner.md                   # /planner (code)
  ├── debug.md                     # /debug (code)
  ├── writer-init.md               # /writer-init
  ├── writer-plan.md               # /writer-plan
  └── writer.md                    # /writer
```

### Per-repo: Code Agent (`metagent init`)

```
your-repo/.agents/code/
├── SPEC.md                        # Project specification
├── AGENTS.md                      # Build commands & learnings
├── TECHNICAL_STANDARDS.md         # Coding patterns
├── queue.jsonl                    # Task queue
└── tasks/{taskname}/
    ├── spec/                      # Specifications
    ├── plan.md                    # Implementation plan
    └── PROMPT.md                  # Build loop prompt
```

### Per-repo: Writer Agent (`metagent --agent writer init`)

```
your-repo/.agents/writer/
├── AGENTS.md                      # Writing config & learnings
├── queue.jsonl                    # Task queue
└── tasks/{projectname}/
    ├── content/                   # Written content
    ├── outline/                   # Structure
    ├── style/                     # Voice, terminology
    ├── research/                  # Research notes
    └── editorial_plan.md          # Task list
```

## Workflows

### Code Agent

```
1. metagent init           # Initialize in your repo
2. /bootstrap              # Configure for your project
3. metagent start          # Interview → spec → planning
4. metagent run <task>     # Execute the plan
5. /debug                  # When bugs are found
```

### Writer Agent

```
1. metagent --agent writer init    # Initialize in your repo
2. metagent --agent writer start   # Interview → plan/write loop (keeps going)
3. metagent --agent writer run <task>  # Resume/continue loops as needed
```

### Stage Transitions

**Code agent:** `spec → planning → ready → completed`

**Writer agent:** `init → plan ⟷ write → completed`

The writer agent cycles between `plan` and `write` stages:
- `metagent finish write --next write` - more pages in section
- `metagent finish write --next plan` - section done, plan next
- `metagent finish write` - all sections complete

## Architecture

Prompts are centralized in `~/.metagent/`:
- Update once, all repos use the latest
- No sync needed
- Repos only contain project-specific files

Per-task files (PROMPT.md, editorial_plan.md) live in the repo since they contain task-specific content.

## Agent Plugin System

Each agent is defined in its own directory with:
- `agent.sh` - Agent-specific functions (stages, prompts, task creation)
- `prompts/` - Prompt files
- `templates/` - Files copied to project on init

To add a new agent type, create a new directory with these files.
