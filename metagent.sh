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
#   metagent review <taskname> [focus]   Review task commits for issues
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
AGENT="${METAGENT_AGENT:-code}"    # Default to code, can be overridden
MODEL="${METAGENT_MODEL:-claude}"  # Default to claude, can be overridden (claude or codex)

# Get the CLI command with appropriate flags
get_cli_cmd() {
    case "$MODEL" in
        codex)
            echo "codex --dangerously-bypass-approvals-and-sandbox"
            ;;
        claude|*)
            echo "claude --dangerously-skip-permissions"
            ;;
    esac
}

# ============================================================================
# Helper Functions
# ============================================================================

show_help() {
    echo "metagent - Agent workflow manager"
    echo ""
    echo "Usage:"
    echo "  metagent                            Start new task (interview → spec → planning)"
    echo "  metagent [options] <command> [args]"
    echo ""
    echo "Agent Types:"
    echo "  code                 Software development (default)"
    echo "  writer               Writing projects"
    echo ""
    echo "Commands:"
    echo "  install                     Setup globally (first time)"
    echo "  uninstall                   Remove metagent globally"
    echo "  init [path]                 Initialize agent in repo"
    echo "  task <taskname>             Create task directory and add to queue"
    echo "  finish <stage>              Signal stage completion"
    echo "  run <taskname>              Run loop for a task"
    echo "  queue [taskname]            Show queue / add task to queue"
    echo "  dequeue <taskname>          Remove task from queue"
    echo "  run-queue                   Process all queued tasks"
    echo "  review <taskname> [focus]   Review task commits for issues"
    echo ""
    echo "Options:"
    echo "  --agent TYPE        Select agent type (code, writer)"
    echo "  --model TYPE        Select model CLI (claude, codex) [default: claude]"
    echo "  -h, --help          Show help"
    echo ""
    echo "Environment variables:"
    echo "  METAGENT_MODEL      Model CLI to use (claude, codex)"
    echo "  METAGENT_AGENT      Agent type to use (code, writer)"
    echo ""
    echo "Finish options:"
    echo "  --next STAGE        Override next stage"
    echo ""
    echo "Examples:"
    echo "  metagent                             # Start new task interactively"
    echo "  metagent install                     # First time setup"
    echo "  metagent init                        # Initialize code agent in current dir"
    echo "  metagent --agent writer init         # Initialize writer agent"
    echo "  metagent --model codex run my-task   # Run task with Codex"
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

# Resolve repo root for queue/task paths (prefers .agents, then .git)
get_repo_root() {
    if [ -n "$METAGENT_REPO_ROOT" ]; then
        echo "$METAGENT_REPO_ROOT"
        return
    fi

    local start_dir="${1:-$(pwd)}"
    local dir="$start_dir"

    while true; do
        if [ -d "$dir/.agents" ] || [ -d "$dir/.git" ]; then
            METAGENT_REPO_ROOT="$dir"
            export METAGENT_REPO_ROOT
            echo "$dir"
            return
        fi
        if [ "$dir" = "/" ]; then
            break
        fi
        dir="$(dirname "$dir")"
    done

    echo -e "${RED}Error: No repo found (missing .agents/ or .git). Run 'metagent init' in a repo.${NC}" >&2
    exit 1
}

get_agent_root() {
    local repo_root
    repo_root=$(get_repo_root)
    if [ ! -d "$repo_root/.agents" ]; then
        echo -e "${RED}Error: .agents/ not found in repo. Run 'metagent init' first.${NC}" >&2
        exit 1
    fi
    echo "$repo_root/.agents/$AGENT"
}

is_valid_task_name() {
    [[ "$1" =~ ^[a-z0-9-]+$ ]]
}

read_marker() {
    local marker_file="$1"
    MARKER_STAGE=""
    MARKER_NEXT=""
    MARKER_TASK=""
    local key value
    while IFS='=' read -r key value; do
        case "$key" in
            stage) MARKER_STAGE="$value" ;;
            next)  MARKER_NEXT="$value" ;;
            task)  MARKER_TASK="$value" ;;
        esac
    done < "$marker_file"
}

