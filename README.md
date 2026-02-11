# metagent

`metagent` is a Rust CLI for running structured multi-stage agent workflows in a repo.

It manages:
- agent stages (`code` and `writer`)
- task state (`.agents/<agent>/tasks/.../task.json`)
- session lifecycle (`.agents/<agent>/sessions/.../session.json`)
- queue/backlog behavior
- issue tracking for the `code` agent

## Requirements

- Rust toolchain (to build from source)
- `codex` CLI and/or `claude` CLI installed
- Git repo for normal usage (`metagent init` warns if no `.git`)

Important defaults:
- Default agent: `code`
- Running `metagent` with no command is the same as `metagent start`
- `code` workflow stages default to `codex` unless you explicitly override with model flags/env

## Install

### Build from source

```bash
cargo build --release
```

Binary path:
- `target/release/metagent`

### Install globally

```bash
cargo run -- install
# or: ./target/release/metagent install
```

This installs:
- `~/.local/bin/metagent`
- prompts under `~/.metagent/code/` and `~/.metagent/writer/`
- slash-command symlinks under:
  - `~/.claude/commands/`
  - `~/.codex/prompts/`

If `~/.local/bin` is not in `PATH`, add:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Uninstall

```bash
metagent uninstall
```

This removes:
- `~/.local/bin/metagent`
- linked metagent prompt files in `~/.claude/commands` and `~/.codex/prompts`
- `~/.metagent/`

## Quick Start

### Code agent (default)

```bash
cd /path/to/repo
metagent init
metagent start
```

Typical flow:
1. `start` runs `spec -> planning`
2. for `code`, `start` stops at `build` handoff
3. continue with `metagent run <task>` or `metagent run-queue`

### Writer agent

```bash
cd /path/to/repo
metagent --agent writer init
metagent --agent writer start
```

Writer stages are: `init -> plan -> write -> edit -> completed`.

## Agent Workflows

### Code stages

- `spec`
- `spec-review`
- `spec-review-issues`
- `planning`
- `build`
- `review`
- `completed`

Default next-stage mapping:
- `spec -> planning`
- `spec-review -> planning`
- `spec-review-issues -> planning`
- `planning -> build`
- `build -> review`
- `review -> completed`

Queue-processing stages (`run-queue` / `run-next`):
- `spec-review-issues`, `build`, `review`

### Coding flow diagram (`code` agent, single-Codex style)

```mermaid
flowchart TD
    A[metagent init] --> B[metagent task <task>]
    B --> C[Run one Codex session for spec work<br/>metagent run <task>]
    C --> D[Research codebase + write spec]
    D --> E[Submit spec<br/>metagent finish spec --session <id> --task <task>]

    E --> F{Improve spec before build?}
    F -->|manual spec review| G[metagent spec-review <task>]
    F -->|targeted research| H[metagent research <task> <focus>]

    G --> I[Refine spec + submit review stage<br/>metagent finish spec-review --session <id> --task <task>]
    H --> J[Update spec files]
    J --> K[Submit spec stage again if needed<br/>metagent finish spec --session <id> --task <task>]
    I --> L[planning]
    K --> L
    F -->|no| L

    L --> M[Submit planning<br/>metagent finish planning --session <id> --task <task>]
    M --> N[Scale execution with queue<br/>metagent run-queue]
    N --> O[build -> review loop]
    O --> P{review pass?}
    P -->|no| O
    P -->|yes| Q[completed]
```

Operational notes:
- this style keeps one primary Codex instance active for spec/planning work.
- “submit spec” in practice is `metagent finish spec ...` (or `finish spec-review ...` after manual review).
- `metagent research` helps improve spec quality before moving into queue execution.
- `run-queue` then drives iterative `build -> review` until completion.

### Writer stages

- `init`
- `plan`
- `write`
- `edit`
- `completed`

Default next-stage mapping:
- `init -> plan`
- `plan -> write`
- `write -> edit`
- `edit -> completed`

Queue-processing stages:
- `write`, `edit`

## Core Commands

Global usage:

```bash
metagent [--agent <code|writer>] [--model <claude|codex>] [--force-model] <command>
```

### Setup and lifecycle

- `metagent install`
- `metagent uninstall`
- `metagent init [path]`
- `metagent start`

### Task and queue management

- `metagent task <name> [--hold] [--description <text>]`
- `metagent hold <name>`
- `metagent activate <name>`
- `metagent queue [task]` (alias: `q`)
- `metagent plan <task>` (show parsed plan/checklist steps)
- `metagent delete <name> [--force]` (alias: `dequeue`)
- `metagent reorder <name> <position>` (build-stage only)
- `metagent set-stage <name> <stage> [--status <status>]`

### Execution

- `metagent run <name>`
- `metagent run-next [name]` (alias: `rn`)
- `metagent run-queue [--loop <n>]` (alias: `rq`)
- `metagent finish [stage] [--next <stage>] [--session <id>] [--task <task>]`

### Review, research, debug

