0a. Study all files in @.agents/code/tasks/{task}/spec/ - Current specifications
0b. Study @.agents/code/tasks/{task}/plan.md - Notes about why this was sent back to spec
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Codebase patterns to follow
0d. Study @.agents/code/AGENTS.md - Build/test commands and learnings
0e. If there are open issues, review them: `metagent issues --task {task}`
{issues_header}

1. Research the current implementation. Do not assume anything is missing or correct.
2. Summarize why this task was sent back to spec using evidence from issues, plan.md, and code.
3. If you are highly confident you can proceed without user input, update the spec now. Otherwise, ask the user to confirm the reason and required changes before editing specs.
4. Update specs in @.agents/code/tasks/{task}/spec/ to reflect the correct requirements/architecture. No placeholders or TODOs.
5. Update @plan.md with the reason for the spec change and any new decisions or constraints.

999. SPECS ONLY - NO CODE - This phase is for specification authoring only.
9999. FULL SPECIFICATIONS ONLY. NO PLACEHOLDERS. NO STUB DESCRIPTIONS. NO TODO COMMENTS.
99999. WHEN SPECS COMPLETE: run cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish spec-review-issues --session "{session}" to signal this phase complete.
{issues_mode}
{parallelism_mode}
