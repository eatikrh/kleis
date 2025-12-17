# Functions

## Defining Functions

Functions are defined with `define`:

```kleis
define square(x) = x * x
define cube(x) = x * x * x
define add(x, y) = x + y
```

## Functions with Type Annotations

For clarity and safety, add type annotations:

```kleis
define square(x : ℝ) : ℝ = x * x

define distance(x : ℝ, y : ℝ) : ℝ = sqrt(x^2 + y^2)

define normalize(v : Vector(3)) : Vector(3) = v / magnitude(v)
```

## Multi-Parameter Functions

Functions can take multiple parameters:

```kleis
define add(x, y) = x + y
define volume_box(l, w, h) = l * w * h
define dot_product(a, b, c, x, y, z) = a*x + b*y + c*z
```

## Recursive Functions

Functions can call themselves:

```kleis
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
```kleis
sin(x)      cos(x)      tan(x)
asin(x)     acos(x)     atan(x)
sinh(x)     cosh(x)     tanh(x)
```

### Exponential and Logarithmic
```kleis
exp(x)      -- e^x
ln(x)       -- natural log
log(x)      -- base-10 log
log(b, x)   -- log base b of x
```

### Other
```kleis
sqrt(x)     -- square root
abs(x)      -- absolute value
floor(x)    -- round down
ceil(x)     -- round up
min(x, y)   -- minimum
max(x, y)   -- maximum
```

## Lambda Expressions (Anonymous Functions)

> 🚧 **Coming Soon: We're working on it!**
>
> Lambda expressions allow you to create anonymous functions inline.
> This feature is planned but not yet implemented in the parser.

Lambda expressions will let you write:

```kleis
-- Planned syntax (not yet working):
λ x . x * x           -- square function
\x -> x + 1           -- increment (ASCII alternative)
λ x . λ y . x + y     -- curried addition
```

**Current workaround:** Use named functions with `define`:

```kleis
-- Instead of: map(λ x . x * 2, list)
-- Define a helper:
define double(x) = x * 2
-- Then use it: map(double, list)
```

## Higher-Order Functions

Functions can take functions as arguments:

```kleis
-- Apply a function twice
define apply_twice(f, x) = f(f(x))

-- Example usage:
define inc(x) = x + 1
apply_twice(inc, 5)   -- Result: 7
```

## Partial Application

> 🚧 **Coming Soon**
>
> Partial application (currying) will be supported once lambda
> expressions are implemented.

## What's Next?

Learn about algebraic data types for structured data!

→ [Next: Algebraic Types](./04-algebraic-types.md)
