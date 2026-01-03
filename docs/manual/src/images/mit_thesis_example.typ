#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1.5in, right: 1in),
  numbering: "1",
)

// Page counter - title page is page 1
#counter(page).update(1)
#set text(
  font: "New Computer Modern",
  size: 12pt,
  lang: "en",
)
#set par(
  justify: true,
  leading: 0.65em,
  first-line-indent: 0.5in,
)

// No indent after headings
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: "1.1")

// Chapter headings (level 1)
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(1in)
  text(size: 16pt, weight: "bold")[Chapter #counter(heading).display()]
  v(0.3in)
  text(size: 14pt, weight: "bold")[#it.body]
  v(0.5in)
}

// Section headings (level 2)
#show heading.where(level: 2): it => {
  v(0.3in)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.15in)
}

// Subsection headings (level 3)
#show heading.where(level: 3): it => {
  v(0.2in)
  text(size: 11pt, weight: "bold", style: "italic")[#counter(heading).display() #it.body]
  v(0.1in)
}
#set figure(placement: auto)
#show figure.caption: it => {
  text(size: 10pt)[#it]
}


#page(margin: (top: 2in, bottom: 1in, left: 1.5in, right: 1in), numbering: none)[
  #align(center)[
    #text(size: 18pt, weight: "bold")[Formal Verification of Knowledge Production Systems]
    
    #v(0.5in)
    by
    #v(0.3in)
    #text(size: 14pt)[Jane Smith]
    
    #v(1in)
    Submitted to the Department of Electrical Engineering and Computer Science #linebreak()
    in partial fulfillment of the requirements for the degree of
    #v(0.3in)
    #text(style: "italic")[Doctor of Philosophy]
    #v(0.3in)
    at the
    #v(0.3in)
    #text(weight: "bold")[MASSACHUSETTS INSTITUTE OF TECHNOLOGY]
    #v(0.3in)
    May 2025
    
    #v(0.3in)
    #text(size: 10pt)[© Jane Smith. This work is licensed under a CC BY-NC-ND 4.0 license.]
  ]
  
  #v(1fr)
  
  #align(left)[
    Thesis Supervisor: Prof. Alice Chen #linebreak()
    Title: Prof. Alice Chen_Formal Verification of Knowledge Production Systems
  ]
]


#page(numbering: none)[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Signature Page]
  ]
  #v(0.5in)
  
  This thesis has been examined by a committee as follows:
  
  #v(1in)
  
  #line(length: 70%, stroke: 0.5pt)
  Prof. Alice Chen #linebreak()
  Thesis Supervisor #linebreak()
  Prof. Alice Chen_Formal Verification of Knowledge Production Systems, Department of Electrical Engineering and Computer Science
  
  #v(0.8in)
  
  #line(length: 70%, stroke: 0.5pt)
  Committee Member Name #linebreak()
  Title #linebreak()
  Department
  
  #v(0.8in)
  
  #line(length: 70%, stroke: 0.5pt)
  Committee Member Name #linebreak()
  Title #linebreak()
  Department
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Abstract]
  ]
  #v(0.3in)
  
  #text(weight: "bold")[Formal Verification of Knowledge Production Systems]
  
  #v(0.2in)
  by Jane Smith
  
  #v(0.2in)
  Submitted to the Department of Electrical Engineering and Computer Science #linebreak()
  on May 2025 in partial fulfillment of the #linebreak()
  requirements for the degree of Doctor of Philosophy
  
  #v(0.3in)
  #par(first-line-indent: 0in)[This thesis presents Kleis, a formal verification system designed as a universal substrate for knowledge production. We demonstrate that mathematical notation, verification rules, and document structure can be treated as first-class concepts that are axiomatized and validated. Our system integrates SMT solvers like Z3 for automated verification, Typst for high-quality document rendering, and Jupyter notebooks for interactive research. We evaluate our approach on case studies in tensor calculus, music theory, and network security, showing that domain-specific notations can be defined while maintaining rigorous verification. The key insight is that knowledge production follows a universal pattern: notation plus rules plus verification plus output. Kleis provides the substrate for this pattern across any domain with formal notation.]
  
  #v(0.5in)
  #text(weight: "bold")[Thesis Supervisor:] Prof. Alice Chen #linebreak()
  #text(weight: "bold")[Title:] Prof. Alice Chen_Formal Verification of Knowledge Production Systems
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Acknowledgments]
  ]
  #v(0.3in)
  
  I would like to express my deepest gratitude to my advisor, Prof. Alice Chen, for her unwavering support, insightful guidance, and endless patience throughout this research journey. Her expertise in formal methods and passion for rigorous thinking have profoundly shaped my approach to computer science.

