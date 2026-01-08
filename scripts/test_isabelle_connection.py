#!/usr/bin/env python3
"""Test Isabelle server connection.

Usage:
    python3 scripts/test_isabelle_connection.py <port> <password>

Example:
    python3 scripts/test_isabelle_connection.py 51617 b041e885-1783-4001-afc8-276000c39acc
"""

import socket
import sys
import json
import time

def send_command(sock, command):
    """Send a command and receive response."""
    sock.sendall((command + "\n").encode())
    time.sleep(0.1)
    
    # Read response (may be multiple lines)
    response = b""
    sock.settimeout(2.0)
    try:
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            response += chunk
            if b"\n" in chunk:
                break
    except socket.timeout:
        pass
    
    return response.decode().strip()

def main():
    if len(sys.argv) != 3:
        print("Usage: python3 test_isabelle_connection.py <port> <password>")
        sys.exit(1)
    
    port = int(sys.argv[1])
    password = sys.argv[2]
    
    print(f"Connecting to Isabelle server on port {port}...")
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", port))
    
    # Authenticate
    response = send_command(sock, password)
    print(f"Auth response: {response}")
    
    if not response.startswith("OK"):
        print("Authentication failed!")
        sock.close()
        sys.exit(1)
    
    # Parse server info
    try:
        info_json = response.split(" ", 1)[1]
        info = json.loads(info_json)
        print(f"Connected to: {info.get('isabelle_name', 'unknown')}")
    except:
        pass
    
    # Test echo
    print("\n--- Testing echo command ---")
    response = send_command(sock, 'echo "Hello from Kleis!"')
    print(f"Echo response: {response}")
    
    # Test help
    print("\n--- Testing help command ---")
    response = send_command(sock, 'help')
    print(f"Help response: {response}")
    
    sock.close()
    print("\n✅ Connection test successful!")

if __name__ == "__main__":
    main()





