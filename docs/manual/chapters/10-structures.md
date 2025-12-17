# Chapter 10: Structures

[← Previous: Conditionals](09-conditionals.md) | [Back to Contents](../index.md) | [Next: Implements →](11-implements.md)

---

## What Are Structures?

Structures define **abstract mathematical concepts** — the properties something must have, without specifying what it actually is.

Think of a structure as a **contract**: "Anything that wants to be called a Group must have these operations and satisfy these laws."

---

## Basic Structure Definition

```kleis
structure StructureName(TypeParams) {
    // Operations (the interface)
    operation name : Type
    
    // Axioms (the laws)
    axiom name : Expression
}
```

### A Simple Example: Magma

A magma is just a set with a binary operation:

```kleis
structure Magma(M) {
    operation (*) : M → M → M
}
```

That's it! No laws required.

---

## Adding Axioms

Axioms are properties that must hold. Let's build up:

### Semigroup (Associative Magma)

```kleis
structure Semigroup(S) {
    operation (*) : S → S → S
    
    axiom associativity :
        ∀(x y z : S). (x * y) * z = x * (y * z)
}
```

### Monoid (Semigroup with Identity)

```kleis
structure Monoid(M) {
    operation (*) : M → M → M
    operation e : M
    
    axiom associativity :
        ∀(x y z : M). (x * y) * z = x * (y * z)
    
    axiom left_identity :
        ∀(x : M). e * x = x
    
    axiom right_identity :
        ∀(x : M). x * e = x
}
```

### Group (Monoid with Inverses)

```kleis
structure Group(G) {
    operation (*) : G → G → G
    operation e : G
    operation inv : G → G
    
    axiom associativity :
        ∀(x y z : G). (x * y) * z = x * (y * z)
    
    axiom identity :
        ∀(x : G). e * x = x ∧ x * e = x
    
    axiom inverse :
        ∀(x : G). x * inv(x) = e ∧ inv(x) * x = e
}
```

---

## Multiple Type Parameters

Structures can have multiple parameters:

```kleis
structure VectorSpace(V, F) {
    // V is the vector type, F is the scalar field
    operation (+) : V → V → V
    operation (*) : F → V → V      // scalar multiplication
    operation zero : V
    
    // Vector addition is a group
    axiom add_assoc : ∀(u v w : V). (u + v) + w = u + (v + w)
    axiom add_identity : ∀(v : V). v + zero = v
    
    // Scalar multiplication distributes
    axiom scalar_distrib : ∀(a : F). ∀(u v : V). a * (u + v) = a*u + a*v
}
```

---

## Operations with Different Arities

### Constants (Nullary)

```kleis
operation e : G           // Identity element
operation zero : R        // Zero element
operation one : R         // One element
```

### Unary Operations

```kleis
operation inv : G → G     // Inverse
operation negate : R → R  // Negation
operation norm : V → ℝ    // Norm
```

### Binary Operations

```kleis
operation (+) : R → R → R
operation (*) : R → R → R
operation (·) : V → V → F  // Dot product
```

---

## Operator Symbols

Operations can use symbols:

```kleis
structure Ring(R) {
    operation (+) : R → R → R
    operation (*) : R → R → R
    operation (-) : R → R → R
    operation zero : R
    operation one : R
    
    // ...axioms...
}
```

Common symbols:
- `(+)`, `(-)`, `(*)`, `(/)` — arithmetic
- `(·)`, `(×)`, `(⊗)` — products
- `(∘)` — composition
- `(≤)`, `(≥)` — ordering

---

## Real-World Structure: Ring

```kleis
structure Ring(R) {
    // Additive structure
    operation (+) : R → R → R
    operation zero : R
    operation negate : R → R
    
    // Multiplicative structure
    operation (*) : R → R → R
    operation one : R
    
    // Addition is an abelian group
    axiom add_assoc : ∀(a b c : R). (a + b) + c = a + (b + c)
    axiom add_comm : ∀(a b : R). a + b = b + a
    axiom add_identity : ∀(a : R). a + zero = a
    axiom add_inverse : ∀(a : R). a + negate(a) = zero
    
    // Multiplication is a monoid
    axiom mul_assoc : ∀(a b c : R). (a * b) * c = a * (b * c)
    axiom mul_identity : ∀(a : R). a * one = a ∧ one * a = a
    
    // Distributivity
    axiom left_distrib : ∀(a b c : R). a * (b + c) = a*b + a*c
    axiom right_distrib : ∀(a b c : R). (a + b) * c = a*c + b*c
}
```

---

## Nested Structures

You can define sub-structures:

```kleis
structure Ring(R) {
    // The additive group
    structure additive : Group(R) {
        operation (+) : R → R → R
        operation zero : R
        operation negate : R → R
    }
    
    // The multiplicative monoid
    structure multiplicative : Monoid(R) {
        operation (*) : R → R → R
        operation one : R
    }
    
    // Distributivity connects them
    axiom distrib : ∀(a b c : R). a * (b + c) = a*b + a*c
}
```

---

## Exercises

1. **Define** a structure `Pointed(P)` with just a distinguished element `point : P`

2. **Define** a structure `Eq(T)` with equality `(=) : T → T → Bool` and reflexivity axiom

3. **Define** a structure `Ord(T)` extending `Eq(T)` with `(≤) : T → T → Bool` and transitivity

4. **Write** the axiom for commutativity of addition in a structure

<details>
<summary>Solutions</summary>

```kleis
// 1.
structure Pointed(P) {
    operation point : P
}

// 2.
structure Eq(T) {
    operation (=) : T → T → Bool
    
    axiom reflexivity : ∀(x : T). x = x
    axiom symmetry : ∀(x y : T). x = y ⟹ y = x
    axiom transitivity : ∀(x y z : T). x = y ∧ y = z ⟹ x = z
}

// 3.
structure Ord(T) extends Eq(T) {
    operation (≤) : T → T → Bool
    
    axiom transitivity : ∀(x y z : T). x ≤ y ∧ y ≤ z ⟹ x ≤ z
    axiom antisymmetry : ∀(x y : T). x ≤ y ∧ y ≤ x ⟹ x = y
    axiom totality : ∀(x y : T). x ≤ y ∨ y ≤ x
}

// 4.
axiom add_comm : ∀(a b : R). a + b = b + a
```

</details>

---

## Summary

- Structures define abstract mathematical concepts
- They contain **operations** (the interface) and **axioms** (the laws)
- Type parameters make structures generic
- Operations can be constants, unary, binary, etc.
- Operator symbols make definitions readable
- Nested structures organize related concepts

---

[← Previous: Conditionals](09-conditionals.md) | [Back to Contents](../index.md) | [Next: Implements →](11-implements.md)

