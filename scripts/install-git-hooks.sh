#!/bin/bash
#
# Install git hooks for quality gates enforcement
# Run this once after cloning the repository
#

echo "Installing git hooks for Kleis quality gates..."

# Copy pre-push hook
cp scripts/pre-push.sh .git/hooks/pre-push
chmod +x .git/hooks/pre-push

echo "✅ Pre-push hook installed"
echo ""
echo "This hook will run before every 'git push' to ensure:"
echo "  1. Code is formatted (cargo fmt --all)"
echo "  2. Clippy passes (cargo clippy --all-targets --all-features)"
echo "  3. ALL tests pass (cargo test - not just --lib!)"
echo "  4. Manual examples validated with STRICT mode (actually parses code blocks!)"
echo ""
echo "To bypass in emergencies: git push --no-verify"
echo "(But don't do this unless absolutely necessary!)"

