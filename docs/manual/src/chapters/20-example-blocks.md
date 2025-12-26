# Example Blocks (v0.93)

Kleis v0.93 introduces **example blocks** — executable documentation that serves as tests, debugging entry points, and living examples.

## Syntax

```kleis
example "descriptive name" {
    let x = 5
    let y = double(x)
    assert(y = 10)
}
```

An example block contains:
- **Let bindings** — bind values to names
- **Assert statements** — verify expected results
- **Expressions** — any valid Kleis expression

## Why Example Blocks?

| Traditional Tests | Example Blocks |
|-------------------|----------------|
| Separate test files | Inline with code |
| Run with test runner | Run with `kleis test` |
| Not visible in docs | Executable documentation |
| Hard to debug | Full DAP debugger support |

Example blocks serve three purposes:

1. **Documentation** — Show how to use your functions
2. **Testing** — Verify behavior with assertions
3. **Debugging** — Set breakpoints and step through

## Running Examples

Use the `kleis test` command:

```bash
$ kleis test examples/math/complex_demo.kleis

✅ complex arithmetic basics
✅ euler's formula
✅ quadratic roots

3/3 examples passed
```

Failed assertions show details:

```bash
$ kleis test broken.kleis

❌ my test
   Assertion failed: expected Const("20"), got Const("15")

0/1 examples passed (1 failed)
```

## Assert Statement

The `assert` statement verifies a condition:

```kleis
example "basic assertions" {
    let x = 5
    
    // Simple equality
    assert(x = 5)
    
    // Expression equality
    assert(x + x = 10)
    
    // Boolean conditions
    assert(x > 0)
    
    // Function results
    assert(double(x) = 10)
}
```

**Important:** `assert(expr)` evaluates `expr` and checks if it equals `True` or if both sides of `=` are equal.

### Symbolic Assertions with Z3

**New in v0.93:** Assertions with symbolic (unbound) variables are verified using Z3!

```kleis
structure CommutativeRing(R) {
    operation (+) : R × R → R
    axiom commutativity: ∀(a b : R). a + b = b + a
}

example "algebraic properties" {
    // Z3 verifies this using the commutativity axiom!
    assert(x + y = y + x)
    
    // Z3 proves associativity if the axiom is defined
    assert((a + b) + c = a + (b + c))
}
```

When an assertion contains unbound variables (like `x`, `y`), Kleis:

1. Detects the expression is symbolic
2. Loads axioms from defined structures
3. Passes the claim to Z3 for verification
4. Reports: Verified, Disproved (with counterexample), or Unknown

```kleis
example "z3 finds counterexamples" {
    // Z3 disproves this with: "Counterexample: y!1 -> 1, x!0 -> 0"
    // assert(x + y = y + y)  // Would fail!
}
```

This enables **theorem proving** in your tests:

```kleis
structure Field(F) {
    operation (*) : F × F → F
    operation inverse : F → F
    
    axiom inverse_right: ∀(x : F). x * inverse(x) = 1
    axiom inverse_left: ∀(x : F). inverse(x) * x = 1
}

example "inverse properties" {
    // Z3 verifies using field axioms
    assert(a * inverse(a) = 1)
    assert(inverse(inverse(a)) = a)  // Derived property!
}
```

## Example Blocks as Entry Points

Example blocks are the **entry points for debugging**. Unlike function definitions which are just declarations, example blocks contain executable code:

```kleis
// Function definition (not executable on its own)
define fib(n) = 
    if n <= 1 then n 
    else fib(n - 1) + fib(n - 2)

// Example block (executable, can set breakpoints)
example "fibonacci test" {
    let f5 = fib(5)      // ← Set breakpoint here
    let f10 = fib(10)    // ← Or here
    assert(f5 = 5)
    assert(f10 = 55)
}
```

When debugging:
1. Set a breakpoint on a line in an example block
2. Launch the debugger
3. Execution stops at your breakpoint
4. Step through, inspect variables, step into functions

## Cross-File Debugging

Example blocks work with imports. When you step into a function from an imported file, the debugger opens that file:

```kleis
// main.kleis
import "stdlib/complex.kleis"

example "complex math" {
    let z = complex(3, 4)
    let mag = abs(z)        // ← Step into this
    assert(mag = 5)         // Opens complex.kleis, shows abs definition
}
```

The debugger tracks source locations across files, showing you exactly where you are.

## Source Location Tracking

Every expression in Kleis carries its **source location** (line, column, file). This enables:

- Accurate error messages
- Precise debugger breakpoints
- Cross-file stepping
- Stack traces with file paths

The location travels with the expression through evaluation, so even after function application, the debugger knows the original source.

## Best Practices

### 1. One Concept Per Example

```kleis
// Good: focused examples
example "addition is commutative" {
    assert(2 + 3 = 3 + 2)
}

example "multiplication distributes" {
    assert(2 * (3 + 4) = 2 * 3 + 2 * 4)
}

// Bad: too much in one example
example "all arithmetic" {
    assert(2 + 3 = 3 + 2)
    assert(2 * 3 = 3 * 2)
    assert(2 * (3 + 4) = 2 * 3 + 2 * 4)
    // ... many more assertions
}
```

### 2. Descriptive Names

```kleis
// Good: describes what's being tested
example "negative numbers square to positive" { ... }

// Bad: vague
example "test1" { ... }
```

### 3. Use Let Bindings for Clarity

```kleis
// Good: intermediate values have names
example "quadratic formula" {
    let a = 1
    let b = -5
    let c = 6
    let discriminant = b * b - 4 * a * c
    let root1 = (-b + sqrt(discriminant)) / (2 * a)
    assert(root1 = 3)
}

// Bad: one big expression
example "quadratic formula" {
    assert((-(-5) + sqrt((-5) * (-5) - 4 * 1 * 6)) / (2 * 1) = 3)
}
```

## Grammar Reference

```ebnf
exampleBlock    ::= "example" string "{" exampleBody "}"
exampleBody     ::= { exampleStatement }
exampleStatement ::= letBinding 
                   | assertStatement 
                   | expression ";"

assertStatement ::= "assert" "(" expression ")"

letBinding      ::= "let" identifier [":" type] "=" expression
```

## What's Next?

Learn how to set up VS Code for debugging:

→ [Appendix: VS Code Debugging](../appendix/vscode-debugging.md)

