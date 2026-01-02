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
exp(x)      // e^x
ln(x)       // natural log
log(x)      // base-10 log
log(b, x)   // log base b of x
```

### Other
```kleis
sqrt(x)     // square root
abs(x)      // absolute value
floor(x)    // round down
ceil(x)     // round up
min(x, y)   // minimum
max(x, y)   // maximum
```

## Lambda Expressions (Anonymous Functions)

Lambda expressions allow you to create anonymous functions inline:

```kleis
define square_lambda = λ x . x * x
define increment = lambda x . x + 1
define add_lambda = λ x . λ y . x + y
define square_typed = λ (x : ℝ) . x^2
define curried_add = λ x . λ y . x + y
```

Lambda expressions are first-class values - you can pass them to functions:

```kleis
// Pass lambda to higher-order function
define doubled_list = map(λ x . x * 2, [1, 2, 3])

// Or define inline
define result = apply(λ x . x + 1, 5)
```

## Higher-Order Functions

Functions can take functions as arguments:

```kleis
// Apply a function twice
define apply_twice(f, x) = f(f(x))

// Example usage:
define inc(x) = x + 1
define result = apply_twice(inc, 5)   // Result: 7
```

## Partial Application and Currying

With lambda expressions, you can create curried functions:

```kleis
// Curried addition
define add = λ x . λ y . x + y

// Partial application creates specialized functions
define add5 = add(5)           // λ y . 5 + y
define eight = add5(3)         // Result: 8
```

## Named Arguments (v0.96)

For plotting and numeric functions, Kleis supports named arguments (keyword arguments):

```kleis
// Named arguments come after positional arguments
diagram(
    bar(xs, ys, offset = -0.2, width = 0.4, label = "Data"),
    plot(x, y, color = "blue", yerr = errors),
    width = 10,
    height = 7,
    title = "My Chart"
)
```

### Syntax

Named arguments use `=` (not `==`) and must come after all positional arguments:

```kleis
// ✅ Valid: positional first, then named
f(a, b, x = 1, y = 2)

// ✅ Valid: all named
f(x = 1, y = 2)

// ❌ Invalid: positional after named
f(x = 1, a, b)      // Error!
```

### Parser Transformation

Named arguments are **syntactic sugar**. The parser transforms them into a `record` expression:

```kleis
// You write:
bar(xs, ys, offset = -0.2, width = 0.4)

// Parser produces:
bar(xs, ys, record(
    field("offset", -0.2),
    field("width", 0.4)
))
```

### Limitations: Numeric Only

> **Important:** Named arguments are designed for **concrete numeric computation** (plotting, configuration). They cannot be used in:

- `structure` definitions
- `axiom` declarations  
- `implements` blocks
- Z3 verification proofs

```kleis
// ❌ Does NOT work in axioms
structure Bad {
    axiom wrong: f(x = 1)  // ERROR: named args not for axioms
}

// ✅ Works in plotting/computation
let xs = [0, 1, 2, 3]
let ys = [10, 20, 15, 25]
diagram(bar(xs, ys, color = "blue"))
```

### Why This Design?

Named arguments are opaque to the type system:

1. **Type inference** sees `record` as an opaque type
2. **Unification** doesn't look inside records
3. **Z3** never receives record expressions
4. **Built-in functions** consume records at runtime

This ensures named arguments don't interfere with symbolic mathematics while providing convenient syntax for plotting and configuration.

## What's Next?

Learn about algebraic data types for structured data!

→ [Next: Algebraic Types](./04-algebraic-types.md)
