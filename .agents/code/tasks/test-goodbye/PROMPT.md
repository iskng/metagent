0a. Study all files in @.agents/code/tasks/test-goodbye/spec/- Task specifications and architecture
0b. Study @.agents/code/TECHNICAL_STANDARDS.md - Codebase patterns to follow
0c. Study @.agents/code/tasks/test-goodbye/plan.md - Current task list
0d. Study @.agents/code/AGENTS.md - Build/test commands and learnings

1. Your task is to implement test-goodbye per the specifications. study @plan.md, choose the most important uncompleted items that you can accomplish in one pass (max 5), research using subagents before implementing (NEVER assume code doesn't exist), implement according to specifications.

2. After implementing functionality or resolving problems, run the tests for that unit of code that was improved. If functionality is missing then it's your job to add it as per the application specifications.

3. When the tests pass update @plan.md, then add changed code and @plan.md with git add the relevant files you created/modified via bash then do git commit -m "feat(test-goodbye): [descriptive message]"

4. ALWAYS KEEP @plan.md up to date with your learnings about the task using a subagent. After wrapping up/finishing your turn append a short session-x summary with what was accomplished and any relevant notes.

5. When you learn something new about how to run the build/tests make sure you update @.agents/code/AGENT.md but keep it brief.




999999. Important: We want single sources of truth, no migrations/adapters. If tests unrelated to your work fail then it's your job to resolve these tests as part of the increment of change.
99999999. Important: When authoring tests capture the WHY - document importance in docstrings.
999999999. IMPORTANT: When you discover a bug resolve it using subagents even if it is unrelated to the current piece of work after documenting it in @plan.md
9999999999. You may add extra logging if required to be able to debug the issues.
99999999999. When @plan.md becomes large periodically clean out the items that are completed from the file using a subagent.
999999999999. If you find inconsistencies in the specs/* then use the oracle (think extra hard) and then update the specs.
9999999999999. Allowed parallelism: Codebase search up to 50 subagents, file reading up to 50 subagents, file writing up to 10 subagents (independent files only), build/test 1 SUBAGENT ONLY, plan.md updates 1 subagent.
99999999999999. FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS. NO STUBS. NO TODO COMMENTS. DO NOT IMPLEMENT PLACEHOLDER OR SIMPLE IMPLEMENTATIONS. WE WANT FULL IMPLEMENTATIONS. DO IT OR I WILL YELL AT YOU
999999999999999. SUPER IMPORTANT DO NOT IGNORE. DO NOT PLACE STATUS REPORT UPDATES INTO @.agents/code/AGENT.md
9999999999999999. **WHEN ITEM DONE:** run `cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish --next build` to signal iteration complete (more items remain).
99999999999999999. **WHEN ALL ASPECTS OF THE PLAN.md ARE COMPLETE:** run a full `cargo build` to verify everything compiles, then run `cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish` to signal task complete (all items done).
