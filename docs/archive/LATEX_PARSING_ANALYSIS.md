# LaTeX Parsing Analysis: Structural vs. Flat Representation

**Date:** 2025-11-23  
**Context:** Analysis of LaTeX expressions that have parsing inconsistencies due to flat symbol approach  
**Related:** ADR-009 (Architectural Decision on Quantifier Parsing)

---

## Executive Summary

Out of ~100 gallery items tested, **28 expressions** exhibit parsing issues when round-tripping through LaTeX → AST → LaTeX. These issues stem from the fundamental tension between:
- **LaTeX's flat/linear notation** (sequence of symbols)
- **Kleis AST's structural/tree representation** (operations with arguments)

Most issues are **expected consequences** of the architectural decision to use flat symbol parsing for ambiguous constructs. This analysis documents which expressions are affected and why.

---

## Issue Categories

### Category 1: Quantifiers & Logic (Flat Symbol Approach)

**Root Cause:** `\forall`, `\exists`, `\Rightarrow` treated as symbols, not operators with scope.

| Expression | Original LaTeX | Parsed As | Issue |
|------------|----------------|-----------|-------|
| Universal quantifier | `\forall x \colon x \in S` | `\forall * x` | Loses `: x ∈ S` body |
| Existential quantifier | `\exists x \colon x \in S` | `\exists * x` | Loses `: x ∈ S` body |
| Logical implication | `P \Rightarrow Q` | `P` | Loses `⇒ Q` |
| Congruence modulo | `a \equiv b \pmod{n}` | `equiv(a, b)` | Loses `\pmod{n}` |

**Impact:** High for logic-heavy documents. Quantifier scope is lost.

**Workaround:** Use programmatic AST construction: `forall(o("x"), in_set(o("x"), o("S")))`.

**Future Fix:** Template-based semantic inference (pattern match LaTeX against template outputs).

---

### Category 2: Limits & Bounds (Subscript Content Loss)

**Root Cause:** Subscript `{x \to 0}` parses as just `x` because `\to` stops parsing.

| Expression | Original LaTeX | Parsed Subscript | Issue |
|------------|----------------|------------------|-------|
| Limit | `\lim_{x \to 0} f(x)` | `{x}` | Loses `→ 0` target |
| Limsup | `\limsup_{x \to \infty} f(x)` | `{x}` | Loses `→ ∞` target |
| Liminf | `\liminf_{x \to a} f(x)` | `{x}` | Loses `→ a` target |
| Euler product | `\prod_{p\,\text{prime}}` | `{p}` | Loses `\text{prime}` label |

**Impact:** Medium. Limits render but without target values shown.

**Root Issue:** `\to` is parsed as `Object("\\to")` (arrow symbol), not as part of limit syntax.

**Workaround:** Programmatic construction: `op("limit", vec![body, var, target])`.

**Potential Fix:** Special parsing for limit subscripts to recognize `var \to target` pattern.

---

### Category 3: Multi-Expression Sequences (Quad Problem)

**Root Cause:** `\quad` skips and continues parsing, causing implicit multiplication chain.

| Expression | Original LaTeX | Parsed As | Issue |
|------------|----------------|-----------|-------|
| Variance/covariance | `\mathrm{Var}(X)\quad\mathrm{Cov}(X, Y)` | `mathrm(Var) * X` | Only first expr, loses second |
| Real/imaginary | `\mathrm{Re}(z)\quad\mathrm{Im}(z)` | `mathrm(Re) * z` | Same |
| Trig functions | `\cos(x)\quad\tan(x)\quad\sin(x)` | `cos(x) * tan(x) * sin(x)` | Multiplication chain |
| Hyperbolic | `\sinh(x)\quad\cosh(x)` | `sinh * x * cosh * x` | Function names not parsed |
| Logarithms | `\ln(x)\quad\log(x)` | `ln(x) * log(x)` | Multiplication chain |
| Floor/ceiling | `\lfloor x \rfloor\quad\lceil x \rceil` | `floor(x)` | Only first, loses second |
| Inverse trig | `\arcsin(x)\quad\arccos(x)\quad\arctan(x)` | `arcsin * x * arccos * x * ...` | Not parsed as functions |
| Reciprocal trig | `\sec(x)\quad\csc(x)\quad\cot(x)` | `sec * x * csc * x * ...` | Same |

