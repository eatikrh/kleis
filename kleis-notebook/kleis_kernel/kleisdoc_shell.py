"""
KleisDoc Shell - Thin Python wrapper around Kleis document compilation.

Philosophy: Python is just a shell. Kleis does ALL the heavy lifting.

Documents are .kleis files that:
- Import styles from stdlib/templates/ (e.g., mit_thesis.kleis)
- Import user libraries, data files, equations, etc.
- Define document structure and content
- Export to Typst via compile functions

Kleis handles ALL imports and resolution. Python just:
1. Calls `kleis test <file.kleis>` to compile
2. Captures the Typst output
3. Optionally calls `typst compile` for PDF

Usage:
    from kleis_kernel.kleisdoc_shell import compile_to_typst, compile_to_pdf
    
    # Compile a .kleis document to Typst
    typst = compile_to_typst("my_thesis.kleis")
    
    # Compile to PDF
    compile_to_pdf("my_thesis.kleis", "my_thesis.pdf")
"""

import subprocess
import os
from pathlib import Path
from typing import Optional

# Import kleis_binary module
try:
    from .kleis_binary import find_kleis_binary, find_kleis_root
except ImportError:
    from kleis_binary import find_kleis_binary, find_kleis_root


def compile_to_typst(kleis_file: str) -> Optional[str]:
    """
    Compile a Kleis document to Typst code.
    
    The .kleis file should have an example block that outputs Typst:
    
        example "compile" {
            let typst = compile_mit_thesis(my_thesis)
            out(typst)
        }
    
    Kleis handles all imports (stdlib, user files, etc.).
    
    Args:
        kleis_file: Path to the .kleis document
    
    Returns:
        Typst code as a string, or None if failed
    """
    kleis_path = find_kleis_binary()
    if not kleis_path:
        print("Error: Kleis binary not found")
        return None
    
    kleis_root = find_kleis_root()
    env = os.environ.copy()
    if kleis_root:
        env["KLEIS_ROOT"] = kleis_root
    
    try:
        result = subprocess.run(
            [kleis_path, "test", kleis_file],
            capture_output=True,
            text=True,
            timeout=60,
            env=env
        )
        
        if result.returncode != 0:
            print(f"Kleis error: {result.stderr}")
            return None
        
        # Extract Typst from output
        # Kleis prints strings with surrounding quotes and some escaping
        # Format: "content with \"escaped quotes\" and \\n for newlines"
        output = result.stdout.strip()
        
        # Find the start and end of the Typst content
        # Look for content that starts with #import or #set (Typst preamble)
        typst_start = None
        for marker in ['#import', '"#import', '"#set']:
            idx = output.find(marker)
            if idx != -1:
                typst_start = idx
                break
        
        if typst_start is None:
            return output  # No Typst found, return raw
        
        # Extract from the marker to the end
        # Remove leading quote if present
        content = output[typst_start:]
        if content.startswith('"'):
            content = content[1:]
        
        # Find the closing quote - look for ✅ which marks end of example output
        # Or look for a standalone " at the end
        end_markers = ['\n✅', '"\n✅', '"✅']
        for marker in end_markers:
            idx = content.find(marker)
            if idx != -1:
                content = content[:idx]
                break
        
        # Remove trailing quote if present
        content = content.rstrip()
        if content.endswith('"'):
            content = content[:-1]
        
        # Unescape the content
        typst = content.replace('\\n', '\n')
        typst = typst.replace('\\"', '"')
        typst = typst.replace('\\\\', '\\')
        
        return typst
        
    except subprocess.TimeoutExpired:
        print("Kleis compilation timed out")
        return None
    except FileNotFoundError:
        print("Kleis binary not found")
        return None


def compile_to_pdf(kleis_file: str, output_pdf: str) -> bool:
    """
    Compile a Kleis document to PDF via Typst.
    
    Args:
        kleis_file: Path to the .kleis document
        output_pdf: Path for the output PDF
    
    Returns:
        True if successful
    """
    typst_code = compile_to_typst(kleis_file)
    if not typst_code:
        return False
    
    # Write Typst to file
    typst_path = output_pdf.replace(".pdf", ".typ")
    with open(typst_path, "w") as f:
        f.write(typst_code)
    
    # Compile with Typst
    try:
        result = subprocess.run(
            ["typst", "compile", typst_path, output_pdf],
            capture_output=True,
            text=True,
            timeout=60
        )
        if result.returncode == 0:
            print(f"✓ PDF created: {output_pdf}")
            return True
        else:
            print(f"Typst error: {result.stderr}")
            return False
    except FileNotFoundError:
        print("Error: typst not found. Install: cargo install typst-cli")
        return False


def validate(kleis_file: str) -> bool:
    """
    Validate a Kleis document (parse + type check).
    
    Args:
        kleis_file: Path to the .kleis document
    
    Returns:
        True if valid
    """
    kleis_path = find_kleis_binary()
    if not kleis_path:
        print("Error: Kleis binary not found")
        return False
    
    kleis_root = find_kleis_root()
    env = os.environ.copy()
    if kleis_root:
        env["KLEIS_ROOT"] = kleis_root
    
    try:
        result = subprocess.run(
            [kleis_path, "check", kleis_file],
            capture_output=True,
            text=True,
            timeout=30,
            env=env
        )
        if result.returncode == 0:
            print(f"✓ {kleis_file} is valid")
            return True
        else:
            print(f"✗ {result.stderr}")
            return False
    except subprocess.TimeoutExpired:
        print("Validation timed out")
        return False


def list_templates() -> list:
    """List available document templates."""
    kleis_root = find_kleis_root()
    if not kleis_root:
        return []
    
    template_dir = Path(kleis_root) / "stdlib" / "templates"
    if template_dir.exists():
        return [f.stem for f in template_dir.glob("*.kleis")]
    return []


# Convenience aliases
def thesis_to_pdf(kleis_file: str, output_pdf: str) -> bool:
    """Compile a thesis document to PDF."""
    return compile_to_pdf(kleis_file, output_pdf)


def paper_to_pdf(kleis_file: str, output_pdf: str) -> bool:
    """Compile a paper document to PDF."""
    return compile_to_pdf(kleis_file, output_pdf)
