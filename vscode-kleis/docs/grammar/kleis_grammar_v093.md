# Kleis Grammar v0.93 - Example Blocks

**Date:** 2025-12-24  
**Based on:** v0.92 (type-level arithmetic)

## Overview

Grammar v0.93 introduces **example blocks** - executable documentation that serves as tests and debugging entry points.

## New Features

### 1. Example Blocks

Example blocks are named, executable sections within Kleis files:

```kleis
example "complex arithmetic" {
    let z1 = Complex(1, 2)
    let z2 = Complex(3, 4)
    let sum = add(z1, z2)
    
    assert(sum.re = 4)
    assert(sum.im = 6)
}
```

**Semantics:**
- Statements execute sequentially (unlike declarations)
- Each `let` binding is a debuggable step
- `assert()` verifies conditions
- The block can have a final expression as its result

### 2. Assert Statement

Assert statements verify **symbolic** conditions using Z3 and axioms in scope:

```kleis
// Symbolic assertions (Z3 proves from axioms)
assert(a + b = b + a)                   // commutativity 
assert(det(A × B) = det(A) × det(B))   // determinant multiplicativity
assert(∀(x : ℝ). x + 0 = x)            // universal property

// Concrete assertions (evaluator checks)
assert(2 + 2 = 4)                       // arithmetic
assert(sum.re = 4)                      // after evaluation
```

**Semantics:**
- **Concrete expressions**: Evaluated and checked directly
- **Symbolic expressions**: Z3 attempts to prove from axioms in scope
- **If Z3 times out**: Reports "unknown", not failure
- **The expression itself is the error message** (no separate message needed)

### 3. REPL `:debug` Command (Runtime)

While not part of the grammar, v0.93 also introduces REPL debugging:

```
λ> :load stdlib/complex.kleis
λ> :debug multiply(Complex(1,2), Complex(3,4))
[step 1] multiply(Complex(1,2), Complex(3,4))
[debug] (n)ext: n
[step 2] let re = 1*3 - 2*4 = -5
...
```

## Grammar Rules

### Example Block

```ebnf
exampleBlock
    ::= "example" string "{" exampleBody "}"
      ;

exampleBody
    ::= { exampleStatement } [ expression ]
      ;

exampleStatement
    ::= letBinding
      | assertStatement
      | expression ";"
      ;

assertStatement
    ::= "assert" "(" expression ")"
      ;
```

### Updated Top-Level

```ebnf
declaration 
    ::= importDecl
      | libraryAnnotation
      | versionAnnotation
      | structureDecl
      | implementsDecl
      | dataDecl
      | functionDef
      | letBinding
      | typeAlias
      | exampleBlock        (* v0.93: NEW *)
      ;
```

## Use Cases

### 1. Executable Documentation

```kleis
structure Vector(n: ℕ, T) {
    operation dot : Vector(n, T) → T
    operation cross : Vector(3, T) → Vector(3, T)  // Only for 3D
}

example "vector dot product" {
    let v1 = Vector(3, [1, 2, 3])
    let v2 = Vector(3, [4, 5, 6])
    let result = dot(v1, v2)
    
    assert(result = 32)  // 1*4 + 2*5 + 3*6
}

example "cross product is antisymmetric" {
    let v1 = Vector(3, [1, 0, 0])
    let v2 = Vector(3, [0, 1, 0])
    
    let c1 = cross(v1, v2)
    let c2 = cross(v2, v1)
    
    assert(c1 = negate(c2))
}
```

### 2. Testing

Run all examples as tests:

```bash
kleis test file.kleis
# ✅ vector dot product: passed
# ✅ cross product is antisymmetric: passed
# 2/2 examples passed
```

### 3. Debugging

Debug a specific example:

```bash
kleis debug file.kleis --example "vector dot product"
```

Or in VS Code:
- Set breakpoints inside example blocks
- Use F5 to start debugging
- Step through each `let` binding

## Examples

### Symbolic Assertion (Z3 Proves from Axioms)

```kleis
structure Matrix(m: ℕ, n: ℕ) {
    operation det : Matrix(n, n) → ℝ
    operation multiply : Matrix(n, p) → Matrix(m, p)
    
    axiom det_multiplicative: ∀(A B : Matrix(n, n)).
        det(multiply(A, B)) = det(A) × det(B)
}

example "determinant of product" {
    let A : Matrix(3, 3)
    let B : Matrix(3, 3)
    
    // Z3 proves this from det_multiplicative axiom
    assert(det(multiply(A, B)) = det(A) × det(B))
}
```

### Universal Quantifier Verification

```kleis
structure VectorSpace(V, F) {
    operation zero : V
    operation add : V → V → V
    
    axiom add_identity: ∀(v : V). add(v, zero) = v
    axiom add_comm: ∀(u v : V). add(u, v) = add(v, u)
}

example "vector space properties" {
    // Z3 verifies these hold for all vectors
    assert(∀(v : V). add(v, zero) = v)
    assert(∀(u v : V). add(u, v) = add(v, u))
}
```

### Concrete Evaluation

```kleis
example "complex arithmetic" {
    let z1 = Complex(1, 2)
    let z2 = Complex(3, 4)
    let sum = add(z1, z2)
    
    // Concrete evaluation - evaluator computes directly
    assert(sum.re = 4)
    assert(sum.im = 6)
}
```

### Mixed Symbolic and Concrete

```kleis
example "matrix inverse property" {
    // Symbolic: Z3 proves from axioms
    let A : Matrix(n, n)
    assert(multiply(A, inverse(A)) = identity)
    
    // Concrete: evaluator computes
    let B = Matrix(2, 2, [[4, 7], [2, 6]])
    let B_inv = inverse(B)
    let I = multiply(B, B_inv)
    assert(I[0,0] = 1)
    assert(I[0,1] = 0)
}
```

## Implementation Notes

### Parser Changes

Add to `kleis_parser.rs`:
- Recognize `example` keyword
- Parse string name
- Parse block body with sequential semantics

### Evaluator Changes

Add to `evaluator.rs`:
- `eval_example_block()` - evaluate statements sequentially
- `eval_assert()` - check condition:
  - **Concrete**: Evaluate expression, check if true
  - **Symbolic**: Ask Z3 to prove from axioms in scope
  - **Timeout**: Report "unknown" (not failure)

### CLI Changes

Add to `src/bin/kleis.rs`:
- `kleis test <file>` - run all examples
- `kleis test <file> --example "name"` - run specific example

### DAP Integration

- Example blocks become "launch configurations"
- Each `let` binding is a steppable line
- Variables panel shows current bindings

## Backward Compatibility

No breaking changes. All v0.92 code remains valid.

## Related Documents

- [EXECUTABLE_KLEIS.md](../plans/EXECUTABLE_KLEIS.md) - Design rationale
- [kleis_grammar_v093.ebnf](kleis_grammar_v093.ebnf) - Formal EBNF
- [kleis_grammar_v092.ebnf](kleis_grammar_v092.ebnf) - Previous version

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.93 | 2025-12-24 | Example blocks, assert statement |
| v0.92 | 2025-12-22 | Type-level arithmetic |
| v0.91 | 2025-12-22 | Parameterized type aliases |
| v0.8 | 2025-12-18 | Pattern matching, imports |

