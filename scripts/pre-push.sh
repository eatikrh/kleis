#!/bin/bash
#
# Pre-push hook: Run quality gates before allowing push
# This enforces the quality gate rules from .cursorrules
#
# Install with: bash scripts/install-git-hooks.sh
#

echo "🔍 Running pre-push quality gates..."
echo ""

# Set Z3 paths (required for build and doc tests)
# Adjust for your platform:
# - macOS ARM: /opt/homebrew/opt/z3/
# - macOS Intel: /usr/local/opt/z3/
# - Linux: /usr/
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
export LIBRARY_PATH=/opt/homebrew/opt/z3/lib:${LIBRARY_PATH:-}

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
echo "   CRITICAL: Running 'cargo test --all' (includes vendored crates)"
echo "   This includes integration tests and vendored Z3 doc tests!"
echo ""

# Run tests and capture output
if ! cargo test --all 2>&1 > /tmp/kleis_test_output.txt; then
    echo ""
    echo "❌ Tests failed!"
    echo "   Some tests did not pass"
    tail -30 /tmp/kleis_test_output.txt
    echo ""
    echo "   Run: cargo test"
    echo "   to see failures"
    echo ""
    exit 1
fi

echo "✅ All tests passed"
echo ""

# Gate 4: Validate manual examples with STRICT mode (actually parse code blocks!)
echo "4️⃣  Validating manual documentation examples (strict mode)..."
echo "   Running actual 'kleis --check' on all code blocks"
if ! python3 scripts/validate_manual_examples.py --strict 2>&1 | tee /tmp/kleis_manual_output.txt | grep -q "All.*files passed"; then
    echo ""
    echo "❌ Manual validation failed!"
    echo "   Some documentation examples have parse errors"
    echo "   Run: python3 scripts/validate_manual_examples.py --strict --verbose"
    echo ""
    exit 1
fi
echo "✅ Manual examples validated (all code blocks parse correctly)"
echo ""

# Gate 5: Regenerate sitemap if SUMMARY.md changed
echo "5️⃣  Checking sitemap..."
if python3 scripts/generate_sitemap.py > /tmp/kleis_sitemap_output.txt 2>&1; then
    # Check if sitemap changed
    if ! git diff --quiet sitemap.xml 2>/dev/null; then
        echo "   📝 Sitemap updated (manual structure changed)"
        echo "   ⚠️  Please commit sitemap.xml and push again:"
        echo "      git add sitemap.xml"
        echo "      git commit --amend --no-edit"
        echo "      git push"
        echo ""
        exit 1
    else
        echo "✅ Sitemap up to date"
    fi
else
    echo "⚠️  Sitemap generation skipped (script not found or failed)"
fi
echo ""

echo "🎉 All quality gates passed! Proceeding with push..."
echo ""
echo "📊 Summary:"
echo "   • Code formatted correctly"
echo "   • Clippy completed"
echo "   • All tests passing (unit + integration)"
echo "   • Manual examples validated"
echo "   • Sitemap verified"
echo ""

