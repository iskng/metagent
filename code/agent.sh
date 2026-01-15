#!/bin/bash
# Code agent - Software development workflow
#
# Stages: spec → planning → build → completed

# Returns all stages in order
agent_stages() {
    echo "spec planning build completed"
}

# Returns stages orchestrated by metagent start
agent_orchestrated_stages() {
    echo "spec planning"
}

# Returns the stage where metagent start should hand off to run/run-queue
agent_handoff_stage() {
    echo "build"
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
        planning) echo "build" ;;
        build)    echo "completed" ;;
        task)     echo "completed" ;;
        *)        echo "" ;;
    esac
}

# Returns human-readable label for stage (used in queue display)
agent_stage_label() {
    local stage="$1"
    case "$stage" in
        spec)      echo "Spec" ;;
        planning)  echo "Planning" ;;
        build)     echo "Build" ;;
        completed) echo "Completed" ;;
    esac
}

# Returns the prompt file for a given stage
agent_prompt_for_stage() {
    local stage="$1"
    local taskname="$2"
    local metagent_dir="$HOME/.metagent/code"
    local repo_root="${METAGENT_REPO_ROOT:-$(pwd)}"

    case "$stage" in
        spec)     echo "$metagent_dir/SPEC_PROMPT.md" ;;
        planning) echo "$metagent_dir/PLANNING_PROMPT.md" ;;
        build)    echo "$repo_root/.agents/code/tasks/$taskname/PROMPT.md" ;;
    esac
}

# Returns message shown when finishing a stage
agent_finish_message() {
    local stage="$1"
    case "$stage" in
        spec)     echo "Spec phase complete." ;;
        planning) echo "Planning phase complete." ;;
        build)    echo "Build iteration complete." ;;
        task)     echo "Task complete." ;;
    esac
}

# Returns valid finish stages for this agent
agent_valid_finish_stages() {
    echo "spec planning build task"
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
0a. Study all files in @.agents/code/tasks/${taskname}/spec/- Task specifications and architecture
0b. Study @.agents/code/TECHNICAL_STANDARDS.md - Codebase patterns to follow
0c. Study @.agents/code/tasks/${taskname}/plan.md - Current task list
0d. Study @.agents/code/AGENTS.md - Build/test commands and learnings

1. Your task is to implement ${taskname} per the specifications. study @plan.md, choose the most important uncompleted items that you can accomplish in one pass (max 5), research using subagents before implementing (NEVER assume code doesn't exist), implement according to specifications.

2. After implementing functionality or resolving problems, run the tests for that unit of code that was improved. If functionality is missing then it's your job to add it as per the application specifications.

3. When the tests pass update @plan.md, then add changed code and @plan.md with git add the relevant files you created/modified via bash then do git commit -m "feat(${taskname}): [descriptive message]"

4. ALWAYS KEEP @plan.md up to date with your learnings about the task using a subagent. Especially after wrapping up/finishing your turn.

5. When you learn something new about how to run the build/tests make sure you update @.agents/code/AGENT.md but keep it brief.


999999. Important: We want single sources of truth, no migrations/adapters. If tests unrelated to your work fail then it's your job to resolve these tests as part of the increment of change.
99999999. Important: When authoring tests capture the WHY - document importance in docstrings.
999999999. IMPORTANT: When you discover a bug resolve it using subagents even if it is unrelated to the current piece of work after documenting it in @plan.md
9999999999. You may add extra logging if required to be able to debug the issues.
99999999999. When @plan.md becomes large periodically clean out the items that are completed from the file using a subagent.
999999999999. If you find inconsistencies in the specs/* then use the oracle (think extra hard) and then update the specs.
9999999999999. Allowed parallelism: Codebase search up to 50 subagents, file reading up to 50 subagents, file writing up to 10 subagents (independent files only), build/test 1 SUBAGENT ONLY, plan.md updates 1 subagent.
99999999999999. FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS. NO STUBS. NO TODO COMMENTS. DO NOT IMPLEMENT PLACEHOLDER OR SIMPLE IMPLEMENTATIONS. WE WANT FULL IMPLEMENTATIONS. DO IT OR I WILL YELL AT YOU
999999999999999. SUPER IMPORTANT DO NOT IGNORE. DO NOT PLACE STATUS REPORT UPDATES INTO @.agents/code/AGENT.md
9999999999999999. **WHEN ITEM DONE:** run \`cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish --next build\` to signal iteration complete (more items remain).
99999999999999999. **WHEN ALL ASPECTS OF THE PLAN.md ARE COMPLETE:** run a full \`cargo build\` to verify everything compiles, then run \`cd "{repo}" && METAGENT_TASK="{task}" metagent --agent code finish\` to signal task complete (all items done).
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
