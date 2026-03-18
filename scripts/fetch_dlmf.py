#!/usr/bin/env python3
"""
DLMF Equation Fetcher
Downloads equations from NIST Digital Library of Mathematical Functions
https://dlmf.nist.gov/

Usage:
    python scripts/fetch_dlmf.py --chapters 1,5,13 --output tests/golden/sources/dlmf/
"""

import argparse
import re
import time
from pathlib import Path
from typing import List, Dict, Tuple
import urllib.request
import urllib.error
from html.parser import HTMLParser
import ssl


class DLMFEquationParser(HTMLParser):
    """Parse DLMF HTML pages to extract LaTeX equations."""
    
    def __init__(self):
        super().__init__()
        self.equations: List[Dict[str, str]] = []
        self.in_equation = False
        self.in_math = False
        self.current_eq_id = None
        self.current_latex = []
        self.in_title = False
        self.current_title = []
        
    def handle_starttag(self, tag, attrs):
        attrs_dict = dict(attrs)
        
        # Detect equation containers
        if tag == 'table' and attrs_dict.get('class', '').startswith('equation'):
            self.in_equation = True
            self.current_eq_id = attrs_dict.get('id', 'unknown')
        
        # Detect math content (look for class with 'math' or 'ltx_Math')
        if tag == 'math' or 'math' in attrs_dict.get('class', '').lower():
            self.in_math = True
            
        # Look for alt text which often contains LaTeX
        if tag == 'img' and 'alt' in attrs_dict:
            alt_text = attrs_dict['alt']
            if alt_text and len(alt_text) > 3:  # Filter out empty alts
                self.current_latex.append(alt_text)
        
        # Section titles
        if tag in ['h2', 'h3'] and 'title' in attrs_dict.get('class', ''):
            self.in_title = True
    
    def handle_endtag(self, tag):
        if tag == 'table' and self.in_equation:
            if self.current_latex:
                latex = ' '.join(self.current_latex).strip()
                if latex:
                    self.equations.append({
                        'id': self.current_eq_id,
                        'latex': latex
                    })
            self.in_equation = False
            self.current_eq_id = None
            self.current_latex = []
        
        if tag == 'math':
            self.in_math = False
            
        if tag in ['h2', 'h3']:
            self.in_title = False
    
    def handle_data(self, data):
        if self.in_math and data.strip():
            # Sometimes math content is in text nodes
            self.current_latex.append(data.strip())
        
        if self.in_title:
            self.current_title.append(data.strip())


def fetch_chapter_html(chapter: int) -> str:
    """Fetch HTML for a DLMF chapter."""
    url = f"https://dlmf.nist.gov/{chapter}"
    print(f"Fetching chapter {chapter} from {url}...")
    
    try:
        # Create SSL context that doesn't verify certificates (for compatibility)
        ctx = ssl.create_default_context()
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        
        with urllib.request.urlopen(url, timeout=30, context=ctx) as response:
            html = response.read().decode('utf-8')
        return html
    except urllib.error.URLError as e:
        print(f"Error fetching chapter {chapter}: {e}")
        return ""


def fetch_section_html(chapter: int, section: int) -> str:
    """Fetch HTML for a specific section."""
    url = f"https://dlmf.nist.gov/{chapter}.{section}"
    print(f"Fetching section {chapter}.{section}...")
    
    try:
        # Create SSL context that doesn't verify certificates (for compatibility)
        ctx = ssl.create_default_context()
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        
        with urllib.request.urlopen(url, timeout=30, context=ctx) as response:
            html = response.read().decode('utf-8')
        return html
    except urllib.error.URLError as e:
        print(f"Error fetching section {chapter}.{section}: {e}")
        return ""


def clean_latex(latex: str) -> str:
    """Clean and normalize LaTeX from DLMF."""
    # Remove HTML entities
    latex = latex.replace('&nbsp;', ' ')
    latex = latex.replace('&lt;', '<')
    latex = latex.replace('&gt;', '>')
    latex = latex.replace('&amp;', '&')
    
    # Remove extra whitespace
    latex = re.sub(r'\s+', ' ', latex)
    latex = latex.strip()
    
    # Ensure math delimiters
    if latex and not latex.startswith('$') and not latex.startswith('\\['):
        if '\n' in latex or len(latex) > 80:
            latex = f"\\[\n{latex}\n\\]"
        else:
            latex = f"${latex}$"
    
    return latex


def extract_equations_from_html(html: str) -> List[Dict[str, str]]:
    """Extract equations from DLMF HTML page."""
    parser = DLMFEquationParser()
    parser.feed(html)
    
    equations = []
    for eq in parser.equations:
        latex = clean_latex(eq['latex'])
        if latex and len(latex) > 5:  # Filter trivial equations
            equations.append({
                'id': eq['id'],
                'latex': latex
            })
    
    return equations


def save_equations_to_file(equations: List[Dict[str, str]], output_path: Path, chapter: int):
    """Save equations to a LaTeX file."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(f"% DLMF Chapter {chapter} - Auto-generated\n")
        f.write(f"% Source: https://dlmf.nist.gov/{chapter}\n")
        f.write(f"% Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        
        for i, eq in enumerate(equations, 1):
            f.write(f"% Equation {eq['id']}\n")
            f.write(f"{eq['latex']}\n\n")
    
    print(f"✓ Saved {len(equations)} equations to {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description='Fetch equations from DLMF for testing'
    )
    parser.add_argument(
        '--chapters',
        type=str,
        default='1,5,13,15,25',
        help='Comma-separated list of chapter numbers (e.g., "1,5,13")'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('tests/golden/sources/dlmf'),
        help='Output directory for LaTeX files'
    )
    parser.add_argument(
        '--max-per-chapter',
        type=int,
        default=50,
        help='Maximum equations per chapter'
    )
    parser.add_argument(
        '--delay',
        type=float,
        default=2.0,
        help='Delay between requests (seconds) to be polite'
    )
    
    args = parser.parse_args()
    
    chapters = [int(c.strip()) for c in args.chapters.split(',')]
    
    print(f"DLMF Equation Fetcher")
    print(f"Chapters: {chapters}")
    print(f"Output: {args.output}")
    print()
    
    for chapter in chapters:
        html = fetch_chapter_html(chapter)
        if not html:
            continue
        
        equations = extract_equations_from_html(html)
        
        # Limit number of equations
        if len(equations) > args.max_per_chapter:
            equations = equations[:args.max_per_chapter]
        
        if equations:
            output_file = args.output / f"chapter{chapter:02d}.tex"
            save_equations_to_file(equations, output_file, chapter)
        else:
            print(f"⚠ No equations found in chapter {chapter}")
        
        # Be polite - don't hammer the server
        if chapter != chapters[-1]:
            time.sleep(args.delay)
    
    print()
    print("✓ Done!")
    print(f"  Total files: {len(list(args.output.glob('*.tex')))}")


if __name__ == '__main__':
    main()

