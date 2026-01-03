#!/usr/bin/env python3
"""
End-to-end test of the KleisDoc rendering pipeline.

Tests: Python → Kleis Server → Typst → PDF

Requirements:
- Kleis server running on port 3000 (`cargo run --bin server`)
- Typst installed (`brew install typst` or similar)

Run from kleis-notebook directory:
    python examples/test_render_pipeline.py
"""

import sys
import subprocess
from pathlib import Path

# Add parent to path for imports
script_dir = Path(__file__).parent
sys.path.insert(0, str(script_dir.parent))

from kleis_kernel.kleisdoc import KleisDoc, Equation, Section

print("=" * 60)
print("KleisDoc End-to-End Rendering Pipeline Test")
print("=" * 60)

# Create document
doc = KleisDoc()
doc.set_metadata(title="Test Document", author="Test Author")

# Test 1: Check server connection
print("\n1. Testing server connection...")
if doc._check_server():
    print("   ✅ Kleis server is running")
else:
    print("   ❌ Kleis server not available!")
    print("   Start it with: cargo run --bin server")
    sys.exit(1)

# Test 2: Render simple equation via server
print("\n2. Testing equation rendering via server...")
einstein_ast = {
    "Operation": {
        "name": "equals",
        "args": [
            {"Object": "E"},
            {"Operation": {
                "name": "scalar_multiply",
                "args": [
                    {"Object": "m"},
                    {"Operation": {"name": "power", "args": [{"Object": "c"}, {"Const": "2"}]}}
                ]
            }}
        ]
    }
}

typst_result = doc.render_ast(einstein_ast, "typst")
if typst_result:
    print(f"   ✅ Typst output: {typst_result}")
else:
    print("   ❌ Failed to render equation")
    sys.exit(1)

latex_result = doc.render_ast(einstein_ast, "latex")
if latex_result:
    print(f"   ✅ LaTeX output: {latex_result}")
else:
    print("   ❌ Failed to render equation as LaTeX")

unicode_result = doc.render_ast(einstein_ast, "unicode")
if unicode_result:
    print(f"   ✅ Unicode output: {unicode_result}")
else:
    print("   ❌ Failed to render equation as Unicode")

# Test 3: Build document with equation
print("\n3. Building document with equation...")
intro = doc.add_section("Introduction", "This document tests the rendering pipeline.")

# Add equation with AST
eq = Equation(
    id="eq1",
    label="eq:einstein",
    ast=einstein_ast,
    numbered=True
)
doc.equations["eq:einstein"] = eq
intro.content.append(eq)

doc.add_section("Conclusion", "The test was successful.")

print(f"   ✅ Document has {len(doc.sections)} sections")
print(f"   ✅ Document has {len(doc.equations)} equations")

# Test 4: Generate Typst output
print("\n4. Generating Typst source...")
output_dir = script_dir / "test_output"
output_dir.mkdir(exist_ok=True)

typst_path = output_dir / "test_doc.typ"
doc.export_typst(str(typst_path))

typst_content = typst_path.read_text()
print(f"   ✅ Generated {len(typst_content)} chars of Typst code")
print(f"   ✅ Saved to: {typst_path}")

# Check if equation was rendered
if "E = m c^(2)" in typst_content:
    print("   ✅ Equation correctly rendered in document")
else:
    print("   ⚠️  Equation may not be rendered correctly")
    print("   Content preview:")
    print(typst_content[:500])

# Test 5: Compile to PDF (if typst available)
print("\n5. Compiling to PDF...")
try:
    pdf_path = output_dir / "test_doc.pdf"
    result = subprocess.run(
        ["typst", "compile", str(typst_path), str(pdf_path)],
        capture_output=True,
        text=True
    )
    if result.returncode == 0:
        pdf_size = pdf_path.stat().st_size
        print(f"   ✅ PDF generated: {pdf_path} ({pdf_size} bytes)")
    else:
        print(f"   ❌ Typst compilation failed:")
        print(f"   {result.stderr}")
except FileNotFoundError:
    print("   ⚠️  Typst not installed, skipping PDF generation")
    print("   Install with: brew install typst")

# Test 6: Test quadratic formula (more complex AST)
print("\n6. Testing complex equation (quadratic formula)...")
quadratic_ast = {
    "Operation": {
        "name": "equals",
        "args": [
            {"Object": "x"},
            {"Operation": {
                "name": "frac",
                "args": [
                    {"Operation": {
                        "name": "plus_minus",
                        "args": [
                            {"Operation": {"name": "negate", "args": [{"Object": "b"}]}},
                            {"Operation": {
                                "name": "sqrt",
                                "args": [
                                    {"Operation": {
                                        "name": "minus",
                                        "args": [
                                            {"Operation": {"name": "power", "args": [{"Object": "b"}, {"Const": "2"}]}},
                                            {"Operation": {
                                                "name": "scalar_multiply",
                                                "args": [
                                                    {"Operation": {
                                                        "name": "scalar_multiply",
                                                        "args": [{"Const": "4"}, {"Object": "a"}]
                                                    }},
                                                    {"Object": "c"}
                                                ]
                                            }}
                                        ]
                                    }}
                                ]
                            }}
                        ]
                    }},
                    {"Operation": {
                        "name": "scalar_multiply",
                        "args": [{"Const": "2"}, {"Object": "a"}]
                    }}
                ]
            }}
        ]
    }
}

quadratic_typst = doc.render_ast(quadratic_ast, "typst")
if quadratic_typst:
    print(f"   ✅ Quadratic formula Typst: {quadratic_typst}")
    # Check for expected elements
    if "plus.minus" in quadratic_typst:
        print("   ✅ Plus-minus operator rendered correctly")
    if "frac" in quadratic_typst:
        print("   ✅ Fraction rendered correctly")
else:
    print("   ❌ Failed to render quadratic formula")

print("\n" + "=" * 60)
print("All tests completed!")
print("=" * 60)

