# RALPH WRITING LOOP

## Workflow Context

The writing workflow cycles between planning and writing:

```
plan → write → write → write → plan → write → write → plan → ... → done
```

This prompt runs at the **write** stage. You write one page per loop. After each page:
- More pages in section? → `--next write` (stay in write stage)
- Section complete, more sections? → `--next plan` (cycle back to planning)
- All done? → no flag (advance to completed)

---

## Context Stack (Loaded Every Loop)

Load and study these files before starting:

- @.agents/writer/tasks/{task}/outline/overview.md - Project overview, goals, audience
- @.agents/writer/tasks/{task}/outline/structure.md - Full structural outline
- @.agents/writer/tasks/{task}/style/voice.md - Voice, tone, and style guidelines
- @.agents/writer/tasks/{task}/style/terminology.md - Glossary and terminology
- @.agents/writer/tasks/{task}/research/notes.md - Research findings
- @.agents/writer/tasks/{task}/editorial_plan.md - Current task list
- @.agents/writer/AGENTS.md - Writing tools and learnings

---

## Primary Directive

**ONE PAGE PER LOOP. THEN STOP.**

Your task each loop:
1. Study editorial_plan.md and identify the current page
2. Research the topic online using subagents BEFORE writing
3. Research existing content using subagents (don't duplicate, stay consistent)
4. Write ONE page (~1000 words) according to outline and style guide
5. Run editorial checks via single subagent
6. If checks pass: update editorial_plan.md, save progress, **STOP**
7. If checks fail: fix issues, then **STOP**

**STOP AFTER COMPLETING ONE PAGE. Do not continue to the next page.**

---

## Subagent Configuration

**Parallelism limits:**
- Online research: up to 3 parallel subagents
- Content search/reading: up to 3 parallel subagents
- Writing: 1 subagent only (one page at a time)
- Editorial checks: 1 subagent only
- editorial_plan.md updates: 1 subagent
- AGENTS.md updates: 1 subagent

---

## CRITICAL: Research Before Writing

**BEFORE writing ANY page, you MUST use subagents to research online.**

### For each page, research:

1. **Current information**
   - Latest facts, data, examples
   - Use WebSearch and WebFetch to get current information

2. **Technical accuracy verification**
   - Verify facts are current
   - Check for updates or changes
   - Confirm accuracy

### Update research notes (APPEND ONLY)

After research, append SMALL additions to research/notes.md:
- One-liner facts with source URL
- Corrections to existing assumptions

**Keep additions minimal** - only add what's essential for future reference.

---

## Research Existing Content Check

Before writing ANY page:

1. **Search existing content using subagents:**
   - What have we already said about this topic?
   - What terminology have we established?
   - What tone/voice have we used in similar sections?
   - Are there any contradictions to avoid?

2. **Check continuity:**
   - Terminology matches glossary?
   - Facts consistent with research?
   - Cross-references accurate?

3. **Review outline:**
   - Does this page match the outlined purpose?
   - Are we covering what was planned?
   - Are we staying within scope?

---

## Writing Standards

1. **ONE PAGE PER LOOP** - Write ~1000 words, then stop
2. **Research first** - Online research REQUIRED before writing
3. **Follow outline exactly** - outline/structure.md is source of truth
4. **Follow style guide** - style/* defines voice and conventions
5. **Match research** - research/* provides facts and sources
6. **Maintain consistency** - Terminology, voice, and facts must align

---

## Back Pressure - Editorial Checks

After writing the page, run checks via single subagent:

1. **Accuracy check:**
   - Are facts accurate and sourced?
   - Any claims that need verification?

2. **Style compliance check:**
   - Voice and tone match style/voice.md?
   - Terminology matches style/terminology.md?
   - Formatting follows conventions?

3. **Consistency check:**
   - No contradictions with existing content?
   - Terminology used consistently?
   - Cross-references accurate?

4. **Quality check:**
   - Grammar and spelling correct?
   - Sentences clear and readable?
   - Examples complete?

---

## On Success - Save Progress

When editorial checks pass:

1. **Update editorial_plan.md** using subagent:
   - Mark page complete
   - Note current progress
   - Add any new items discovered

2. **Append to research/notes.md** if essential new facts discovered

3. **Save the content**

4. **STOP** - Do not continue to next page

---

## On Issues - Document and Continue

When problems are found:

1. **Document in editorial_plan.md** using subagent:
   - Add to appropriate section
   - Note the file and problem

2. **Attempt to resolve** if within scope

3. **STOP** after resolving or documenting

---

## Self-Improvement

1. **Writing learnings → AGENTS.md**
   When you discover effective patterns, update AGENTS.md using a subagent.

2. **Style clarifications → style guide**
   If you make a style decision not covered by the guide, document it.

3. **Research discoveries → research notes**
   If you find essential new information, append a one-liner to research/notes.md.

---

## Signal Completion

**YOU MUST RUN ONE OF THESE COMMANDS before stopping.** The orchestrator depends on this signal.

Check editorial_plan.md to determine which command to run:

1. **More pages in current section?** (unchecked `- [ ]` pages exist)
   ```bash
   cd "{repo}" && METAGENT_TASK="{task}" metagent --agent writer finish write --session "{session}" --next write
   ```

2. **Section complete, more sections to plan?**
   ```bash
   cd "{repo}" && METAGENT_TASK="{task}" metagent --agent writer finish write --session "{session}" --next plan
   ```

3. **All sections complete?**
   ```bash
   cd "{repo}" && METAGENT_TASK="{task}" metagent --agent writer finish write --session "{session}"
   ```

**RUN THE APPROPRIATE COMMAND NOW using Bash tool, then STOP.**

---

## Priority Rules

0. **Research online FIRST** - Use subagents to get current information
1. Study outline, style guide, and editorial_plan before starting
2. Identify current page number
3. Research existing content before writing
4. Write ONE page according to outline and style
5. Run editorial checks
6. Save only if checks pass
7. **STOP**

9. Keep editorial_plan.md updated with findings
99. Keep AGENTS.md updated with learnings
999. Check existing content thoroughly - don't contradict yourself
9999. Maintain consistent voice throughout
99999. Stay within outlined scope - no tangents

999999. Follow the outline structure. The outline is the spec. Do not deviate.

9999999. **WRITE ONE COMPLETE PAGE (~1000 WORDS). NO PLACEHOLDER TEXT. THEN STOP.**

99999999. **ACCURACY IS NON-NEGOTIABLE. RESEARCH BEFORE WRITING.**

999999999. **MAINTAIN VOICE CONSISTENCY. Read existing content before writing.**

9999999999. **STOP AFTER ONE PAGE. DO NOT CONTINUE TO THE NEXT PAGE.**
