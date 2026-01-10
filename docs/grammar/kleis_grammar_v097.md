# Kleis Grammar v0.97 - ASCII Logical Operators

**Date:** 2026-01-05  
**Based on:** v0.96 (Named Arguments)  
**EBNF:** `kleis_grammar_v097.ebnf`

## Summary

v0.97 adds **ASCII equivalents for logical operators** that work in all expression contexts:

| Unicode | ASCII | Description |
|---------|-------|-------------|
| `∧` | `and` | Logical conjunction |
| `∨` | `or` | Logical disjunction |
| `¬` | `not` | Logical negation |

## Motivation

Previously, `and`/`or` only worked inside `let` bindings, not in general expressions:

```kleis
// v0.96: ❌ This failed
assert(P and Q)

// v0.96: ✅ This worked (Unicode required)
assert(P ∧ Q)
```

This was a barrier for beginners coming from Python, C, or other languages where `and`/`or` are standard keywords.

## New Syntax (v0.97)

```kleis
// Both now work identically:
assert(P and Q)   // ASCII
assert(P ∧ Q)     // Unicode

// In axioms:
structure Logic {
    axiom excluded_middle: forall P : Bool . P or not P
    axiom double_negation: forall P : Bool . not (not P) = P
}

// In example blocks:
example "De Morgan" {
    assert(not (P and Q) = (not P) or (not Q))
}
```

## Grammar Change

### Conjunction (parse_conjunction)

**Previous (v0.96):**
```
Check for '∧' only
```

**New (v0.97):**
```
Check for '∧' OR keyword "and"
```

### Disjunction (parse_disjunction)

**Previous (v0.96):**
```
Check for '∨' only
```

**New (v0.97):**
```
Check for '∨' OR keyword "or"
```

### Negation (unaryOp)

**Previous (v0.96):**
```
unaryOp ::= "-" | "¬" | "not"
```

**New (v0.97):** Same, but `not` now recognized in all contexts (not just let bindings).

## Precedence

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 3 | `∨`, `or` | Left |
| 4 | `∧`, `and` | Left |
| 5 | `¬`, `not` (prefix) | Prefix |

This matches the Unicode operators exactly.

## Reserved Keywords

With v0.97, the following are **reserved keywords** and cannot be used as identifiers:

- `and`
- `or`
- `not`

```kleis
// ❌ No longer valid (was valid in v0.96 if not in let context)
define and = 5   // Error: 'and' is a reserved keyword

// ✅ Use different names
define logical_and = 5
```

## Backward Compatibility

| Aspect | Compatible? |
|--------|-------------|
| Unicode operators (`∧`, `∨`, `¬`) | ✅ Unchanged |
| `and`/`or` in let bindings | ✅ Still works |
| `and`/`or` as identifiers | ❌ Now reserved |

## Implementation

The parser was modified in `src/kleis_parser.rs`:

1. **`parse_conjunction`**: Added check for `and` keyword
2. **`parse_disjunction`**: Added check for `or` keyword
3. **`parse_unary`**: Already supported `not`, now works consistently

## Examples

### Basic Logic

```kleis
example "basic logic" {
    let P = True in
    let Q = False in
    assert(P and True = P)
    assert(P or False = P)
    assert(not False = True)
}
```

### De Morgan's Laws

```kleis
structure DeMorgan {
    axiom law1: forall P : Bool . forall Q : Bool .
        not (P and Q) = (not P) or (not Q)
    
    axiom law2: forall P : Bool . forall Q : Bool .
        not (P or Q) = (not P) and (not Q)
}
```

### Mixed Unicode and ASCII

```kleis
// Both work, can be mixed (though not recommended for readability)
structure Mixed {
    axiom example: forall P : Bool . forall Q : Bool .
        (P and Q) ∨ (¬P ∧ not Q)
}
```

## Version History

- **v0.97 (2026-01-05)**: ASCII logical operators (`and`, `or`, `not`) work everywhere
- **v0.96 (2026-01-01)**: Named arguments for function calls
- **v0.95 (2025-12-29)**: Big operator syntax (Σ, Π, ∫, lim)
- **v0.94 (2025-12-26)**: N-ary product types
- **v0.93 (2025-12-24)**: Example blocks and assert statements