**Impact:** Low to Medium. Visually renders correctly (spaces between terms), but semantically wrong (treated as products).

**Current Workaround:** Split into separate gallery entries to avoid `\quad`.

**Architectural Question:** Should `\quad` separate expressions create a list/sequence node, or is multiplication acceptable?

**Note:** For trig functions without parentheses (e.g., `\sinh x`), parser doesn't recognize them as function calls. Need handlers for these commands to consume arguments.

---

### Category 4: Text Mode in Math (Character Splitting)

**Root Cause:** `\text{if}` parses as `text(o("if"))`, then `o("if")` renders as `i * f` (implicit multiplication of letters).

| Expression | Original LaTeX | Parsed/Rendered | Issue |
|------------|----------------|-----------------|-------|
| Text "if" | `\text{if }` | `i \, f` | Characters split |
| Text "otherwise" | `\text{otherwise}` | `o \, t \, h \, e \, r \, w \, i \, s \, e` | Characters split |

**Impact:** High for piecewise functions with text labels.

**Root Issue:** `text` operation contains `Object("if")`. When rendered, `Object` goes through `latex_to_typst_symbol` which doesn't quote multi-letter text.

**Fix:** The `text` template already uses quotes: `"{arg}"`. But `{arg}` is replaced with rendered content. If `arg` is `Object("if")`, it renders as `i f` (with implicit mult), then gets quoted as `"i f"`.

**Proper Fix:** `text` operation should preserve raw string, not parse it. Or `render_expression` should detect `text` operation and not parse the argument.

---

### Category 5: Special Operators

**Root Cause:** Various parsing issues with specific operators.

| Expression | Original LaTeX | Issue |
|------------|----------------|-------|
| Factorial | `n!` | `!` not recognized as postfix operator, lost |
| Outer product | `outer_product(\psi, \phi)` | Function name split: `o * u * t * e * r * ...` |
| Curl | `\nabla \times \mathbf{B}` | `\mathbf` appears twice (bug) |
| Set membership | `x \in \mathbb{R}` | Renders as `in_set(x, ℝ)` instead of `x ∈ ℝ` |
| Not equal | `a \neq b` | Renders as `not_equal(a, b)` instead of `a ≠ b` |
| Proportional | `F \propto ma` | Renders as `proportional(F, m*a)` instead of `F ∝ ma` |

**Impact:** Medium. Some are bugs (outer_product name), others are template choices.

**Fixes Needed:**
- Add `!` as postfix operator (factorial)
- Fix `outer_product` to not be parsed as identifier
- Investigate `\mathbf` duplication bug
- Ensure infix operators (`\in`, `\neq`, `\propto`) parse correctly

---

### Category 6: Integral Variables

**Root Cause:** Differential variables at end of integrals are lost during parsing.

| Expression | Original LaTeX | Issue |
|------------|----------------|-------|
| Mellin integral | `... \, \mathrm{d}x` | Loses `dx` at end |
| Double integral | `\iint_{D} f(x,y) \, \mathrm{d}D \, \mathrm{d}y` | Loses `dD dy` |
| Triple integral | `\iiint_{V} f(x,y,z) \, \mathrm{d}V \, \mathrm{d}y \, \mathrm{d}z` | Loses `dV dy dz` |

**Impact:** Medium. Integrals render but without differential notation.

**Root Issue:** `\mathrm{d}x` at the end is parsed as separate expression, not part of integral.

**Architectural Question:** Should integrals consume trailing `\mathrm{d}var` patterns? Or is this too magical?

---

### Category 7: Minor Issues (Typos, Formatting)

