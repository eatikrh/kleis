#!/usr/bin/env python3
"""
Export the complete_demo.kleis document to PDF.

This demonstrates loading a KleisDoc from a .kleis file and exporting to PDF.
"""

import sys
import os

# Add the package to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from kleis_kernel.kleisdoc import KleisDoc

def main():
    # Get paths
    script_dir = os.path.dirname(os.path.abspath(__file__))
    kleis_file = os.path.join(script_dir, "complete_demo.kleis")
    output_pdf = os.path.join(script_dir, "complete_demo.pdf")
    
    print("=" * 60)
    print("KleisDoc Export Demo")
    print("=" * 60)
    print()
    
    # Load the document
    print(f"Loading: {kleis_file}")
    doc = KleisDoc.load(kleis_file)
    
    # Show what's in the document
    print()
    print("Document contents:")
    print(f"  Title: {doc.metadata.get('title', 'Untitled')}")
    print(f"  Author: {doc.metadata.get('author', 'Unknown')}")
    print(f"  Sections: {len(doc.sections)}")
    print(f"  Equations: {len(doc.equations)}")
    print(f"  Figures: {len(doc.figures)}")
    print(f"  Tables: {len(doc.tables)}")
    print(f"  Theorems: {len(doc.theorems)}")
    print(f"  Algorithms: {len(doc.algorithms)}")
    print(f"  Bibliography: {len(doc.bibliography)}")
    print()
    
    # List sections
    print("Sections:")
    for section in doc.sections:
        indent = "  " * section.level
        print(f"  {indent}{section.title}")
    print()
    
    # List equations
    print("Equations:")
    for label, eq in doc.equations.items():
        print(f"  {label}: {eq.latex[:40]}...")
    print()
    
    # List figures
    print("Figures:")
    for label, fig in doc.figures.items():
        print(f"  {label}: {fig.caption[:40]}...")
    print()
    
    # Export to PDF
    print(f"Exporting to: {output_pdf}")
    try:
        doc.export_pdf(output_pdf)
        print(f"SUCCESS! PDF created at: {output_pdf}")
    except Exception as e:
        print(f"ERROR: {e}")
        # Try to show the Typst output for debugging
        typst_output = os.path.join(script_dir, "complete_demo.typ")
        doc.export_typst(typst_output)
        print(f"Typst file saved for debugging: {typst_output}")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main())

