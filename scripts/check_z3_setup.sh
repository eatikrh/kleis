#!/bin/bash
# Z3 Setup Health Check
# Run this to verify Z3 integration is working correctly

set -e

echo "🔍 Checking Z3 setup..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

failure() {
    echo -e "${RED}❌ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check system architecture
echo "1. System Architecture"
SYSTEM_ARCH=$(uname -m)
echo "   System: $SYSTEM_ARCH"

RUST_ARCH=$(rustc --version --verbose | grep host | awk '{print $2}')
echo "   Rust:   $RUST_ARCH"

if [[ "$SYSTEM_ARCH" == "arm64" && "$RUST_ARCH" == "aarch64-apple-darwin" ]]; then
    success "Architecture match"
elif [[ "$SYSTEM_ARCH" == "x86_64" && "$RUST_ARCH" == "x86_64-apple-darwin" ]]; then
    success "Architecture match"
else
    failure "Architecture mismatch! Rust toolchain doesn't match system."
    echo "   Run: rustup default stable-aarch64-apple-darwin (for ARM)"
    echo "   Or:  rustup default stable-x86_64-apple-darwin (for Intel)"
    exit 1
fi
echo ""

# Check Z3 installation
echo "2. Z3 Installation"
if command -v z3 &> /dev/null; then
    Z3_PATH=$(which z3)
    Z3_VERSION=$(z3 --version | head -1)
    success "Z3 installed: $Z3_PATH"
    echo "   Version: $Z3_VERSION"
else
    failure "Z3 not found in PATH"
    echo "   Install: brew install z3 (macOS)"
    echo "   Or:      sudo apt-get install libz3-dev (Linux)"
    exit 1
fi
echo ""

# Check Z3 library
echo "3. Z3 Library"
if [[ "$SYSTEM_ARCH" == "arm64" ]]; then
    LIB_PATH="/opt/homebrew/opt/z3/lib/libz3.dylib"
    HEADER_PATH="/opt/homebrew/opt/z3/include/z3.h"
else
    LIB_PATH="/usr/local/opt/z3/lib/libz3.dylib"
    HEADER_PATH="/usr/local/opt/z3/include/z3.h"
fi

if [ -f "$LIB_PATH" ]; then
    success "Library found: $LIB_PATH"
else
    failure "Library not found: $LIB_PATH"
    echo "   Check Z3 installation"
    exit 1
fi

if [ -f "$HEADER_PATH" ]; then
    success "Header found: $HEADER_PATH"
else
    failure "Header not found: $HEADER_PATH"
    echo "   Check Z3 installation"
    exit 1
fi
echo ""

# Check .cargo/config.toml
echo "4. Cargo Configuration"
if [ -f ".cargo/config.toml" ]; then
    success "Config found: .cargo/config.toml"
    
    # Check if it has the right content
    if grep -q "rustflags" .cargo/config.toml && grep -q "Z3_SYS_Z3_HEADER" .cargo/config.toml; then
        success "Config has Z3 settings"
    else
        warning "Config exists but may be missing Z3 settings"
    fi
else
    failure "Config not found: .cargo/config.toml"
    echo "   This file should be created automatically"
    exit 1
fi
echo ""

# Check local Z3 source
echo "5. Local Z3 Source"
if [ -d "../Z3/z3.rs/z3" ]; then
    success "Z3 Rust bindings found: ../Z3/z3.rs/z3"
else
    failure "Z3 Rust bindings not found: ../Z3/z3.rs/z3"
    echo "   Clone: git clone https://github.com/prove-rs/z3.rs.git ../Z3/z3.rs"
    exit 1
fi
echo ""

# Try building
echo "6. Build Test"
if cargo build --quiet 2>/dev/null; then
    success "Build successful"
else
    failure "Build failed"
    echo "   Run: cargo build (to see errors)"
    exit 1
fi
echo ""

# Try running Z3 tests
echo "7. Z3 Tests"
if cargo test --quiet --test z3_axiom_experiments 2>/dev/null; then
    success "Z3 tests pass (7/7)"
else
    failure "Z3 tests failed"
    echo "   Run: cargo test --test z3_axiom_experiments (to see errors)"
    exit 1
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
success "All checks passed! Z3 integration ready 🚀"
echo ""
echo "Next steps:"
echo "  • cargo test --test z3_axiom_experiments"
echo "  • cargo test --test z3_kleis_grammar_tests"
echo "  • cargo test --test z3_e_unification_tests"
echo ""

