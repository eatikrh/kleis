# Appendix: Standard Library Reference

The Kleis standard library provides 30 files covering mathematics, physics, and computer science.

## File Organization

```
stdlib/
├── types.kleis              // Core type definitions (Bool, Option, List, etc.)
├── prelude.kleis            // Core structures and operations
├── minimal_prelude.kleis    // Arithmetic, Equatable, Ordered structures
├── func_core.kleis          // Higher-order functions (map, fold, etc.)
│
├── # Mathematics
├── complex.kleis            // Complex number axioms (ℂ field)
├── rational.kleis           // Rational number axioms (ℚ ordered field)
├── matrices.kleis           // Matrix algebra (605 lines!)
├── lists.kleis              // List operations and axioms
├── sets.kleis               // Z3-backed set theory
├── combinatorics.kleis      // Permutations, factorials, binomials
├── bigops.kleis             // Σ, Π, ∫, lim as polymorphic HOFs
├── bitvector.kleis          // Z3 BitVec theory
├── text.kleis               // String operations
├── math_functions.kleis     // Trig, hyperbolic, special functions
│
├── # Calculus & Analysis
├── calculus.kleis           // Derivatives, integrals, limits
├── calculus_hof.kleis       // Derivative as (F → F) → F → F
├── symbolic_diff.kleis      // Expression AST with diff(e, x)
│
├── # Tensors & Differential Geometry
├── tensors.kleis            // Abstract tensor algebra
├── tensors_functional.kleis // Pure Kleis tensor operations
├── tensors_concrete.kleis   // Component-based tensors for Z3
├── tensors_minimal.kleis    // Physics palette notation
├── differential_forms.kleis // Cartan calculus (d, ∧, ⋆, ι)
├── cartan_geometry.kleis    // Axiomatic framework for curvature
├── cartan_compute.kleis     // Tetrad → Connection → Curvature pipeline
│
├── # Physics
├── maxwell.kleis            // Covariant electromagnetism
├── fluid_dynamics.kleis     // Navier-Stokes, Bernoulli, Stokes
├── solid_mechanics.kleis    // Stress/strain, Von Mises, Mohr-Coulomb
├── quantum.kleis            // Hilbert space, Dirac notation, Pauli
├── quantum_minimal.kleis    // Physics palette quantum notation
└── cosmology.kleis          // Minkowski, de Sitter, FLRW, Schwarzschild
```

## Core Files

### `types.kleis` — Self-Hosted Type System

The type system is defined *in Kleis itself*:

```kleis
data Type =
    Scalar
    | Vector(n: Nat, T)
    | Matrix(m: Nat, n: Nat, T)
    | Complex
    | Set(T: Type)
    | List(T: Type)
    | Tensor(dims: List(Nat))

data Bool = True | False
data Option(T) = None | Some(value: T)
data Result(T, E) = Ok(value: T) | Err(error: E)
data List(T) = Nil | Cons(head: T, tail: List(T))
```

### `prelude.kleis` — Core Structures

Defines fundamental mathematical structures:

```kleis
structure Ring(R) {
    zero : R
    one : R
    operation add : R × R → R
    operation mul : R × R → R
    operation neg : R → R
    
    axiom add_assoc : ∀(a b c : R). add(add(a, b), c) = add(a, add(b, c))
    axiom distributive : ∀(a b c : R). mul(a, add(b, c)) = add(mul(a, b), mul(a, c))
}
```

### `func_core.kleis` — Functional Primitives

Higher-order functions and function composition:

```kleis
define compose(f, g) = λ x . f(g(x))
define id(x) = x
define const(x) = λ _ . x
define flip(f) = λ x y . f(y, x)
```

## Mathematics Files

### `matrices.kleis` — Full Matrix Algebra (605 lines)

Comprehensive matrix operations with axioms:

```kleis
structure Matrix(m: Nat, n: Nat, T) {
    operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
    operation matmul : Matrix(m, n, T) × Matrix(n, p, T) → Matrix(m, p, T)
    operation det : Matrix(n, n, T) → T
    operation inv : Matrix(n, n, T) → Matrix(n, n, T)
    operation eigenvalues : Matrix(n, n, T) → List(ℂ)
    
    axiom transpose_involution : ∀ A : Matrix(m, n, T) .
        transpose(transpose(A)) = A
}
```

### `symbolic_diff.kleis` — Symbolic Differentiation

Expression AST with pattern-matching differentiation:

```kleis
data Expression = 
    ENumber(value : ℝ)
  | EVariable(name : String)
  | EOperation(name : String, args : List(Expression))

define diff(e, x) = match e {
    ENumber(_) => num(0)
    | EVariable(name) => if str_eq(name, x) then num(1) else num(0)
    | EOperation("plus", [a, b]) => e_add(diff(a, x), diff(b, x))
    | EOperation("times", [a, b]) => 
        e_add(e_mul(diff(a, x), b), e_mul(a, diff(b, x)))  // Product rule
    | EOperation("sin", [f]) => e_mul(e_cos(f), diff(f, x))  // Chain rule
    // ... more rules
}
```

### `calculus_hof.kleis` — Derivative as Higher-Order Function

