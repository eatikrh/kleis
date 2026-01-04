# ADR-028: Dimension Expressions

**Status:** Accepted  
**Date:** 2024 (Grammar v0.92)

## Context

Grammar v0.92 introduces **type-level dimension expressions** for expressing 
relationships between matrix dimensions:

```kleis
operation realify : ComplexMatrix(n, n) → Matrix(2*n, 2*n, ℝ)
operation augment : Matrix(m, n, ℝ) → Vector(m) → Matrix(m, n+1, ℝ)
operation split   : Matrix(2*n, 2*n, ℝ) → (Matrix(n, n, ℝ), Matrix(n, n, ℝ), ...)
```

## Two Systems: Types vs Values

### Type-Level Dimensions (Symbolic)

```kleis
structure Realification(n: Nat) {
    operation realify : ComplexMatrix(n, n) → Matrix(2*n, 2*n, ℝ)
}
```

- **Parsed as**: `DimExpr::Mul(Lit(2), Var("n"))`
- **Purpose**: Documentation, type signatures, expressing relationships
- **Evaluation**: NOT evaluated — kept symbolic

### Value-Level Dimensions (Concrete)

```rust
// In evaluator.rs
if let Some((n, m, elements)) = self.extract_matrix(&args[0]) {
    let n2_size = 2 * n;  // Concrete Rust arithmetic!
    // Build actual 2n×2n matrix...
}
```

- **Source**: Extracted from actual matrix data at runtime
- **Purpose**: LAPACK calls, memory allocation, bounds checking
- **Evaluation**: Always concrete usize values

## Why Symbolic Dimensions Are Sufficient (For Now)

### 1. Type Inference Doesn't Need Concrete Values

The Hindley-Milner type inference algorithm works with **unification**, not evaluation:

```
realify : ComplexMatrix(n, n) → Matrix(2*n, 2*n, ℝ)
M : ComplexMatrix(3, 3)
realify(M) : Matrix(2*n, 2*n, ℝ)[n := 3] = Matrix(2*3, 2*3, ℝ)
```

The result type is `Matrix(2*3, 2*3, ℝ)` — still symbolic! We substitute but 
don't evaluate. This is fine because:

- We know the **structure** of the result type
- We know the **relationship** between input and output dimensions
- We can propagate this through the type system

### 2. Runtime Has Concrete Values

When `realify` actually executes:

```rust
fn apply_realify(&self, args: &[Expression]) -> Result<Expression, String> {
    // Extract CONCRETE dimensions from the actual matrix
    let (a_expr, b_expr) = self.extract_complex_matrix(&args[0])?;
    let (n, _, _) = self.extract_matrix(&a_expr)?;  // n is concrete!
    
    // Build 2n×2n result using plain Rust arithmetic
    let n2_size = 2 * n;  // 2 * 3 = 6
    // ...
}
```

The evaluator never sees `DimExpr` — it extracts dimensions from actual data.

### 3. Type Signatures Are Documentation

Currently, the type signatures serve as:
- **Documentation** for humans reading the code
- **Contracts** describing dimensional relationships
- **Future-proofing** for static type checking

## When Would We Need DimExpr Evaluation?

### Scenario 1: Static Dimension Checking

If we want to catch dimension errors at type-check time (before runtime):

```kleis
let M : Matrix(3, 4, ℝ) = ...
let N : Matrix(5, 6, ℝ) = ...
let R = M * N  // ERROR: 4 ≠ 5 (should fail at type-check time!)
```

To detect this, we'd need to:
1. Evaluate `4` and `5` to concrete values
2. Check that they're equal
3. Report error if not

### Scenario 2: Dimension Inference

If a dimension is constrained but not explicit:

```kleis
let M : Matrix(n, n, ℝ) = eye(?)  // What should ? be?
let R = realify(M)                 // R : Matrix(2*n, 2*n, ℝ)
assert(R.rows == 6)                // Implies n = 3, so ? = 3
```

Solving for `n` requires evaluating `2*n = 6` → `n = 3`.

### Scenario 3: Compile-Time Code Generation

If generating specialized code for specific dimensions:

```kleis
// User writes
operation fast_mult : Matrix(4, 4, ℝ) → Matrix(4, 4, ℝ) → Matrix(4, 4, ℝ)

// Compiler generates SIMD-optimized 4×4 multiplication
```

Knowing `4` at compile time enables optimizations.

### Scenario 4: Division and Subtraction Safety

```kleis
operation halve : Matrix(2*n, 2*n, ℝ) → Matrix(n, n, ℝ)
```

If someone calls `halve(M)` where `M : Matrix(5, 5, ℝ)`:
- Need to check: does `5 = 2*n` for some natural `n`?
- Answer: No! Should be a type error.

This requires solving arithmetic constraints.

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Parsing `2*n`, `n+1`, etc. | ✅ Done | Grammar v0.92 |
| Pretty-printing DimExpr | ✅ Done | Shows in type signatures |
| Substitution in DimExpr | ✅ Done | `n := 3` in type context |
| Symbolic type propagation | ✅ Done | Types carry DimExpr through |
| DimExpr evaluation | ❌ Not done | Would evaluate `2*3` → `6` |
| Dimension unification | ❌ Not done | Would solve `2*n = 6` → `n=3` |
| Static dimension checking | ❌ Not done | Would catch errors at type-check |
| Division/subtraction safety | ❌ Not done | Would reject invalid dimensions |

## Current Recommendation

For now, the symbolic approach is sufficient because:

1. **Runtime always has concrete values** — LAPACK calls work correctly
2. **Type signatures document intent** — humans understand `Matrix(2*n, 2*n, ℝ)`
3. **No false negatives** — we don't reject valid programs
4. **Implementation is simple** — no constraint solving needed

When to add evaluation:
- When static type checking becomes a priority
- When users request compile-time dimension validation
- When generating dimension-specialized code

## Potential Issues

### Division by Zero
```kleis
operation dangerous : Matrix(m, n/m, ℝ) → ...  // What if m = 0?
```

If we evaluate `n/m` where `m = 0`, we'd crash. Options:
- Keep symbolic (current) — no crash, but no checking
- Evaluate with error handling — `Err(DimError::DivisionByZero)`
- Require proof that `m ≠ 0` (dependent types, advanced)

### Negative Dimensions
```kleis
operation shrink : Matrix(n, n, ℝ) → Matrix(n-1, n-1, ℝ)  // What if n = 0?
```

If `n = 0`, then `n - 1` would be negative (invalid dimension).

### Non-Integer Results
```kleis
operation half : Matrix(n, n/2, ℝ) → ...  // What if n is odd?
```

If `n = 5`, then `n/2 = 2.5` (not a valid dimension).

## Summary

- **Symbolic dimensions** (current): Safe, simple, sufficient for documentation
- **Evaluated dimensions** (future): Needed for static checking and optimization
- **The gap**: Runtime evaluation happens separately from type-level DimExpr

The two systems are currently independent — and that's fine for a working system.
When static type checking becomes important, we can bridge them.

