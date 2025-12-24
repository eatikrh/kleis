# Executable Kleis: Examples and REPL Debugging

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
- `assert` statements are verified
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

## Grammar Changes

### New `example` Production

```ebnf
example_block ::= 'example' STRING '{' example_body '}'

example_body ::= (let_binding | assert_stmt | expression)*

let_binding ::= 'let' IDENT '=' expression

assert_stmt ::= 'assert' '(' expression ')'
              | 'assert' '(' expression '≈' expression ')'  // approximate equality

top_level ::= structure | implements | define | let_binding | example_block
```

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
- [ ] Implement `assert` with error reporting
- [ ] Track which examples passed/failed

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

### From Physics

```kleis
structure Pendulum(length: ℝ, mass: ℝ) {
  operation period : ℝ
  
  axiom period_formula: ∀(p : Pendulum).
    period(p) = 2 * π * sqrt(p.length / g)
}

example "simple pendulum period" {
  let p = Pendulum(length: 1.0, mass: 0.5)  // 1 meter pendulum
  let T = period(p)
  
  // With g ≈ 9.81, period should be ≈ 2.006 seconds
  assert(T ≈ 2.006)
}
```

### From Linear Algebra

```kleis
structure Matrix(m: ℕ, n: ℕ) {
  operation multiply : Matrix(n, p) → Matrix(m, p)
  operation transpose : Matrix(n, m)
}

example "matrix multiplication" {
  let A = Matrix(2, 3, [[1,2,3], [4,5,6]])
  let B = Matrix(3, 2, [[7,8], [9,10], [11,12]])
  let C = multiply(A, B)
  
  assert(C.rows = 2)
  assert(C.cols = 2)
  assert(C[0,0] = 58)   // 1*7 + 2*9 + 3*11
  assert(C[0,1] = 64)   // 1*8 + 2*10 + 3*12
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

