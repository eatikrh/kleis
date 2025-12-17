#!/bin/bash
#
# Pre-push hook: Run quality gates before allowing push
# This enforces the quality gate rules from .cursorrules
#
# Install with: bash scripts/install-git-hooks.sh
#

echo "🔍 Running pre-push quality gates..."
echo ""

# Set Z3 header path (required for build)
# Adjust for your platform:
# - macOS ARM: /opt/homebrew/opt/z3/include/z3.h
# - macOS Intel: /usr/local/opt/z3/include/z3.h
# - Linux: /usr/include/z3.h
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h

# Gate 1: Check formatting
echo "1️⃣  Checking code formatting..."
if ! cargo fmt --all -- --check 2>&1; then
    echo ""
    echo "❌ Formatting check failed!"
    echo "   Fix with: cargo fmt --all"
    echo ""
    exit 1
fi
echo "✅ Formatting check passed"
echo ""

# Gate 2: Run clippy (STRICT - no warnings allowed)
echo "2️⃣  Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1 > /tmp/kleis_clippy_output.txt; then
    echo ""
    echo "❌ Clippy failed!"
    echo "   Fix ALL warnings before pushing"
    echo ""
    tail -30 /tmp/kleis_clippy_output.txt
    echo ""
    exit 1
fi
echo "✅ Clippy passed (no warnings)"
echo ""

# Gate 3: Run ALL tests (unit + integration) - THE CRITICAL ONE
echo "3️⃣  Running ALL tests (unit + integration)..."
echo "   CRITICAL: Running 'cargo test' (not --lib)"
echo "   This includes integration tests that caught bugs today!"
echo ""

# Run tests and capture output
if ! cargo test 2>&1 | tee /tmp/kleis_test_output.txt | grep -q "test result: ok"; then
    echo ""
    echo "❌ Tests failed!"
    echo "   Some tests did not pass"
    echo "   Run: cargo test"
    echo "   to see failures"
    echo ""
    exit 1
fi

# Check for any FAILED in output
if grep -q "FAILED" /tmp/kleis_test_output.txt; then
    echo ""
    echo "❌ Some tests FAILED!"
    echo "   Run: cargo test"
    echo "   to see which tests failed"
    echo ""
    exit 1
fi

echo "✅ All tests passed"
echo ""

# Gate 4: Validate manual examples
echo "4️⃣  Validating manual documentation examples..."
if ! python3 scripts/validate_manual_examples.py 2>&1 | tee /tmp/kleis_manual_output.txt | grep -q "All.*files passed"; then
    echo ""
    echo "❌ Manual validation failed!"
    echo "   Some documentation examples have issues"
    echo "   Run: python3 scripts/validate_manual_examples.py"
    echo ""
    exit 1
fi
echo "✅ Manual examples validated"
echo ""

echo "🎉 All quality gates passed! Proceeding with push..."
echo ""
echo "📊 Summary:"
echo "   • Code formatted correctly"
echo "   • Clippy completed"
echo "   • All tests passing (unit + integration)"
echo "   • Manual examples validated"
echo ""

