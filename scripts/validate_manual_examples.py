#!/usr/bin/env python3
"""
Validate Kleis code examples in the manual.

This script extracts all ```kleis code blocks from the manual markdown files
and checks them for known deprecated patterns and syntax issues.

Usage:
    python3 scripts/validate_manual_examples.py
"""

import os
import re
import sys
from pathlib import Path

# Known deprecated patterns (regex, description, suggestion)
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
    
    # Check for potential issues - but NOT typed params like λ (x : T) . body
    # Only flag λ(x). without type annotation (Haskell-style)
    (r'λ\s*\([a-zA-Z_][a-zA-Z0-9_]*\)\s*\.',
     "Potential syntax issue: λ(x).body should be λ x . body",
     "Use 'λ x . body' for untyped or 'λ (x : T) . body' for typed params"),
]

# Patterns that are VALID (to avoid false positives)
VALID_PATTERNS = [
    r'--\s*$',  # Empty comment line
    r'd/dx',    # If it's in a comment or string
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


def check_deprecated_patterns(code: str, line_offset: int, filepath: str) -> list[str]:
    """Check for deprecated patterns in code."""
    issues = []
    
    for line_num, line in enumerate(code.split('\n'), 1):
        # Skip if line is inside a string (simple heuristic)
        if line.count('"') >= 2:
            continue
            
        for pattern, description, suggestion in DEPRECATED_PATTERNS:
            if re.search(pattern, line):
                # Check it's not a valid pattern
                is_valid = any(re.search(vp, line) for vp in VALID_PATTERNS)
                if not is_valid:
                    actual_line = line_offset + line_num
                    issues.append(
                        f"  Line {actual_line}: {description}\n"
                        f"    Code: {line.strip()}\n"
                        f"    Suggestion: {suggestion}"
                    )
    
    return issues


def validate_basic_syntax(code: str, line_offset: int, filepath: str) -> list[str]:
    """Basic syntax validation for common errors."""
    issues = []
    
    # Check for unbalanced braces/parens (simple check)
    open_parens = code.count('(')
    close_parens = code.count(')')
    if open_parens != close_parens:
        issues.append(
            f"  Block starting at line {line_offset}: "
            f"Unbalanced parentheses ({open_parens} open, {close_parens} close)"
        )
    
    open_braces = code.count('{')
    close_braces = code.count('}')
    if open_braces != close_braces:
        issues.append(
            f"  Block starting at line {line_offset}: "
            f"Unbalanced braces ({open_braces} open, {close_braces} close)"
        )
    
    return issues


def validate_file(filepath: Path) -> list[str]:
    """Validate a single markdown file."""
    issues = []
    
    try:
        content = filepath.read_text(encoding='utf-8')
    except Exception as e:
        return [f"  Error reading file: {e}"]
    
    blocks = extract_kleis_blocks(content, str(filepath))
    
    for line_offset, code in blocks:
        # Skip empty blocks
        if not code.strip():
            continue
            
        # Check deprecated patterns
        pattern_issues = check_deprecated_patterns(code, line_offset, str(filepath))
        issues.extend(pattern_issues)
        
        # Basic syntax validation
        syntax_issues = validate_basic_syntax(code, line_offset, str(filepath))
        issues.extend(syntax_issues)
    
    return issues


def main():
    """Main entry point."""
    # Find the manual directory
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    manual_src = project_root / "docs" / "manual" / "src"
    
    if not manual_src.exists():
        print(f"❌ Manual source directory not found: {manual_src}")
        sys.exit(1)
    
    print("🔍 Validating Kleis examples in the manual...\n")
    
    # Find all markdown files
    md_files = find_markdown_files(manual_src)
    print(f"Found {len(md_files)} markdown files\n")
    
    total_issues = 0
    files_with_issues = 0
    
    for filepath in sorted(md_files):
        relative_path = filepath.relative_to(project_root)
        issues = validate_file(filepath)
        
        if issues:
            files_with_issues += 1
            print(f"❌ {relative_path}")
            for issue in issues:
                print(issue)
                total_issues += 1
            print()
    
    # Summary
    print("-" * 60)
    if total_issues == 0:
        print(f"✅ All {len(md_files)} files passed validation!")
        print("   No deprecated patterns or syntax issues found.")
        sys.exit(0)
    else:
        print(f"❌ Found {total_issues} issue(s) in {files_with_issues} file(s)")
        print("\nPlease fix the issues above before committing.")
        sys.exit(1)


if __name__ == "__main__":
    main()

