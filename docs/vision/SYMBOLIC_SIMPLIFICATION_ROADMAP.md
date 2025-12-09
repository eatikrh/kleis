# Symbolic Simplification Roadmap

**Date:** December 9, 2024  
**Phase:** 5+ (Post Self-Hosting)  
**Status:** Future enhancement after self-hosting stabilizes

---

## Overview

Kleis should simplify symbolic expressions using algebraic laws while preserving type information.

**Goal:** Transform `1 * a → a` and `a / a → 1` while maintaining type safety.

---

## Current State (Phase 3)

**Kleis preserves structure:**
```kleis
1 * a  →  Operation { name: "times", args: [Const("1"), Object("a")] }
// Stays as-is, no simplification
```

**Why this is OK for now:**
- ✅ Preserves mathematical structure
- ✅ Type information intact
- ✅ User sees what they wrote
- ✅ Can pass to external solvers

**Why we need simplification eventually:**
- ❌ Expressions get bloated: `((x + 0) * 1) + (0 * y)`
- ❌ Hard to read results
- ❌ Inefficient for large expressions
- ❌ Misses mathematical equivalences

---

## Phase 5: Symbolic Simplification

### Algebraic Simplification Rules

**Identity laws:**
```kleis
1 * a → a           // Multiplicative identity
a * 1 → a
0 + a → a           // Additive identity
a + 0 → a
```

**Annihilation:**
```kleis
0 * a → 0
a * 0 → 0
```

**Self-operations:**
```kleis
a - a → 0           // Self-subtraction
a / a → 1           // Self-division (where a ≠ 0)
a^0 → 1             // Zero exponent
a^1 → a             // Unit exponent
```

**Associativity:**
```kleis
(a + b) + c ≡ a + (b + c)
(a * b) * c ≡ a * (b * c)
```

**Commutativity:**
```kleis
a + b ≡ b + a
a * b ≡ b * a
```

**Distributivity:**
```kleis
a * (b + c) → a*b + a*c
(a + b) * c → a*c + b*c
```

**Constant folding:**
```kleis
2 + 3 → 5
4 * 5 → 20
// But only for pure constants, not symbolic
```

---

## Implementation Approaches

### Approach 1: Pattern-Based Simplification (Easiest)

**Define simplify function in Kleis:**
```kleis
define simplify(e: Expr) : Expr = match e {
  // Identity
  Multiply(Const("1"), x) => simplify(x)
  Multiply(x, Const("1")) => simplify(x)
  Plus(x, Const("0")) => simplify(x)
  Plus(Const("0"), x) => simplify(x)
  
  // Annihilation
  Multiply(Const("0"), _) => Const("0")
  Multiply(_, Const("0")) => Const("0")
  
  // Self-operations
  Minus(x, y) if x == y => Const("0")
  Divide(x, y) if x == y => Const("1")
  
  // Recursive
  Plus(a, b) => Plus(simplify(a), simplify(b))
  Multiply(a, b) => Multiply(simplify(a), simplify(b))
  
  // Base case
  _ => e
}
```

**Pros:**
- ✅ Simple to implement
- ✅ Written in Kleis (self-hosting!)
- ✅ Easy to extend
- ✅ Type-safe by construction

**Cons:**
- ⚠️ Order matters (must try all rules)
- ⚠️ May miss opportunities (limited look-ahead)
- ⚠️ No guarantee of termination

---

### Approach 2: E-Unification with AC Theory (Powerful)

**Use equational unification for algebraic properties:**

```kleis
// Declare algebraic laws
structure CommutativeOperation(T) {
    operation (#) : T → T → T
    axiom commutativity: ∀(a b : T). a # b = b # a
}

structure AssociativeOperation(T) {
    operation (#) : T → T → T
    axiom associativity: ∀(a b c : T). (a # b) # c = a # (b # c)
}

// E-unification automatically respects these!
```

**Pros:**
- ✅ Theoretically sound
- ✅ Handles commutativity/associativity automatically
- ✅ Complete w.r.t. equational theory
- ✅ Can prove equivalences

**Cons:**
- ⚠️ Complex to implement (see UnifChapter.pdf)
- ⚠️ Can be undecidable for some theories
- ⚠️ Performance overhead
- ⚠️ Requires careful theory design

**Mentioned in:** `docs/type-system/UNIFICATION_IMPLEMENTATION.md`

---

### Approach 3: Rewrite System (Middle Ground)

**Define rewrite rules with priorities:**

