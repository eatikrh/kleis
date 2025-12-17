# Chapter 7: Let Bindings

[← Previous: Pattern Matching](06-pattern-matching.md) | [Back to Contents](../index.md) | [Next: Quantifiers →](08-quantifiers-logic.md)

---

## The Let Expression

`let` bindings introduce local variables with a specific scope.

```kleis
let x = value in body
```

Read this as: "Let x equal value, then evaluate body (where x is available)."

---

## Basic Syntax

```kleis
>>> let x = 5 in x + x
10

>>> let pi = 3.14159 in 2 * pi * 10
62.8318

>>> let name = "Kleis" in name
"Kleis"
```

---

## What Does "in" Mean?

The `in` keyword separates the **binding** from the **scope**:

```
let x = 5 in x + x
│        │   └───── body: x is available here
│        └──── "in" = "within the following expression"
└──────────────── binding: x = 5
```

Think of it as: "Define x as 5, **in** (the context of) x + x"

---

## Scoping Rules

### The Bound Variable is Local

The variable only exists within the body:

```kleis
>>> let x = 5 in x + 1
6

>>> x  // Error! x is not defined here
Error: Unknown variable 'x'
```

### Nested Let Bindings

Each `in` introduces a new scope:

```kleis
>>> let x = 5 in let y = 3 in x * y
15

// Equivalent to:
>>> let x = 5 in (let y = 3 in (x * y))
15
```

Inner bindings can see outer bindings:

```kleis
>>> let a = 10 in
    let b = a + 5 in
    let c = a + b in
    c
25
```

### Shadowing

Inner bindings can shadow outer ones:

```kleis
>>> let x = 1 in let x = 2 in x
2  // Inner x shadows outer x

>>> let x = 1 in let x = x + 1 in x
2  // The inner x = (outer x) + 1
```

---

## Type Annotations

Add type annotations for clarity:

```kleis
>>> let x : ℝ = 5 in x^2
25

>>> let v : Vector(3) = [1, 2, 3] in norm(v)
3.7416...

>>> let f : ℝ → ℝ = (\x. x^2) in f(5)
25
```

The syntax is: `let name : Type = value in body`

---

## Complex Bodies

The body after `in` can be arbitrarily complex:

```kleis
// Body is a conditional
let threshold = 100 in
    if x > threshold then "high" else "low"

// Body is another let
let dx = x2 - x1 in
let dy = y2 - y1 in
    sqrt(dx^2 + dy^2)

// Body is a function call
let coefficients = [1, -3, 2] in
    solve_quadratic(coefficients)

// Body is a match expression
let result = compute() in
    match result {
        None => 0
        Some(x) => x
    }
```

---

## In Functions

Let bindings are perfect for intermediate calculations:

```kleis
define quadratic_formula(a, b, c) =
    let discriminant = b^2 - 4*a*c in
    let sqrt_disc = sqrt(discriminant) in
    let two_a = 2 * a in
    (-b + sqrt_disc) / two_a

define normalize(v : Vector(3)) =
    let n = norm(v) in
    [v[0]/n, v[1]/n, v[2]/n]
```

---

## Pure Substitution Semantics

Let bindings have **pure substitution** semantics:

```kleis
let x = expr in body
```

is equivalent to:

```kleis
body[x := expr]  // Replace every x in body with expr
```

This means:

```kleis
let x = 5 in x + x
// Same as: 5 + 5 = 10

let f = (\x. x^2) in f(3)
// Same as: (\x. x^2)(3) = 3^2 = 9
```

---

## Comparison to Other Languages

| Language | Syntax | Notes |
|----------|--------|-------|
| **Kleis** | `let x = 5 in x + x` | Expression-based |
| **Haskell** | `let x = 5 in x + x` | Identical |
| **OCaml** | `let x = 5 in x + x` | Identical |
| **Rust** | `{ let x = 5; x + x }` | Statement, needs block |
| **Python** | N/A | No direct equivalent |
| **Math** | `x + x where x = 5` | Definition comes last |

---

## Exercises

1. **Evaluate** `let a = 3 in let b = a + 1 in a * b`

2. **Write** an expression using let that computes (a + b)² using the identity a² + 2ab + b²

3. **Rewrite** this without nested lets:
   ```kleis
   let x = 5 in let y = x + 1 in let z = y + 1 in x + y + z
   ```

4. **What's the value of:** `let x = 1 in let x = x + x in let x = x + x in x`

<details>
<summary>Solutions</summary>

```kleis
// 1.
>>> let a = 3 in let b = a + 1 in a * b
12  // a=3, b=4, so 3*4=12

// 2.
let a = 3 in
let b = 4 in
let a_sq = a * a in
let b_sq = b * b in
let two_ab = 2 * a * b in
a_sq + two_ab + b_sq
// = 9 + 24 + 16 = 49 = (3+4)² ✓

// 3. Without nested lets, we'd need to inline:
5 + (5 + 1) + ((5 + 1) + 1)
// = 5 + 6 + 7 = 18

// 4.
>>> let x = 1 in let x = x + x in let x = x + x in x
4  // x=1 → x=2 → x=4
```

</details>

---

## Summary

- `let x = value in body` introduces a local binding
- `in` marks where the binding is available
- Variables are only visible in their scope
- Inner bindings can shadow outer ones
- Type annotations: `let x : Type = value in body`
- Let has pure substitution semantics

---

[← Previous: Pattern Matching](06-pattern-matching.md) | [Back to Contents](../index.md) | [Next: Quantifiers →](08-quantifiers-logic.md)

