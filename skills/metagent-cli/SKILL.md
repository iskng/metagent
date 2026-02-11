---
name: metagent-cli
description: Operate the Metagent Rust CLI to manage code or writer agent workflows in a repo, including install/uninstall, init/start/run/finish, queue/dequeue, review/spec-review, debug, and code-agent issue tracking with required flags, environment variables, and file locations.
---

# Metagent CLI

## Setup and prerequisites
- Select agent with `--agent code|writer` or `METAGENT_AGENT` (default: code).
- Point to a specific repo with `METAGENT_REPO_ROOT`; otherwise Metagent searches upward for `.agents/` or `.git`.
- On macOS, control codesigning with `METAGENT_CODESIGN_ID` or `METAGENT_SKIP_CODESIGN=1` during `install`.

## Core workflow
- Initialize a repo: `mung init [path]`.
- Start an interactive run: `mung start`.
- Create a task explicitly: `mung task <name>`.
- Run a task loop: `mung run <name>`.
- Process the queue: `mung run-queue`.

## Stage model
- Code stages: `spec -> spec-review -> planning -> build -> review -> completed` (handoff stage: build).
- Writer stages: `init -> plan -> write -> edit -> completed`.
- Advance a session with `mung finish <stage> --session <id> [--next <stage>] [--task <name>]`.
- Omit `<stage>` to use `task`, which marks the task as completed.
- Resolve session ID in this order: `--session`, `METAGENT_SESSION`, or a single running session in `.agents/<agent>/sessions/`.
- Resolve task in this order: `--task`, `METAGENT_TASK`, session task; for non-`task` stages, Metagent may pick the unique running task at that stage.
- Use `--next` to override the default next stage; otherwise the agent stage order applies.
- Expect tasks with open issues to stay in status `issues` after `finish`.
- Manually update a task stage: `mung set-stage <task> <stage> [--status pending|running|incomplete|failed|completed|issues]`.

## Queue and task rules
- Use task names with lowercase letters, digits, and hyphens only; max 100 chars; no leading dot or `..`.
- List tasks: `mung queue`.
- Add an existing task directory to state: `mung queue <task>` (creates `task.json` if missing).
- Remove a task and its files: `mung dequeue <task>`.

## Issues (code agent only)
- List issues: `mung issues [--task <name>|--unassigned] [--status open|resolved|all] [--priority P0..P3] [--type spec|build|bug|test|perf|other] [--source review|debug|submit|manual]`.
- Add an issue: `mung issue add --title "..." [--task <name>] [--priority P0..P3] [--type spec|build|bug|test|perf|other] [--source review|debug|submit|manual] [--file <path>] [--stage <stage>] [--body <text>|--stdin-body]`.
- Resolve an issue: `mung issue resolve <id> [--resolution "..."]`.
- Assign an issue: `mung issue assign <id> --task <name> [--stage <stage>]`.
- Show the raw issue file: `mung issue show <id>`.
- Expect adding or assigning an issue to mark the task as `issues`; completed tasks return to `spec` for spec issues or `build` otherwise (unless `--stage` overrides).

## Review and debug
- Review a task: `mung review <task> [focus text]` (injects focus; does not change stage unless `finish` is called).
- Spec review: `mung spec-review <task>` (one-shot; does not change stage unless `finish` is called).
- Debug: `mung debug [--file <path>|--stdin] [bug text...]` (always uses codex and prepends bug context to the debug prompt).

## Files and state
- Prompts: `~/.mung/<agent>/` (used if present; otherwise embedded defaults).
- Repo state: `.agents/<agent>/tasks/<task>/task.json`, `.agents/<agent>/sessions/<id>/session.json`, `.agents/<agent>/issues/*.md`, `.agents/<agent>/claims/*.lock`.
