# Unification Implementation in Kleis

**Date:** December 9, 2024  
**Context:** Self-hosting implementation (Wire 2 fix)  
**Reference:** `docs/type-system/UnifChapter.pdf`

---

## Overview

This document explains how Kleis implements type unification for algebraic data types (ADTs) and how it relates to formal unification theory.

---

## Our Implementation: Type-Level Unification

### The Rule

When unifying two ADT constructor types, we check:

1. ✅ **Same parent ADT** (`type_name` must match)
2. ✅ **Same number of type arguments** (`args.len()` must match)
3. ✅ **All type arguments recursively unify** (structural recursion)
4. ❌ **Constructor names are ignored** (True ≠ False is OK)

### Code

```rust
// In src/type_inference.rs, unify() for Data types:
(Type::Data { type_name: t1, constructor: c1, args: a1 },
 Type::Data { type_name: t2, constructor: c2, args: a2 }) => {
    
    // Must have same parent ADT type
    if t1 != t2 {
        return Err(format!("Cannot unify different types: {} vs {}", t1, t2));
    }
    
    // Constructor names ignored! (True vs False is OK)
    
    // Must have same number of type arguments
    if a1.len() != a2.len() {
        return Err(...);
    }
    
    // Recursively unify all type arguments
    for (arg1, arg2) in a1.iter().zip(a2.iter()) {
        self.unify(arg1.clone(), arg2.clone())?;
    }
    
    Ok(())
}
```

---

## Why This Is NOT Standard Syntactic Unification

### Standard Syntactic Unification (First-Order Terms)

**From UnifChapter.pdf:**

> **Symbol Clash Rule**: If `s = f(...)` and `t = g(...)` and `f ≠ g`, then Exit with failure

Standard syntactic unification requires:
- **Identical function symbols** (constructors must match exactly)
- `True` and `False` would be different symbols → **Symbol Clash** → **Failure**

### Our Approach: Semantic/Type-Level Unification

We **ignore constructor names** and only check:
- Same **type** (parent ADT)
- Compatible **type arguments**

This is **NOT** syntactic unification of first-order terms!

---

## Why Our Approach Is Correct for Kleis

### Distinction: Types vs Terms

**Kleis performs TYPE CHECKING, not TERM REWRITING**

```kleis
data Bool = True | False

match b { True => False | False => True }
```

**Type level** (what we care about):
- `True : Bool`
- `False : Bool`
- Both branches return `Bool` → **Type checks!** ✅

