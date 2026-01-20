# RALPH WRITING LOOP

## Workflow Context

The writing workflow cycles between planning, writing, and editing:

```
plan → write → edit → write → edit → write → edit → plan → ...
```

This prompt runs at the **write** stage. You write one page per loop. After writing:
- Always advance to **edit** stage (the editor will humanize the page and decide what's next)

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
5. Update editorial_plan.md, save progress, **STOP**

**STOP AFTER COMPLETING ONE PAGE. Do not continue to the next page.**

---

## Subagent Configuration

**Parallelism limits:**
- Online research: up to 3 parallel subagents
- Content search/reading: up to 3 parallel subagents
- Writing: 1 subagent only (one page at a time)
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



## On Success - Save Progress

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

**YOU MUST RUN THIS COMMAND before stopping.** The orchestrator depends on this signal.

After writing and saving the page, always advance to the edit stage:

```bash
cd "{repo}" && METAGENT_TASK="{task}" metagent --agent writer finish write --session "{session}"
```

The edit stage will humanize the page and determine what comes next (more writing, planning, or completion).

**RUN THE COMMAND NOW using Bash tool, then STOP.**

---

## Priority Rules

0. **Research online FIRST** - Use subagents to get current information
1. Study outline, style guide, and editorial_plan before starting
2. Identify current page number
3. Research existing content before writing
4. Write ONE page according to outline and style
5. Save
6. **STOP**

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
