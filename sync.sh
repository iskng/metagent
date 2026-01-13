#!/bin/bash
# sync.sh - Sync generic prompts to a target repository (or all tracked repos)
#
# Usage:
#   ./sync.sh                           # Sync default agent (code) to current directory
#   ./sync.sh /path/to/repo             # Sync default agent to specific repo
#   ./sync.sh -a code                   # Sync specific agent to current directory
#   ./sync.sh -a code /path/to/repo     # Sync specific agent to specific repo
#   ./sync.sh --all                     # Sync all tracked repos
#   ./sync.sh --list                    # List tracked repos
#   ./sync.sh --dry-run [path]          # Show what would change
#
# This will update ONLY the generic prompts (not project-specific files).
# Project-specific files (AGENTS.md, TECHNICAL_STANDARDS.md, tasks/*) are NOT touched.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT="code"
DRY_RUN=false

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

find_tracked_repos() {
    # Find repos that have been installed from this metagent
    local found=()

    # Search common parent directories
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

    # Remove duplicates and print
    printf '%s\n' "${found[@]}" | sort -u
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

show_usage() {
    echo "Usage:"
    echo "  ./sync.sh                       Sync 'code' agent to current directory"
    echo "  ./sync.sh /path/to/repo         Sync 'code' agent to specific repo"
    echo "  ./sync.sh -a AGENT              Sync specific agent to current directory"
    echo "  ./sync.sh -a AGENT /path        Sync specific agent to specific repo"
    echo "  ./sync.sh --all                 Sync all tracked repositories"
    echo "  ./sync.sh --list                List tracked repositories"
    echo "  ./sync.sh --agents              List available agents"
    echo "  ./sync.sh --dry-run [path]      Show what would change"
    echo ""
    echo "Files synced (generic):"
    echo "  - All files in {agent}/prompts/"
    echo "  - All files in {agent}/scripts/"
    echo ""
    echo "Files NOT synced (project-specific):"
    echo "  - AGENTS.md"
    echo "  - TECHNICAL_STANDARDS.md"
    echo "  - tasks/*"
}

# Parse arguments
TARGET_REPO=""
ACTION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--agent)
            AGENT="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        --agents)
            list_agents
            exit 0
            ;;
        --list)
            ACTION="list"
            shift
            ;;
        --all)
            ACTION="all"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_usage
            exit 1
            ;;
        *)
            TARGET_REPO="$1"
            shift
            ;;
    esac
done

# Handle actions
case "$ACTION" in
    list)
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
        exit 0
        ;;
    all)
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
        exit 0
        ;;
esac

# Default: sync to target repo (or current dir)
if [ -z "$TARGET_REPO" ]; then
    TARGET_REPO="$(pwd)"
else
    TARGET_REPO="$(cd "$TARGET_REPO" 2>/dev/null && pwd)" || {
        echo -e "${RED}Error: Target directory does not exist: $TARGET_REPO${NC}"
        exit 1
    }
fi

if [ "$DRY_RUN" = true ]; then
    echo -e "${CYAN}Dry run - showing what would change:${NC}"
    echo ""
fi

sync_repo "$TARGET_REPO" "$AGENT"

if [ "$DRY_RUN" = false ]; then
    echo -e "${GREEN}Sync complete!${NC}"
fi
