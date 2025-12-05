# Commit Strategy: Type System Work

**Question:** Which files should we commit?

---

## Essential Files (MUST Commit)

### 1. Implementation ⭐⭐⭐
**Critical for continuing work**

```
src/type_inference.rs               Type inference POC (working!)
examples/type_inference_demo.rs     Demonstration
```

### 2. Standard Library ⭐⭐⭐
**Critical - the actual Kleis standard library**

```
stdlib/prelude.kleis                Kleis standard library code
stdlib/README.md                    Overview
```

### 3. Formal Grammars ⭐⭐⭐
**Critical - needed to parse stdlib**

```
docs/grammar/Kleis_v03.g4           ANTLR4 grammar
docs/grammar/kleis_grammar_v03.ebnf EBNF grammar
docs/grammar/kleis_grammar_v03.md   Prose documentation
```

### 4. Architectural Decision ⭐⭐⭐
**Critical - documents the decision**

```
docs/adr-014-hindley-milner-type-system.md
```

### 5. Core Type System Docs ⭐⭐
**Important - reference documentation**

```
docs/type-system/KLEIS_TYPE_SYSTEM.md          Core spec (existing)
docs/type-system/CONTEXT_AND_OPERATIONS.md     How it all works
docs/type-system/TYPE_INFERENCE_POC.md         Current status
```

### 6. Overview for NotebookLM ⭐⭐
**Important - external documentation**

```
docs/KLEIS_OVERVIEW.md
docs/KLEIS_OVERVIEW.pdf
```

### 7. Session Summary ⭐
**Helpful for context**

```
SESSION_SUMMARY_2024-12-05_TYPE_SYSTEM.md
```

---

## Optional Files (Consider Archiving)

### Design Exploration Docs ⚠️
**Useful but not critical to continue work**

```
docs/type-system/HASKELL_INTEGRATION.md        Design reasoning
docs/type-system/TYPE_CHECKING_UX.md           UX exploration
docs/type-system/SYNTAX_COMPARISON_AND_PROPOSAL.md  Design choices
```

**Option 1:** Commit all (comprehensive documentation)  
**Option 2:** Archive to `docs/archive/design-notes/` (keep repo lighter)

**Recommendation:** Commit them - they document WHY we made choices

### Examples ⚠️
```
docs/type-system/examples/context_bootstrap_demo.md
```

**Recommendation:** Commit - helpful for understanding

---

## Recommended Commit Strategy

### Commit 1: Type System Core
```bash
git add src/type_inference.rs
git add examples/type_inference_demo.rs
git add src/lib.rs  # (updated to include type_inference module)
git commit -m "Type System POC: Hindley-Milner inference for symbolic math

- Implement Type enum (Scalar, Vector, Matrix, Var, Function, ForAll)
- Implement unification algorithm
- Implement constraint generation and solving
- Working POC with 9 test cases
- Proof: Type checking works on symbolic expressions
- Infers polymorphic types (α, β) and concrete types (ℝ)

Status: POC complete, ready for expansion"
```

### Commit 2: Standard Library
```bash
git add stdlib/prelude.kleis
git add stdlib/README.md
git commit -m "Standard Library: Self-hosting algebraic structures

- stdlib/prelude.kleis: 500 lines of Kleis code!
- Algebraic hierarchy: Semigroup → Monoid → Group → Ring → Field
- 12 structures, 8 implementations, 47 operations
- Implementations: Field(ℝ), Field(ℂ), VectorSpace(Vector)
- Operations: dot, cross, norm, det, trace, ∂, ∫, ∇
- Constants: π, e, i, φ

Self-hosting: Type system defined in Kleis itself
Loaded at server startup to bootstrap type context"
```

### Commit 3: Grammar v0.3
```bash
git add docs/grammar/Kleis_v03.g4
git add docs/grammar/kleis_grammar_v03.ebnf
git add docs/grammar/kleis_grammar_v03.md
git commit -m "Grammar v0.3: Formal specification for type system

- ANTLR4 grammar (300 lines) - executable parser spec
- EBNF grammar (250 lines) - ISO standard notation
- Prose documentation (400 lines) - with examples

New constructs:
- structure/extends/implements keywords
- Polymorphic types with ∀
- Axiom declarations
- Operation manifests (supports block)
- Type annotations with :

All stdlib/prelude.kleis syntax now formally specified
Ready for parser implementation"
```

