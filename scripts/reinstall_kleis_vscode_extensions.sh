#!/bin/bash
# Reinstall Kleis VS Code Extension
# Builds the unified kleis binary with Z3 support
# LSP and DAP run in-process via 'kleis server'
#
# IMPORTANT: Close VS Code before running this script!

set -e

KLEIS_ROOT="/Users/eatik_1/Documents/git/cee/kleis"

# Check if VS Code is running
if pgrep -x "Code" > /dev/null 2>&1; then
    echo "⚠️  VS Code is running. Please close it first (Cmd+Q)"
    echo "   Then run this script again."
    exit 1
fi

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

# Also build standalone repl (used by VS Code REPL panel)
echo ""
echo "🔨 Building standalone repl binary..."
cargo build --release --bin repl

echo ""
echo "📦 Building VS Code extension..."

# Switch to Node 20
source ~/.nvm/nvm.sh
nvm use 20

# Go to the vscode-kleis directory
cd "$KLEIS_ROOT/vscode-kleis"

# Install npm dependencies (required for vscode-languageclient)
echo "   Installing npm dependencies..."
npm install --silent

# Rebuild the VSIX (includes node_modules thanks to .vscodeignore fix)
npm run package

# Get the version from package.json
VERSION=$(node -p "require('./package.json').version")
VSIX_FILE="kleis-${VERSION}.vsix"

echo "Built ${VSIX_FILE}"

# Remove old extension versions
echo ""
echo "🗑️  Removing old extension versions..."
rm -rf ~/.vscode/extensions/eatikrh.kleis-*

# Install fresh
echo ""
echo "📦 Installing extension..."
code --install-extension "${VSIX_FILE}"

# Verify installation
if ls ~/.vscode/extensions/eatikrh.kleis-* > /dev/null 2>&1; then
    echo "   ✅ Extension installed"
    # Verify node_modules included
    if ls ~/.vscode/extensions/eatikrh.kleis-*/node_modules/vscode-languageclient > /dev/null 2>&1; then
        echo "   ✅ Dependencies bundled correctly"
    else
        echo "   ⚠️  Warning: vscode-languageclient not found in extension"
    fi
else
    echo "   ❌ Installation failed"
    exit 1
fi

# Update VS Code user settings to point to correct binaries
echo ""
echo "🔧 Updating VS Code settings..."
VSCODE_SETTINGS=~/Library/Application\ Support/Code/User/settings.json
if [[ -f "$VSCODE_SETTINGS" ]]; then
    # Replace any old kleis paths with correct ones
    sed -i '' "s|/Users/eatik_1/Downloads/kleis-copy/kleis/target/release/kleis-lsp|$KLEIS_ROOT/target/release/kleis|g" "$VSCODE_SETTINGS" 2>/dev/null || true
    sed -i '' "s|/Users/eatik_1/Downloads/kleis-copy/kleis/target/release/kleis|$KLEIS_ROOT/target/release/kleis|g" "$VSCODE_SETTINGS" 2>/dev/null || true
    sed -i '' "s|/Users/eatik_1/Downloads/kleis-copy/kleis/target/release/repl|$KLEIS_ROOT/target/release/repl|g" "$VSCODE_SETTINGS" 2>/dev/null || true
    echo "   Updated VS Code user settings"
fi

# Create/update workspace settings
mkdir -p "$KLEIS_ROOT/.vscode"
cat > "$KLEIS_ROOT/.vscode/settings.json" << EOF
{
  // Workspace settings for Kleis extension
  "kleis.serverPath": "$KLEIS_ROOT/target/release/kleis",
  "kleis.replPath": "$KLEIS_ROOT/target/release/repl",
  "kleis.trace.server": "messages"
}
EOF
echo "   Created workspace .vscode/settings.json"

# Also update vscode-kleis workspace settings
mkdir -p "$KLEIS_ROOT/vscode-kleis/.vscode"
cat > "$KLEIS_ROOT/vscode-kleis/.vscode/settings.json" << EOF
{
  // Workspace settings for Kleis extension
  "kleis.serverPath": "$KLEIS_ROOT/target/release/kleis",
  "kleis.replPath": "$KLEIS_ROOT/target/release/repl",
  "kleis.trace.server": "messages"
}
EOF
echo "   Created vscode-kleis/.vscode/settings.json"

echo ""
echo "=============================================="
echo "✅ Extension v${VERSION} installed successfully!"
echo "=============================================="
echo ""
echo "   Binaries:"
echo "   • $KLEIS_ROOT/target/release/kleis"
echo "   • $KLEIS_ROOT/target/release/repl"
echo ""
echo "   Subcommands:"
echo "   • kleis server         - Unified LSP + DAP (VS Code uses this)"
echo "   • kleis eval <expr>    - Evaluate expression"
echo "   • kleis eval -f <file> - Evaluate file"
echo "   • kleis check <file>   - Parse/type-check a file"
echo ""
echo "   Now open VS Code and open a .kleis file."
echo ""
