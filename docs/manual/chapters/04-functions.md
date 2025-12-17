# Chapter 4: Functions

[← Previous: Types](03-types.md) | [Back to Contents](../index.md) | [Next: Algebraic Types →](05-algebraic-types.md)

---

## Defining Functions

Functions are the heart of Kleis. Define them with the `define` keyword:

```kleis
define name(parameters) = body
```

### Simple Functions

```kleis
define square(x) = x * x

define double(x) = x + x

define cube(x) = x * x * x
```

### Multiple Parameters

```kleis
define add(a, b) = a + b

define multiply(a, b) = a * b

define distance(x1, y1, x2, y2) = sqrt((x2-x1)^2 + (y2-y1)^2)
```

### With Type Annotations

```kleis
define square(x : ℝ) : ℝ = x * x

define dot(u : Vector(3), v : Vector(3)) : ℝ = 
    u[0]*v[0] + u[1]*v[1] + u[2]*v[2]
```

---

## Function Calls

Call functions with parentheses:

```kleis
>>> square(5)
25

>>> add(3, 4)
7

>>> distance(0, 0, 3, 4)
5
```

### Nested Calls

```kleis
>>> square(square(2))
16

>>> double(triple(5))   // assuming triple is defined
30

>>> add(square(2), cube(3))
31
```

---

## Operators as Functions

In Kleis, operators like `+` and `*` are just functions with special syntax.

### Defining Operators

```kleis
// Define a custom operator
define (⊕)(a, b) = a + b + 1

>>> 3 ⊕ 4
8
```

### Using Operators as Prefix

```kleis
// Operators can be called as functions
>>> (+)(3, 4)
7

>>> (*)(5, 6)
30
```

---

## Local Definitions with Let

Use `let` to define local variables within a function:

```kleis
define quadratic_formula(a, b, c) =
    let discriminant = b^2 - 4*a*c in
    let sqrt_disc = sqrt(discriminant) in
    (-b + sqrt_disc) / (2*a)
```

### Nested Let

```kleis
define surface_area_cylinder(r, h) =
    let pi = 3.14159 in
    let circle_area = pi * r^2 in
    let side_area = 2 * pi * r * h in
    2 * circle_area + side_area
```

### Typed Let

```kleis
define normalize(v : Vector(3)) =
    let n : ℝ = norm(v) in
    [v[0]/n, v[1]/n, v[2]/n]
```

---

## Recursion

Functions can call themselves:

```kleis
define factorial(n) =
    if n = 0 then 1
    else n * factorial(n - 1)

>>> factorial(5)
120
```

### Recursive Examples

```kleis
// Fibonacci
define fib(n) =
    if n ≤ 1 then n
    else fib(n-1) + fib(n-2)

>>> fib(10)
55

// Power function
define power(base, exp) =
    if exp = 0 then 1
    else base * power(base, exp - 1)

>>> power(2, 10)
1024
```

---

## Higher-Order Functions

Functions can take other functions as arguments:

```kleis
define apply_twice(f, x) = f(f(x))

>>> apply_twice(square, 2)
16  // square(square(2)) = square(4) = 16

define compose(f, g, x) = f(g(x))

>>> compose(double, square, 3)
18  // double(square(3)) = double(9) = 18
```

---

## Anonymous Functions (Lambdas)

> 🚧 **Coming Soon:** Lambda expressions are currently being implemented.
> The syntax below shows what's coming — we're working on it!

Lambda expressions (also called anonymous functions) let you create functions without giving them a name. Use `λ` or `\`:

```kleis
\x. x + 1           // A function that adds 1
λx. x * x           // A function that squares (using Unicode λ)
\x y. x + y         // Multiple parameters
```

### How Lambdas Work

The syntax `\x. body` means "a function that takes `x` and returns `body`":

```kleis
(\x. x + 1)(5)      // Apply to 5 → returns 6
(λx. x * x)(4)      // Apply to 4 → returns 16
(\x y. x + y)(3, 4) // Apply to 3, 4 → returns 7
```

### Why Lambdas Matter

Lambdas are essential for higher-order functions:

```kleis
// Pass a function to map
map(\x. x * 2, [1, 2, 3])    // → [2, 4, 6]

// Create functions on the fly
let double = \x. x + x in double(5)  // → 10

// Function composition
let f = \x. x + 1 in
let g = \x. x * 2 in
f(g(3))  // → 7
```

### Current Workaround

Until lambdas are implemented, use named functions:

```kleis
// Instead of: map(\x. x * 2, xs)
// Use:
define double(x) = x * 2
map(double, xs)
```

---

## Function Composition

Compose functions with `∘`:

```kleis
>>> let f = square in let g = double in (f ∘ g)(3)
36  // square(double(3)) = square(6) = 36
```

---

## Exercises

1. **Define** `triple(x)` that returns `3 * x`

2. **Define** `average3(a, b, c)` that returns the average of three numbers

3. **Define** `is_even(n)` that returns `True` if n is even (hint: use `mod`)

4. **Define** `gcd(a, b)` using Euclid's algorithm (recursive)

5. **Define** `sum_to(n)` that returns `1 + 2 + ... + n`

<details>
<summary>Solutions</summary>

```kleis
// 1.
define triple(x) = 3 * x

// 2.
define average3(a, b, c) = (a + b + c) / 3

// 3.
define is_even(n) = mod(n, 2) = 0

// 4.
define gcd(a, b) =
    if b = 0 then a
    else gcd(b, mod(a, b))

// 5.
define sum_to(n) =
    if n = 0 then 0
    else n + sum_to(n - 1)
```

</details>

---

## Summary

- Functions are defined with `define name(params) = body`
- Functions can have type annotations
- Operators are functions with special syntax
- Use `let` for local definitions
- Recursion is natural and encouraged
- Higher-order functions take functions as arguments
- Lambda expressions create anonymous functions (coming soon!)

---

[← Previous: Types](03-types.md) | [Back to Contents](../index.md) | [Next: Algebraic Types →](05-algebraic-types.md)