| Expression | Issue | Severity |
|------------|-------|----------|
| Matrix 3x3 | `a_{13}` appears twice (should be `a_{21}`) | Typo in original |
| Matrix with ellipsis | `a_{1n}` parses as `a_{1 * n}` | Subscript implicit mult |
| Vmatrix 3x3 | `4` appears as `3` (typo?) | Data issue |
| Prime notation | `y'` → `y^{'}` | Acceptable (equivalent) |
| Spacing | `\partial\,L` vs `\partial \, L` | Cosmetic (whitespace) |
| Sequence with ellipsis | `1, 2, 3, \ldots, n` | Only parses `1` (commas?) |

---

## Architectural Implications

### What This Analysis Reveals

1. **~72% of expressions parse correctly** (72 out of 100)
2. **~28% have issues**, mostly in 3 categories:
   - Quantifiers/logic (scope ambiguity)
   - Multi-expression sequences (list vs. product)
   - Text mode (string vs. symbol)

3. **Most issues are cosmetic** when rendered (visual output is acceptable)
4. **Semantic structure is lost** for quantifiers, limits, and sequences

### Design Principles Validated

✅ **Flat parsing is correct for ambiguous syntax**  
   - `\forall x P(x)` has no clear scope boundary
   - Forcing structure would be arbitrary

✅ **Templates provide structure**  
   - `forall(var, body)` template in editor palette has clear boundaries
   - Users insert structured templates, not arbitrary LaTeX

✅ **LaTeX import is best-effort**  
   - Handles common cases well
   - Degrades gracefully for complex cases
   - Not the primary workflow

### Recommendations

**For Structural Editor Implementation:**

1. **Palette templates** should provide structured operations:
   - `forall(var, body)` with two placeholders
   - `limit(body, var, target)` with three placeholders
   - `double_integral(integrand, region, var1, var2)` with clear structure

2. **LaTeX import** should remain flat/simple:
   - Parse what we can
   - Don't force structure on ambiguous syntax
   - Document limitations

3. **Future enhancement: Template-based inference**:
   - After flat parsing, pattern-match against template outputs
   - If `\forall x \colon P` matches `forall` template pattern, upgrade to structured
   - Requires pattern matching engine (future work)

4. **Fix critical bugs**:
   - Text mode character splitting (high priority)
   - Factorial `!` operator (medium priority)
   - `outer_product` name parsing (bug)
   - Integral variable handling (medium priority)

---

## Specific Fixes Needed

### High Priority (Breaks Functionality)

1. **Text mode preservation**: `\text{if}` should not split into `i * f`
   - Fix: `text` operation should store raw string, not parse content
   - Or: Special handling in renderer to not parse text content

2. **Factorial operator**: `n!` should parse as `factorial(n)`
   - Fix: Add `!` to postfix operators in `parse_postfix`

3. **Function name parsing**: `outer_product(...)` shouldn't parse as `o*u*t*e*r*...`
   - Fix: This is actually a LaTeX rendering issue - `outer_product` is operation name, not LaTeX command
   - Gallery should use `\ket{\psi}\bra{\phi}` notation instead

### Medium Priority (Semantic Loss)

4. **Limit subscripts**: `\lim_{x \to 0}` should preserve target
   - Fix: Special parsing for limit subscripts (detect `\to` pattern)
   - Or: Accept limitation, rely on programmatic construction

5. **Integral variables**: `\int ... \mathrm{d}x` should capture `dx`
   - Fix: Integrals could consume trailing `\mathrm{d}var` patterns
   - Or: Accept limitation (visual rendering is correct anyway)

6. **Quantifier scope**: `\forall x \colon P(x)` should parse as operation
   - Fix: Implement template-based inference (future work)
   - Or: Accept flat parsing (current decision)

### Low Priority (Cosmetic)

7. **Quad sequences**: `A \quad B` creates multiplication
   - Current: Split into separate gallery entries
   - Future: Consider sequence/list node type

