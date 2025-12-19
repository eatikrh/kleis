# Types and Values

## Why Types Matter

Types are the foundation of Kleis. Every expression has a type, and the type system catches errors before they become problems.

```kleis
define answer = 42                // 42 is an integer
define pi_val = 3.14              // 3.14 is a real number
define flag = True                // True is a boolean
```

## Built-in Types

### Numeric Types

| Type | Unicode | Full Name | ASCII | Examples |
|------|---------|-----------|-------|----------|
| Natural | `ℕ` | `Nat` | `N` | `0`, `42`, `100` |
| Integer | `ℤ` | `Int` | `Z` | `-5`, `0`, `17` |
| Real | `ℝ` | `Real` or `Scalar` | `R` | `3.14`, `-2.5`, `√2` |
| Complex | `ℂ` | `Complex` | `C` | `3 + 4i`, `i` |

### Other Basic Types

| Type | Unicode | Full Name | Values |
|------|---------|-----------|--------|
| Boolean | `𝔹` | `Bool` | `True`, `False` |
| Unit | — | `Unit` | `()` |

```kleis
// Boolean type annotations - all equivalent
axiom bool_unicode : ∀(p : 𝔹). p = p
axiom bool_full    : ∀(q : Bool). q = q

// Boolean values
define flag = True
define not_flag = False

// Unit value
define empty = ()
```

## Type Annotations

You can explicitly annotate types with `:`:

```kleis
// Variable annotation
define typed_let = let x : ℝ = 3.14 in x * 2

// Function parameter and return types
define f(x : ℝ) : ℝ = x * x

// Expression-level annotation (ascription)
define sum_typed(a, b) = (a + b) : ℝ
```

## Function Types

Functions have types too! The notation `A → B` means "a function from A to B":

```kleis
// square takes a Real and returns a Real
define square(x : ℝ) : ℝ = x * x
// Type: ℝ → ℝ

// add takes two Reals and returns a Real
define add(x : ℝ, y : ℝ) : ℝ = x + y
// Type: ℝ × ℝ → ℝ (or equivalently: ℝ → ℝ → ℝ)
```

### Higher-Order Function Types

Functions can take other functions as arguments or return functions. These are called **higher-order functions**:

```kleis
// A function that takes a function as an argument
define apply_twice(f : ℝ → ℝ, x : ℝ) : ℝ = f(f(x))
// Type: (ℝ → ℝ) × ℝ → ℝ

// A function that returns a function
define make_adder(n : ℝ) : ℝ → ℝ = ???
// Type: ℝ → (ℝ → ℝ)
```

The parentheses matter! Compare:
- `(ℝ → ℝ) → ℝ` — takes a function, returns a number
- `ℝ → (ℝ → ℝ)` — takes a number, returns a function
- `ℝ → ℝ → ℝ` — curried function (associates right: `ℝ → (ℝ → ℝ)`)

### Function Type Examples

| Type | Meaning |
|------|---------|
| `ℝ → ℝ` | Function from real to real |
| `ℝ → ℝ → ℝ` | Curried binary function |
| `(ℝ → ℝ) → ℝ` | Takes a function, returns a value (e.g., definite integral) |
| `ℝ → (ℝ → ℝ)` | Returns a function (function factory) |
| `(ℝ → ℝ) → (ℝ → ℝ)` | Function transformer (e.g., derivative operator) |

## Parametric Types

Types can have parameters:

```kleis
// Parametric type examples:
List(ℤ)           // List of integers
Matrix(3, 3, ℝ)   // 3×3 matrix of reals
Vector(4)         // 4-dimensional vector
```

## Type Inference

Kleis often infers types automatically:

```kleis
define double(x) = x + x
// Kleis infers: double : ℝ → ℝ (or more general)

define square_five = let y = 5 in y * y
// Kleis infers: y : ℤ
```

But explicit types make code clearer and catch errors earlier!

## The Type Hierarchy

```
              Any
         /    |    \
     Scalar  String  Collection
     /    \              |
    ℂ    Bool          List
    |                 /    \
    ℝ            Vector   Matrix
    |
    ℤ
    |
    ℕ
```

Note: `ℕ ⊂ ℤ ⊂ ℝ ⊂ ℂ` (natural numbers are integers are reals are complex)

## What's Next?

Types are the foundation. Now let's see how to define functions!

→ [Next: Functions](./03-functions.md)
