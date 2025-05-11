# Kleis

**Kleis** is a symbolic language and computational framework for formalizing **Projected Ontology Theory (POT)** and **Hilbert Ontology (HONT)**.

It blends metaphysical clarity with mathematical structure to express concepts like modal flow, projection, residue, bifurcation, and emergent phenomena. Kleis aims to unify ontological reasoning, physics-inspired structure, and symbolic computation under a single formal umbrella.

---

## 🧠 What is Kleis?

Kleis serves two main purposes:
1. A **symbolic language** for expressing modal ontological structures.
2. A **Rust-based engine** for parsing, rendering, and evaluating expressions in the Kleis framework.

Kleis is inspired by:
- Functional symbolic systems like **LISP** and **REDUCE**
- Formal language design
- The deep need to represent non-temporal structure, projection mechanics, and modal coherence in a computable and legible way

---

## 📁 Project Structure

## Formal Statement for Kleis Future Research

Cognitive load associated with mathematical expressions can be estimated by a combination of symbolic description length, derivation step counts, and chunking effectiveness.
While absolute quantization is difficult, relative cognitive complexity can often be inferred, and may guide automatic simplification and beautification decisions in Kleis.

## Formal Philosophical Statement for Kleis

Simplification, like typesetting, is fundamentally about cognition.
It is about producing forms that are easier for humans to understand,
not necessarily changing the mathematical correctness.

Simplification in Kleis is a cognitive optimization layer.
Its purpose is not to alter mathematical truth, but to reframe expressions into forms that are easier for humans to grasp, recognize patterns in, and reason about.
Simplification parallels typesetting: both optimize cognitive access to underlying meaning without modifying the meaning itself.

## Formal Conclusion for Kleis Specification

In Kleis, Expression Evaluation is minimal, meaning-preserving, and necessary.
Simplification is a separate, optional service that applies known mathematical identities to transform expressions without altering their meaning.
Simplification may involve heuristic or exhaustive search techniques, but it remains distinct from basic evaluation.

## Formal Statement for Kleis Future Specification

Expression evaluation in Kleis is not mere computation.
It is a controlled, meaning-preserving, type-aware unfolding of symbolic structures.
Evaluation rules must respect semantic meaning, context, types, and proof obligations.
Where evaluation cannot proceed without assumptions, the system must preserve unevaluated forms explicitly.


## Mental Model for Tackling Expression Evaluation

| Phase                            | Approach                                                                              |
| :------------------------------- | :------------------------------------------------------------------------------------ |
| **Symbolic execution**           | Traverse expression tree, apply meaning-aware transformations                         |
| **Context awareness**            | Carry symbol tables, type environments, and simplification rules alongside evaluation |
| **Deferred evaluation**          | Where needed, return "unevaluated forms" to avoid wrong assumptions                   |
| **Explicit typing**              | Always know the "type" (scalar, vector, operator) at each node                        |
| **Proof-backed transformations** | In sensitive cases, allow only transformations that can be justified or proven valid  |

## Why Expression Evaluation is Hairy

| Challenge                      | Why It’s Hard                                                                                   |
| :----------------------------- | :---------------------------------------------------------------------------------------------- |
| **Symbolic algebra**           | Not just numbers — you must manipulate structures, preserve symbolic relationships              |
| **Scope and context**          | Variables, constants, operators might have different meanings based on definition layers        |
| **Type resolution**            | Expressions are not just scalars — they can be vectors, matrices, spinors, tensors, functors    |
| **Lazy vs eager evaluation**   | Do you compute immediately, or delay until more information arrives? (Critical for proofs)      |
| **Side conditions**            | Surface integral over what? Does surface have orientation? Does grad act on scalars or vectors? |
| **Simplification rules**       | Not all expressions can/should be simplified automatically; you must control it carefully       |
| **Non-commutative operations** | $AB \neq BA$ for operators, matrices, etc.                                                      |
| **Units and dimensions**       | (Maybe later) — quantities with units (e.g., meters, seconds) cannot be blindly combined        |
| **Branching evaluation**       | Certain operations produce multiple outcomes (e.g., ± roots)                                    |
| **Proof obligations**          | You might have to prove that certain manipulations are valid before simplifying                 |


## Some Future Vision

In the future, instead of just reading natural language responses,
I will ask the LLM to rephrase the reasoning into formal Kleis DSL.
Then the "reasoning" becomes part of a structured, provable formal chain — no hallucination possible.
