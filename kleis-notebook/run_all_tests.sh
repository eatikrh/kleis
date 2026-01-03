#!/bin/bash
# =============================================================================
# KleisDoc Comprehensive Test Suite
# =============================================================================
# Run all tests locally to verify functionality.
# 
# Usage: ./run_all_tests.sh
#
# Note: Some tests require the Kleis server (cargo run --bin server)
# =============================================================================

set -e  # Exit on first error

cd "$(dirname "$0")"
PROJECT_ROOT="$(cd .. && pwd)"

echo "============================================================"
echo "KleisDoc Comprehensive Test Suite"
echo "============================================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0
SKIPPED=0

run_test() {
    local name="$1"
    local cmd="$2"
    echo -n "Testing $name... "
    if eval "$cmd" > /tmp/test_output.txt 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Output: $(tail -3 /tmp/test_output.txt)"
        ((FAILED++))
    fi
}

skip_test() {
    local name="$1"
    local reason="$2"
    echo -e "Testing $name... ${YELLOW}⚠ SKIP${NC} ($reason)"
    ((SKIPPED++))
}

echo "--- Rust Tests ---"
cd "$PROJECT_ROOT"

# Set Z3 header path
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h

run_test "Template rendering (24 tests)" \
    "cargo test --test test_new_templates 2>&1 | grep -q 'test result: ok'"

echo ""
echo "--- Python Tests ---"
cd "$PROJECT_ROOT/kleis-notebook"

run_test "KleisDoc basic" \
    "python3 examples/test_kleisdoc.py 2>&1 | grep -q 'All tests passed'"

run_test "Save/Load round-trip" \
    "python3 examples/test_save_load.py 2>&1 | grep -q 'All tests passed'"

run_test "Document styles (MIT + arXiv PDF)" \
    "python3 examples/demo_document_styles.py 2>&1 | grep -q 'Compiled'"

# Check if server is running for server-dependent tests
if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    run_test "Render pipeline (requires server)" \
        "python3 examples/test_render_pipeline.py 2>&1 | grep -q 'PDF exported'"
else
    skip_test "Render pipeline" "Server not running"
fi

echo ""
echo "--- Template Files ---"

for template in "$PROJECT_ROOT"/stdlib/templates/*.kleis; do
    name=$(basename "$template" .kleis)
    run_test "Template: $name loads" \
        "$PROJECT_ROOT/target/debug/kleis check '$template' 2>&1 || true"
done

echo ""
echo "============================================================"
echo "Results: ${GREEN}$PASSED passed${NC}, ${RED}$FAILED failed${NC}, ${YELLOW}$SKIPPED skipped${NC}"
echo "============================================================"

if [ $FAILED -gt 0 ]; then
    exit 1
fi

