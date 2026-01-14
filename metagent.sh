#!/bin/bash
# metagent.sh - Agent workflow manager
#
# Usage:
#   metagent install                     Setup metagent globally (first time)
#   metagent uninstall                   Remove metagent globally
#   metagent init [path]                 Initialize agent in repo
#   metagent start                       Start new task (interview → spec → planning)
#   metagent task <taskname>             Create task directory and add to queue
#   metagent finish <stage>              Signal stage completion
#   metagent run <taskname>              Run loop for a task
#   metagent queue [taskname]            Add task to queue / show queue
#   metagent dequeue <taskname>          Remove task from queue
#   metagent run-queue                   Process all queued tasks
#
# Common options:
#   -h, --help          Show help

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Resolve symlinks to get the actual script directory
SCRIPT_SOURCE="${BASH_SOURCE[0]}"
while [ -L "$SCRIPT_SOURCE" ]; do
    SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_SOURCE")" && pwd)"
    SCRIPT_SOURCE="$(readlink "$SCRIPT_SOURCE")"
    # Handle relative symlinks
    [[ $SCRIPT_SOURCE != /* ]] && SCRIPT_SOURCE="$SCRIPT_DIR/$SCRIPT_SOURCE"
done
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_SOURCE")" && pwd)"
AGENT="${METAGENT_AGENT:-code}"  # Default to code, can be overridden

# ============================================================================
# Helper Functions
# ============================================================================

show_help() {
    echo "metagent - Agent workflow manager"
    echo ""
    echo "Usage:"
    echo "  metagent [--agent TYPE] <command> [args]"
    echo ""
    echo "Agent Types:"
    echo "  code                 Software development (default)"
    echo "  writer               Writing projects"
    echo ""
    echo "Commands:"
    echo "  install                     Setup globally (first time)"
    echo "  uninstall                   Remove metagent globally"
    echo "  init [path]                 Initialize agent in repo"
    echo "  start                       Start new task interactively"
    echo "  task <taskname>             Create task directory and add to queue"
    echo "  finish <stage>              Signal stage completion"
    echo "  run <taskname>              Run loop for a task"
    echo "  queue [taskname]            Show queue / add task to queue"
    echo "  dequeue <taskname>          Remove task from queue"
    echo "  run-queue                   Process all queued tasks"
    echo ""
    echo "Options:"
    echo "  --agent TYPE        Select agent type (code, writer)"
    echo "  -h, --help          Show help"
    echo ""
    echo "Examples:"
    echo "  metagent install                     # First time setup"
    echo "  metagent init                        # Initialize code agent in current dir"
    echo "  metagent --agent writer init         # Initialize writer agent"
    echo "  metagent start                       # Start new task interactively"
    echo "  metagent task my-feature             # Create task (used by model)"
    echo "  metagent finish spec                 # Signal spec phase complete"
    echo "  metagent run my-feature              # Run loop for existing task"
    echo "  metagent queue                       # Show task queue"
}

# Load agent-specific functions
load_agent() {
    local agent_script="$SCRIPT_DIR/$AGENT/agent.sh"
    if [ ! -f "$agent_script" ]; then
        echo -e "${RED}Error: Agent '$AGENT' not found at $agent_script${NC}"
        exit 1
    fi
    source "$agent_script"
}

# Validate and sanitize task name
# Only allows alphanumeric, hyphens, and underscores
# Prevents path traversal and regex injection
sanitize_taskname() {
    local name="$1"
    # Check for empty
    if [ -z "$name" ]; then
        echo ""
        return 0
    fi
    # Check for invalid characters (only allow alphanumeric, hyphens, underscores)
    if [[ ! "$name" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        echo -e "${RED}Error: Invalid task name '$name'${NC}" >&2
        echo -e "Task names can only contain letters, numbers, hyphens, and underscores." >&2
        return 1
    fi
    # Check for path traversal attempts
    if [[ "$name" == *".."* ]] || [[ "$name" == "."* ]]; then
        echo -e "${RED}Error: Invalid task name '$name'${NC}" >&2
        return 1
    fi
    # Check reasonable length
    if [ ${#name} -gt 100 ]; then
        echo -e "${RED}Error: Task name too long (max 100 chars)${NC}" >&2
        return 1
    fi
    echo "$name"
}

# ============================================================================
# Init Command (per-repo setup)
# ============================================================================

do_init() {
    local target_repo="$1"
    local metagent_dir="$HOME/.metagent"

    # Check metagent is installed
    if [ ! -d "$metagent_dir/$AGENT" ]; then
        echo -e "${RED}Error: metagent $AGENT agent not installed. Run 'metagent install' first.${NC}"
        exit 1
    fi

    # Use current directory if no path given
    if [ -z "$target_repo" ]; then
        target_repo="$(pwd)"
    else
        target_repo="$(cd "$target_repo" 2>/dev/null && pwd)" || {
            echo -e "${RED}Error: Target directory does not exist: $target_repo${NC}"
            exit 1
        }
    fi

    # Check if target is a git repo
    if [ ! -d "$target_repo/.git" ]; then
        echo -e "${YELLOW}Warning: Target is not a git repository${NC}"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 0
        fi
    fi

    # Check if .agents/$AGENT already exists
    local do_overwrite=false
    if [ -d "$target_repo/.agents/$AGENT" ]; then
        echo -e "${YELLOW}Warning: .agents/$AGENT/ already exists in target${NC}"
        read -p "Overwrite? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 0
        fi
        do_overwrite=true
    fi

    echo -e "${BLUE}Installing $AGENT agent to: ${target_repo}${NC}"

    # Create directory structure
    mkdir -p "$target_repo/.agents/$AGENT/tasks"

    # Copy templates (project-specific files)
    local templates_dir="$SCRIPT_DIR/$AGENT/templates"
    if [ -d "$templates_dir" ]; then
        for file in "$templates_dir"/*; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                local dest="$target_repo/.agents/$AGENT/$filename"
                if [ ! -f "$dest" ]; then
                    cp "$file" "$dest"
                    echo -e "  ${GREEN}✓${NC} $filename"
                elif [ "$do_overwrite" = true ]; then
                    cp "$file" "$dest"
                    echo -e "  ${GREEN}✓${NC} $filename (overwritten)"
                else
                    echo -e "  ${YELLOW}⊘${NC} $filename (already exists)"
                fi
            fi
        done
    fi

    echo ""
    echo -e "${GREEN}✓ Installed!${NC}"
    echo ""
    echo "Created: $target_repo/.agents/$AGENT/"
    echo ""
    agent_install_message
}

# ============================================================================
# Install/Uninstall Commands (global setup)
# ============================================================================

do_install() {
    local bin_dir="$HOME/.local/bin"
    local metagent_dir="$HOME/.metagent"
    local claude_commands="$HOME/.claude/commands"
    local codex_commands="$HOME/.codex/prompts"

    # Create directories
    mkdir -p "$bin_dir"
    mkdir -p "$claude_commands"
    mkdir -p "$codex_commands"

    # Link metagent to PATH
    ln -sf "$SCRIPT_DIR/metagent.sh" "$bin_dir/metagent"
    echo -e "${GREEN}✓${NC} Linked metagent to $bin_dir/metagent"

    # Install each agent's prompts
    for agent_dir in "$SCRIPT_DIR"/*/; do
        local agent_name=$(basename "$agent_dir")
        # Skip if not a valid agent (must have agent.sh)
        if [ ! -f "$agent_dir/agent.sh" ]; then
            continue
        fi

        mkdir -p "$metagent_dir/$agent_name"

        # Copy prompts
        if [ -d "$agent_dir/prompts" ]; then
            echo -e "${BLUE}Installing $agent_name prompts...${NC}"
            for file in "$agent_dir/prompts"/*; do
                if [ -f "$file" ]; then
                    cp "$file" "$metagent_dir/$agent_name/"
                    echo -e "  ${GREEN}✓${NC} $(basename "$file")"
                fi
            done
        fi

        # Source agent to get slash command config
        source "$agent_dir/agent.sh"
        if type agent_slash_commands &>/dev/null; then
            agent_slash_commands "$metagent_dir/$agent_name" "$claude_commands" "$codex_commands"
        fi
    done

    # Check if ~/.local/bin is in PATH
    echo ""
    if [[ ":$PATH:" != *":$bin_dir:"* ]]; then
        echo -e "${YELLOW}Note: $bin_dir is not in your PATH${NC}"
        echo "Add this to your shell profile (.bashrc, .zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Then restart your shell or run: source ~/.zshrc"
    else
        echo -e "${GREEN}✓ metagent is ready to use${NC}"
    fi

    echo ""
    echo "Installed:"
    echo "  ~/.local/bin/metagent     - CLI tool"
    echo "  ~/.metagent/              - Agent prompts"
    echo "  ~/.claude/commands/       - Slash commands"
    echo "  ~/.codex/prompts/         - Slash commands"
}

# ============================================================================
# Queue Commands
# ============================================================================

# Queue file format (JSON lines for easy parsing):
# {"task":"name","added":"2024-01-13T10:30:00","stage":"spec","status":"pending"}
#
# Status values: pending, running, failed

get_queue_file() {
    echo ".agents/$AGENT/queue.jsonl"
}

# Helper to extract JSON field value
json_field() {
    echo "$1" | grep -o "\"$2\":\"[^\"]*\"" | cut -d'"' -f4
}

do_queue() {
    local taskname
    taskname=$(sanitize_taskname "$1") || exit 1
    local queue_file
    queue_file=$(get_queue_file)

    # No argument - show queue
    if [ -z "$taskname" ]; then
        if [ ! -f "$queue_file" ]; then
            echo -e "${YELLOW}No tasks${NC}"
            exit 0
        fi

        echo -e "${BLUE}Tasks:${NC}"
        echo ""

        # Group by stage (get stages from agent)
        local stages
        stages=$(agent_stages)
        for stage in $stages; do
            local stage_tasks
            stage_tasks=$(grep "\"stage\":\"$stage\"" "$queue_file" 2>/dev/null || true)

            if [ -n "$stage_tasks" ]; then
                local label
                label=$(agent_stage_label "$stage")
                case "$stage" in
                    *completed*) echo -e "${GREEN}${label}:${NC}" ;;
                    *ready*)     echo -e "${BLUE}${label}:${NC}" ;;
                    *planning*)  echo -e "${CYAN}${label}:${NC}" ;;
                    *)           echo -e "${YELLOW}${label}:${NC}" ;;
                esac

                echo "$stage_tasks" | while IFS= read -r line; do
                    local task status
                    task=$(json_field "$line" "task")
                    status=$(json_field "$line" "status")

                    case "$status" in
                        pending) echo -e "  ○ ${task}" ;;
                        running) echo -e "  ${BLUE}●${NC} ${task} (running)" ;;
                        failed)  echo -e "  ${RED}✗${NC} ${task} (failed)" ;;
                        *)       echo -e "  ○ ${task}" ;;
                    esac
                done
                echo ""
            fi
        done
        exit 0
    fi

    # Check if already in queue
    if [ -f "$queue_file" ] && grep -q "\"task\":\"$taskname\"" "$queue_file"; then
        echo -e "${YELLOW}Task '$taskname' already exists${NC}"
        # Show current stage
        local current
        current=$(grep "\"task\":\"$taskname\"" "$queue_file")
        local stage
        stage=$(json_field "$current" "stage")
        echo "  Stage: $stage"
        exit 0
    fi

    # Add to queue at initial stage
    mkdir -p "$(dirname "$queue_file")"
    local timestamp initial_stage
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S")
    initial_stage=$(agent_initial_stage)
    echo "{\"task\":\"$taskname\",\"added\":\"$timestamp\",\"stage\":\"$initial_stage\",\"status\":\"pending\"}" >> "$queue_file"

    echo -e "${GREEN}✓${NC} Added '$taskname' (stage: $initial_stage)"
}

do_dequeue() {
    local taskname
    taskname=$(sanitize_taskname "$1") || exit 1
    local queue_file
    queue_file=$(get_queue_file)

    if [ -z "$taskname" ]; then
        echo -e "${RED}Error: Task name required${NC}"
        echo "Usage: metagent dequeue <taskname>"
        exit 1
    fi

    if [ ! -f "$queue_file" ]; then
        echo -e "${YELLOW}No tasks${NC}"
        exit 0
    fi

    if ! grep -q "\"task\":\"$taskname\"" "$queue_file"; then
        echo -e "${YELLOW}Task '$taskname' not found${NC}"
        exit 0
    fi

    # Remove from queue
    local temp_file="${queue_file}.tmp"
    grep -v "\"task\":\"$taskname\"" "$queue_file" > "$temp_file" || true
    mv "$temp_file" "$queue_file"

    # Remove empty file
    if [ ! -s "$queue_file" ]; then
        rm -f "$queue_file"
    fi

    echo -e "${GREEN}✓${NC} Removed '$taskname'"
}

update_task_field() {
    local taskname="$1"
    local field="$2"
    local value="$3"
    local queue_file
    queue_file=$(get_queue_file)

    if [ ! -f "$queue_file" ]; then
        return
    fi

    local temp_file="${queue_file}.tmp"
    while IFS= read -r line; do
        if echo "$line" | grep -q "\"task\":\"$taskname\""; then
            echo "$line" | sed "s/\"$field\":\"[^\"]*\"/\"$field\":\"$value\"/"
        else
            echo "$line"
        fi
    done < "$queue_file" > "$temp_file"
    mv "$temp_file" "$queue_file"
}

# ============================================================================
# Task Command - Create a new task (called by model during spec phase)
# ============================================================================

do_task() {
    local taskname
    taskname=$(sanitize_taskname "$1") || exit 1

    if [ -z "$taskname" ]; then
        echo -e "${RED}Error: Task name required${NC}"
        echo "Usage: metagent task <taskname>"
        exit 1
    fi

    local task_dir=".agents/$AGENT/tasks/${taskname}"
    local queue_file
    queue_file=$(get_queue_file)

    # Check if task already exists in queue
    if [ -f "$queue_file" ] && grep -q "\"task\":\"$taskname\"" "$queue_file"; then
        echo -e "${GREEN}Task '$taskname' already exists${NC}"
        local current
        current=$(grep "\"task\":\"$taskname\"" "$queue_file")
        local stage
        stage=$(json_field "$current" "stage")
        echo "  Stage: $stage"
        echo "  Directory: $task_dir"
        exit 0
    fi

    # Create agent-specific directory structure (calls agent_create_task from agent.sh)
    agent_create_task "$taskname" "$task_dir"

    # Add to queue
    mkdir -p "$(dirname "$queue_file")"
    local timestamp initial_stage
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S")
    initial_stage=$(agent_initial_stage)
    echo "{\"task\":\"$taskname\",\"added\":\"$timestamp\",\"stage\":\"$initial_stage\",\"status\":\"pending\"}" >> "$queue_file"

    echo -e "${GREEN}✓ Created task: ${taskname}${NC}"
    echo ""
    echo "  Directory: ${task_dir}"
    echo "  Stage: $initial_stage"
}

# ============================================================================
# Finish Command - Signal stage/task completion
# ============================================================================

do_finish() {
    local stage="${1:-task}"
    local override_next=""
    shift || true

    # Parse --next flag
    while [ $# -gt 0 ]; do
        case "$1" in
            --next)
                override_next="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    local queue_file
    queue_file=$(get_queue_file)
    local marker_file="${CLAUDE_DONE_MARKER:-}"
    local task="${METAGENT_TASK:-}"

    # Validate stage
    local valid_stages
    valid_stages=$(agent_valid_finish_stages)
    local stage_valid=false
    for s in $valid_stages; do
        if [ "$s" = "$stage" ]; then
            stage_valid=true
            break
        fi
    done

    if [ "$stage_valid" = false ]; then
        echo -e "${RED}Unknown stage: $stage${NC}"
        echo "Usage: metagent finish [$valid_stages] [--next <stage>]"
        exit 1
    fi

    # Get next stage: use override if provided, otherwise ask agent
    local next_stage=""
    if [ -n "$override_next" ]; then
        next_stage="$override_next"
    elif [ "$stage" != "task" ]; then
        next_stage=$(agent_next_stage "$stage")
    fi
    agent_finish_message "$stage"

    # Update queue file if we have a task name and next stage
    if [ -n "$task" ] && [ -n "$next_stage" ] && [ -f "$queue_file" ]; then
        local temp_file="${queue_file}.tmp"
        while IFS= read -r line; do
            if echo "$line" | grep -q "\"task\":\"$task\""; then
                # Update stage and reset status to pending
                echo "$line" | sed "s/\"stage\":\"[^\"]*\"/\"stage\":\"$next_stage\"/" | sed "s/\"status\":\"[^\"]*\"/\"status\":\"pending\"/"
            else
                echo "$line"
            fi
        done < "$queue_file" > "$temp_file"
        mv "$temp_file" "$queue_file"
        echo "Advanced '$task' to stage: $next_stage"
    fi

    # Signal orchestrator if running under metagent start
    if [ -n "$marker_file" ]; then
        echo "$stage" > "$marker_file"
    else
        # Running manually - just print next steps
        echo ""
        if [ -n "$next_stage" ]; then
            echo "Next: Run 'metagent start' to continue, or manually run the $next_stage phase."
        else
            echo "Run 'metagent start' to continue with remaining tasks."
        fi
    fi
}

# ============================================================================
# Start Command - Interactive task creation and orchestration
# ============================================================================

do_start() {
    local queue_file
    queue_file=$(get_queue_file)
    local marker_file="/tmp/.metagent-done-$$"
    local metagent_dir="$HOME/.metagent/$AGENT"
    local initial_stage
    initial_stage=$(agent_initial_stage)
    local initial_prompt
    initial_prompt=$(agent_prompt_for_stage "$initial_stage" "")

    # Verify metagent is installed
    if [ ! -f "$initial_prompt" ]; then
        echo -e "${RED}Error: metagent not installed. Run 'metagent install' first.${NC}"
        exit 1
    fi

    # Setup environment for metagent finish
    export CLAUDE_DONE_MARKER="$marker_file"
    export METAGENT_AGENT="$AGENT"

    cleanup() {
        rm -f "$marker_file"
        # Kill claude if still running
        if [ -n "$CLAUDE_PID" ] && kill -0 "$CLAUDE_PID" 2>/dev/null; then
            kill "$CLAUDE_PID" 2>/dev/null
            wait "$CLAUDE_PID" 2>/dev/null
        fi
    }
    trap cleanup EXIT

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Starting new task...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Get the stages that need orchestration (before ready)
    local stages
    stages=$(agent_stages)
    local orchestrated_stages=""
    for s in $stages; do
        if [ "$s" = "ready" ] || [ "$s" = "completed" ]; then
            break
        fi
        orchestrated_stages="$orchestrated_stages $s"
    done

    # Main loop - keeps running until all stages complete
    while true; do
        rm -f "$marker_file"

        local taskname=""
        local stage=""
        local prompt_file=""

        if [ -f "$queue_file" ]; then
            # Find first pending task at an orchestrated stage
            for s in $orchestrated_stages; do
                local found_line
                found_line=$(grep "\"stage\":\"$s\"" "$queue_file" 2>/dev/null | grep "\"status\":\"pending\"" | head -1 || true)
                if [ -n "$found_line" ]; then
                    taskname=$(json_field "$found_line" "task")
                    stage="$s"
                    break
                fi
            done
        fi

        if [ -z "$taskname" ]; then
            # Check if we have tasks at ready stage
            local ready_tasks
            ready_tasks=$(grep "\"stage\":\"ready\"" "$queue_file" 2>/dev/null | grep "\"status\":\"pending\"" | head -1 || true)
            if [ -n "$ready_tasks" ]; then
                # Tasks are ready for build - stop here
                echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                echo -e "${GREEN}Planning complete!${NC}"
                echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                echo ""
                echo "Tasks ready for build:"
                grep "\"stage\":\"ready\"" "$queue_file" | while IFS= read -r line; do
                    local task
                    task=$(json_field "$line" "task")
                    echo "  - $task"
                done
                echo ""
                echo "Run 'metagent run-queue' or 'metagent run <taskname>' to start."
                exit 0
            fi

            # No pending tasks at all - start with initial stage interview
            stage="interview"
            prompt_file="$initial_prompt"
            echo -e "${CYAN}Starting $(agent_stage_label "$initial_stage") interview...${NC}"
            echo ""
        else
            # Run the pending task's stage
            prompt_file=$(agent_prompt_for_stage "$stage" "$taskname")
            export METAGENT_TASK="$taskname"
            update_task_field "$taskname" "status" "running"

            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo -e "${BLUE}Task: ${taskname} | Stage: ${stage}${NC}"
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
        fi

        if [ ! -f "$prompt_file" ]; then
            echo -e "${RED}Error: Prompt file not found: ${prompt_file}${NC}"
            exit 1
        fi

        # Run claude in background
        if [ "$stage" = "interview" ]; then
            sed "s/{task}/$taskname/g" "$prompt_file" | claude --dangerously-skip-permissions &
        else
            (echo "Task: $taskname"; echo ""; sed "s/{task}/$taskname/g" "$prompt_file") | claude --dangerously-skip-permissions &
        fi
        CLAUDE_PID=$!

        # Monitor for marker file or claude exit
        while kill -0 "$CLAUDE_PID" 2>/dev/null; do
            if [ -f "$marker_file" ]; then
                local marker_content
                marker_content=$(cat "$marker_file")
                rm -f "$marker_file"

                echo ""
                local next_stage
                next_stage=$(agent_next_stage "$marker_content")

                if [ "$next_stage" = "ready" ]; then
                    # Last orchestrated stage complete - let user review
                    echo -e "${GREEN}✓${NC} $(agent_stage_label "$marker_content") phase complete"
                    echo ""
                    echo -e "${CYAN}Review with Claude. Exit when ready.${NC}"
                    wait "$CLAUDE_PID" 2>/dev/null

                    echo ""
                    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                    echo -e "${GREEN}Planning complete!${NC}"
                    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                    echo ""
                    echo "Task '$taskname' is ready."
                    echo ""
                    echo "Run 'metagent run $taskname' or 'metagent run-queue' to start."
                    exit 0
                else
                    # More orchestrated stages - continue
                    echo -e "${GREEN}✓${NC} $(agent_stage_label "$marker_content") phase complete"
                    kill "$CLAUDE_PID" 2>/dev/null
                    wait "$CLAUDE_PID" 2>/dev/null
                    echo ""
                    sleep 1
                    break
                fi
            fi
            sleep 0.5
        done

        # Check if we broke out due to stage completion (need to continue loop)
        if [ "$stage" = "interview" ] && [ -f "$queue_file" ]; then
            continue
        elif [ -n "$stage" ] && [ "$stage" != "interview" ]; then
            local next
            next=$(agent_next_stage "$stage")
            if [ -n "$next" ] && [ "$next" != "ready" ]; then
                continue
            fi
        fi

        # Claude exited without marker - unexpected
        wait "$CLAUDE_PID" 2>/dev/null

        if [ -n "$taskname" ]; then
            update_task_field "$taskname" "status" "failed"
            echo ""
            echo -e "${RED}✗${NC} Task '$taskname' exited without completing stage: $stage"
        else
            echo ""
            echo -e "${RED}✗${NC} Interview ended without creating a task"
        fi
        exit 1
    done
}

run_stage() {
    local taskname="$1"
    local stage="$2"
    local prompt_file
    prompt_file=$(agent_prompt_for_stage "$stage" "$taskname")

    if [ ! -f "$prompt_file" ]; then
        echo -e "${RED}Error: Prompt file not found: ${prompt_file}${NC}"
        return 1
    fi

    # Setup environment for metagent finish
    local marker_file="/tmp/.metagent-done-$$"
    export CLAUDE_DONE_MARKER="$marker_file"
    export METAGENT_TASK="$taskname"
    export METAGENT_AGENT="$AGENT"
    rm -f "$marker_file"

    # Run Claude with the prompt
    local initial_stage
    initial_stage=$(agent_initial_stage)
    if [ "$stage" = "$initial_stage" ] || [ "$stage" = "planning" ]; then
        echo -e "${CYAN}Task: ${taskname}${NC}"
        echo ""
        (echo "Task: $taskname"; echo ""; sed "s/{task}/$taskname/g" "$prompt_file") | claude --dangerously-skip-permissions
    else
        sed "s/{task}/$taskname/g" "$prompt_file" | claude --dangerously-skip-permissions
    fi
    local exit_code=$?

    # Check if metagent finish was called
    if [ -f "$marker_file" ]; then
        local marker_content
        marker_content=$(cat "$marker_file")
        rm -f "$marker_file"

        local next_stage
        next_stage=$(agent_next_stage "$marker_content")
        if [ -n "$next_stage" ] && [ "$next_stage" != "completed" ]; then
            return 0  # Stage complete
        fi
        if [ "$marker_content" = "task" ]; then
            return 2  # Task complete but more work to do
        fi
    fi

    # Claude exited without calling metagent finish
    if [ "$stage" = "ready" ]; then
        return 0  # All tasks complete
    else
        return $exit_code
    fi
}

do_run_queue() {
    local queue_file
    queue_file=$(get_queue_file)

    if [ ! -f "$queue_file" ]; then
        echo -e "${YELLOW}No tasks${NC}"
        exit 0
    fi

    echo -e "${BLUE}Processing tasks...${NC}"
    echo ""

    # Get stages from agent
    local stages
    stages=$(agent_stages)

    # Keep processing until no pending tasks remain
    while true; do
        if [ ! -f "$queue_file" ]; then
            break
        fi

        # Find first pending task
        local taskname=""
        local stage=""
        local found_line=""

        for s in $stages; do
            if [ "$s" = "completed" ]; then
                continue
            fi
            found_line=$(grep "\"stage\":\"$s\"" "$queue_file" 2>/dev/null | grep "\"status\":\"pending\"" | head -1 || true)
            if [ -n "$found_line" ]; then
                taskname=$(json_field "$found_line" "task")
                stage="$s"
                break
            fi
        done

        # No pending tasks
        if [ -z "$taskname" ]; then
            break
        fi

        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BLUE}Task: ${taskname} | Stage: ${stage}${NC}"
        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""

        update_task_field "$taskname" "status" "running"

        run_stage "$taskname" "$stage"
        local result=$?

        if [ $result -eq 0 ]; then
            echo ""
            echo -e "${GREEN}✓${NC} '$taskname' stage complete"
        elif [ $result -eq 2 ]; then
            echo ""
            echo -e "${CYAN}↻${NC} '$taskname' continuing..."
        else
            update_task_field "$taskname" "status" "failed"
            echo ""
            echo -e "${RED}✗${NC} '$taskname' failed at stage: $stage"
        fi

        echo ""
    done

    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}Queue processing complete!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# ============================================================================
# Run Command
# ============================================================================

do_run() {
    local taskname
    taskname=$(sanitize_taskname "$1") || exit 1

    if [ -z "$taskname" ]; then
        echo -e "${RED}Error: Task name required${NC}"
        echo "Usage: metagent run <taskname>"
        exit 1
    fi

    local task_dir=".agents/$AGENT/tasks/${taskname}"
    local queue_file
    queue_file=$(get_queue_file)

    # Get current stage from queue
    local current_stage=""
    if [ -f "$queue_file" ]; then
        local task_entry
        task_entry=$(grep "\"task\":\"$taskname\"" "$queue_file" || true)
        if [ -n "$task_entry" ]; then
            current_stage=$(json_field "$task_entry" "stage")
        fi
    fi

    if [ -z "$current_stage" ]; then
        echo -e "${RED}Error: Task '$taskname' not found in queue${NC}"
        echo "Run 'metagent queue $taskname' to add it first."
        exit 1
    fi

    if [ "$current_stage" = "completed" ]; then
        echo -e "${GREEN}Task '$taskname' is already completed${NC}"
        exit 0
    fi

    local prompt_file
    prompt_file=$(agent_prompt_for_stage "$current_stage" "$taskname")

    if [ ! -f "$prompt_file" ]; then
        echo -e "${RED}Error: Prompt file not found: ${prompt_file}${NC}"
        exit 1
    fi

    # Marker file for metagent finish to signal task completion
    local marker_file="/tmp/.metagent-done-$$"
    export CLAUDE_DONE_MARKER="$marker_file"
    export METAGENT_AGENT="$AGENT"
    export METAGENT_TASK="$taskname"

    cleanup() {
        rm -f "$marker_file"
    }
    trap cleanup EXIT

    echo -e "${BLUE}Starting loop for: ${taskname} (stage: ${current_stage})${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""

    local loop_count=0
    while true; do
        loop_count=$((loop_count + 1))
        rm -f "$marker_file"

        # Re-read current stage (may have changed via --next flag)
        task_entry=$(grep "\"task\":\"$taskname\"" "$queue_file" || true)
        current_stage=$(json_field "$task_entry" "stage")
        prompt_file=$(agent_prompt_for_stage "$current_stage" "$taskname")

        if [ "$current_stage" = "completed" ]; then
            echo -e "${GREEN}Task '$taskname' completed!${NC}"
            break
        fi

        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BLUE}Loop #${loop_count} - ${current_stage} - $(date '+%Y-%m-%d %H:%M:%S')${NC}"
        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""

        # Run claude in background so we can monitor for marker file
        sed "s/{task}/$taskname/g" "$prompt_file" | claude --dangerously-skip-permissions &
        local CLAUDE_PID=$!

        # Monitor for marker file or claude exit
        local marker_found=false
        while kill -0 "$CLAUDE_PID" 2>/dev/null; do
            if [ -f "$marker_file" ]; then
                # Marker found - kill claude and continue loop
                marker_found=true
                kill "$CLAUDE_PID" 2>/dev/null
                wait "$CLAUDE_PID" 2>/dev/null
                rm -f "$marker_file"
                echo ""
                echo -e "${GREEN}Stage completed, continuing...${NC}"
                echo ""
                sleep 2
                break
            fi
            sleep 0.5
        done

        # Continue loop if marker was found
        if [ "$marker_found" = true ]; then
            continue
        fi

        # Claude exited naturally - check if marker was written just before exit
        if [ -f "$marker_file" ]; then
            rm -f "$marker_file"
            echo ""
            echo -e "${GREEN}Stage completed, continuing...${NC}"
            echo ""
            sleep 2
            continue
        fi

        # No marker - claude exited without signaling, stop loop
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}Session ended. Run 'metagent run $taskname' to continue.${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        exit 0
    done
}

do_uninstall() {
    local bin_dir="$HOME/.local/bin"
    local metagent_dir="$HOME/.metagent"
    local claude_commands="$HOME/.claude/commands"
    local codex_commands="$HOME/.codex/prompts"

    # Remove metagent from PATH
    if [ -L "$bin_dir/metagent" ]; then
        rm "$bin_dir/metagent"
        echo -e "${GREEN}✓${NC} Removed $bin_dir/metagent"
    fi

    # Remove all slash commands (handled by agents)
    for commands_dir in "$claude_commands" "$codex_commands"; do
        # Find and remove all symlinks pointing to metagent
        for link in "$commands_dir"/*.md; do
            if [ -L "$link" ]; then
                local target
                target=$(readlink "$link")
                if [[ "$target" == *".metagent"* ]]; then
                    rm "$link"
                fi
            fi
        done
    done
    echo -e "${GREEN}✓${NC} Removed slash commands"

    # Remove ~/.metagent/
    if [ -d "$metagent_dir" ]; then
        rm -rf "$metagent_dir"
        echo -e "${GREEN}✓${NC} Removed $metagent_dir"
    fi

    echo ""
    echo -e "${GREEN}✓ metagent uninstalled${NC}"
}

# ============================================================================
# Main
# ============================================================================

# No arguments - show help
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

# Parse global options first (before command)
while [[ $# -gt 0 ]]; do
    case $1 in
        --agent)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                echo -e "${RED}Error: --agent requires a value (code, writer)${NC}"
                exit 1
            fi
            AGENT="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        -*)
            break
            ;;
        *)
            break
            ;;
    esac
done

# Validate agent exists
if [ ! -f "$SCRIPT_DIR/$AGENT/agent.sh" ]; then
    echo -e "${RED}Error: Unknown agent type '$AGENT'.${NC}"
    echo "Available agents:"
    for d in "$SCRIPT_DIR"/*/; do
        if [ -f "$d/agent.sh" ]; then
            echo "  - $(basename "$d")"
        fi
    done
    exit 1
fi

# Load agent-specific functions
load_agent

# No command after options - show help
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

# Get command
COMMAND="$1"
shift

# Execute command (pass all remaining args to the handler)
case "$COMMAND" in
    install)
        do_install
        ;;
    uninstall)
        do_uninstall
        ;;
    init|i)
        do_init "$@"
        ;;
    start)
        do_start
        ;;
    task|t)
        do_task "$@"
        ;;
    finish|f)
        do_finish "$@"
        ;;
    run|r)
        do_run "$@"
        ;;
    queue|q)
        do_queue "$@"
        ;;
    dequeue)
        do_dequeue "$@"
        ;;
    run-queue|rq)
        do_run_queue
        ;;
    *)
        echo -e "${RED}Error: Unknown command '$COMMAND'${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
