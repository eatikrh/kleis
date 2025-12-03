#!/usr/bin/env python3
"""Quick script to inspect DLMF HTML structure."""

import urllib.request
import ssl
import re

def fetch_page(chapter):
    url = f"https://dlmf.nist.gov/{chapter}"
    ctx = ssl.create_default_context()
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE
    
    with urllib.request.urlopen(url, timeout=30, context=ctx) as response:
        return response.read().decode('utf-8')

html = fetch_page(5)

# Find all math/equation patterns
print("=== Looking for LaTeX in alt attributes ===")
alt_pattern = re.compile(r'alt="([^"]*\$[^"]+)"')
matches = list(alt_pattern.finditer(html))[:10]
for i, match in enumerate(matches):
    print(f"{i+1}. {match.group(1)[:100]}")

print("\n=== Looking for MathML ===")
mathml_pattern = re.compile(r'<math[^>]*>(.*?)</math>', re.DOTALL)
matches = mathml_pattern.findall(html)
print(f"Found {len(matches)} MathML blocks")

print("\n=== Looking for script tags with LaTeX ===")
script_pattern = re.compile(r'<script[^>]*type=["\']math/tex["\'][^>]*>(.*?)</script>', re.DOTALL)
matches = script_pattern.findall(html)
print(f"Found {len(matches)} math/tex script tags")
if matches:
    for i, m in enumerate(matches[:5]):
        print(f"{i+1}. {m[:100]}")

print("\n=== Looking for data-latex attributes ===")
data_latex = re.compile(r'data-latex="([^"]+)"')
matches = data_latex.findall(html)
print(f"Found {len(matches)} data-latex attributes")
if matches:
    for i, m in enumerate(matches[:5]):
        print(f"{i+1}. {m[:100]}")

print("\n=== Sample of HTML structure ===")
# Find first 500 chars containing "equation" or "math"
eq_idx = html.lower().find('equation')
if eq_idx > 0:
    print(html[max(0, eq_idx-200):eq_idx+500])