### Commit 4: ADR and Documentation
```bash
git add docs/adr-014-hindley-milner-type-system.md
git add docs/type-system/CONTEXT_AND_OPERATIONS.md
git add docs/type-system/HASKELL_INTEGRATION.md
git add docs/type-system/TYPE_CHECKING_UX.md
git add docs/type-system/SYNTAX_COMPARISON_AND_PROPOSAL.md
git add docs/type-system/TYPE_INFERENCE_POC.md
git add docs/type-system/examples/context_bootstrap_demo.md
git add docs/KLEIS_OVERVIEW.md
git add docs/KLEIS_OVERVIEW.pdf
git add SESSION_SUMMARY_2024-12-05_TYPE_SYSTEM.md
git commit -m "Type System Design Documentation

ADR-014: Adopt Hindley-Milner type system for Kleis
- Type inference for symbolic math (no evaluation needed)
- Algebraic structures (type classes → structures)
- 5-state incremental checking (Error/Incomplete/Polymorphic/Concrete/Unknown)
- Self-hosting via stdlib/prelude.kleis
- Universal: works for math, business, physics

Documentation:
- CONTEXT_AND_OPERATIONS: Bootstrap strategy + operation registry
- HASKELL_INTEGRATION: Why Haskell's approach works
- TYPE_CHECKING_UX: 5-state design with user-defined types
- SYNTAX_COMPARISON: Design choices
- TYPE_INFERENCE_POC: Current implementation status
- KLEIS_OVERVIEW: Comprehensive PDF for NotebookLM
- Session summary: Complete overview of today's work

Rationale: Type checking verifies STRUCTURE not VALUES
Therefore: Haskell's type system perfect for symbolic math"
```

---

## Alternative: Single Comprehensive Commit

```bash
git add -A
git commit -m "Type System: Hindley-Milner inference + Self-hosting stdlib

IMPLEMENTATION:
- src/type_inference.rs: Working POC (445 lines)
- Type inference, unification, constraint solving
- Demo: 9 test cases passing

STANDARD LIBRARY:
- stdlib/prelude.kleis: Self-hosting! (500 lines of Kleis)
- Algebraic hierarchy: Monoid → Group → Ring → Field
- 12 structures, 8 implementations, 47 operations
- Loaded at startup to bootstrap type context

GRAMMAR:
- Kleis v0.3: Formal ANTLR4 + EBNF specifications
- All stdlib syntax formally defined
- structure, implements, axiom, ∀, supports keywords
- Ready for parser implementation

DOCUMENTATION:
- ADR-014: Architectural decision
- Type system design docs (8 focused documents)
- KLEIS_OVERVIEW.pdf for NotebookLM
- Session summary

KEY INSIGHT: Type checking works on structure not values
RESULT: Haskell's type system perfect for symbolic math

Next: Parse grammar v0.3 and load stdlib"
```

---

## Recommendation

### Strategy: Single comprehensive commit

**Why:**
- Logically cohesive work (one feature)
- Everything needed to continue is included
- Documentation explains the design
- Clear milestone

**What to commit:**
- ✅ All implementation (type_inference.rs, demo)
- ✅ All stdlib (prelude.kleis)
- ✅ All grammars (v0.3)
- ✅ ADR-014
- ✅ Core type system docs (8 docs)
- ✅ Overview PDF
- ✅ Session summary

**What NOT to commit:**
- ❌ Planning docs (already archived)
- ❌ Temporary consolidation plans (archived)

---

## Command

```bash
cd /Users/eatik_1/Documents/git/cee/kleis

# Add all new/modified files
git add src/type_inference.rs
git add src/lib.rs
git add examples/type_inference_demo.rs
git add stdlib/
git add docs/grammar/Kleis_v03.g4
git add docs/grammar/kleis_grammar_v03.*
git add docs/adr-014-hindley-milner-type-system.md
git add docs/type-system/
git add docs/KLEIS_OVERVIEW.md
git add docs/KLEIS_OVERVIEW.pdf
git add SESSION_SUMMARY_2024-12-05_TYPE_SYSTEM.md
git add docs/archive/session-2024-12-05/

# Commit
git commit -m "Type System: Hindley-Milner inference + Self-hosting stdlib

[Message from above]
"
```

---

**Recommendation:** Commit everything - it's all needed to continue!

