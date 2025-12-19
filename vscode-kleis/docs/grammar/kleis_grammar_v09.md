# Kleis Grammar v0.9 - Nested Quantifiers & Function Types

**Date:** 2025-12-19  
**Status:** ✅ Implemented  
**Purpose:** Enable Bourbaki-level mathematical expressiveness

## New in v0.9

### 1. Quantifiers as Expression Operands

Quantifiers (`∀`, `∃`) can now appear inside logical expressions:

```kleis
// Quantifier inside conjunction
axiom bounded: (x > 0) ∧ (∀(y : ℝ). y = y)

// Quantifier inside implication
axiom dense: ∀(a b : ℝ). a < b → (∃(q : ℚ). a < q ∧ q < b)

// Epsilon-delta limit definition
axiom limit: ∀(L a : ℝ, ε : ℝ). ε > 0 → 
    (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))
```

### 2. Function Types in Type Annotations

Function types using `→` or `->` are now supported in quantifier type annotations:

```kleis
// Simple function type
axiom func: ∀(f : ℝ → ℝ). f(0) = f(0)

// Multiple function variables
axiom compose: ∀(f : ℝ → ℝ, g : ℝ → ℝ). compose(f, g) = compose(f, g)

// Curried function type (right-associative)
axiom curried: ∀(f : ℝ → ℝ → ℝ). f = f

// Topology continuity
axiom continuity: ∀(f : X → Y, V : Set(Y)). 
    is_open(V) → is_open(preimage(f, V))
```

## Grammar Changes

### EBNF Additions

```ebnf
(* v0.8: term only had these *)
term ::= literal | identifier | "(" expression ")" | function_call

(* v0.9: quantifier_expr added as valid term *)
term ::= literal | identifier | "(" expression ")" | function_call
       | quantifier_expr

quantifier_expr ::= ("∀" | "∃" | "forall" | "exists") 
                    "(" var_list ")" "." expression

(* v0.8: type_annotation was simple *)
type_annotation ::= simple_type | parameterized_type

(* v0.9: function types added *)
type_annotation ::= function_type | simple_type | parameterized_type
function_type   ::= type_annotation ("→" | "->") type_annotation
```

## Impact

| Metric | Before (v0.8) | After (v0.9) |
|--------|---------------|--------------|
| Bourbaki Expressibility | ~20% | ~80% |
| Epsilon-delta Limits | ❌ | ✅ |
| Topology Definitions | ❌ | ✅ |
| Function Types | ❌ | ✅ |

## Migration

No breaking changes. All v0.8 code remains valid in v0.9.

## Related Documents

- [Main Grammar v0.9 Doc](../../../docs/grammar/kleis_grammar_v09.md)
- [Capability Assessment](../../../docs/CAPABILITY_ASSESSMENT.md)

