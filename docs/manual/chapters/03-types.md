# Chapter 3: Types and Type Annotations

[← Previous: Starting Out](02-starting-out.md) | [Back to Contents](../index.md) | [Next: Functions →](04-functions.md)

---

## Why Types Matter

Imagine you're writing a physics simulation and accidentally add a velocity vector to a position scalar. In most languages, this would silently produce garbage. In Kleis, it's a type error.

Types are your first line of defense against bugs.

---

## Basic Types

Kleis has several built-in types:

| Type | Description | Examples |
|------|-------------|----------|
| `ℝ` (or `Real`) | Real numbers | `3.14`, `-2.5`, `0` |
| `ℤ` (or `Int`) | Integers | `42`, `-7`, `0` |
| `ℕ` (or `Nat`) | Natural numbers | `0`, `1`, `100` |
| `Bool` | Booleans | `True`, `False` |

```kleis
// These all have types:
5           // ℤ (integer)
3.14        // ℝ (real)
True        // Bool
```

---

## Type Annotations in Let Bindings

You can annotate variables with their types:

```kleis
// Without annotation (type inferred)
let x = 5 in x + x

// With annotation (explicit type)
let x : ℝ = 5 in x + x
```

The syntax is `let name : Type = value in body`.

### Why Annotate?

1. **Documentation** — Makes code clearer
2. **Disambiguation** — When inference is ambiguous
3. **Error catching** — Ensures you meant what you wrote

```kleis
// Clear intent: x is explicitly a real number
let x : ℝ = 5 in sqrt(x)

// Catch mistakes early
let x : ℤ = 3.14 in x  // Error! 3.14 is not an integer
```

---

## Type Ascription (Haskell-style)

Beyond let bindings, you can annotate any expression with a type:

```kleis
// Annotate a simple expression
x : ℝ

// Annotate a complex expression
(a + b) : ℝ

// Annotate a function result
sqrt(x) : ℝ
```

The syntax is `(expression) : Type`.

### When to Use Ascription

Type ascription is useful when:

1. **Type inference is ambiguous**

```kleis
// Which numeric type is 0?
let zero : ℝ = 0 in zero + 1.5  // Clear: real
let zero : ℤ = 0 in zero + 1    // Clear: integer
```

2. **You want to document intent**

```kleis
// Reader knows the result should be real
define energy(m, v) = (0.5 * m * v^2) : ℝ
```

3. **You're debugging type errors**

```kleis
// Add ascription to narrow down where types go wrong
let result = (complicated_expression) : ExpectedType in ...
```

---

## Parametric Types

Some types take parameters:

```kleis
Vector(3)           // 3-dimensional vector
Matrix(3, 3, ℝ)     // 3×3 matrix of reals
List(ℤ)             // List of integers
Option(ℝ)           // Optional real value
```

### Using Parametric Types

```kleis
// A 3D vector
let v : Vector(3) = [1, 2, 3] in norm(v)

// A 2×2 matrix
let M : Matrix(2, 2, ℝ) = [[1, 0], [0, 1]] in det(M)

// An optional value
let maybe_x : Option(ℝ) = Some(5.0) in 
    match maybe_x {
        None => 0
        Some(x) => x
    }
```

---

## Type Variables

Sometimes you want to write code that works with *any* type. Use type variables:

```kleis
// Works for any type T
define identity(x : T) = x

>>> identity(5)
5

>>> identity(True)
True
```

Type variables are typically single letters: `T`, `A`, `B`, `M`, etc.

### Constraints on Type Variables

You can constrain type variables:

```kleis
// T must be a numeric type
define double(x : T) where Num(T) = x + x

// G must be a group
define square_group(x : G) where Group(G) = x + x
```

We'll explore constraints more in the chapter on structures.

---

## The Option Type

`Option(T)` represents a value that might not exist:

```kleis
data Option(T) = None | Some(T)
```

- `None` — No value present
- `Some(x)` — Value `x` is present

```kleis
// Safe division: might fail if divisor is zero
define safe_div(a, b) =
    if b = 0 then None
    else Some(a / b)

>>> safe_div(10, 2)
Some(5)

>>> safe_div(10, 0)
None
```

---

## The Bool Type

`Bool` has exactly two values:

```kleis
data Bool = True | False
```

Used with comparisons and conditionals:

```kleis
>>> 5 > 3
True

>>> 2 = 3
False

>>> if True then 1 else 0
1
```

---

## Type Errors

When types don't match, Kleis tells you:

```kleis
>>> let x : ℤ = 3.14 in x
Error: Type mismatch
  Expected: ℤ
  Found: ℝ
  
>>> True + 5
Error: Cannot apply (+) to Bool and ℤ
```

These errors are your friends! They catch bugs before runtime.

---

## Type Inference

Most of the time, you don't need type annotations. Kleis infers types:

```kleis
// Kleis knows these types automatically
define square(x) = x * x          // x : T where Num(T), result : T
define not(b) = if b then False else True  // b : Bool, result : Bool
```

Add annotations when:
- The code is clearer with them
- Inference can't figure it out
- You want to catch errors early

---

## Summary

In this chapter, you learned:

- Basic types: `ℝ`, `ℤ`, `ℕ`, `Bool`
- Type annotations in let: `let x : ℝ = 5 in ...`
- Type ascription: `(expr) : Type`
- Parametric types: `Vector(3)`, `Matrix(m, n, T)`
- Type variables: `T`, `A`, `B`
- `Option(T)` for optional values
- How type errors help catch bugs

Next, we'll dive deeper into functions!

---

[← Previous: Starting Out](02-starting-out.md) | [Back to Contents](../index.md) | [Next: Functions →](04-functions.md)