escape_sed_replacement() {
    printf '%s' "$1" | sed -e 's/[&|\\]/\\&/g'
}

render_prompt() {
    local prompt_file="$1"
    local taskname="$2"
    local repo_root
    repo_root=$(get_repo_root)
    local repo_escaped
    repo_escaped=$(escape_sed_replacement "$repo_root")

    # Check if task has issues status
    local issues_mode=""
    if [ -n "$taskname" ]; then
        local queue_file
        queue_file=$(get_queue_file)
        if [ -f "$queue_file" ]; then
            local task_entry
            task_entry=$(grep "\"task\":\"$taskname\"" "$queue_file" 2>/dev/null || true)
            if [ -n "$task_entry" ]; then
                local status
                status=$(json_field "$task_entry" "status")
                if [ "$status" = "issues" ]; then
                    issues_mode="99999999999999. **REVIEW ISSUES:** This task returned from review with issues. Read @.agents/code/tasks/${taskname}/issues.md and address all open issues for this phase. Discuss decisions with the user as needed. Update issue status to \"resolved\" when addressed. All issues must be resolved before finishing this phase."
                fi
            fi
        fi
    fi
    local issues_escaped
    issues_escaped=$(escape_sed_replacement "$issues_mode")

    if [ -n "$taskname" ]; then
        local task_escaped
        task_escaped=$(escape_sed_replacement "$taskname")
        sed -E \
            -e "s|{repo}|$repo_escaped|g" \
            -e "s|{taskname}|$task_escaped|g" \
            -e "s|{task}|$task_escaped|g" \
            -e "s|{issues_mode}|$issues_escaped|g" \
            "$prompt_file"
    else
        sed -E \
            -e "s|{repo}|$repo_escaped|g" \
            -e "s|{issues_mode}||g" \
            "$prompt_file"
    fi
}

find_unique_task() {
    local queue_file="$1"
    local stage="$2"
    local status_filter="$3"
    local task=""
    local count=0

    if [ ! -f "$queue_file" ]; then
        return 1
    fi

    while IFS= read -r line; do
        if echo "$line" | grep -q "\"stage\":\"$stage\""; then
            if [ -n "$status_filter" ]; then
                local status
                status=$(json_field "$line" "status")
                if [ "$status" != "$status_filter" ]; then
                    continue
                fi
            fi
            task=$(json_field "$line" "task")
            count=$((count + 1))
        fi
    done < "$queue_file"

    if [ $count -eq 1 ]; then
        echo "$task"
        return 0
    fi

    return 1
}

