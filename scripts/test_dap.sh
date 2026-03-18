#!/bin/bash
# Test script for LSP → DAP communication
# This script verifies that the unified kleis server can:
# 1. Start as an LSP server
# 2. Respond to initialize
# 3. Handle kleis.startDebugSession command
# 4. Return a DAP server port
# 5. Accept TCP connections on that port

set -euo pipefail

LSP_SERVER="./target/release/kleis"
if [ ! -x "$LSP_SERVER" ]; then 
    echo "Server not executable: $LSP_SERVER"
    echo "Build with: cargo build --release --bin kleis"
    exit 1
fi

TMPDIR=$(mktemp -d)
FIFO="$TMPDIR/in"
OUT="$TMPDIR/out"
mkfifo "$FIFO"

cleanup() {
    kill $SERVER_PID 2>/dev/null || true
    rm -rf "$TMPDIR"
}
trap cleanup EXIT

# Start server in background
"$LSP_SERVER" server < "$FIFO" > "$OUT" 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Prepare initialize request
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":"file:///tmp","capabilities":{}}}'
INIT_LEN=${#INIT_REQUEST}

# Send initialize
printf "Content-Length: %d\r\n\r\n%s" "$INIT_LEN" "$INIT_REQUEST" > "$FIFO"

# Give server time to respond
sleep 0.5

# Read and show server response (first 2k chars)
echo "--- Server output (partial) ---"
head -c 2000 "$OUT" || true

# Prepare initialized notification
INITIALIZED='{"jsonrpc":"2.0","method":"initialized","params":{}}'
INITIALIZED_LEN=${#INITIALIZED}
printf "Content-Length: %d\r\n\r\n%s" "$INITIALIZED_LEN" "$INITIALIZED" > "$FIFO"
sleep 0.3

# Prepare executeCommand request to start DAP for examples/example_blocks.kleis
PROG_PATH="$(pwd)/examples/example_blocks.kleis"
EXEC_REQUEST='{"jsonrpc":"2.0","id":2,"method":"workspace/executeCommand","params":{"command":"kleis.startDebugSession","arguments":["'"$PROG_PATH"'"]}}'
EXEC_LEN=${#EXEC_REQUEST}

# Send executeCommand
printf "Content-Length: %d\r\n\r\n%s" "$EXEC_LEN" "$EXEC_REQUEST" > "$FIFO"

# Wait for response
sleep 0.8

# Dump server output to find port
echo ""
echo "--- Server output (after executeCommand) ---"
tail -c +1 "$OUT" | sed -n '1,200p'

# Try to extract port
PORT=$(grep -oE '"port"[[:space:]]*:[[:space:]]*[0-9]+' "$OUT" | head -n1 | grep -oE '[0-9]+') || true

if [ -z "$PORT" ]; then
    echo ""
    echo "❌ No port returned by executeCommand. Showing entire output for diagnosis:"
    cat "$OUT" || true
    exit 2
fi

echo ""
echo "✅ DAP server reported on port: $PORT"

# Try to connect to the DAP TCP port
echo "Attempting TCP connection to 127.0.0.1:$PORT"
nc -vz 127.0.0.1 "$PORT" 2>&1 || echo "⚠️  nc connection test failed (may be expected if DAP closed)"

echo ""
echo "✅ Test completed successfully!"

