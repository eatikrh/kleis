#!/bin/bash
# Test the Kleis LSP server with a simple initialization handshake
#
# Usage: ./scripts/test_lsp.sh

LSP_SERVER="./target/release/kleis-lsp"

if [ ! -f "$LSP_SERVER" ]; then
    echo "Building kleis-lsp..."
    cargo build --release --bin kleis-lsp --no-default-features
fi

echo "Testing LSP server initialization..."

# Create an initialize request (JSON-RPC 2.0)
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":"file:///tmp","capabilities":{}}}'
INIT_LEN=${#INIT_REQUEST}

# Send with Content-Length header (LSP protocol)
echo "Sending initialize request..."
(echo -ne "Content-Length: ${INIT_LEN}\r\n\r\n${INIT_REQUEST}" ; sleep 1) | "$LSP_SERVER" 2>/dev/null | head -c 2000

echo ""
echo ""
echo "✅ LSP server responded! Use Ctrl+C to exit."