# Validate and sanitize task name
# Only allows lowercase letters, numbers, and hyphens
# Prevents path traversal and regex injection
sanitize_taskname() {
    local name="$1"
    # Check for empty
    if [ -z "$name" ]; then
        echo ""
        return 0
    fi
    # Check for invalid characters (only allow lowercase letters, numbers, hyphens)
    if ! is_valid_task_name "$name"; then
        echo -e "${RED}Error: Invalid task name '$name'${NC}" >&2
        echo -e "Task names can only contain lowercase letters, numbers, and hyphens." >&2
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

    # Copy metagent to PATH
    cp "$SCRIPT_DIR/metagent.sh" "$bin_dir/metagent"
    echo -e "${GREEN}✓${NC} Copied metagent to $bin_dir/metagent"

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
        unset -f agent_slash_commands 2>/dev/null || true
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
# Status values: pending, incomplete, running, failed, completed

get_queue_file() {
    local agent_root
    agent_root=$(get_agent_root)
    echo "$agent_root/queue.jsonl"
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
                        pending)    echo -e "  ○ ${task}" ;;
                        issues)     echo -e "  ${CYAN}◆${NC} ${task} (issues)" ;;
                        incomplete) echo -e "  ${YELLOW}◐${NC} ${task} (incomplete)" ;;
                        running)    echo -e "  ${BLUE}●${NC} ${task} (running)" ;;
                        failed)     echo -e "  ${RED}✗${NC} ${task} (failed)" ;;
                        completed)  echo -e "  ${GREEN}✓${NC} ${task}" ;;
                        *)          echo -e "  ○ ${task}" ;;
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
    local taskname=""

    if [ $# -gt 0 ]; then
        taskname="$1"
    fi

    taskname=$(sanitize_taskname "$taskname") || exit 1

    if [ -z "$taskname" ]; then
        echo -e "${RED}Error: Task name required${NC}"
        echo "Usage: metagent task <taskname>"
        exit 1
    fi

    local agent_root
    agent_root=$(get_agent_root)
    local task_dir="$agent_root/tasks/${taskname}"
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
    local stage=""
    local override_next=""
    local task="${METAGENT_TASK:-}"

    # Parse flags and stage
    while [ $# -gt 0 ]; do
        case "$1" in
            --next)
                override_next="$2"
                shift 2
                ;;
            *)
                if [ -z "$stage" ]; then
                    stage="$1"
                fi
                shift
                ;;
        esac
    done

    stage="${stage:-task}"

    local queue_file
    local marker_file=""
    local agent_root
    agent_root=$(get_agent_root)
    marker_file="$agent_root/.done_marker"

    queue_file=$(get_queue_file)

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

    if [ -n "$override_next" ]; then
        local next_valid=false
        local all_stages
        all_stages=$(agent_stages)
        for s in $all_stages; do
            if [ "$s" = "$override_next" ]; then
                next_valid=true
                break
            fi
        done
        if [ "$next_valid" = false ]; then
            echo -e "${RED}Unknown next stage: $override_next${NC}"
            echo "Valid stages: $all_stages"
            exit 1
        fi
    fi

    if [ -z "$task" ] && [ "$stage" != "task" ]; then
        task=$(find_unique_task "$queue_file" "$stage" "running" || true)
        if [ -z "$task" ]; then
            task=$(find_unique_task "$queue_file" "$stage" "pending" || true)
        fi
        if [ -n "$task" ]; then
            echo -e "${YELLOW}Warning: METAGENT_TASK not set; using '$task' from queue.${NC}"
        else
            echo -e "${RED}Error: METAGENT_TASK not set and no unique task found for stage '$stage'.${NC}"
            exit 1
        fi
    fi

    # Get next stage: use override if provided, otherwise ask agent
    local next_stage=""
    if [ -n "$override_next" ]; then
        next_stage="$override_next"
    elif [ "$stage" = "task" ]; then
        next_stage="completed"
    else
        next_stage=$(agent_next_stage "$stage")
    fi
    agent_finish_message "$stage"

    # Update queue file if we have a task name and next stage
    if [ -n "$task" ] && [ -n "$next_stage" ] && [ -f "$queue_file" ]; then
        local temp_file="${queue_file}.tmp"

        # Get current status to determine if we should propagate "issues"
        local current_status=""
        local task_entry
        task_entry=$(grep "\"task\":\"$task\"" "$queue_file" 2>/dev/null || true)
        if [ -n "$task_entry" ]; then
            current_status=$(json_field "$task_entry" "status")
        fi

        # Determine new status
        local new_status="pending"
        if [ "$next_stage" = "completed" ]; then
            new_status="completed"
        elif [ "$current_status" = "issues" ]; then
            # Propagate issues status through stages
            new_status="issues"
        elif [ "$stage" = "review" ] && [ -n "$override_next" ]; then
            # Coming from review with issues - mark status as issues
            new_status="issues"
        fi

        while IFS= read -r line; do
            if echo "$line" | grep -q "\"task\":\"$task\""; then
                # Update stage and status
                echo "$line" | sed "s/\"stage\":\"[^\"]*\"/\"stage\":\"$next_stage\"/" | sed "s/\"status\":\"[^\"]*\"/\"status\":\"$new_status\"/"
            else
                echo "$line"
            fi
        done < "$queue_file" > "$temp_file"
        mv "$temp_file" "$queue_file"
        echo "Advanced '$task' to stage: $next_stage"
    fi

    # Signal orchestrator by writing marker file
    mkdir -p "$(dirname "$marker_file")"
    {
        echo "stage=$stage"
        echo "next=$next_stage"
        echo "task=$task"
    } > "$marker_file"
}

