# RALPH WRITING - Section Planning

## Workflow Context

The writing workflow cycles between planning and writing:

```
plan → write → write → write → plan → write → write → plan → ... → done
```

This prompt runs at the **plan** stage. After you finish, the orchestrator switches to the writing prompt to write pages. When all pages in a section are done, the writing prompt signals `--next plan` to return here for the next section.

## Purpose

Run this prompt BEFORE writing each section. It will:
1. Research the section's topics online
2. Generate a page breakdown
3. Update editorial_plan.md with the page plan
4. STOP

**This is PLANNING ONLY. Do not write content.**

---

## CONTEXT LOADING

Study these files first:
- @.agents/writer/AGENTS.md - Writing tools and learnings
- @.agents/writer/tasks/{task}/outline/overview.md - Project goals and audience
- @.agents/writer/tasks/{task}/outline/structure.md - Section outline (source of truth)
- @.agents/writer/tasks/{task}/style/voice.md - Voice and tone guidelines
- @.agents/writer/tasks/{task}/style/terminology.md - Glossary
- @.agents/writer/tasks/{task}/research/notes.md - Existing research
- @.agents/writer/tasks/{task}/editorial_plan.md - Current progress

---

## STEP 1: IDENTIFY CURRENT SECTION

Check editorial_plan.md to find which section needs planning next.
Read that section's details from outline/structure.md.

---

## STEP 2: RESEARCH THE SECTION TOPICS

Use up to 3 parallel subagents to research the section's content.

For each topic in the section:
- Official documentation and sources
- Current facts, data, examples
- How experts discuss these topics
- Common misconceptions or pain points

Return:
- Key facts with source URLs
- Current accurate information
- What we need to get technically right

---

## STEP 3: GENERATE PAGE BREAKDOWN

Based on research and the outline, break the section into pages (~1000 words each).

Consider:
- What order makes pedagogical sense?
- Which topics need more depth based on research?
- Where should examples go?
- How to balance conceptual vs. practical content?

Each page should have:
- Clear topic/focus
- Connection to outline section
- Rough sense of what it covers

---

## STEP 4: UPDATE EDITORIAL_PLAN.MD

Replace the "Current Section Plan" with:

```markdown
## Current Section Plan

**Section N: {Title}**
> Generated: {DATE}
> Based on outline and research

### Page Breakdown

- [ ] Page 1: {Topic} - {Brief description}
- [ ] Page 2: {Topic} - {Brief description}
- [ ] Page 3: {Topic} - {Brief description}
...

### Research Summary

Key findings that inform this section:
- {Finding 1}
- {Finding 2}

### Facts to Get Right

- {Specific fact/detail to verify}
- {Source to cite}
```

---

## STEP 5: APPEND TO RESEARCH NOTES

Add essential new findings to research/notes.md (append only, one-liners):

```markdown
- {DATE} {FINDING} (source: URL)
```

---

## STEP 6: UPDATE SECTION STATUS

In editorial_plan.md, update the section status:

```markdown
| 1. Introduction | Planning complete | 0/N | - |
```

Also update "Current Task" to:

```markdown
## Current Task

**Section N, Page 1: {Topic}**
```

---

## STEP 7: SIGNAL COMPLETION

```bash
cd "{repo}" && METAGENT_TASK="{task}" metagent --agent writer finish plan
```

---

## STOP

After completing these steps, STOP. Do not begin writing content.

The next loop will automatically use the writing prompt to write Page 1.

---

## OUTPUT CHECKLIST

Before stopping, verify:
- [ ] Section topics researched via subagents
- [ ] Page breakdown generated and makes sense
- [ ] editorial_plan.md updated with page plan
- [ ] research/notes.md appended with key findings (brief)
- [ ] Section status updated to "Planning complete"
- [ ] Current Task points to Page 1

---

## SELF-IMPROVEMENT

When you discover useful patterns during planning:
- **Research techniques → AGENTS.md** - Effective search strategies, sources
- **Planning patterns → AGENTS.md** - Page breakdown approaches that work

---

## RULES

1. **Research first** - Use subagents to gather information
2. **Plan thoroughly** - Each page should have clear purpose
3. **Update editorial_plan.md** - This is source of truth for writing
4. **Append to research** - Don't rewrite, just add
5. **Update AGENTS.md** - Document effective patterns

## PRIORITY

9. Research the section topics thoroughly
99. Generate sensible page breakdown
999. Update editorial_plan.md completely

9999. **PLANNING ONLY - NO WRITING**
99999. **STOP AFTER UPDATING EDITORIAL_PLAN**