I am grateful to my thesis committee members for their valuable feedback and thought-provoking questions that helped refine this work.

Special thanks to my colleagues in the Programming Languages group for countless discussions, debugging sessions, and coffee breaks. The collaborative environment at MIT has been instrumental in developing these ideas.

Finally, I thank my family for their love and encouragement. This thesis would not have been possible without their support.
]


#page[
  #v(2in)
  #align(center)[
    #text(style: "italic")[To my parents, who taught me to question everything and never stop learning.]
  ]
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Table of Contents]
  ]
  #v(0.3in)
  #outline(
    title: none,
    depth: 3,
    indent: 1.5em,
  )
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[List of Figures]
  ]
  #v(0.3in)
  #outline(
    title: none,
    target: figure.where(kind: image),
  )
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[List of Tables]
  ]
  #v(0.3in)
  #outline(
    title: none,
    target: figure.where(kind: table),
  )
]


= Introduction

Knowledge production in science and mathematics relies on precise notation and rigorous verification. Traditional approaches separate these concerns, leading to errors and inconsistencies. This thesis presents a unified framework that treats notation, verification, and document structure as first-class concepts.

We introduce Kleis, a system where mathematical statements are simultaneously:
- Notation that renders beautifully (via Typst)
- Assertions that can be verified (via SMT solvers)
- Structured data that can be queried and transformed

The key insight is captured by our main theorem:

== Motivation

The gap between mathematical notation and formal verification has long plagued scientific computing. Researchers must maintain separate representations: one for publication and one for verification. This leads to errors when the two diverge.

Kleis addresses this by treating the document itself as the source of truth. Every equation, every axiom, every theorem is both rendered and verified from the same source.

== Contributions

This thesis makes the following contributions:

1. A unified representation for notation, verification, and document structure
2. Integration with SMT solvers for automated verification
3. High-quality document output via Typst
4. Jupyter notebook integration for interactive research
5. Self-hosting capability where Kleis types are defined in Kleis

$ forall phi . op("axiom")(phi) => op("valid")(phi) $ <eq:verify>

= Background

We build on prior work in formal verification, type theory, and scientific computing. SMT solvers like Z3 provide decidable procedures for many useful theories. Typst offers a modern approach to document typesetting.

Our type system follows Hindley-Milner inference with the following application rule:

Previous work has focused on either verification OR document generation, but not both. We bridge this gap with a unified approach.

$ frac(Gamma tack.r e_1 : tau_1 -> tau_2 quad Gamma tack.r e_2 : tau_1, Gamma tack.r e_1 space e_2 : tau_2) $ <eq:typing>

= The Kleis System

Kleis is built on three key abstractions: structures for mathematical domains, axioms for verification rules, and templates for notation.

The overall architecture is shown below. User documents import domain-specific libraries which define structures, axioms, and notation. The Kleis core dispatches to appropriate backends for type checking, verification, and rendering.

The satisfiability condition that drives verification is:

This combination enables domain-specific languages for any field with formal notation.

== Parser Architecture

The Kleis parser is implemented as a recursive descent parser in Rust. It supports Unicode identifiers, allowing mathematical notation like Greek letters directly in source code. The parser produces an Abstract Syntax Tree (AST) that preserves source location information for error reporting.

