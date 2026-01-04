# Expression-Level Source Spans

> **Status: ✅ IMPLEMENTED (by design)** (Dec 2024)  
> Spans are implemented for `Operation` type expressions only - this is a **deliberate design decision**.
> Debugger steps on executable expressions, not literals/atoms.
> LSP diagnostics and DAP breakpoints work correctly with this design.
> Full spans on ALL variants was considered unnecessary overhead.

**Original Status:** Planned  
**Priority:** High (Infrastructure)  
**Estimated Effort:** 3-5 days  
**Date:** December 2024

## Executive Summary

Adding source location spans (`SourceSpan`) to every `Expression` node in the AST is foundational infrastructure that enables precise error reporting, accurate debugging, and professional-grade IDE features. This document captures the technical analysis, benefits, implementation approach, and scope of changes.

## Current State

### What We Have

1. **`SourceSpan` struct** - Already defined in `src/kleis_parser.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq)]
   pub struct SourceSpan {
       pub line: u32,    // 1-based
       pub column: u32,  // 1-based
   }
   ```

2. **Parser tracks position** - `KleisParser` already maintains `line` and `column` fields and has a `current_span()` method.

3. **Function-level spans** - `FunctionDef` and `Closure` have `span: Option<SourceSpan>` for function definition locations.

### What's Missing

`Expression` variants do not carry source location information. When we evaluate or type-check an expression, we don't know where in the source file it came from.

**Current evaluator code:**
```rust
fn eval_internal(&self, expr: &Expression, depth: usize) -> Result<Expression, String> {
    // Hardcoded location - we don't know the real position!
    let location = SourceLocation::new(1, 1);
    // ...
}
```

## Benefits

### 1. LSP Diagnostics (Precise Error Locations)

**Without spans:**
```
Error: Cannot add Matrix and Scalar
  at line 5
```

**With spans:**
```
Error: Cannot add Matrix and Scalar
  --> file.kleis:5:12
   |
 5 |   let result = matrix + scalar
   |                ^^^^^^^^^^^^^^^ expected Matrix, found Scalar
```

### 2. LSP Hover (Sub-Expression Types)

With spans, the LSP can determine exactly which sub-expression the cursor is over and show its type:

```kleis
let x = foo(bar(baz))
            ^^^
            │
            └── Hovering here shows: bar(baz) : Int
```

### 3. LSP Go to Definition

When the user clicks on an identifier, we can determine exactly which one (in case of multiple on the same line) and navigate to its definition.

### 4. DAP Breakpoints (Line-Level Stopping)

**Current limitation:** Breakpoints only work at function entry points because we only have `Closure.span`.

**With expression spans:** Can stop at any expression on any line.

```kleis
define compute(x) =
    let a = expensive(x) in    ← Can set breakpoint here
    let b = transform(a) in    ← And here
    a + b                      ← And here
```

### 5. DAP Variables (Expression Evaluation at Location)

The debugger can show the value of the specific sub-expression at the current location, not just top-level bindings.

### 6. Error Recovery and Partial Parsing

With spans, the parser can report multiple errors in a single file by knowing exactly where each construct begins and ends.

## Technical Approach

### Option A: Add `span` to Every Expression Variant (Recommended)

Convert the `Expression` enum so every variant carries a span:

**Before:**
```rust
pub enum Expression {
    Const(String),
    String(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
    // ...
}
```

**After:**
```rust
pub enum Expression {
    Const { value: String, span: Option<SourceSpan> },
    String { value: String, span: Option<SourceSpan> },
    Object { name: String, span: Option<SourceSpan> },
    Operation { name: String, args: Vec<Expression>, span: Option<SourceSpan> },
    // ...
}
```

**Pros:**
- Direct access to span from any expression
- Pattern matching naturally includes span
- No wrapper type needed

**Cons:**
- Large number of code changes (2,112 usages across 42 files)
- All pattern matches need updating

### Option B: Wrapper Struct

```rust
pub struct Spanned<T> {
    pub inner: T,
    pub span: Option<SourceSpan>,
}

pub type SpannedExpr = Spanned<Expression>;
```

**Pros:**
- Expression enum unchanged
- Generic, reusable for other AST nodes

**Cons:**
- Changes type signature everywhere
- Extra indirection when accessing expression data
- Similar scope of changes as Option A

### Recommendation

**Option A** is preferred because:
1. More idiomatic Rust (spans are part of the data, not wrapped around it)
2. Pattern matching is cleaner
3. Consistent with how `FunctionDef` already has `span`

## Scope of Changes

### Files Affected

| File Category | File Count | Match Count | Notes |
|---------------|------------|-------------|-------|
| `src/evaluator.rs` | 1 | 547 | Core evaluation logic |
| `src/kleis_parser.rs` | 1 | 197 | Must populate spans |
| `src/pretty_print.rs` | 1 | 129 | Printing (ignores spans) |
| `src/type_inference.rs` | 1 | 92 | Can use spans for errors |
| `src/pattern_matcher.rs` | 1 | 118 | Pattern matching |
| Other src files | 37 | ~1,029 | Various usages |
| **Total** | **42** | **2,112** | |

