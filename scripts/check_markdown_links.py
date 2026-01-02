#!/usr/bin/env python3
"""Check all markdown links in the repository - ignoring code blocks."""

import os
import re
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.resolve()  # Auto-detect from script location
EXCLUDE_DIRS = {"node_modules", "target", ".git", "vendor"}

def find_markdown_files():
    """Find all .md files excluding certain directories."""
    md_files = []
    for root, dirs, files in os.walk(REPO_ROOT):
        # Remove excluded directories from traversal
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
        
        for file in files:
            if file.endswith('.md'):
                md_files.append(Path(root) / file)
    return sorted(md_files)

def remove_code_blocks(content):
    """Remove code blocks and inline code from content."""
    # Remove fenced code blocks (``` ... ```)
    content = re.sub(r'```[\s\S]*?```', '', content, flags=re.MULTILINE)
    # Remove inline code (`...`)
    content = re.sub(r'`[^`]+`', '', content)
    return content

def extract_links(md_file):
    """Extract all markdown links from a file, excluding code blocks."""
    try:
        content = md_file.read_text(encoding='utf-8')
    except Exception as e:
        print(f"⚠️  Cannot read {md_file}: {e}")
        return []
    
    # Remove code blocks first
    content_no_code = remove_code_blocks(content)
    
    # Pattern: [text](link)
    # Exclude: external links (http/https), anchors (#)
    pattern = r'\[([^\]]+)\]\(([^)]+)\)'
    links = []
    
    for match in re.finditer(pattern, content_no_code):
        link_text = match.group(1)
        link = match.group(2)
        
        # Skip external links and anchors
        if link.startswith(('http://', 'https://', '#', 'mailto:')):
            continue
        
        # Skip likely mathematical notation:
        # - Single char/symbol link text: [f], [ψ], [T]
        # - Mathematical functions: [exp(-t²)], [sin(x)]
        # - Greek letters or math symbols
        # - Non-.md/.html/.pdf link paths (likely variable names)
        if not link.endswith(('.md', '.html', '.pdf', '.png', '.jpg', '.svg', '.txt', '.rs', '.kleis')):
            # Not a file extension - likely math notation
            continue
        
        if (len(link_text) <= 3 or  # Single char like [f], [ψ]
            '(' in link_text or     # Function notation like [exp(-t²)]
            link_text in ['T', 'E', 'F', 'S']):  # Common math symbols
            continue
        
        # Remove anchor from end if present
        link = link.split('#')[0]
        if link:  # Only include non-empty links
            links.append((link_text, link))
    
    return links

def resolve_link(md_file, link):
    """Resolve a relative link from a markdown file."""
    md_dir = md_file.parent
    
    if link.startswith('/'):
        # Absolute path from repo root
        target = REPO_ROOT / link.lstrip('/')
    else:
        # Relative path from markdown file
        target = md_dir / link
    
    # Resolve to absolute path
    try:
        target = target.resolve()
    except Exception:
        pass
    
    return target

def main():
    print("🔍 Checking all markdown links (excluding code blocks)...\n")
    
    md_files = find_markdown_files()
    print(f"Found {len(md_files)} markdown files\n")
    
    broken_links = []
    total_links = 0
    
    for md_file in md_files:
        links = extract_links(md_file)
        
        for link_text, link_path in links:
            total_links += 1
            target = resolve_link(md_file, link_path)
            
            if not target.exists():
                rel_md = md_file.relative_to(REPO_ROOT)
                broken_links.append((rel_md, link_text, link_path, target))
    
    # Report results
    if broken_links:
        print(f"❌ Found {len(broken_links)} broken links:\n")
        
        current_file = None
        for md_file, link_text, link_path, target in broken_links:
            if md_file != current_file:
                print(f"\n📄 {md_file}")
                current_file = md_file
            print(f"   ❌ [{link_text}]({link_path})")
            try:
                rel_target = target.relative_to(REPO_ROOT)
            except:
                rel_target = target
            print(f"      → Resolves to: {rel_target}")
    
    print(f"\n📊 Summary:")
    print(f"   Markdown files: {len(md_files)}")
    print(f"   Total links checked: {total_links}")
    print(f"   Broken links: {len(broken_links)}")
    
    if broken_links:
        print("\n⚠️  Please fix broken links")
        return 1
    else:
        print("\n✅ All links are valid!")
        return 0

if __name__ == "__main__":
    exit(main())
