#!/usr/bin/env bash
# ============================================================================
# theory.sh — Toggle Kleis theory MCP for Cursor
# ============================================================================
#
# Usage:
#   scripts/theory.sh on      # Enable (alwaysApply: true)
#   scripts/theory.sh off     # Disable (alwaysApply: false)
#   scripts/theory.sh status  # Show current state
#
# ============================================================================

set -euo pipefail

RULE_FILE=".cursor/rules/kleis-theory.mdc"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RULE_PATH="$PROJECT_ROOT/$RULE_FILE"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

usage() {
    echo "Usage: scripts/theory.sh {on|off|status}"
    echo ""
    echo "  on      Enable Kleis theory MCP (alwaysApply: true)"
    echo "  off     Disable Kleis theory MCP (alwaysApply: false)"
    echo "  status  Show current state"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

if [ ! -f "$RULE_PATH" ]; then
    echo -e "${RED}Error:${NC} $RULE_FILE not found."
    echo "Run from the project root or ensure the rule file exists."
    exit 1
fi

current_state() {
    if grep -q "^alwaysApply: true" "$RULE_PATH"; then
        echo "on"
    else
        echo "off"
    fi
}

case "$1" in
    on)
        sed -i.bak 's/^alwaysApply: false/alwaysApply: true/' "$RULE_PATH" && rm -f "$RULE_PATH.bak"
        echo -e "${GREEN}✓ Theory MCP ENABLED${NC}"
        echo "  The agent can co-author Kleis theories interactively."
        ;;
    off)
        sed -i.bak 's/^alwaysApply: true/alwaysApply: false/' "$RULE_PATH" && rm -f "$RULE_PATH.bak"
        echo -e "${YELLOW}✗ Theory MCP DISABLED${NC}"
        echo "  The agent will not see theory MCP tools."
        ;;
    status)
        STATE=$(current_state)
        if [ "$STATE" = "on" ]; then
            echo -e "${GREEN}● Theory MCP is ON${NC} (alwaysApply: true)"
        else
            echo -e "${YELLOW}○ Theory MCP is OFF${NC} (alwaysApply: false)"
        fi
        echo "  Rule file: $RULE_FILE"
        ;;
    *)
        usage
        ;;
esac
