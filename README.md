# Metagent - Agent Workflow Manager

Centralized agent workflows for autonomous coding and writing.

## Quick Start

```bash
# First time setup
./metagent.sh link

# Install code agent to a repo
cd ~/projects/my-app
metagent install
/bootstrap

# Or install writer agent
cd ~/projects/my-book
metagent --agent writer install
/writer-init
```

## Agent Types

| Agent | Purpose | Workflow |
|-------|---------|----------|
| `code` (default) | Software development | spec → planning → build loop |
| `writer` | Writing projects | init → planning → writing loop |

## Commands

```bash
metagent [--agent TYPE] <command> [args]

metagent link              # Setup globally (first time)
metagent install [path]    # Install agent to repo
metagent start             # Start new task interactively
metagent task <name>       # Create task (used by model)
metagent finish <stage>    # Signal stage completion
metagent run <name>        # Run loop
metagent queue             # Show task queue
metagent dequeue <name>    # Remove from queue
metagent run-queue         # Process all queued tasks
metagent unlink            # Remove metagent
```

## What Gets Installed

### Global (`metagent link`)

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

### Per-repo: Code Agent (`metagent install`)

```
your-repo/.agents/code/
├── SPEC.md                        # Project specification
├── AGENTS.md                      # Build commands
├── TECHNICAL_STANDARDS.md         # Coding patterns
├── queue.jsonl                    # Task queue
└── tasks/{taskname}/
    ├── spec/                      # Specifications
    ├── plan.md                    # Implementation plan
    └── PROMPT.md                  # Build loop prompt
```

### Per-repo: Writer Agent (`metagent --agent writer install`)

```
your-repo/.agents/writer/
├── WRITER.md                      # Writing config
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
1. /bootstrap              # Configure for your project (once)
2. metagent start          # Interview → spec → planning
3. metagent run <task>     # Execute the plan
4. /debug                  # When bugs are found
```

### Writer Agent

```
1. metagent --agent writer install
2. /writer-init            # Interview → project setup
3. /writer-plan            # Plan section (research + page breakdown)
4. /writer                 # Write one page per loop
```

## Architecture

Prompts are centralized in `~/.metagent/`:
- Update once, all repos use the latest
- No sync needed
- Repos only contain project-specific files

Per-task files (PROMPT.md, editorial_plan.md) live in the repo since they contain task-specific content.
