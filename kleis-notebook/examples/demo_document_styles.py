#!/usr/bin/env python3
"""
Demonstrate KleisDoc with different document styles.

Creates:
1. MIT Thesis style document
2. arXiv paper style document

Both use the same KleisDoc API but produce different outputs.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from kleis_kernel.kleisdoc import KleisDoc

output_dir = Path(__file__).parent / "test_output"
output_dir.mkdir(exist_ok=True)

# =============================================================================
# Document 1: MIT Thesis Style
# =============================================================================
print("=" * 60)
print("Creating MIT Thesis Style Document")
print("=" * 60)

thesis = KleisDoc.new()

# Thesis-specific metadata
thesis.set_metadata(
    title="Formal Verification of Knowledge Production Systems",
    author="Jane Smith",
    date="May 2025",
    abstract="This thesis presents a unified framework for formal verification of knowledge production systems. We introduce Kleis, a language that treats notation, verification, and document structure as first-class concepts."
)

# Thesis-specific content blocks
thesis.set_content_block("degree", "Doctor of Philosophy")
thesis.set_content_block("department", "Electrical Engineering and Computer Science")
thesis.set_content_block("supervisor", "Prof. Alice Chen")
thesis.set_content_block("institution", "Massachusetts Institute of Technology")

# Add chapters (sections at level 1 = chapters for thesis)
intro = thesis.add_section("Introduction", """
Knowledge production in science and mathematics relies on precise notation 
and rigorous verification. Traditional approaches separate these concerns, 
leading to errors when notation outpaces verification.

This thesis addresses this fundamental tension by introducing a unified framework.
""", level=1)

# Add equation with AST
thesis.add_equation("eq:main", ast={
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
}, section=intro)

methods = thesis.add_section("Methods", """
We develop a type-theoretic foundation based on dependent types and 
homotopy type theory, enabling both computation and verification 
within a single framework.
""", level=1)

thesis.add_theorem("thm:soundness", "theorem", 
    "The type system is sound: well-typed terms do not get stuck.",
    proof="By induction on the structure of derivations...")

results = thesis.add_section("Results", """
Our implementation demonstrates significant improvements in 
verification time while maintaining expressiveness.
""", level=1)

thesis.add_table("tab:performance", 
    ["System", "Verification Time", "Expressiveness"],
    [["Coq", "2.3s", "High"],
     ["Lean", "1.8s", "High"],
     ["Kleis", "0.4s", "High"]])

conclusion = thesis.add_section("Conclusion", """
We have presented Kleis, a unified framework for knowledge production
that bridges the gap between notation and verification.
""", level=1)

# Save as .kleis
thesis.save(str(output_dir / "mit_thesis.kleis"))
print(f"Saved: {output_dir / 'mit_thesis.kleis'}")

# Generate MIT-style Typst
mit_typst = output_dir / "mit_thesis.typ"
with open(mit_typst, "w") as f:
    f.write("""// MIT Thesis Style
#set document(title: "Formal Verification of Knowledge Production Systems", author: "Jane Smith")
#set page(paper: "us-letter", margin: (top: 1.5in, bottom: 1in, left: 1.5in, right: 1in))
#set text(font: "New Computer Modern", size: 12pt)
#set heading(numbering: "1.1")
#set par(justify: true, leading: 0.65em)

// Title Page
#align(center)[
  #v(2in)
  #text(size: 18pt, weight: "bold")[Formal Verification of Knowledge Production Systems]
  #v(0.5in)
  by
  #v(0.3in)
  #text(size: 14pt)[Jane Smith]
  #v(1in)
  Submitted to the Department of \\
  Electrical Engineering and Computer Science \\
  in partial fulfillment of the requirements for the degree of
  #v(0.3in)
  #text(style: "italic")[Doctor of Philosophy]
  #v(0.3in)
  at the
  #v(0.3in)
  #text(weight: "bold")[MASSACHUSETTS INSTITUTE OF TECHNOLOGY]
  #v(0.3in)
  May 2025
]
#pagebreak()

// Abstract
#heading(level: 1, numbering: none)[Abstract]
This thesis presents a unified framework for formal verification of knowledge production systems. We introduce Kleis, a language that treats notation, verification, and document structure as first-class concepts.

#pagebreak()

// Chapters
#heading(level: 1)[Introduction]
Knowledge production in science and mathematics relies on precise notation and rigorous verification. Traditional approaches separate these concerns, leading to errors when notation outpaces verification.

This thesis addresses this fundamental tension by introducing a unified framework.

The fundamental equation is:
$ E = m c^2 $ <eq:main>

