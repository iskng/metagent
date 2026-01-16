0a. Study @.agents/code/SPEC.md - Project specification (what this project is and does)
0b. Study @.agents/code/AGENTS.md - Project build commands and structure
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
0d. If task exists, check @.agents/code/tasks/{task}/issues.md - If it has open Spec Issues, these MUST be addressed first

1. **PRIORITY: Spec Issues** - If issues.md exists with open Spec Issues (from code review), address those FIRST. These are requirements clarifications or architectural decisions needed. Update issue status to "resolved" when addressed.

2. Your task is to gather requirements through conversation and research of existing code and create a spec. Ask 2-4 batches of questions - don't overwhelm. Questions like: "What problem does this solve?", "What does success look like?", "What are explicit NON-goals?". Don't ask questions you already have answers to.
3. After each set of answers spawn subagents to research the codebase or search online. Make sure to clarify any key decisions that you are not sure about. Document answers immediately in working notes.md
4. Summarize understanding back to user, probe ambiguous areas, continue until user confirms understanding is complete.
5. Once you understand the problem, scope, boundaries, and key requirements, create the task using: cd "{repo}" && metagent --agent code task {taskname} this will create the .agents/code/tasks/{taskname}/spec/ dir for you.
6. Use subagents (up to 50 parallel) to explore codebase. Find: APIs called/exposed, modules imported/importing, shared types, database tables, external services. Analyze: error handling patterns, logging patterns, testing patterns, validation patterns, naming conventions, similar implementations.
7. Author detailed specs in .agents/code/tasks/{taskname}/spec/ including: overview.md (purpose, goals, non-goals, architecture, dependencies, success criteria), types.md (complete type definitions with fields, invariants, examples), modules.md (logical modules with public interface, parameters, errors, edge cases), errors.md (error types, when/contains/recovery).
8. Before completing spec phase verify: Types are EXACT, Signatures are COMPLETE, Examples are CONCRETE, Edge cases are EXHAUSTIVE, Errors are SPECIFIC, Dependencies are MAPPED, Success criteria are TESTABLE.
9. After specs are complete and validated, run:
bash
cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish spec

999. Confirm understanding with user before creating task.
9999. Complete specs - no placeholders. All fields listed, all constraints specified.
99999. SPECS ONLY - NO CODE - This phase is for specification authoring only.
999999. Allowed parallelism: Codebase search up to 50 subagents, file reading up to 50 subagents.
9999999. ALWAYS document findings immediately in working notes.md using a subagent.
9999999999. If you find inconsistencies in existing code patterns, document them in the spec for resolution during implementation.
99999999999. FULL SPECIFICATIONS ONLY. NO PLACEHOLDER SPECS. NO STUB DESCRIPTIONS. NO TODO COMMENTS. WE WANT COMPLETE SPECIFICATIONS. DO IT OR I WILL YELL AT YOU
999999999999. WHEN SPECS COMPLETE: run cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish spec to signal spec phase complete.
{issues_mode}
