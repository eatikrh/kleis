# Let Bindings

## Introduction

Let bindings introduce local variables with limited scope. They're essential for breaking complex expressions into readable parts.

```kleis
let x = 5 in x * x
// Result: 25
```

## Basic Syntax

```kleis
let <name> = <value> in <body>
```

The variable `name` is only visible within `body`:

```kleis
let radius = 10 in
    π * radius^2
// Result: 314.159...

// 'radius' is not visible here!
```

## With Type Annotations

Add explicit types for clarity:

```kleis
let x : ℝ = 3.14 in x * 2
let n : ℕ = 42 in factorial(n)
let v : Vector(3) = [1, 2, 3] in magnitude(v)
```

## Nested Let Bindings

Chain multiple bindings:

```kleis
let x = 5 in
let y = 3 in
let z = x + y in
    x * y * z
// Result: 5 * 3 * 8 = 120
```

## Shadowing

Inner bindings can shadow outer ones:

```kleis
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
let x = 5 in x + x
// is the same as:
5 + 5
```

This is **pure functional semantics** — no mutation, no side effects.

## Practical Examples

### Quadratic Formula

```kleis
define quadratic_roots(a : ℝ, b : ℝ, c : ℝ) : (ℝ, ℝ) =
    let discriminant = b^2 - 4*a*c in
    let sqrt_d = sqrt(discriminant) in
    let denom = 2 * a in
        ((-b + sqrt_d) / denom, (-b - sqrt_d) / denom)
```

### Heron's Formula

```kleis
define triangle_area(a : ℝ, b : ℝ, c : ℝ) : ℝ =
    let s = (a + b + c) / 2 in  // semi-perimeter
        sqrt(s * (s - a) * (s - b) * (s - c))
```

### Complex Calculations

```kleis
define schwarzschild_metric(r : ℝ, M : ℝ) : ℝ =
    let rs = 2 * G * M / c^2 in      // Schwarzschild radius
    let factor = 1 - rs / r in
        -c^2 * factor                 // g_tt component
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

// Local temporary
let two_pi = 2 * pi in
    two_pi * radius
```

## What's Next?

Learn about quantifiers and logic!

→ [Next: Quantifiers and Logic](./07-quantifiers-logic.md)
