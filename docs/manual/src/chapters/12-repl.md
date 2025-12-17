# The REPL

## What is the REPL?

The REPL (Read-Eval-Print Loop) is an interactive environment for experimenting with Kleis:

```bash
$ cargo run --bin repl
Kleis REPL v0.7
Type 'help' for commands, 'quit' to exit.

kleis>
```

## Basic Usage

Enter expressions to evaluate them:

```
λ> 2 + 2
2 + 2

λ> let x = 5 in x * x
times(5, 5)

λ> sin(π / 2)
sin(divide(π, 2))
```

## Defining Functions

The REPL prompt evaluates **expressions**, not declarations. To define functions, use `:load` with a `.kleis` file:

```
λ> define square(x) = x * x
❌ Parse error: Kleis parse error at position 7: Unexpected character: 's'
```

Instead, create a file `mymath.kleis`:

```kleis
define square(x) = x * x
define compose(f, g, x) = f(g(x))
```

Then load it in the REPL:

```
λ> :load mymath.kleis
✅ Loaded: 2 functions, 0 structures, 0 data types, 0 type aliases

λ> square(7)
times(7, 7)

λ> compose(square, square, 2)
compose(square, square, 2)
```

> **Note:** The REPL performs symbolic evaluation. `square(7)` becomes `times(7, 7)` rather than computing `49`. This is by design - Kleis focuses on symbolic manipulation and verification, not numeric computation.

## Working with Types

Check types and use annotations:

```
λ> type 42
❌ Parse error: Kleis parse error at position 5: Unexpected character: '4'

λ> :type sin
📐 Type: α0

λ> let x : ℝ = 3.14 in x * 2
times(3.14, 2)
```

## REPL Commands

| Command | Description |
|---------|-------------|
| `:help` or `?` | Show help |
| `:type <expr>` | Show expression type |
| `:quit` or `q` | Exit REPL |
| `:clear` | Clear definitions |
| `:load <file>` | Load Kleis file |
| `:env` | Show current environment |

## Loading Files

Load Kleis source files:

```
kleis> :load stdlib/prelude.kleis
Loaded: stdlib/prelude.kleis

kleis> :load examples/geometry.kleis
Loaded: examples/geometry.kleis
```

## Verification in REPL

Run verifications interactively:

```
kleis> verify x + y = y + x
✓ Valid

kleis> verify ∀ n : ℕ . n ≥ 0
✓ Valid

kleis> verify x > 0
✗ Invalid
Counterexample: x = -1
```

## Multi-line Input

For complex expressions, use continuation:

```
kleis> define factorial(n) =
...>     if n = 0 then 1
...>     else n * factorial(n - 1)
Defined: factorial
```

## Lambda Expressions in REPL

Lambda expressions work in the REPL:

```
kleis> λ x . x * 2
λ x . x * 2

kleis> (λ x . x + 1)(5)
6

kleis> define double = λ x . x * 2
Defined: double

kleis> double(21)
42
```

You can use both the `λ` symbol and the `lambda` keyword.

## Example Session

```
kleis> // Define a structure
kleis> structure Point { x : ℝ, y : ℝ }

kleis> // Create a point
kleis> let p = Point { x = 3, y = 4 } in
...>     sqrt(p.x^2 + p.y^2)
5.0

kleis> // Verify properties
kleis> verify ∀ a : ℝ . ∀ b : ℝ . (a + b)^2 = a^2 + 2*a*b + b^2
✓ Valid

kleis> :quit
Goodbye!
```

## Tips and Tricks

1. **Use tab completion** for function names
2. **Arrow keys** navigate history
3. **Ctrl+C** cancels current input
4. **Ctrl+D** exits (like `:quit`)

## What's Next?

See practical applications!

→ [Next: Applications](./13-applications.md)
