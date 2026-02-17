#!/bin/bash
# Build the unified kleis binary
#
# Usage:
#   ./scripts/build-kleis.sh              # Release build with axiom verification
#   ./scripts/build-kleis.sh --debug      # Debug build
#   ./scripts/build-kleis.sh --numerical  # With numerical features (BLAS/LAPACK)
#   ./scripts/build-kleis.sh --minimal    # Without Z3 (no axiom verification)
#   ./scripts/build-kleis.sh --clean      # Full cargo clean before build
#   ./scripts/build-kleis.sh --no-gc      # Skip automatic stale artifact cleanup

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
DO_CLEAN=false
SKIP_GC=false

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
        --clean)
            DO_CLEAN=true
            ;;
        --no-gc)
            SKIP_GC=true
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
            echo "  --clean      Full cargo clean before building"
            echo "  --no-gc      Skip automatic stale artifact cleanup"
            echo "  --help       Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                      # Standard release build"
            echo "  $0 --numerical          # With BLAS/LAPACK support"
            echo "  $0 --debug --numerical  # Debug build with numerical"
            echo "  $0 --clean              # Clean rebuild"
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

# ---- Artifact cleanup ----
# Rust's target/ grows unboundedly with incremental builds and can reach tens of
# GB / hundreds of thousands of files, which chokes editors that scan the workspace.

TARGET_SIZE_LIMIT_GB=5  # warn & gc above this

if [ "$DO_CLEAN" = true ]; then
    echo -e "${YELLOW}♻${NC}  --clean: running cargo clean"
    cargo clean 2>/dev/null || true
elif [ "$SKIP_GC" = false ] && [ -d "target" ]; then
    # Measure target/ size (in GB, macOS + Linux compatible)
    TARGET_KB=$(du -sk target 2>/dev/null | awk '{print $1}')
    TARGET_GB=$(( TARGET_KB / 1048576 ))

    if [ "$TARGET_GB" -ge "$TARGET_SIZE_LIMIT_GB" ]; then
        echo -e "${YELLOW}♻${NC}  target/ is ${TARGET_GB}GB (limit: ${TARGET_SIZE_LIMIT_GB}GB) — cleaning stale artifacts"

        # For release builds: purge debug artifacts (the bulk of the bloat)
        if [ "$BUILD_TYPE" = "release" ] && [ -d "target/debug" ]; then
            DU_DEBUG=$(du -sh target/debug 2>/dev/null | awk '{print $1}')
            echo -e "   Removing target/debug/ (${DU_DEBUG})..."
            rm -rf target/debug
        fi

        # For debug builds: purge release artifacts
        if [ "$BUILD_TYPE" = "debug" ] && [ -d "target/release" ]; then
            DU_RELEASE=$(du -sh target/release 2>/dev/null | awk '{print $1}')
            echo -e "   Removing target/release/ (${DU_RELEASE})..."
            rm -rf target/release
        fi

        # Always clean incremental caches older than 7 days
        if [ -d "target" ]; then
            STALE=$(find target -name "incremental" -type d -maxdepth 3 2>/dev/null)
            if [ -n "$STALE" ]; then
                for dir in $STALE; do
                    OLD_FILES=$(find "$dir" -type f -mtime +7 2>/dev/null | wc -l | tr -d ' ')
                    if [ "$OLD_FILES" -gt 0 ]; then
                        echo -e "   Pruning $dir ($OLD_FILES stale files)..."
                        find "$dir" -type f -mtime +7 -delete 2>/dev/null
                        find "$dir" -type d -empty -delete 2>/dev/null
                    fi
                done
            fi
        fi

        # Report new size
        NEW_KB=$(du -sk target 2>/dev/null | awk '{print $1}')
        NEW_GB=$(( NEW_KB / 1048576 ))
        NEW_MB=$(( NEW_KB / 1024 ))
        echo -e "${GREEN}✓${NC}  target/ now ${NEW_MB}MB"
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

    # Install to PATH locations so the LSP extension picks up the latest binary
    if [ "$BUILD_TYPE" = "release" ]; then
        INSTALLED=()
        if [ -d "$HOME/.cargo/bin" ]; then
            cp "$BINARY" "$HOME/.cargo/bin/kleis"
            INSTALLED+=("~/.cargo/bin/kleis")
        fi
        if [ -d "$HOME/bin" ]; then
            ln -sf "$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")" "$HOME/bin/kleis"
            INSTALLED+=("~/bin/kleis")
        fi
        if [ ${#INSTALLED[@]} -gt 0 ]; then
            echo -e "${GREEN}✓ Installed to:${NC} ${INSTALLED[*]}"
        fi
    fi

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