```kleis
// Rewrite rule syntax (hypothetical)
rewrite identity_multiply {
  pattern: 1 * x
  result: x
  priority: high
}

rewrite identity_add {
  pattern: x + 0
  result: x
  priority: high
}

rewrite annihilation {
  pattern: 0 * x
  result: 0
  priority: high
}
```

**Pros:**
- ✅ Declarative (easy to understand)
- ✅ Controllable (set priorities)
- ✅ Extensible (add new rules)
- ✅ Can ensure termination (careful ordering)

**Cons:**
- ⚠️ Need new syntax (rewrite rules)
- ⚠️ Confluence not guaranteed
- ⚠️ May need strategy language

---

## Recommended Approach for Kleis

### Start Simple, Grow Sophisticated

**Phase 5a: Pattern-Based Simplification** (Approach 1)
- Implement `simplify(e)` function in Kleis
- Cover basic algebraic laws
- Self-hosting demonstration
- **Effort:** 2-3 hours

**Phase 5b: Rewrite System** (Approach 3)
- Add rewrite rule syntax
- Priority-based application
- User-definable rules
- **Effort:** 1-2 weeks

**Phase 6: E-Unification** (Approach 2)
- Full AC theory support
- Automatic commutativity/associativity
- Proof capabilities
- **Effort:** 1-2 months

---

## Example: Simplifying Einstein Equation

**Input:**
```kleis
((1 * G_μν) + (0 * R_μν)) + Λg_μν = κT_μν
```

**After simplification:**
```kleis
G_μν + Λg_μν = κT_μν
```

**Type preserved:**
```
Before: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ)
After:  Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ)
// Type information intact! ✓
```

---

## Why This Matters

### External solvers can't do this:

```python
# Python/NumPy
result = 1 * G + 0 * R + Lambda * g
# Executes, but:
# - Doesn't simplify 1 * G to G
# - Doesn't remove 0 * R
# - No type checking (could add velocity to mass!)
```

### Kleis would:

1. **Type-check:** Ensure all terms are Tensor(0, 2, 4, ℝ)
2. **Simplify:** Remove identity/annihilation operations
3. **Validate:** Confirm equation is well-typed
4. **Export:** Send simplified, validated expression to solver

**Kleis = Smart pre-processor that catches errors and simplifies before numerical computation**

---

## Connection to Your Insight

> "Number systems are just conventions - tallies work too"

**Kleis's insight:**
> "Decimal is just ONE representation - symbolic expressions are ANOTHER representation"

**Why preserve symbolic form:**
- Contains more information than decimals
- Preserves mathematical structure
- Enables algebraic reasoning
- Type-safe transformations

**Then simplify symbolically:**
- Apply algebraic laws
- Reduce complexity
- Keep type information
- **Still** symbolic (not numeric)

**Finally, if needed:**
- Export to numerical solver
- Get decimal approximation
- But Kleis has already validated correctness!

---

## Roadmap

### Phase 5: Symbolic Simplification (After self-hosting stable)

**Milestone 5a: Basic Simplification** (2-3 hours)
```kleis
define simplify(e: Expr) : Expr = match e { ... }
```

**Milestone 5b: Extended Rules** (1-2 days)
- All identity laws
- All annihilation laws
- Constant folding
- Nested simplification

**Milestone 5c: Rewrite System** (1-2 weeks)
- Declarative rewrite rules
- User-extensible
- Priority-based application

**Milestone 6: E-Unification** (1-2 months)
- AC theory support
- Automatic commutativity
- Proof capabilities

---

## Dependencies

**Requires:**
- ✅ Self-hosting (complete!)
- ✅ Pattern matching (complete!)
- ✅ ADT unification (complete!)
- 🔜 Expression equality checking
- 🔜 Conditional guards in patterns (if needed)

**Enables:**
- 🚀 Cleaner equation display
- 🚀 Automatic simplification
- 🚀 Algebraic reasoning
- 🚀 Symbolic proof assistant

---

## Summary

**Your observation is profound:**

Traditional view: "Kleis doesn't compute because it doesn't reduce to decimals"

**Correct view:** "Kleis DOES compute, using symbolic representation + algebraic laws"

**Adding simplification** makes this explicit:
- Computation via term rewriting
- Guided by type information
- Preserving mathematical structure
- **This is what Computer Algebra Systems do!**

**Kleis = CAS + Strong Type System**

And yes, `1 * a → a` should absolutely be automatic in the future! 🎯

---

**Next step:** Add to `NEXT_PRIORITIES.md` as Phase 5 milestone?

