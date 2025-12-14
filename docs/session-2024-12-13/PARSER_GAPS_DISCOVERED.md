# Parser Gaps Discovered by Level 2 Tests

## Summary

Level 2 round-trip tests revealed **11 parser gaps**. These must be fixed to complete the Equation Editor → Kleis verification pipeline.

## Test Results

### ✅ PASSED (Parser supports)

| Expression | Rendered | Parsed As |
|------------|----------|-----------|
| Gradient | `∇f` | `gradient(f)` |
| Simple integral | `∫f` | `Integrate(f)` |
| Implies | `(P ⟹ Q)` | `implies(P, Q)` |
| Addition | `(a + b)` | `plus(a, b)` |
| Multiplication | `(a × b)` | `times(a, b)` |
| Power | `x^2` | `power(x, 2)` |
| Equals | `a = b` | `equals(a, b)` |
| Less than | `a < b` | `less_than(a, b)` |
| Greek letters | `α`, `π` | Objects |
| Conditional | `if x > 0 then x else -x` | Conditional AST |
| Let binding | `let x = 5 in (x + x)` | Let AST |

### ❌ FAILED (Parser gaps)

| # | Expression | Rendered | Error |
|---|------------|----------|-------|
| 1 | Integral w/ bounds | `∫_{{0}}^{{1}} f dx` | `Unexpected character: '{'` |
| 2 | Summation | `Σ_{{i=1}}^{{n}} a_i` | `Unexpected character: '{'` |
| 3 | Product | `Π_{{i=1}}^{{n}} a_i` | `Unexpected character: '{'` |
| 4 | Limit | `lim_{{x→0}} f(x)` | `Unexpected character: '{'` |
| 5 | Partial derivative | `∂f/∂x` | `Expected expression` |
| 6 | Set membership | `x ∈ S` | `Unexpected character: '∈'` |
| 7 | Bra notation | `⟨φ\|` | `Expected expression` |
| 8 | Ket notation | `\|ψ⟩` | `Expected expression` |
| 9 | Placeholder | `□` | `Expected expression` |
| 10 | Forall | `∀(P(x)). x` | `Expected expression` |
| 11 | Exists | `∃(P(x)). x` | `Expected expression` |

---

## Root Cause Analysis

### Issue 1: Subscript/Superscript Syntax Mismatch

**Renderer outputs LaTeX-style:**
```
∫_{0}^{1} f dx
Σ_{i=1}^{n} a_i
lim_{x→0} f(x)
```

**Parser expects Kleis grammar:**
```
∫_0^1 f dx          # Subscript without braces
Σ_(i=1)^n a_i       # Maybe with parens?
```

**Fix options:**
1. Change renderer to output parser-compatible syntax
2. Change parser to accept LaTeX-style subscripts
3. Use function call syntax: `Integrate(f, x, 0, 1)`

### Issue 2: Missing Prefix Operators

Parser doesn't recognize these as prefix operators:
- `Σ` (summation)
- `Π` (product)
- `∂` alone (partial, without being part of D(f,x))
- `lim`

The parser HAS:
- `∇` (gradient)
- `∫` (integral)
- `-` (negate)
- `¬` (logical not)

### Issue 3: Missing Infix Operators

Parser doesn't recognize:
- `∈` (set membership)

### Issue 4: Missing Special Syntax

Parser doesn't recognize:
- `□` (placeholder)
- `⟨...|` (bra notation)
- `|...⟩` (ket notation)

### Issue 5: Quantifier Rendering Mismatch

**Renderer outputs:**
```
∀(P(x)). x
∃(P(x)). x
```

**Parser expects:**
```
∀(x : T). P(x)    # Variable first, then body
∃(x : T). P(x)    # Variable first, then body
```

---

## Recommended Fixes

### Priority 1: Fix Renderer (Quick Win)

Change templates to output parser-compatible syntax:

```rust
// BEFORE (LaTeX-style)
"int_bounds" → "∫_{{{lower}}}^{{{upper}}} {integrand} d{int_var}"

// AFTER (Function-call style - parser already supports)
"int_bounds" → "Integrate({integrand}, {int_var}, {lower}, {upper})"
```

This sidesteps the subscript parsing problem entirely.

### Priority 2: Add Missing Prefix Operators to Parser

```rust
// In kleis_parser.rs parse_primary()
if self.peek() == Some('Σ') {
    self.advance();
    let arg = self.parse_primary()?;
    return Ok(Expression::Operation {
        name: "Sum".to_string(),
        args: vec![arg],
    });
}

if self.peek() == Some('Π') {
    self.advance();
    let arg = self.parse_primary()?;
    return Ok(Expression::Operation {
        name: "Product".to_string(),
        args: vec![arg],
    });
}
```

### Priority 3: Add Missing Infix Operators

```rust
// Add ∈ as infix operator returning in_set
if self.peek() == Some('∈') {
    self.advance();
    let right = self.parse_term()?;
    return Ok(op("in_set", vec![left, right]));
}
```

### Priority 4: Add Placeholder Support

```rust
// In parse_primary()
if self.peek() == Some('□') {
    self.advance();
    return Ok(Expression::Placeholder { id: 0, hint: "".to_string() });
}
```

### Priority 5: Fix Quantifier Rendering

Change template to match parser expectation:
```rust
// BEFORE
"forall" → "∀({var}). {body}"

// AFTER  
"forall" → "∀({var}). {body}"  // Same, but args need reordering!
```

Actually the issue is the AST args order:
- Renderer receives: `forall(var, body)`
- Template uses: `∀({var}). {body}` → outputs `∀(x). P(x)`
- Parser expects: `∀(x). P(x)` but parses variable as `x`, body as `P(x)`

So the rendering IS correct, but the parse result puts things in different arg positions.

---

## Decision: Function-Call Style for Calculus

Given the complexity of subscript/superscript parsing, recommend:

**Use function-call style for all calculus operations:**

| Visual | Kleis Syntax |
|--------|--------------|
| ∫₀¹ f dx | `Integrate(f, x, 0, 1)` |
| Σᵢ₌₁ⁿ aᵢ | `Sum(a_i, i, 1, n)` |
| Πᵢ₌₁ⁿ aᵢ | `Product(a_i, i, 1, n)` |
| limₓ→₀ f(x) | `Limit(f(x), x, 0)` |
| ∂f/∂x | `D(f, x)` |

This is the Mathematica style and:
- Already works with current parser for function calls
- Unambiguous semantics
- Easy for Z3 translation
- Matches existing `D(f, x)` and `Integrate(f, x)` patterns

---

## Action Items

1. [ ] Update Kleis templates to use function-call style
2. [ ] Add `Σ`, `Π` as prefix operators (for simple cases)
3. [ ] Add `∈` as infix operator
4. [ ] Add `□` placeholder support
5. [ ] Add bra/ket support (lower priority)
6. [ ] Update tests to verify fixes