```
structure Derivative(F) {
    operation D : (F → F) → F → F
    
    axiom chain_rule : ∀(f g : F → F)(x : F).
        D(compose(f, g))(x) = times(D(f)(g(x)), D(g)(x))
    
    axiom product_rule : ∀(f g : F → F)(x : F).
        D(times_fn(f, g))(x) = plus(times(D(f)(x), g(x)), times(f(x), D(g)(x)))
    
    axiom linearity : ∀(f g : F → F)(x : F).
        D(plus_fn(f, g))(x) = plus(D(f)(x), D(g)(x))
}
```

### `bigops.kleis` — Polymorphic Big Operators

Σ, Π, ∫, lim that work on any type with the right structure:

```
// Summation requires additive monoid (has +, 0)
operation sum_bounds : (ℤ → T) × ℤ × ℤ → T

// Product requires multiplicative monoid (has ×, 1)
operation prod_bounds : (ℤ → T) × ℤ × ℤ → T

// Integral requires Banach space (complete normed vector space)
operation int_bounds : (T → S) × T × T × T → S

// Probability expectation
operation E : (Ω → ℝ) → ℝ
```

## Physics Files

### `differential_forms.kleis` — Cartan Calculus

Full exterior calculus with wedge products, exterior derivative, and Hodge star:

```
structure WedgeProduct(p: Nat, q: Nat, dim: Nat) {
    operation wedge : DifferentialForm(p, dim) → DifferentialForm(q, dim) 
                    → DifferentialForm(p + q, dim)
    
    axiom graded_antisymmetric : ∀ α β .
        wedge(α, β) = scale(power(-1, p*q), wedge(β, α))
}

structure ExteriorDerivative(p: Nat, dim: Nat) {
    operation d : DifferentialForm(p, dim) → DifferentialForm(p + 1, dim)
    
    axiom d_squared_zero : ∀ α . d(d(α)) = 0  // Fundamental!
}

// Cartan's Magic Formula: ℒ_X = d ∘ ι_X + ι_X ∘ d
define cartan_magic_impl(X, alpha) = 
    plus(d(interior(X, alpha)), interior(X, d(alpha)))
```

### `cartan_compute.kleis` — Schwarzschild Curvature

Complete pipeline from tetrad to Riemann curvature:

```kleis
define schwarzschild_tetrad(M) =
    let f = e_sub(num(1), e_div(e_mul(num(2), M), var("r"))) in
    let sqrt_f = e_sqrt(f) in
    [
        scale1(sqrt_f, dt),
        scale1(e_div(num(1), sqrt_f), dr),
        scale1(var("r"), dtheta),
        scale1(e_mul(var("r"), e_sin(var("theta"))), dphi)
    ]

define compute_riemann(tetrad) =
    let omega = solve_connection(tetrad) in
    compute_curvature(omega)

define schwarzschild_curvature(M) = compute_riemann(schwarzschild_tetrad(M))
```

### `quantum.kleis` — Hilbert Space Formalism

Full quantum mechanics with Dirac notation:

```kleis
structure Ket(dim: Nat, T) {
    operation normalize : Ket(dim, T) → Ket(dim, T)
    operation scale : T → Ket(dim, T) → Ket(dim, T)
}

structure Operator(dim: Nat, T) {
    operation apply : Operator(dim, T) → Ket(dim, T) → Ket(dim, T)
    operation adjoint : Operator(dim, T) → Operator(dim, T)
    operation compose : Operator(dim, T) → Operator(dim, T) → Operator(dim, T)
}

structure Commutator(dim: Nat, T) {
    operation commutator : Operator(dim, T) → Operator(dim, T) → Operator(dim, T)
    // [x̂, p̂] = iℏ (Heisenberg uncertainty!)
}
```

### `maxwell.kleis` — Covariant Electromagnetism

```kleis
structure MaxwellInhomogeneous {
    operation F : Nat → Nat → ℝ  // Field tensor
    operation J : Nat → ℝ         // 4-current
    
    // ∂_μ F^μν = μ₀ J^ν
    axiom maxwell_inhomogeneous : ∀ nu : Nat .
        divF(nu) = times(mu0, J(nu))
}
```

### `fluid_dynamics.kleis` — Navier-Stokes

```kleis
structure MomentumEquation {
    // ρ ∂u_i/∂t + ∂(ρu_i u_j)/∂x_j = -∂p/∂x_i + ∂τ_ij/∂x_j + ρf_i
    axiom momentum : ∀ i : Nat .
        plus(times(rho, du_dt(i)), div_momentum(i)) = 
        plus(plus(negate(grad_p(i)), div_tau(i)), times(rho, f(i)))
}
```

## Loading the Standard Library

In files:

```kleis
import "stdlib/prelude.kleis"
import "stdlib/matrices.kleis"
import "stdlib/symbolic_diff.kleis"
```

In the REPL:

```
kleis> :load stdlib/prelude.kleis
Loaded standard library.
```

## See Also

- [Cartan Geometry Appendix](./cartan-geometry.md) — Full Schwarzschild example
- [ODE Solver Appendix](./ode-solver.md) — Control systems with LQR
- [LAPACK Functions](./lapack.md) — Numerical linear algebra
- [Built-in Functions](./builtin-functions.md) — Complete function reference
