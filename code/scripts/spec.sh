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
0a. Study @.agents/code/SPEC.md - **Project specification** (what this project is and does)

0b. Study @.agents/code/AGENTS.md - Build commands and learnings

0c. Study @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns

0d. Study @.agents/code/tasks/{{TASKNAME}}/spec/*.md - All specification files

0e. Study @.agents/code/tasks/{{TASKNAME}}/plan.md - Current task list

1. Your task is to implement {{TASKNAME}} per the specifications in spec/. Study plan.md and choose the **most important 10 items**. Before making changes search codebase (don't assume not implemented) using subagents. You may use up to 100 parallel subagents for all operations but only 1 subagent for build/tests.

2. After implementing functionality or resolving problems, run the tests for that unit of code that was improved. If functionality is missing then it's your job to add it as per the specifications. Think hard.

3. When you discover a bug or issue, immediately update @.agents/code/tasks/{{TASKNAME}}/plan.md with your findings using a subagent. When the issue is resolved, update plan.md and remove the item using a subagent.

4. When the tests pass update plan.md, then add changed code and plan.md with "git add -A" via bash then do a "git commit" with a message that describes the changes you made to the code. After the commit do a "git push" to push the changes to the remote repository.

5. When you discover bugs unrelated to your current work, document them in plan.md immediately then resolve them using subagents before continuing.

---

9. Keep @.agents/code/tasks/{{TASKNAME}}/plan.md up to date with your learnings using a subagent. Especially after wrapping up/finishing your turn.

99. When you learn something new about how to build, test, or run the project make sure you update @.agents/code/AGENTS.md using a subagent but keep it brief.

999. You may add extra logging if required to debug issues.

9999. Single source of truth - no duplicates, no migrations/adapters. If tests unrelated to your work fail then it's your job to resolve these tests as part of the increment of change.

99999. When authoring documentation capture the "why" - why tests exist and why the backing implementation matters. Add README.md next to source code when documenting modules.

999999. If you find inconsistencies in the spec/*.md files then use a subagent to think hard (the oracle) and then update the specs to resolve the inconsistency.

9999999. When plan.md becomes large, periodically clean out the completed items from the file using a subagent.

99999999. For any bugs you notice, document them in plan.md then resolve them using a subagent.

999999999. **FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS, STUBS, OR TODOS.**

9999999999. **DO NOT PUT STATUS UPDATES IN AGENTS.md**

99999999999. **DO ONLY ONE TASK FROM THE PLAN PER SESSION.**
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
