# Let Bindings

## Introduction

Let bindings introduce local variables with limited scope. They're essential for breaking complex expressions into readable parts.

```kleis
define square_five = let x = 5 in x * x
// Result: 25
```

## Basic Syntax

```kleis
let <name> = <value> in <body>
```

The variable `name` is only visible within `body`:

```kleis
define circle_area = let radius = 10 in π * radius^2
// Result: 314.159...
// 'radius' is not visible outside the let binding
```

## With Type Annotations

Add explicit types for clarity:

```kleis
define typed_example1 = let x : ℝ = 3.14 in x * 2
define typed_example2 = let n : ℕ = 42 in factorial(n)
define typed_example3 = let v : Vector(3) = [1, 2, 3] in magnitude(v)
```

## Nested Let Bindings

Chain multiple bindings:

```kleis
define nested_example =
    let x = 5 in
    let y = 3 in
    let z = x + y in
        x * y * z
// Result: 5 * 3 * 8 = 120
```

## Shadowing

Inner bindings can shadow outer ones:

```kleis
define shadowing_example =
    let x = 1 in
    let x = x + 1 in
    let x = x * 2 in
        x
// Result: 4  (not 1!)
```

Each `let` creates a new scope where `x` is rebound.

## Pure Substitution Semantics

In Kleis, `let x = e in body` is equivalent to substituting `e` for `x` in `body`:

```kleis
define substitution_demo = let x = 5 in x + x
// is the same as:
define substitution_result = 5 + 5
```

This is **pure functional semantics** — no mutation, no side effects.

## Practical Examples

### Quadratic Formula

```kleis
define quadratic_roots(a, b, c) =
    let discriminant = b^2 - 4*a*c in
    let sqrt_d = sqrt(discriminant) in
    let denom = 2 * a in
        Pair((-b + sqrt_d) / denom, (-b - sqrt_d) / denom)
```

### Heron's Formula

```kleis
define triangle_area(a, b, c) =
    let s = (a + b + c) / 2 in
        sqrt(s * (s - a) * (s - b) * (s - c))
```

### Complex Calculations

```kleis
define schwarzschild_metric(r, M) =
    let rs = 2 * G * M / c^2 in
    let factor = 1 - rs / r in
        -c^2 * factor
```

## Let vs Define

| `define` | `let ... in` |
|----------|--------------|
| Top-level, global | Local scope only |
| Named function/constant | Temporary binding |
| Visible everywhere | Visible only in body |

```kleis
// Global constant
define pi = 3.14159

// Local temporary in a function
define circumference(radius) = let two_pi = 2 * pi in two_pi * radius
```

## What's Next?

Learn about quantifiers and logic!

→ [Next: Quantifiers and Logic](./07-quantifiers-logic.md)
