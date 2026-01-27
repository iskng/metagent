# Metagent Rust Rewrite Spec

## Summary
Replace the shell-based metagent orchestration with a single Rust CLI that owns all state, concurrency,
and lifecycle logic. Preserve user-facing behavior, prompts, and agent workflows while eliminating
shared-file races and multi-agent collisions. Shell scripts remain untouched and become legacy only.

## Goals
- Keep the same workflows, prompts, and stage semantics for `code` and `writer` agents.
- Make correctness-critical state updates atomic and race-free.
- Enforce one session per model run and make finish explicit via session IDs.
- Remove the shared queue file and derive queue views from per-task state.
- Keep prompts template-driven with `{repo}`, `{task}`, `{taskname}`, `{session}`, `{issues_mode}`,
  `{issues_header}`, `{parallelism_mode}`, `{focus_section}` replacements.
- Remain compatible with existing `~/.metagent/` prompt installs and `.agents/` repo structure.

## Non-goals
- Changing prompt content, stage order, or task semantics.
- Redesigning slash commands or prompt templates.
- Introducing a database backend (SQLite) in this iteration.
- Breaking CLI UX or requiring new environment variables for normal use.

## Current Behavior (Reference)
- Agents:
  - `code`: stages `spec -> spec-review -> planning -> build -> review -> completed`
  - `writer`: stages `init -> plan -> write -> completed` (plan/write can cycle via `--next`)
- Commands: `start`, `task`, `finish`, `run`, `run-queue`, `queue`, `dequeue`, `review`, `spec-review`,
  `install`, `uninstall`, `init`
- `start` orchestrates interview -> spec -> planning then hands off at `build` for `code`.
- `finish` updates queue stage/status and signals the orchestrator.
- Prompts embed finish commands with `--session {session}` and `METAGENT_TASK`.
- Queue is `queue.jsonl` (append + in-place edits), vulnerable to concurrent writes.

## Proposed Architecture (Rust-First)
### Rust CLI
- A single `metagent` Rust binary provides all current commands.
- Shell scripts remain unchanged; the Rust binary becomes the primary entry point.
- CLI uses `clap` for flags and subcommands; `anyhow` or `thiserror` for errors.

### Agent Model
Implement agents as Rust structs that satisfy a shared trait.

`Agent` trait:
- `name() -> &str`
- `stages() -> &[Stage]`
- `initial_stage() -> Stage`
- `next_stage(stage) -> Option<Stage>`
- `valid_finish_stages() -> &[Stage]`
- `handoff_stage() -> Option<Stage>`
- `prompt_path(stage, task) -> PathBuf`
- `review_prompt_path() -> PathBuf`
- `model_for_stage(stage) -> Option<Model>`
- `create_task(task_dir, task_name) -> Result<()>`
- `stage_label(stage) -> &str`

Built-in agents:
- `code` and `writer`, matching current `agent.sh` logic and templates.
- Use installed prompts from `~/.metagent/<agent>/` when present, otherwise embedded defaults.

## State Model (No Shared Queue)
All state lives under `.agents/<agent>/`.

Paths:
- `.agents/<agent>/tasks/<task>/task.json`
- `.agents/<agent>/sessions/<session_id>/session.json`
- `.agents/<agent>/claims/<task>.lock`
- `.agents/<agent>/issues/<issue_id>.md` (code agent issue tracking)
- Optional: `.agents/<agent>/logs/<session_id>.log`

### task.json
Fields:
- `task`: string
- `agent`: string
- `stage`: string
- `status`: enum (`pending`, `running`, `incomplete`, `failed`, `completed`, `issues`)
- `held`: boolean (backlog item excluded from run-queue)
- `added_at`: ISO8601
- `updated_at`: ISO8601
- `last_session`: string or null
- `last_error`: string or null (optional)

### session.json
Fields:
- `session_id`: string
- `task`: string
- `agent`: string
- `stage`: string
- `status`: enum (`running`, `finished`, `failed`)
- `started_at`: ISO8601
- `finished_at`: ISO8601 or null
- `next_stage`: string or null
- `pid`: integer
- `host`: string
- `repo_root`: string

### claims/<task>.lock
Fields:
- `task`: string
- `agent`: string
- `pid`: integer
- `host`: string
- `started_at`: ISO8601
- `ttl_seconds`: integer

Lock semantics:
- Create with `create_new(true)` to ensure atomic claim.
- Treat claim as stale if `now - started_at > ttl_seconds` or PID not alive on same host.

## Session IDs
- Format: `<epoch_seconds>-<pid>` (example: `1704811163-8421`).
- Sequential per process; no random or long UUIDs.
- Always injected into prompt templates as `{session}`.

## Prompt Rendering
Replace the following placeholders:
- `{repo}`: repo root path.
- `{task}` and `{taskname}`: task name.
- `{session}`: session ID.
- `{issues_header}` and `{issues_mode}`: added when task status is `issues`.
- `{parallelism_mode}`: injected for `claude` models only.
- `{focus_section}`: only for `review` command with `--focus`.

Non-replaced braces remain for the model to fill (dates, examples, etc).

## Command Mapping
All commands mirror current behavior and flags.

### install
- Write `metagent` binary into `~/.local/bin/metagent`.
- Copy embedded prompts to `~/.metagent/<agent>/`.
- Create slash command symlinks in:
  - `~/.claude/commands`
  - `~/.codex/prompts`
- Warn if `~/.local/bin` is not in PATH.

