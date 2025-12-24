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

Assert statements verify conditions within example blocks:

```kleis
assert(x = 4)                           // equality
assert(x ≈ 4.0)                         // approximate equality
assert(valid(p))                        // predicate
assert(x = 4, "x should equal 4")       // with message
```

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
    ::= "assert" "(" expression [ "," string ] ")"
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

### Physics Example

```kleis
structure Pendulum(length: ℝ, mass: ℝ) {
    operation period : ℝ
    
    axiom period_formula: ∀(p : Pendulum).
        period(p) = 2 * π * sqrt(p.length / g)
}

example "pendulum period calculation" {
    let p = Pendulum(length: 1.0, mass: 0.5)
    let T = period(p)
    
    // With g ≈ 9.81, period ≈ 2.006 seconds
    assert(T ≈ 2.006)
}
```

### Linear Algebra Example

```kleis
example "matrix multiplication dimensions" {
    let A = Matrix(2, 3, [[1,2,3], [4,5,6]])
    let B = Matrix(3, 2, [[7,8], [9,10], [11,12]])
    let C = multiply(A, B)
    
    // Result is 2×2
    assert(C.rows = 2)
    assert(C.cols = 2)
    
    // Verify specific values
    assert(C[0,0] = 58)   // 1*7 + 2*9 + 3*11
    assert(C[0,1] = 64)   // 1*8 + 2*10 + 3*12
}
```

### Approximate Equality

```kleis
example "euler identity" {
    let i = Complex(0, 1)
    let result = exp(i * π)
    
    // e^(iπ) ≈ -1 (within floating-point tolerance)
    assert(result ≈ Complex(-1, 0))
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
- `eval_assert()` - check condition, report failures

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

