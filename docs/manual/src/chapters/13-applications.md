# Applications

## Differential Geometry

Kleis excels at differential geometry calculations:

```kleis
// Christoffel symbols for spherical coordinates
structure SphericalMetric {
    operation metric : (ℝ, ℝ) → Matrix(2, 2, ℝ)
    operation christoffel : (ℝ, ℝ) → Tensor(1, 2)
}

implements SphericalMetric {
    // Metric tensor: ds² = r²(dθ² + sin²θ dφ²)
    operation metric(r, θ) = Matrix [
        [r^2, 0],
        [0, r^2 * sin(θ)^2]
    ]
    
    // Christoffel symbols Γⁱⱼₖ
    operation christoffel(r, θ) = 
        let g = metric(r, θ) in
        let g_inv = inverse(g) in
        // ... compute from metric derivatives
}
```

## Tensor Calculus

```kleis
// Einstein field equations
structure EinsteinEquations {
    // Ricci tensor
    operation ricci : Manifold → Tensor(0, 2)
    // Scalar curvature
    operation scalar : Manifold → ℝ
    // Einstein tensor
    operation einstein : Manifold → Tensor(0, 2)
    
    axiom einstein_tensor : ∀(M : Manifold).
        einstein(M) = ricci(M) - (scalar(M) / 2) * metric(M)
}
```

## Symbolic Differentiation

```kleis
data Expr = Const(value : ℝ) 
          | Var(name : String) 
          | Add(left : Expr, right : Expr) 
          | Mul(left : Expr, right : Expr) 
          | Pow(base : Expr, exp : Expr)
          | Sin(arg : Expr)
          | Cos(arg : Expr)
          | Exp(arg : Expr)
          | Ln(arg : Expr)

define diff(e, x) =
    match e {
        Const(_) => Const(0)
        Var(y) => if y = x then Const(1) else Const(0)
        
        Add(f, g) => Add(diff(f, x), diff(g, x))
        
        Mul(f, g) =>
            Add(Mul(diff(f, x), g), Mul(f, diff(g, x)))
            
        Pow(f, Const(n)) =>
            Mul(Mul(Const(n), Pow(f, Const(n - 1))), diff(f, x))
            
        Sin(f) => Mul(Cos(f), diff(f, x))
        Cos(f) => Mul(Mul(Const(-1), Sin(f)), diff(f, x))
        Exp(f) => Mul(Exp(f), diff(f, x))
        Ln(f) => Mul(Pow(f, Const(-1)), diff(f, x))
        
        _ => Const(0)
    }
```

## Linear Algebra

```kleis
structure LinearSystem(n : ℕ) {
    operation solve : Matrix(n, n, ℝ) × Vector(n, ℝ) → Vector(n, ℝ)
    
    // Solution satisfies Ax = b
    axiom solution_correct : ∀(A : Matrix(n, n, ℝ))(b : Vector(n, ℝ)).
        det(A) ≠ 0 → mul(A, solve(A, b)) = b
}

// Eigenvalue problem
structure Eigen(n : ℕ) {
    operation eigenvalues : Matrix(n, n, ℂ) → List(ℂ)
    operation eigenvectors : Matrix(n, n, ℂ) → List(Vector(n, ℂ))
    
    axiom eigenpair : ∀(A : Matrix(n, n, ℂ))(i : ℕ).
        let lam = nth(eigenvalues(A), i) in
        let v = nth(eigenvectors(A), i) in
            mul(A, v) = scale(lam, v)
}
```

## Quantum Mechanics

```kleis
structure QuantumState(n : ℕ) {
    operation amplitudes : Vector(n, ℂ)
    
    // States must be normalized
    axiom normalized : ∀(psi : QuantumState(n)).
        sum(map(λ a . abs(a)^2, amplitudes(psi))) = 1
}

structure Observable(n : ℕ) {
    operation matrix : Matrix(n, n, ℂ)
    
    // Observables are Hermitian
    axiom hermitian : ∀(O : Observable(n)).
        matrix(O) = conjugate_transpose(matrix(O))
}

// Expectation value
define expectation(psi, O) =
    real(inner_product(amplitudes(psi), mul(matrix(O), amplitudes(psi))))
```

## Category Theory

```kleis
structure Category(Obj, Mor) {
    operation id : Obj → Mor
    operation compose : Mor × Mor → Mor
    operation dom : Mor → Obj
    operation cod : Mor → Obj
    
    axiom identity_left : ∀(f : Mor).
        compose(id(cod(f)), f) = f
        
    axiom identity_right : ∀(f : Mor).
        compose(f, id(dom(f))) = f
        
    axiom associativity : ∀(f : Mor)(g : Mor)(h : Mor).
        compose(compose(h, g), f) = compose(h, compose(g, f))
}

structure Functor(C, D) {
    operation map_obj : C → D
    operation map_mor : C → D
    
    axiom preserves_id : ∀(x : C).
        map_mor(id(x)) = id(map_obj(x))
        
    axiom preserves_compose : ∀(f : C)(g : C).
        map_mor(compose(g, f)) = compose(map_mor(g), map_mor(f))
}
```

## Physics: Classical Mechanics

```kleis
structure LagrangianMechanics(n : ℕ) {
    // Generalized coordinates and velocities
    operation q : ℕ → ℝ     // Position
    operation q_dot : ℕ → ℝ  // Velocity
    operation t : ℝ          // Time
    
    // Lagrangian L = T - V
    operation lagrangian : ℝ
    
    // Euler-Lagrange equations
    // Using Mathematica-style: Dt for total derivative, D for partial
    axiom euler_lagrange : ∀ i : ℕ . i < n →
        Dt(D(lagrangian, q_dot(i)), t) = D(lagrangian, q(i))
}
```

## What's Next?

Check out the reference appendices!

→ [Appendix A: Grammar Reference](../appendix/grammar.md)
→ [Appendix B: Operators](../appendix/operators.md)
→ [Appendix C: Standard Library](../appendix/stdlib.md)
