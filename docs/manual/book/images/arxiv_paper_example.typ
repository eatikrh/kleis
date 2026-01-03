#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  numbering: "1",
  header: align(right)[_Preprint_],
)
#set text(
  font: "New Computer Modern",
  size: 11pt,
  lang: "en",
)
#set par(
  justify: true,
  leading: 0.65em,
  first-line-indent: 1em,
)

// No indent after headings
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: "1.1")

// Section headings (level 1)
#show heading.where(level: 1): it => {
  v(1em)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.5em)
}

// Subsection headings (level 2)
#show heading.where(level: 2): it => {
  v(0.8em)
  text(size: 11pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.4em)
}

// Subsubsection headings (level 3)
#show heading.where(level: 3): it => {
  v(0.6em)
  text(size: 10pt, weight: "bold", style: "italic")[#counter(heading).display() #it.body]
  v(0.3em)
}
#set figure(placement: auto)
#show figure.caption: it => {
  text(size: 9pt)[#it]
}
#show link: it => text(fill: blue.darken(20%))[#underline[#it]]


#align(center)[
  #text(size: 17pt, weight: "bold")[Kleis: A Unified Substrate for Verified Knowledge Production]
  
  #v(1em)
  
  Jane Smith#super[1], Alex Chen#super[2], Maria Garcia#super[1,3]
  
  #v(0.5em)
  
  #super[1]Massachusetts Institute of Technology, Cambridge, MA #linebreak()
    #super[2]University of Michigan, Ann Arbor, MI #linebreak()
    #super[3]Stanford University, Stanford, CA
]

#v(1em)

#align(center)[
  #rect(width: 85%, stroke: none)[
    #align(left)[
      #text(weight: "bold")[Abstract]
      #v(0.3em)
      #text(size: 10pt)[We present Kleis, a novel knowledge production substrate that unifies notation, rules, verification, and output generation across arbitrary domains. Unlike traditional proof assistants that target specific mathematical foundations, Kleis provides a domain-agnostic framework where users define their own structures, axioms, and verification conditions. We demonstrate that this approach enables formal verification in domains ranging from tensor calculus to music theory, while maintaining soundness guarantees through integration with SMT solvers. Our evaluation shows that Kleis can verify complex mathematical identities in milliseconds and generate publication-quality documents directly from verified specifications. We release Kleis as open-source software with comprehensive documentation and examples.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* formal verification, knowledge representation, SMT solving, document generation, domain-specific languages]

#v(1em)


= Introduction

The formalization of mathematical and scientific knowledge has long been a goal of computer science. Systems like Coq, Lean, and Isabelle have demonstrated remarkable success in verifying complex proofs, yet their adoption outside specialized communities remains limited. The gap between formal verification and everyday scientific practice suggests that current approaches may not fully address the needs of working researchers.

We identify three key challenges with existing systems. First, they require significant expertise in dependent type theory or higher-order logic before users can express domain-specific concepts. Second, the notation used in formal proofs often diverges substantially from standard mathematical notation. Third, the gap between verified specifications and publishable documents creates additional work for researchers who must maintain both.

This paper presents Kleis, a system designed to address these challenges through a novel architecture we call the knowledge production substrate. The core insight is that many domains share a common structure: they have notation for expressing concepts, rules that govern valid transformations, and outputs that communicate results. By providing primitives for each of these components, Kleis enables domain experts to create verified workflows without deep expertise in formal methods.

= Related Work

Our work builds on several lines of research in formal verification, domain-specific languages, and literate programming.

== Interactive Theorem Provers

Systems like Coq, Lean, Isabelle, and Agda provide powerful frameworks for constructing formal proofs. These systems have been used to verify major mathematical results including the Four Color Theorem and the Kepler Conjecture. However, their steep learning curves and departure from standard notation limit adoption. Kleis takes a complementary approach: rather than providing a universal foundation, we enable users to define domain-specific foundations that can be verified by external solvers.

== SMT Solvers

Satisfiability Modulo Theories (SMT) solvers like Z3, CVC5, and Yices have become practical tools for automated reasoning. These solvers support decidable theories including linear arithmetic, arrays, and bitvectors. Kleis leverages Z3 as its primary verification backend, automatically translating user-defined axioms into SMT queries. This approach trades the expressiveness of dependent types for automation and decidability.

= The Kleis Architecture

Kleis is organized around three core concepts: structures, axioms, and renderers. Structures define the vocabulary of a domain, axioms specify the rules that govern valid reasoning, and renderers translate verified specifications into outputs.

