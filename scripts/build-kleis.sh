#!/bin/bash
# Build the unified kleis binary
#
# Usage:
#   ./scripts/build-kleis.sh              # Release build with axiom verification
#   ./scripts/build-kleis.sh --debug      # Debug build
#   ./scripts/build-kleis.sh --numerical  # With numerical features (BLAS/LAPACK)
#   ./scripts/build-kleis.sh --minimal    # Without Z3 (no axiom verification)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
BUILD_TYPE="release"
FEATURES=""
EXTRA_FLAGS=""

for arg in "$@"; do
    case $arg in
        --debug)
            BUILD_TYPE="debug"
            ;;
        --numerical)
            FEATURES="${FEATURES},numerical"
            ;;
        --minimal)
            EXTRA_FLAGS="--no-default-features"
            ;;
        --help|-h)
            echo "Build the unified kleis binary"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug      Build debug version (faster compile, slower runtime)"
            echo "  --numerical  Enable numerical features (eigenvalues, SVD, Schur)"
            echo "  --minimal    Disable Z3 (no axiom verification)"
            echo "  --help       Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                      # Standard release build"
            echo "  $0 --numerical          # With BLAS/LAPACK support"
            echo "  $0 --debug --numerical  # Debug build with numerical"
            exit 0
            ;;
    esac
done

# Detect Z3 header location
detect_z3() {
    # macOS ARM (Homebrew)
    if [ -f "/opt/homebrew/opt/z3/include/z3.h" ]; then
        echo "/opt/homebrew/opt/z3/include/z3.h"
        return 0
    fi
    
    # macOS Intel (Homebrew)
    if [ -f "/usr/local/opt/z3/include/z3.h" ]; then
        echo "/usr/local/opt/z3/include/z3.h"
        return 0
    fi
    
    # Linux
    if [ -f "/usr/include/z3.h" ]; then
        echo "/usr/include/z3.h"
        return 0
    fi
    
    # Not found
    return 1
}

# Set up Z3 if not minimal build
if [ "$EXTRA_FLAGS" != "--no-default-features" ]; then
    if Z3_PATH=$(detect_z3); then
        export Z3_SYS_Z3_HEADER="$Z3_PATH"
        echo -e "${GREEN}✓${NC} Z3 found: $Z3_PATH"
    else
        echo -e "${YELLOW}⚠${NC} Z3 not found. Building without axiom verification."
        echo "  Install Z3: brew install z3 (macOS) or apt install libz3-dev (Linux)"
        EXTRA_FLAGS="--no-default-features"
    fi
fi

# Build command
BUILD_CMD="cargo build --bin kleis"

if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_CMD="$BUILD_CMD --release"
fi

if [ -n "$FEATURES" ]; then
    # Remove leading comma if present
    FEATURES="${FEATURES#,}"
    BUILD_CMD="$BUILD_CMD --features $FEATURES"
fi

if [ -n "$EXTRA_FLAGS" ]; then
    BUILD_CMD="$BUILD_CMD $EXTRA_FLAGS"
fi

echo -e "${GREEN}Building:${NC} $BUILD_CMD"
echo ""

# Run build
$BUILD_CMD

# Report result
if [ "$BUILD_TYPE" = "release" ]; then
    BINARY="target/release/kleis"
else
    BINARY="target/debug/kleis"
fi

if [ -f "$BINARY" ]; then
    echo ""
    echo -e "${GREEN}✓ Build successful!${NC}"
    echo "  Binary: $BINARY"
    echo ""
    echo "Usage:"
    echo "  $BINARY server          # Start LSP + DAP server"
    echo "  $BINARY eval '1 + 2'    # Evaluate expression"
    echo "  $BINARY check file.kleis # Check file for errors"
    echo "  $BINARY repl            # Interactive REPL"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

