# Kleis Grammar v0.7 - Mathematica-Style Calculus

**Date:** 2024-12-13  
**Status:** Official  

## Breaking Changes

### Derivative Notation

**REMOVED** (no longer valid Kleis):
```
∂f/∂x, df/dx, ∂²f/∂x∂y
```

**USE INSTEAD** (Mathematica-style):
```kleis
D(f, x)         // Partial derivative ∂f/∂x
D(f, x, y)      // Mixed partial ∂²f/∂x∂y  
Dt(f, x)        // Total derivative df/dx (chain rule)
```

## Rationale

1. **Parsing Simplicity**: Function-call syntax is unambiguous
2. **Mathematica Compatibility**: Follows established conventions
3. **Extensibility**: Easy to add options (order, direction)
4. **Structural Editor**: Visual `∂f/∂x` renders to `D(f, x)` for validation

## New in v0.7

### Limit Notation
```kleis
Limit(f, x, 0)        // lim_{x→0} f
Limit(sin(x)/x, x, 0) // = 1
```

### Function-Call Calculus

| Visual | Function-Call |
|--------|---------------|
| `∫ f dx` | `Integrate(f, x)` |
| `∫_a^b f dx` | `Integrate(f, x, a, b)` |
| `Σ_{i=1}^n` | `Sum(expr, i, 1, n)` |
| `Π_{i=1}^n` | `Product(expr, i, 1, n)` |
| `lim_{x→a}` | `Limit(f, x, a)` |
| `∂f/∂x` | `D(f, x)` |
| `df/dx` | `Dt(f, x)` |

## Structural Editor Compatibility

Visual rendering maps to function-call syntax for verification:

| User Sees | Kleis Output |
|-----------|--------------|
| ∂f/∂x | `D(f, x)` |
| ∫₀¹ f dx | `Integrate(f, x, 0, 1)` |
| lim_{x→0} | `Limit(f, x, 0)` |

