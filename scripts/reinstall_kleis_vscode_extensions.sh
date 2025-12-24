#!/bin/bash
# Reinstall Kleis VS Code Extension
# Builds the unified kleis binary with Z3 support
# LSP and DAP run in-process via 'kleis server'

set -e

KLEIS_ROOT="/Users/eatik_1/Documents/git/cee/kleis"

# Detect platform and set Z3 header path
if [[ "$(uname -m)" == "arm64" ]]; then
    # macOS ARM (Apple Silicon)
    export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
elif [[ "$(uname)" == "Darwin" ]]; then
    # macOS Intel
    export Z3_SYS_Z3_HEADER=/usr/local/opt/z3/include/z3.h
elif [[ "$(uname)" == "Linux" ]]; then
    # Linux
    export Z3_SYS_Z3_HEADER=/usr/include/z3.h
else
    echo "⚠️  Unknown platform. Please set Z3_SYS_Z3_HEADER manually."
    exit 1
fi

echo "🔧 Z3_SYS_Z3_HEADER=$Z3_SYS_Z3_HEADER"

# Build unified kleis binary (includes LSP, REPL, DAP)
echo ""
echo "🔨 Building unified kleis binary (with Z3 axiom verification)..."
cd "$KLEIS_ROOT"
cargo build --release --bin kleis

echo ""
echo "📦 Building VS Code extension..."

# Switch to Node 20
source ~/.nvm/nvm.sh
nvm use 20

# Go to the vscode-kleis directory
cd "$KLEIS_ROOT/vscode-kleis"

# Rebuild the VSIX
npm run package

# Get the version from package.json
VERSION=$(node -p "require('./package.json').version")
VSIX_FILE="kleis-${VERSION}.vsix"

echo "Built ${VSIX_FILE}"

# Uninstall old extension (more reliable than --force)
echo ""
echo "🔄 Reinstalling extension..."
code --uninstall-extension eatikrh.kleis 2>/dev/null || true
sleep 2

# Install fresh
code --install-extension "${VSIX_FILE}"

# Update VS Code user settings to point to correct binaries
echo ""
echo "🔧 Updating VS Code settings..."
VSCODE_SETTINGS=~/Library/Application\ Support/Code/User/settings.json
if [[ -f "$VSCODE_SETTINGS" ]]; then
    # Replace any old kleis paths with correct ones
    sed -i '' "s|/Users/eatik_1/Downloads/kleis-copy/kleis/target/release/kleis-lsp|$KLEIS_ROOT/target/release/kleis|g" "$VSCODE_SETTINGS"
    sed -i '' "s|/Users/eatik_1/Downloads/kleis-copy/kleis/target/release/kleis|$KLEIS_ROOT/target/release/kleis|g" "$VSCODE_SETTINGS"
    sed -i '' "s|/Users/eatik_1/Downloads/kleis-copy/kleis/target/release/repl|$KLEIS_ROOT/target/release/repl|g" "$VSCODE_SETTINGS"
    echo "   Updated VS Code user settings"
fi

# Create/update workspace settings
mkdir -p "$KLEIS_ROOT/.vscode"
cat > "$KLEIS_ROOT/.vscode/settings.json" << EOF
{
  // Workspace settings for Kleis extension
  "kleis.serverPath": "$KLEIS_ROOT/target/release/kleis",
  "kleis.replPath": "$KLEIS_ROOT/target/release/repl",
  "kleis.trace.server": "verbose"
}
EOF
echo "   Created workspace .vscode/settings.json"

echo ""
echo "✅ Extension v${VERSION} reinstalled."
echo ""
echo "   Binary: $KLEIS_ROOT/target/release/kleis"
echo ""
echo "   Subcommands:"
echo "   • kleis server         - Unified LSP + DAP (VS Code uses this)"
echo "   • kleis eval <expr>    - Evaluate expression"
echo "   • kleis eval -f <file> - Evaluate file"
echo "   • kleis check <file>   - Parse/type-check a file"
echo "   • kleis repl           - Interactive REPL (stub - use 'cargo run --bin repl')"
echo ""
echo "   Restart VS Code/Cursor to use it."