#heading(level: 1)[Methods]
We develop a type-theoretic foundation based on dependent types and homotopy type theory, enabling both computation and verification within a single framework.

#block(stroke: 1pt + gray, inset: 10pt, radius: 4pt)[
  *Theorem 1 (Soundness).* The type system is sound: well-typed terms do not get stuck.
  
  _Proof._ By induction on the structure of derivations...
]

#heading(level: 1)[Results]
Our implementation demonstrates significant improvements in verification time while maintaining expressiveness.

#figure(
  table(
    columns: 3,
    [*System*], [*Verification Time*], [*Expressiveness*],
    [Coq], [2.3s], [High],
    [Lean], [1.8s], [High],
    [Kleis], [0.4s], [High],
  ),
  caption: [Performance comparison]
) <tab:performance>

#heading(level: 1)[Conclusion]
We have presented Kleis, a unified framework for knowledge production that bridges the gap between notation and verification.
""")
print(f"Generated: {mit_typst}")

# Compile to PDF
import subprocess
try:
    result = subprocess.run(["typst", "compile", str(mit_typst), str(output_dir / "mit_thesis.pdf")],
                          capture_output=True, text=True)
    if result.returncode == 0:
        print(f"✓ Compiled: {output_dir / 'mit_thesis.pdf'}")
    else:
        print(f"Error: {result.stderr}")
except FileNotFoundError:
    print("typst not found")

# =============================================================================
# Document 2: arXiv Paper Style
# =============================================================================
print("\n" + "=" * 60)
print("Creating arXiv Paper Style Document")
print("=" * 60)

paper = KleisDoc.new()

# arXiv-specific metadata
paper.set_metadata(
    title="Kleis: A Unified Framework for Knowledge Production",
    author="Jane Smith, Alice Chen",
    date="January 2025",
    abstract="We present Kleis, a domain-specific language for knowledge production that unifies notation, verification, and document generation. Our approach is based on dependent type theory and enables both symbolic computation and formal verification within a single framework."
)

# arXiv-specific content blocks
paper.set_content_block("arxiv_id", "2501.12345")
paper.set_content_block("pacs_numbers", ["03.65.-w", "02.10.Yn"])
paper.set_content_block("msc_codes", ["81P05", "03G12"])
paper.set_content_block("keywords", ["formal verification", "type theory", "scientific computing", "domain-specific languages"])
paper.set_content_block("acknowledgments", "This work was supported by NSF grant XXX-YYYY.")

# Paper sections
intro = paper.add_section("Introduction", """
The landscape of scientific computing is characterized by a growing gap 
between expressive notation and formal verification. Existing tools 
either prioritize expressiveness (Mathematica, Maple) or verification 
(Coq, Lean), but rarely both.
""")

# Add the quadratic formula
paper.add_equation("eq:quadratic", ast={
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
                                                    {"Operation": {"name": "scalar_multiply", "args": [{"Const": "4"}, {"Object": "a"}]}},
                                                    {"Object": "c"}
                                                ]
                                            }}
                                        ]
                                    }}
                                ]
                            }}
                        ]
                    }},
                    {"Operation": {"name": "scalar_multiply", "args": [{"Const": "2"}, {"Object": "a"}]}}
                ]
            }}
        ]
    }
}, section=intro)

related = paper.add_section("Related Work", """
Prior work in this area includes the Lean theorem prover, which provides
a foundation for mathematical formalization, and Typst, which enables
high-quality document generation.
""")

methods = paper.add_section("Methodology", """
Our approach combines three key innovations: a dependent type system,
a template-based rendering engine, and an axiom verification system.
""")

paper.add_theorem("thm:completeness", "theorem",
    "For any well-formed Kleis program P, if P type-checks, then P terminates with a value or diverges.")

results = paper.add_section("Results", """
We evaluated Kleis on a benchmark of 100 mathematical proofs from 
various domains including analysis, algebra, and topology.
""")

paper.add_table("tab:benchmarks",
    ["Domain", "Problems", "Verified", "Time (avg)"],
    [["Analysis", "35", "35", "0.3s"],
     ["Algebra", "40", "38", "0.5s"],
     ["Topology", "25", "24", "0.4s"]])

conclusion = paper.add_section("Conclusion", """
We have presented Kleis, a unified framework that bridges the gap between
expressive notation and formal verification. Future work will extend
the system to support interactive theorem proving.
""")

# Save as .kleis
paper.save(str(output_dir / "arxiv_paper.kleis"))
print(f"Saved: {output_dir / 'arxiv_paper.kleis'}")

# Generate arXiv-style Typst
arxiv_typst = output_dir / "arxiv_paper.typ"
with open(arxiv_typst, "w") as f:
    f.write("""// arXiv Paper Style (two-column)