**Term level** (what we DON'T care about in type checking):
- `True ≠ False` as values
- Different constructors → would fail symbol clash
- **Irrelevant for type checking!**

### How Haskell/ML Handle This

From the research summary provided:

> **"All constructors of a single ADT produce values of that same single type during unification."**

Haskell doesn't care that `True ≠ False` when type-checking. It only cares that both are `Bool`.

```haskell
-- Haskell
case b of
  True  -> False  -- Type: Bool
  False -> True   -- Type: Bool
-- Whole expression: Bool ✓

-- Type checker unifies the branch TYPES, not the branch VALUES
```

**Our implementation matches this semantic approach!** ✅

---

## Examples: What Works and Why

### ✅ Case 1: Bool Enumeration

```kleis
data Bool = True | False

define not(b) = match b { True => False | False => True }
```

**Unification:**
```rust
True  → Data{Bool, True,  []}
False → Data{Bool, False, []}

Check:
- type_name: Bool = Bool ✅
- args: [] = [] ✅
- constructor: True ≠ False (IGNORED)
→ UNIFY to Bool ✅
```

### ✅ Case 2: Matrix Dimensions

```kleis
data Type = Scalar | Matrix(m: Nat, n: Nat, T)

Matrix(2, 3, ℝ) vs Matrix(3, 2, ℝ)
```

**Unification:**
```rust
Data{Type, Matrix, [NatValue(2), NatValue(3), Named(ℝ)]}
Data{Type, Matrix, [NatValue(3), NatValue(2), Named(ℝ)]}

Check:
- type_name: Type = Type ✅
- args count: 3 = 3 ✅
- args unify:
  - NatValue(2) vs NatValue(3) → FAIL ❌
→ CANNOT UNIFY (different dimensions) ✅
```

### ✅ Case 3: Option with Type Parameters

```kleis
data Option(T) = None | Some(value: T)

None vs Some(5)
```

**Unification:**
```rust
None    → Data{Option, None, [Var(α)]}     // Fresh var for T
Some(5) → Data{Option, Some, [infer(5)]}   // T = infer(5)

Check:
- type_name: Option = Option ✅
- args count: 1 = 1 ✅
- args unify: Var(α) vs infer(5) → α := infer(5) ✅
→ UNIFY to Option(infer(5)) ✅
```

### ✅ Case 4: Same Matrix Dimensions

```kleis
Matrix(2, 3, ℝ) vs Matrix(2, 3, ℝ)
```

**Unification:**
```rust
Check:
- type_name: Type = Type ✅
- args count: 3 = 3 ✅
- args unify:
  - NatValue(2) vs NatValue(2) ✅
  - NatValue(3) vs NatValue(3) ✅
  - Named(ℝ) vs Named(ℝ) ✅
→ UNIFY to Matrix(2, 3, ℝ) ✅
```

---

## Additional Fixes Required

### Fix 1: Type Variable Self-Unification

**Problem:**
```rust
Var(α) unifying with Var(α) was failing occurs check
```

**From UnifChapter.pdf:**

> **Trivial Rule**: `{s = s} ∪ P' ; S ⟹ P' ; S`

Reflexive unification should succeed trivially.

**Our Fix:**
```rust
(Type::Var(v1), Type::Var(v2)) if v1 == v2 => {
    Ok(Substitution::empty())  // Identity substitution
}
```

### Fix 2: Type Parameters for Nullary Constructors

**Problem:**
```kleis
data Option(T) = None | Some(value: T)
```

`None` has no fields, but `Option(T)` has type parameter `T`.

**Solution:**
```rust
if variant.fields.is_empty() && !data_def.type_params.is_empty() {
    // Create fresh type variables for each type parameter
    for _type_param in &data_def.type_params {
        constructor_args.push(self.context.fresh_var());
    }
}
```

**Result:**
```rust
None → Data{Option, None, [Var(fresh)]}  // Now has T!
```

---

## Theoretical Classification

### What We Implemented

**Name:** Type-Level Structural Unification (or Semantic ADT Unification)

**Characteristics:**
- Operates on **types** (what constructors return)
- Not on **terms** (constructor values themselves)
- Appropriate for **type checking**
- Not appropriate for **term rewriting** or **symbolic computation**

### Relation to Standard Algorithms

| Algorithm | Constructor Check | Use Case |
|-----------|------------------|----------|
| **Syntactic Unification** | Required (symbol clash) | Term rewriting, symbolic computation |
| **Type-Level Unification** (ours) | Ignored (type-based) | Type checking, type inference |
| **E-Unification** | Modulo equations | Algebraic theories (AC, ACU, AG) |

---

## Future Considerations

### 1. Equational Unification (E-Unification)

**From UnifChapter.pdf:**

> E-unification makes terms equivalent with respect to equational axioms E

**Potential applications in Kleis:**

```kleis
// Commutativity (C)
x + y  should unify with  y + x

// Associativity (A)
(a + b) + c  should unify with  a + (b + c)

// AC Theory (Associative-Commutative)
// For operations like:
operation (+) : ℝ × ℝ → ℝ
  where { associative, commutative }
```

**When to implement:**
- Phase 4: Symbolic simplification
- Phase 5: Equation solving
- Use cases: Simplify expressions, prove equivalences

### 2. Matching (One-Way Unification)

**From UnifChapter.pdf:**

> Matching: find θ such that sθ = t (variables only in s)

**Can be reduced to:** Unification with constants

**Potential use cases in Kleis:**

```kleis
// Template matching
template: frac(□, □)
expression: frac(x, y)
→ Match! Bind placeholders

// Structure instance checking
structure Numeric(N) requires { (+), (*), abs }
type ℝ
→ Does ℝ match Numeric? (one-way check)

// Pattern matching in transforms
pattern: a * (b + c)
expression: x * (y + z)
→ Match! Apply distributivity
```

**Implementation approach:**
- Use existing unification
- Treat pattern variables as mutable
- Treat target term as constants (immutable)

---

## Relationship to Hindley-Milner

### Our Implementation Aligns with HM Type Inference

**Hindley-Milner (ADR-014) uses:**
- Constraint generation (what we do)
- Unification for constraint solving (what we do)
- Type-level reasoning (what we do)

**Our unification supports:**
- ✅ Type variables (`Var(α)`)
- ✅ Polymorphic types (`ForAll`)
- ✅ User-defined types (`Data`)
- ✅ Structural recursion (nested types)

**This is standard HM with algebraic data types!**

---

## Implementation Notes

### Where the Logic Lives

**File:** `src/type_inference.rs`

**Key functions:**
- `unify()` (line ~900) - Core unification algorithm
- `occurs()` (line ~968) - Occurs check for infinite types
- `infer_data_constructor()` (line ~711) - Type inference for constructors
- `infer_match()` (line ~366) - Pattern matching type inference

**Tests:**
- `test_unify_same_enum_constructors()` - Bool unification
- `test_unify_different_matrix_dimensions()` - Dimension checking

### Design Decisions

1. **Type-level over term-level** - Appropriate for type checking
2. **Constraint-based solving** - Follows HM algorithm
3. **Fresh vars for type params** - Handles nullary constructors
4. **Recursive arg unification** - Handles nested types

---

## Correctness Verification

### Test Coverage

**Unification tests:**
- ✅ Same enum constructors unify (True/False)
- ✅ Different dimensions fail (Matrix(2,3) ≠ Matrix(3,2))
- ✅ Reflexive unification succeeds (α = α)
- ✅ Type parameters work (None : Option(T))

**Integration tests:**
- ✅ Pattern matching on Bool works
- ✅ Functions with match expressions type-check
- ✅ Multiple function definitions
- ✅ Mixed with data types and structures

**Total: 413 tests passing** ✅

---

## Comparison with Theory

| Theoretical Concept | Our Implementation | Status |
|---------------------|-------------------|--------|
| **Syntactic Unification** | Not used (type-level instead) | N/A |
| **Occurs Check** | Implemented with reflexive fix | ✅ |
| **Constraint-Based** | Core approach (HM algorithm) | ✅ |
| **Type-Level Unification** | For ADT type checking | ✅ |
| **E-Unification** | Not yet (future: math equivalences) | 🔜 |
| **Matching (one-way)** | Not yet (reducible to unification) | 🔜 |

---

## Open Questions

### 1. Should We Support E-Unification?

**For mathematical equivalences:**
- Commutativity: `x + y ≡ y + x`
- Associativity: `(a + b) + c ≡ a + (b + c)`
- Distributivity: `a * (b + c) ≡ a*b + a*c`

**Use cases:**
- Symbolic simplification
- Equation solving
- Proving equivalences
- Pattern-based transformations

**Complexity:** High (E-unification is undecidable for some theories)

### 2. Do We Need One-Way Matching?

**Potential use cases:**
- Template matching (does expression fit template?)
- Type class instance checking (does type fit constraint?)
- Rewrite rules (does pattern match expression?)

**Implementation:** Can reduce to unification with constants (per UnifChapter.pdf)

### 3. Should We Support Recursive Types?

**Examples:**
```kleis
data List(T) = Nil | Cons(head: T, tail: List(T))
data Tree(T) = Leaf(T) | Node(Tree(T), Tree(T))
```

**Current status:** Parser supports, type system supports
**Occurs check:** Prevents infinite types (correct for finite terms)
**Question:** Do we need μ-types (infinite/recursive types)?

---

## Recommendations

### Immediate (Phase 3 - Complete)

✅ **Done:**
- Type-level ADT unification
- Occurs check with reflexive case
- Type parameters for nullary constructors
- Full HM inference with ADTs

### Short-term (Phase 4 - Next Quarter)

**When self-hosting is stable:**
1. Document edge cases (polymorphic constructors, higher-kinded types)
2. Add more unification tests (nested ADTs, mutually recursive types)
3. Performance optimization (unification is on critical path)

### Long-term (Phase 5+)

**When doing symbolic mathematics:**
1. **E-Unification for AC theories**
   - Commutative operations (addition, multiplication)
   - Associative operations (most binary ops)
   - Enables symbolic simplification

2. **One-way matching**
   - Pattern-based transformations
   - Rewrite systems
   - Symbolic manipulation

3. **Higher-order unification**
   - For meta-programming (defining transformations in Kleis)
   - Type-level computation
   - Dependent types (if needed)

---

## Connection to ADRs

### ADR-014: Hindley-Milner Type System

Our unification is the **core of HM inference**:
- Constraint generation → equations to solve
- Unification → solving those equations
- Substitution → finding the MGU (Most General Unifier)

### ADR-021: Algebraic Data Types

User-defined types require unification to:
- Check pattern matching exhaustiveness
- Infer types in match expressions
- Validate constructor applications

### ADR-016: Operations in Structures

Structure-based operations combine with unification:
- Check if type supports operation
- Infer operation result types
- Validate type constraints

---

## Theoretical Foundation

### From UnifChapter.pdf

**Key concepts we use:**

1. **Transformation-based unification** (Section 3)
   - Start with equations (constraints)
   - Apply transformation rules
   - Reach solved form (substitution)

2. **Occurs check** (Section 3.1)
   - Prevents infinite terms
   - Ensures well-formed substitutions
   - We added reflexive case (α = α)

3. **Constraint-based solving** (Chapter context)
   - Generate constraints during inference
   - Solve as a system
   - More flexible than immediate substitution

**What we DON'T use (yet):**

4. **E-Unification** (Section 7+)
   - Unification modulo equations
   - Required for algebraic properties
   - Future: symbolic simplification

5. **Matching** (Section 7)
   - One-way unification
   - Can reduce to unification with constants
   - Future: pattern-based transformations

---

## Implementation Quality

### Correctness

✅ **Follows HM semantics** - Types unify, not values  
✅ **Handles ADTs correctly** - Haskell-style semantics  
✅ **Occurs check** - Prevents infinite types  
✅ **Type parameters** - Nullary constructors get type params  
✅ **Recursive structures** - List, Tree, etc. work  

### Test Coverage

```
413 tests passing
- 2 specific unification tests (Bool, Matrix)
- 33 match expression tests
- 8 data constructor tests
- Full integration test suite
```

### Performance

**Current:** Naive recursive algorithm (fine for POC)  
**Future:** Consider almost-linear algorithm from UnifChapter.pdf for production

---

## Analogies

### For Understanding Our Approach

**Standard syntactic unification:**
> "Are these two LEGO structures identical brick-by-brick?"

**Our type-level unification:**
> "Do these two LEGO structures fit the same BLUEPRINT?"

Different brick colors (True vs False) don't matter if they fit the same blueprint (Bool).

**E-unification:**
> "Are these two LEGO structures equivalent if we can rearrange them according to rules?"

---

## References

1. **UnifChapter.pdf** - Formal unification theory foundation
2. **src/type_inference.rs** - Implementation code
3. **ADR-014** - Hindley-Milner Type System
4. **ADR-021** - Algebraic Data Types
5. **stdlib/types.kleis** - ADT definitions using this unification

---

## Summary

**What we implemented:**
- Type-level structural unification for ADTs
- Semantically correct for type checking
- Not standard syntactic unification (by design!)
- Aligns with Haskell/ML semantics

**Why it's correct:**
- Type checking operates on types, not values
- Constructor names are value-level distinctions
- All constructors of same ADT produce same type

**Future enhancements:**
- E-unification for mathematical equivalences
- One-way matching for pattern-based transforms
- Performance optimization for production use

**Result:**
🎉 **Self-hosting complete with correct ADT type checking!** 🎉