8. **Spacing normalization**: `\partial\,L` vs `\partial \, L`
   - Acceptable: Semantically equivalent

---

## Testing Methodology

**Script:** `/tmp/analyze_mismatches.rs`

```rust
// For each gallery item:
// 1. Parse original LaTeX → AST
// 2. Render AST → LaTeX
// 3. Compare normalized strings
// 4. Report mismatches
```

**Normalization:** Remove spaces and `\,` for comparison.

**Result:** 28 mismatches identified (documented above).

---

## Recommendations for Editor Implementation

### Palette Design

The editor palette should provide **structured templates** for operations that have ambiguous LaTeX syntax:

1. **Quantifiers:**
   ```
   Template: forall(var: □, body: □)
   Renders: ∀ var : body
   ```

2. **Limits:**
   ```
   Template: limit(body: □, var: □, target: □)
   Renders: lim_{var→target} body
   ```

3. **Integrals:**
   ```
   Template: integral(integrand: □, lower: □, upper: □, var: □)
   Renders: ∫_{lower}^{upper} integrand dvar
   ```

These templates have **clear boundaries** and **explicit placeholders**, avoiding the ambiguity of parsing arbitrary LaTeX.

### LaTeX Import Strategy

When importing LaTeX:

1. **Parse with flat symbol approach** (current)
2. **Accept limitations** for ambiguous constructs
3. **Document known issues** (this file)
4. **Provide manual upgrade path**: User can select flat symbols and convert to structured template

### Future: Template-Based Inference

**Phase 1:** Implement pattern matcher
- For each operation template, generate regex/pattern for its LaTeX output
- Example: `forall` template → pattern `\\forall\s+(\w+)\s*\\colon\s*(.+)`

**Phase 2:** Post-process parsed AST
- Detect flat symbol sequences that match template patterns
- Upgrade to structured operations
- Example: `[\forall, x, \colon, x, \in, S]` → `forall(x, in_set(x, S))`

**Phase 3:** Handle ambiguity
- When multiple templates could match, use heuristics or user confirmation
- Preserve flat structure as fallback

---

## Conclusion

The flat symbol parsing approach is **architecturally sound** for handling arbitrary LaTeX. The 28 identified issues are mostly **expected trade-offs** between simplicity and semantic richness.

**Key Insight:** The structural editor's primary workflow is **template insertion**, not LaTeX parsing. Templates provide clear structure. LaTeX import is a convenience feature that works well for common cases and degrades gracefully for complex ones.

**Next Steps:**
1. Fix high-priority bugs (text mode, factorial)
2. Document limitations in user-facing docs
3. Implement palette with structured templates
4. Consider template-based inference as future enhancement

---

## Proof of Concept: Template-Based Inference for Double Integrals

**Date:** 2025-11-23  
**Status:** Feasibility demonstrated, implementation deferred

### The Challenge

LaTeX: `\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y`

**Flat parsing produces:**
```
scalar_multiply(
  scalar_multiply(
    scalar_multiply(
      scalar_multiply(
        scalar_multiply(
          sub(\iint, D),
          function_call(f, x, y)
        ),
        mathrm(d)
      ),
      x
    ),
    mathrm(d)
  ),
  y
)
```

**Desired structured AST:**
```
double_integral(
  integrand: function_call(f, x, y),
  region: D,
  var1: x,
  var2: y
)
```

### Pattern Matching Algorithm

**Step 1: Flatten multiplication chain**
```rust
fn flatten_multiply(expr: &Expression) -> Vec<Expression> {
    match expr {
        Operation { name: "scalar_multiply", args } if args.len() == 2 => {
            let mut result = flatten_multiply(&args[0]);
            result.extend(flatten_multiply(&args[1]));
            result
        }
        _ => vec![expr.clone()],
    }
}
```

Result: `[sub(\iint, D), f(x,y), mathrm(d), x, mathrm(d), y]` (6 terms)

