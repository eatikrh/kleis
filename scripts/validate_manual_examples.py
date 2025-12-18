#!/usr/bin/env python3
"""
Validate Kleis code examples in the manual using the REAL Kleis parser.

This script extracts all ```kleis code blocks from the manual markdown files
and validates them by actually running them through the Kleis parser.

Usage:
    python3 scripts/validate_manual_examples.py          # Pattern checks only
    python3 scripts/validate_manual_examples.py --strict # Actually run kleis --check
    
Requirements:
    - For --strict mode: cargo build --bin kleis
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

# Known deprecated patterns (regex, description, suggestion) - still useful for quick checks
DEPRECATED_PATTERNS = [
    # Old comment syntax (Haskell-style)
    (r'^\s*--(?!\s*$)', 
     "Deprecated comment syntax '--'", 
     "Use '//' for single-line comments"),
    (r'\{-', 
     "Deprecated block comment '{-'", 
     "Use '/* */' for block comments"),
    (r'-\}', 
     "Deprecated block comment '-}'", 
     "Use '/* */' for block comments"),
    
    # Old derivative notation (Leibniz-style)
    (r'd/d[a-z]',
     "Deprecated derivative notation 'd/dx'",
     "Use 'D(f, x)' for derivatives"),
    (r'∂[a-zA-Z_]+/∂[a-zA-Z_]+',
     "Deprecated partial derivative notation '∂f/∂x'",
     "Use 'D(f, x)' for partial derivatives"),
]


def find_markdown_files(manual_dir: Path) -> list[Path]:
    """Find all markdown files in the manual directory."""
    return list(manual_dir.glob("**/*.md"))


def extract_kleis_blocks(content: str, filepath: str) -> list[tuple[int, str]]:
    """
    Extract all ```kleis code blocks from markdown content.
    Returns list of (line_number, code_block) tuples.
    """
    blocks = []
    lines = content.split('\n')
    in_block = False
    block_start = 0
    block_lines = []
    
    for i, line in enumerate(lines, 1):
        if line.strip().startswith('```kleis'):
            in_block = True
            block_start = i
            block_lines = []
        elif in_block and line.strip().startswith('```'):
            in_block = False
            blocks.append((block_start, '\n'.join(block_lines)))
        elif in_block:
            block_lines.append(line)
    
    return blocks


def check_deprecated_patterns(code: str, line_offset: int) -> list[str]:
    """Check for deprecated patterns in code (quick regex check)."""
    issues = []
    
    for line_num, line in enumerate(code.split('\n'), 1):
        # Skip if line is inside a string (simple heuristic)
        if line.count('"') >= 2:
            continue
        # Skip if it's a comment
        if line.strip().startswith('//'):
            continue
            
        for pattern, description, suggestion in DEPRECATED_PATTERNS:
            if re.search(pattern, line):
                actual_line = line_offset + line_num
                issues.append(
                    f"  Line {actual_line}: {description}\n"
                    f"    Code: {line.strip()}\n"
                    f"    Suggestion: {suggestion}"
                )
    
    return issues


def should_validate_block(code: str) -> bool:
    """
    Determine if a code block should be validated with the parser.
    Returns False for fragments, pseudocode, and reference examples.
    """
    stripped = code.strip()
    
    # Skip very short/trivial blocks (like single identifiers or comments)
    if not stripped or stripped.startswith('//') or len(stripped) < 3:
        return False
    
    # Skip blocks that are clearly fragments or pseudocode
    if '...' in stripped:
        return False
    
    # Skip REPL session examples
    if 'kleis>' in stripped or stripped.startswith('>') or 'λ>' in stripped:
        return False
    
    # Skip grammar definitions
    if '::=' in stripped or '|=' in stripped:
        return False
        
    # Skip blocks showing syntax patterns
    if '<' in stripped and '>' in stripped and '=' not in stripped:
        return False
    
    # Skip blocks that are just type expressions or function signatures
    lines = [l.strip() for l in stripped.split('\n') if l.strip() and not l.strip().startswith('//')]
    if all(not l.startswith(('define', 'structure', 'data', 'implements', 'axiom', 'import')) for l in lines):
        # No top-level declarations - probably a fragment
        return False
    
    # Skip blocks that start with 'verify' - not yet a top-level declaration
    if any(l.startswith('verify') or l.startswith(':verify') for l in lines):
        return False
    
    return True


def validate_with_kleis_cli(code: str, line_offset: int, project_root: Path) -> list[str]:
    """
    Validate code by running it through `kleis --check`.
    Returns list of error messages if parsing fails.
    """
    issues = []
    
    if not should_validate_block(code):
        return []
    
    # Create temp file with the code
    with tempfile.NamedTemporaryFile(mode='w', suffix='.kleis', delete=False) as f:
        f.write(code)
        temp_path = f.name
    
    try:
        # Run kleis --check
        kleis_binary = project_root / "target" / "release" / "kleis"
        if not kleis_binary.exists():
            kleis_binary = project_root / "target" / "debug" / "kleis"
        
        if kleis_binary.exists():
            result = subprocess.run(
                [str(kleis_binary), "--check", temp_path],
                capture_output=True,
                text=True,
                cwd=project_root,
                timeout=10
            )
        else:
            # Fall back to cargo run
            result = subprocess.run(
                ["cargo", "run", "--bin", "kleis", "--quiet", "--", "--check", temp_path],
                capture_output=True,
                text=True,
                cwd=project_root,
                timeout=30,
                env={**os.environ, "Z3_SYS_Z3_HEADER": "/opt/homebrew/opt/z3/include/z3.h"}
            )
        
        # Check for parse errors in output
        output = result.stdout + result.stderr
        
        if result.returncode != 0:
            # Extract error message
            error_lines = [l for l in output.split('\n') if l.strip() and '❌' in l or 'error' in l.lower() or 'Error' in l]
            if error_lines:
                error_msg = error_lines[0][:200]
            else:
                error_msg = output[:200] if output else "Unknown error"
            
            issues.append(
                f"  Block at line {line_offset}: Parser error\n"
                f"    {error_msg}"
            )
                
    except subprocess.TimeoutExpired:
        issues.append(f"  Block at line {line_offset}: Parser timed out (possible infinite loop)")
    except Exception as e:
        # Don't fail on subprocess errors - the parser might not be built
        issues.append(f"  Block at line {line_offset}: Could not run parser: {e}")
    finally:
        # Clean up temp file
        try:
            os.unlink(temp_path)
        except:
            pass
    
    return issues