# ============================================================================
# Start Command - Interactive task creation and orchestration
# ============================================================================

do_start() {
    local queue_file
    queue_file=$(get_queue_file)
    local initial_stage
    initial_stage=$(agent_initial_stage)
    local initial_prompt
    initial_prompt=$(agent_prompt_for_stage "$initial_stage" "")
    local agent_root
    agent_root=$(get_agent_root)
    local marker_file="$agent_root/.done_marker"

    # Verify metagent is installed
    if [ ! -f "$initial_prompt" ]; then
        echo -e "${RED}Error: metagent not installed. Run 'metagent install' first.${NC}"
        exit 1
    fi

    # Setup environment for metagent finish
    export METAGENT_AGENT="$AGENT"

    cleanup() {
        rm -f "$marker_file"
        # Kill claude if still running (use SIGINT for graceful exit)
        if [ -n "$CLAUDE_PID" ] && kill -0 "$CLAUDE_PID" 2>/dev/null; then
            kill -INT "$CLAUDE_PID" 2>/dev/null || true
            wait "$CLAUDE_PID" 2>/dev/null
        fi
        if [ -n "$taskname" ]; then
            local task_line status
            task_line=$(grep "\"task\":\"$taskname\"" "$queue_file" 2>/dev/null || true)
            status=$(json_field "$task_line" "status")
            if [ "$status" = "running" ]; then
                update_task_field "$taskname" "status" "incomplete"
            fi
        fi
    }
    trap cleanup EXIT

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Starting new task...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    local handoff_stage=""
    if type agent_handoff_stage &>/dev/null; then
        handoff_stage=$(agent_handoff_stage)
    fi

    local orchestrated_stages=""
    if type agent_orchestrated_stages &>/dev/null; then
        orchestrated_stages=$(agent_orchestrated_stages)
    else
        local stages
        stages=$(agent_stages)
        for s in $stages; do
            if [ -n "$handoff_stage" ] && [ "$s" = "$handoff_stage" ]; then
                break
            fi
            if [ "$s" = "completed" ]; then
                break
            fi
            orchestrated_stages="$orchestrated_stages $s"
        done
    fi

    # Start with interview, then loop through stages until handoff
    local taskname=""
    local stage="interview"
    local prompt_file="$initial_prompt"

    echo -e "${CYAN}Starting interview...${NC}"
    echo ""

    # Main loop - runs interview → spec → planning → handoff
    while true; do

        if [ ! -f "$prompt_file" ]; then
            echo -e "${RED}Error: Prompt file not found: ${prompt_file}${NC}"
            exit 1
        fi

        rm -f "$marker_file"

        # Build prompt and run claude interactively with prompt as argument
        local prompt_text
        if [ "$stage" = "interview" ]; then
            prompt_text="$(render_prompt "$prompt_file" "$taskname")"
        else
            prompt_text="Task: $taskname

$(render_prompt "$prompt_file" "$taskname")"
        fi
        $(get_cli_cmd) "$prompt_text" &
        CLAUDE_PID=$!

        # Monitor for marker file or claude exit
        local marker_found=false
        while kill -0 "$CLAUDE_PID" 2>/dev/null; do
            if [ -f "$marker_file" ]; then
                marker_found=true
                read_marker "$marker_file"
                rm -f "$marker_file"

                local marker_stage
                marker_stage="$MARKER_STAGE"
                local marker_next
                marker_next="$MARKER_NEXT"

                echo ""

                if [ -n "$handoff_stage" ] && [ "$marker_next" = "$handoff_stage" ]; then
                    echo -e "${GREEN}✓${NC} $(agent_stage_label "$marker_stage") phase complete"
                    echo ""
                    echo -e "${CYAN}Review with Claude. Exit when ready.${NC}"
                    wait "$CLAUDE_PID" 2>/dev/null

                    echo ""
                    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                    echo -e "${GREEN}Ready to continue${NC}"
                    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                    echo ""
                    echo "Task '$taskname' is ready."
                    echo ""
                    echo "Run 'metagent run $taskname' or 'metagent run-queue' to start."
                    exit 0
                fi

                echo -e "${GREEN}✓${NC} $(agent_stage_label "$marker_stage") phase complete"
                kill -INT "$CLAUDE_PID" 2>/dev/null || true
                wait "$CLAUDE_PID" 2>/dev/null
                echo ""
                sleep 1
                break
            fi
            sleep 0.5
        done

        if [ "$marker_found" = false ] && [ -f "$marker_file" ]; then
            marker_found=true
            read_marker "$marker_file"
            rm -f "$marker_file"

            local marker_stage
            marker_stage="$MARKER_STAGE"
            local marker_next
            marker_next="$MARKER_NEXT"

            echo ""

            if [ -n "$handoff_stage" ] && [ "$marker_next" = "$handoff_stage" ]; then
                echo -e "${GREEN}✓${NC} $(agent_stage_label "$marker_stage") phase complete"
                echo ""
                echo -e "${GREEN}Task '$taskname' is ready.${NC}"
                echo ""
                echo "Run 'metagent run $taskname' or 'metagent run-queue' to start."
                exit 0
            fi

            echo -e "${GREEN}✓${NC} $(agent_stage_label "$marker_stage") phase complete"
            echo ""
        fi

        if [ "$marker_found" = true ]; then
            # Update for next iteration
            taskname="${MARKER_TASK:-$taskname}"
            stage="$MARKER_NEXT"
            prompt_file=$(agent_prompt_for_stage "$stage" "$taskname")
            export METAGENT_TASK="$taskname"
            update_task_field "$taskname" "status" "running"

            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo -e "${BLUE}Task: ${taskname} | Stage: ${stage}${NC}"
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
            continue
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
    local agent_root
    agent_root=$(get_agent_root)
    local marker_file="$agent_root/.done_marker"

    if [ ! -f "$prompt_file" ]; then
        echo -e "${RED}Error: Prompt file not found: ${prompt_file}${NC}"
        return 1
    fi

    # Setup environment for metagent finish
    export METAGENT_TASK="$taskname"
    export METAGENT_AGENT="$AGENT"
    rm -f "$marker_file"

    # Run Claude with the prompt as argument
    echo -e "${CYAN}Task: ${taskname}${NC}"
    echo ""
    local prompt_text
    prompt_text="Task: $taskname

$(render_prompt "$prompt_file" "$taskname")"

    # Run claude in background and monitor for marker file
    $(get_cli_cmd) "$prompt_text" &
    local CLAUDE_PID=$!

    # Track if we found marker (vs user interrupt)
    local marker_triggered=false

    # Monitor for marker file, send SIGINT (ctrl-c) to claude when found
    while kill -0 "$CLAUDE_PID" 2>/dev/null; do
        if [ -f "$marker_file" ]; then
            marker_triggered=true
            kill -INT "$CLAUDE_PID" 2>/dev/null || true
            break
        fi
        sleep 0.5
    done

    wait "$CLAUDE_PID" 2>/dev/null || true

    # Check if metagent finish was called
    if [ -f "$marker_file" ]; then
        read_marker "$marker_file"
        echo "[DEBUG] Marker found: stage=$MARKER_STAGE next=$MARKER_NEXT task=$MARKER_TASK" >&2
        rm -f "$marker_file"
        if [ "$MARKER_NEXT" = "completed" ]; then
            echo "[DEBUG] Returning 2 (task complete)" >&2
            return 2  # Task complete
        fi
        if [ -n "$MARKER_NEXT" ]; then
            echo "[DEBUG] Returning 0 (stage complete)" >&2
            return 0  # Stage complete
        fi
        echo "[DEBUG] Marker found but MARKER_NEXT empty, falling through" >&2
    else
        echo "[DEBUG] No marker file found at $marker_file" >&2
    fi

    # Claude exited without marker - was it interrupted or natural exit?
    if [ "$marker_triggered" = true ]; then
        # We triggered the kill, marker should have been there
        echo "[DEBUG] Returning 1 (marker_triggered but no marker)" >&2
        return 1
    fi

    # User interrupted (Ctrl+C) or Claude exited on its own
    echo "[DEBUG] Returning 130 (interrupted)" >&2
    return 130  # Signal interrupted
}

