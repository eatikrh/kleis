#!/usr/bin/env python3
"""
Direct DAP protocol test - connect to a DAP server and send full initialization sequence
"""

import socket
import json
import time
import subprocess
import sys

def send_dap_message(sock, msg):
    """Send a DAP message with Content-Length header"""
    body = json.dumps(msg)
    header = f"Content-Length: {len(body)}\r\n\r\n"
    sock.sendall(header.encode() + body.encode())
    print(f">>> Sent: {msg.get('command', msg.get('event', 'unknown'))}")

def recv_dap_message(sock, timeout=2.0):
    """Receive a DAP message"""
    sock.settimeout(timeout)
    try:
        # Read Content-Length header
        header = b""
        while b"\r\n\r\n" not in header:
            chunk = sock.recv(1)
            if not chunk:
                return None
            header += chunk
        
        # Parse Content-Length
        header_str = header.decode()
        content_length = 0
        for line in header_str.split("\r\n"):
            if line.lower().startswith("content-length:"):
                content_length = int(line.split(":")[1].strip())
                break
        
        if content_length == 0:
            return None
        
        # Read body
        body = b""
        while len(body) < content_length:
            chunk = sock.recv(content_length - len(body))
            if not chunk:
                break
            body += chunk
        
        msg = json.loads(body.decode())
        msg_type = msg.get('type', 'unknown')
        if msg_type == 'response':
            print(f"<<< Recv: response to {msg.get('command', '?')} (success={msg.get('success')})")
        elif msg_type == 'event':
            print(f"<<< Recv: event {msg.get('event', '?')}")
        return msg
    except socket.timeout:
        return None

def main():
    # Start the kleis server and get DAP port
    print("Starting kleis server...")
    proc = subprocess.Popen(
        ["/Users/eatik_1/Documents/git/cee/kleis/target/release/kleis", "server", "-v"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    
    # Send LSP initialize
    init_req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {"processId": None, "rootUri": "file:///tmp", "capabilities": {}}
    }
    body = json.dumps(init_req)
    msg = f"Content-Length: {len(body)}\r\n\r\n{body}"
    proc.stdin.write(msg.encode())
    proc.stdin.flush()
    time.sleep(0.3)
    
    # Send initialized notification
    init_notif = {"jsonrpc": "2.0", "method": "initialized", "params": {}}
    body = json.dumps(init_notif)
    msg = f"Content-Length: {len(body)}\r\n\r\n{body}"
    proc.stdin.write(msg.encode())
    proc.stdin.flush()
    time.sleep(0.3)
    
    # Send startDebugSession command
    program = "/Users/eatik_1/Documents/git/cee/kleis/examples/example_blocks.kleis"
    exec_cmd = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "workspace/executeCommand",
        "params": {"command": "kleis.startDebugSession", "arguments": [program]}
    }
    body = json.dumps(exec_cmd)
    msg = f"Content-Length: {len(body)}\r\n\r\n{body}"
    proc.stdin.write(msg.encode())
    proc.stdin.flush()
    time.sleep(0.5)
    
    # Read response and get port
    output = proc.stdout.read(2000).decode()
    print("LSP output:", output[:500])
    
    # Extract port from response
    import re
    match = re.search(r'"port"\s*:\s*(\d+)', output)
    if not match:
        print("ERROR: No port found in response")
        proc.terminate()
        return 1
    
    port = int(match.group(1))
    print(f"\n=== Connecting to DAP on port {port} ===\n")
    
    # Connect to DAP
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", port))
    
    seq = 0
    def next_seq():
        nonlocal seq
        seq += 1
        return seq
    
    # DAP: initialize
    send_dap_message(sock, {
        "seq": next_seq(),
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "kleis"}
    })
    resp = recv_dap_message(sock)
    if resp:
        caps = resp.get('body', {})
        print(f"    Capabilities: supportsConfigurationDoneRequest={caps.get('supportsConfigurationDoneRequest')}")
    
    # DAP: launch
    send_dap_message(sock, {
        "seq": next_seq(),
        "type": "request",
        "command": "launch",
        "arguments": {"program": program, "stopOnEntry": True}
    })
    resp = recv_dap_message(sock)
    
    # Check for any events
    while True:
        event = recv_dap_message(sock, timeout=0.5)
        if not event:
            break
    
    # DAP: configurationDone
    send_dap_message(sock, {
        "seq": next_seq(),
        "type": "request",
        "command": "configurationDone"
    })
    resp = recv_dap_message(sock)
    
    # Should receive stopped event now
    print("\nWaiting for stopped event...")
    for i in range(5):
        event = recv_dap_message(sock, timeout=1.0)
        if event:
            if event.get('type') == 'event' and event.get('event') == 'stopped':
                print(f"    ✅ Got stopped event! reason={event.get('body', {}).get('reason')}")
                break
        else:
            print(f"    (timeout {i+1})")
    
    # DAP: stackTrace
    send_dap_message(sock, {
        "seq": next_seq(),
        "type": "request",
        "command": "stackTrace",
        "arguments": {"threadId": 1}
    })
    resp = recv_dap_message(sock)
    if resp:
        frames = resp.get('body', {}).get('stackFrames', [])
        print(f"    Stack frames: {len(frames)}")
        for f in frames:
            print(f"      - {f.get('name')} at line {f.get('line')}")
    
    # Cleanup
    sock.close()
    proc.terminate()
    print("\n=== Test complete ===")
    return 0

if __name__ == "__main__":
    sys.exit(main())