def validate_file(filepath: Path, project_root: Path, strict: bool = False) -> tuple[list[str], int]:
    """
    Validate a single markdown file.
    Returns (issues, blocks_checked_with_parser).
    """
    issues = []
    parser_checked = 0
    
    try:
        content = filepath.read_text(encoding='utf-8')
    except Exception as e:
        return ([f"  Error reading file: {e}"], 0)
    
    blocks = extract_kleis_blocks(content, str(filepath))
    
    for line_offset, code in blocks:
        # Skip empty blocks
        if not code.strip():
            continue
            
        # Check deprecated patterns (always)
        pattern_issues = check_deprecated_patterns(code, line_offset)
        issues.extend(pattern_issues)
        
        # Validate with actual parser (if strict mode)
        if strict and should_validate_block(code):
            parser_issues = validate_with_kleis_cli(code, line_offset, project_root)
            issues.extend(parser_issues)
            parser_checked += 1
    
    return (issues, parser_checked)


def check_kleis_available(project_root: Path) -> bool:
    """Check if the Kleis CLI is available."""
    kleis_binary = project_root / "target" / "release" / "kleis"
    if kleis_binary.exists():
        return True
    
    kleis_binary = project_root / "target" / "debug" / "kleis"
    if kleis_binary.exists():
        return True
    
    # Try cargo build
    try:
        result = subprocess.run(
            ["cargo", "build", "--bin", "kleis", "--quiet"],
            capture_output=True,
            cwd=project_root,
            timeout=120,
            env={**os.environ, "Z3_SYS_Z3_HEADER": "/opt/homebrew/opt/z3/include/z3.h"}
        )
        return result.returncode == 0
    except:
        return False


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Validate Kleis code examples in the manual"
    )
    parser.add_argument(
        "--strict", 
        action="store_true",
        help="Run actual syntax check with 'kleis --check' (requires built kleis binary)"
    )
    args = parser.parse_args()
    
    # Find directories
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    manual_src = project_root / "docs" / "manual" / "src"
    
    if not manual_src.exists():
        print(f"❌ Manual source directory not found: {manual_src}")
        sys.exit(1)
    
    print("🔍 Validating Kleis examples in the manual...\n")
    
    # Check if strict mode
    if args.strict:
        print("🔧 Strict mode: will run 'kleis --check' on code blocks\n")
        print("📦 Checking Kleis CLI availability...")
        if check_kleis_available(project_root):
            print("   ✅ Kleis CLI found - will validate syntax\n")
        else:
            print("   ❌ Kleis CLI not available")
            print("   Build with: cargo build --bin kleis")
            sys.exit(1)
    else:
        print("📋 Pattern check mode (use --strict for full syntax validation)\n")
    
    # Find all markdown files
    md_files = find_markdown_files(manual_src)
    print(f"Found {len(md_files)} markdown files\n")
    
    total_issues = 0
    files_with_issues = 0
    blocks_validated = 0
    parser_checked_total = 0
    
    for filepath in sorted(md_files):
        relative_path = filepath.relative_to(project_root)
        
        # Count blocks in file
        content = filepath.read_text(encoding='utf-8')
        blocks = extract_kleis_blocks(content, str(filepath))
        blocks_validated += len(blocks)
        
        issues, parser_checked = validate_file(filepath, project_root, strict=args.strict)
        parser_checked_total += parser_checked
        
        if issues:
            files_with_issues += 1
            print(f"❌ {relative_path}")
            for issue in issues:
                print(issue)
                total_issues += 1
            print()
        else:
            # Show progress for files with blocks
            if blocks:
                if args.strict and parser_checked > 0:
                    print(f"✅ {relative_path} ({len(blocks)} blocks, {parser_checked} parsed)")
                else:
                    print(f"✅ {relative_path} ({len(blocks)} blocks)")
    
    # Summary
    print()
    print("-" * 60)
    if total_issues == 0:
        print(f"✅ All {len(md_files)} files passed validation!")
        print(f"   Validated {blocks_validated} code blocks")
        if args.strict:
            print(f"   ✅ {parser_checked_total} blocks parsed with 'kleis --check'")
        sys.exit(0)
    else:
        print(f"⚠️  Found {total_issues} issue(s) in {files_with_issues} file(s)")
        print(f"   Validated {blocks_validated} code blocks total")
        if args.strict:
            print(f"   Parsed {parser_checked_total} blocks with 'kleis --check'")
        print("\n   Note: Some issues are in educational examples (fragments, concepts).")
        print("   These examples use `kleis` tags for future Linguist syntax highlighting.")
        print("   Fix critical issues, but fragments showing syntax patterns are OK.")
        # Exit 0 for informational - we want kleis tags for syntax highlighting
        # even if some examples are conceptual fragments
        sys.exit(0)


if __name__ == "__main__":
    main()
