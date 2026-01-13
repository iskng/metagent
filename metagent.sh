#!/bin/bash
# metagent.sh - Install and sync agent workflows
#
# Usage:
#   metagent link                        Setup metagent globally (first time)
#   metagent install [options] [path]    Install agent to repo
#   metagent sync [options] [path]       Sync prompts to repo
#   metagent unlink                      Remove metagent
#
# Install options:
#   -a, --agent NAME    Agent to install (default: code)
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

# ============================================================================
# Helper Functions
# ============================================================================

show_help() {
    echo "metagent - Install and sync agent workflows"
    echo ""
    echo "Usage:"
    echo "  metagent link                        Setup globally (first time)"
    echo "  metagent install [options] [path]    Install agent to repo"
    echo "  metagent sync [options] [path]       Sync prompts to repo"
    echo "  metagent unlink                      Remove metagent"
    echo ""
    echo "Install/Sync options:"
    echo "  -a, --agent NAME    Agent to use (default: code)"
    echo "  --all               Sync all tracked repos"
    echo "  --dry-run           Preview sync changes"
    echo ""
    echo "Other options:"
    echo "  -l, --list          List available agents"
    echo "  --tracked           List tracked repos"
    echo "  -h, --help          Show help"
    echo ""
    echo "Examples:"
    echo "  metagent link                        # First time setup"
    echo "  metagent install                     # Install to current dir"
    echo "  metagent install ~/projects/app      # Install to specific dir"
    echo "  metagent sync --all                  # Sync all tracked repos"
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

    echo ""
    echo -e "${GREEN}✓ Agent '$AGENT' installed successfully!${NC}"
    echo ""
    echo "Installed to: $target_repo/.agents/$AGENT/"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Run bootstrap to configure for your project:"
    echo "   /bootstrap"
    echo ""
    echo "2. Start a task:"
    echo "   /spec my-feature"
    echo "   /plan my-feature"
    echo ""
    echo "3. Run build loop:"
    echo "   while :; do cat .agents/$AGENT/tasks/{taskname}/PROMPT.md | claude-code; done"
    echo ""
    echo "(If slash commands aren't available, run: metagent link)"
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
    local metagent_dir="$HOME/.metagent"
    local claude_commands="$HOME/.claude/commands"
    local codex_commands="$HOME/.codex/commands"

    # Create directories
    mkdir -p "$bin_dir"
    mkdir -p "$metagent_dir"
    mkdir -p "$claude_commands"
    mkdir -p "$codex_commands"

    # Link metagent to PATH
    ln -sf "$SCRIPT_DIR/metagent.sh" "$bin_dir/metagent"
    echo -e "${GREEN}✓${NC} Linked metagent to $bin_dir/metagent"

    # Copy prompts to ~/.metagent/
    echo -e "${BLUE}Installing prompts to $metagent_dir...${NC}"
    if [ -d "$SCRIPT_DIR/$AGENT/prompts" ]; then
        for file in "$SCRIPT_DIR/$AGENT/prompts"/*; do
            if [ -f "$file" ]; then
                cp "$file" "$metagent_dir/"
                echo -e "  ${GREEN}✓${NC} $(basename "$file")"
            fi
        done
    fi

    # Link slash commands to ~/.metagent/ prompts (for both Claude and Codex)
    echo -e "${BLUE}Installing slash commands...${NC}"
    for commands_dir in "$claude_commands" "$codex_commands"; do
        ln -sf "$metagent_dir/BOOTSTRAP_PROMPT.md" "$commands_dir/bootstrap.md"
        ln -sf "$metagent_dir/SPEC_PROMPT.md" "$commands_dir/spec.md"
        ln -sf "$metagent_dir/PLANNING_PROMPT.md" "$commands_dir/plan.md"
    done
    echo -e "  ${GREEN}✓${NC} /bootstrap"
    echo -e "  ${GREEN}✓${NC} /spec"
    echo -e "  ${GREEN}✓${NC} /plan"

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
    echo "  ~/.metagent/              - Global prompts"
    echo "  ~/.claude/commands/       - Slash commands (/bootstrap, /spec, /plan)"
    echo "  ~/.codex/commands/        - Slash commands (/bootstrap, /spec, /plan)"
}

do_unlink() {
    local bin_dir="$HOME/.local/bin"
    local metagent_dir="$HOME/.metagent"
    local claude_commands="$HOME/.claude/commands"
    local codex_commands="$HOME/.codex/commands"

    # Remove metagent from PATH
    if [ -L "$bin_dir/metagent" ]; then
        rm "$bin_dir/metagent"
        echo -e "${GREEN}✓${NC} Removed $bin_dir/metagent"
    fi

    # Remove slash commands from both Claude and Codex
    for commands_dir in "$claude_commands" "$codex_commands"; do
        for cmd in bootstrap.md spec.md plan.md; do
            if [ -L "$commands_dir/$cmd" ]; then
                rm "$commands_dir/$cmd"
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
    echo -e "${GREEN}✓ metagent unlinked${NC}"
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
