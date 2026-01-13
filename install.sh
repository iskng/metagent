#!/bin/bash
# install.sh - Install Ralph workflow into a target repository
#
# Usage: ./install.sh /path/to/target/repo
#
# This will:
# 1. Create .agents/code/ directory structure
# 2. Copy all generic prompts
# 3. Copy template files for AGENTS.md and TECHNICAL_STANDARDS.md
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

# Validate arguments
if [ -z "$1" ]; then
    echo -e "${RED}Error: Target repository path required${NC}"
    echo "Usage: ./install.sh /path/to/target/repo"
    echo ""
    echo "Examples:"
    echo "  ./install.sh ~/projects/my-app"
    echo "  ./install.sh ../other-repo"
    exit 1
fi

TARGET_REPO="$(cd "$1" 2>/dev/null && pwd)" || {
    echo -e "${RED}Error: Target directory does not exist: $1${NC}"
    exit 1
}

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

# Check if .agents/code already exists
if [ -d "$TARGET_REPO/.agents/code" ]; then
    echo -e "${YELLOW}Warning: .agents/code/ already exists in target${NC}"
    read -p "Overwrite? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

echo -e "${BLUE}Installing Ralph workflow to: ${TARGET_REPO}${NC}"

# Create directory structure
echo -e "${BLUE}Creating directory structure...${NC}"
mkdir -p "$TARGET_REPO/.agents/code/scripts"
mkdir -p "$TARGET_REPO/.agents/code/tasks"

# Copy generic prompts
echo -e "${BLUE}Copying generic prompts...${NC}"
cp "$SCRIPT_DIR/prompts/BOOTSTRAP_PROMPT.md" "$TARGET_REPO/.agents/code/"
cp "$SCRIPT_DIR/prompts/PLANNING_PROMPT.md" "$TARGET_REPO/.agents/code/"
cp "$SCRIPT_DIR/prompts/README.md" "$TARGET_REPO/.agents/code/"
cp "$SCRIPT_DIR/prompts/RECOVERY_PROMPT.md" "$TARGET_REPO/.agents/code/"
cp "$SCRIPT_DIR/prompts/REFRESH_PROMPT.md" "$TARGET_REPO/.agents/code/"
cp "$SCRIPT_DIR/prompts/SPEC_PROMPT.md" "$TARGET_REPO/.agents/code/"

# Copy scripts
echo -e "${BLUE}Copying scripts...${NC}"
cp "$SCRIPT_DIR/scripts/spec.sh" "$TARGET_REPO/.agents/code/scripts/"
chmod +x "$TARGET_REPO/.agents/code/scripts/spec.sh"

# Copy templates (only if they don't exist - don't overwrite project-specific config)
if [ ! -f "$TARGET_REPO/.agents/code/AGENTS.md" ]; then
    echo -e "${BLUE}Copying AGENTS.md template...${NC}"
    cp "$SCRIPT_DIR/templates/AGENTS.md" "$TARGET_REPO/.agents/code/"
else
    echo -e "${YELLOW}Skipping AGENTS.md (already exists - project-specific)${NC}"
fi

if [ ! -f "$TARGET_REPO/.agents/code/TECHNICAL_STANDARDS.md" ]; then
    echo -e "${BLUE}Copying TECHNICAL_STANDARDS.md template...${NC}"
    cp "$SCRIPT_DIR/templates/TECHNICAL_STANDARDS.md" "$TARGET_REPO/.agents/code/"
else
    echo -e "${YELLOW}Skipping TECHNICAL_STANDARDS.md (already exists - project-specific)${NC}"
fi

# Create .metagent marker file for sync tracking
echo "$SCRIPT_DIR" > "$TARGET_REPO/.agents/code/.metagent-source"
date +%Y-%m-%d >> "$TARGET_REPO/.agents/code/.metagent-source"

echo ""
echo -e "${GREEN}✓ Ralph workflow installed successfully!${NC}"
echo ""
echo "Installed files:"
echo "  $TARGET_REPO/.agents/code/"
ls -la "$TARGET_REPO/.agents/code/" | grep -v "^total" | awk '{print "    " $NF}'
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Run bootstrap to configure for your project:"
echo "   cd $TARGET_REPO"
echo "   cat .agents/code/BOOTSTRAP_PROMPT.md | claude-code"
echo ""
echo "2. Start a task:"
echo "   .agents/code/scripts/spec.sh my-feature"
echo "   cat .agents/code/SPEC_PROMPT.md | claude-code"
