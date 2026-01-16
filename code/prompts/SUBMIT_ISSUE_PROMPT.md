# SUBMIT ISSUE - Inbox Capture

## Purpose

Use this when issues have already been identified and just need to be logged in the inbox.
Do **not** debug, fix, or run tests.

---

## Steps

1. Create `.agents/code/inbox/` if missing.
2. Create one issue file per bug in `.agents/code/inbox/{bug-title}.md`.
3. Update `.agents/code/inbox.md` with an entry for each issue.
4. Keep it concise and factual; no speculation.

---

## Issue Template

```markdown
# Bug: {Human-Readable Title}

> Created: {date}
> Status: OPEN
> Priority: {Critical/High/Medium/Low}
> Task: unassigned
> Source: submit-issue

## Description

{Clear description of the bug}

## Evidence

{Logs, errors, or references}

## Affected Area (if known)

- {file/module}

## Suggested Fix (optional)

{If already known}
```

---

## Inbox Index Template

```markdown
# Inbox

> Last updated: {date}

## Open

| Issue | Priority | Task | Created |
|------|----------|------|---------|
| [{bug-title}](inbox/{bug-title}.md) | {priority} | unassigned | {date} |
```

---

## Rules

1. One issue per file.
2. Do not assign to a task unless explicitly instructed.
3. Do not make code changes.
