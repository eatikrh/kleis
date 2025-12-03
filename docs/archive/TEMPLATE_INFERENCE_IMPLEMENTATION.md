# Template Inference Implementation

**Date:** 2024-11-23  
**Status:** Implemented and Working  
**Module:** `src/template_inference.rs`

---

## Overview

Template-based semantic inference is a post-processing step that upgrades flat parsed ASTs to structured operations by pattern matching against known template outputs.

**Key Principle:** If LaTeX parses as a flat chain that matches what a template would generate, infer the semantic structure.

---

## Implemented Patterns

### 1. Double Integrals

**LaTeX:** `\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y`

**Flat Parse:**
```
scalar_multiply chain of 6 terms:
[sub(\iint, D), f(x,y), mathrm(d), x, mathrm(d), y]
```

**Pattern:**
- First term: `sub(\iint, region)`
- Second term: integrand
- Remaining: `mathrm(d) * var` repeated 2 times

**Inferred:**
```rust
double_integral(
  integrand: f(x,y),
  region: D,
  var1: x,
  var2: y
)
```

**Rendering Template (render.rs):**
```rust
typst_templates.insert("double_integral", 
  "integral.double _({right}) {left} dif {idx2} dif {idx3}")
```

**Result:** ✅ `\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y` → `integral.double _(D) f(x,y) dif x dif y`

---

### 2. Triple Integrals

**LaTeX:** `\iiint_{V} f(x,y,z) \, \mathrm{d}x \, \mathrm{d}y \, \mathrm{d}z`

**Pattern:** Same as double integral, but 3 differential variables

**Inferred:**
```rust
triple_integral(integrand, region, var1, var2, var3)
```

**Rendering Template:**
```rust
typst_templates.insert("triple_integral",
  "integral.triple _({right}) {left} dif {idx2} dif {idx3} dif {idx4}")
```

**Result:** ✅ Works

---

### 3. Logical Implication

**LaTeX:** `P \Rightarrow Q`

**Flat Parse:**
```
scalar_multiply chain of 3 terms:
[P, \Rightarrow, Q]
```

**Pattern:**
- Find `\Rightarrow` (or `\Leftarrow`, `\Leftrightarrow`) in chain
- Everything before: left operand
- Everything after: right operand

**Inferred:**
```rust
implies(P, Q)
// or implied_by(P, Q)
// or iff(P, Q)
```

**Rendering Templates:**
```rust
typst_templates.insert("implies", "{left} => {right}")
typst_templates.insert("implied_by", "{left} <= {right}")
typst_templates.insert("iff", "{left} <=> {right}")
```

**Result:** ✅ `P \Rightarrow Q` → `P => Q`

---

### 4. Universal Quantifier

**LaTeX:** `\forall x \colon x \in S`

**Flat Parse:**
```
in_set(
  scalar_multiply(\exists, x, __SPACE__, x),
  S
)
```

**Pattern:**
- Top-level is relational operation (in_set, equals, etc.)
- Left operand is multiplication chain starting with `\forall` or `\exists`
- Extract: quantifier, bound variable, body

**Inferred:**
```rust
forall(
  var: x,
  body: in_set(x, S)
)
```

**Rendering Template:**
```rust
typst_templates.insert("forall", "forall {left} : {right}")
```

**Result:** ✅ `\forall x \colon x \in S` → `forall x : x in S`

---

### 5. Existential Quantifier

**LaTeX:** `\exists x \colon x \in S`

**Pattern:** Same as universal quantifier

**Inferred:** `exists(var, body)`

**Rendering Template:**
```rust
typst_templates.insert("exists", "exists {left} : {right}")
```

**Result:** ✅ Works

---

## Architecture: Inference ↔ Rendering Symmetry

**The Key Insight:** Every inferred operation has a corresponding rendering template.

| Inferred Operation | Rendering Template | Typst Output |
|-------------------|-------------------|--------------|
| `double_integral(f, D, x, y)` | `integral.double _(D) f dif x dif y` | ∬_D f dx dy |
| `triple_integral(f, V, x, y, z)` | `integral.triple _(V) f dif x dif y dif z` | ∭_V f dx dy dz |
| `implies(P, Q)` | `P => Q` | P ⇒ Q |
| `iff(P, Q)` | `P <=> Q` | P ⇔ Q |
| `forall(x, body)` | `forall x : body` | ∀x: body |
| `exists(x, body)` | `exists x : body` | ∃x: body |

**This creates a virtuous cycle:**
1. User inserts template from palette → Structured AST
2. AST renders to LaTeX → Template output
3. LaTeX is parsed → Flat chain
4. Inference recognizes template pattern → Structured AST (round-trip!)

---

## Implementation Details

### Core Functions

**`infer_templates(expr: Expression) -> Expression`**
- Entry point for template inference
- Tries each pattern matcher in priority order
- Returns structured AST if match found, otherwise returns original

**`flatten_multiply(expr: &Expression) -> Vec<Expression>`**
- Converts nested multiplication tree to flat list
- Example: `((a * b) * c)` → `[a, b, c]`

**`rebuild_multiply(terms: &[Expression]) -> Expression`**
- Inverse of flatten: `[a, b, c]` → `((a * b) * c)`
- Used to reconstruct sub-expressions

