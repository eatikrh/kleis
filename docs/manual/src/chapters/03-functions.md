# Functions

## Defining Functions

Functions are defined with `define`:

```text
define square(x) = x * x
define cube(x) = x * x * x
define add(x, y) = x + y
```

## Functions with Type Annotations

For clarity and safety, add type annotations:

```text
define square(x : ℝ) : ℝ = x * x

define distance(x : ℝ, y : ℝ) : ℝ = sqrt(x^2 + y^2)

define normalize(v : Vector(3)) : Vector(3) = v / magnitude(v)
```

## Multi-Parameter Functions

Functions can take multiple parameters:

```text
define add(x, y) = x + y
define volume_box(l, w, h) = l * w * h
define dot_product(a, b, c, x, y, z) = a*x + b*y + c*z
```

## Recursive Functions

Functions can call themselves:

```text
define factorial(n : ℕ) : ℕ =
    if n = 0 then 1
    else n * factorial(n - 1)

define fibonacci(n : ℕ) : ℕ =
    if n ≤ 1 then n
    else fibonacci(n-1) + fibonacci(n-2)
```

## Built-in Mathematical Functions

Kleis includes standard mathematical functions:

### Trigonometric
```text
sin(x)      cos(x)      tan(x)
asin(x)     acos(x)     atan(x)
sinh(x)     cosh(x)     tanh(x)
```

### Exponential and Logarithmic
```text
exp(x)      // e^x
ln(x)       // natural log
log(x)      // base-10 log
log(b, x)   // log base b of x
```

### Other
```text
sqrt(x)     // square root
abs(x)      // absolute value
floor(x)    // round down
ceil(x)     // round up
min(x, y)   // minimum
max(x, y)   // maximum
```

## Lambda Expressions (Anonymous Functions)

Lambda expressions allow you to create anonymous functions inline:

```text
define square_lambda = λ x . x * x
define increment = lambda x . x + 1
define add_lambda = λ x . λ y . x + y
define square_typed = λ (x : ℝ) . x^2
define curried_add = λ x . λ y . x + y
```

Lambda expressions are first-class values - you can pass them to functions:

```text
// Pass lambda to higher-order function
define doubled_list = map(λ x . x * 2, [1, 2, 3])

// Or define inline
define result = apply(λ x . x + 1, 5)
```

## Higher-Order Functions

Functions can take functions as arguments:

```text
// Apply a function twice
define apply_twice(f, x) = f(f(x))

// Example usage:
define inc(x) = x + 1
define result = apply_twice(inc, 5)   // Result: 7
```

## Partial Application and Currying

With lambda expressions, you can create curried functions:

```text
// Curried addition
define add = λ x . λ y . x + y

// Partial application creates specialized functions
define add5 = add(5)           // λ y . 5 + y
define eight = add5(3)         // Result: 8
```

## What's Next?

Learn about algebraic data types for structured data!

→ [Next: Algebraic Types](./04-algebraic-types.md)
