# ADR-002: Separation of Expression Evaluation and Simplification

## Status
Accepted

## Context
During design of expression evaluation, it became evident that some operations (e.g. binomial expansion, trigonometric identities) are not strictly part of "evaluation" but rather "simplification".

Simplification can involve:
- Identity rule matching
- Heuristic or exhaustive transformation
- Human-friendly reformatting

Evaluation, by contrast, must:
- Preserve semantics exactly
- Avoid unnecessary transformations
- Be minimal, deterministic, and safe

## Decision
Kleis will maintain a **strict separation** between:
- **Expression Evaluation**: Necessary, semantic computation (e.g., symbolic application of grad, division, substitution)
- **Simplification Service**: Optional, transformation engine applying identity rules, reductions, and formatting

## Consequences
- Evaluation can be used reliably as a backend for proofs, transformation, or rendering
- Simplification remains an opt-in cognitive ergonomics layer
- Architecture can now cleanly isolate deterministic evaluation from potentially expansive transformation processes

## Related
- Kleis Grammar v0.2
- Planned: cognitive weight modeling and simplifier heuristics