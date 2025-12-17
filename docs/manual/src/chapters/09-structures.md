# Structures

## What Are Structures?

Structures define mathematical objects with their properties and operations. Think of them as "blueprints" for mathematical concepts.

```kleis
structure Vector(n : ℕ) {
    -- Operations this structure supports
    operation add : Vector(n) → Vector(n)
    operation scale : ℝ → Vector(n)
    operation dot : Vector(n) → ℝ
    
    -- Properties that must hold
    axiom commutative : ∀ u : Vector(n) . ∀ v : Vector(n) .
        add(u, v) = add(v, u)
}
```

## Structure Syntax

```kleis
structure Name(parameters) {
    -- Fields (data)
    field1 : Type1
    field2 : Type2
    
    -- Operations (functions)
    operation op1 : InputType → OutputType
    
    -- Axioms (properties)
    axiom property : logical_statement
}
```

## Example: Complex Numbers

```kleis
structure Complex {
    field re : ℝ  -- real part
    field im : ℝ  -- imaginary part
    
    operation add : Complex → Complex
    operation mul : Complex → Complex
    operation conj : Complex           -- conjugate
    operation mag : ℝ                  -- magnitude
    
    axiom add_commutative : ∀ z : Complex . ∀ w : Complex .
        add(z, w) = add(w, z)
        
    axiom magnitude_positive : ∀ z : Complex .
        mag(z) ≥ 0
        
    axiom conj_involution : ∀ z : Complex .
        conj(conj(z)) = z
}
```

## Parametric Structures

Structures can have type parameters:

```kleis
structure Matrix(m : ℕ, n : ℕ, T) {
    operation transpose : Matrix(n, m, T)
    operation add : Matrix(m, n, T) → Matrix(m, n, T)
    
    axiom transpose_involution : ∀ A : Matrix(m, n, T) .
        transpose(transpose(A)) = A
}

-- Square matrices have more operations
structure SquareMatrix(n : ℕ, T) extends Matrix(n, n, T) {
    operation det : T
    operation trace : T
    operation inv : SquareMatrix(n, T)
    
    axiom det_mul : ∀ A : SquareMatrix(n, T) . ∀ B : SquareMatrix(n, T) .
        det(mul(A, B)) = det(A) * det(B)
}
```

## The `extends` Keyword

Structures can extend other structures:

```kleis
structure Monoid(M) {
    operation e : M
    operation mul : M × M → M
    
    axiom identity : ∀ x : M . mul(e, x) = x ∧ mul(x, e) = x
    axiom associative : ∀ x : M . ∀ y : M . ∀ z : M .
        mul(mul(x, y), z) = mul(x, mul(y, z))
}

structure Group(G) extends Monoid(G) {
    operation inv : G → G
    
    axiom inverse : ∀ x : G . mul(x, inv(x)) = e ∧ mul(inv(x), x) = e
}

structure AbelianGroup(G) extends Group(G) {
    axiom commutative : ∀ x : G . ∀ y : G . mul(x, y) = mul(y, x)
}
```

## The `where` Clause

Constrain type parameters:

```kleis
structure VectorSpace(V, F) where F : Field {
    operation add : V × V → V
    operation scale : F × V → V
    
    axiom distributive : ∀ a : F . ∀ u : V . ∀ v : V .
        scale(a, add(u, v)) = add(scale(a, u), scale(a, v))
}
```

## Differential Geometry Structures

Kleis shines for differential geometry:

```kleis
structure Manifold(M, dim : ℕ) {
    operation tangent : M → TangentSpace(M)
    operation metric : M → Tensor(0, 2)
    
    axiom metric_symmetric : ∀ p : M .
        metric(p) = transpose(metric(p))
}

structure RiemannianManifold(M, dim : ℕ) extends Manifold(M, dim) {
    operation christoffel : M → Tensor(1, 2)
    operation riemann : M → Tensor(1, 3)
    operation ricci : M → Tensor(0, 2)
    operation scalar_curvature : M → ℝ
    
    axiom first_bianchi : ∀ p : M .
        -- R^a_{bcd} + R^a_{cdb} + R^a_{dbc} = 0
        cyclic_sum(riemann(p)) = 0
}
```

## What's Next?

Learn how to implement structures!

→ [Next: Implements](./10-implements.md)
