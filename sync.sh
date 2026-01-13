#!/bin/bash
# sync.sh - Sync generic prompts to a target repository (or all tracked repos)
#
# Usage:
#   ./sync.sh /path/to/target/repo    # Sync to specific repo
#   ./sync.sh --all                   # Sync to all repos that have .metagent-source
#   ./sync.sh --list                  # List repos that would be synced
#   ./sync.sh --dry-run /path/to/repo # Show what would be synced without doing it
#
# This will update ONLY the generic prompts (not project-specific files):
# - BOOTSTRAP_PROMPT.md
# - PLANNING_PROMPT.md
# - README.md
# - RECOVERY_PROMPT.md
# - REFRESH_PROMPT.md
# - SPEC_PROMPT.md
# - scripts/spec.sh
#
# It will NOT touch:
# - AGENTS.md (project-specific)
# - TECHNICAL_STANDARDS.md (project-specific)
# - tasks/* (project-specific)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Generic prompts to sync
GENERIC_PROMPTS=(
    "BOOTSTRAP_PROMPT.md"
    "PLANNING_PROMPT.md"
    "README.md"
    "RECOVERY_PROMPT.md"
    "REFRESH_PROMPT.md"
    "SPEC_PROMPT.md"
)

GENERIC_SCRIPTS=(
    "spec.sh"
)

DRY_RUN=false

sync_repo() {
    local target_repo="$1"
    local agents_dir="$target_repo/.agents/code"

    if [ ! -d "$agents_dir" ]; then
        echo -e "${RED}Error: No .agents/code/ found in $target_repo${NC}"
        return 1
    fi

    echo -e "${BLUE}Syncing to: ${target_repo}${NC}"

    # Sync generic prompts
    for prompt in "${GENERIC_PROMPTS[@]}"; do
        local source="$SCRIPT_DIR/prompts/$prompt"
        local dest="$agents_dir/$prompt"

        if [ -f "$source" ]; then
            if [ "$DRY_RUN" = true ]; then
                if [ -f "$dest" ]; then
                    if ! diff -q "$source" "$dest" > /dev/null 2>&1; then
                        echo -e "  ${CYAN}[would update]${NC} $prompt"
                    else
                        echo -e "  ${GREEN}[unchanged]${NC} $prompt"
                    fi
                else
                    echo -e "  ${CYAN}[would create]${NC} $prompt"
                fi
            else
                cp "$source" "$dest"
                echo -e "  ${GREEN}✓${NC} $prompt"
            fi
        else
            echo -e "  ${YELLOW}⚠${NC} $prompt (source not found)"
        fi
    done

    # Sync scripts
    for script in "${GENERIC_SCRIPTS[@]}"; do
        local source="$SCRIPT_DIR/scripts/$script"
        local dest="$agents_dir/scripts/$script"

        if [ -f "$source" ]; then
            if [ "$DRY_RUN" = true ]; then
                if [ -f "$dest" ]; then
                    if ! diff -q "$source" "$dest" > /dev/null 2>&1; then
                        echo -e "  ${CYAN}[would update]${NC} scripts/$script"
                    else
                        echo -e "  ${GREEN}[unchanged]${NC} scripts/$script"
                    fi
                else
                    echo -e "  ${CYAN}[would create]${NC} scripts/$script"
                fi
            else
                mkdir -p "$agents_dir/scripts"
                cp "$source" "$dest"
                chmod +x "$dest"
                echo -e "  ${GREEN}✓${NC} scripts/$script"
            fi
        fi
    done

    # Update sync timestamp
    if [ "$DRY_RUN" = false ]; then
        echo "$SCRIPT_DIR" > "$agents_dir/.metagent-source"
        date +%Y-%m-%d >> "$agents_dir/.metagent-source"
    fi

    echo ""
}

find_tracked_repos() {
    # Find repos that have been installed from this metagent
    # Look for .metagent-source files that reference this directory
    local found_repos=()

    # Search common parent directories
    for search_dir in "$HOME/dev" "$HOME/projects" "$HOME/code" "$HOME/repos" "$(dirname "$SCRIPT_DIR")"; do
        if [ -d "$search_dir" ]; then
            while IFS= read -r -d '' marker; do
                local repo_dir="$(dirname "$(dirname "$(dirname "$marker")")")"
                if grep -q "$SCRIPT_DIR" "$marker" 2>/dev/null; then
                    found_repos+=("$repo_dir")
                fi
            done < <(find "$search_dir" -name ".metagent-source" -print0 2>/dev/null)
        fi
    done

    # Remove duplicates
    printf '%s\n' "${found_repos[@]}" | sort -u
}

show_usage() {
    echo "Usage:"
    echo "  ./sync.sh /path/to/repo     Sync to specific repository"
    echo "  ./sync.sh --all             Sync to all tracked repositories"
    echo "  ./sync.sh --list            List tracked repositories"
    echo "  ./sync.sh --dry-run /path   Show what would change"
    echo ""
    echo "Generic files synced:"
    for prompt in "${GENERIC_PROMPTS[@]}"; do
        echo "  - $prompt"
    done
    for script in "${GENERIC_SCRIPTS[@]}"; do
        echo "  - scripts/$script"
    done
    echo ""
    echo "Project-specific files (NOT synced):"
    echo "  - AGENTS.md"
    echo "  - TECHNICAL_STANDARDS.md"
    echo "  - tasks/*"
}

# Parse arguments
case "${1:-}" in
    --help|-h)
        show_usage
        exit 0
        ;;
    --list)
        echo -e "${BLUE}Tracked repositories:${NC}"
        repos=$(find_tracked_repos)
        if [ -z "$repos" ]; then
            echo "  (none found)"
        else
            echo "$repos" | while read -r repo; do
                if [ -n "$repo" ]; then
                    echo "  $repo"
                fi
            done
        fi
        exit 0
        ;;
    --all)
        repos=$(find_tracked_repos)
        if [ -z "$repos" ]; then
            echo -e "${YELLOW}No tracked repositories found${NC}"
            exit 0
        fi
        echo -e "${BLUE}Syncing to all tracked repositories...${NC}"
        echo ""
        echo "$repos" | while read -r repo; do
            if [ -n "$repo" ]; then
                sync_repo "$repo"
            fi
        done
        echo -e "${GREEN}Done!${NC}"
        exit 0
        ;;
    --dry-run)
        DRY_RUN=true
        if [ -z "$2" ]; then
            echo -e "${RED}Error: --dry-run requires a target path${NC}"
            show_usage
            exit 1
        fi
        TARGET_REPO="$(cd "$2" 2>/dev/null && pwd)" || {
            echo -e "${RED}Error: Target directory does not exist: $2${NC}"
            exit 1
        }
        echo -e "${CYAN}Dry run - showing what would change:${NC}"
        echo ""
        sync_repo "$TARGET_REPO"
        exit 0
        ;;
    "")
        echo -e "${RED}Error: Target repository path required${NC}"
        echo ""
        show_usage
        exit 1
        ;;
    *)
        TARGET_REPO="$(cd "$1" 2>/dev/null && pwd)" || {
            echo -e "${RED}Error: Target directory does not exist: $1${NC}"
            exit 1
        }
        sync_repo "$TARGET_REPO"
        echo -e "${GREEN}Sync complete!${NC}"
        exit 0
        ;;
esac
