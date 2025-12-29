# Kleis Grammar v0.95 - Big Operator Syntax

**Date:** 2025-12-29  
**Based on:** v0.94 (n-ary product types)

## Overview

v0.95 adds **big operator syntax** for summation, product, integral, and limit:

```kleis
-- Summation: Σ(from, to, body)
Σ(1, n, λ i . f(i))

-- Product: Π(from, to, body)
Π(1, n, λ i . g(i))

-- Integral: ∫(lower, upper, body, var)
∫(0, 1, λ x . x * x, x)

-- Limit: lim(var, target, body)
lim(x, 0, sin(x) / x)
```

## Motivation

Big operators (Σ, Π, ∫, lim) are fundamental to mathematics:
- **Σ** — Summation over a range or set
- **Π** — Product over a range or set  
- **∫** — Definite integral over an interval
- **lim** — Limit as a variable approaches a value

These are **binders** — they introduce bound variables that range over domains.

## Grammar Changes

### New Productions

```ebnf
-- Big operator expressions (function call form)
bigOpExpr
    ::= "Σ" "(" expr "," expr "," expr ")"              -- sum_bounds(body, from, to)
      | "Π" "(" expr "," expr "," expr ")"              -- prod_bounds(body, from, to)
      | "∫" "(" expr "," expr "," expr "," expr ")"     -- int_bounds(body, lower, upper, var)
      | "lim" "(" expr "," expr "," expr ")"            -- lim(body, var, target)
      ;

-- Simple prefix form (existing, unchanged)
prefixExpr
    ::= "Σ" primary       -- Sum(arg)
      | "Π" primary       -- Product(arg)
      | "∫" primary       -- Integrate(arg)
      | "∇" primary       -- gradient(arg)
      | "¬" primary       -- logical_not(arg)
      | "-" primary       -- negate(arg)
      ;
```

### AST Mapping

The parser reorders arguments for equation-editor compatibility:

| Surface Syntax | AST Operation | Argument Order |
|----------------|---------------|----------------|
| `Σ(from, to, body)` | `sum_bounds` | `(body, from, to)` |
| `Π(from, to, body)` | `prod_bounds` | `(body, from, to)` |
| `∫(lower, upper, body, var)` | `int_bounds` | `(body, lower, upper, var)` |
| `lim(var, target, body)` | `lim` | `(body, var, target)` |

This reordering ensures round-trip compatibility:
1. Equation editor creates `int_bounds(integrand, lower, upper, var)`
2. Kleis renderer outputs `∫(lower, upper, integrand, var)`
3. Parser parses back to `int_bounds(integrand, lower, upper, var)`

## Kleis Renderer Output

The Kleis target now outputs parseable function call syntax:

```kleis
-- Before v0.95 (unparseable display notation):
∫_{0}^{1} x² dx

-- v0.95 (parseable):
∫(0, 1, λ x . x * x, x)
```

## Examples

### Summation
```kleis
-- Sum of squares from 1 to n
Σ(1, n, λ i . i * i)

-- Equivalent direct call
sum_bounds(λ i . i * i, 1, n)
```

### Product
```kleis
-- Factorial as product
Π(1, n, λ i . i)

-- Equivalent direct call
prod_bounds(λ i . i, 1, n)
```

### Integral
```kleis
-- Integral of x² from 0 to 1
∫(0, 1, λ x . x * x, x)

-- Equivalent direct call
int_bounds(λ x . x * x, 0, 1, x)
```

### Limit
```kleis
-- Limit of sin(x)/x as x approaches 0
lim(x, 0, sin(x) / x)

-- Equivalent direct call (body first in AST)
lim(sin(x) / x, x, 0)
```

## Z3 Limitations

Z3 is first-order — it cannot quantify over functions. Higher-order axioms like:

```kleis
axiom sum_linearity: ∀(f g : ℤ → ℝ). ∀(a b : ℤ).
    Σ(a, b, λ i . f(i) + g(i)) = Σ(a, b, f) + Σ(a, b, g)
```

are **specifications**, not Z3-verifiable assertions. They are documented as comments in `stdlib/bigops.kleis`.

## Future: Binder-Style Syntax

A more principled future syntax would treat big operators as first-class binders:

```kleis
-- Future (not implemented):
Σ(i : ℤ, 1 ≤ i ≤ n). f(i)
Π(i ∈ S). g(i)
∫(x : ℝ, a ≤ x ≤ b). h(x) dx
lim(x → a). f(x)
```

This would add a `BigOp` variant to the Expression AST, similar to `Quantifier`.

## Files Changed

- `src/kleis_parser.rs` — Added big operator parsing
- `src/render.rs` — Updated Kleis templates for round-trip
- `stdlib/bigops.kleis` — Big operator declarations
- `examples/calculus/sum_examples.kleis` — Test examples
- `examples/calculus/integral_examples.kleis` — Test examples

## Backward Compatibility

✅ Fully backward compatible with v0.94:
- Existing code continues to work
- Simple prefix forms (`∫f`) unchanged
- New function call forms are additive

