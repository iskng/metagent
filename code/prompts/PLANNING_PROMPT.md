0a. Study @.agents/code/SPEC.md - What this project is and does
0b. Study @.agents/code/AGENTS.md - Build/test commands
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
0d. Study @.agents/code/tasks/{taskname}/spec/*.md - Task specifications
0e. Study @.agents/code/tasks/{taskname}/plan.md - Current plan (may be incomplete, incorrect, or not created yet)

1. If task name is provided (e.g., "Task: auth-system"), use that name. Otherwise ask which task to plan. Verify .agents/code/tasks/{taskname}/ exists.

2. Study existing source code and compare against specifications. Search for TODO, minimal implementations, and placeholders. Create/update plan.md as a prioritized bullet point list of items yet to be implemented. Read all of the relevant files mentioned in the spec and research in the code any files that might be related. 

3. Study tests, examples and existing implementations, compare against specifications. Update plan.md with items that are missing or incomplete. Search for TODO, minimal implementations, and placeholders.

4. Write plan.md as a bullet point implementation plan sorted by priority:
{taskname}
[ ] [task] - blocks: {what this enables}
[ ] [task]
[ ] [task]

5. Study ALL spec files and existing plan before researching codebase.

6. Research codebase exhaustively before generating tasks.

999. Every spec requirement must map to a task in plan.md.
9999. Keep plan.md up to date with your findings.
99999. Think extra hard and use the oracle when planning complex items.
999999. If you find inconsistencies in specs, resolve and update them.
9999999. Combine small (S) tasks into medium (M) tasks - avoid many tiny items.
99999999. EVERY SPEC REQUIREMENT MUST MAP TO A TASK
999999999. PLAN ONLY - NO IMPLEMENTATION
9999999999. WHEN PLAN IS COMPLETE: run cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish planning --session "{session}" to advance to build phase.
{issues_mode}
{parallelism_mode}
