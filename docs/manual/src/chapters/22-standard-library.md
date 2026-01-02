# Standard Library Types

## Self-Hosting: Kleis Defines Kleis

One of Kleis's most elegant features is **meta-circularity**: the type system is defined *in Kleis itself*.

```kleis
// From stdlib/types.kleis - loaded before anything else
data Type =
    Scalar
    | Vector(n: Nat, T)
    | Matrix(m: Nat, n: Nat, T)
    | Complex
    | Set(T: Type)
    | List(T: Type)
    | Tensor(dims: List(Nat))
```

This means:
- Types aren't hardcoded in the Rust compiler
- Users can extend the type system without recompiling
- The type checker can reason about types *as data*

## Core Types

### Bool

The fundamental boolean type:

```kleis
data Bool = True | False
```

**Operations:**

```kleis
define not(b) = match b {
    True => False
    | False => True
}

define and(b1, b2) = match b1 {
    False => False
    | True => b2
}

define or(b1, b2) = match b1 {
    True => True
    | False => b2
}
```

### Option(T)

For values that might not exist (like Haskell's `Maybe`):

```kleis
data Option(T) =
    None
    | Some(value: T)
```

**Operations:**

```kleis
define isSome(opt) = match opt {
    None => False
    | Some(_) => True
}

define isNone(opt) = match opt {
    None => True
    | Some(_) => False
}

define getOrDefault(opt, default) = match opt {
    None => default
    | Some(x) => x
}
```

**Usage:**

```kleis
// Safe division that doesn't crash on zero
define safeDivide(a, b) =
    if b = 0 then None
    else Some(a / b)

// Use with pattern matching
define showResult(result) =
    match result {
        None => "undefined"
        Some(x) => x
    }
```

### Result(T, E)

For operations that can succeed or fail with an error (like Rust's `Result`):

```kleis
data Result(T, E) =
    Ok(value: T)
    | Err(error: E)
```

**Usage:**

```kleis
define parseNumber(s) =
    if isNumeric(s) then Ok(toNumber(s))
    else Err("not a number")

define processInput(input) =
    match parseNumber(input) {
        Ok(n) => n * 2
        Err(msg) => 0
    }
```

### List(T)

Recursive linked list:

```kleis
data List(T) =
    Nil
    | Cons(head: T, tail: List(T))
```

**Operations:**

```kleis
define isEmpty(list) = match list {
    Nil => True
    | Cons(_, _) => False
}

define head(list) = match list {
    Nil => None
    | Cons(h, _) => Some(h)
}

define tail(list) = match list {
    Nil => None
    | Cons(_, t) => Some(t)
}
```

**Note:** For numeric computation, use the `[1, 2, 3]` list literal syntax with built-in functions like `list_map`, `list_filter`, etc. The `Cons`/`Nil` form is for symbolic reasoning and pattern matching.

### Unit

The type with only one value (like `void` in C, but safer):

```kleis
data Unit = Unit
```

**Usage:**

```kleis
// A function that returns nothing meaningful
define printAndReturn(x) = Unit

// Flags without associated data
define flagSet : Option(Unit) = Some(Unit)
define flagUnset : Option(Unit) = None
```

### Ordering

For comparison results:

```kleis
data Ordering = LT | EQ | GT
```

**Usage:**

```kleis
define compareNumbers(a, b) =
    if a < b then LT
    else if a > b then GT
    else EQ
```

## Numeric Types

### Scalar

The base numeric type (real numbers ℝ):

```kleis
// Part of the Type data type
Scalar
```

### Complex

Complex numbers with real and imaginary parts:

```kleis
// Part of the Type data type  
Complex
```

See [Chapter 14: Complex Numbers](./14-complex-numbers.md) for operations.

### Vector(n, T)

Fixed-length vectors:

```kleis
Vector(n: Nat, T)
```

Example: `Vector(3, ℝ)` is a 3D real vector.

### Matrix(m, n, T)

Matrices with row and column dimensions:

```kleis
Matrix(m: Nat, n: Nat, T)
```

Example: `Matrix(2, 3, ℝ)` is a 2×3 real matrix.

**Delimiter Variants:**

```kleis
Matrix(m, n, T)   // [a b; c d] - square brackets
PMatrix(m, n, T)  // (a b; c d) - parentheses
VMatrix(m, n, T)  // |a b; c d| - vertical bars (determinants)
BMatrix(m, n, T)  // {a b; c d} - braces
```

See [Chapter 19: Matrices](./19-matrices.md) for operations.

## Tensor Types (xAct-style)

Kleis includes types for differential geometry with xAct-style tensor notation.

### IndexVariance

Whether an index is upper (contravariant) or lower (covariant):

```kleis
data IndexVariance = Contravariant | Covariant
```

### TensorIdx

A single tensor index:

```kleis
data TensorIdx = TIdx(name: String, variance: IndexVariance)
```

Example: `TIdx("μ", Contravariant)` represents the upper index μ.

### TensorRank

Tensor rank as (contravariant, covariant) pair:

```kleis
data TensorRank = Rank(upper: Nat, lower: Nat)
```

Example: `Rank(1, 2)` for a (1,2)-tensor like Γ^μ_{νρ}.

### TensorType

Full tensor type with explicit index structure:

```kleis
data TensorType = TensorT(name: String, indices: List(TensorIdx))
```

**Usage in physics:**

```kleis
// Christoffel symbol Γ^λ_μν
let christoffel = TensorT("Gamma", [
    TIdx("lambda", Contravariant),
    TIdx("mu", Covariant),
    TIdx("nu", Covariant)
])

// Riemann tensor R^ρ_σμν
let riemann = TensorT("R", [
    TIdx("rho", Contravariant),
    TIdx("sigma", Covariant),
    TIdx("mu", Covariant),
    TIdx("nu", Covariant)
])
```

## The Prelude: Core Structures

After types load, `minimal_prelude.kleis` defines the fundamental structures that make operations work.

### Arithmetic(T)

Basic math operations for any type:

```kleis
structure Arithmetic(T) {
    operation plus : T → T → T
    operation minus : T → T → T
    operation times : T → T → T
    operation divide : T → T → T
    operation frac : T → T → T
}

// Implementation for real numbers
implements Arithmetic(ℝ) {
    operation plus = builtin_add
    operation minus = builtin_sub
    operation times = builtin_mul
    operation divide = builtin_div
    operation frac = builtin_div
}
```

### Equatable(T)

Equality comparison:

```kleis
structure Equatable(T) {
    operation equals : T → T → Bool
    operation not_equals : T → T → Bool
}

implements Equatable(ℝ) {
    operation equals = builtin_eq
    operation not_equals = builtin_neq
}

// Matrices have component-wise equality
implements Equatable(Matrix(m, n, ℝ)) {
    operation equals = builtin_matrix_eq
    operation not_equals = builtin_matrix_neq
}
```

### Ordered(T)

Comparison operations (only for types with natural ordering):

```kleis
structure Ordered(T) {
    operation less_than : T → T → Bool
    operation greater_than : T → T → Bool
    operation less_equal : T → T → Bool
    operation greater_equal : T → T → Bool
}

// Only scalars are ordered - matrices are NOT!
implements Ordered(ℝ) {
    operation less_than = builtin_lt
    operation greater_than = builtin_gt
    operation less_equal = builtin_le
    operation greater_equal = builtin_ge
}
```

**Note:** Matrices don't implement `Ordered` because matrix comparison isn't well-defined (is [[1,2],[3,4]] < [[5,6],[7,8]]?).

### Numeric(N)

Advanced numeric operations:

```kleis
structure Numeric(N) {
    operation abs : N → N
    operation floor : N → N
    operation sqrt : N → N
    operation power : N → N → N
}

implements Numeric(ℝ) {
    operation abs = builtin_abs
    operation floor = builtin_floor
    operation sqrt = builtin_sqrt
    operation power = builtin_pow
}
```

### Calculus Structures

For symbolic differentiation and integration:

```kleis
structure Differentiable(F) {
    operation derivative : F → F
    operation partial : F → F
}

structure Integrable(F) {
    operation integral : F → ℝ
    operation int_bounds : F → ℝ → ℝ → F → ℝ
}
```

## Other Standard Library Files

The stdlib contains domain-specific modules:

| File | Purpose |
|------|---------|
| `complex.kleis` | Complex number operations |
| `matrices.kleis` | Matrix algebra, transpose, determinant |
| `tensors.kleis` | Tensor algebra, index contraction |
| `calculus.kleis` | Derivatives, integrals, limits |
| `sets.kleis` | Set operations ∪, ∩, ⊆ |
| `lists.kleis` | List operations, folds |
| `rational.kleis` | Rational number arithmetic |
| `bitvector.kleis` | Bit manipulation |
| `combinatorics.kleis` | Factorials, binomials |
| `bigops.kleis` | Σ, Π, big operators |
| `quantum.kleis` | Quantum mechanics notation |

Import what you need:

```kleis
import "stdlib/complex.kleis"
import "stdlib/tensors.kleis"
```

## Loading Order

The standard library loads in a specific order:

1. **types.kleis** - Core type definitions (Bool, Option, etc.)
2. **minimal_prelude.kleis** - Core structures (Arithmetic, Equatable, etc.)
3. **Domain modules** - As imported by user

This ensures types are defined before they're used in structures.

## Extending the Type System

Because types are defined in Kleis, you can add your own:

```kleis
// Physics domain
data Particle = Electron | Proton | Neutron | Photon
data Spin = SpinUp | SpinDown

// Chemistry domain  
data Element = H | He | Li | Be | B | C | N | O

// Business domain
data Currency = USD | EUR | GBP | JPY
```

Your custom types work with pattern matching, axioms, and Z3 verification just like built-in types.

## What's Next?

Explore specific type domains in depth:

→ [Complex Numbers](./14-complex-numbers.md)  
→ [Matrices](./19-matrices.md)  
→ [Set Theory](./18-sets.md)

