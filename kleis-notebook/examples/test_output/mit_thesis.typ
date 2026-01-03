// MIT Thesis Style
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
  Submitted to the Department of \
  Electrical Engineering and Computer Science \
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