== Type System

Kleis employs a Hindley-Milner type system with constraint-based inference. Types are inferred automatically, though explicit annotations are supported. The system handles polymorphic functions and can express complex mathematical structures.

#figure(
  box(stroke: 1pt, inset: 1em)[
      #grid(
        columns: 1,
        row-gutter: 1em,
        align(center)[*User Document* \\ (.kleis file)],
        align(center)[#sym.arrow.b],
        align(center)[*Kleis Core* \\ Parser | Type Checker | Evaluator],
        align(center)[#sym.arrow.b],
        grid(
          columns: 3,
          column-gutter: 2em,
          align(center)[Z3 Backend \\ (verification)],
          align(center)[Type Registry \\ (inference)],
          align(center)[Typst Renderer \\ (output)]
        )
      )
    ],
  caption: [Kleis system architecture showing the three-layer design]
) <fig:architecture>

$ op("SAT")(phi) <=> exists sigma . sigma tack.double phi $ <eq:sat>

= Evaluation

We evaluate Kleis on several case studies including tensor calculus for general relativity, counterpoint rules for music theory, and protocol verification for network security.

Our performance measurements are summarized in the following table. The verification time scales polynomially with domain size.

We also compare Kleis features against existing theorem provers in our feature comparison table.

#figure(
  table(
    columns: (auto, auto, auto, auto),
    align: (left, center, center, center),
    table.header([*Backend*], [*Time (ms)*], [*Memory (MB)*], [*Completeness*]),
    [Z3], [42], [128], [Complete],
    [CVC5], [38], [156], [Complete],
    [Yices], [51], [98], [Incomplete],
    [MiniSat], [12], [32], [Incomplete],
  ),
  caption: [Performance comparison of verification backends]
) <tab:performance>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [Verification Time vs Domain Size],
        xlabel: [Domain Size (x1000)],
        ylabel: [Time (ms)],
        legend: (position: left + top),
        lq.plot((1, 2, 3, 4, 5), (10, 25, 45, 80, 120), mark: "o", label: [Z3]),
        lq.plot((1, 2, 3, 4, 5), (8, 20, 38, 65, 95), mark: "x", label: [CVC5]),
      )
    },
  caption: [Verification time comparison between Z3 and CVC5 backends]
) <fig:performance>

#figure(
  table(
    columns: (auto, auto, auto, auto, auto),
    align: (left, center, center, center, center),
    table.header([*Feature*], [*Kleis*], [*Lean*], [*Coq*], [*Isabelle*]),
    [SMT Integration], [Y], [Y], [-], [-],
    [Custom Notation], [Y], [-], [Y], [Y],
    [Document Export], [Y], [-], [-], [-],
    [Jupyter Support], [Y], [-], [-], [-],
    [Self-Hosting], [Y], [Y], [Y], [Y],
  ),
  caption: [Feature comparison with existing systems]
) <tab:features>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [System Feature Comparison],
        xlabel: [Feature Category],
        ylabel: [Score (1-5)],
        legend: (position: right + top),
        xaxis: (ticks: ((0, [SMT]), (1, [Notation]), (2, [Docs]), (3, [Proofs]))),
        lq.bar((0, 1, 2, 3), (5, 5, 5, 4), offset: -0.2, width: 0.35, label: [Kleis]),
        lq.bar((0, 1, 2, 3), (4, 3, 2, 5), offset: 0.2, width: 0.35, label: [Lean]),
      )
    },
  caption: [Feature comparison between Kleis and Lean across key categories]
) <fig:comparison>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [Type Inference Performance],
        xlabel: [Expression Count],
        ylabel: [Inference Time (ms)],
        xaxis: (scale: "log"),
        yaxis: (scale: "log"),
        lq.scatter((100, 200, 500, 1000, 2000, 5000), (1.2, 2.1, 4.8, 9.5, 18.3, 42.1), mark: "o", label: [Measurements]),
      )
    },
  caption: [Type inference time scales linearly with program size on log-log axes]
) <fig:inference>

