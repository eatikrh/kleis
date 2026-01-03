#!/usr/bin/env python3
"""
Test rendering the quadratic formula with KleisDoc.
"""

import sys
import subprocess
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from kleis_kernel.kleisdoc import KleisDoc, Equation

print("=" * 60)
print("Quadratic Formula Rendering Test")
print("=" * 60)

doc = KleisDoc()
doc.set_metadata(title="Quadratic Formula Test", author="Kleis")

# Create the quadratic formula AST: x = (-b ± √(b² - 4ac)) / 2a
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

# Create E=mc² for comparison
emc2_ast = {
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

# Add content
intro = doc.add_section("Introduction", "Testing mathematical formulas.")

# Render equations
print("\n1. Rendering E=mc²...")
typst_emc2 = doc.render_ast(emc2_ast, "typst")
print(f"   Typst: {typst_emc2}")

print("\n2. Rendering quadratic formula...")
typst_quad = doc.render_ast(quadratic_ast, "typst")
print(f"   Typst: {typst_quad}")

latex_quad = doc.render_ast(quadratic_ast, "latex")
print(f"   LaTeX: {latex_quad}")

unicode_quad = doc.render_ast(quadratic_ast, "unicode")
print(f"   Unicode: {unicode_quad}")

# Add equations to document
doc.add_equation("eq:emc2", ast=emc2_ast)
doc.add_equation("eq:quadratic", ast=quadratic_ast)

# Add sections with equations
eq_section = doc.add_section("Famous Equations", "")
eq_section.content.append(doc.get_equation("eq:emc2"))
eq_section.content.append("Einstein's mass-energy equivalence.")
eq_section.content.append(doc.get_equation("eq:quadratic"))
eq_section.content.append("The quadratic formula for solving ax² + bx + c = 0.")

# Export
output_dir = Path(__file__).parent / "test_output"
output_dir.mkdir(exist_ok=True)

typst_path = output_dir / "quadratic_test.typ"
pdf_path = output_dir / "quadratic_test.pdf"

doc.export_typst(str(typst_path))
print(f"\n3. Exported Typst: {typst_path}")

# Compile to PDF
try:
    result = subprocess.run(
        ["typst", "compile", str(typst_path), str(pdf_path)],
        capture_output=True, text=True, check=True
    )
    print(f"4. Compiled PDF: {pdf_path}")
except subprocess.CalledProcessError as e:
    print(f"Error compiling PDF: {e}")
    print(f"Stderr: {e.stderr}")

print("\n" + "=" * 60)
print("Test complete!")
print("=" * 60)

