# Exponentiation Operator (^) Issues

*Discovered: December 19, 2024*
*Context: REPL satisfiability testing with complex numbers*

## The Problem

The `^` operator for exponentiation is not fully implemented across numeric types.

### Observed Behavior

```
λ> :sat ∃(z : ℂ). z^2 = -1
thread 'main' panicked at vendor/z3/src/func_decl.rs:224:18:
called `Option::unwrap()` on a `None` value
```

### Related Issue: Superscript Notation

```
λ> :sat ∃(z : ℂ). z² = -1
✅ Satisfiable
   Witness: z²!2 -> (-1)   ← WRONG: "z²" parsed as single variable name!
```

### Workaround

Use explicit multiplication:
```
λ> :sat ∃(z : ℂ). z * z = complex(-1, 0)
✅ Satisfiable
   Witness: z = -i
```

## Scope

Check exponentiation (`^`, `**`, `power()`) for all numeric types:

| Type | `x^2` | `power(x, 2)` | Status |
|------|-------|---------------|--------|
| ℕ (Natural) | ? | ? | Needs testing |
| ℤ (Integer) | ? | ? | Needs testing |
| ℚ (Rational) | ? | ? | Needs testing |
| ℝ (Real) | ? | ? | Needs testing |
| ℂ (Complex) | ❌ Crashes | ? | **Broken** |

## Root Cause (Hypothesis)

The Z3 backend's `translate_operation` likely doesn't handle:
1. `power` or `^` for Complex types
2. Falls through to uninterpreted function
3. Z3's `func_decl` lookup fails with `unwrap()` on None

## Files to Investigate

- `src/solvers/z3/backend.rs` - `translate_operation` for power/^
- `src/lowering.rs` - How `^` is lowered for different types
- `src/kleis_parser.rs` - How `^` is parsed (vs superscript Unicode)

## Fix Approach

1. Add explicit `power` handling for each numeric type in Z3 backend
2. For Complex: `power(complex(a,b), n)` = multiply n times (for integer n)
3. For Real: Use Z3's native `Real::power()`
4. Consider: Should `z²` (superscript) be lexed as `power(z, 2)`?

## Priority

Medium - workaround exists (`z * z`), but syntax should work.

