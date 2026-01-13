#!/bin/bash
# spec.sh - Bootstrap a new task directory for Ralph workflow
#
# Usage: .agents/code/scripts/spec.sh <taskname>
#
# Creates:
#   .agents/code/<taskname>/
#   ├── spec/
#   │   └── .gitkeep
#   ├── plan.md (template)
#   └── PROMPT.md (template)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Validate arguments
if [ -z "$1" ]; then
    echo -e "${RED}Error: Task name required${NC}"
    echo "Usage: .agents/code/scripts/spec.sh <taskname>"
    echo ""
    echo "Examples:"
    echo "  .agents/code/scripts/spec.sh auth-system"
    echo "  .agents/code/scripts/spec.sh api-endpoints"
    echo "  .agents/code/scripts/spec.sh database-migration"
    exit 1
fi

TASKNAME="$1"
TASK_DIR=".agents/code/tasks/${TASKNAME}"

sed_inplace() {
    local expr="$1"
    local file="$2"
    if sed --version >/dev/null 2>&1; then
        sed -i "$expr" "$file"
    else
        sed -i '' "$expr" "$file"
    fi
}

# Check if task already exists
if [ -d "$TASK_DIR" ]; then
    echo -e "${YELLOW}Warning: Task directory already exists: ${TASK_DIR}${NC}"
    read -p "Overwrite templates? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Create directory structure
echo -e "${BLUE}Creating task directory: ${TASK_DIR}${NC}"
mkdir -p "${TASK_DIR}/spec"
mkdir -p ".agents/code/tasks"  # Ensure tasks parent exists

# Create spec/.gitkeep
touch "${TASK_DIR}/spec/.gitkeep"

# Create plan.md template
cat > "${TASK_DIR}/plan.md" << 'PLAN_EOF'
# Implementation Plan - {{TASKNAME}}

> Generated: {{DATE}}
> Status: PENDING_SPEC | READY | IN_PROGRESS | COMPLETE

## Overview

> Brief description of what this task accomplishes

{{OVERVIEW}}

## Dependencies

> What must exist before this task can begin

- [ ] {{DEPENDENCY}}

## Critical (Blocking)

> Items that block other work - do these first

- [ ] {{CRITICAL_ITEM}} (complexity: S/M/L)
  - Blocks: {{what_it_blocks}}

## High Priority (Core Functionality)

> Main features from specifications

- [ ] {{HIGH_ITEM}} per spec/{{file}}.md (complexity: M)

## Medium Priority

> Important but not blocking

- [ ] {{MEDIUM_ITEM}} (complexity: M)

## Low Priority (Polish)

> Nice to have, defer if needed

- [ ] Documentation
- [ ] Performance optimization

## Discovered Issues

> Found during implementation - add here as discovered

## Placeholders to Replace

> Minimal implementations needing full implementation

## Completed

> Move items here when done

## Notes

> Decisions, context, learnings
PLAN_EOF

# Replace placeholders in plan.md
sed_inplace "s/{{TASKNAME}}/${TASKNAME}/g" "${TASK_DIR}/plan.md"
sed_inplace "s/{{DATE}}/$(date +%Y-%m-%d)/g" "${TASK_DIR}/plan.md"

# Create PROMPT.md template (build loop prompt)
cat > "${TASK_DIR}/PROMPT.md" << 'PROMPT_EOF'
# RALPH BUILD LOOP - {{TASKNAME}}

## Context Stack (Load Every Loop)

> Load these files in order for consistent knowledge