**`extract_differential_vars(terms, start) -> Vec<Expression>`**
- Finds `mathrm(d) * var` patterns in term sequence
- Returns list of variables

**`is_mathrm_d(expr) -> bool`**
- Checks if expression is `mathrm(d)`

**`is_relational_op(name) -> bool`**
- Checks if operation is relational (=, ∈, ⊆, etc.)

### Pattern Matchers

Each pattern matcher:
- Takes `&Expression` (flat AST)
- Returns `Option<Expression>` (structured AST or None)
- Is independent and composable

**Implemented:**
- `try_infer_double_integral`
- `try_infer_triple_integral`
- `try_infer_logical_implication`
- `try_infer_quantifier`

**Future (deferred):**
- `try_infer_limit` (needs `\to` subscript parsing fix)
- `try_infer_sum_bounds`
- `try_infer_modular_congruence`

---

## Testing

**Test Coverage:** 7 tests, all passing

```rust
#[test] fn test_infer_double_integral() { ... }
#[test] fn test_infer_triple_integral() { ... }
#[test] fn test_infer_logical_implication() { ... }
#[test] fn test_infer_iff() { ... }
#[test] fn test_infer_forall_quantifier() { ... }
#[test] fn test_infer_exists_quantifier() { ... }
#[test] fn test_no_inference_fallback() { ... } // Graceful degradation
```

Each test:
1. Parses LaTeX to flat AST
2. Applies inference
3. Verifies structured operation is created
4. Confirms it renders correctly

---

## Integration with Parser

**Location:** `src/parser.rs`, `parse_latex()` function

```rust
pub fn parse_latex(input: &str) -> Result<Expression, ParseError> {
    let mut parser = Parser::new(input);
    let flat_ast = parser.parse()?;
    
    // Apply template-based semantic inference
    // If inference fails, returns the original flat AST (graceful fallback)
    let structured_ast = crate::template_inference::infer_templates(flat_ast);
    
    Ok(structured_ast)
}
```

**Graceful Fallback:** If no pattern matches, the original flat AST is returned unchanged. This ensures:
- No errors for expressions that don't match patterns
- Backward compatibility with existing code
- Incremental enhancement (add patterns over time)

---

## Parser Enhancements Required

To enable full expression capture, the following were added to `is_term_starter`:

**Symbols:**
- `hbar`, `infty`, `emptyset` (constants)
- `mathrm` (text formatting - critical for `\mathrm{d}x` patterns)

**Logical operators:**
- `Rightarrow`, `Leftarrow`, `Leftrightarrow` (implications)
- `colon` (separator for quantifiers)
- `forall`, `exists` (quantifiers)

This allows the parser to capture full expressions instead of stopping early.

---

## Impact on Issue Count

**Before Template Inference:**
- 28 parsing issues identified
- Many expressions lost semantic structure

**After Template Inference:**
- 22 issues remaining (6 fixed)
- Critical semantic structures now preserved:
  - Multiple integrals with differential variables
  - Logical implications with both operands
  - Quantifiers with bound variables and predicates

**Percentage Improvement:** 21% reduction in issues (6/28)

---

## Future Enhancements

### Priority 1: Limits (High Value)

**Pattern:** `sub(\lim, var) * body` + need to extract target from subscript

**Blocker:** `\to` in subscripts not parsed correctly

**Fix Required:**
1. Update subscript parsing to recognize `var \to target` pattern
2. Add `try_infer_limit` pattern matcher

**Estimated Effort:** 4-6 hours

### Priority 2: Sum/Product with Bounds (Medium Value)

**Pattern:** `sup(sub(\sum, lower), upper) * body`

**Complexity:** Medium (already have sub/sup structure)

**Estimated Effort:** 2-3 hours

### Priority 3: Modular Congruence (Low Value)

**Pattern:** `equiv(a, b) * \pmod{n}`

**Complexity:** Medium (need to detect `\pmod` in chain)

**Estimated Effort:** 2-3 hours

---

## Code Statistics

**Module:** `src/template_inference.rs`
- **Lines of Code:** ~320
- **Functions:** 9 (4 pattern matchers + 5 helpers)
- **Tests:** 7
- **Patterns Implemented:** 6
- **Time to Implement:** ~4 hours
- **Lines per Pattern:** ~50 (very efficient!)

**Maintainability:** High
- Each pattern matcher is independent
- Clear separation of concerns
- Well-tested with graceful fallback

---

## Conclusion

Template-based semantic inference successfully bridges the gap between flat LaTeX parsing and structured semantic representation. The implementation is:

✅ **Practical** - Working code, not just theory  
✅ **Efficient** - ~50 LOC per pattern  
✅ **Robust** - Graceful fallback, no breaking changes  
✅ **Extensible** - Easy to add new patterns  
✅ **Tested** - 7 tests, all passing  
✅ **Aligned** - Inference operations match rendering templates  

This validates the architectural decision documented in ADR-009 and provides a clear path forward for handling more complex LaTeX constructs.

---

**Implementation Date:** 2024-11-23  
**Module:** `src/template_inference.rs` (new)  
**Tests:** 7 passing  
**Issues Fixed:** 6 (out of 28)  
**Zero Compilation Errors:** Maintained  
**Status:** Production-ready

