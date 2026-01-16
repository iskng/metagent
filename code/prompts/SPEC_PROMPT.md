0a. Study @.agents/code/SPEC.md - Project specification (what this project is and does)
0b. Study @.agents/code/AGENTS.md - Project build commands and structure
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
{issues_header}

1. Your task is to gather requirements through conversation and research of existing code and create a spec. Ask 2-4 batches of questions - don't overwhelm. Questions like: "What problem does this solve?", "What does success look like?", "What are explicit NON-goals?". Don't ask questions you already have answers to.
2. After each set of answers research the codebase or search online. Make sure to clarify any key decisions that you are not sure about. Document answers immediately in working notes.md
3. Summarize understanding back to user, probe ambiguous areas, continue until user confirms understanding is complete.
4. Once you understand the problem, scope, boundaries, and key requirements, create the task using: cd "{repo}" && metagent --agent code task {taskname} this will create the .agents/code/tasks/{taskname}/spec/ dir for you.
5. Explore codebase. Find: APIs called/exposed, modules imported/importing, shared types, database tables, external services. Analyze: error handling patterns, logging patterns, testing patterns, validation patterns, naming conventions, similar implementations.
6. Author detailed specs in .agents/code/tasks/{taskname}/spec/ including: overview.md (purpose, goals, non-goals, architecture, dependencies, success criteria, relevant files), types.md (complete type definitions with fields, invariants, examples), modules.md (logical modules with public interface, parameters, errors, edge cases), errors.md (error types, when/contains/recovery).
7. Before completing spec phase verify: Types are EXACT, Signatures are COMPLETE, Examples are CONCRETE, Edge cases are EXHAUSTIVE, Errors are SPECIFIC, Dependencies are MAPPED, Success criteria are TESTABLE.
8. After specs are complete and validated, run:
bash
cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish spec

999. Confirm understanding with user before creating task.
9999. Complete specs - no placeholders. All fields listed, all constraints specified.
99999. SPECS ONLY - NO CODE - This phase is for specification authoring only.
999999. ALWAYS document findings immediately in working notes.md.
9999999. If you find inconsistencies in existing code patterns, document them in the spec for resolution during implementation.
99999999. FULL SPECIFICATIONS ONLY. NO PLACEHOLDER SPECS. NO STUB DESCRIPTIONS. NO TODO COMMENTS. WE WANT COMPLETE SPECIFICATIONS. DO IT OR I WILL YELL AT YOU
999999999. WHEN SPECS COMPLETE: run cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish spec to signal spec phase complete.
{issues_mode}
{parallelism_mode}
