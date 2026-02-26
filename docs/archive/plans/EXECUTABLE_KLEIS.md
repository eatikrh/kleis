# Executable Kleis: Examples and REPL Debugging

> **Status: ✅ IMPLEMENTED** (Dec 2024, Grammar v0.93)  
> Example blocks, `assert()` statements, and REPL `:debug` command are all working.
> Assert uses Z3 for symbolic verification. See `kleis test` command.
> This document serves as the design rationale.

---

## Problem Statement

Kleis files are **declarative** - they contain structures, axioms, and function definitions.
There's nothing to "execute" in the traditional sense, which makes debugging challenging.

**We need execution points for:**
- Step-through debugging
- Testing/verification  
- Interactive exploration

## Solution: Two Complementary Approaches

### 1. `example` Blocks (in files)

Mathematical proofs often include examples to illustrate concepts. We adopt this convention:

```kleis
structure Complex(re: ℝ, im: ℝ) {
  operation add : Complex → Complex
  operation multiply : Complex → Complex
  
  axiom add_commutative: ∀(z1 z2 : Complex).
    add(z1, z2) = add(z2, z1)
}

example "complex arithmetic" {
  let z1 = Complex(1, 2)
  let z2 = Complex(3, 4)
  let sum = add(z1, z2)
  
  // Assertions are verified
  assert(sum.re = 4)
  assert(sum.im = 6)
  
  // Can also just compute
  let product = multiply(z1, z2)
  product  // Returns: Complex(-5, 10)
}

example "euler identity" {
  let i = Complex(0, 1)
  let result = exp(multiply(i, π))
  
  assert(result ≈ Complex(-1, 0))
}
```

**Semantics:**
- `example` blocks are **executable** - they evaluate sequentially
- Each `let` binding is a step (debugger can pause)
- `assert` statements are verified:
  - **Concrete**: Evaluate and check directly
  - **Symbolic**: Use Z3 to prove from axioms in scope
- Examples can be run as tests: `kleis test file.kleis`
- Examples can be debugged: debugger steps through the block

**Benefits:**
- Fits mathematical terminology perfectly
- Serves as documentation
- Serves as tests
- Provides entry points for debugging
- Doesn't pollute the declaration space

### 2. REPL `:debug` Command

For interactive exploration, the REPL gains a `:debug` command:

```
λ> :load stdlib/complex.kleis
Loaded: 3 structures, 12 operations, 8 axioms

λ> :debug multiply(Complex(1,2), Complex(3,4))
[debug] Starting debug session...
[step 1] multiply(Complex(1,2), Complex(3,4))
         ├─ Expanding multiply...
[debug] (n)ext (s)tep-in (o)ut (c)ontinue (v)ars (q)uit: n

[step 2] let re = 1*3 - 2*4 = -5
[debug] (n)ext: n

[step 3] let im = 1*4 + 2*3 = 10  
[debug] (n)ext: n

[step 4] Complex(-5, 10)
[debug] Evaluation complete.
Result: Complex(-5, 10)

λ> 
```

**REPL Debug Commands:**
| Command | Action |
|---------|--------|
| `:debug <expr>` | Start debugging an expression |
| `n` / `next` | Step over (next top-level step) |
| `s` / `step` | Step into (expand current expression) |
| `o` / `out` | Step out (finish current function) |
| `c` / `continue` | Run to completion |
| `v` / `vars` | Show current bindings |
| `q` / `quit` | Abort debug session |

## Grammar Changes (v0.93)

### New `example` Production

```ebnf
example_block ::= 'example' STRING '{' example_body '}'

example_body ::= (let_binding | assert_stmt | expression)*

let_binding ::= 'let' IDENT '=' expression

assert_stmt ::= 'assert' '(' expression ')'

top_level ::= structure | implements | define | let_binding | example_block
```

**Assert Design:**
- Single form: `assert(expression)` 
- The expression itself serves as the error message on failure
- Symbolic expressions → Z3 proves from axioms
- Concrete expressions → evaluator checks directly
- No separate message parameter needed

