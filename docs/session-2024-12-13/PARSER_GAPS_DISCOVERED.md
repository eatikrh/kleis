# Parser Gaps Discovered by Level 2 Tests

## Design Principle

**The official Kleis grammar (`docs/grammar/kleis_grammar_v07.ebnf`) is the source of truth.**

**Grammar v0.7 BREAKING CHANGES (Dec 13, 2024):**
- `∂f/∂x` and `df/dx` notation **REMOVED**
- Use Mathematica-style: `D(f, x)` for partial, `Dt(f, x)` for total derivative
- Added `Limit(body, var, target)` for limits

1. **Parser** must implement what the grammar defines
2. **Renderer** must output grammar-conforming syntax
3. If renderer outputs non-grammar syntax → **fix the renderer**
4. If parser doesn't support grammar syntax → **fix the parser**

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

| # | Expression | Rendered | Error | Status |
|---|------------|----------|-------|--------|
| 1 | Integral w/ bounds | `∫_{{0}}^{{1}} f dx` | `Unexpected character: '{'` | Pending |
| 2 | Summation | `Σ_{{i=1}}^{{n}} a_i` | `Unexpected character: '{'` | Pending |
| 3 | Product | `Π_{{i=1}}^{{n}} a_i` | `Unexpected character: '{'` | Pending |
| 4 | Limit | `Limit(f, x, 0)` | Needs function call parsing | **Fixed in v0.7** |
| 5 | Partial derivative | `D(f, x)` | Function call works | **Fixed in v0.7** |
| 6 | Total derivative | `Dt(y, t)` | Function call works | **Fixed in v0.7** |
| 7 | Set membership | `x ∈ S` | `Unexpected character: '∈'` | Pending |
| 8 | Bra notation | `⟨φ\|` | `Expected expression` | Pending |
| 9 | Ket notation | `\|ψ⟩` | `Expected expression` | Pending |
| 10 | Placeholder | `□` | `Expected expression` | Pending |
| 11 | Forall | `∀(P(x)). x` | `Expected expression` | Pending |
| 12 | Exists | `∃(P(x)). x` | `Expected expression` | Pending |

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

## Design Principle: Renderer Shows What Parser Must Support

**CRITICAL:** The Kleis renderer outputs the FULL semantic representation.
The parser must adapt to support it, NOT the other way around.

This approach ensures we discover ALL parser gaps rather than hiding them
by dumbing down the renderer output.

**Renderer outputs (canonical Kleis syntax):**
```
∫_{0}^{1} f dx      ← Parser must learn to parse this
Σ_{i=1}^{n} a_i     ← Parser must learn to parse this
∂f/∂x               ← Parser must learn to parse this
x ∈ S               ← Parser must learn to parse this
□                   ← Parser must learn to parse this
```

**NOT:** Simplify renderer to match parser limitations.

---

## Action Items (Parser Must Catch Up)

### ✅ RESOLVED in Grammar v0.7

5. [x] **Partial derivatives** → Use `D(f, x)` (Mathematica-style)
6. [x] **Total derivatives** → Use `Dt(f, x)` (Mathematica-style)  
7. [x] **Limits** → Use `Limit(body, var, target)` (function-call style)

### High Priority (Calculus)

1. [ ] **Add subscript/superscript parsing** for `_{...}^{...}` syntax
   - Needed for: `∫_{a}^{b}`, `Σ_{i=1}^{n}`, `Π_{i=1}^{n}`
   - OR use function-call alternatives: `Integrate(f, x, a, b)`, `Sum(expr, i, 1, n)`, `Product(expr, i, 1, n)`

2. [ ] **Add `Σ` as prefix operator** with subscript/superscript support
   - Grammar: `summation ::= "Σ" [ subscript ] [ superscript ] expression`

3. [ ] **Add `Π` as prefix operator** with subscript/superscript support
   - Grammar: `productNotation ::= "Π" [ subscript ] [ superscript ] expression`

### Medium Priority (Logic & Sets)

6. [ ] **Add `∈` as infix operator**
   - `x ∈ S` → `in_set(x, S)`

7. [ ] **Add `□` placeholder parsing**
   - Grammar already has: `placeholder ::= "□"`

### Lower Priority (Quantum)

8. [ ] **Add bra notation** `⟨...|`
9. [ ] **Add ket notation** `|...⟩`

### Verification

10. [ ] Re-run Level 2 tests after each parser fix
11. [ ] All 26 tests should eventually pass

