# Kleis Grammar v0.9 - Nested Quantifiers & Function Types

**Date:** 2025-12-19 (planned)  
**Status:** Draft - Not Yet Implemented  
**Purpose:** Enable Bourbaki-level mathematical expressiveness

## Motivation

Grammar v0.8 cannot express many standard mathematical formulations:

```kleis
// FAILS in v0.8 - quantifier inside conjunction
axiom limit: (x > 0) ∧ (∀(y : ℝ). y > 0)
//                      ^ Parser error: "Expected expression"

// FAILS in v0.8 - function type in quantifier
axiom continuity: ∀(f : ℝ → ℝ). f(0) = f(0)
//                       ^ Parser error: "Expected ')'"
```

This blocks ~80% of Bourbaki expressibility.

---

## New in v0.9

### 1. Quantifiers as Expression Operands

**Grammar change:**

```ebnf
(* v0.8 - quantifiers only at statement level *)
expression ::= term (binop term)*
term ::= literal | identifier | "(" expression ")" | function_call

(* v0.9 - quantifiers allowed in expressions *)
expression ::= term (binop term)*
term ::= literal | identifier | "(" expression ")" | function_call
       | quantifier_expr

quantifier_expr ::= ("∀" | "∃" | "forall" | "exists") 
                    "(" var_list ")" "." expression
```

**Examples now valid:**

```kleis
// Nested quantifier in conjunction
axiom nested: (x > 0) ∧ (∀(y : ℝ). y > 0)

// Epsilon-delta limit definition
axiom epsilon_delta: ∀(L a : ℝ, ε : ℝ). ε > 0 → 
    (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))

// Multiple nested quantifiers
axiom continuity: ∀(ε : ℝ). ε > 0 → (∃(δ : ℝ). δ > 0 ∧ 
    (∀(x : ℝ). condition(x) → result(x)))
```

### 2. Function Types in Type Annotations

**Grammar change:**

```ebnf
(* v0.8 - only simple types in annotations *)
type_annotation ::= simple_type | parameterized_type
simple_type ::= "ℝ" | "ℕ" | "ℤ" | "ℂ" | "Bool" | identifier
parameterized_type ::= identifier "(" type_list ")"

(* v0.9 - function types allowed *)
type_annotation ::= function_type | simple_type | parameterized_type
function_type ::= type_annotation "→" type_annotation
                | "(" type_list ")" "→" type_annotation
```

**Examples now valid:**

```kleis
// Function from reals to reals
axiom func: ∀(f : ℝ → ℝ). f(0) = f(0)

// Binary function
axiom binary: ∀(g : ℝ × ℝ → ℝ). g(a, b) = g(b, a)

// Higher-order function
axiom compose: ∀(f : ℝ → ℝ, g : ℝ → ℝ). compose(f, g) = λ x . f(g(x))

// Curried function
axiom curry: ∀(f : ℝ → ℝ → ℝ). f(a)(b) = f(a)(b)
```

---

## Implementation Notes

### File: `src/kleis_parser.rs`

**Change 1: `parse_primary()` or `parse_expression()`**

When parsing the RHS of binary operators (∧, ∨, →), check for quantifier keywords:

```rust
// Pseudocode
fn parse_primary(&mut self) -> Result<Expression, ParseError> {
    match self.peek() {
        Token::Forall | Token::Exists => self.parse_quantifier_expr(),
        // ... existing cases
    }
}
```

**Change 2: `parse_type_annotation()`**

After parsing a simple type, check for `→`:

```rust
// Pseudocode
fn parse_type_annotation(&mut self) -> Result<Type, ParseError> {
    let left = self.parse_simple_type()?;
    if self.peek() == Token::Arrow {
        self.advance();
        let right = self.parse_type_annotation()?; // Right-associative
        Ok(Type::Function(Box::new(left), Box::new(right)))
    } else {
        Ok(left)
    }
}
```

---

## Test Cases

These must all pass after v0.9 implementation:

```kleis
// Test 1: Quantifier inside conjunction
structure Test1 {
    axiom nested: (x > 0) ∧ (∀(y : ℝ). y > 0)
}

// Test 2: Function type in quantifier
structure Test2 {
    axiom func: ∀(f : ℝ → ℝ). f(0) = f(0)
}

// Test 3: Epsilon-delta (the ultimate goal)
structure Limits {
    axiom epsilon_delta: ∀(L a : ℝ, ε : ℝ). ε > 0 → 
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))
}

// Test 4: Topology continuity
structure Topology {
    axiom continuity: ∀(f : X → Y, V : Set(Y)). 
        is_open(V) → is_open(preimage(f, V))
}
```

---

## Impact

| Before (v0.8) | After (v0.9) |
|---------------|--------------|
| ~20% Bourbaki expressible | ~80% Bourbaki expressible |
| Cannot write ε-δ definitions | Full analysis expressible |
| Cannot quantify over functions | Higher-order math works |

---

## Migration

No breaking changes. All v0.8 code remains valid in v0.9.

---

## Related Documents

- [Capability Assessment](../CAPABILITY_ASSESSMENT.md) - Why this matters
- [Grammar v0.8](kleis_grammar_v08.md) - Current grammar
- [NEXT_SESSION.md](../NEXT_SESSION.md) - Implementation roadmap

