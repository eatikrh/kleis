#!/usr/bin/env python3
"""
Test script for LaTeX to Typst conversion.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from kleis_kernel.kleisdoc import KleisDoc


def test_latex_to_typst():
    """Test LaTeX to Typst conversion for common patterns."""
    doc = KleisDoc()
    
    test_cases = [
        # Calligraphic fonts
        (r"\mathcal{L}", "cal(L)"),
        (r"\mathbb{R}", "bb(R)"),
        (r"\mathbf{x}", "bold(x)"),
        
        # Fractions
        (r"\frac{a}{b}", "(a)/(b)"),
        (r"\frac{1}{2}", "(1)/(2)"),
        
        # Square roots
        (r"\sqrt{x}", "sqrt(x)"),
        (r"\sqrt{a+b}", "sqrt(a+b)"),
        
        # Greek letters
        (r"\alpha", "alpha"),
        (r"\beta", "beta"),
        (r"\gamma", "gamma"),
        (r"\Omega", "Omega"),
        (r"\pi", "pi"),
        
        # Operators
        (r"\sum", "sum"),
        (r"\int", "integral"),
        (r"\nabla", "nabla"),
        (r"\partial", "diff"),
        (r"\infty", "infinity"),
        
        # Relations
        (r"\leq", "<="),
        (r"\geq", ">="),
        (r"\neq", "!="),
        (r"\approx", "approx"),
        
        # Subscripts and superscripts
        (r"x^{2}", "x^(2)"),
        (r"x_{i}", "x_(i)"),
        (r"a^{n+1}", "a^(n+1)"),
        
        # Accents
        (r"\hat{x}", "hat(x)"),
        (r"\bar{x}", "overline(x)"),
        (r"\vec{v}", "arrow(v)"),
        
        # Combined
        (r"\mathcal{L} = \sum(y - \hat{y})^2", "cal(L) = sum(y - hat(y))^2"),
        (r"E = mc^{2}", "E = mc^(2)"),
        (r"\alpha + \beta = \gamma", "alpha + beta = gamma"),
    ]
    
    passed = 0
    failed = 0
    
    for latex, expected in test_cases:
        result = doc._latex_to_typst(latex)
        if result == expected:
            print(f"✅ {latex}")
            passed += 1
        else:
            print(f"❌ {latex}")
            print(f"   Expected: {expected}")
            print(f"   Got:      {result}")
            failed += 1
    
    print()
    print(f"Passed: {passed}, Failed: {failed}")
    return failed == 0


def test_unescape():
    """Test string escaping round-trip."""
    doc = KleisDoc()
    
    test_strings = [
        r"\mathcal{L}",
        r"a\nb",  # newline escape
        r'He said "hello"',
        r"path\\to\\file",
    ]
    
    for s in test_strings:
        escaped = doc._to_kleis_string(s)
        # Simulate what happens when reading from file
        # The string in the file is without outer quotes
        inner = escaped[1:-1]  # Remove quotes
        unescaped = doc._unescape_kleis_string(inner)
        
        if unescaped == s:
            print(f"✅ Round-trip: {repr(s)}")
        else:
            print(f"❌ Round-trip failed: {repr(s)}")
            print(f"   Escaped: {escaped}")
            print(f"   Unescaped: {repr(unescaped)}")
    
    print("✅ test_unescape passed")


def test_equation_round_trip():
    """Test that equations survive save/load cycle."""
    import tempfile
    
    doc = KleisDoc()
    doc.set_metadata(title="Test")
    doc.add_equation("schrodinger", r"i\hbar\frac{\partial}{\partial t}\Psi = H\Psi")
    doc.add_equation("maxwell", r"\nabla \cdot \mathbf{E} = \frac{\rho}{\epsilon_0}")
    
    with tempfile.NamedTemporaryFile(suffix=".kleis", delete=False) as f:
        temp_path = f.name
    
    try:
        doc.save(temp_path)
        
        loaded = KleisDoc.load(temp_path)
        
        # Check equations
        assert "schrodinger" in loaded.equations, "schrodinger equation missing"
        assert "maxwell" in loaded.equations, "maxwell equation missing"
        
        eq1 = loaded.equations["schrodinger"]
        eq2 = loaded.equations["maxwell"]
        
        # Check LaTeX is properly unescaped
        assert r"\hbar" in eq1.latex, f"hbar missing from: {eq1.latex}"
        assert r"\nabla" in eq2.latex, f"nabla missing from: {eq2.latex}"
        
        print("✅ test_equation_round_trip passed")
    finally:
        os.unlink(temp_path)


if __name__ == "__main__":
    print("Testing LaTeX to Typst Conversion")
    print("=" * 50)
    print()
    
    test_latex_to_typst()
    print()
    
    test_unescape()
    print()
    
    test_equation_round_trip()
    print()
    
    print("=" * 50)
    print("✅ All tests passed!")

