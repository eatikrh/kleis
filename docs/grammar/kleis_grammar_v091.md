# Kleis Grammar v0.91

**Date:** 2025-12-22  
**Status:** Proposed  
**Based on:** v0.8

## Overview

This version adds two features essential for user-defined types:

1. **Parameterized Type Aliases** - `type Name(params) = Type`
2. **Tuple Types** - `(A, B)` syntax in type expressions

These are **backward-compatible** extensions.

## Motivation

Without these features, users cannot define reusable generic types like:

```kleis
-- NOT POSSIBLE in v0.8:
type ComplexMatrix(m, n) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))
type StateSpace(n, m, p) = { A: Matrix(n,n,ℝ), B: Matrix(n,m,ℝ), ... }
```

This blocks control theory, complex algebra, and other domains requiring parameterized types.

## Changes

### 1. Parameterized Type Aliases

**v0.8 (simple aliases only):**
```ebnf
typeAlias ::= "type" identifier "=" type ;
```

**v0.91 (with optional parameters):**
```ebnf
typeAlias ::= "type" identifier [ "(" typeAliasParams ")" ] "=" type ;

typeAliasParams ::= typeAliasParam { "," typeAliasParam } ;
typeAliasParam  ::= identifier [ ":" kind ] ;
```

**Examples:**
```kleis
-- Simple alias (unchanged)
type RealVector = Vector(ℝ)

-- Parameterized alias (NEW)
type ComplexMatrix(m, n) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))

-- With kind annotations
type ComplexMatrix(m: Nat, n: Nat) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))

-- Nested parameters
type StateSpace(n, m, p) = {
    A : Matrix(n, n, ℝ),
    B : Matrix(n, m, ℝ),
    C : Matrix(p, n, ℝ),
    D : Matrix(p, m, ℝ)
}
```

### 2. Tuple Types

**v0.8 (only × syntax):**
```ebnf
type ::= ... | type "×" type
```

**v0.91 (add tuple syntax):**
```ebnf
type ::= ... | tupleType

tupleType ::= "(" type "," type { "," type } ")" ;
```

**Desugaring:**
| Syntax | Desugars To |
|--------|-------------|
| `(A, B)` | `Pair(A, B)` |
| `(A, B, C)` | `Tuple3(A, B, C)` |
| `(A, B, C, D)` | `Tuple4(A, B, C, D)` |
| `(A)` | `A` (grouping, not tuple) |

**Examples:**
```kleis
-- Pair type
operation swap : (A, B) → (B, A)

-- Complex matrix as pair of real matrices
type ComplexMatrix(m, n) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))

-- Multiple return values
operation eigendecomp : Matrix(n, n, ℝ) → (Matrix(n, n, ℝ), [ℂ])
```

**Consistency:** This matches the tuple pattern syntax from v0.8:
```kleis
-- v0.8 pattern (already works)
match pair {
    (a, b) => a + b
}

-- v0.91 type (NEW)
define swap(p : (A, B)) : (B, A) = match p { (a, b) => (b, a) }
```

## Implementation Requirements

### Parser Changes (`src/kleis_parser.rs`)

1. **`parse_type_alias()`** - Add optional parameter parsing after identifier
2. **`parse_type()`** - When seeing `(`, check if it's a tuple (has comma) or grouping

### AST Changes (`src/kleis_ast.rs`)

1. **`TypeAlias`** - Add `params: Vec<TypeAliasParam>` field
2. **`TypeExpr`** - `Product` variant already exists, just need parsing

### Estimated Effort

| Change | Complexity | Lines |
|--------|------------|-------|
| Parameterized type alias (parser) | Medium | ~40 |
| Tuple type syntax (parser) | Low | ~25 |
| AST updates | Low | ~15 |
| Tests | Medium | ~50 |
| **Total** | | ~130 |

## Backward Compatibility

✅ **Fully backward compatible**

- All v0.8 programs remain valid
- New syntax is purely additive
- No existing semantics changed

## Related Features

- **Tuple Patterns (v0.8)** - `(a, b)` in patterns — already works
- **Product Types (existing)** - `A × B` syntax — unchanged
- **Structure Parameters** - `structure Name(params)` — already works

This fills the gap: patterns and structures support parameters, now types do too.

