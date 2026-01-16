# RALPH WRITING - Project Initialization

## Purpose

This prompt sets up a new writing project by:
1. Conducting an interview to understand the project
2. Creating the project directory structure
3. Generating all configuration files

**This phase produces project setup. NO content writing.**

---

## PHASE 1: DISCOVERY INTERVIEW

Have a conversation to understand the project. Ask 1-2 questions at a time.

### Project Basics
- What are you writing? (book, article, documentation, blog series, etc.)
- What's the working title?
- What's the core purpose in one sentence?

### Audience
- Who is this for? (Be specific)
- What does your audience already know?
- What do they want to achieve by reading this?
- How will they read it? (cover to cover, reference, skim)

### Scope & Constraints
- Target word count or length?
- What's explicitly OUT of scope?
- Any format requirements? (chapters, sections, specific structure)

### Voice & Style
- What person? (I/we, you, they)
- What tone? (formal, conversational, authoritative, friendly)
- Reading level target?
- Examples of writing you want to sound like?

### Content Structure
- Do you have an outline already, or should we create one?
- Major sections or chapters you envision?
- Logical flow - what comes first, what builds on what?

### Research & Sources
- Do you have research gathered already?
- Key facts or data you know you'll need?

### Special Requirements
- Any terminology that must be used consistently?
- Names, products, or proper nouns to track?

---

## PHASE 2: CONFIRM UNDERSTANDING

Before generating files, summarize back:
- "Here's what I understand about your project..."
- Confirm the structure
- Confirm the voice/style
- Ask if anything is missing

---

## PHASE 3: CREATE TASK

Once requirements are clear, create the task:

```bash
cd "{repo}" && metagent --agent writer task {projectname}
```

This creates the task directory at `.agents/writer/tasks/{projectname}/`.

---

## PHASE 4: GENERATE PROJECT FILES

**First, read `.agents/writer/AGENTS.md`** - This contains project-wide writing conventions and learnings.

Create the complete project structure in `.agents/writer/tasks/{projectname}/`:

```
{projectname}/
├── content/                    # Empty, ready for writing
├── outline/
│   ├── overview.md            # Project overview and goals
│   └── structure.md           # Complete outline
├── style/
│   ├── voice.md               # Voice/style decisions
│   └── terminology.md         # Glossary
├── research/
│   └── notes.md               # Research and sources
└── editorial_plan.md          # Task list
```

### outline/overview.md

```markdown
# {Project Title} - Overview

## Purpose
{One paragraph: What this accomplishes and why it matters}

## Goals
1. {Specific, measurable goal}
2. {Specific, measurable goal}

## Non-Goals
> Explicitly out of scope
1. {Thing this will NOT cover}

## Target Audience
{Detailed description of who will read this}

## Success Criteria
- [ ] {Testable criterion}
- [ ] {Testable criterion}

## Constraints
- Word count: {target}
- Format: {requirements}
```

### outline/structure.md

```markdown
# {Project Title} - Structure

## Overview
{Brief description of organization}

## Sections

### Section 1: {Title}
**Purpose:** {What this section accomplishes}
**Target length:** {word count}
**Key points:**
- {Point 1}
- {Point 2}

### Section 2: {Title}
...

## Writing Order
{Recommended order based on dependencies}
```

### style/voice.md

```markdown
# Voice & Style Guide

## Person
{First/second/third with examples}

## Tone
{Description with examples}

## Formality
{Level and guidelines}

## Good Examples
> {Example of correct voice}

## Bad Examples
> {Example of what to avoid}

## Formatting
- Headings: {style}
- Lists: {when to use}
- Code/examples: {format}
```

### style/terminology.md

```markdown
# Terminology Guide

## Core Terms

| Term | Definition | Usage Notes |
|------|------------|-------------|
| {term} | {definition} | {notes} |

## Capitalization Rules
- {rule}

## Avoid These Terms
- {term}: Use {alternative} instead
```

### research/notes.md

```markdown
# Research Notes

## Key Facts
- {fact with source}

## Sources
- {source}

## To Research
- {topic needing research}
```

### editorial_plan.md

```markdown
# Editorial Plan - {Project Title}

> Generated: {DATE}

## Current Task
**Section 1, Page 1: {Topic}**

## Module Status

| Section | Status | Progress | Notes |
|---------|--------|----------|-------|
| 1. {Title} | Not started | 0/N | - |
| 2. {Title} | Not started | 0/N | - |

## Current Section Plan

**Section 1: {Title}**
> Status: Planning required

Run PLANNING_PROMPT to generate page breakdown.

## Issues & Blockers
(none yet)
```

---

## PHASE 5: COMPLETION

After all files are generated:

1. **Summarize what was created**
2. **Explain next steps:**
   - Review the outline - adjust if needed
   - Review the style guide - add examples
   - Run planning prompt for first section
   - Then start writing loops

3. **Signal completion:**

```bash
cd "{repo}" && METAGENT_TASK="{task}" metagent --agent writer finish init --session "{session}"
```

---

## RULES

1. **Interview first** - Understand before creating
2. **Confirm understanding** - Summarize back to user
3. **Complete files** - No placeholders
4. **Use metagent task** - Creates proper directory structure

## PRIORITY

9. Conduct thorough interview
99. Confirm understanding before generating
999. Create all files with actual content

9999. **SETUP ONLY - NO CONTENT WRITING**
99999. **COMPLETE FILES - NO PLACEHOLDERS**
