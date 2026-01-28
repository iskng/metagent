0a. Study @.agents/code/tasks/{task}/spec/ - All specification files
0b. Study @.agents/code/SPEC.md - Project context
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow

1. Review each {task}/spec/ file for: Completeness (all requirements defined? missing edge cases?), Clarity (unambiguous? testable?), Consistency (conflicts between specs? contradictory requirements?), Feasibility (technically possible? dependencies identified?), Scope (well-bounded? scope creep?) Correctness (are there any issues with the spec?). For each part of the spec research the relevant existing code. 

2. Flag an issue ONLY IF: it would block or derail planning/implementation, the spec author would want to know before proceeding, it's actionable (can be resolved with more information or a decision).

3. Update the spec to fix any issues with the best long term approach as long as you are confident. Also add anything that is missing or update anything that could use better clarity or needs improvement. 

4. If there are any key decisions that the spec author would want to know about but you dont have high confidence in the correct answer think really hard about the best options and present them. 

5. Use this to signal next stage only if you can resolve all of the issues: `cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish spec-review --session "{session}"`

999. If you are in doubt about any aspect of the spec then research the existing code or any relevant material until it is clear.
9999. If you can not alleviate doubt, do not signal finish - research and present the key decisions.
99999. Think really hard about what impact this will have on any downstream functionality, is this breaking change, how significant is this, how risky is this. 
999999. Focus on what blocks implementation, not stylistic preferences.
9999999. Fully research all relevant code before editing any aspect of the spec.  
{issues_mode}

{parallelism_mode}
