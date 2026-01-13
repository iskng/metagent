#!/bin/bash
# metagent.sh - Install and sync agent workflows
#
# Usage:
#   metagent install [options] [path]    Install agent to repo
#   metagent sync [options] [path]       Sync prompts to repo
#   metagent link                        Add metagent to PATH
#   metagent unlink                      Remove metagent from PATH
#
# Install options:
#   -a, --agent NAME    Agent to install (default: code)
#   --no-commands       Skip installing ~/.claude/commands
#
# Sync options:
#   -a, --agent NAME    Agent to sync (default: code)
#   --all               Sync all tracked repos
#   --dry-run           Preview changes
#
# Common options:
#   -l, --list          List available agents
#   --tracked           List tracked repos
#   -h, --help          Show help

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT="code"
DRY_RUN=false
INSTALL_COMMANDS=true

# ============================================================================
# Helper Functions
# ============================================================================

show_help() {
    echo "metagent - Install and sync agent workflows"
    echo ""
    echo "Usage:"
    echo "  metagent install [options] [path]    Install agent to repo (default: current dir)"
    echo "  metagent sync [options] [path]       Sync prompts to repo (default: current dir)"
    echo "  metagent link                        Add metagent to PATH (~/.local/bin)"
    echo "  metagent unlink                      Remove metagent from PATH"
    echo ""
    echo "Install options:"
    echo "  -a, --agent NAME    Agent to install (default: code)"
    echo "  --no-commands       Skip installing ~/.claude/commands"
    echo ""
    echo "Sync options:"
    echo "  -a, --agent NAME    Agent to sync (default: code)"
    echo "  --all               Sync all tracked repos"
    echo "  --dry-run           Preview changes"
    echo ""
    echo "Common options:"
    echo "  -l, --list          List available agents"
    echo "  --tracked           List tracked repos"
    echo "  -h, --help          Show help"
    echo ""
    echo "Examples:"
    echo "  metagent install                     # Install to current dir"
    echo "  metagent install ~/projects/app      # Install to specific dir"
    echo "  metagent sync                        # Sync current dir"
    echo "  metagent sync --all                  # Sync all tracked repos"
    echo "  metagent link                        # Make 'metagent' available globally"
}

