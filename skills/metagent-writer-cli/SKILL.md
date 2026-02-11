---
name: metagent-writer-cli
description: Use the Metagent Rust CLI for writer agent workflows (init → plan → write → edit), including init/start/run/finish, queue/dequeue, flags, env vars, and writer-specific file locations.
---

# Metagent Writer Agent CLI

## Setup
- Run `mung install` to copy the binary to `~/.local/bin/mung` and install prompts into `~/.mung/writer/`.
- Ensure the model CLI is on PATH (typically `claude --dangerously-skip-permissions`).
- Use `--agent writer` or `METAGENT_AGENT=writer`.
- Choose model with `--model claude|codex` or `METAGENT_MODEL` (defaults to claude).
- Set `METAGENT_REPO_ROOT` to pin a repo; otherwise Metagent searches up for `.agents/` or `.git`.

## Core workflow
- Initialize: `mung --agent writer init [path]`.
- Start interview/plan/write loop: `mung --agent writer start`.
- Create task: `mung --agent writer task <name>`.
- Run a task loop: `mung --agent writer run <name>`.
- Process queue: `mung --agent writer run-queue`.

## Stages and finish
- Stages: `init -> plan -> write -> edit -> completed`.
- Advance a session: `mung --agent writer finish <stage> --session <id> [--next <stage>] [--task <name>]`.
- Use `<stage>=task` to mark the task complete.
- Use `--next` to cycle or hold a stage (for example `finish write --next write` to continue writing; `finish write --next plan` to re-plan).
- Resolve session ID in order: `--session`, `METAGENT_SESSION`, or a single running session in `.agents/writer/sessions/`.
- Resolve task in order: `--task`, `METAGENT_TASK`, session task; for non-`task` stages, a unique running task at that stage may be used.
- Manually update a task stage: `mung --agent writer set-stage <task> <stage> [--status pending|running|incomplete|failed|completed|issues]`.

## Queue and task rules
- Task names: lowercase letters, digits, hyphens only; max 100 chars; no leading dot or `..`.
- List tasks: `mung --agent writer queue`.
- Add existing task dir to state: `mung --agent writer queue <task>` (creates `task.json` if missing).
- Remove task + files: `mung --agent writer dequeue <task>`.

## Unsupported commands
- Do not use `issues`, `issue`, `review`, `spec-review`, or `debug` with the writer agent; these are code-agent-only and will fail due to missing prompts.

## Files and state
- Prompts: `~/.mung/writer/` (used if present; otherwise embedded defaults).
- Repo state: `.agents/writer/tasks/<task>/task.json`, `.agents/writer/sessions/<id>/session.json`, `.agents/writer/claims/*.lock`.
- Task structure created by `mung task`: `content/`, `outline/`, `style/`, `research/`, plus `editorial_plan.md`.
