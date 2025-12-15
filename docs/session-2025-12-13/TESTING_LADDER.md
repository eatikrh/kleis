# Testing Ladder: Kleis Renderer → Parser → Z3

## The Three-Level Testing Strategy

This document describes our incremental testing approach for the Kleis renderer.

```
Level 1: Naive Rendering Tests (CURRENT)
────────────────────────────────────────
AST → Render → Kleis String → Assert contains expected symbols
                    │
                    │ "Does it look right?"
                    ▼
Level 2: Parse Round-Trip Tests (NEXT)
────────────────────────────────────────
AST → Render → Kleis String → PARSE → AST' → Compare
                    │                    │
                    │                    └── Reveals: What's missing in PARSER
                    ▼
Level 3: Verification Tests (FINAL)
────────────────────────────────────────
AST → Render → Kleis String → Parse → AST' → Z3 Verify → Result
                                                │
                                                └── Reveals: What's missing in Z3 TRANSLATION
```

---

## Level 1: Naive Rendering Tests (Current State)

**Location:** `tests/kleis_renderer_test.rs`

**What it tests:**
- Kleis renderer produces strings containing expected Unicode symbols
- Basic template substitution works

**Example:**
```rust
#[test]
fn kleis_render_integral_with_bounds() {
    let expr = op("int_bounds", vec![o("f"), c("0"), c("1"), o("x")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);
    
    assert!(out.contains("∫"));   // Has integral symbol
    assert!(out.contains("dx"));  // Has differential
}
```

**Limitations:**
- Doesn't verify the output is valid Kleis syntax
- Doesn't verify the parser can read it back
- Doesn't verify Z3 can process it

---

## Level 2: Parse Round-Trip Tests (Next Step)

**What it will test:**
- Rendered Kleis string can be parsed by `kleis_parser`
- Parser produces equivalent AST

**Example (proposed):**
```rust
#[test]
fn roundtrip_integral() {
    let original_ast = op("int_bounds", vec![o("f"), c("0"), c("1"), o("x")]);
    
    // Render to Kleis syntax
    let kleis_text = render_expression(&original_ast, &ctx, &RenderTarget::Kleis);
    // e.g., "∫_{0}^{1} f dx"
    
    // Parse back to AST
    let parsed_ast = kleis_parser::parse(&kleis_text);
    
    // This will FAIL if parser doesn't support the syntax!
    assert!(parsed_ast.is_ok(), "Parser failed on: {}", kleis_text);
    
    // Compare (may need structural comparison, not exact equality)
    assert_equivalent(&original_ast, &parsed_ast.unwrap());
}
```

**What failures reveal:**
- `KleisParseError` → Parser doesn't support this syntax
- Need to add grammar rules for: `lim`, `∫` with bounds, `Σ` with bounds, etc.

**Action items from failures:**
1. Add missing syntax to `kleis_parser.rs`
2. Update grammar documentation
3. Add parser tests

---

## Level 3: Z3 Verification Tests (Final Step)

**What it will test:**
- Parsed AST can be translated to Z3
- Z3 can verify/simplify the expression

**Example (proposed):**
```rust
#[test]
fn verify_sum_identity() {
    // Sum from i=1 to n of 1 = n
    let sum_expr = op("sum_bounds", vec![c("1"), o("i=1"), o("n")]);
    
    // Render → Parse
    let kleis_text = render_expression(&sum_expr, &ctx, &RenderTarget::Kleis);
    let parsed = kleis_parser::parse(&kleis_text).unwrap();
    
    // Build verification expression: Sum(...) = n
    let equation = op("equals", vec![parsed, o("n")]);
    
    // Verify with Z3
    let result = verifier.verify(&equation);
    
    // This will FAIL if Z3 backend doesn't handle Sum!
    assert!(result.is_valid() || result.is_unknown());
}
```

**What failures reveal:**
- `Z3 Error: Unknown operation "Sum"` → Need to add to Z3 backend
- Need to define semantics for calculus operations

**Action items from failures:**
1. Add operation to `src/solvers/z3/backend.rs`
2. Define how operation translates to Z3 (likely uninterpreted function)
3. Consider adding axioms for known identities

---

## Gap Discovery Matrix

| Level | Tests | Discovers Gaps In |
|-------|-------|-------------------|
| 1 | Render assertions | Template definitions, glyph mappings |
| 2 | Parse round-trip | Parser grammar, syntax support |
| 3 | Z3 verification | Z3 backend, operation translation |

---

## Current Status

| Calculus Op | Level 1 | Level 2 | Level 3 |
|-------------|---------|---------|---------|
| `∫` (integral) | ✅ | ❓ parser? | ❓ Z3? |
| `Σ` (sum) | ✅ | ❓ parser? | ❓ Z3? |
| `Π` (product) | ✅ | ❓ parser? | ❓ Z3? |
| `lim` (limit) | ✅ | ❌ not in grammar | ❓ Z3? |
| `∂/∂x` (partial) | ✅ | ❓ parser? | ✅ (D) |
| `d/dx` (total) | ✅ | ❓ parser? | ✅ (Dt) |
| `∇` (gradient) | ✅ | ✅ parser | ✅ Z3 |

---

## Next Steps

1. **Implement Level 2 tests** for all calculus operations
2. **Fix parser gaps** revealed by Level 2 failures
3. **Implement Level 3 tests** once parsing works
4. **Fix Z3 gaps** revealed by Level 3 failures

This incremental approach ensures we discover ALL gaps systematically rather than guessing what might be missing.

