# SUBMIT ISSUE - Quick Capture

## Purpose

Use this when issues have already been identified and just need to be logged.
Do **not** debug, fix, or run tests.

---

## Steps

1. Create a new issue using the CLI.
2. Keep it concise and factual; no speculation.
3. Do not assign to a task unless explicitly instructed.
4. If explicitly instructed to assign, either include `--task <taskname>` in the add command or run `metagent issue assign <issue-id> --task <taskname>` afterward.

---

## CLI Template

```bash
cat <<'EOF' | metagent issue add --title "{Human-Readable Title}" --priority P2 --type bug --source submit --stdin-body
## Description
{Clear description of the bug}

## Evidence
{Logs, errors, or references}

## Affected Area (if known)
- {file/module}

## Suggested Fix (optional)
{If already known}
EOF
```

To assign directly (only if instructed), add `--task <taskname>`:

```bash
cat <<'EOF' | metagent issue add --title "{Human-Readable Title}" --task <taskname> --priority P2 --type bug --source submit --stdin-body
## Description
{Clear description of the bug}
EOF
```

---

## Rules

1. One issue per command.
2. Do not assign to a task unless explicitly instructed.
3. Do not make code changes.
