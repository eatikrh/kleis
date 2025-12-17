# Chapter 13: The REPL

[← Previous: Z3 Verification](12-z3-verification.md) | [Back to Contents](../index.md) | [Next: Applications →](14-applications.md)

---

## Starting the REPL

Launch the Kleis REPL:

```bash
cargo run --bin repl
```

You'll see:

```
Kleis REPL v0.1.0
Type :help for commands, :quit to exit
>>> 
```

---

## Basic Usage

### Evaluating Expressions

```kleis
>>> 2 + 3
5

>>> let x = 5 in x * x
25

>>> sqrt(16)
4
```

### Defining Functions

```kleis
>>> define square(x) = x * x
Function 'square' defined

>>> square(7)
49
```

### Multi-line Input

For complex definitions, the REPL continues until complete:

```kleis
>>> define factorial(n) =
...     if n = 0 then 1
...     else n * factorial(n - 1)
Function 'factorial' defined

>>> factorial(5)
120
```

---

## REPL Commands

Commands start with `:`:

| Command | Description |
|---------|-------------|
| `:help` | Show help |
| `:quit` or `:q` | Exit REPL |
| `:load <file>` | Load a Kleis file |
| `:type <expr>` | Show expression type |
| `:clear` | Clear all definitions |
| `:list` | List defined functions |
| `:verify` | Verify loaded structures |

### Examples

```kleis
>>> :type 5
5 : ℤ

>>> :type sqrt
sqrt : ℝ → ℝ

>>> :list
Functions:
  - square : a → a
  - factorial : ℤ → ℤ

>>> :load my_theory.kleis
Loaded 15 definitions from my_theory.kleis
```

---

## Loading Files

Create a file `mymath.kleis`:

```kleis
// mymath.kleis
define double(x) = x + x
define triple(x) = x + x + x

structure Pointed(P) {
    operation point : P
}
```

Load it:

```kleis
>>> :load mymath.kleis
Loaded 3 definitions

>>> double(5)
10

>>> triple(5)
15
```

---

## Working with Types

### Type Inference

```kleis
>>> :type let x = 5 in x + 1
let x = 5 in x + 1 : ℤ

>>> :type \x. x + 1      // Coming soon!
λx. x + 1 : ℤ → ℤ
```

### Type Annotations

```kleis
>>> let x : ℝ = 5 in sqrt(x)
2.236...

>>> (3 + 4) : ℤ
7
```

---

## Verification in REPL

### Verifying Structures

```kleis
>>> structure Group(G) {
...     operation (*) : G → G → G
...     operation e : G
...     operation inv : G → G
...     axiom identity : ∀(x : G). x * e = x
...     axiom inverse : ∀(x : G). x * inv(x) = e
... }
Structure 'Group' defined

>>> :verify Group
Verifying Group...
✓ identity: consistent
✓ inverse: consistent
All axioms verified!
```

### Checking Specific Axioms

```kleis
>>> :verify Group.identity
Verifying axiom 'identity'...
✓ Verified
```

---

## History and Editing

The REPL supports:

- **Up/Down arrows**: Navigate history
- **Ctrl+R**: Reverse search
- **Tab**: Autocomplete
- **Ctrl+C**: Cancel current input
- **Ctrl+D**: Exit (same as :quit)

---

## Debugging

### Show Parse Tree

```kleis
>>> :parse 2 + 3 * 4
Operation {
  name: "plus",
  args: [
    Const("2"),
    Operation {
      name: "times",
      args: [Const("3"), Const("4")]
    }
  ]
}
```

### Trace Evaluation

```kleis
>>> :trace factorial(3)
factorial(3)
  = 3 * factorial(2)
  = 3 * (2 * factorial(1))
  = 3 * (2 * (1 * factorial(0)))
  = 3 * (2 * (1 * 1))
  = 3 * (2 * 1)
  = 3 * 2
  = 6
```

---

## Tips and Tricks

### 1. Use Tab Completion

```kleis
>>> sq<TAB>
>>> square
```

### 2. Quick Testing

```kleis
>>> define test() = assert(square(5) = 25)
>>> test()
✓ Assertion passed
```

### 3. Explore Types

```kleis
>>> :type
Usage: :type <expression>
Shows the inferred type of the expression

>>> :type []
[] : List(a)

>>> :type [1, 2, 3]
[1, 2, 3] : List(ℤ)
```

### 4. Reset State

```kleis
>>> :clear
All definitions cleared

>>> square(5)
Error: Unknown function 'square'
```

---

## Summary

- Start with `cargo run --bin repl`
- Evaluate expressions directly
- Define functions with `define`
- Use `:commands` for special operations
- Load files with `:load`
- Verify structures with `:verify`
- Use history and tab completion

---

[← Previous: Z3 Verification](12-z3-verification.md) | [Back to Contents](../index.md) | [Next: Applications →](14-applications.md)

