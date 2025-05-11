# ADR-003: Parsing Strategy and Eventual Self-Hosting of Kleis

## Status
Accepted

## Context
Kleis currently uses an external parser (e.g., written in Rust using `pest`, `nom`, or `lalrpop`) to parse `.kleis` input files and produce symbolic expression trees.

As the language grows to include higher-order concepts (e.g., Functors, Categories, Law Inference, Proofs), hand-rolled parsing and evaluation in Rust will become:
- Verbose and hard to maintain
- Inflexible in face of symbolic meta-rules
- Incapable of introspection and transformation within the language itself

Historical precedent from Lisp, Coq, Lean, Agda, and others shows that mature symbolic systems benefit from self-hosting and introspective manipulation.

## Decision
Kleis will adopt a phased strategy for self-hosting:

### Phase 1: External Parser (Rust)
- Use `pest` or `nom` to parse Kleis DSL and generate AST structs.
- Keep rendering and basic symbolic evaluation in Rust.

### Phase 2: Internal Interpreter
- Add AST walking logic in Kleis for symbolic evaluation and simplification.
- Begin expressing laws and transformation rules in Kleis syntax.

### Phase 3: Bootstrapped Self-Hosting
- Allow Kleis to parse, interpret, and transform its own expression structures.
- Enable definition of evaluators, proof systems, and simplifiers *within Kleis*.

## Consequences
- Parsing remains performant and flexible in early versions.
- Self-hosting allows extensibility, introspection, and semantic expressiveness.
- Maintains architectural clarity: Rust for bootstrapping, Kleis for symbolic logic.
- Aligns Kleis with systems like Coq, Lean, Lisp, TeX, where language is data.

## Related
- ADR-002: Separation of Evaluation and Simplification
- Future: Meta-programming, transformation rules, cognitive-cost modeling