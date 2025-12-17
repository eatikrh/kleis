# Appendix C: Standard Library

The Kleis standard library provides foundational types, structures, and operations.

## Core Types

### Numeric Types

```kleis
-- Natural numbers (0, 1, 2, ...)
ℕ  // or Nat

-- Integers (..., -1, 0, 1, ...)
ℤ  // or Int

-- Real numbers
ℝ  // or Real

-- Complex numbers
ℂ  // or Complex
```

### Boolean Type

```kleis
Bool  // True or False
```

### Unit Type

```kleis
Unit  // Single value: ()
```

## Collection Types

### List

```kleis
structure List(T) {
    operation head : T
    operation tail : List(T)
    operation length : ℕ
    operation append : List(T) → List(T)
    operation map : (T → U) → List(U)
    operation filter : (T → Bool) → List(T)
    operation fold : (U × T → U) → U → U
}
```

### Vector

```kleis
structure Vector(n : ℕ, T) {
    operation get : ℕ → T
    operation length : ℕ
    operation dot : Vector(n, T) → T
    operation magnitude : ℝ
    operation normalize : Vector(n, T)
}
```

### Matrix

```kleis
structure Matrix(m : ℕ, n : ℕ, T) {
    operation get : ℕ × ℕ → T
    operation rows : ℕ
    operation cols : ℕ
    operation transpose : Matrix(n, m, T)
    operation add : Matrix(m, n, T) → Matrix(m, n, T)
    operation mul : Matrix(n, p, T) → Matrix(m, p, T)
}

structure SquareMatrix(n : ℕ, T) extends Matrix(n, n, T) {
    operation det : T
    operation trace : T
    operation inv : SquareMatrix(n, T)
    operation eigenvalues : List(ℂ)
}
```

## Algebraic Structures

### Monoid

```kleis
structure Monoid(M) {
    operation e : M               // Identity element
    operation mul : M × M → M     // Binary operation
    
    axiom identity_left : ∀ x : M . mul(e, x) = x
    axiom identity_right : ∀ x : M . mul(x, e) = x
    axiom associative : ∀ x : M . ∀ y : M . ∀ z : M .
        mul(mul(x, y), z) = mul(x, mul(y, z))
}
```

### Group

```kleis
structure Group(G) extends Monoid(G) {
    operation inv : G → G         // Inverse
    
    axiom inverse_left : ∀ x : G . mul(inv(x), x) = e
    axiom inverse_right : ∀ x : G . mul(x, inv(x)) = e
}
```

### Ring

```kleis
structure Ring(R) {
    operation zero : R
    operation one : R
    operation add : R × R → R
    operation mul : R × R → R
    operation neg : R → R
    
    // (R, add, zero) is an abelian group
    // (R, mul, one) is a monoid
    // mul distributes over add
}
```

### Field

```kleis
structure Field(F) extends Ring(F) {
    operation inv : F → F  // Multiplicative inverse (for non-zero)
    
    axiom mul_inverse : ∀ x : F . x ≠ zero → mul(x, inv(x)) = one
    axiom mul_commutative : ∀ x : F . ∀ y : F . mul(x, y) = mul(y, x)
}
```

### Vector Space

```kleis
structure VectorSpace(V, F) where F : Field {
    operation add : V × V → V
    operation scale : F × V → V
    operation zero : V
    
    // (V, add, zero) is an abelian group
    // scale distributes over add
    // scale is associative with field multiplication
}
```

## Geometric Structures

### Metric Space

```kleis
structure MetricSpace(M) {
    operation distance : M × M → ℝ
    
    axiom non_negative : ∀ x : M . ∀ y : M . distance(x, y) ≥ 0
    axiom identity : ∀ x : M . ∀ y : M . distance(x, y) = 0 ↔ x = y
    axiom symmetric : ∀ x : M . ∀ y : M . distance(x, y) = distance(y, x)
    axiom triangle : ∀ x : M . ∀ y : M . ∀ z : M .
        distance(x, z) ≤ distance(x, y) + distance(y, z)
}
```

### Manifold

```kleis
structure Manifold(M, dim : ℕ) {
    operation tangent : M → TangentSpace(dim)
    operation chart : M → ℝ^dim  // Local coordinates
}
```

### Riemannian Manifold

```kleis
structure RiemannianManifold(M, dim : ℕ) extends Manifold(M, dim) {
    operation metric : M → Matrix(dim, dim, ℝ)
    operation christoffel : M → Tensor(1, 2)
    operation riemann : M → Tensor(1, 3)
    operation ricci : M → Matrix(dim, dim, ℝ)
    operation scalar_curvature : M → ℝ
}
```

## Option and Result Types

### Option

```kleis
enum Option(T) {
    Some(value : T)
    None
}

-- Operations
define is_some(opt : Option(T)) : Bool =
    match opt { Some(_) => True, None => False }

define unwrap_or(opt : Option(T), default : T) : T =
    match opt { Some(v) => v, None => default }
```

### Result

```kleis
enum Result(T, E) {
    Ok(value : T)
    Err(error : E)
}

-- Operations
define is_ok(res : Result(T, E)) : Bool =
    match res { Ok(_) => True, Err(_) => False }

define map(res : Result(T, E), f : T → U) : Result(U, E) =
    match res { Ok(v) => Ok(f(v)), Err(e) => Err(e) }
```

## Loading the Standard Library

In the REPL:

```
kleis> :load stdlib/prelude.kleis
Loaded standard library.
```

In files:

```kleis
import stdlib.prelude
import stdlib.linear_algebra
import stdlib.differential_geometry
```

## File Organization

```
stdlib/
├── prelude.kleis          // Core types and functions
├── numeric.kleis          // Numeric operations
├── collections.kleis      // List, Vector, Matrix
├── algebraic.kleis        // Group, Ring, Field, etc.
├── linear_algebra.kleis   // Matrix operations
├── differential_geometry.kleis  // Manifolds, tensors
└── category_theory.kleis  // Categories, functors
```
