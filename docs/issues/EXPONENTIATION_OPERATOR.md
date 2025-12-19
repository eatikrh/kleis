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

### Option 1: Z3 Backend Enhancement
1. Add explicit `power` handling for each numeric type in Z3 backend
2. For Real: Use Z3's native `Real::power()`
3. For Complex: Translate to repeated multiplication or encode formula

### Option 2: Axiomatic Definition (Preferred)

Define `power` axiomatically in stdlib - no Z3 native support needed:

```kleis
// stdlib/power.kleis

structure Power(T) over Monoid(T) {
    // power(x, n) for natural number exponent
    operation power : T × ℕ → T
    
    // Base cases
    axiom power_zero : ∀(x : T). power(x, 0) = e
    axiom power_one : ∀(x : T). power(x, 1) = x
    
    // Recursive definition
    axiom power_succ : ∀(x : T)(n : ℕ). 
        power(x, n + 1) = x * power(x, n)
    
    // Useful properties (derivable)
    axiom power_add : ∀(x : T)(m n : ℕ). 
        power(x, m + n) = power(x, m) * power(x, n)
    
    axiom power_mul : ∀(x : T)(m n : ℕ). 
        power(x, m * n) = power(power(x, m), n)
}

// Complex-specific power
implements Power(ℂ) {
    // power(z, n) = z * z * ... * z (n times)
    operation power = complex_power
}
```

### Option 3: Parser Enhancement
- Lex `z²` (superscript) as `power(z, 2)`
- Desugar `x^n` to `power(x, n)` before type inference

## Benefits of Axiomatic Approach

1. **Works for any Monoid** - not just numbers
2. **Z3 verifies properties** - even without native support
3. **Consistent with Kleis philosophy** - define, don't hardcode
4. **Extensible** - users can implement for custom types

## Testing Results (Dec 19, 2024)

Tested axiomatic `power` definition in REPL:

```kleis
structure RealPower {
    operation power : ℝ × ℕ → ℝ
    axiom power_zero : ∀(x : ℝ). power(x, 0) = 1
    axiom power_one : ∀(x : ℝ). power(x, 1) = x
    axiom power_succ : ∀(x : ℝ)(n : ℕ). power(x, n + 1) = x * power(x, n)
}
```

| Query | Result | Notes |
|-------|--------|-------|
| `power(2, 0) = 1` | ✅ Valid | Concrete value |
| `power(3, 1) = 3` | ✅ Valid | Concrete value |
| `power(5, 2) = 25` | ✅ Valid | Z3 chains axioms for concrete |
| `∀(x). power(x, 2) = x*x` | ❌ Invalid | Z3 finds countermodels |

### E-Matching Limitation

Z3 can find models that satisfy the axioms but not the intended property.
For universally quantified properties over uninterpreted functions, Z3's
E-matching may construct countermodels where `power` behaves unexpectedly.

**What works:** Concrete expressions (power(5, 2) = 25)
**What fails:** Universal quantification (∀x. power(x, 2) = x*x)

### Implications

1. **Concrete evaluation:** Works - use for calculation
2. **Theorem proving:** Limited - needs induction or native support
3. **For Complex numbers:** `z * z` workaround is reliable

## Priority

Medium - workaround exists (`z * z`), but syntax should work.

