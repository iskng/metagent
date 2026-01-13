#!/bin/bash
# install.sh - Install agent workflow into a target repository
#
# Usage: ./install.sh [options] [/path/to/target/repo]
#
# Options:
#   -a, --agent NAME    Agent to install (default: code)
#   -l, --list          List available agents
#   -h, --help          Show this help
#
# If no path given, installs to current directory.
#
# This will:
# 1. Create .agents/{agent}/ directory structure
# 2. Copy all generic prompts
# 3. Copy template files for project-specific config
# 4. Create empty tasks directory
# 5. Prompt to run bootstrap

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT="code"
TARGET_REPO=""

show_help() {
    echo "Usage: ./install.sh [options] [/path/to/target/repo]"
    echo ""
    echo "Options:"
    echo "  -a, --agent NAME    Agent to install (default: code)"
    echo "  -l, --list          List available agents"
    echo "  -h, --help          Show this help"
    echo ""
    echo "If no path given, installs to current directory."
    echo ""
    echo "Examples:"
    echo "  ./install.sh                      # Install 'code' agent to current dir"
    echo "  ./install.sh ~/projects/my-app    # Install 'code' agent to specific dir"
    echo "  ./install.sh -a code .            # Explicitly specify agent"
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

# Parse arguments
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

# Use current directory if no path given
if [ -z "$TARGET_REPO" ]; then
    TARGET_REPO="$(pwd)"
else
    TARGET_REPO="$(cd "$TARGET_REPO" 2>/dev/null && pwd)" || {
        echo -e "${RED}Error: Target directory does not exist: $TARGET_REPO${NC}"
        exit 1
    }
fi

# Validate agent exists
AGENT_DIR="$SCRIPT_DIR/$AGENT"
if [ ! -d "$AGENT_DIR" ]; then
    echo -e "${RED}Error: Agent '$AGENT' not found${NC}"
    list_agents
    exit 1
fi

# Check if target is a git repo
if [ ! -d "$TARGET_REPO/.git" ]; then
    echo -e "${YELLOW}Warning: Target is not a git repository${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Check if .agents/{agent} already exists
if [ -d "$TARGET_REPO/.agents/$AGENT" ]; then
    echo -e "${YELLOW}Warning: .agents/$AGENT/ already exists in target${NC}"
    read -p "Overwrite? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

echo -e "${BLUE}Installing '$AGENT' agent to: ${TARGET_REPO}${NC}"

# Create directory structure
echo -e "${BLUE}Creating directory structure...${NC}"
mkdir -p "$TARGET_REPO/.agents/$AGENT/scripts"
mkdir -p "$TARGET_REPO/.agents/$AGENT/tasks"

# Copy prompts
if [ -d "$AGENT_DIR/prompts" ]; then
    echo -e "${BLUE}Copying prompts...${NC}"
    for file in "$AGENT_DIR/prompts"/*; do
        if [ -f "$file" ]; then
            cp "$file" "$TARGET_REPO/.agents/$AGENT/"
            echo -e "  ${GREEN}✓${NC} $(basename "$file")"
        fi
    done
fi

# Copy scripts
if [ -d "$AGENT_DIR/scripts" ]; then
    echo -e "${BLUE}Copying scripts...${NC}"
    for file in "$AGENT_DIR/scripts"/*; do
        if [ -f "$file" ]; then
            cp "$file" "$TARGET_REPO/.agents/$AGENT/scripts/"
            chmod +x "$TARGET_REPO/.agents/$AGENT/scripts/$(basename "$file")"
            echo -e "  ${GREEN}✓${NC} scripts/$(basename "$file")"
        fi
    done
fi

# Copy templates (only if they don't exist - don't overwrite project-specific config)
if [ -d "$AGENT_DIR/templates" ]; then
    echo -e "${BLUE}Copying templates...${NC}"
    for file in "$AGENT_DIR/templates"/*; do
        if [ -f "$file" ]; then
            dest="$TARGET_REPO/.agents/$AGENT/$(basename "$file")"
            if [ ! -f "$dest" ]; then
                cp "$file" "$dest"
                echo -e "  ${GREEN}✓${NC} $(basename "$file")"
            else
                echo -e "  ${YELLOW}⊘${NC} $(basename "$file") (already exists - skipped)"
            fi
        fi
    done
fi

# Create .metagent marker file for sync tracking
mkdir -p "$TARGET_REPO/.agents/$AGENT"
cat > "$TARGET_REPO/.agents/$AGENT/.metagent-source" << EOF
source=$SCRIPT_DIR
agent=$AGENT
installed=$(date +%Y-%m-%d)
EOF

echo ""
echo -e "${GREEN}✓ Agent '$AGENT' installed successfully!${NC}"
echo ""
echo "Installed to: $TARGET_REPO/.agents/$AGENT/"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Run bootstrap to configure for your project:"
echo "   cd $TARGET_REPO"
echo "   cat .agents/$AGENT/BOOTSTRAP_PROMPT.md | claude-code"
echo ""
echo "2. Start a task:"
echo "   .agents/$AGENT/scripts/spec.sh my-feature"
echo "   cat .agents/$AGENT/SPEC_PROMPT.md | claude-code"
