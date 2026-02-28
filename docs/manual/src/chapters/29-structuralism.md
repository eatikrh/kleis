# Structuralism

> *"Structure is the universal abstraction."*

This chapter is not about Kleis the language. It is about the idea that Kleis embodies — and the architecture that follows from that idea.

## The Thesis

Any domain amenable to formalization follows the same pattern:

| Component        | What it does                        |
|------------------|-------------------------------------|
| **Notation**     | Represents objects and relations    |
| **Rules**        | Governs what is valid               |
| **Verification** | Checks that rules hold              |
| **Output**       | Produces a result for humans        |

Mathematics has notation (symbols), rules (axioms), verification (proof), and output (papers). Physics has notation (tensor indices), rules (field equations), verification (experiment or proof), and output (predictions). A code review has notation (source code), rules (coding standards), verification (pass/fail checks), and output (a review report).

Kleis provides the substrate for this universal pattern. Not a tool for one domain — a substrate for all of them.

## Presentation Facilitates Cognition

The fourth component — output — is not an afterthought. Presentation is how knowledge becomes accessible.

This is why Kleis renders equations in proper mathematical typography rather than displaying raw syntax trees. It is why the Kleis grammar uses `∀`, `∃`, `→`, `×` instead of ASCII approximations — the notation on screen should be the notation in the textbook. It is why Kleis generates Typst PDFs suitable for papers and theses, not just terminal dumps.

The principle runs through the entire system. The Equation Editor exists because when you see `Γᵏₘₙ` rendered with proper indices, you immediately understand what it means — the visual form *is* the meaning. Stare at `christoffel(k, -m, -n)` and you have to reconstruct the mathematics in your head. The five render targets exist because the same expression needs to look right in a terminal, a web page, a LaTeX document, a PDF, and a computation engine — and "looking right" is not vanity, it is legibility. Legibility is comprehension. Comprehension is where new ideas come from.

A system that can verify theorems but cannot present them readably fails at half of its purpose. Structure without presentation is a tree falling in a forest.

## Two ASTs

Kleis has two distinct abstract syntax trees. Understanding why is the key to the architecture.

**The Editor AST** lives in the Equation Editor (JavaScript, `static/index.html`). It carries *semantic and visual* information: operation names like `gamma`, `riemann`, `index_mixed`, plus typesetting hints — superscripts, subscripts, matrix delimiters, bracket styles. It knows what something *is* and how it should *look*.

**The Kleis AST** lives in the parser and evaluator (Rust, `src/kleis_parser.rs`). It carries *semantic and computational* information: expressions with source spans, types, operations that can be evaluated, differentiated, and sent to Z3. It knows what something *means* and how to *compute* with it.

The Editor AST is richer visually but cannot compute. The Kleis AST can compute but does not know about rendering. The renderers bridge them.

## Five Projections

The Editor AST renders to five targets:

```
Editor AST (semantic + visual)
    │
    ├── → Unicode    Γᵏₘₙ              (terminal display)
    ├── → LaTeX      \Gamma^{\lambda}_{\mu\nu}   (papers)
    ├── → HTML       <sup>λ</sup>...    (web)
    ├── → Typst      Gamma^λ_{μν}       (PDF generation)
    └── → Kleis      Γ(λ, -μ, -ν)      (computation)
```

The first four targets produce representations for human consumption. The fifth — Kleis notation — produces text that the Kleis parser can ingest, creating a Kleis AST. That AST can then be type-checked, evaluated, differentiated symbolically, or verified by Z3.

This means the Equation Editor is not just a renderer. It is a *compiler* — from visual mathematics to executable specification.

## The Equation Editor Knows Mathematics

The Equation Editor is not a drawing tool. Beneath the visual rendering, it displays the algebraic data type of the expression and has buttons for formal verification:

- **Type display** — shows the inferred type: `Matrix(2, 2, Scalar)`, `Tensor(Metric, [down, down])`, etc.
- **Verify** — sends the Kleis-rendered expression to Z3 for verification against loaded axioms
- **Sat?** — checks satisfiability of the expression as a constraint

