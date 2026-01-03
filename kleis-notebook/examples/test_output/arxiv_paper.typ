// arXiv Paper Style (two-column)
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
  *Keywords:* formal verification, type theory, scientific computing, domain-specific languages \
  *PACS:* 03.65.-w, 02.10.Yn \
  *MSC:* 81P05, 03G12 \
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
