# Appendix C: Standard Library

[← Back to Contents](../index.md)

---

## Overview

Kleis includes a standard library (`stdlib/`) that provides:

- Built-in type definitions
- Common algebraic structures
- Mathematical functions
- Utility types like Option and List

The stdlib is written in Kleis itself, demonstrating the language's self-hosting capabilities.

---

## Core Types

### Numeric Types

```kleis
// stdlib/numeric.kleis

// Real numbers
type ℝ    // also: Real

// Integers  
type ℤ    // also: Int

// Natural numbers
type ℕ    // also: Nat

// Built-in operations
operation (+) : ℝ → ℝ → ℝ
operation (-) : ℝ → ℝ → ℝ
operation (*) : ℝ → ℝ → ℝ
operation (/) : ℝ → ℝ → ℝ
operation (^) : ℝ → ℝ → ℝ
```

### Boolean

```kleis
// stdlib/bool.kleis

data Bool = True | False

define not(b : Bool) : Bool =
    match b {
        True => False
        False => True
    }

define (∧)(a : Bool, b : Bool) : Bool =
    match a {
        False => False
        True => b
    }

define (∨)(a : Bool, b : Bool) : Bool =
    match a {
        True => True
        False => b
    }
```

### Option

```kleis
// stdlib/option.kleis

data Option(T) = None | Some(T)

define is_some(opt : Option(T)) : Bool =
    match opt {
        None => False
        Some(_) => True
    }

define is_none(opt : Option(T)) : Bool =
    match opt {
        None => True
        Some(_) => False
    }

define unwrap_or(opt : Option(T), default : T) : T =
    match opt {
        None => default
        Some(x) => x
    }

define map(f : A → B, opt : Option(A)) : Option(B) =
    match opt {
        None => None
        Some(x) => Some(f(x))
    }
```

### List

```kleis
// stdlib/list.kleis

data List(T) = Nil | Cons(T, List(T))

define length(xs : List(T)) : ℕ =
    match xs {
        Nil => 0
        Cons(_, t) => 1 + length(t)
    }

define head(xs : List(T)) : Option(T) =
    match xs {
        Nil => None
        Cons(h, _) => Some(h)
    }

define tail(xs : List(T)) : List(T) =
    match xs {
        Nil => Nil
        Cons(_, t) => t
    }

define map(f : A → B, xs : List(A)) : List(B) =
    match xs {
        Nil => Nil
        Cons(h, t) => Cons(f(h), map(f, t))
    }

define filter(pred : T → Bool, xs : List(T)) : List(T) =
    match xs {
        Nil => Nil
        Cons(h, t) =>
            if pred(h) then Cons(h, filter(pred, t))
            else filter(pred, t)
    }

define fold(f : A → B → B, init : B, xs : List(A)) : B =
    match xs {
        Nil => init
        Cons(h, t) => f(h, fold(f, init, t))
    }

define append(xs : List(T), ys : List(T)) : List(T) =
    match xs {
        Nil => ys
        Cons(h, t) => Cons(h, append(t, ys))
    }

define reverse(xs : List(T)) : List(T) =
    fold(\x acc. Cons(x, acc), Nil, xs)
```

---

## Algebraic Structures

### Semigroup

```kleis
// stdlib/algebra/semigroup.kleis

structure Semigroup(S) {
    operation (<>) : S → S → S
    
    axiom associativity :
        ∀(x y z : S). (x <> y) <> z = x <> (y <> z)
}
```

### Monoid

```kleis
// stdlib/algebra/monoid.kleis

structure Monoid(M) extends Semigroup(M) {
    operation mempty : M
    
    axiom left_identity : ∀(x : M). mempty <> x = x
    axiom right_identity : ∀(x : M). x <> mempty = x
}

// List is a Monoid under append
implements Monoid(List(T)) {
    define (<>) = append
    define mempty = Nil
}
```

### Group

```kleis
// stdlib/algebra/group.kleis

structure Group(G) extends Monoid(G) {
    operation inverse : G → G
    
    axiom left_inverse : ∀(x : G). inverse(x) <> x = mempty
    axiom right_inverse : ∀(x : G). x <> inverse(x) = mempty
}
```

### Ring