**Step 2: Pattern match**
```rust
fn infer_double_integral(expr: &Expression) -> Option<Expression> {
    let terms = flatten_multiply(expr);
    
    // Pattern: sub(\iint, region) * integrand * mathrm(d) * var1 * mathrm(d) * var2
    
    // 1. Check first term is sub(\iint, region)
    let region = match &terms[0] {
        Operation { name: "sub", args } if args.len() == 2 => {
            match &args[0] {
                Object(s) if s == "\\iint" => args[1].clone(),
                _ => return None,
            }
        }
        _ => return None,
    };
    
    // 2. Integrand is term 1
    let integrand = terms[1].clone();
    
    // 3. Extract variables: look for mathrm(d) * var pattern
    let mut variables = Vec::new();
    let mut i = 2;
    
    while i + 1 < terms.len() {
        let is_diff = matches!(&terms[i], 
            Operation { name: "mathrm", args } 
            if args.len() == 1 && matches!(&args[0], Object(s) if s == "d")
        );
        
        if is_diff {
            variables.push(terms[i + 1].clone());
            i += 2;
        } else {
            break;
        }
    }
    
    // 4. Verify we have exactly 2 variables
    if variables.len() == 2 {
        Some(Operation {
            name: "double_integral".to_string(),
            args: vec![integrand, region, variables[0].clone(), variables[1].clone()],
        })
    } else {
        None
    }
}
```

**Step 3: Apply inference**
```rust
match parse_latex(latex) {
    Ok(flat_ast) => {
        let structured_ast = infer_double_integral(&flat_ast)
            .unwrap_or(flat_ast);
        // Use structured_ast for rendering
    }
}
```

### Results

**Input:** `\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y`

**Flat parse:** 6-term multiplication chain (as shown above)

**Pattern match:** ✓ Success
- Detected: `sub(\iint, D)` at start
- Extracted: region = `D`, integrand = `f(x,y)`
- Found: 2 differential patterns (`mathrm(d) * x`, `mathrm(d) * y`)

**Inferred AST:** `double_integral(f(x,y), D, x, y)`

**Re-rendered LaTeX:** `\iint_{D} f(x, y) \, \mathrm{d}x \, \mathrm{d}y` ✓

### Feasibility Assessment

**Difficulty: MEDIUM (Definitely Achievable)**

**Pros:**
- ✅ Pattern is unambiguous (`\iint` + 2 differentials = double integral)
- ✅ ~80 lines of code for proof-of-concept
- ✅ Successfully reconstructs structure
- ✅ Generalizable to triple integrals, limits, etc.

**Cons:**
- Requires flattening multiplication chains
- Need to handle edge cases (missing differentials, extra terms)
- Must maintain pattern matchers for each template

**Estimated Implementation Effort:**

| Component | Lines of Code | Time Estimate |
|-----------|---------------|---------------|
| Core infrastructure (flatten, match framework) | ~200 | 4-6 hours |
| Double/triple integral patterns | ~100 | 2-3 hours |
| Limit patterns (lim, limsup, liminf) | ~150 | 3-4 hours |
| Sum/product with bounds | ~100 | 2-3 hours |
| Quantifiers (forall, exists) | ~100 | 2-3 hours |
| Testing & edge cases | ~200 | 4-6 hours |
| **Total** | **~850 lines** | **2-3 days** |

### Generalization to Other Templates

**Similar patterns that would benefit:**

1. **Triple integral:** `\iiint_{V} ... \mathrm{d}x \mathrm{d}y \mathrm{d}z`
   - Pattern: `sub(\iiint, region) * integrand * (mathrm(d) * var){3}`
   - Complexity: Same as double integral

2. **Limits:** `\lim_{x \to 0} f(x)`
   - Pattern: `sub(\lim, x) * f(x)` + unparsed `\to 0`
   - Complexity: Higher (need to handle unparsed remainder)
   - Alternative: Fix parser to handle `\to` in subscripts