#set document(title: "Kleis: A Unified Framework for Knowledge Production", author: "Jane Smith, Alice Chen")
#set page(paper: "us-letter", margin: 0.75in, columns: 1)
#set text(font: "New Computer Modern", size: 10pt)
#set heading(numbering: "1.")
#set par(justify: true, first-line-indent: 1em)

// Header
#align(center)[
  #text(size: 16pt, weight: "bold")[Kleis: A Unified Framework for Knowledge Production]
  #v(0.3in)
  #text(size: 11pt)[Jane Smith#super[1], Alice Chen#super[1]]
  #v(0.1in)
  #text(size: 9pt, style: "italic")[#super[1]MIT CSAIL, Cambridge, MA]
  #v(0.2in)
]

// Abstract box
#block(fill: luma(245), inset: 10pt, radius: 4pt, width: 100%)[
  #text(weight: "bold")[Abstract.]
  We present Kleis, a domain-specific language for knowledge production that unifies notation, verification, and document generation. Our approach is based on dependent type theory and enables both symbolic computation and formal verification within a single framework.
]

#v(0.1in)
#text(size: 9pt)[
  *Keywords:* formal verification, type theory, scientific computing, domain-specific languages \\
  *PACS:* 03.65.-w, 02.10.Yn \\
  *MSC:* 81P05, 03G12 \\
  *arXiv:* 2501.12345
]
#v(0.2in)

#heading[Introduction]
The landscape of scientific computing is characterized by a growing gap between expressive notation and formal verification. Existing tools either prioritize expressiveness (Mathematica, Maple) or verification (Coq, Lean), but rarely both.

The well-known quadratic formula demonstrates our notation:
$ x = frac(-b plus.minus sqrt(b^2 - 4 a c), 2 a) $ <eq:quadratic>

#heading[Related Work]
Prior work in this area includes the Lean theorem prover, which provides a foundation for mathematical formalization, and Typst, which enables high-quality document generation.

#heading[Methodology]
Our approach combines three key innovations: a dependent type system, a template-based rendering engine, and an axiom verification system.

#block(stroke: (left: 3pt + blue), inset: (left: 10pt, y: 5pt))[
  *Theorem 1.* For any well-formed Kleis program P, if P type-checks, then P terminates with a value or diverges.
]

#heading[Results]
We evaluated Kleis on a benchmark of 100 mathematical proofs from various domains including analysis, algebra, and topology.

#figure(
  table(
    columns: 4,
    stroke: 0.5pt,
    [*Domain*], [*Problems*], [*Verified*], [*Time (avg)*],
    [Analysis], [35], [35], [0.3s],
    [Algebra], [40], [38], [0.5s],
    [Topology], [25], [24], [0.4s],
  ),
  caption: [Benchmark results across mathematical domains]
) <tab:benchmarks>

#heading[Conclusion]
We have presented Kleis, a unified framework that bridges the gap between expressive notation and formal verification. Future work will extend the system to support interactive theorem proving.

#v(0.3in)
#line(length: 100%, stroke: 0.5pt)
#text(size: 9pt)[
  *Acknowledgments.* This work was supported by NSF grant XXX-YYYY.
]
""")
print(f"Generated: {arxiv_typst}")

# Compile to PDF
try:
    result = subprocess.run(["typst", "compile", str(arxiv_typst), str(output_dir / "arxiv_paper.pdf")],
                          capture_output=True, text=True)
    if result.returncode == 0:
        print(f"✓ Compiled: {output_dir / 'arxiv_paper.pdf'}")
    else:
        print(f"Error: {result.stderr}")
except FileNotFoundError:
    print("typst not found")

print("\n" + "=" * 60)
print("Summary")
print("=" * 60)
print(f"""
Generated files:
  MIT Thesis:
    - {output_dir / 'mit_thesis.kleis'}
    - {output_dir / 'mit_thesis.typ'}
    - {output_dir / 'mit_thesis.pdf'}
    
  arXiv Paper:
    - {output_dir / 'arxiv_paper.kleis'}
    - {output_dir / 'arxiv_paper.typ'}  
    - {output_dir / 'arxiv_paper.pdf'}

Key differences:
  MIT Thesis                    arXiv Paper
  -----------                   -----------
  US Letter, wide margins       US Letter, narrow margins
  12pt New Computer Modern      10pt New Computer Modern
  Chapter numbering (1, 2...)   Section numbering (1., 2.)
  Formal title page             Compact header with affiliations
  Separate abstract page        Abstract box at top
  degree, department, etc.      PACS, MSC codes, arXiv ID
""")

