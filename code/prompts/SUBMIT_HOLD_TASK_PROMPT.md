0a. Study @.agents/code/SPEC.md - Project specification (what this project is and does)
0b. Study @.agents/code/AGENTS.md - Project build commands and structure
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow

1. Do NOT ask any questions. Use only what the conversation has already confirmed.
2. Pick a short, lowercase task slug (kebab-case) based on confirmed scope.
3. Create the task in held/backlog state:
   `cd "{repo}" && metagent --agent code task {taskname} --hold`
4. Update spec files in `.agents/code/tasks/{taskname}/spec/` with ONLY confirmed information. Do not speculate. If there is no confirmed info for a file, leave it as just the header (no placeholders, no TODOs).
5. Stop after writing the partial spec. Do not run planning, build, or `metagent finish`.

999. NO QUESTIONS.
9999. NO SPECULATION - ONLY CONFIRMED DETAILS.
99999. SUBMIT HOLD TASK ONLY - DO NOT START SPECS OR PLANNING.
999999. DO NOT call `metagent finish` in this flow.
