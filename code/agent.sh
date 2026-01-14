#!/bin/bash
# Code agent - Software development workflow
#
# Stages: spec → planning → ready → completed

# Returns all stages in order
agent_stages() {
    echo "spec planning ready completed"
}

# Returns the initial stage for new tasks
agent_initial_stage() {
    echo "spec"
}

# Returns the next stage after the given stage
agent_next_stage() {
    local stage="$1"
    case "$stage" in
        spec)     echo "planning" ;;
        planning) echo "ready" ;;
        ready)    echo "completed" ;;
        *)        echo "" ;;
    esac
}

# Returns human-readable label for stage (used in queue display)
agent_stage_label() {
    local stage="$1"
    case "$stage" in
        spec)      echo "Spec" ;;
        planning)  echo "Planning" ;;
        ready)     echo "Ready" ;;
        completed) echo "Completed" ;;
    esac
}

# Returns the prompt file for a given stage
agent_prompt_for_stage() {
    local stage="$1"
    local taskname="$2"
    local metagent_dir="$HOME/.metagent/code"

    case "$stage" in
        spec)     echo "$metagent_dir/SPEC_PROMPT.md" ;;
        planning) echo "$metagent_dir/PLANNING_PROMPT.md" ;;
        ready)    echo ".agents/code/tasks/$taskname/PROMPT.md" ;;
    esac
}

# Returns message shown when finishing a stage
agent_finish_message() {
    local stage="$1"
    case "$stage" in
        spec)     echo "Spec phase complete." ;;
        planning) echo "Planning phase complete." ;;
        task)     echo "Task complete." ;;
    esac
}

# Returns valid finish stages for this agent
agent_valid_finish_stages() {
    echo "spec planning task"
}

# Creates the task directory structure
agent_create_task() {
    local taskname="$1"
    local task_dir="$2"

    mkdir -p "${task_dir}/spec"

    # Create plan.md template
    cat > "${task_dir}/plan.md" << EOF
# Implementation Plan - ${taskname}

> Generated: $(date +%Y-%m-%d)
> Status: PENDING_SPEC

- [ ] (tasks will be added during planning phase)
EOF

    # Create PROMPT.md (build loop prompt)
    cat > "${task_dir}/PROMPT.md" << EOF
Study these files to understand the project and task:

- @.agents/code/SPEC.md - Project specification
- @.agents/code/AGENTS.md - Build commands and learnings
- @.agents/code/TECHNICAL_STANDARDS.md - Coding patterns
- @.agents/code/tasks/${taskname}/spec/*.md - Task specifications
- @.agents/code/tasks/${taskname}/plan.md - Current task list

---

Implement ${taskname} per the specifications. Study plan.md and choose the most important task. Search codebase first (don't assume not implemented).

After implementing, run tests. If functionality is missing, add it per specifications.

When you discover issues, update plan.md immediately.

When tests pass, commit: \`git add -A && git commit -m "description" && git push\`

---

9. Keep plan.md up to date with learnings.

99. Update AGENTS.md when you learn build/test commands.

999. Single source of truth - no duplicates.

9999. **FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS.**

99999. **ONE TASK PER SESSION.**

999999. **WHEN DONE:** run \`metagent finish\` to signal completion.
EOF

    # Create spec README
    cat > "${task_dir}/spec/README.md" << EOF
# Specifications for ${taskname}

Specification files will be created here during the spec phase.

## Structure

- overview.md - High-level goals and architecture
- types.md - Type definitions and interfaces
- {module}.md - Individual module specifications
- errors.md - Error handling specifications
EOF

    echo ""
    echo "Continue writing specifications to: ${task_dir}/spec/"
}

# Message shown after install
agent_install_message() {
    echo "Next: Run /bootstrap to configure for this project"
}

# Register slash commands for this agent
agent_slash_commands() {
    local prompt_dir="$1"
    local claude_commands="$2"
    local codex_commands="$3"

    echo -e "${BLUE}Installing code slash commands...${NC}"
    for commands_dir in "$claude_commands" "$codex_commands"; do
        ln -sf "$prompt_dir/BOOTSTRAP_PROMPT.md" "$commands_dir/bootstrap.md"
        ln -sf "$prompt_dir/SPEC_PROMPT.md" "$commands_dir/spec.md"
        ln -sf "$prompt_dir/PLANNING_PROMPT.md" "$commands_dir/planner.md"
        ln -sf "$prompt_dir/DEBUG_PROMPT.md" "$commands_dir/debug.md"
    done
    echo -e "  ${GREEN}✓${NC} /bootstrap, /spec, /planner, /debug"
}