== Case Study: Tensor Calculus

General relativity requires complex tensor manipulations with strict symmetry rules. We implemented Einstein notation with automatic index contraction and verified the Bianchi identity. The complexity bound for our verification algorithm is:

$ T(n) = O(n^2 log n) $ <eq:complexity>

#figure(
  table(
    columns: (auto, auto, auto, auto),
    align: (left, center, center, center),
    table.header([*Domain*], [*Axioms*], [*Verification Time*], [*Memory*]),
    [Tensor Calculus], [47], [1.2s], [256 MB],
    [Music Theory], [23], [0.4s], [128 MB],
    [Network Protocols], [31], [0.8s], [192 MB],
    [Financial Models], [18], [0.3s], [96 MB],
  ),
  caption: [Benchmark results across different domains]
) <tab:benchmarks>

== Case Study: Music Theory

Renaissance counterpoint has strict rules about voice leading. We encoded Fux's rules as Kleis axioms and verified counterpoint exercises automatically. This demonstrates Kleis's applicability beyond traditional mathematics.

= Conclusion

We have presented Kleis, a substrate for formal knowledge production. Our main contributions are:

1. A unified representation for notation, verification, and document structure
2. Integration with SMT solvers for automated verification  
3. High-quality document output via Typst
4. Jupyter notebook integration for interactive research

The soundness of our approach is captured by the theorem:

Future work includes extending the solver abstraction layer, building domain-specific libraries, and integrating with additional theorem provers.

$ forall e, tau . tack.r e : tau => tack.r.double e : tau $ <eq:soundness>

$ R_(mu nu) - 1/2 g_(mu nu) R + Lambda g_(mu nu) = (8 pi G) / c^4 T_(mu nu) $ <eq:einstein>

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(1in)
  text(size: 16pt, weight: "bold")[Appendix #counter(heading).display()]
  v(0.3in)
  text(size: 14pt, weight: "bold")[#it.body]
  v(0.5in)
}
= Kleis Grammar

This appendix presents the complete EBNF grammar for the Kleis language.

The grammar is organized into the following sections:
- Top-level declarations (imports, structures, definitions)
- Expressions (function application, operators, literals)
- Types (base types, function types, parameterized types)
- Patterns (for match expressions)

The full grammar specification is available in the project repository at `docs/grammar/kleis_grammar_v05.ebnf`.

= Soundness Proofs

This appendix contains the formal proofs of type soundness for the Kleis type system.

*Theorem (Progress)*: If $e : tau$ then either $e$ is a value or there exists $e'$ such that $e -> e'$.

*Theorem (Preservation)*: If $e : tau$ and $e -> e'$ then $e' : tau$.

The proofs follow standard techniques for Hindley-Milner type systems and are mechanized in Z3.

#pagebreak()
#heading(numbering: none)[References]
[demoura2008] de Moura, L. and Bjorner, N. Z3: An Efficient SMT Solver. In *Proceedings of TACAS 2008*, pp. 337-340.

[typst2023] Madje, M. and Haug, L. Typst: A New Markup-based Typesetting System. 2023. https://typst.app/

[hindley1969] Hindley, R. The Principal Type-Scheme of an Object in Combinatory Logic. *Transactions of the American Mathematical Society*, 146:29-60, 1969.

[milner1978] Milner, R. A Theory of Type Polymorphism in Programming. *Journal of Computer and System Sciences*, 17(3):348-375, 1978.

[einstein1915] Einstein, A. Die Feldgleichungen der Gravitation. *Sitzungsberichte der Preussischen Akademie der Wissenschaften*, pp. 844-847, 1915.

[fux1725] Fux, J.J. Gradus ad Parnassum. Vienna, 1725. Translated by A. Mann as *The Study of Counterpoint*, W.W. Norton, 1965.