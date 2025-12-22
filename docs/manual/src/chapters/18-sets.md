# Set Theory

Kleis provides native set theory support backed by Z3's set theory solver. This enables rigorous mathematical reasoning about collections, membership, and set operations.

## Importing Set Theory

```kleis
import "stdlib/sets.kleis"
```

This imports the `SetTheory(T)` structure with all operations and axioms.

## Basic Operations

### Set Membership

```kleis
// Check if element x is in set S
in_set(x, S)  // Returns Bool

// Example: Define a membership property
structure ClosedUnderAddition {
    axiom closed: ∀(S : Set(ℤ), x y : ℤ). 
        in_set(x, S) ∧ in_set(y, S) → in_set(x + y, S)
}
```

### Set Construction

```kleis
empty_set        // The empty set ∅
singleton(x)     // Set containing just x: {x}
insert(x, S)     // Add x to S: S ∪ {x}
remove(x, S)     // Remove x from S: S \ {x}
```

### Set Operations

```kleis
union(A, B)       // Union: A ∪ B
intersect(A, B)   // Intersection: A ∩ B
difference(A, B)  // Difference: A \ B
complement(A)     // Complement: ᶜA
```

### Set Relations

```kleis
subset(A, B)         // Subset: A ⊆ B
proper_subset(A, B)  // Proper subset: A ⊂ B (A ⊆ B and A ≠ B)
```

## Verification Example

Here's a complete example proving De Morgan's laws:

```kleis
import "stdlib/sets.kleis"

// Verify De Morgan's law: complement(A ∪ B) = complement(A) ∩ complement(B)
structure DeMorganProof(T) {
    axiom de_morgan_union: ∀(A B : Set(T)).
        complement(union(A, B)) = intersect(complement(A), complement(B))
}
```

In the REPL:

```
λ> :load stdlib/sets.kleis
✅ Loaded stdlib/sets.kleis

λ> :verify ∀(A B : Set(ℤ), x : ℤ). in_set(x, complement(union(A, B))) ↔ (¬in_set(x, A) ∧ ¬in_set(x, B))
✅ Valid (follows from axioms)
```

## Mathematical Structures with Sets

### Open Balls in Metric Spaces

```kleis
import "stdlib/sets.kleis"

structure MetricSpace(X) {
    operation d : X → X → ℝ
    
    // Metric axioms
    axiom positive: ∀(x y : X). d(x, y) >= 0
    axiom zero_iff_equal: ∀(x y : X). d(x, y) = 0 ↔ x = y
    axiom symmetric: ∀(x y : X). d(x, y) = d(y, x)
    axiom triangle: ∀(x y z : X). d(x, z) <= d(x, y) + d(y, z)
}

structure OpenBalls(X) {
    operation d : X → X → ℝ
    operation ball : X → ℝ → Set(X)
    
    // x is in ball(center, r) iff d(x, center) < r
    axiom ball_def: ∀(center : X, r : ℝ, x : X).
        in_set(x, ball(center, r)) ↔ d(x, center) < r
    
    // Open balls are non-empty when radius is positive
    axiom ball_nonempty: ∀(center : X, r : ℝ).
        r > 0 → in_set(center, ball(center, r))
}
```

### Measure Spaces

```kleis
import "stdlib/sets.kleis"

structure MeasureSpace(X) {
    element sigma_algebra : Set(Set(X))
    operation measure : Set(X) → ℝ
    
    // σ-algebra contains empty set
    axiom contains_empty: in_set(empty_set, sigma_algebra)
    
    // Closed under complement
    axiom closed_complement: ∀(A : Set(X)).
        in_set(A, sigma_algebra) → in_set(complement(A), sigma_algebra)
    
    // Measure is non-negative
    axiom measure_positive: ∀(A : Set(X)). measure(A) >= 0
    
    // Measure of empty set is zero
    axiom measure_empty: measure(empty_set) = 0
}
```

## Full Axiom Reference

The `SetTheory(T)` structure includes these axioms:

| Axiom | Statement |
|-------|-----------|
| Extensionality | `∀(A B). (∀x. x ∈ A ↔ x ∈ B) → A = B` |
| Empty Set | `∀x. ¬(x ∈ ∅)` |
| Singleton | `∀x y. y ∈ {x} ↔ y = x` |
| Union | `∀(A B) x. x ∈ A∪B ↔ (x ∈ A ∨ x ∈ B)` |
| Intersection | `∀(A B) x. x ∈ A∩B ↔ (x ∈ A ∧ x ∈ B)` |
| Difference | `∀(A B) x. x ∈ A\B ↔ (x ∈ A ∧ ¬(x ∈ B))` |
| Complement | `∀A x. x ∈ Aᶜ ↔ ¬(x ∈ A)` |
| Subset | `∀(A B). A ⊆ B ↔ (∀x. x ∈ A → x ∈ B)` |
| De Morgan (Union) | `∀(A B). (A∪B)ᶜ = Aᶜ ∩ Bᶜ` |
| De Morgan (Intersection) | `∀(A B). (A∩B)ᶜ = Aᶜ ∪ Bᶜ` |
| Double Complement | `∀A. (Aᶜ)ᶜ = A` |

## Unicode Operators (Future)

Currently, you must use function-style syntax. Future versions will support:

| Unicode | Function Style |
|---------|----------------|
| `x ∈ S` | `in_set(x, S)` |
| `A ⊆ B` | `subset(A, B)` |
| `A ∪ B` | `union(A, B)` |
| `A ∩ B` | `intersect(A, B)` |
| `A \ B` | `difference(A, B)` |

## See Also

- [Types and Values](./02-types.md) — `Set(T)` type documentation
- [Z3 Verification](./11-z3-verification.md) — Using Z3 for proofs
- [Structures](./09-structures.md) — Defining mathematical structures

