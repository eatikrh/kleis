# ADR-026: Self-Hosted Differential Forms

**Status:** Accepted  
**Date:** 2024-12-30  
**Deciders:** Kleis Architecture Team

## Context

Kleis needs to support exterior algebras and Cartan calculus for CERN-style physics:
- Wedge product (∧)
- Exterior derivative (d)
- Hodge star (⋆)
- Interior product (ι_X)
- Lie derivative (ℒ_X)

The initial approach was to implement these as Rust builtins (`builtin_wedge_product`, 
`builtin_exterior_derivative`, etc.). This would require:
- 12+ new Rust functions in `evaluator.rs`
- Possibly a new Rust crate for permutations
- Users cannot read or modify the implementations
- Cannot verify builtin implementations with Z3

## Decision

**Implement all differential form operations in pure Kleis.**

The only Rust primitives needed are those already existing:
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `<`, `>`, `=`
- List primitives: `Cons`, `Nil`, pattern matching
- Control flow: `if`/`then`/`else`, `match`, `λ`
- Recursion: function self-reference

Everything else is built in Kleis:

```
Layer 4: Differential Geometry (pure Kleis)
  wedge, d, ⋆, ι_X, ℒ_X
           ↓
Layer 3: Tensor Algebra (pure Kleis)
  antisymmetrize, permute_indices, tensor_product
           ↓
Layer 2: Combinatorics (pure Kleis)
  all_permutations, perm_sign, factorial
           ↓
Layer 1: Functional Core (pure Kleis)
  fold, map, range, filter, length, append
           ↓
Layer 0: Primitives (Rust - invisible)
  +, -, *, /, Cons, Nil, match, if, λ
```

### Key Implementation Details

**Tensors as Nested Lists:**
```kleis
// Rank-0: scalar
// Rank-1: [a, b, c]
// Rank-2: [[a, b], [c, d]]
// Rank-3: [[[...]]]

define tensor_get(T, indices) = match indices {
    Nil => T
    | Cons(i, rest) => tensor_get(list_get(T, i), rest)
}
```

**Permutations in Pure Kleis:**
```kleis
define perm_sign(p) = power(negate(1), count_inversions(p))

define all_permutations(n) = 
    if n = 0 then Cons(Nil, Nil)
    else flat_map(λ p . insert_all_positions(n - 1, p), 
                  all_permutations(n - 1))
```

**Antisymmetrization:**
```kleis
define antisymmetrize(T) = 
    let n = rank(T) in
    let perms = all_permutations(n) in
    let terms = map(λ σ . scale(perm_sign(σ), permute_indices(T, σ)), perms) in
    scale(1 / factorial(n), sum_tensors(terms))
```

**Wedge Product:**
```kleis
define wedge(α, β) = antisymmetrize(tensor_product(α, β))
```

**Cartan's Formula (the axiom IS the implementation):**
```kleis
define lie(X, α) = plus(d(interior(X, α)), interior(X, d(α)))
```

## Consequences

### Positive

1. **Full Self-Hosting:** Kleis defines Kleis. Users can read all implementations.

2. **Verifiable:** Z3 can verify axioms about Kleis-defined operations.

3. **Extensible:** Users can add Clifford algebras, spinors, etc. without Rust.

4. **Educational:** The stdlib IS the documentation.

5. **No New Dependencies:** No `permutations` crate or other Rust dependencies needed.

6. **Block Matrix Precedent:** We already support block matrices (nested structures), 
   proving the architecture works.

### Negative

1. **Performance:** Pure Kleis recursion slower than native Rust for large computations.
   Mitigated by: physics typically uses dim ≤ 4, and numerical Kleis can optimize hot paths.

2. **Point-Free Limitation:** `define f = λ x . x` doesn't work; must use `define f(x) = x`.
   Minor cosmetic issue.

### Neutral

1. **Rust Layer Invisible:** Builtins still exist for arithmetic, but users never see them.
   This is the correct abstraction - the math lives in Kleis, the runtime is invisible.

## Implementation Plan

1. **Create `stdlib/prelude.kleis`:**
   - `fold`, `map`, `filter`, `range`, `length`, `append`
   - `flat_map`, `concat_lists`

2. **Create `stdlib/combinatorics.kleis`:**
   - `factorial`, `binomial`
   - `all_permutations`, `perm_sign`, `perm_apply`

3. **Create `stdlib/tensors_functional.kleis`:**
   - `tensor_get`, `tensor_set`, `tensor_map`
   - `tensor_product`, `permute_indices`
   - `antisymmetrize`, `symmetrize`

4. **Update `stdlib/differential_forms.kleis`:**
   - Replace `builtin_*` with Kleis definitions
   - Axioms become implementations (Cartan formula!)

## Validation

Each layer will have `example` blocks that verify:
```kleis
example "wedge antisymmetric" {
    wedge(α, β) = negate(wedge(β, α))
}

example "d squared zero" {
    d(d(f)) = zero
}

example "cartan formula" {
    lie(X, α) = plus(d(interior(X, α)), interior(X, d(α)))
}
```

## References

- ADR-003: Self-Hosting Strategy
- ADR-015: Text as Source of Truth
- ADR-016: Operations in Structures
- Block matrices in `stdlib/matrices.kleis`

