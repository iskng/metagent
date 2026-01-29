0a. Study all files in @.agents/code/tasks/{task}/spec/ - Task specifications and architecture
0b. Study @.agents/code/TECHNICAL_STANDARDS.md - Codebase patterns to follow
0c. Study @.agents/code/tasks/{task}/plan.md - Current task list
0d. Study @.agents/code/AGENTS.md - Build/test commands and learnings
{issues_header}

1. Your task is to implement {task} per the specifications. Study @plan.md, choose the most important uncompleted items that you can accomplish in one pass (max 5), research before implementing (NEVER assume code doesn't exist), implement according to specifications.

2. After implementing functionality or resolving problems, run the tests for that unit of code that was improved. If functionality is missing then it's your job to add it as per the application specifications.

3. When the tests pass update @plan.md. Only commit if there are actual code changes. If there are no code changes (e.g., only @plan.md/session notes updated), skip git add/commit for this loop. If you did change code, add the relevant files (and @plan.md if updated) then do git commit -m "feat({task}): [descriptive message; plan steps completed]". The commit message must include the specific plan step(s) completed (e.g., "feat({task}): add X (plan: 2,4)").

4. ALWAYS KEEP @plan.md up to date with your learnings about the task. After wrapping up/finishing your turn append a short session-x summary with what was accomplished and any relevant notes.

5. When you learn something new that is critical or make a mistake several times, wrong timeouts, bad syntax, etc. make sure you update @.agents/code/AGENTS.md but keep it brief.

999999. Important: We want single sources of truth, no migrations/adapters. If tests unrelated to your work fail then it's your job to resolve these tests as part of the increment of change.
99999999. Important: When authoring tests capture the WHY - document importance in docstrings.
999999999. IMPORTANT: When you discover a bug resolve it even if it is unrelated to the current piece of work after documenting it in @plan.md
9999999999. You may add extra logging if required to be able to debug the issues.
99999999999. If you find inconsistencies in the specs/* then use the oracle (think extra hard) and then update the specs.
999999999999. FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS. NO STUBS. NO TODO COMMENTS. DO NOT IMPLEMENT PLACEHOLDER OR SIMPLE IMPLEMENTATIONS. WE WANT FULL IMPLEMENTATIONS. DO IT OR I WILL YELL AT YOU
9999999999999. SUPER IMPORTANT DO NOT IGNORE. DO NOT PLACE STATUS REPORT UPDATES INTO @.agents/code/AGENTS.md
99999999999999. **WHEN ITEM DONE:** run `cd "{repo}" && METAGENT_SESSION="{session}" METAGENT_TASK="{task}" metagent --agent code finish --next build` to signal iteration complete (more items remain).
999999999999999. **WHEN ALL ASPECTS OF THE PLAN.md ARE COMPLETE:** run a full `cargo build` to verify everything compiles, then run `cd "{repo}" && METAGENT_SESSION="{session}" METAGENT_TASK="{task}" metagent --agent code finish` to signal task complete (all items done).
{issues_mode}
{parallelism_mode}
