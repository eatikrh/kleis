#!/usr/bin/env python3
"""Test KleisDoc functionality."""

import sys
sys.path.insert(0, '..')

from kleis_kernel.kleisdoc import KleisDoc, list_templates

def test_basic_document():
    """Test creating a basic document."""
    print("Testing basic document creation...")
    
    # Create a blank document
    doc = KleisDoc.new()
    
    # Set metadata
    doc.set_metadata(
        title="My Research Paper",
        author="Jane Smith",
        date="January 2026",
        abstract="This paper presents novel findings..."
    )
    
    print(f"  Title: {doc.get_metadata('title')}")
    print(f"  Author: {doc.get_metadata('author')}")
    
    # Add sections
    intro = doc.add_section("Introduction", 
        "This paper explores the relationship between...")
    
    methods = doc.add_section("Methods",
        "We employed a mixed-methods approach...")
    
    # Add a subsection
    doc.add_subsection(methods, "Data Collection", 
        "Data was collected over a 6-month period...")
    
    results = doc.add_section("Results")
    doc.add_text("Our analysis revealed significant patterns.", results)
    
    print(f"  Sections: {len(doc.sections)}")
    for s in doc.sections:
        print(f"    - {s.title} (level {s.level})")
    
    print("  ✓ Basic document test passed")
    print()

def test_equations_and_figures():
    """Test adding equations and figures."""
    print("Testing equations and figures...")
    
    doc = KleisDoc()
    
    # Add an equation with AST (for re-editing in Equation Editor)
    eq = doc.add_equation(
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
    print(f"  Added equation: {eq.label} -> {eq.latex}")
    
    # Retrieve and verify
    retrieved = doc.get_equation("eq:einstein")
    assert retrieved is not None
    assert retrieved.ast is not None
    print(f"  Retrieved equation AST: {retrieved.ast is not None}")
    
    # Add a figure with Kleis code (regenerable)
    fig = doc.add_figure(
        label="fig:performance",
        caption="System performance over time",
        kleis_code='diagram([line([1, 2, 3], [10, 20, 15])])'
    )
    print(f"  Added figure: {fig.label}")
    
    # Add a static image figure
    static_fig = doc.add_figure(
        label="fig:diagram",
        caption="System architecture",
        image_path="images/architecture.png"
    )
    print(f"  Added static figure: {static_fig.label}")
    
    print(f"  Total equations: {len(doc.equations)}")
    print(f"  Total figures: {len(doc.figures)}")
    
    print("  ✓ Equations and figures test passed")
    print()

def test_export():
    """Test Typst export."""
    print("Testing Typst export...")
    
    doc = KleisDoc()
    doc.set_metadata(
        title="Test Document",
        author="Test Author",
        abstract="This is a test abstract."
    )
    doc.add_section("Introduction", "This is a test document.")
    doc.add_section("Conclusion", "In conclusion, tests are important.")
    
    # Generate Typst
    typst_code = doc._generate_typst()
    print("  Generated Typst:")
    for line in typst_code.split('\n')[:12]:
        print(f"    {line}")
    print("    ...")
    
    assert "title:" in typst_code
    assert "Introduction" in typst_code
    
    print("  ✓ Export test passed")
    print()

def test_template_loading():
    """Test loading templates from files."""
    print("Testing template loading...")
    
    # List available templates
    templates = list_templates("../../stdlib/templates")
    print(f"  Found {len(templates)} templates in stdlib/templates")
    for t in templates:
        print(f"    - {t}")
    
    # Try to create from template (may not exist)
    if templates:
        template_path = templates[0]
        doc = KleisDoc.from_template(template_path)
        print(f"  Loaded template: {doc.template_path}")
        print(f"  Template info: {doc.template_info}")
    else:
        print("  No templates found (that's OK for now)")
    
    print("  ✓ Template loading test passed")
    print()

def test_html_repr():
    """Test Jupyter HTML representation."""
    print("Testing HTML representation...")
    
    doc = KleisDoc()
    doc.set_metadata(title="Test Doc")
    doc.add_section("Section 1")
    doc.add_equation("eq:test", "x = y")
    
    html = doc._repr_html_()
    assert "Test Doc" in html
    assert "1 sections" in html
    assert "1 equations" in html
    
    print("  ✓ HTML representation test passed")
    print()

if __name__ == "__main__":
    print("=" * 60)
    print("KleisDoc Tests")
    print("=" * 60)
    print()
    
    test_basic_document()
    test_equations_and_figures()
    test_export()
    test_template_loading()
    test_html_repr()
    
    print("=" * 60)
    print("All tests passed! ✓")
    print("=" * 60)