### Expression Variants to Modify

| Variant | Current Form | Change Required |
|---------|--------------|-----------------|
| `Const` | `Const(String)` | → `Const { value: String, span: Option<SourceSpan> }` |
| `String` | `String(String)` | → `StringLit { value: String, span: Option<SourceSpan> }` |
| `Object` | `Object(String)` | → `Object { name: String, span: Option<SourceSpan> }` |
| `Operation` | `Operation { name, args }` | Add `span` field |
| `Placeholder` | `Placeholder { id, hint }` | Add `span` field |
| `Match` | `Match { scrutinee, cases }` | Add `span` field |
| `List` | `List(Vec<Expression>)` | → `List { elements: Vec<Expression>, span: Option<SourceSpan> }` |
| `Quantifier` | `Quantifier { ... }` | Add `span` field |
| `Conditional` | `Conditional { ... }` | Add `span` field |
| `Let` | `Let { ... }` | Add `span` field |
| `Ascription` | `Ascription { ... }` | Add `span` field |
| `Lambda` | `Lambda { ... }` | Add `span` field |

**Note:** Renamed `String` variant to `StringLit` to avoid confusion with `std::string::String`.

### Helper Methods to Update

```rust
impl Expression {
    // These need span parameters:
    pub fn constant(s: impl Into<String>) -> Self
    pub fn string(s: impl Into<String>) -> Self
    pub fn object(s: impl Into<String>) -> Self
    pub fn operation(name: impl Into<String>, args: Vec<Expression>) -> Self
    // ... etc
}
```

Suggest adding overloaded versions:
```rust
impl Expression {
    // Without span (for tests, generated code)
    pub fn constant(s: impl Into<String>) -> Self {
        Self::constant_at(s, None)
    }
    
    // With span (for parser)
    pub fn constant_at(s: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Expression::Const { value: s.into(), span }
    }
}
```

## Implementation Plan

### Phase 1: Expression Enum Changes (Day 1)

1. Modify `src/ast.rs`:
   - Convert all tuple variants to struct variants
   - Add `span: Option<SourceSpan>` to each variant
   - Update helper methods with `_at` variants

2. Run `cargo check` - collect all compile errors (expected: ~2,000)

### Phase 2: Mechanical Updates (Days 2-3)

For each file with errors:

1. Update pattern matches to include `span` (can use `..` to ignore):
   ```rust
   // Before
   Expression::Const(s) => ...
   
   // After
   Expression::Const { value: s, .. } => ...
   ```

2. Update expression construction to include `span: None`:
   ```rust
   // Before
   Expression::Const("42".to_string())
   
   // After
   Expression::Const { value: "42".to_string(), span: None }
   ```

**Tip:** Can use `sed` or IDE refactoring for bulk changes.

### Phase 3: Parser Updates (Day 3-4)

Update `src/kleis_parser.rs` to populate spans:

```rust
fn parse_primary(&mut self) -> Result<Expression, String> {
    let span = self.current_span();  // Capture before consuming
    
    if let Some(num) = self.parse_number() {
        return Ok(Expression::Const { 
            value: num, 
            span: Some(span) 
        });
    }
    // ...
}
```

### Phase 4: Span Propagation (Day 4)

1. Update `src/debug.rs` `SourceLocation` to use real spans
2. Update evaluator's `eval_internal` to extract span from expression
3. Update LSP diagnostic handlers to use spans

### Phase 5: Testing (Day 5)

1. Verify all tests pass
2. Add tests for span accuracy
3. Test LSP error locations
4. Test DAP breakpoint hitting

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Large number of mechanical changes | Use regex/sed for bulk updates; IDE refactoring |
| Tests might hardcode expression structure | Use `..` pattern to ignore span in tests |
| Serialization format changes | `serde(skip_serializing_if = "Option::is_none")` for backwards compat |
| Performance (larger AST nodes) | `SourceSpan` is 8 bytes; minimal impact |

## Success Criteria

1. ✅ All Expression variants have `span: Option<SourceSpan>`
2. ✅ Parser populates spans for all expressions
3. ✅ LSP diagnostics show precise column ranges
4. ✅ DAP breakpoints work at line level (not just function entry)
5. ✅ All existing tests pass
6. ✅ New tests verify span accuracy

## Related Work

- **ADR-014:** Hindley-Milner Type System (will benefit from spans for error locations)
- **Phase 6:** Debugging (DAP) - blocked on this for line-level breakpoints
- **LSP Enhancement:** Hover and diagnostics will use spans

## Appendix: Regex Patterns for Bulk Updates

### Pattern Match Updates

```bash
# Const(s) → Const { value: s, .. }
s/Expression::Const\((\w+)\)/Expression::Const { value: $1, .. }/g

# Object(s) → Object { name: s, .. }
s/Expression::Object\((\w+)\)/Expression::Object { name: $1, .. }/g
```

### Construction Updates

```bash
# Expression::Const(x) → Expression::Const { value: x, span: None }
s/Expression::Const\(([^)]+)\)/Expression::Const { value: $1, span: None }/g
```

---

*This document serves as the technical specification for implementing expression-level source spans in Kleis.*