do_run_queue() {
    local queue_file
    queue_file=$(get_queue_file)

    if [ ! -f "$queue_file" ]; then
        echo -e "${YELLOW}No tasks${NC}"
        exit 0
    fi

    # Reset any stale "running" tasks to "incomplete" (from previous interrupted runs)
    local temp_file="${queue_file}.tmp"
    while IFS= read -r line; do
        if echo "$line" | grep -q "\"status\":\"running\""; then
            echo "$line" | sed 's/"status":"running"/"status":"incomplete"/'
        else
            echo "$line"
        fi
    done < "$queue_file" > "$temp_file"
    mv "$temp_file" "$queue_file"

    local running_task=""
    local interrupted=false
    cleanup() {
        if [ -n "$running_task" ]; then
            # Don't mark as incomplete if task already completed
            local task_line stage
            task_line=$(grep "\"task\":\"$running_task\"" "$queue_file" 2>/dev/null || true)
            stage=$(json_field "$task_line" "stage")
            if [ "$stage" != "completed" ]; then
                update_task_field "$running_task" "status" "incomplete"
            fi
        fi
    }
    trap cleanup EXIT
    trap 'interrupted=true; exit 130' INT

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
            # Pick up pending or incomplete tasks
            found_line=$(grep "\"stage\":\"$s\"" "$queue_file" 2>/dev/null | grep -E "\"status\":\"(pending|incomplete)\"" | head -1 || true)
            if [ -n "$found_line" ]; then
                taskname=$(json_field "$found_line" "task")
                stage="$s"
                break
            fi
        done

        # No pending/incomplete tasks
        if [ -z "$taskname" ]; then
            break
        fi

        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BLUE}Task: ${taskname} | Stage: ${stage}${NC}"
        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""

        running_task="$taskname"
        update_task_field "$taskname" "status" "running"

        run_stage "$taskname" "$stage"
        local result=$?

        if [ $result -eq 0 ]; then
            echo ""
            echo -e "${GREEN}✓${NC} '$taskname' stage complete"
        elif [ $result -eq 2 ]; then
            echo ""
            echo -e "${CYAN}↻${NC} '$taskname' continuing..."
        elif [ $result -eq 130 ]; then
            # User interrupted - exit the queue
            update_task_field "$taskname" "status" "incomplete"
            echo ""
            echo -e "${YELLOW}⊘${NC} '$taskname' interrupted"
            break
        else
            update_task_field "$taskname" "status" "failed"
            echo ""
            echo -e "${RED}✗${NC} '$taskname' failed at stage: $stage"
        fi

        running_task=""
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

    local agent_root
    agent_root=$(get_agent_root)
    local marker_file="$agent_root/.done_marker"
    local task_dir="$agent_root/tasks/${taskname}"
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

    export METAGENT_AGENT="$AGENT"
    export METAGENT_TASK="$taskname"

    cleanup() {
        rm -f "$marker_file"
        if [ -n "$taskname" ]; then
            local task_line stage status
            task_line=$(grep "\"task\":\"$taskname\"" "$queue_file" 2>/dev/null || true)
            stage=$(json_field "$task_line" "stage")
            status=$(json_field "$task_line" "status")
            # Only mark incomplete if not completed and currently running
            if [ "$stage" != "completed" ] && [ "$status" = "running" ]; then
                update_task_field "$taskname" "status" "incomplete"
            fi
        fi
    }
    trap cleanup EXIT

    echo -e "${BLUE}Starting loop for: ${taskname} (stage: ${current_stage})${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""

    local loop_count=0
    while true; do
        loop_count=$((loop_count + 1))
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

        rm -f "$marker_file"
        update_task_field "$taskname" "status" "running"

        # Build prompt and pass as argument for interactive UI
        local prompt_text
        prompt_text="Task: $taskname