- `metagent review <task> [focus]`
- `metagent spec-review <task>`
- `metagent research <task> [focus]` (`code` agent only)
- `metagent debug [--file <path> | --stdin | <bug...>]` (uses `codex`)
- `metagent how [topic]`

### Issues (`code` agent only)

- `metagent issues [--task <task> | --unassigned] [--status <open|resolved|all>] [--priority <P0..P3>] [--type <spec|build|bug|test|perf|other>] [--source <review|debug|submit|manual>]`
- `metagent issue list ...` (same filters)
- `metagent issue add --title <title> [--task <task>] [--priority ...] [--type ...] [--source ...] [--file <path>] [--stage <stage>] [--body <text> | --stdin-body]`
- `metagent issue resolve <id> [--resolution <text>]`
- `metagent issue assign <id> --task <task> [--stage <stage>]`
- `metagent issue show <id>`

## How to Use

This section is the practical operator workflow.

### 1) One-time machine setup

```bash
# from this repo
cargo build --release
./target/release/metagent install
```

Check install:

```bash
metagent --version
metagent --help
```

### 2) One-time repo setup

Code repo:

```bash
cd /path/to/your/repo
metagent init
```

Writer repo:

```bash
cd /path/to/your/writing/repo
metagent --agent writer init
```

### 3) Create and inspect tasks

Create a normal task:

```bash
metagent task add-login-rate-limit --description "Protect login endpoint from abuse"
```

Create backlog/held task:

```bash
metagent task migrate-settings-schema --hold --description "Schema migration after Q2 launch"
```

Inspect all tasks:

```bash
metagent queue
```

Inspect one task (if it exists as task dir but not tracked yet, this also tracks it):

```bash
metagent queue add-login-rate-limit
```

### 4) Run a single code task end-to-end

Use this when focusing on one task:

```bash
metagent run add-login-rate-limit
```

What happens:
1. task is claimed (lock created)
2. current stage prompt is rendered and model CLI is started
3. stage advances only when `metagent finish ...` is called
4. loop continues until task reaches `completed`, interruption, or failure

Manual `finish` examples for code:

```bash
# default stage progression
metagent finish spec --session <session-id> --task add-login-rate-limit
metagent finish planning --session <session-id> --task add-login-rate-limit
metagent finish build --session <session-id> --task add-login-rate-limit

# review stage: explicit branch
metagent finish review --session <session-id> --task add-login-rate-limit --next build
metagent finish review --session <session-id> --task add-login-rate-limit --next spec-review-issues

# review pass
metagent finish review --session <session-id> --task add-login-rate-limit
```

### 5) Run many tasks from queue

Run exactly one eligible task-stage and return:

```bash
metagent run-next
```

Run a specific task for one stage:

```bash
metagent run-next add-login-rate-limit
```

Run queue continuously:

```bash
metagent run-queue
```

Tune review/build bounce protection:

```bash
metagent run-queue --loop 8
```

Queue control:

```bash
metagent hold migrate-settings-schema
metagent activate migrate-settings-schema
metagent reorder add-login-rate-limit 1
metagent delete old-experiment --force
```

### 6) Use issue workflow (code agent)

Create issue:

```bash
metagent issue add \
  --title "Race condition in token refresh" \
  --task add-login-rate-limit \
  --type bug \
  --priority P1 \
  --source manual \
  --body "Observed concurrent refresh requests creating duplicate writes."
```

List open issues:

```bash
metagent issues --task add-login-rate-limit
```

Resolve issue:

```bash
metagent issue resolve <issue-id> --resolution "Added per-user lock and idempotency key."
```

Assign unassigned issue to task and target stage:

```bash
metagent issue assign <issue-id> --task add-login-rate-limit --stage build
```

### 7) Review/spec-review/research/debug workflows

Focused manual review run:

```bash
metagent review add-login-rate-limit "Focus on auth middleware and cache invalidation"
```

Spec review run:

```bash
metagent spec-review add-login-rate-limit
```

Research run (no direct state mutation by command itself):

```bash
metagent research add-login-rate-limit "Compare current approach with existing session architecture"
```

Debug run:

```bash
metagent debug "Login endpoint returns 500 after refresh token rotation"
# or
metagent debug --file crash.log
# or
cat crash.log | metagent debug --stdin
```

### 8) Writer workflow (practical)

Create and run:

```bash
metagent --agent writer task handbook-v1 --description "Internal engineering handbook"
metagent --agent writer run handbook-v1
```

Writer stage transitions are explicit:

```bash
metagent --agent writer finish init --session <session-id> --task handbook-v1
metagent --agent writer finish plan --session <session-id> --task handbook-v1
metagent --agent writer finish write --session <session-id> --task handbook-v1
metagent --agent writer finish edit --session <session-id> --task handbook-v1
```

Important:
- for writer, always pass stage to `finish` (`task` is not a valid writer finish stage)

### 9) Recovery and manual correction

If interrupted (`Ctrl+C`), resume:

```bash
metagent run add-login-rate-limit
```

