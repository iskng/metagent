0a. Study @.agents/code/tasks/{task}/spec/ - Task specifications (what was supposed to be built)
0b. Study @.agents/code/tasks/{task}/plan.md - Implementation plan
0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns to follow
{focus_section}

1. Find all commits for this task: `git log --oneline --grep="{task}"`. For each commit, review the full diff with `git show <hash>`.

2. Review each commit for: Spec compliance (matches requirements? missing features? scope creep?), Code quality (follows patterns? duplication? naming?), Correctness (edge cases? bugs? race conditions?), Security (hardcoded secrets? input validation? injection?), Testing (tests exist? meaningful? cover edge cases?), Performance (N+1 queries? unnecessary loops? memory leaks?).

3. Flag an issue ONLY IF: it meaningfully impacts accuracy/performance/security/maintainability, is discrete and actionable, fixing it doesn't demand rigor absent from rest of codebase, was introduced in THIS commit (not pre-existing), author would likely fix if aware, doesn't rely on unstated assumptions about codebase or intent, other affected code is provably identified (no speculation), is clearly not an intentional change by the author. Ignore trivial style unless it violates documented standards.

4. Output ALL findings the author would definitely want to fix. Do not stop at the first qualifying finding - continue until every qualifying finding is listed. If no finding qualifies, output none.

5. Create/update @.agents/code/tasks/{task}/issues.md with findings. For each issue: clear problem description, file path and line number, concrete suggested fix (code snippets ≤3 lines). Priority: [P0] blocking/universal, [P1] urgent, [P2] normal, [P3] low.

Format:
# Code Review - {task}
> Reviewed: {date}
> Commits: {hashes}
> Verdict: PASS | NEEDS BUILD FIXES | NEEDS SPEC CLARIFICATION

## Spec Issues
Issues requiring spec phase (requirements/architecture decisions).
### [P1] Issue title
- **Problem:** {description}
- **Decision needed:** {what to decide}
- **Status:** open

## Build Issues
Issues requiring build phase (implementation fixes).
### [P2] Issue title
- **File:** {path}:{line}
- **Commit:** {hash}
- **Problem:** {description}
- **Suggested fix:** {concrete fix}
- **Status:** open

## Suggestions
Optional improvements (not blocking).

6. Categorize each finding: Spec Issues = missing/unclear requirements, architectural decisions needed, scope questions, wrong approach. Build Issues = bugs, code quality, missing tests, security flaws, performance problems. When in doubt, it's a build issue (easier to fix).

7. Signal next stage:
- Spec issues exist (any open): `cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish review --next spec`
- Only build issues (no spec issues): `cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish review --next build`
- Pass (no issues): `cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish review`

999. Spec issues = requirements/architecture → back to spec.
9999. Build issues = implementation → back to build.
99999. When in doubt, it's a build issue (easier to fix).
999999. One issue per distinct problem.
9999999. ONLY flag bugs introduced in this task's commits - pre-existing bugs should NOT be flagged.
99999999. NO SPECULATION - other affected code must be provably identified, not guessed.
999999999. Don't rely on unstated assumptions about codebase or author's intent.
9999999999. Allowed parallelism: Codebase search up to 50 subagents, file reading up to 50 subagents.
{issues_mode}
