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
✅ Loaded: 1 files, 5 functions, 0 structures, 0 data types, 0 type aliases

λ> :env
📋 Defined functions:
  next_seq (seq) = ...
  valid_ack (sent, ack) = ...
  sender_next_state (current_seq, ack_received) = ...
  receiver_accepts (expected, received) = ...
  receiver_next_state (expected, received) = ...
```

More examples to load:

```
λ> :load examples/business/order_to_cash.kleis
✅ Loaded: 1 files, 21 functions, 0 structures, 4 data types, 0 type aliases

λ> :load examples/authorization/zanzibar.kleis
✅ Loaded: 1 files, 13 functions, 0 structures, 0 data types, 0 type aliases
```

## Verification with Z3

Run verifications interactively with `:verify`:

```
λ> :verify x + y = y + x
✅ Valid

λ> :verify x > 0
❌ Invalid - Counterexample: x!2 -> 0
```

## Satisfiability with Z3

Use `:sat` to find solutions (equation solving):

```
λ> :sat ∃(z : ℂ). z * z = complex(-1, 0)
✅ Satisfiable
   Witness: z_re = 0, z_im = -1

λ> :sat ∃(x : ℝ). x * x = 4
✅ Satisfiable
   Witness: x = -2

λ> :sat ∃(x : ℝ). x * x = -1
❌ Unsatisfiable (no solution exists)

λ> :sat ∃(x : ℝ)(y : ℝ). x + y = 10 ∧ x - y = 4
✅ Satisfiable
   Witness: x = 7, y = 3
```

**`:verify` vs `:sat`:**

| Command | Question | Use Case |
|---------|----------|----------|
| `:verify` | Is it always true? (∀) | Prove theorems |
| `:sat` | Does a solution exist? (∃) | Solve equations |

## Lambda Expressions

Lambda expressions work at the prompt:

```
λ> λ x . x * 2
λ x . times(x, 2)

λ> λ x y . x + y
λ x y . x + y
```

## Type Inference

Check types with `:type`:

```
λ> :type 42
📐 Type: Scalar

λ> :type sin
📐 Type: α0
```

## Concrete Evaluation with `:eval`

The `:eval` command performs **concrete evaluation** — it actually computes results, including recursive functions:

```
λ> :load docs/grammar/lisp_parser.kleis
✅ Loaded: 60 functions

λ> :eval run("(+ 2 3)")
VNum(5)

λ> :eval run("(letrec ((fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))) (fact 5))")
VNum(120)
```

**`:eval` vs `:sat` vs `:verify`:**

| Command | Execution | Handles Recursion | Use Case |
|---------|-----------|-------------------|----------|
| `:eval` | **Concrete** (Rust) | ✅ Yes | Compute actual values |
| `:sat` | Symbolic (Z3) | ❌ No (may timeout) | Find solutions |
| `:verify` | Symbolic (Z3) | ❌ No (may timeout) | Prove theorems |

> **Key insight:** Z3 cannot symbolically unroll recursive functions over unbounded data types. Use `:eval` for concrete computation, `:sat`/`:verify` for symbolic reasoning.

This is what makes Kleis **Turing complete** — the combination of ADTs, pattern matching, recursion, and concrete evaluation enables arbitrary computation. See [Appendix: LISP Interpreter](../appendix/lisp-interpreter.md) for a complete example.

## REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show all commands |
| `:load <file>` | Load a .kleis file |
| `:env` | Show defined functions |
| `:eval <expr>` | **Concrete evaluation** (computes actual values) |
| `:verify <expr>` | Verify with Z3 (is it always true?) |
| `:sat <expr>` | Check satisfiability (does a solution exist?) |
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
λ> :load examples/authorization/zanzibar.kleis
✅ Loaded: 1 files, 13 functions, 0 structures, 0 data types, 0 type aliases

λ> :env
📋 Defined functions:
  can_share (perm) = ...
  can_edit (perm) = ...
  can_delete (perm) = ...
  effective_permission (direct, group) = ...
  inherited_permission (child_perm, parent_perm) = ...
  can_comment (perm) = ...
  is_allowed (perm, action) = ...
  doc_access (doc_perm, folder_perm, action) = ...
  has_at_least (user_perm, required_perm) = ...
  can_read (perm) = ...
  multi_group_permission (perm1, perm2, perm3) = ...
  can_grant (granter_perm, grantee_perm) = ...
  can_transfer_ownership (perm) = ...

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
