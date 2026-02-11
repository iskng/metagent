---
name: metagent-code-cli
description: Use the Metagent Rust CLI for code agent workflows (spec → planning → build → review), including init/start/run/finish, queue/dequeue, review/spec-review/debug, and code-agent issue tracking with flags, env vars, and file locations.
---

# Metagent Code Agent CLI

## Setup
- Run `mung install` to copy the binary to `~/.local/bin/mung` and install prompts into `~/.mung/code/`.
- Ensure the model CLIs are on PATH: `claude --dangerously-skip-permissions` and `codex --dangerously-bypass-approvals-and-sandbox`.
- Use `--agent code` or `METAGENT_AGENT=code` (default).
- Choose model with `--model claude|codex` or `METAGENT_MODEL` (defaults to claude unless stage rules select codex).
- Set `METAGENT_REPO_ROOT` to pin a repo; otherwise Metagent searches up for `.agents/` or `.git`.

## Core workflow
- Initialize: `mung init [path]`.
- Start interview/spec/planning: `mung start` (stops at build handoff).
- Create task: `mung task <name>`.
- Run a task loop: `mung run <name>`.
- Process queue: `mung run-queue`.

## Stages and finish
- Stages: `spec -> spec-review -> planning -> build -> review -> completed`.
- Advance a session: `mung finish <stage> --session <id> [--next <stage>] [--task <name>]`.
- Use `<stage>=task` to mark the task complete.
- Resolve session ID in order: `--session`, `METAGENT_SESSION`, or a single running session in `.agents/code/sessions/`.
- Resolve task in order: `--task`, `METAGENT_TASK`, session task; for non-`task` stages, a unique running task at that stage may be used.
- Use `--next` to override the next stage; otherwise the agent stage order applies.
- Expect tasks with open issues to keep status `issues` after `finish`.
- Manually update a task stage: `mung set-stage <task> <stage> [--status pending|running|incomplete|failed|completed|issues]`.

## Queue and task rules
- Task names: lowercase letters, digits, hyphens only; max 100 chars; no leading dot or `..`.
- List tasks: `mung queue`.
- Add existing task dir to state: `mung queue <task>` (creates `task.json` if missing).
- Remove task + files: `mung dequeue <task>`.

## Issues (code agent only)
- List issues: `mung issues [--task <name>|--unassigned] [--status open|resolved|all] [--priority P0..P3] [--type spec|build|bug|test|perf|other] [--source review|debug|submit|manual]`.
- Add: `mung issue add --title "..." [--task <name>] [--priority P0..P3] [--type spec|build|bug|test|perf|other] [--source review|debug|submit|manual] [--file <path>] [--stage <stage>] [--body <text>|--stdin-body]`.
- Resolve: `mung issue resolve <id> [--resolution "..."]`.
- Assign: `mung issue assign <id> --task <name> [--stage <stage>]`.
- Show raw issue file: `mung issue show <id>`.
- Expect adding/assigning an issue to mark the task as `issues`; completed tasks return to `spec` for spec issues or `build` otherwise (unless `--stage` overrides).

## Review and debug
- Review: `mung review <task> [focus text]` (injects focus; stage changes only via `finish`).
- Spec review: `mung spec-review <task>` (one-shot; stage changes only via `finish`).
- Debug: `mung debug [--file <path>|--stdin] [bug text...]` (always uses codex and prepends bug context).

## Files and state
- Prompts: `~/.mung/code/` (used if present; otherwise embedded defaults).
- Repo state: `.agents/code/tasks/<task>/task.json`, `.agents/code/sessions/<id>/session.json`, `.agents/code/issues/*.md`, `.agents/code/claims/*.lock`.
