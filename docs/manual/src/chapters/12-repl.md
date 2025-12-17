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
kleis> 2 + 2
4

kleis> let x = 5 in x * x
25

kleis> sin(π / 2)
1.0
```

## Defining Functions

Define functions interactively:

```
kleis> define square(x) = x * x
Defined: square

kleis> square(7)
49

kleis> define compose(f, g, x) = f(g(x))
Defined: compose

kleis> compose(square, square, 2)
16
```

## Working with Types

Check types and use annotations:

```
kleis> :type 42
42 : ℤ

kleis> :type sin
sin : ℝ → ℝ

kleis> let x : ℝ = 3.14 in x * 2
6.28
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

> 🚧 **Coming Soon: We're working on it!**
>
> Lambda expressions are not yet implemented. Use named functions instead:

```
kleis> -- Instead of: (\x -> x * 2)
kleis> define double(x) = x * 2
Defined: double

kleis> double(21)
42
```

## Example Session

```
kleis> -- Define a structure
kleis> structure Point { x : ℝ, y : ℝ }

kleis> -- Create a point
kleis> let p = Point { x = 3, y = 4 } in
...>     sqrt(p.x^2 + p.y^2)
5.0

kleis> -- Verify properties
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
