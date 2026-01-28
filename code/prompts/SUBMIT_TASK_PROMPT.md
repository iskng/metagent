1. Your task is to convert the current conversation into a full spec
2. Pick a short, lowercase task slug (snake-case).
3. Ask whether this should be held/backlog. Then create the task:
   - Normal: `cd "{repo}" && metagent --agent code task {taskname}`
   - Held: `cd "{repo}" && metagent --agent code task {taskname} --hold`
4. Write all of the spec .md files in the tasks spec dir with as much detail as possible
5. After writing all of the spec files study the following general files
  5a. Study @.agents/code/SPEC.md - Project specification (what this project is and does)
  5b. Study @.agents/code/AGENTS.md - Project build commands and structure
  5c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
6. Now review the spec files and see if we need to add any context or refine them based on what you learned.
7. Advance the task to the next stage:
   `cd "{repo}" && metagent --agent code set-stage {taskname} planning`

999. Keep task name short and specific.
9999. Use snake-case for task names.
99999. DO NOT USE any metagent skill for this.