$ "Domain" = "Notation" + "Rules" + "Verification" + "Output" $ <eq:formula>

== Structures

A structure in Kleis defines a collection of types, operations, and axioms. Unlike type classes in Haskell or traits in Rust, Kleis structures are designed for mathematical specification rather than implementation dispatch.

== Verification

When users write assertions, Kleis translates them into SMT-LIB format and queries Z3. The translation handles quantified formulas, user-defined operations, and domain-specific types. Verification results are reported with counterexamples when assertions fail.

= Evaluation

We evaluate Kleis on three dimensions: verification performance, expressiveness, and usability. Our experiments cover examples from linear algebra, differential geometry, and formal language theory.

#figure(
  table(
      columns: (auto, auto, auto, auto),
      inset: 8pt,
      align: center,
      [*Example*], [*Axioms*], [*Assertions*], [*Time (ms)*],
      [Commutativity], [3], [5], [12],
      [Matrix Algebra], [8], [12], [45],
      [Tensor Identities], [15], [8], [120],
      [Bianchi Identity], [12], [3], [89],
      [Counterpoint Rules], [20], [15], [340],
    ),
  caption: [Verification times for representative examples]
) <tab:results>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [Verification Time vs Axiom Count],
        xlabel: [Number of Axioms],
        ylabel: [Time (ms)],
        lq.plot((5, 10, 15, 20, 25, 30), (15, 35, 80, 150, 280, 450), mark: "o"),
      )
    },
  caption: [Verification time scaling with problem size]
) <fig:performance>

= Case Studies

We present three case studies demonstrating Kleis in different domains.

== Tensor Calculus

We implemented the tensor algebra used in general relativity, including the Riemann curvature tensor, Ricci tensor, and Einstein field equations. Kleis verified the symmetries of these tensors and the Bianchi identities in under 100 milliseconds.

$ G_(mu nu) = R_(mu nu) - 1/2 g_(mu nu) R $ <eq:einstein>

== Music Theory

We encoded the rules of four-voice counterpoint as Kleis axioms. The system can verify that a musical passage satisfies constraints like no parallel fifths, proper voice leading, and resolution of dissonances. This demonstrates that Kleis extends beyond traditional mathematics to any domain with formalizable rules.

= Conclusion

We have presented Kleis, a unified substrate for verified knowledge production. By separating notation, rules, verification, and output, Kleis enables domain experts to create formally verified workflows without deep expertise in proof theory. Our evaluation demonstrates that the approach is both practical and expressive, handling examples from mathematics, physics, and music theory.

Future work includes extending the verification backend to support additional SMT theories, developing a visual equation editor for non-programmers, and creating templates for additional publication formats.

#heading(numbering: none)[Acknowledgments]
We thank the anonymous reviewers for their helpful feedback. This work was supported in part by NSF grant XXX-XXXXXX. Jane Smith is supported by a Google PhD Fellowship.

#heading(numbering: none)[References]
#set text(size: 9pt)
[moura2008] de Moura, L., & Bjorner, N. (2008). Z3: An efficient SMT solver. TACAS 2008.

[nipkow2002] Nipkow, T., Paulson, L., & Wenzel, M. (2002). Isabelle/HOL: A Proof Assistant for Higher-Order Logic. Springer.

[moura2021] de Moura, L., et al. (2021). The Lean 4 Theorem Prover and Programming Language. CADE 2021.

[knuth1984] Knuth, D. E. (1984). Literate Programming. The Computer Journal.

[hahnle2019] Hahnle, R., & Huisman, M. (2019). Deductive Software Verification: From Pen-and-Paper Proofs to Industrial Tools. Computing and Software Science.

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#align(center)[
  #text(size: 14pt, weight: "bold")[Appendix]
]
#v(1em)
= Kleis Grammar Summary

The Kleis grammar supports the following top-level declarations:

- `structure Name(params) { ... }` - Define a structure with types, operations, and axioms
- `implements Structure(args) { ... }` - Provide implementations for a structure
- `data TypeName = Constructor1(...) | Constructor2(...)` - Define algebraic data types
- `define name(params) = expr` - Define functions
- `example "name" { ... }` - Define test cases with assertions

Expressions include:
- Quantifiers: `forall(x : T). P(x)`, `exists(x : T). P(x)`
- Operations: `f(x, y)`, `x + y`, `x * y`
- Let bindings: `let x = e1 in e2`
- Pattern matching: `match e { P1 => e1 | P2 => e2 }`