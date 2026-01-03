#!/usr/bin/env python3
"""Test KleisDoc functionality."""

import sys
sys.path.insert(0, '..')

from kleis_kernel.kleisdoc import KleisDoc, templates

def test_templates():
    """Test template listing."""
    print("Available templates:")
    for t in templates.list():
        print(f"  - {t}")
    print()

def test_mit_thesis():
    """Test MIT Thesis template."""
    print("Testing MIT Thesis template...")
    
    # Create from template
    thesis = KleisDoc.from_template("MIT Thesis")
    
    # Set metadata
    thesis.set_metadata(
        title="Formal Verification of Knowledge Production Systems",
        author="Jane Smith",
        email="jane@mit.edu",
        department="Electrical Engineering and Computer Science",
        degree="Doctor of Philosophy",
        date="May 2025",
        supervisor="Prof. Alice Chen",
        keywords=["formal verification", "type theory", "scientific computing"]
    )
    
    # Print structure
    print(f"  Title: {thesis.metadata.get('title')}")
    print(f"  Chapters: {len(thesis.sections)}")
    for section in thesis.sections:
        print(f"    - {section.title}")
    
    # Add content
    thesis.sections[0].content.append(
        "This thesis presents a unified framework for knowledge production."
    )
    
    # Add an equation
    eq = thesis.add_equation(
        label="eq:einstein",
        latex="E = mc^2",
        ast={"Operation": {"name": "equals", "args": [
            {"Object": "E"},
            {"Operation": {"name": "times", "args": [
                {"Object": "m"},
                {"Operation": {"name": "power", "args": [
                    {"Object": "c"},
                    {"Const": "2"}
                ]}}
            ]}}
        ]}}
    )
    print(f"  Added equation: {eq.label}")
    
    # Add a figure
    fig = thesis.add_figure(
        label="fig:performance",
        caption="System performance over time",
        kleis_code='plot(line([1, 2, 3, 4], [10, 20, 15, 25]))'
    )
    print(f"  Added figure: {fig.label}")
    
    print("  ✓ MIT Thesis test passed")
    print()

def test_export():
    """Test Typst export."""
    print("Testing Typst export...")
    
    doc = KleisDoc.new()
    doc.set_metadata(
        title="Test Document",
        author="Test Author"
    )
    doc.add_chapter("Introduction", "This is a test document.")
    
    # Export to Typst
    typst_code = doc._generate_typst()
    print("  Generated Typst:")
    for line in typst_code.split('\n')[:15]:
        print(f"    {line}")
    if typst_code.count('\n') > 15:
        print("    ...")
    
    print("  ✓ Export test passed")
    print()

def test_html_repr():
    """Test Jupyter HTML representation."""
    print("Testing HTML representation...")
    
    doc = KleisDoc.from_template("MIT Thesis")
    doc.set_metadata(title="Test Thesis")
    html = doc._repr_html_()
    
    assert "Test Thesis" in html
    assert "MIT Thesis" in html
    print("  ✓ HTML representation test passed")
    print()

if __name__ == "__main__":
    print("=" * 60)
    print("KleisDoc Tests")
    print("=" * 60)
    print()
    
    test_templates()
    test_mit_thesis()
    test_export()
    test_html_repr()
    
    print("=" * 60)
    print("All tests passed! ✓")
    print("=" * 60)