If stage/status drifted and you need manual correction:

```bash
metagent set-stage add-login-rate-limit build --status pending
```

If multiple sessions exist and `finish` cannot resolve one uniquely, pass session explicitly:

```bash
metagent finish build --session <session-id> --task add-login-rate-limit
```

### Command behavior notes

- `metagent finish`:
  - default stage is `task` (works for `code`; `writer` should pass an explicit stage)
  - resolves session from `--session`, then `METAGENT_SESSION`, then a unique running session
- `metagent review <task> [focus]` runs a one-shot manual review stage (no auto-`finish` instruction)
- `metagent spec-review <task>` runs the spec-review stage once
- `metagent queue <task>` adds an existing task directory into tracked queue state if `task.json` is missing
- `metagent task <name>` creates a task; if task already exists it prints current state/history and can update `--description`

## End-to-End Code Workflow

### 1. Initialize repo

```bash
metagent init
```

Creates `.agents/code/` with templates and state folders. On first init, it may run bootstrap if placeholders are still present.

### 2. Create/enter tasks

Interactive route:

```bash
metagent start
```

Manual route:

```bash
metagent task my-feature
metagent run my-feature
```

Task name rules:
- lowercase letters, digits, `-`
- max length 100
- no leading `.` or `..`

### 3. Advance stages with `finish`

The model process is expected to call `metagent finish ...` for each stage.

Examples:

```bash
# default transition for a review session
metagent finish review --session <session-id> --task my-feature

# explicit next stage (review found issues)
metagent finish review --session <session-id> --task my-feature --next build

# send back to spec-review-issues
metagent finish review --session <session-id> --task my-feature --next spec-review-issues
```

Notes:
- `--session` can be omitted only when there is exactly one running session (or `METAGENT_SESSION` is set).
- If a task has open issues, finishing to `completed` is automatically redirected to `build`.

### 4. Run one task vs whole queue

```bash
metagent run my-feature        # keep running this task until completion/interruption
metagent run-next              # run one eligible stage once
metagent run-queue             # process eligible tasks until queue is empty
```

`run-queue` behavior:
- skips held tasks
- claims tasks via lock files to avoid collisions
- for `code`, enforces a review/build loop limit (default 4, `--loop 0` means 100)

### 5. Use issue tracking when blocked

```bash
metagent issue add --title "Fix flaky test" --task my-feature --type test --priority P1
metagent issues --task my-feature
metagent issue resolve <issue-id> --resolution "Stabilized retry logic"
```

Open issues set task status to `issues` and affect stage progression until resolved.

## Writer Workflow

Typical loop:

```bash
metagent --agent writer init
metagent --agent writer task my-book
metagent --agent writer run my-book
```

Writer has no issue subsystem. Queue stages are `write` and `edit`.

## State and File Layout

### Global (user home)

```text
~/.local/bin/metagent
~/.metagent/
  code/
  writer/
~/.claude/commands/
~/.codex/prompts/
```

### Per repo

```text
.agents/
  code/
    AGENTS.md
    SPEC.md
    TECHNICAL_STANDARDS.md
    tasks/<task>/
      spec/
      plan.md
      task.json
    sessions/<session-id>/session.json
    claims/<task>.lock
    issues/<issue-id>.md
  writer/
    AGENTS.md
    tasks/<task>/
      content/
      outline/
      style/
      research/
      editorial_plan.md
      task.json
    sessions/<session-id>/session.json
    claims/<task>.lock
```

Task status values:
- `pending`
- `running`
- `incomplete`
- `failed`
- `completed`
- `issues`

## Model Selection

Global options/env:
- `--model <claude|codex>` or `METAGENT_MODEL`
- `--force-model` or `METAGENT_FORCE_MODEL=1|true|yes`

Selection logic summary:
1. if task has open issues, `codex` is forced unless explicit model + force-model are both set
2. otherwise explicit model wins
3. otherwise agent stage defaults apply (`code` stages default to `codex`)

Other useful env vars:
- `METAGENT_AGENT` (default agent)
- `METAGENT_REPO_ROOT` (override repo root detection)
- `METAGENT_SESSION` and `METAGENT_TASK` (used by `finish` and model subprocesses)
- `METAGENT_CODESIGN_ID`, `METAGENT_SKIP_CODESIGN` (macOS install/signing)

## Development

### Build

```bash
cargo build
cargo build --release
```

### Test

Use `cargo nextest`:

```bash
cargo nextest run
```

Optional macOS build helper:

```bash
tools/build.sh
```

## Troubleshooting

- `No repo found (missing .agents/ or .git)`:
  - run inside a git repo, or run `metagent init` first, or set `METAGENT_REPO_ROOT`.
- `Task '<name>' is already claimed`:
  - another `run`/`run-queue` is active for that task.
- `METAGENT_SESSION not set and no unique active session found`:
  - pass `--session <id>` explicitly to `finish`.
- `Issue tracking is only supported for the code agent`:
  - run issue commands with `--agent code`.
