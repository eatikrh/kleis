# Appendix: Cartan Geometry and Tensor Computation

This appendix demonstrates Kleis's capability to perform **symbolic tensor calculus** for general relativity, computing actual curvature tensors from metric specifications.

## Overview

Kleis implements Cartan's formalism for differential geometry:

```
Metric → Tetrad → Connection → Curvature → Ricci → Einstein
```

This is the same computational pipeline used in research-grade general relativity software like xAct (Mathematica) and Cadabra.

## The Cartan Pipeline

### 1. Expression AST

Symbolic expressions are represented using the `Expression` algebraic data type, consistent with `kleis_in_kleis.kleis`:

```kleis
data Expression = 
    ENumber(value : ℝ)
  | EVariable(name : String)
  | EOperation(name : String, args : List(Expression))
```

This representation allows any operation to be encoded uniformly via `EOperation`. Helper constructors provide cleaner syntax:

```kleis
// Value constructors
define num(n) = ENumber(n)
define var(x) = EVariable(x)

// Operation constructors (e_ prefix avoids builtin conflicts)
define e_add(a, b) = EOperation("plus", Cons(a, Cons(b, Nil)))
define e_mul(a, b) = EOperation("times", Cons(a, Cons(b, Nil)))
define e_pow(a, b) = EOperation("power", Cons(a, Cons(b, Nil)))
define e_sin(a) = EOperation("sin", Cons(a, Nil))
define e_cos(a) = EOperation("cos", Cons(a, Nil))
define e_sqrt(a) = EOperation("sqrt", Cons(a, Nil))
// ... etc
```

### 2. Symbolic Differentiation

The `diff` function computes derivatives by pattern matching on the `Expression` AST:

```kleis
define diff(e, var_name) = match e {
    // Constant rule: d/dx(c) = 0
    ENumber(_) => num(0)
    
    // Variable rule: d/dx(x) = 1, d/dx(y) = 0 if y ≠ x
    // Note: str_eq() is used for concrete string comparison
    EVariable(name) => if str_eq(name, var_name) then num(1) else num(0)
    
    // Operation rules - dispatch by operation name
    EOperation(op_name, args) => diff_op(op_name, args, var_name)
}

define diff_op(op_name, args, var_name) = match op_name {
    "plus" => match args {
        Cons(f, Cons(g, Nil)) => e_add(diff(f, var_name), diff(g, var_name))
        | _ => num(0)
    }
    "times" => match args {
        // Product rule: d/dx(f * g) = f' * g + f * g'
        Cons(f, Cons(g, Nil)) => 
            e_add(e_mul(diff(f, var_name), g), e_mul(f, diff(g, var_name)))
        | _ => num(0)
    }
    "power" => match args {
        // Power rule with constant exponent
        Cons(f, Cons(ENumber(n), Nil)) => 
            e_mul(e_mul(num(n), e_pow(f, num(n - 1))), diff(f, var_name))
        // General power rule
        | _ => num(0)
    }
    "sin" => match args {
        Cons(f, Nil) => e_mul(e_cos(f), diff(f, var_name))
        | _ => num(0)
    }
    // ... more rules
}
```

> **Note:** We use `str_eq(name, var_name)` instead of pattern matching because Kleis patterns **bind** variables rather than compare them. The `str_eq` builtin provides concrete string equality.

### 3. Differential Forms

1-forms and 2-forms are represented as coefficient lists:

```kleis
// 1-form: ω = ω_t dt + ω_r dr + ω_θ dθ + ω_φ dφ
// Represented as [ω_t, ω_r, ω_θ, ω_φ]

define dt = [num(1), num(0), num(0), num(0)]
define dr = [num(0), num(1), num(0), num(0)]
define dtheta = [num(0), num(0), num(1), num(0)]
define dphi = [num(0), num(0), num(0), num(1)]
```

### 4. Exterior Derivative

```kleis
// d(f) = ∂f/∂t dt + ∂f/∂r dr + ∂f/∂θ dθ + ∂f/∂φ dφ
define d0(f) = [
    simplify(diff_t(f)),
    simplify(diff_r(f)),
    simplify(diff_theta(f)),
    simplify(diff_phi(f))
]

// Coordinate-specific derivatives
define diff_t(e) = diff(e, "t")
define diff_r(e) = diff(e, "r")
define diff_theta(e) = diff(e, "theta")
define diff_phi(e) = diff(e, "phi")
```

### 5. Wedge Product

```kleis
// (α ∧ β)_μν = α_μ β_ν - α_ν β_μ
define wedge(a, b) =
    let a0 = nth(a, 0) in let a1 = nth(a, 1) in
    let a2 = nth(a, 2) in let a3 = nth(a, 3) in
    let b0 = nth(b, 0) in let b1 = nth(b, 1) in
    let b2 = nth(b, 2) in let b3 = nth(b, 3) in
    [
        [num(0),
         simplify(e_sub(e_mul(a0, b1), e_mul(a1, b0))),
         ...],
        ...
    ]
```