### Parser Updates

Add to `kleis_parser.rs`:
- `parse_example_block()`
- Add `Example` variant to AST
- Handle in evaluator

## Implementation Plan

### Phase 1: Grammar & Parser
- [ ] Add `example` keyword to lexer
- [ ] Add `Example` AST node
- [ ] Parse example blocks
- [ ] Parse `assert` statements

### Phase 2: Evaluator Support
- [ ] Evaluate example blocks sequentially
- [ ] Implement `assert`:
  - Concrete: evaluate expression, check if true
  - Symbolic: ask Z3 to prove from axioms in scope
  - Report assertion expression on failure
- [ ] Track which examples passed/failed/unknown

### Phase 3: CLI Integration
- [ ] `kleis test <file>` - run all examples
- [ ] `kleis test <file> --example "name"` - run specific example
- [ ] Exit codes for CI integration

### Phase 4: REPL `:debug` Command
- [ ] Add `:debug` command to REPL
- [ ] Integrate with existing `DebugHook` infrastructure
- [ ] Interactive stepping UI

### Phase 5: VS Code/DAP Integration
- [ ] Debugger can launch example blocks
- [ ] Set breakpoints in example blocks
- [ ] Step through example evaluation

## Examples in the Wild

### Symbolic Proof (Z3 Verifies from Axioms)

```kleis
structure Matrix(m: ℕ, n: ℕ) {
  operation det : Matrix(n, n) → ℝ
  operation multiply : Matrix(n, p) → Matrix(m, p)
  
  axiom det_multiplicative: ∀(A B : Matrix(n, n)).
    det(multiply(A, B)) = det(A) × det(B)
}

example "determinant of product" {
  let A : Matrix(3, 3)  // symbolic - no concrete values
  let B : Matrix(3, 3)
  
  // Z3 proves this from det_multiplicative axiom
  assert(det(multiply(A, B)) = det(A) × det(B))
}
```

### Universal Properties

```kleis
structure VectorSpace(V, F) {
  operation zero : V
  operation add : V → V → V
  
  axiom add_identity: ∀(v : V). add(v, zero) = v
  axiom add_assoc: ∀(u v w : V). add(add(u, v), w) = add(u, add(v, w))
}

example "vector space axioms" {
  // Z3 verifies these hold for all vectors
  assert(∀(v : V). add(v, zero) = v)
  assert(∀(u v w : V). add(add(u, v), w) = add(u, add(v, w)))
}
```

### Concrete Arithmetic

```kleis
example "complex multiplication" {
  let z1 = Complex(1, 2)
  let z2 = Complex(3, 4)
  let product = multiply(z1, z2)
  
  // Evaluator computes directly
  assert(product.re = -5)   // 1*3 - 2*4
  assert(product.im = 10)   // 1*4 + 2*3
}
```

### Mixed Symbolic and Concrete

```kleis
example "matrix inverse" {
  // Symbolic part: Z3 proves from axioms
  let M : Matrix(n, n)
  assert(multiply(M, inverse(M)) = identity)
  
  // Concrete part: evaluator computes
  let A = Matrix(2, 2, [[4, 7], [2, 6]])
  let result = multiply(A, inverse(A))
  assert(result[0,0] = 1)
  assert(result[0,1] = 0)
}
```

## Relationship to Existing Infrastructure

- **DebugHook trait**: Already implemented in `src/debug.rs`
- **Evaluator integration**: Hooks already in `src/evaluator.rs`
- **DAP adapter**: Already has basic structure in `src/dap.rs`
- **VS Code extension**: Debug adapter factory ready

The `example` block provides the **entry point** that connects all these pieces.

## Decision

**Approved approach:** Implement `example` blocks + REPL `:debug` command

This preserves Kleis's declarative nature while providing:
- Executable documentation
- Testable specifications  
- Debuggable entry points
- Mathematical terminology fit