3. **Sum with bounds:** `\sum_{n=1}^{\infty} a_n`
   - Pattern: `sup(sub(\sum, equals(n, 1)), \infty) * a_n`
   - Complexity: Medium (already have subscript/superscript structure)

4. **Quantifiers:** `\forall x \colon P(x)`
   - Pattern: `\forall * x * \colon * P(x)`
   - Complexity: Medium (need to determine where predicate ends)

### Implementation Strategy

**Phase 1: Infrastructure (Day 1)**
```rust
// src/parser_inference.rs (new file)

pub fn infer_templates(expr: Expression) -> Expression {
    // Try each pattern matcher in priority order
    try_infer_double_integral(&expr)
        .or_else(|| try_infer_triple_integral(&expr))
        .or_else(|| try_infer_limit(&expr))
        .or_else(|| try_infer_sum_bounds(&expr))
        .or_else(|| try_infer_quantifier(&expr))
        .unwrap_or(expr) // Keep flat if no pattern matches
}

fn flatten_multiply(expr: &Expression) -> Vec<Expression> { ... }
fn is_mathrm_d(expr: &Expression) -> bool { ... }
fn extract_differential_vars(terms: &[Expression], start: usize) -> Vec<Expression> { ... }
```

**Phase 2: Pattern Matchers (Days 1-2)**
```rust
fn try_infer_double_integral(expr: &Expression) -> Option<Expression> {
    // Pattern: sub(\iint, region) * integrand * mathrm(d) * var1 * mathrm(d) * var2
    // (Implemented in POC above)
}

fn try_infer_triple_integral(expr: &Expression) -> Option<Expression> {
    // Pattern: sub(\iiint, region) * integrand * (mathrm(d) * var){3}
    // Nearly identical to double integral
}

fn try_infer_limit(expr: &Expression) -> Option<Expression> {
    // Pattern: sub(\lim, var) * body
    // Challenge: Need to extract target from subscript (requires fixing \to parsing)
}
```

**Phase 3: Integration (Day 2-3)**
```rust
// In parse_latex()
pub fn parse_latex(input: &str) -> Result<Expression, ParseError> {
    let mut parser = Parser::new(input);
    let flat_ast = parser.parse()?;
    
    // Apply template inference
    let structured_ast = infer_templates(flat_ast);
    
    Ok(structured_ast)
}
```

**Phase 4: Testing (Day 3)**
- Test each pattern with variations
- Handle edge cases (missing differentials, extra terms)
- Ensure no false positives

### Advantages of This Approach

1. **Non-breaking**: Flat parsing still works, inference is optional post-processing
2. **Incremental**: Can add pattern matchers one at a time
3. **Testable**: Each pattern matcher is independent
4. **Fallback**: If inference fails, keep flat structure (graceful degradation)
5. **Extensible**: Users could define custom patterns for their templates

### Limitations

1. **Heuristic-based**: Might misidentify patterns in edge cases
2. **Maintenance**: Each new template needs a pattern matcher
3. **Performance**: Extra pass over AST (negligible for typical expressions)
4. **Ambiguity**: Some patterns might match multiple templates (need priority order)

### Recommendation

**Implement template inference for high-value cases:**
- ✅ Double/triple integrals (unambiguous pattern)
- ✅ Sum/product with bounds (already have sub/sup structure)
- ⚠️ Limits (need to fix `\to` in subscripts first)
- ❌ Quantifiers (scope ambiguity, defer to future)

**Priority 1:** Integrals (demonstrated feasibility, clear benefit)  
**Priority 2:** Limits (after fixing subscript parsing)  
**Priority 3:** Quantifiers (requires scope resolution heuristics)

---

**Analysis Date:** 2025-11-23  
**Analyzed Items:** ~100 gallery expressions  
**Issues Found:** 28 (28%)  
**Critical Issues:** 3 (text mode, factorial, outer_product)  
**Acceptable Limitations:** 25 (semantic loss with visual correctness)  
**POC Status:** Template inference proven feasible (~80 LOC, 2-3 days for full implementation)