### uninstall
- Remove `~/.local/bin/metagent`.
- Remove slash command symlinks pointing into `~/.metagent/`.
- Remove `~/.metagent/`.

### init [path]
- Ensure target repo exists; warn if no `.git`.
- Create `.agents/<agent>/` and `.agents/<agent>/tasks/`.
- Copy templates (`AGENTS.md`, `SPEC.md`, `TECHNICAL_STANDARDS.md` for `code`) if missing.
- Prompt user before overwrite if directory exists.

### task <name>
- Validate task name: lowercase, numbers, hyphens; max 100; no traversal.
- Create task directory and templates (agent-specific).
- Create `task.json` with initial stage and status `pending`.
- `--hold` creates a backlog task excluded from `run-queue` until activated.

### queue
- Scan `.agents/<agent>/tasks/*/task.json` to build view.
- Group by stage, ordered by `agent.stages()`.
- Display per-task status (pending/issues/incomplete/running/failed/completed).
- Held tasks show under a Backlog section.

### dequeue <name>
- Remove task directory (including `task.json` and task files).
- No shared queue file to edit.

### start
- Orchestrate `interview -> spec -> planning` for `code`; `init -> plan -> write` for `writer`.
- Start a new session for each model run, render prompt, spawn model CLI.
- Wait for `finish` to update `session.json`, then advance stage.
- Stop at handoff stage (`build`) for `code`.

### run <task>
- Load `task.json`; if completed, exit.
- Loop: create session, spawn model, wait for finish; update `task.json`.
- If model exits without finish, mark `incomplete` and exit.

### run-queue
- Scan all tasks; pick next `pending`/`incomplete` by stage order and `added_at`.
- Claim the task with `claims/<task>.lock`.
- Run a single stage loop (same as `run`), then release claim.
- Repeat until no eligible tasks remain.
- Skip held tasks.

### run-next
- Run the next eligible task for a single stage, then exit.
- Skip held tasks.

### hold/activate
- `hold`: mark a task as held/backlog (excluded from run-queue).
- `activate`: un-hold a task.

### issues
- List issues with filters (task, status, priority, type, source).

### issue
- Manage issues (add/assign/resolve/show).

### finish <stage> [--next <stage>] [--session <id>] [--task <task>]
- Require a session ID from `--session` (prompted) or `METAGENT_SESSION` (fallback).
- If not provided, and exactly one active session exists, use it.
- Resolve session -> task/stage; validate stage against agent.
- Determine `next_stage` from override or agent.
- Update `session.json` with `finished_at` and `next_stage`.
- Update `task.json`:
  - `stage = next_stage`
  - `status = completed` if `next_stage == completed`
  - `status = issues` if review stage returned with `--next` or prior status is `issues`
  - otherwise `status = pending`

### review <task> [--focus <text>]
- One-shot model run using review prompt.
- Inject `{focus_section}` if provided.
- Does not change stage unless `finish` is called.

### spec-review <task>
- One-shot model run using spec review prompt.
- Does not change stage unless `finish` is called.

## Orchestration and Model Runner
- Use `std::process::Command` with `Stdio::inherit()` to keep interactive UX.
- Pass prompt as a single argument to the model CLI, same as today.
- Model CLI selection:
  - `claude --dangerously-skip-permissions`
  - `codex --dangerously-bypass-approvals-and-sandbox`
- Stage-specific model overrides mirror `agent_model_for_stage`.
- If task status is `issues`, default to `codex` when no explicit model is set.
- Use `ctrlc` to trap SIGINT; mark running task as `incomplete` and close session.

## Concurrency and Atomicity
- All writes to `task.json` and `session.json` use atomic write + rename.
- Use `fs2` or `fd-lock` for file locks when mutating state.
- Claims prevent two `run-queue` workers from running the same task.
- No shared queue file means no global write contention.

## Migration
On first Rust run:
- If `queue.jsonl` exists, parse each line into `task.json` entries.
- Use `added` from queue line as `added_at`.
- Preserve `stage` and `status`.
- Archive `queue.jsonl` as `queue.jsonl.bak` (or delete if configured).
- If a task directory exists without a queue entry, create `task.json` as `pending`.

## Testing
Unit tests:
- Task name validation.
- Stage transitions (including `review` with `--next` and `issues` propagation).
- Prompt rendering replacement logic.
- Session ID formatting.

Integration tests:
- Concurrent `finish` calls for different sessions.
- `run-queue` claim contention and stale claim recovery.
- Crash recovery (runner exits before finish).

CLI tests:
- `task`, `run`, `finish`, `queue`, `dequeue` happy paths and errors.
- `start` handoff semantics for `code`.

## Rollout Plan
1. Implement Rust CLI and ship as `metagent` binary.
2. Keep shell scripts untouched for reference/rollback.
3. Add migration on first run.
4. Validate with a dry-run on an existing repo; compare behavior with current shell.

## Alternatives Considered
- Append-only event log (beads-style) with reducers:
  - Pros: full audit trail, easy replay.
  - Cons: more complex reducers, harder migration, still needs locking on log writes.
- SQLite:
  - Pros: built-in locking, easy queries.
  - Cons: heavier dependency, more complex install and schema migrations.
- Current queue.jsonl + locks:
  - Pros: minimal change.
  - Cons: still fragile, lock coordination in shell is error-prone.

Chosen approach is per-task state files with atomic writes and lightweight claims.

## Open Questions
- Should we keep an optional event log for audit/debug?
- Should finish accept task-only (no session) when there is a single running session for that task?
- How long should claim TTL be by default (minutes vs hours)?