0a. Study @.agents/code/AGENTS.md - Build commands and learnings
0b. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns
0c. Study @.agents/code/tasks/{{TASKNAME}}/spec/*.md - All specification files
0d. Study @.agents/code/tasks/{{TASKNAME}}/plan.md - Current task list

## Primary Directive

You are implementing: {{TASKNAME}}

Each loop:
1. Study plan.md, choose the **most important incomplete item**
2. Research using subagents before implementing (NEVER assume not implemented)
3. Implement according to specifications in spec/
4. Run tests via single subagent
5. If tests pass: commit, update plan.md, push
6. If tests fail: fix them or document in plan.md

## Subagent Configuration

> Primary context = SCHEDULER. Heavy work = SUBAGENTS.

**Allowed parallelism:**
- Codebase search: up to 100 parallel subagents
- File reading: up to 100 parallel subagents
- File writing: up to 50 parallel subagents (independent files only)
- Build/test: **1 SUBAGENT ONLY**
- plan.md updates: 1 subagent
- AGENTS.md updates: 1 subagent

## Research Protocol (CRITICAL)

> Prevent duplicate implementations - ripgrep is non-deterministic

Before implementing ANYTHING:

1. **Exhaustive search** (minimum 5 different terms):
   - Exact names, partial matches, synonyms, concepts
   - Check: {UPDATE THIS WITH RELEVANT TERMS}

2. **DO NOT ASSUME** code doesn't exist because:
   - One search returned empty
   - It's not in obvious places

3. **Think hard** about alternative locations/names

## Implementation Standards

1. **One item per loop** - Most important first
2. **Follow specs exactly** - spec/*.md is source of truth
3. **Follow technical standards** - @.agents/code/TECHNICAL_STANDARDS.md
4. **Full implementations only** - NO placeholders, stubs, TODOs
5. **Single source of truth** - No duplicates

## Back Pressure - Testing

After implementing:

1. Run unit tests for changed code (1 subagent)
2. If pass, run related tests (1 subagent)
3. Document test results
4. Fix ALL failures before committing

## Success Protocol

When tests pass:

1. Update plan.md  - mark complete, add discoveries
2. Commit:
   ```bash
   git add -A
   git commit -m "{{TASKNAME}}: {description}"
   git push
   ```

## Failure Protocol

1. Document in plan.md
2. Attempt to resolve if within scope
3. If unresolvable: capture in plan.md, move to next item

## Self-Improvement

1. Build learnings → @.agents/code/AGENTS.md
2. Bugs → plan.md
3. Spec inconsistencies → Update spec files

## Priority Rules

0. Study all specs and plan before starting
1. Choose ONE item from plan.md
2. Research exhaustively before implementing
3. Implement fully (no placeholders)
4. Test the implementation
5. Commit only if tests pass

9. Keep plan.md updated with findings
99. Keep AGENTS.md updated with learnings
999. Search thoroughly - don't assume missing
9999. Resolve ALL test failures before committing
99999. Single source of truth - no duplicates

999999. **FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS.**
9999999. **DO NOT PUT STATUS UPDATES IN AGENTS.md**
PROMPT_EOF

# Replace placeholders in PROMPT.md
sed_inplace "s/{{TASKNAME}}/${TASKNAME}/g" "${TASK_DIR}/PROMPT.md"

# Create a spec template readme
cat > "${TASK_DIR}/spec/README.md" << 'SPEC_README_EOF'
# Specifications for {{TASKNAME}}

This directory contains the specifications for this task.

## Files

Specification files will be created here during the spec phase.

Typical structure:
- `overview.md` - High-level goals and architecture
- `{module}.md` - Individual module specifications
- `types.md` - Type definitions and interfaces
- `errors.md` - Error handling specifications

## Spec File Template

Each spec file should include:
- Purpose
- Dependencies
- Public interface (types, functions)
- Edge cases
- Testing requirements
SPEC_README_EOF

sed_inplace "s/{{TASKNAME}}/${TASKNAME}/g" "${TASK_DIR}/spec/README.md"

echo -e "${GREEN}✓ Created task directory: ${TASK_DIR}${NC}"
echo ""
echo "Created files:"
echo "  ${TASK_DIR}/spec/README.md"
echo "  ${TASK_DIR}/plan.md"
echo "  ${TASK_DIR}/PROMPT.md"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Run spec phase:   cat .agents/code/SPEC_PROMPT.md | claude-code"
echo "   (Tell it to work on task: ${TASKNAME})"
echo ""
echo "2. Run planning:     cat .agents/code/PLANNING_PROMPT.md | claude-code"
echo "   (Point it at: ${TASKNAME})"
echo ""
echo "3. Run build loop:   while :; do cat ${TASK_DIR}/PROMPT.md | claude-code; done"