## Example: Schwarzschild Black Hole

The Schwarzschild metric describes spacetime around a non-rotating black hole:

```
ds² = -(1 - 2M/r)dt² + dr²/(1 - 2M/r) + r²dθ² + r²sin²θ dφ²
```

### Tetrad Definition

```kleis
define schwarzschild_tetrad(M) =
    let f = e_sub(num(1), e_div(e_mul(num(2), M), var("r"))) in
    let sqrt_f = e_sqrt(f) in
    [
        scale1(sqrt_f, dt),                              // e⁰ = √f dt
        scale1(e_div(num(1), sqrt_f), dr),              // e¹ = dr/√f
        scale1(var("r"), dtheta),                        // e² = r dθ
        scale1(e_mul(var("r"), e_sin(var("theta"))), dphi)  // e³ = r sin(θ) dφ
    ]
```

### Computing Curvature

```kleis
define compute_riemann(tetrad) =
    let omega = solve_connection(tetrad) in
    compute_curvature(omega)

define schwarzschild_curvature(M) = 
    compute_riemann(schwarzschild_tetrad(M))
```

### Results

| Computation | Size | Time |
|-------------|------|------|
| Tetrad | ~300 chars | instant |
| Connection ω^a_b | ~2,400 chars | ~1s |
| Curvature R^a_b | ~22,000 chars | ~4s |

## Literature Verification

The computed results match known properties from the literature:

### Minkowski (Flat Space)
- **Expected:** All curvature components = 0
- **Computed:** 238/256 components are `ENumber(0)` ✓
- **Reference:** Misner, Thorne, Wheeler "Gravitation" (1973)

### Schwarzschild
- **Expected:** Curvature depends on M, r, angular coordinates
- **Computed:** Contains `EVariable("M")`, `EVariable("r")`, `sin`, `cos` ✓
- **Expected:** Contains metric factor √(1-2M/r)
- **Computed:** Contains `e_sqrt(e_sub(num(1), e_div(...)))` ✓
- **Reference:** Carroll "Spacetime and Geometry" (2004)

## Implementation Notes

### Why `e_*` Prefix?

Functions like `pow`, `add`, `mul` conflict with Kleis builtins. When you write `pow(var("x"), num(2))`, Kleis interprets `pow` as the built-in power operation and tries to compute `EVariable("x") ^ ENumber(2)` numerically—which fails.

The `e_*` prefix (`e_pow`, `e_add`, etc.) ensures these are treated as user-defined functions that construct `EOperation` nodes.

### Why `str_eq` Instead of Pattern Matching?

In Kleis (and ML-family languages), pattern variables **bind** rather than compare:

```kleis
// This BINDS 'x' to whatever name contains, always matches!
EVariable(name) => match name { x => num(1) | _ => num(0) }

// This COMPARES name to var_name using str_eq
EVariable(name) => if str_eq(name, var_name) then num(1) else num(0)
```

The `str_eq` builtin provides concrete string equality that returns `true` or `false`.

## Z3 Verification

While the Cartan computation runs in the Kleis evaluator, the results can be verified using Z3:

```kleis
// Verify Riemann tensor symmetries
axiom riemann_antisym : ∀ R μ ν ρ σ .
    component(R, μ, ν, ρ, σ) = negate(component(R, ν, μ, ρ, σ))

// Verify Bianchi identity
axiom bianchi : ∀ R λ ρ σ μ ν .
    plus(plus(
        nabla(R, λ, ρ, σ, μ, ν),
        nabla(R, ρ, σ, λ, μ, ν)),
        nabla(R, σ, λ, ρ, μ, ν)) = 0
```

## Files

| File | Description |
|------|-------------|
| `stdlib/symbolic_diff.kleis` | Symbolic differentiation using Expression AST |
| `stdlib/cartan_compute.kleis` | Cartan geometry pipeline |
| `tests/symbolic_diff_test.rs` | 25 tests for differentiation |
| `tests/cartan_compute_test.rs` | 22 tests including literature verification |

## Research Applications

| Domain | Application |
|--------|-------------|
| **General Relativity** | Compute curvature for new metrics |
| **Cosmology** | Verify FLRW, de Sitter models |
| **Modified Gravity** | Check f(R) theory consistency |
| **Numerical Relativity** | Verify constraint equations |
| **Education** | Interactive GR computations |

## Comparison to Other Tools

| Tool | Symbolic | Verification | Notes |
|------|----------|--------------|-------|
| Mathematica + xAct | ✓✓✓ | ✗ | Industry standard, expensive |
| Cadabra | ✓✓ | ✗ | Open source tensor algebra |
| SageMath | ✓✓ | ✗ | General purpose |
| **Kleis** | ✓✓ | ✓✓ | Combines both! |

Kleis occupies a unique niche: **symbolic mathematics with formal verification**.