$(render_prompt "$prompt_file" "$taskname")"

        # Run claude interactively with prompt as argument
        $(get_cli_cmd) "$prompt_text" &
        local CLAUDE_PID=$!

        # Monitor for marker file, send SIGINT (ctrl-c) to claude when found
        while kill -0 "$CLAUDE_PID" 2>/dev/null; do
            if [ -f "$marker_file" ]; then
                kill -INT "$CLAUDE_PID" 2>/dev/null || true
                break
            fi
            sleep 0.5
        done

        wait "$CLAUDE_PID" 2>/dev/null || true

        # Check if marker was found
        local marker_found=false
        if [ -f "$marker_file" ]; then
            marker_found=true
            rm -f "$marker_file"
            echo ""
            echo -e "${GREEN}Stage completed, continuing...${NC}"
            echo ""
            sleep 2
        fi

        if [ "$marker_found" = true ]; then
            continue
        fi

        # No marker - claude exited without signaling, stop loop
        update_task_field "$taskname" "status" "incomplete"
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}Session ended. Run 'metagent run $taskname' to continue.${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        exit 0
    done
}

do_review() {
    local taskname
    taskname=$(sanitize_taskname "$1") || exit 1
    local focus_prompt="$2"

    if [ -z "$taskname" ]; then
        echo -e "${RED}Error: Task name required${NC}"
        echo "Usage: metagent review <taskname> [focus]"
        exit 1
    fi

    local agent_root
    agent_root=$(get_agent_root)
    local task_dir="$agent_root/tasks/${taskname}"
    local prompt_file
    prompt_file=$(agent_review_prompt)

    if [ ! -d "$task_dir" ]; then
        echo -e "${RED}Error: Task '$taskname' not found${NC}"
        exit 1
    fi

    if [ ! -f "$prompt_file" ]; then
        echo -e "${RED}Error: Review prompt not found. Run 'metagent install' first.${NC}"
        exit 1
    fi

    echo -e "${BLUE}Reviewing task: ${taskname}${NC}"
    if [ -n "$focus_prompt" ]; then
        echo -e "${CYAN}Focus: ${focus_prompt}${NC}"
    fi
    echo ""

    export METAGENT_AGENT="$AGENT"
    export METAGENT_TASK="$taskname"

    # Default to codex for review (user can override with --model claude)
    if [ "$MODEL" = "claude" ] && [ -z "$METAGENT_MODEL" ]; then
        MODEL="codex"
    fi

    # Build focus section if provided
    local focus_section=""
    if [ -n "$focus_prompt" ]; then
        focus_section="## FOCUS AREA

The user has requested special attention to:
> $focus_prompt

Prioritize investigating this area first, then continue with full review."
    fi

    local prompt_text
    prompt_text="Task: $taskname

$(render_prompt "$prompt_file" "$taskname" | sed "s|{focus_section}|$focus_section|g")"

    $(get_cli_cmd) "$prompt_text"
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
        --model)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                echo -e "${RED}Error: --model requires a value (claude, codex)${NC}"
                exit 1
            fi
            MODEL="$2"
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

# No command after options - run interview flow
if [ $# -eq 0 ]; then
    do_start
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
    review)
        do_review "$@"
        ;;
    *)
        echo -e "${RED}Error: Unknown command '$COMMAND'${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