```kleis
// stdlib/algebra/ring.kleis

structure Ring(R) {
    operation (+) : R → R → R
    operation (*) : R → R → R
    operation zero : R
    operation one : R
    operation negate : R → R
    
    // Additive abelian group
    axiom add_assoc : ∀(a b c : R). (a + b) + c = a + (b + c)
    axiom add_comm : ∀(a b : R). a + b = b + a
    axiom add_identity : ∀(a : R). a + zero = a
    axiom add_inverse : ∀(a : R). a + negate(a) = zero
    
    // Multiplicative monoid
    axiom mul_assoc : ∀(a b c : R). (a * b) * c = a * (b * c)
    axiom mul_identity : ∀(a : R). a * one = a ∧ one * a = a
    
    // Distributivity
    axiom distrib : ∀(a b c : R). a * (b + c) = a*b + a*c
}
```

---

## Mathematical Functions

### Arithmetic

```kleis
// stdlib/math.kleis

define abs(x : ℝ) : ℝ = if x < 0 then -x else x

define sign(x : ℝ) : ℝ =
    if x > 0 then 1
    else if x < 0 then -1
    else 0

define max(a : ℝ, b : ℝ) : ℝ = if a > b then a else b

define min(a : ℝ, b : ℝ) : ℝ = if a < b then a else b

define clamp(x : ℝ, lo : ℝ, hi : ℝ) : ℝ =
    max(lo, min(x, hi))
```

### Transcendental Functions

```kleis
// Built-in (implemented in Rust)
operation sqrt : ℝ → ℝ
operation exp : ℝ → ℝ
operation log : ℝ → ℝ
operation sin : ℝ → ℝ
operation cos : ℝ → ℝ
operation tan : ℝ → ℝ
operation asin : ℝ → ℝ
operation acos : ℝ → ℝ
operation atan : ℝ → ℝ
operation atan2 : ℝ → ℝ → ℝ
```

### Constants

```kleis
// Constants are defined in user code per ADR-016
// Example:
define π : ℝ = 3.14159265358979
define e : ℝ = 2.71828182845904
define τ : ℝ = 2 * π
```

---

## Vectors and Matrices

### Vector

```kleis
// stdlib/linear/vector.kleis

structure Vector(n : ℕ, F) where Field(F) {
    operation (+) : Vector(n, F) → Vector(n, F) → Vector(n, F)
    operation (*) : F → Vector(n, F) → Vector(n, F)  // scalar mult
    operation zero : Vector(n, F)
    operation dot : Vector(n, F) → Vector(n, F) → F
}

define norm(v : Vector(n, ℝ)) : ℝ = sqrt(dot(v, v))

define normalize(v : Vector(n, ℝ)) : Vector(n, ℝ) =
    let n = norm(v) in
    (1/n) * v
```

### Matrix

```kleis
// stdlib/linear/matrix.kleis

structure Matrix(m : ℕ, n : ℕ, F) where Field(F) {
    operation (+) : Matrix(m,n,F) → Matrix(m,n,F) → Matrix(m,n,F)
    operation (*) : Matrix(m,k,F) → Matrix(k,n,F) → Matrix(m,n,F)
    operation transpose : Matrix(m,n,F) → Matrix(n,m,F)
    operation zero : Matrix(m,n,F)
    operation identity : Matrix(n,n,F)
}
```

---

## Using the Standard Library

### In the REPL

The stdlib is loaded automatically:

```kleis
>>> Some(5)
Some(5)

>>> map(\x. x + 1, [1, 2, 3])
[2, 3, 4]

>>> sqrt(16)
4
```

### In Files

```kleis
// my_file.kleis

// stdlib is implicitly available
define my_function(xs : List(ℤ)) =
    fold((+), 0, xs)
```

---

## File Locations

```
stdlib/
├── bool.kleis
├── option.kleis
├── list.kleis
├── numeric.kleis
├── math.kleis
├── algebra/
│   ├── semigroup.kleis
│   ├── monoid.kleis
│   ├── group.kleis
│   ├── ring.kleis
│   └── field.kleis
└── linear/
    ├── vector.kleis
    └── matrix.kleis
```

---

[← Back to Contents](../index.md)

