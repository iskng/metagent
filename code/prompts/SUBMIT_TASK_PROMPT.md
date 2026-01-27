0a. Study @.agents/code/SPEC.md - Project specification (what this project is and does)
0b. Study @.agents/code/AGENTS.md - Project build commands and structure
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow

1. Your task is to quickly capture a new task name and minimal scope. Ask 1–2 short batches of clarification questions (don’t overwhelm). Don’t ask questions you already have answers to.
2. Summarize understanding back to the user to confirm scope.
3. Pick a short, lowercase task slug (kebab-case).
4. Ask whether this should be held/backlog. Then create the task:
   - Normal: `cd "{repo}" && metagent --agent code task {taskname}`
   - Held: `cd "{repo}" && metagent --agent code task {taskname} --hold`
5. Stop after task creation. Do not proceed to spec or planning.

999. Keep it short and specific.
9999. Use kebab-case for task names.
99999. SUBMIT TASK ONLY - NO SPECS - NO PLANNING.
999999. DO NOT call `metagent finish` in this flow.
9999999. Ensure the user explicitly confirms whether the task should be held/backlog.
