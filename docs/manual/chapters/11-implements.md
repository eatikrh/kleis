# Chapter 11: Implements, Where, and Extends

[← Previous: Structures](10-structures.md) | [Back to Contents](../index.md) | [Next: Z3 Verification →](12-z3-verification.md)

---

## Implements Blocks

An `implements` block says "this type satisfies this structure" and provides the concrete implementations.

```kleis
implements StructureName(ConcreteType) {
    // Provide concrete definitions for operations
    define operation = implementation
}
```

### Example: Integers as a Monoid

```kleis
structure Monoid(M) {
    operation (*) : M → M → M
    operation e : M
    axiom assoc : ∀(x y z : M). (x * y) * z = x * (y * z)
    axiom identity : ∀(x : M). e * x = x ∧ x * e = x
}

// Integers under addition form a monoid
implements Monoid(ℤ) {
    define (*)(a, b) = a + b    // "multiplication" is addition
    define e = 0                // identity is zero
}
```

### Example: Reals as a Group

```kleis
structure Group(G) {
    operation (+) : G → G → G
    operation zero : G
    operation negate : G → G
    // ... axioms ...
}

implements Group(ℝ) {
    define (+)(a, b) = a + b
    define zero = 0
    define negate(x) = -x
}
```

---

## Where Clauses

`where` clauses add **constraints** — requirements that must be satisfied.

### Basic Where

```kleis
implements VectorSpace(Vector(n), ℝ) where Nat(n) {
    // Vector(n) is a vector space over ℝ for any natural number n
}
```

### Multiple Constraints

```kleis
implements Eq(Pair(A, B)) where Eq(A), Eq(B) {
    // Pairs are equal if both components are equal
    define (=)(p1, p2) =
        fst(p1) = fst(p2) ∧ snd(p1) = snd(p2)
}
```

### Dependent Constraints

```kleis
implements Monoid(List(T)) where Eq(T) {
    define (*) = append
    define e = Nil
}
```

---

## Where in Functions

Functions can also have constraints:

```kleis
define double(x : G) where Group(G) = x + x

define sum(xs : List(M)) where Monoid(M) =
    match xs {
        Nil => e
        Cons(h, t) => h * sum(t)
    }
```

This means:
- `double` works for any type `G` that implements `Group`
- `sum` works for any type `M` that implements `Monoid`

---

## Extends

`extends` says one structure **inherits from** another:

```kleis
structure Monoid(M) extends Semigroup(M) {
    operation e : M
    axiom identity : ∀(x : M). e * x = x ∧ x * e = x
}
```

This means:
- A Monoid has everything a Semigroup has
- Plus the identity element `e` and identity axiom
- Any Monoid is automatically a Semigroup

### The Algebraic Hierarchy

```kleis
structure Magma(M) {
    operation (*) : M → M → M
}

structure Semigroup(S) extends Magma(S) {
    axiom assoc : ∀(x y z : S). (x * y) * z = x * (y * z)
}

structure Monoid(M) extends Semigroup(M) {
    operation e : M
    axiom identity : ∀(x : M). e * x = x ∧ x * e = x
}

structure Group(G) extends Monoid(G) {
    operation inv : G → G
    axiom inverse : ∀(x : G). x * inv(x) = e ∧ inv(x) * x = e
}

structure AbelianGroup(A) extends Group(A) {
    axiom commutativity : ∀(x y : A). x * y = y * x
}
```

### Diagram

```
    Magma
      ↓
  Semigroup    (+ associativity)
      ↓
   Monoid      (+ identity)
      ↓
   Group       (+ inverse)
      ↓
AbelianGroup   (+ commutativity)
```

---

## Multiple Inheritance

Structures can extend multiple parents:

```kleis
structure Ring(R) extends AbelianGroup(R), Monoid(R) {
    // Gets (+) from AbelianGroup
    // Gets (*) from Monoid
    axiom distrib : ∀(a b c : R). a * (b + c) = a*b + a*c
}
```

### Diamond Problem

What if two parents define the same operation? Kleis requires explicit resolution:

```kleis
structure Ring(R) extends AbelianGroup(R), Monoid(R) {
    // AbelianGroup uses (+) with identity 0
    // Monoid uses (*) with identity 1
    // They're different operations, no conflict!
}
```

---

## Implements with Where

```kleis
// Matrix(n, n, F) is a Ring when F is a Field
implements Ring(Matrix(n, n, F)) where Field(F), Nat(n) {
    define (+)(A, B) = matrix_add(A, B)
    define (*)(A, B) = matrix_mul(A, B)
    define zero = zero_matrix(n, n)
    define one = identity_matrix(n)
    define negate(A) = matrix_negate(A)
}
```

---

## Built-in Type Classes

Kleis has some built-in type classes:

```kleis
// Numeric types
structure Num(N) {
    operation (+) : N → N → N
    operation (-) : N → N → N
    operation (*) : N → N → N
    operation negate : N → N
}

// Equality
structure Eq(T) {
    operation (=) : T → T → Bool
    operation (≠) : T → T → Bool
}

// Ordering
structure Ord(T) extends Eq(T) {
    operation (<) : T → T → Bool
    operation (≤) : T → T → Bool
    operation (>) : T → T → Bool
    operation (≥) : T → T → Bool
}
```

---

## Exercises

1. **Write** an `implements` block for `ℤ` as a `Group` under addition

2. **Write** a function `square(x)` that works for any `Num(T)`

3. **Define** a structure `Lattice(L)` with meet and join operations

4. **Write** `implements Eq(Option(T)) where Eq(T)`

<details>
<summary>Solutions</summary>

```kleis
// 1.
implements Group(ℤ) {
    define (+)(a, b) = a + b
    define zero = 0
    define negate(x) = -x
}

// 2.
define square(x : T) where Num(T) = x * x

// 3.
structure Lattice(L) {
    operation meet : L → L → L  // greatest lower bound
    operation join : L → L → L  // least upper bound
    
    axiom meet_comm : ∀(a b : L). meet(a, b) = meet(b, a)
    axiom join_comm : ∀(a b : L). join(a, b) = join(b, a)
    axiom meet_assoc : ∀(a b c : L). meet(meet(a,b), c) = meet(a, meet(b,c))
    axiom absorption : ∀(a b : L). meet(a, join(a, b)) = a
}

// 4.
implements Eq(Option(T)) where Eq(T) {
    define (=)(a, b) =
        match (a, b) {
            (None, None) => True
            (Some(x), Some(y)) => x = y
            _ => False
        }
    define (≠)(a, b) = ¬(a = b)
}
```

</details>

---

## Summary

- `implements` provides concrete implementations for structures
- `where` adds constraints (type requirements)
- `extends` creates structure hierarchies
- Structures can extend multiple parents
- Where clauses work on both implements and functions
- This enables generic programming with mathematical constraints

---

[← Previous: Structures](10-structures.md) | [Back to Contents](../index.md) | [Next: Z3 Verification →](12-z3-verification.md)

