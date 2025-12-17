# The REPL

## What is the REPL?

The REPL (Read-Eval-Print Loop) is an interactive environment for experimenting with Kleis:

```bash
$ cargo run --bin repl

🧮 Kleis REPL v0.1.0
   Type :help for commands, :quit to exit

λ>
```

## Basic Usage

Enter expressions to evaluate them symbolically:

```
λ> 2 + 2
2 + 2

λ> let x = 5 in x * x
times(5, 5)

λ> sin(π / 2)
sin(divide(π, 2))
```

> **Note:** The REPL performs **symbolic evaluation**, not numeric computation. Expressions are simplified symbolically, not calculated to numbers.

## Loading Files

The REPL prompt evaluates expressions. For definitions (`define`, `structure`, etc.), use `:load`:

```
λ> :load examples/protocols/stop_and_wait.kleis
✅ Loaded: 5 functions, 0 structures, 0 data types, 0 type aliases

λ> :env
📋 Defined functions:
  next_seq (seq) = ...
  valid_ack (sent, ack) = ...
  receiver_accepts (expected, received) = ...
  sender_next_state (current_seq, ack_received) = ...
  receiver_next_state (expected, received) = ...
```

More examples to load:

```
λ> :load examples/business/order_to_cash.kleis
✅ Loaded: 7 functions, 1 structures, 0 data types, 0 type aliases

λ> :load examples/control/lqg_controller.kleis
✅ Loaded: 12 functions, 3 structures, 0 data types, 0 type aliases
```

## Verification with Z3

Run verifications interactively with `:verify`:

```
λ> :verify x + y = y + x
✅ Valid

λ> :verify x > 0
❌ Invalid - Counterexample: x!2 -> 0
```

## Lambda Expressions

Lambda expressions work at the prompt:

```
λ> λ x . x * 2
λ x . times(x, 2)

λ> λ x y . x + y
λ x y . plus(x, y)
```

## Type Inference

Check types with `:type`:

```
λ> :type 42
📐 Type: Scalar

λ> :type sin
📐 Type: α0
```

## REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show all commands |
| `:load <file>` | Load a .kleis file |
| `:env` | Show defined functions |
| `:verify <expr>` | Verify with Z3 |
| `:type <expr>` | Show inferred type |
| `:ast <expr>` | Show parsed AST |
| `:symbols` | Unicode math symbols palette |
| `:syntax` | Complete syntax reference |
| `:examples` | Show example expressions |
| `:quit` | Exit REPL |

## Multi-line Input

For complex expressions, end lines with `\` or use block mode:

```
λ> :verify ∀(a : R, b : R). \
   (a + b) * (a - b) = a * a - b * b
✅ Valid
```

Or use `:{ ... :}` for blocks:

```
λ> :{
   :verify ∀(x : R, y : R, z : R).
     (x + y) + z = x + (y + z)
   :}
✅ Valid
```

## Example Session

```
 :load examples/authorization/zanzibar.kleis
✅ Loaded: 13 functions, 0 structures, 0 data types, 0 type aliases

λ> :env
📋 Defined functions:
  can_delete (perm) = ...
  effective_permission (direct, group) = ...
  inherited_permission (child_perm, parent_perm) = ...
  has_at_least (user_perm, required_perm) = ...
  can_share (perm) = ...
  doc_access (doc_perm, folder_perm, action) = ...
  multi_group_permission (perm1, perm2, perm3) = ...
  can_read (perm) = ...
  can_comment (perm) = ...
  can_transfer_ownership (perm) = ...
  can_edit (perm) = ...
  is_allowed (perm, action) = ...
  can_grant (granter_perm, grantee_perm) = ...

λ> :verify ∀(x : ℝ). x * x ≥ 0
✅ Valid

λ> :quit
Goodbye! 👋
```

## Tips

1. Press **Ctrl+C** to cancel input
2. Press **Ctrl+D** or type `:quit` to exit
3. Use `:symbols` to copy-paste Unicode math symbols
4. Use `:help <topic>` for detailed help (e.g., `:help quantifiers`)

## What's Next?

See practical applications!

→ [Next: Applications](./13-applications.md)
