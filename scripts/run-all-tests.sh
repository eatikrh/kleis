#!/bin/bash
#
# Run all 1700+ tests with full output and summary
#
# This script runs the complete test suite including:
# - Unit tests (src/lib.rs)
# - Integration tests (tests/*.rs)
# - Golden tests
# - Z3 integration tests
# - Vendored Z3 library tests
#

set -e

echo "🧪 Running all Kleis tests..."
echo ""

# Set Z3 paths (required for build and tests)
# Adjust for your platform:
# - macOS ARM: /opt/homebrew/opt/z3/
# - macOS Intel: /usr/local/opt/z3/
# - Linux: /usr/
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
export LIBRARY_PATH=/opt/homebrew/opt/z3/lib:${LIBRARY_PATH:-}

# Run all tests and capture output
cargo test --all 2>&1 | tee /tmp/kleis_all_tests.txt

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Test Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Count passed tests (parse "X passed; Y failed; Z ignored")
passed=$(grep "test result:" /tmp/kleis_all_tests.txt | sed 's/.*ok\. //' | awk -F'[; ]' '{sum+=$1} END {print sum}')
failed=$(grep "test result:" /tmp/kleis_all_tests.txt | sed 's/.*ok\. //' | awk -F'[; ]' '{sum+=$4} END {print sum}')
ignored=$(grep "test result:" /tmp/kleis_all_tests.txt | sed 's/.*ok\. //' | awk -F'[; ]' '{sum+=$7} END {print sum}')

echo "   ✅ Passed:  $passed"
echo "   ⏭️  Ignored: $ignored"
echo "   ❌ Failed:  $failed"
echo ""

if [ "$failed" -eq 0 ]; then
    echo "🎉 All tests passed!"
else
    echo "⚠️  Some tests failed. Check output above."
    exit 1
fi

