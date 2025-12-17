# Starting Out

## Your First Kleis Expression

The simplest things in Kleis are **expressions**. An expression is anything that has a value:

```kleis
42              -- A number
3.14159         -- A decimal
x + y           -- An arithmetic expression
sin(θ)          -- A function call
```

## The REPL

The easiest way to experiment with Kleis is the **REPL** (Read-Eval-Print Loop):

```bash
$ cargo run --bin repl
Kleis REPL v0.7
Type 'help' for commands, 'quit' to exit.

kleis> 2 + 2
4

kleis> let x = 5 in x * x
25
```

## Basic Arithmetic

Kleis supports the usual arithmetic operations:

```kleis
2 + 3       -- Addition: 5
10 - 4      -- Subtraction: 6
3 * 7       -- Multiplication: 21
15 / 3      -- Division: 5
2 ^ 10      -- Exponentiation: 1024
```

## Variables and Definitions

Use `define` to create named values:

```kleis
define pi = 3.14159
define e = 2.71828
define golden_ratio = (1 + sqrt(5)) / 2
```

Functions are defined similarly:

```kleis
define square(x) = x * x
define cube(x) = x * x * x
define area_circle(r) = pi * r^2
```

## Comments

Kleis uses `--` for single-line comments:

```kleis
-- This is a comment
define x = 42  -- Inline comment

{- 
   Multi-line comments
   use curly braces with dashes
-}
```

## Unicode Support

Kleis embraces mathematical notation with full Unicode support:

```kleis
-- Greek letters
define α = 0.5
define β = 1.0
define θ = π / 4

-- Mathematical symbols
∀ x . x = x           -- Universal quantifier
∃ y . y > 0           -- Existential quantifier
x ∈ ℝ                 -- Set membership
A ⊆ B                 -- Subset
```

You can use ASCII alternatives too:

| Unicode | ASCII Alternative |
|---------|-------------------|
| `∀`     | `forall`          |
| `∃`     | `exists`          |
| `→`     | `->`              |
| `×`     | `*`               |
| `ℝ`     | `Real`            |
| `ℕ`     | `Nat`             |

## What's Next?

Now that you can write basic expressions, let's learn about the type system!

→ [Next: Types and Values](./02-types.md)
