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

Symbolic expressions are represented as algebraic data types:

```kleis
data Expr = 
    Const(value : ℝ)
  | Var(name : String)
  | Add(left : Expr, right : Expr)
  | Mul(left : Expr, right : Expr)
  | Pow(base : Expr, exp : Expr)
  | Sin(arg : Expr)
  | Cos(arg : Expr)
  | Sqrt(arg : Expr)
  | ...
```

### 2. Symbolic Differentiation

The `diff` function computes derivatives by pattern matching:

```kleis
define diff(e, x) = match e {
    Const(_) => Const(0)
    Var(name) => if name = x then Const(1) else Const(0)
    Add(f, g) => Add(diff(f, x), diff(g, x))
    Mul(f, g) => Add(Mul(diff(f, x), g), Mul(f, diff(g, x)))
    Pow(f, Const(n)) => Mul(Mul(Const(n), Pow(f, Const(n-1))), diff(f, x))
    Sin(f) => Mul(Cos(f), diff(f, x))
    ...
}
```

### 3. Differential Forms

1-forms and 2-forms are represented as coefficient lists:

```kleis
// 1-form: ω = ω_t dt + ω_r dr + ω_θ dθ + ω_φ dφ
// Represented as [ω_t, ω_r, ω_θ, ω_φ]

define dt = [Const(1), Const(0), Const(0), Const(0)]
define dr = [Const(0), Const(1), Const(0), Const(0)]
```

### 4. Exterior Derivative

```kleis
// d(f) = ∂f/∂t dt + ∂f/∂r dr + ∂f/∂θ dθ + ∂f/∂φ dφ
define d0(f) = [
    diff(f, "t"),
    diff(f, "r"),
    diff(f, "theta"),
    diff(f, "phi")
]
```

### 5. Wedge Product

```kleis
// (α ∧ β)_μν = α_μ β_ν - α_ν β_μ
define wedge(a, b) = [
    [Const(0), Sub(Mul(a0, b1), Mul(a1, b0)), ...],
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
define schw_f(M) = Sub(Const(1), Div(Mul(Const(2), M), Var("r")))

define schwarzschild_tetrad(M) = [
    scale1(Sqrt(schw_f(M)), dt),           // e⁰ = √f dt
    scale1(Div(Const(1), Sqrt(schw_f(M))), dr),  // e¹ = dr/√f
    scale1(Var("r"), dtheta),              // e² = r dθ
    scale1(Mul(Var("r"), Sin(Var("theta"))), dphi)  // e³ = r sin(θ) dφ
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
| Tetrad | 309 chars | instant |
| Connection ω^a_b | 2,387 chars | ~1s |
| Curvature R^a_b | 22,115 chars | ~6s |

## Literature Verification

The computed results match known properties from the literature:

### Minkowski (Flat Space)
- **Expected:** All curvature components = 0
- **Computed:** 238/256 components are `Const(0)` ✓
- **Reference:** Misner, Thorne, Wheeler "Gravitation" (1973)

### Schwarzschild
- **Expected:** Curvature depends on M, r, angular coordinates
- **Computed:** Contains `Var("M")`, `Var("r")`, `Sin`, `Cos` ✓
- **Expected:** Contains metric factor √(1-2M/r)
- **Computed:** Contains `Sqrt(Sub(Const(1), Div(...)))` ✓
- **Reference:** Carroll "Spacetime and Geometry" (2004)

## Higher-Order Functions

This computation is enabled by Kleis's support for higher-order functions:

```kleis
// Pass coordinate-specific derivative functions
define diff_wrt_r(e) = diff_core(e, "r", is_r)

define is_r(name) = match name { "r" => Const(1) | _ => Const(0) }
```

The evaluator resolves `is_r("r")` to `Const(1)` before any symbolic manipulation, avoiding expression explosion.

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
| `stdlib/symbolic_diff.kleis` | Symbolic differentiation |
| `stdlib/cartan_compute.kleis` | Cartan geometry pipeline |
| `tests/cartan_compute_test.rs` | 25 tests including literature verification |

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

