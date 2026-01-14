# Planning Phase

If the task name is provided (e.g., "Task: auth-system"), use that name. Otherwise ask which task to plan.

Verify `.agents/code/tasks/{taskname}/` exists.

---

Study these files to understand the project and task:

- @.agents/code/SPEC.md - What this project is and does
- @.agents/code/AGENTS.md - Build/test commands
- @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
- @.agents/code/tasks/{taskname}/spec/*.md - Task specifications
- @.agents/code/tasks/{taskname}/plan.md - Current plan (may be incomplete or incorrect)

---

First task is to study the spec/*.md files and plan.md and use up to 100 subagents to study existing source code and compare it against the specifications. From that create/update plan.md which is a bullet point list sorted in priority of the items which have yet to be implemented. Think extra hard and use the oracle to plan. Consider searching for TODO, minimal implementations and placeholders. 

Second task is to use up to 100 subagents to study existing tests and examples, then compare against the specifications. From that update plan.md with items that are missing or incomplete. Consider searching for TODO, minimal implementations and placeholders.

Write plan.md as a bullet point list sorted by priority:

```markdown
# Implementation Plan - {taskname}

- [ ] {task} - blocks: {what this enables}
- [ ] {task}
- [ ] {task}
...
```

---

9. Study ALL spec files and existing plan before researching codebase

99. Research codebase exhaustively before generating tasks - use subagents liberally

999. Every spec requirement must map to a task in plan.md

9999. Keep plan.md up to date with your findings using subagents

99999. Think extra hard and use the oracle when planning complex items

999999. If you find inconsistencies in specs, use a subagent to resolve and update them

9999999. Combine small (S) tasks into medium (M) tasks - avoid many tiny items

99999999. **PLAN ONLY - NO IMPLEMENTATION**

999999999. **EVERY SPEC REQUIREMENT MUST MAP TO A TASK**

9999999999. **WHEN PLAN IS COMPLETE, run `metagent finish planning` to advance to build phase**
