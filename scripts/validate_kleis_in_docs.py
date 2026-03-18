#!/usr/bin/env python3
"""
Validate Kleis code examples in documentation files

Extracts Kleis code from .tex files and tests if they parse correctly.
"""

import re
import subprocess
import sys
from pathlib import Path

def extract_kleis_from_tex(tex_file):
    """Extract Kleis code blocks from verbatim environments in LaTeX files"""
    with open(tex_file, 'r') as f:
        content = f.read()
    
    # Find all verbatim blocks
    pattern = r'\\begin{verbatim}(.*?)\\end{verbatim}'
    blocks = re.findall(pattern, content, re.DOTALL)
    
    # Filter for blocks that look like Kleis code (contain 'structure' or 'operation')
    kleis_blocks = []
    for block in blocks:
        if 'structure' in block or 'operation' in block or 'axiom' in block:
            kleis_blocks.append(block.strip())
    
    return kleis_blocks

def test_kleis_parse(code, source_file):
    """Test if Kleis code parses correctly"""
    # Write to temp file
    temp_file = Path('/tmp/test_kleis_example.kleis')
    temp_file.write_text(code)
    
    # Try to parse with check_parser
    try:
        result = subprocess.run(
            ['cargo', 'run', '--bin', 'check_parser', '--', str(temp_file)],
            capture_output=True,
            text=True,
            timeout=10,
            env={'Z3_SYS_Z3_HEADER': '/opt/homebrew/opt/z3/include/z3.h'}
        )
        
        if result.returncode == 0:
            return True, None
        else:
            return False, result.stderr
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, str(e)

def main():
    # Find all .tex files in docs/
    docs_dir = Path('docs')
    tex_files = list(docs_dir.rglob('*.tex'))
    
    print(f"Found {len(tex_files)} .tex files")
    print()
    
    total_examples = 0
    failed_examples = 0
    
    for tex_file in sorted(tex_files):
        kleis_blocks = extract_kleis_from_tex(tex_file)
        
        if not kleis_blocks:
            continue
        
        print(f"📄 {tex_file.relative_to('.')}")
        print(f"   Found {len(kleis_blocks)} Kleis code blocks")
        
        for i, block in enumerate(kleis_blocks, 1):
            total_examples += 1
            success, error = test_kleis_parse(block, tex_file)
            
            if success:
                print(f"   ✅ Block {i}: Parses correctly")
            else:
                failed_examples += 1
                print(f"   ❌ Block {i}: Parse error")
                print(f"      {error[:200] if error else 'Unknown error'}")
        
        print()
    
    # Summary
    print("=" * 60)
    print(f"Total Kleis examples found: {total_examples}")
    print(f"Passing: {total_examples - failed_examples}")
    print(f"Failing: {failed_examples}")
    
    if failed_examples > 0:
        print()
        print("⚠️  Some documentation examples don't parse!")
        print("   These need to be updated to match the current grammar.")
        return 1
    else:
        print()
        print("✅ All documentation examples parse correctly!")
        return 0

if __name__ == '__main__':
    sys.exit(main())