list_agents() {
    echo -e "${BLUE}Available agents:${NC}"
    for agent_dir in "$SCRIPT_DIR"/*/; do
        if [ -d "$agent_dir" ] && [ "$(basename "$agent_dir")" != ".git" ]; then
            agent_name=$(basename "$agent_dir")
            if [ "$agent_name" = "code" ]; then
                echo "  $agent_name (default)"
            else
                echo "  $agent_name"
            fi
        fi
    done
}

find_tracked_repos() {
    local found=()
    for search_dir in "$HOME/dev" "$HOME/projects" "$HOME/code" "$HOME/repos" "$(dirname "$SCRIPT_DIR")"; do
        if [ -d "$search_dir" ]; then
            while IFS= read -r -d '' marker; do
                if grep -q "source=$SCRIPT_DIR" "$marker" 2>/dev/null; then
                    local repo_dir=$(dirname "$(dirname "$(dirname "$marker")")")
                    local agent=$(grep "^agent=" "$marker" 2>/dev/null | cut -d= -f2)
                    found+=("$repo_dir:$agent")
                fi
            done < <(find "$search_dir" -name ".metagent-source" -print0 2>/dev/null)
        fi
    done
    printf '%s\n' "${found[@]}" | sort -u
}

list_tracked() {
    echo -e "${BLUE}Tracked repositories:${NC}"
    repos=$(find_tracked_repos)
    if [ -z "$repos" ]; then
        echo "  (none found)"
    else
        echo "$repos" | while read -r entry; do
            if [ -n "$entry" ]; then
                repo=$(echo "$entry" | cut -d: -f1)
                agent=$(echo "$entry" | cut -d: -f2)
                echo "  $repo ($agent)"
            fi
        done
    fi
}

# ============================================================================
# Install Command
# ============================================================================

do_install() {
    local target_repo="$1"

    # Use current directory if no path given
    if [ -z "$target_repo" ]; then
        target_repo="$(pwd)"
    else
        target_repo="$(cd "$target_repo" 2>/dev/null && pwd)" || {
            echo -e "${RED}Error: Target directory does not exist: $target_repo${NC}"
            exit 1
        }
    fi

    # Validate agent exists
    local agent_dir="$SCRIPT_DIR/$AGENT"
    if [ ! -d "$agent_dir" ]; then
        echo -e "${RED}Error: Agent '$AGENT' not found${NC}"
        list_agents
        exit 1
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

    # Check if .agents/{agent} already exists
    if [ -d "$target_repo/.agents/$AGENT" ]; then
        echo -e "${YELLOW}Warning: .agents/$AGENT/ already exists in target${NC}"
        read -p "Overwrite? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 0
        fi
    fi

    echo -e "${BLUE}Installing '$AGENT' agent to: ${target_repo}${NC}"

    # Create directory structure
    echo -e "${BLUE}Creating directory structure...${NC}"
    mkdir -p "$target_repo/.agents/$AGENT/scripts"
    mkdir -p "$target_repo/.agents/$AGENT/tasks"

    # Copy prompts
    if [ -d "$agent_dir/prompts" ]; then
        echo -e "${BLUE}Copying prompts...${NC}"
        for file in "$agent_dir/prompts"/*; do
            if [ -f "$file" ]; then
                cp "$file" "$target_repo/.agents/$AGENT/"
                echo -e "  ${GREEN}✓${NC} $(basename "$file")"
            fi
        done
    fi

    # Copy scripts
    if [ -d "$agent_dir/scripts" ]; then
        echo -e "${BLUE}Copying scripts...${NC}"
        for file in "$agent_dir/scripts"/*; do
            if [ -f "$file" ]; then
                cp "$file" "$target_repo/.agents/$AGENT/scripts/"
                chmod +x "$target_repo/.agents/$AGENT/scripts/$(basename "$file")"
                echo -e "  ${GREEN}✓${NC} scripts/$(basename "$file")"
            fi
        done
    fi

    # Copy templates (only if they don't exist)
    if [ -d "$agent_dir/templates" ]; then
        echo -e "${BLUE}Copying templates...${NC}"
        for file in "$agent_dir/templates"/*; do
            if [ -f "$file" ]; then
                dest="$target_repo/.agents/$AGENT/$(basename "$file")"
                if [ ! -f "$dest" ]; then
                    cp "$file" "$dest"
                    echo -e "  ${GREEN}✓${NC} $(basename "$file")"
                else
                    echo -e "  ${YELLOW}⊘${NC} $(basename "$file") (already exists - skipped)"
                fi
            fi
        done
    fi

    # Create .metagent marker file
    cat > "$target_repo/.agents/$AGENT/.metagent-source" << EOF
source=$SCRIPT_DIR
agent=$AGENT
installed=$(date +%Y-%m-%d)
EOF

    # Install slash commands
    if [ "$INSTALL_COMMANDS" = true ]; then
        local commands_dir="$HOME/.claude/commands"
        echo -e "${BLUE}Installing slash commands to ${commands_dir}...${NC}"
        mkdir -p "$commands_dir"

        ln -sf "$target_repo/.agents/$AGENT/BOOTSTRAP_PROMPT.md" "$commands_dir/bootstrap.md"
        echo -e "  ${GREEN}✓${NC} /bootstrap -> .agents/$AGENT/BOOTSTRAP_PROMPT.md"

        ln -sf "$target_repo/.agents/$AGENT/SPEC_PROMPT.md" "$commands_dir/spec.md"
        echo -e "  ${GREEN}✓${NC} /spec -> .agents/$AGENT/SPEC_PROMPT.md"

        ln -sf "$target_repo/.agents/$AGENT/PLANNING_PROMPT.md" "$commands_dir/plan.md"
        echo -e "  ${GREEN}✓${NC} /plan -> .agents/$AGENT/PLANNING_PROMPT.md"
        echo ""
    fi

    echo -e "${GREEN}✓ Agent '$AGENT' installed successfully!${NC}"
    echo ""
    echo "Installed to: $target_repo/.agents/$AGENT/"
    if [ "$INSTALL_COMMANDS" = true ]; then
        echo ""
        echo "Slash commands installed:"
        echo "  /bootstrap  - Configure workflow for this repo"
        echo "  /spec       - Start specification phase for a task"
        echo "  /plan       - Generate implementation plan from specs"
    fi
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Run bootstrap to configure for your project:"
    echo "   cd $target_repo"
    if [ "$INSTALL_COMMANDS" = true ]; then
        echo "   /bootstrap"
    else
        echo "   cat .agents/$AGENT/BOOTSTRAP_PROMPT.md | claude-code"
    fi
    echo ""
    echo "2. Start a task:"
    if [ "$INSTALL_COMMANDS" = true ]; then
        echo "   /spec my-feature"
        echo "   /plan my-feature"
    else
        echo "   .agents/$AGENT/scripts/spec.sh my-feature"
        echo "   cat .agents/$AGENT/SPEC_PROMPT.md | claude-code"
    fi
    echo ""
    echo "3. Run build loop:"
    echo "   while :; do cat .agents/$AGENT/tasks/{taskname}/PROMPT.md | claude-code; done"
}

# ============================================================================
# Sync Command
# ============================================================================

sync_repo() {
    local target_repo="$1"
    local agent="$2"
    local agents_dir="$target_repo/.agents/$agent"
    local agent_source="$SCRIPT_DIR/$agent"

    if [ ! -d "$agents_dir" ]; then
        echo -e "${RED}Error: No .agents/$agent/ found in $target_repo${NC}"
        return 1
    fi

    if [ ! -d "$agent_source" ]; then
        echo -e "${RED}Error: Agent '$agent' not found in metagent${NC}"
        return 1
    fi

    echo -e "${BLUE}Syncing '$agent' to: ${target_repo}${NC}"

    # Sync prompts
    if [ -d "$agent_source/prompts" ]; then
        for file in "$agent_source/prompts"/*; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                local dest="$agents_dir/$filename"

                if [ "$DRY_RUN" = true ]; then
                    if [ -f "$dest" ]; then
                        if ! diff -q "$file" "$dest" > /dev/null 2>&1; then
                            echo -e "  ${CYAN}[would update]${NC} $filename"
                        else
                            echo -e "  ${GREEN}[unchanged]${NC} $filename"
                        fi
                    else
                        echo -e "  ${CYAN}[would create]${NC} $filename"
                    fi
                else
                    cp "$file" "$dest"
                    echo -e "  ${GREEN}✓${NC} $filename"
                fi
            fi
        done
    fi

    # Sync scripts
    if [ -d "$agent_source/scripts" ]; then
        for file in "$agent_source/scripts"/*; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                local dest="$agents_dir/scripts/$filename"

                if [ "$DRY_RUN" = true ]; then
                    if [ -f "$dest" ]; then
                        if ! diff -q "$file" "$dest" > /dev/null 2>&1; then
                            echo -e "  ${CYAN}[would update]${NC} scripts/$filename"
                        else
                            echo -e "  ${GREEN}[unchanged]${NC} scripts/$filename"
                        fi
                    else
                        echo -e "  ${CYAN}[would create]${NC} scripts/$filename"
                    fi
                else
                    mkdir -p "$agents_dir/scripts"
                    cp "$file" "$dest"
                    chmod +x "$dest"
                    echo -e "  ${GREEN}✓${NC} scripts/$filename"
                fi
            fi
        done
    fi

    # Update sync timestamp
    if [ "$DRY_RUN" = false ]; then
        cat > "$agents_dir/.metagent-source" << EOF
source=$SCRIPT_DIR
agent=$agent
synced=$(date +%Y-%m-%d)
EOF
    fi

    echo ""
}

do_sync() {
    local target_repo="$1"
    local sync_all="$2"

    if [ "$sync_all" = true ]; then
        repos=$(find_tracked_repos)
        if [ -z "$repos" ]; then
            echo -e "${YELLOW}No tracked repositories found${NC}"
            exit 0
        fi
        echo -e "${BLUE}Syncing all tracked repositories...${NC}"
        echo ""
        echo "$repos" | while read -r entry; do
            if [ -n "$entry" ]; then
                repo=$(echo "$entry" | cut -d: -f1)
                agent=$(echo "$entry" | cut -d: -f2)
                sync_repo "$repo" "$agent"
            fi
        done
        echo -e "${GREEN}Done!${NC}"
    else
        # Use current directory if no path given
        if [ -z "$target_repo" ]; then
            target_repo="$(pwd)"
        else
            target_repo="$(cd "$target_repo" 2>/dev/null && pwd)" || {
                echo -e "${RED}Error: Target directory does not exist: $target_repo${NC}"
                exit 1
            }
        fi

        if [ "$DRY_RUN" = true ]; then
            echo -e "${CYAN}Dry run - showing what would change:${NC}"
            echo ""
        fi

        sync_repo "$target_repo" "$AGENT"

        if [ "$DRY_RUN" = false ]; then
            echo -e "${GREEN}Sync complete!${NC}"
        fi
    fi
}

# ============================================================================
# Link/Unlink Commands
# ============================================================================

do_link() {
    local bin_dir="$HOME/.local/bin"
    mkdir -p "$bin_dir"

    ln -sf "$SCRIPT_DIR/metagent.sh" "$bin_dir/metagent"
    echo -e "${GREEN}✓ Linked metagent to $bin_dir/metagent${NC}"

    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$bin_dir:"* ]]; then
        echo ""
        echo -e "${YELLOW}Note: $bin_dir is not in your PATH${NC}"
        echo "Add this to your shell profile (.bashrc, .zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Then restart your shell or run: source ~/.zshrc"
    else
        echo ""
        echo "You can now run 'metagent' from anywhere."
    fi
}

do_unlink() {
    local bin_dir="$HOME/.local/bin"
    if [ -L "$bin_dir/metagent" ]; then
        rm "$bin_dir/metagent"
        echo -e "${GREEN}✓ Removed metagent from $bin_dir${NC}"
    else
        echo -e "${YELLOW}metagent is not linked in $bin_dir${NC}"
    fi
}

# ============================================================================
# Main
# ============================================================================

# No arguments - show help
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

# Get command
COMMAND="$1"
shift

# Handle global options before command
case "$COMMAND" in
    -h|--help)
        show_help
        exit 0
        ;;
    -l|--list)
        list_agents
        exit 0
        ;;
    --tracked)
        list_tracked
        exit 0
        ;;
esac

# Parse command-specific options
TARGET_REPO=""
SYNC_ALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--agent)
            AGENT="$2"
            shift 2
            ;;
        -l|--list)
            list_agents
            exit 0
            ;;
        --tracked)
            list_tracked
            exit 0
            ;;
        --no-commands)
            INSTALL_COMMANDS=false
            shift
            ;;
        --all)
            SYNC_ALL=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_help
            exit 1
            ;;
        *)
            TARGET_REPO="$1"
            shift
            ;;
    esac
done

# Execute command
case "$COMMAND" in
    install|i)
        do_install "$TARGET_REPO"
        ;;
    sync|s)
        do_sync "$TARGET_REPO" "$SYNC_ALL"
        ;;
    link)
        do_link
        ;;
    unlink)
        do_unlink
        ;;
    *)
        echo -e "${RED}Error: Unknown command '$COMMAND'${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
