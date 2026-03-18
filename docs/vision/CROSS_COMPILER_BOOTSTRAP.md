# Kleis Cross-Compiler Bootstrap Vision

*Discovered: December 19, 2024*
*Origin: kleis_in_kleis.kleis experiment*

## The Key Insight

A Kleis program IS its source code:

```kleis
define fib_source : String = "define fib(n) = if n < 2 then n else fib(n-1) + fib(n-2)"
define fibonacci_program : Program = Source(fib_source)
```

This means a **cross-compiler is just a Kleis function**:

```kleis
define kleis_to_rust : Program → String
```

## The Bootstrap Process

```
┌─────────────────────────────────────────────────────────────────┐
│  STAGE 1: Write cross-compiler in Kleis                        │
│                                                                 │
│    kleis_to_rust.kleis  (a Kleis program)                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  STAGE 2: Run it on itself (in Kleis interpreter)              │
│                                                                 │
│    kleis kleis_to_rust.kleis < kleis_to_rust.kleis              │
│                                    ↓                            │
│                           kleis_to_rust.rs                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  STAGE 3: Compile the Rust                                      │
│                                                                 │
│    rustc kleis_to_rust.rs -o kleis_to_rust                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  RESULT: Native binary cross-compiler!                          │
│                                                                 │
│    ./kleis_to_rust my_program.kleis > my_program.rs             │
│                                                                 │
│    ⚡ Fast (native speed)                                        │
│    ✓ Verified (Z3 proved Stage 2)                               │
│    🔄 Self-hosting (compiled itself)                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Why This Works

1. **Homoiconicity**: `Program = Source(String)` - code is data
2. **String operations**: Kleis has Z3-backed string manipulation
3. **AST as data types**: Grammar defined in `data Expression = ...`
4. **Pattern matching**: Transform AST nodes naturally
5. **Z3 verification**: Prove compiler correctness with axioms

## Compiler Structure

```kleis
// The cross-compiler is a pure function
define kleis_to_rust(p : Program) : String =
    match p {
        Source(code) => emit_rust(parse(code))
    }

// Code generation is string manipulation
define emit_rust(ast : List(Declaration)) : String =
    match ast {
        [] => ""
      | [DDefine(name, params, ret, body) | rest] =>
            concat(
                "fn ", name, "(", emit_params(params), ") -> ", emit_type(ret),
                " { ", emit_expr(body), " }\n",
                emit_rust(rest)
            )
    }
```

## Verification

The killer feature: Z3 can verify the compiler is correct!

```kleis
structure CompilerCorrectness {
    // Semantics preservation
    axiom semantics_preserved : ∀(p : Program, input : Value).
        eval_kleis(p, input) = eval_rust(kleis_to_rust(p), input)
    
    // Well-formedness: output is valid Rust
    axiom output_valid : ∀(p : Program).
        is_valid_kleis(p) → is_valid_rust(kleis_to_rust(p))
}
```

## Multiple Targets

Same pattern for any target:

```kleis
define kleis_to_rust : Program → String
define kleis_to_python : Program → String  
define kleis_to_javascript : Program → String
define kleis_to_wasm : Program → String
define kleis_to_c : Program → String
```

All are Kleis programs. All can be verified. All can bootstrap themselves.

## Historical Significance

This is the dream of formal methods since the 1970s:
- **CompCert** (2006): Verified C compiler in Coq - took years, thousands of lines
- **Kleis approach**: Verified cross-compiler in Kleis - could be hundreds of lines

The difference: Kleis was designed from the ground up for this.
- Bourbaki-style foundations
- Z3 integration
- String operations as first-class
- `Program = Source(String)` - the key insight

## Next Steps

1. Implement `parse : String → Option(List(Declaration))` in Kleis
2. Implement `emit_rust : List(Declaration) → String`
3. Write verification axioms
4. Bootstrap!

## See Also

- `docs/grammar/kleis_in_kleis.kleis` - The experiment that discovered this
- `experiment/self-describing-grammar` branch - Working code
- ADR-003: Self-Hosting Strategy