The editor knows that covariant indices go down and contravariant indices go up. It knows tensor symmetry axioms. It knows dimension constraints on matrix multiplication. None of this is hardcoded — the knowledge comes from axioms written in Kleis structures (`stdlib/` and user-defined `.kleis` files). Kleis does not just parse mathematical notation; it understands the axiomatic semantics of what it parses. The grammar tells it *how to read*. The axioms tell it *what things mean*.

## Self-Hosting

The universal formula applies to Kleis itself:

| Component        | Kleis-on-Kleis                                    |
|------------------|---------------------------------------------------|
| **Notation**     | `.kleis` source files                             |
| **Rules**        | `axiom` declarations in structures                |
| **Verification** | Pluggable solver backend, type inference, evaluator |
| **Output**       | Typst PDFs, review reports, REPL results          |

The verification slot is not welded to one tool. Z3 is the default SMT solver, but the backend is pluggable — an experimental branch has integrated Isabelle, whose tactic-based proof strategies are a different shape than Z3's satisfiability checks. The slot has a contract (accept a proposition, return a judgment), but the strategies behind that contract vary: SMT solving, tactic-based proof search, and model checking are structurally different ways to inhabit the same slot. The architecture accommodates this, but does not pretend the differences are invisible.

But self-hosting goes deeper than using Kleis to write Kleis programs. The structural scanner for Rust code review — `rust_parser.kleis` — is written *in Kleis*. It defines a `Crate` structure with operations like `scan()`, `crate_functions()`, `non_test_fns_containing()`. The review policy — `rust_review_policy.kleis` — imports this scanner and uses it to analyze Rust source code.

The result is three layers of AST, all evaluated by the same tree walker:

```
Layer 1: rust_review_policy.kleis     (the rules)
    ↓ calls into
Layer 2: rust_parser.kleis            (the scanner)
    ↓ produces
Layer 3: Crate structure              (the scanned Rust code)
    ↓ queried by
Layer 1: rust_review_policy.kleis     (back to the rules)
    ↓ produces
"pass" or "fail: reason"
```

One evaluator. One tree walker. Three languages deep.

## The Seed

The architecture was not designed top-down. It grew from a seed: the template.

The Equation Editor uses templates — patterns with placeholders for sub-expressions. A fraction template has slots for numerator and denominator. A matrix template has slots for entries. A summation template has slots for variable, bounds, and body.

The template system is not a convenience layer — it is an extension point. Templates are designed to be extendable in the same way that Kleis structures and axioms are extendable. Because the platform is axiomatic, a new domain only needs new templates and new structures. Molecular diagrams backed by chemistry structures and reaction axioms written in Kleis. Musical notation backed by counterpoint rules. Circuit schematics backed by Kirchhoff's laws. The template provides the visual entry; the structure provides the semantics; the axioms provide the verification. Same pattern, new domain.

Templates with placeholders for other templates is a recursive structure. It is also, in embryonic form, the Bourbaki program: mathematics built from structures composed of other structures, all the way down.

From templates came the Editor AST. From the Editor AST came the multi-target renderer. From the Kleis render target came the connection to the parser. From the parser came the evaluator. From the evaluator came Z3 verification. From verification came the review engine. Each layer emerged because the substrate — structure with slots for more structure — was the right shape for the next thing.

## The Substrate

Kleis is 7.4 MB. Inside that binary:

- A language parser and evaluator
- Hindley-Milner type inference
- Z3 theorem prover integration
- A WYSIWYG equation editor with five render targets
- LSP server and DAP debugger
- Typst PDF renderer
- Jupyter kernel and REPL
- Three MCP servers (policy, theory, review)
- A structural Rust parser written in itself
- ODE solver, symbolic differentiation, LAPACK numerics
- Tensor calculus, category theory structures, Bell inequality proofs

It does not announce itself as any one of these things. It shows up as whatever you need today — a code reviewer, an equation editor, a proof assistant, a document generator — and the rest is there when you are ready for it.

> *"Any domain that has notation, rules, and outputs fits the same pattern. Kleis provides the substrate."*

Kleis is not a language. It is a structure transformer — accepting structure as input, applying structural rules, and projecting the result into forms that humans and machines can use.

The architecture is the philosophy. The philosophy is the architecture. Structure all the way down.
