#!/bin/bash
# Writer agent - Writing project workflow
#
# Stages: init → plan → write → completed
# The plan/write stages cycle: after planning a section, write its pages,
# then use --next plan to go back and plan the next section.

# Returns all stages in order
agent_stages() {
    echo "init plan write completed"
}

# Returns the initial stage for new tasks
agent_initial_stage() {
    echo "init"
}

# Returns the next stage after the given stage
agent_next_stage() {
    local stage="$1"
    case "$stage" in
        init)  echo "plan" ;;
        plan)  echo "write" ;;
        write) echo "completed" ;;
        *)     echo "" ;;
    esac
}

# Returns human-readable label for stage (used in queue display)
agent_stage_label() {
    local stage="$1"
    case "$stage" in
        init)      echo "Init" ;;
        plan)      echo "Plan" ;;
        write)     echo "Write" ;;
        completed) echo "Completed" ;;
    esac
}

# Returns the prompt file for a given stage
agent_prompt_for_stage() {
    local stage="$1"
    local taskname="$2"
    local metagent_dir="$HOME/.metagent/writer"

    case "$stage" in
        init)  echo "$metagent_dir/INIT_PROMPT.md" ;;
        plan)  echo "$metagent_dir/PLANNING_PROMPT.md" ;;
        write) echo "$metagent_dir/PROMPT.md" ;;
    esac
}

# Returns message shown when finishing a stage
agent_finish_message() {
    local stage="$1"
    case "$stage" in
        init)  echo "Init phase complete." ;;
        plan)  echo "Section planned." ;;
        write) echo "Page complete." ;;
    esac
}

# Returns valid finish stages for this agent
agent_valid_finish_stages() {
    echo "init plan write"
}

# Creates the task directory structure
agent_create_task() {
    local taskname="$1"
    local task_dir="$2"

    # Create writer project structure
    mkdir -p "${task_dir}/content"
    mkdir -p "${task_dir}/outline"
    mkdir -p "${task_dir}/style"
    mkdir -p "${task_dir}/research"

    # Create editorial_plan.md template
    cat > "${task_dir}/editorial_plan.md" << EOF
# Editorial Plan - ${taskname}

> Generated: $(date +%Y-%m-%d)
> Status: Awaiting project setup

## Current Task

Run /writer-init to set up the project.

## Section Status

| Section | Status | Progress | Notes |
|---------|--------|----------|-------|
| (sections added after init) | - | - | - |

## Issues & Blockers

(none yet)
EOF

    echo ""
    echo "Run /writer-init to set up your writing project."
}

# Message shown after install
agent_install_message() {
    echo "Next: Run /writer-init to set up your writing project"
}

# Register slash commands for this agent
agent_slash_commands() {
    local prompt_dir="$1"
    local claude_commands="$2"
    local codex_commands="$3"

    echo -e "${BLUE}Installing writer slash commands...${NC}"
    for commands_dir in "$claude_commands" "$codex_commands"; do
        ln -sf "$prompt_dir/INIT_PROMPT.md" "$commands_dir/writer-init.md"
        ln -sf "$prompt_dir/PLANNING_PROMPT.md" "$commands_dir/writer-plan.md"
        ln -sf "$prompt_dir/PROMPT.md" "$commands_dir/writer.md"
    done
    echo -e "  ${GREEN}✓${NC} /writer-init, /writer-plan, /writer"
}
