#!/usr/bin/env bash
# ============================================================================
# policy.sh — Toggle Kleis policy enforcement for Cursor
# ============================================================================
#
# Usage:
#   scripts/policy.sh on      # Enable enforcement (alwaysApply: true)
#   scripts/policy.sh off     # Disable enforcement (alwaysApply: false)
#   scripts/policy.sh status  # Show current state
#
# ============================================================================

set -euo pipefail

RULE_FILE=".cursor/rules/kleis-policy.mdc"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RULE_PATH="$PROJECT_ROOT/$RULE_FILE"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

usage() {
    echo "Usage: scripts/policy.sh {on|off|status}"
    echo ""
    echo "  on      Enable Kleis policy enforcement (alwaysApply: true)"
    echo "  off     Disable Kleis policy enforcement (alwaysApply: false)"
    echo "  status  Show current enforcement state"
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
        echo -e "${GREEN}✓ Policy enforcement ENABLED${NC}"
        echo "  The agent will call check_action before every action."
        ;;
    off)
        sed -i.bak 's/^alwaysApply: true/alwaysApply: false/' "$RULE_PATH" && rm -f "$RULE_PATH.bak"
        echo -e "${YELLOW}✗ Policy enforcement DISABLED${NC}"
        echo "  The agent will not be required to call check_action."
        ;;
    status)
        STATE=$(current_state)
        if [ "$STATE" = "on" ]; then
            echo -e "${GREEN}● Policy enforcement is ON${NC} (alwaysApply: true)"
        else
            echo -e "${YELLOW}○ Policy enforcement is OFF${NC} (alwaysApply: false)"
        fi
        echo "  Rule file: $RULE_FILE"
        ;;
    *)
        usage
        ;;
esac


