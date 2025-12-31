# The Kleis Manual

> *"Mathematics is the language with which God has written the universe."* — Galileo Galilei

Welcome to **The Kleis Manual**, the official guide to the Kleis mathematical specification language.

## What is Kleis?

Kleis is a **structure-oriented mathematical formalization language** with Z3 verification and LAPACK numerics.

> **Philosophy:** *Structures — the foundation of everything.*

| Metric | Value |
|--------|-------|
| Grammar | Fully implemented |
| Tests | 1,762 Rust unit tests |
| Examples | 71 Kleis files across 15+ domains |
| Built-in Functions | 100+ (including LAPACK numerical operations) |

### Core Capabilities

- 🏗️ **Structure-first design** — define mathematical objects by their axioms, not just their data
- ✅ **Z3 verification** — prove properties with SMT solving
- 🔢 **LAPACK numerics** — eigenvalues, SVD, matrix exponentials, and more
- 📐 **Symbolic mathematics** — work with expressions, not just numbers
- 🔬 **Scientific computing** — differential geometry, tensor calculus, control systems
- 🔄 **Turing complete** — a full programming language, not just notation

> **Computational Universality:** Kleis is Turing complete. This was demonstrated by implementing a complete LISP interpreter in Kleis (see [Appendix: LISP Interpreter](./appendix/lisp-interpreter.md)). The combination of algebraic data types, pattern matching, and recursion enables arbitrary computation.

### Domain Coverage

Kleis has been used to formalize:

| Domain | Examples |
|--------|----------|
| **Mathematics** | Differential forms, tensor algebra, complex analysis, number theory |
| **Physics** | Dimensional analysis, quantum entanglement, orbital mechanics |
| **Control Systems** | LQG controllers, eigenvalue analysis, state-space models |
| **Ontology** | Projected Ontology Theory, spacetime types |
| **Protocols** | IPv4 packets, IP routing, stop-and-wait ARQ |
| **Authorization** | OAuth2 scopes, Google Zanzibar |
| **Formal Methods** | Petri nets, mutex verification |
| **Games** | Chess, Contract Bridge, Sudoku |

## Who is This For?

Kleis is for anyone who thinks in terms of **structures and axioms**:

- **Mathematicians** — formalize theorems, verify properties, explore number theory
- **Physicists** — tensor algebra, differential geometry, dimensional analysis
- **Engineers** — control systems, protocol specifications, verified designs
- **Security architects** — authorization policies (Zanzibar, OAuth2)
- **Researchers** — formalize new theories with Z3 verification
- **Functional programmers** — if you enjoy Haskell or ML, you'll feel at home

If you've ever wished you could *prove* your specifications are consistent, Kleis is for you.

## Why Kleis Now?

Modern systems demand formal verification:

| Challenge | How Kleis Helps |
|-----------|-----------------|
| **Security & Compliance** | Machine-checkable proofs for audit trails across sectors |
| **Complex Systems** | Verify rules across IoT, enterprise, and distributed systems |
| **AI-Generated Content** | Verify AI outputs against formal specifications |

> *Universal verification* — same rigor for mathematics, business rules, and beyond.

## How to Read This Guide

Each chapter builds on the previous ones. We start with the basics:

1. **Starting Out** — expressions, operators, basic syntax
2. **Types** — naming and composing structures
3. **Functions** — operations with laws

Then we explore core concepts:

4. **Algebraic Types** — data definitions and constructors
5. **Pattern Matching** — elegant case analysis
6. **Let Bindings** — local definitions
7. **Quantifiers and Logic** — ∀, ∃, and logical operators
8. **Conditionals** — if-then-else

And advanced features:

9. **Structures** — the foundation of everything
10. **Implements** — structure implementations
11. **Z3 Verification** — proving things with SMT

> **Philosophy:** In Kleis, *structures* define what things **are** through their operations and axioms. Types are names for structures. A metric tensor isn't "a 2D array" — it's "something satisfying metric axioms."

## A Taste of Kleis

Here's what Kleis looks like:

```kleis
// Define a function
define square(x) = x * x

// With type annotation
define double(x : ℝ) : ℝ = x + x

// Create a structure with axioms
structure Group(G) {
    operation e : G                    // identity
    operation inv : G → G              // inverse
    operation mul : G × G → G          // multiplication
    
    axiom left_identity : ∀ x : G . mul(e, x) = x
    axiom left_inverse : ∀ x : G . mul(inv(x), x) = e
}

// Numerical computation
example "eigenvalues" {
    let A = Matrix([[1, 2], [3, 4]]) in
    out(eigenvalues(A))  // Pretty-printed output
}
```

## Getting Started

Ready? Let's dive in!

→ [Start with Chapter 1: Starting Out](./chapters/01-starting-out.md)
