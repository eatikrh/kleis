#!/usr/bin/env python3
"""
Test script for edit operations (update/remove).
"""

import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from kleis_kernel.kleisdoc import KleisDoc, Author


def test_update_equation():
    """Test updating an existing equation."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add equation
    doc.add_equation("loss", latex=r"\mathcal{L} = \sum(y - \hat{y})^2")
    
    # Verify original
    eq = doc.get_equation("loss")
    assert eq is not None
    assert r"\mathcal{L}" in eq.latex
    
    # Update equation
    doc.update_equation("loss", latex=r"\mathcal{L}_{new} = \frac{1}{n}\sum(y - \hat{y})^2")
    
    # Verify updated
    eq = doc.get_equation("loss")
    assert r"\mathcal{L}_{new}" in eq.latex
    assert r"\frac{1}{n}" in eq.latex
    
    print("✅ test_update_equation passed")


def test_remove_equation():
    """Test removing an equation."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add equations
    doc.add_equation("eq1", latex="x = 1")
    doc.add_equation("eq2", latex="y = 2")
    doc.add_equation("eq3", latex="z = 3")
    
    assert len(doc.equations) == 3
    
    # Remove one
    result = doc.remove_equation("eq2")
    assert result == True
    assert len(doc.equations) == 2
    assert "eq2" not in doc.equations
    assert "eq1" in doc.equations
    assert "eq3" in doc.equations
    
    # Try to remove non-existent
    result = doc.remove_equation("nonexistent")
    assert result == False
    
    print("✅ test_remove_equation passed")


def test_update_section():
    """Test updating a section."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add section
    doc.add_section("Introduction", content="This is the intro.")
    
    # Verify original
    section = doc.get_section("Introduction")
    assert section is not None
    assert "This is the intro." in section.content[0]
    
    # Update section
    doc.update_section("Introduction", content="This is the REVISED intro.")
    
    # Verify updated
    section = doc.get_section("Introduction")
    assert "REVISED" in section.content[0]
    
    # Update title
    doc.update_section("Introduction", new_title="Chapter 1: Introduction")
    section = doc.get_section("Chapter 1: Introduction")
    assert section is not None
    
    print("✅ test_update_section passed")


def test_remove_section():
    """Test removing a section."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add sections
    doc.add_section("Chapter 1")
    doc.add_section("Chapter 2")
    doc.add_section("Chapter 3")
    
    assert len(doc.sections) == 3
    
    # Remove one
    result = doc.remove_section("Chapter 2")
    assert result == True
    assert len(doc.sections) == 2
    
    # Verify correct ones remain
    titles = [s.title for s in doc.sections]
    assert "Chapter 1" in titles
    assert "Chapter 3" in titles
    assert "Chapter 2" not in titles
    
    # Try to remove non-existent
    result = doc.remove_section("nonexistent")
    assert result == False
    
    print("✅ test_remove_section passed")


def test_update_figure():
    """Test updating a figure."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add figure
    doc.add_figure("fig1", caption="Original caption")
    
    # Verify original
    fig = doc.get_figure("fig1")
    assert fig is not None
    assert fig.caption == "Original caption"
    
    # Update figure
    doc.update_figure("fig1", caption="Updated caption")
    
    # Verify updated
    fig = doc.get_figure("fig1")
    assert fig.caption == "Updated caption"
    
    print("✅ test_update_figure passed")


def test_remove_figure():
    """Test removing a figure."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add figures
    doc.add_figure("fig1", caption="Figure 1")
    doc.add_figure("fig2", caption="Figure 2")
    
    assert len(doc.figures) == 2
    
    # Remove one
    result = doc.remove_figure("fig1")
    assert result == True
    assert len(doc.figures) == 1
    assert "fig2" in doc.figures
    
    print("✅ test_remove_figure passed")


def test_update_table():
    """Test updating a table."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add table
    doc.add_table("tab1", 
                  headers=["A", "B", "C"],
                  rows=[[1, 2, 3], [4, 5, 6]],
                  caption="Original table")
    
    # Verify original
    table = doc.get_table("tab1")
    assert table is not None
    assert table["caption"] == "Original table"
    assert table["headers"] == ["A", "B", "C"]
    
    # Update table
    doc.update_table("tab1", 
                     headers=["X", "Y", "Z"],
                     caption="Updated table")
    
    # Verify updated
    table = doc.get_table("tab1")
    assert table["caption"] == "Updated table"
    assert table["headers"] == ["X", "Y", "Z"]
    # Rows should be unchanged
    assert table["rows"] == [[1, 2, 3], [4, 5, 6]]
    
    print("✅ test_update_table passed")


def test_remove_table():
    """Test removing a table."""
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    
    # Add tables
    doc.add_table("tab1", headers=[], rows=[], caption="Table 1")
    doc.add_table("tab2", headers=[], rows=[], caption="Table 2")
    
    assert len(doc.tables) == 2
    
    # Remove one
    result = doc.remove_table("tab1")
    assert result == True
    assert len(doc.tables) == 1
    assert "tab2" in doc.tables
    
    print("✅ test_remove_table passed")


def test_full_workflow():
    """Test a realistic PhD student workflow."""
    doc = KleisDoc()
    doc.set_metadata(
        title="My Thesis Draft",
        authors=[Author(name="Jane Smith")],
        date="2026"
    )
    
    # Day 1: Initial draft
    intro = doc.add_section("Introduction", content="This is my thesis.")
    doc.add_equation("main_eq", latex=r"E = mc^2")
    
    # Day 2: Advisor says "revise introduction"
    doc.update_section("Introduction", content="This thesis explores quantum computing applications.")
    
    # Day 3: Fix the equation
    doc.update_equation("main_eq", latex=r"E = mc^2 + \text{corrections}")
    
    # Day 4: Add more content
    methods = doc.add_section("Methods")
    doc.add_text("We use the following approach...", methods)
    doc.add_figure("fig1", caption="System architecture")
    
    # Day 5: Remove draft notes section
    doc.add_section("Draft Notes", content="TODO: fix everything")
    doc.remove_section("Draft Notes")
    
    # Verify final state
    assert len(doc.sections) == 2  # Intro and Methods
    assert doc.get_section("Draft Notes") is None
    eq = doc.get_equation("main_eq")
    assert "corrections" in eq.latex
    
    # Save and reload
    with tempfile.NamedTemporaryFile(suffix=".kleis", delete=False) as f:
        temp_path = f.name
    
    try:
        doc.save(temp_path)
        loaded = KleisDoc.load(temp_path)
        
        # Verify persistence
        eq = loaded.get_equation("main_eq")
        assert eq is not None
        assert "corrections" in eq.latex
        
        intro = loaded.get_section("Introduction")
        assert intro is not None
        assert "quantum computing" in intro.content[0]
        
        print("✅ test_full_workflow passed")
    finally:
        os.unlink(temp_path)


if __name__ == "__main__":
    print("Testing Edit Operations")
    print("=" * 50)
    print()
    
    test_update_equation()
    test_remove_equation()
    test_update_section()
    test_remove_section()
    test_update_figure()
    test_remove_figure()
    test_update_table()
    test_remove_table()
    test_full_workflow()
    
    print()
    print("=" * 50)
    print("✅ All edit operation tests passed!")

