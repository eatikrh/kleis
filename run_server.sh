#!/bin/bash
# Run Kleis server with proper Z3 configuration
#
# Usage: ./run_server.sh
#
# This script sets the required Z3_SYS_Z3_HEADER environment variable
# before running the server, so you don't have to remember it!

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

echo "🔧 Setting Z3_SYS_Z3_HEADER=$Z3_SYS_Z3_HEADER"
echo "🚀 Starting Kleis server..."
echo ""

# Kill any existing server on port 3000
lsof -ti:3000 | xargs kill -9 2>/dev/null
sleep 1

# Run the server
cargo run --bin server

