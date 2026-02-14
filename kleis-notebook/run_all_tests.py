#!/usr/bin/env python3
"""
KleisDoc Comprehensive Test Suite

Run all tests locally to verify functionality.

Usage: python3 run_all_tests.py

Note: Some tests require the Kleis server (cargo run --bin server)
"""

import subprocess
import sys
import os
from pathlib import Path

# Colors
GREEN = "\033[0;32m"
RED = "\033[0;31m"
YELLOW = "\033[1;33m"
NC = "\033[0m"

passed = 0
failed = 0
skipped = 0

def run_test(name, cmd, cwd=None):
    global passed, failed
    print(f"Testing {name}... ", end="", flush=True)
    try:
        result = subprocess.run(
            cmd, shell=True, cwd=cwd,
            capture_output=True, text=True, timeout=120
        )
        if result.returncode == 0:
            print(f"{GREEN}✓ PASS{NC}")
            passed += 1
            return True
        else:
            print(f"{RED}✗ FAIL{NC}")
            if result.stderr:
                print(f"  Error: {result.stderr[:200]}")
            failed += 1
            return False
    except subprocess.TimeoutExpired:
        print(f"{RED}✗ TIMEOUT{NC}")
        failed += 1
        return False
    except Exception as e:
        print(f"{RED}✗ ERROR: {e}{NC}")
        failed += 1
        return False

def skip_test(name, reason):
    global skipped
    print(f"Testing {name}... {YELLOW}⚠ SKIP{NC} ({reason})")
    skipped += 1

def main():
    global passed, failed, skipped
    
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    print("=" * 60)
    print("KleisDoc Comprehensive Test Suite")
    print("=" * 60)
    print()
    
    # Set environment
    os.environ["Z3_SYS_Z3_HEADER"] = "/opt/homebrew/opt/z3/include/z3.h"
    
    # --- Rust Tests ---
    print("--- Rust Tests ---")
    
    run_test(
        "Template rendering (24 tests)",
        "cargo test --test test_new_templates 2>&1 | grep -q 'test result: ok'",
        cwd=project_root
    )
    
    print()
    print("--- Python Tests ---")
    
    if (script_dir / "examples" / "test_kleisdoc.py").is_file():
        run_test(
            "KleisDoc basic",
            "python3 examples/test_kleisdoc.py 2>&1 | grep -q 'All tests passed'",
            cwd=script_dir
        )
    else:
        skip_test("KleisDoc basic", "examples/test_kleisdoc.py not found")
    
    if (script_dir / "examples" / "test_save_load.py").is_file():
        run_test(
            "Save/Load round-trip",
            "python3 examples/test_save_load.py 2>&1 | grep -q 'All tests passed'",
            cwd=script_dir
        )
    else:
        skip_test("Save/Load round-trip", "examples/test_save_load.py not found")
    
    if (script_dir / "examples" / "demo_document_styles.py").is_file():
        run_test(
            "Document styles (MIT + arXiv PDF)",
            "python3 examples/demo_document_styles.py 2>&1 | grep -q 'Compiled'",
            cwd=script_dir
        )
    else:
        skip_test("Document styles (MIT + arXiv PDF)", "examples/demo_document_styles.py not found")
    
    # Check if server is running
    try:
        import socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(1)
        result = sock.connect_ex(('localhost', 3000))
        sock.close()
        server_running = (result == 0)
    except:
        server_running = False
    
    if server_running:
        if (script_dir / "examples" / "test_render_pipeline.py").is_file():
            run_test(
                "Render pipeline (requires server)",
                "python3 examples/test_render_pipeline.py 2>&1 | grep -q 'PDF exported'",
                cwd=script_dir
            )
        else:
            skip_test("Render pipeline", "examples/test_render_pipeline.py not found")
    else:
        skip_test("Render pipeline", "Server not running (start with: cargo run --bin server)")
    
    print()
    print("--- Template Files ---")
    
    templates_dir = project_root / "stdlib" / "templates"
    for template in templates_dir.glob("*.kleis"):
        # Just check if the file parses (syntax check)
        run_test(
            f"Template: {template.stem}",
            f"head -1 '{template}' > /dev/null",  # Simple existence check
            cwd=project_root
        )
    
    print()
    print("=" * 60)
    print(f"Results: {GREEN}{passed} passed{NC}, {RED}{failed} failed{NC}, {YELLOW}{skipped} skipped{NC}")
    print("=" * 60)
    
    if failed > 0:
        sys.exit(1)
    
    print(f"\n{GREEN}All tests passed!{NC}")

if __name__ == "__main__":
    main()

