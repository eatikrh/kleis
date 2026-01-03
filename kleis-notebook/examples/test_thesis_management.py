#!/usr/bin/env python3
"""
Test script for thesis management features.

Tests:
- list_documents(): Find .kleis files
- open_recent(): Open most recent document
- doc.toc(): Table of contents
- doc.summary(): Document summary
- doc.list_equations(): List equations
- doc.list_figures(): List figures
- doc.list_tables(): List tables
"""

import os
import sys
import tempfile
import time
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from kleis_kernel.kleisdoc import KleisDoc, Author, list_documents, open_recent


def test_list_documents_empty():
    """Test list_documents with no .kleis files."""
    with tempfile.TemporaryDirectory() as tmpdir:
        docs = list_documents(tmpdir, verbose=False)
        assert docs == [], f"Expected empty list, got {docs}"
    print("✅ test_list_documents_empty passed")


def test_list_documents_with_files():
    """Test list_documents finds .kleis files."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create some .kleis files
        file1 = Path(tmpdir) / "thesis.kleis"
        file1.write_text('define meta_title = "My PhD Thesis"\n')
        
        file2 = Path(tmpdir) / "chapter1.kleis"
        file2.write_text('define meta_title = "Chapter 1 Draft"\n')
        
        # Small delay to ensure different modification times
        time.sleep(0.1)
        file1.touch()  # Make thesis.kleis most recent
        
        docs = list_documents(tmpdir, verbose=False)
        
        assert len(docs) == 2, f"Expected 2 docs, got {len(docs)}"
        assert docs[0]["filename"] == "thesis.kleis", "Most recent should be first"
        assert docs[0]["title"] == "My PhD Thesis"
        assert docs[1]["title"] == "Chapter 1 Draft"
    
    print("✅ test_list_documents_with_files passed")


def test_list_documents_verbose():
    """Test list_documents prints output."""
    with tempfile.TemporaryDirectory() as tmpdir:
        file1 = Path(tmpdir) / "test.kleis"
        file1.write_text('define meta_title = "Test Document"\n')
        
        # Capture output (just make sure it doesn't crash)
        docs = list_documents(tmpdir, verbose=True)
        assert len(docs) == 1
    
    print("✅ test_list_documents_verbose passed")


def test_toc_empty():
    """Test toc() on empty document."""
    doc = KleisDoc()
    items = doc.toc(verbose=False)
    assert items == [], f"Expected empty list, got {items}"
    print("✅ test_toc_empty passed")


def test_toc_with_sections():
    """Test toc() with sections."""
    doc = KleisDoc()
    doc.add_section("Introduction", level=1)
    doc.add_section("Background", level=2)
    doc.add_section("Motivation", level=2)
    doc.add_section("Methods", level=1)
    
    items = doc.toc(verbose=False)
    
    assert len(items) == 4, f"Expected 4 items, got {len(items)}"
    assert items[0]["number"] == "1"
    assert items[0]["title"] == "Introduction"
    assert items[1]["number"] == "1.1"
    assert items[1]["title"] == "Background"
    assert items[2]["number"] == "1.2"
    assert items[2]["title"] == "Motivation"
    assert items[3]["number"] == "2"
    assert items[3]["title"] == "Methods"
    
    print("✅ test_toc_with_sections passed")


def test_summary_empty():
    """Test summary() on empty document."""
    doc = KleisDoc()
    info = doc.summary(verbose=False)
    
    assert info["title"] == "Untitled"
    assert info["sections"] == 0
    assert info["equations"] == 0
    
    print("✅ test_summary_empty passed")


def test_summary_with_content():
    """Test summary() with content."""
    doc = KleisDoc()
    doc.set_metadata(
        title="Test Thesis",
        authors=[Author(name="Jane Smith", affiliation="MIT")],
        date="2026",
        abstract="This is a test abstract with some words."
    )
    doc.add_section("Introduction", level=1)
    doc.add_section("Methods", level=1)
    doc.add_equation("eq1", r"E = mc^2")
    doc.add_equation("eq2", r"F = ma")
    doc.add_figure("fig1", "A test figure", kleis_code="diagram()")
    
    info = doc.summary(verbose=False)
    
    assert info["title"] == "Test Thesis"
    assert info["sections"] == 2
    assert info["equations"] == 2
    assert info["figures"] == 1
    
    print("✅ test_summary_with_content passed")


def test_list_equations():
    """Test list_equations()."""
    doc = KleisDoc()
    doc.add_equation("schrodinger", r"i\hbar\frac{\partial}{\partial t}\Psi = H\Psi")
    doc.add_equation("entropy", r"S = -k_B \sum p_i \log p_i")
    
    eqs = doc.list_equations(verbose=False)
    
    assert len(eqs) == 2
    labels = [eq["label"] for eq in eqs]
    assert "schrodinger" in labels
    assert "entropy" in labels
    
    print("✅ test_list_equations passed")


def test_list_figures():
    """Test list_figures()."""
    doc = KleisDoc()
    doc.add_figure("sine", "Sine wave", kleis_code="plot(sin(x))")
    doc.add_figure("data", "Experimental data", image_path="data.png")
    
    figs = doc.list_figures(verbose=False)
    
    assert len(figs) == 2
    
    # Check that we track code vs image
    sine_fig = next(f for f in figs if f["label"] == "sine")
    assert sine_fig["has_code"] == True
    
    data_fig = next(f for f in figs if f["label"] == "data")
    assert data_fig["has_image"] == True
    
    print("✅ test_list_figures passed")


def test_list_tables():
    """Test list_tables()."""
    doc = KleisDoc()
    doc.add_table("results", ["Method", "Score"], [["A", "90"], ["B", "85"]], "Results table")
    
    tables = doc.list_tables(verbose=False)
    
    assert len(tables) == 1
    assert tables[0]["label"] == "results"
    assert tables[0]["caption"] == "Results table"
    
    print("✅ test_list_tables passed")


def test_open_recent():
    """Test open_recent()."""
    with tempfile.TemporaryDirectory() as tmpdir:
        os.chdir(tmpdir)
        
        # Create a document and save it
        doc = KleisDoc()
        doc.set_metadata(title="Recent Test", date="2026")
        doc.add_section("Test Section")
        doc.save("recent_test.kleis")
        
        # Open it with open_recent
        loaded = open_recent(tmpdir)
        
        assert loaded is not None
        assert loaded.metadata.get("title") == "Recent Test"
    
    print("✅ test_open_recent passed")


def test_verbose_output():
    """Test that verbose output works without errors."""
    doc = KleisDoc()
    doc.set_metadata(title="Verbose Test", authors=[Author(name="Test User")])
    doc.add_section("Chapter 1", level=1)
    doc.add_section("Section 1.1", level=2)
    doc.add_equation("test_eq", r"x = y")
    doc.add_figure("test_fig", "Test figure", kleis_code="plot()")
    doc.add_table("test_tab", ["A", "B"], [["1", "2"]], "Test table")
    
    # These should all print without errors
    print("\n--- Verbose Output Test ---")
    doc.summary(verbose=True)
    doc.toc(verbose=True)
    doc.list_equations(verbose=True)
    doc.list_figures(verbose=True)
    doc.list_tables(verbose=True)
    print("--- End Verbose Output ---\n")
    
    print("✅ test_verbose_output passed")


if __name__ == "__main__":
    print("Testing Thesis Management Features")
    print("=" * 40)
    
    test_list_documents_empty()
    test_list_documents_with_files()
    test_list_documents_verbose()
    test_toc_empty()
    test_toc_with_sections()
    test_summary_empty()
    test_summary_with_content()
    test_list_equations()
    test_list_figures()
    test_list_tables()
    test_open_recent()
    test_verbose_output()
    
    print()
    print("=" * 40)
    print("✅ All thesis management tests passed!")

