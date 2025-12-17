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
λ> :load stdlib/prelude.kleis
✅ Loaded: 4 functions, 15 structures, 0 data types, 0 type aliases

λ> :load examples/authorization/zanzibar.kleis
✅ Loaded: 13 functions, 0 structures, 0 data types, 0 type aliases
```

## Verification in REPL

Run verifications interactively:

```
λ> :verify x + y = y + x
DEBUG: Found dependencies: {}
✅ Valid

λ> :verify ∀ n : ℕ . n ≥ 0
DEBUG: Found dependencies: {}
❌ Invalid - Counterexample: n!2 -> (- 1)

λ> :verify x > 0
DEBUG: Found dependencies: {}
❌ Invalid - Counterexample: x!3 -> 0

```

## Multi-line Input

For complex expressions, use continuation:

```
<multiline example>
```

## Lambda Expressions in REPL

Lambda expressions work in the REPL:

```
λ> λ x . x * 2
λ x . times(x, 2)

λ> (λ x . x + 1)(5)
❌ Parse error: Kleis parse error at position 13: Unexpected character: '('

λ> ddefine double = λ x . x * 2
❌ Parse error: Kleis parse error at position 7: Unexpected character: 'd'

λ> double(21)
double(21)
```

You can use both the `λ` symbol and the `lambda` keyword.

## Example Session

```
λ> // Define a structure
❌ Parse error: Kleis parse error at position 21: Expected expression
λ> structure Point { x : ℝ, y : ℝ }
❌ Parse error: Kleis parse error at position 10: Unexpected character: 'P'


λ> // Create a point
❌ Parse error: Kleis parse error at position 17: Expected expression
λ> let p = Point { x = 3, y = 4 } in \
   sqrt(p.x^2 + p.y^2)
❌ Parse error: Kleis parse error at position 14: Expected keyword 'in'

λ> // Verify properties
❌ Parse error: Kleis parse error at position 20: Expected expression
λ>  :verify ∀ a : ℝ . ∀ b : ℝ . (a + b)^2 = a^2 + 2*a*b + b^2
DEBUG: Found dependencies: {}
❌ Invalid - Counterexample: b!5 -> 0.0
a!4 -> 0.0
power -> {
  (- 1)
}

λ> :quit
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
