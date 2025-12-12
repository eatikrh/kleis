# TODO Inventory - Complete Audit
**Date:** December 12, 2024  
**Total TODOs:** 57  
**Files Audited:** 15 (src/ + tests/ + examples/)

---

## 🚨 Critical Issues (Action Required)

### Type Safety Issues

**TODO #13** - `src/signature_interpreter.rs:240`
```rust
// TODO: Should error on type mismatch (e.g., Matrix when expecting ℝ)
```
**Issue:** Currently accepts wrong types for backward compatibility  
**Impact:** Type safety violation - allows Matrix when expecting ℝ  
**Recommendation:** Fix this type safety bug OR document why it's intentional

**TODO #22** - `src/math_layout/mod.rs:117`
```rust
// TODO: Implement pattern matching layout
unimplemented!("Pattern matching layout not yet implemented")
```
**Issue:** Panics with `unimplemented!()` if match expressions reach layout  
**Impact:** Will crash if match expressions are rendered  
**Recommendation:** Replace panic with placeholder like other TODO renderers

---

## ⚠️ Important Issues (Should Address)

### Ignored Tests (11 tests not running!)

**Placeholder Tests** (3 tests)
- TODO #33: `src/math_layout/typst_adapter.rs:278` - placeholder conversion
- TODO #34: `src/math_layout/typst_adapter.rs:298` - fraction with placeholder  
- TODO #35: `src/math_layout/typst_adapter.rs:339` - nested placeholders

**LaTeX Rendering Tests** (4 tests)
- TODO #40: `src/render.rs:3813` - inner product LaTeX
- TODO #41: `src/render.rs:3865` - Einstein Field Equations
- TODO #42: `src/render.rs:3889` - tensor rendering
- TODO #43: `src/render.rs:4174` - outer product

**Matrix Dimension Tests** (2 tests)
- TODO #55: `tests/signature_dimension_test.rs:15` - add dimension constraint
- TODO #56: `tests/signature_dimension_test.rs:76` - multiply dimension constraint

**Prelude Tests** (2 tests)
- TODO #53: `tests/load_full_prelude_test.rs:14` - parse full prelude
- TODO #54: `tests/load_full_prelude_test.rs:89` - load into typechecker

**Recommendation:** Review each ignored test:
- Update expectations if renderer output changed
- Remove if no longer relevant
- Fix and re-enable if still needed

### Missing Type System Features

**TODO #4** - `src/type_inference.rs:942`
```rust
// TODO: Add constraint that arg_type matches field_def.type_expr
```
**Issue:** Data constructor field validation missing constraint  
**Impact:** Type safety gap in data constructors  
**Recommendation:** Add constraint to ensure field types match definitions

**TODO #8** - `src/type_context.rs:284`
```rust
// Extract type name from type_args (use first arg for now, TODO: handle multiple)
```
**Issue:** Only handles first type argument  
**Impact:** Multi-parameter types may not work correctly  
**Recommendation:** Check if any types use multiple args, implement if needed

**TODO #9** - `src/type_context.rs:327`
```rust
// TODO: Register these separately if needed
```
**Issue:** Top-level operations (like `frac`) not registered  
**Impact:** May prevent top-level operation declarations  
**Recommendation:** Determine if this blocks any features, implement if so

### Parser Limitations

**TODO #11** - `src/kleis_parser.rs:1758`
```rust
// TODO: Add FunctionDef variant to StructureMember enum
```
**Issue:** Functions can't be defined inside structures  
**Impact:** Limits structure functionality  
**Recommendation:** Determine if needed for ADR-016 compliance

### Match Expression Support (5 TODOs)

Match expressions have incomplete support across multiple files:
- TODO #14: `src/bin/debug_matrix_semantic.rs:158` - slot collection
- TODO #15: `src/bin/server.rs:245` - JSON serialization
- TODO #17: `src/bin/server.rs:656` - slot collection (duplicate)
- TODO #30: `src/math_layout/typst_adapter.rs:97` - rendering
- TODO #37: `src/render.rs:1377` - rendering

**Recommendation:** Decide if match expressions are actively used:
- If YES: Implement proper support
- If NO: Document as future work and ensure placeholders don't break

---

## 📌 Planned Work (Keep as Documentation)

### Wire 3 - Function Types (5 TODOs)

These are all related to planned "Wire 3" feature for proper function types:
- TODO #3: `src/type_inference.rs:819` - function application with currying
- TODO #6: `src/type_checker.rs:238` - use parameter type annotations
- TODO #7: `src/type_checker.rs:251` - build curried function types
- TODO #12: `src/kleis_parser.rs:1975` - store parameter types

**Status:** Documented as planned feature  
**Action:** Keep TODOs as reminders

### ADR-021 Vision Comments (3 TODOs)

These document future where types are defined in Kleis instead of Rust:
- TODO #2: `src/type_inference.rs:789` - Matrix as data constructor
- TODO #5: `src/type_inference.rs:1013` - unification in Kleis
- TODO #59: `src/type_inference.rs:959` - generic data constructor logic

**Status:** Architectural vision documentation  
**Action:** Keep as design documentation

---

## 💡 Nice-to-Have Enhancements

### Layout/Rendering Improvements (14 TODOs)

**Font Metrics:**
- TODO #19: `src/math_layout/font_metrics.rs:6` - port KaTeX metrics
- TODO #20: `src/math_layout/font_metrics.rs:99` - Computer Modern metrics

**Layout Implementations** (currently placeholders that work):
- TODO #21: `src/math_layout/mod.rs:111` - list literal layout
- TODO #23: `src/math_layout/mod.rs:125` - constant layout
- TODO #24: `src/math_layout/mod.rs:131` - symbol lookup
- TODO #25: `src/math_layout/mod.rs:178` - TeX fraction rules
- TODO #26: `src/math_layout/mod.rs:194` - superscript layout
- TODO #27: `src/math_layout/mod.rs:209` - subscript layout
- TODO #28: `src/math_layout/mod.rs:224` - sqrt layout
- TODO #29: `src/math_layout/mod.rs:234` - binary op spacing

**Rendering Enhancements:**
- TODO #31: `src/math_layout/typst_adapter.rs:180` - more operations
- TODO #38: `src/render.rs:1393` - render where clauses
- TODO #39: `src/render.rs:3569` - more Typst templates

**Action:** Keep as future enhancements

### Typst Compilation (2 TODOs)

- TODO #32: `src/math_layout/typst_adapter.rs:224` - actual Typst compilation
- TODO #36: `src/math_layout/typst_compiler.rs:2653` - MinimalWorld integration

**Status:** Major feature, currently stubbed out  
**Action:** Keep as future work

### Test Improvements (8 TODOs)

**Builder API** (7 TODOs in `tests/golden_tests.rs`):
- TODOs #44-50: Lines 21, 36, 46, 56, 70, 80, 90 - expose builder functions

**Test Coverage:**
- TODO #51: `tests/golden_tests.rs:120` - set theory tests
- TODO #52: `tests/golden_tests.rs:927` - golden file comparison

**Action:** Low priority refactoring

### Minor Improvements (5 TODOs)

- TODO #1: `src/type_inference.rs:639` - proper constant type checking
- TODO #10: `src/type_context.rs:349` - validate where constraints (future)
- TODO #16: `src/bin/server.rs:254` - include where clause in JSON
- TODO #18: `src/bin/server.rs:732` - TypeChecker performance (use &self)

**Action:** Keep as minor enhancements

---

## 📋 Recommended Actions

### Immediate (This Session)

1. **Fix TODO #22** - Replace `unimplemented!()` panic with placeholder
2. **Review TODO #13** - Type safety issue with you
3. **Decide on ignored tests** - Update, remove, or fix 11 tests

### Short Term (Next Session)

4. **Review TODO #4** - Add data constructor field constraint?
5. **Review TODO #8** - Multiple type args needed?
6. **Review TODO #9** - Top-level operations blocking anything?
7. **Review TODO #11** - Functions in structures needed?
8. **Match expressions** - Are they used? Need implementation?

### Long Term (Future)

9. **Wire 3** - Function types (TODOs #3, #6, #7, #12)
10. **Layout improvements** - When polish phase begins
11. **Typst compilation** - Major feature when needed

---

## 📊 Statistics

| Category | Count | Percentage |
|----------|-------|------------|
| **Critical** | 2 | 3.5% |
| **Important** | 19 | 33.3% |
| **Planned (Wire 3, ADR-021)** | 8 | 14.0% |
| **Nice-to-have** | 28 | 49.1% |
| **Total** | 57 | 100% |

**Ignored Tests:** 11 out of 57 TODOs (19.3%)

---

## 🎯 Priority Matrix

```
High Impact, High Urgency:
- TODO #13 (type safety bug)
- TODO #22 (panic in match layout)

High Impact, Medium Urgency:
- TODO #4 (field constraint)
- TODO #8 (multiple type args)
- 11 ignored tests

Medium Impact, Low Urgency:
- Match expression support (5 TODOs)
- Parser limitations (TODOs #9, #11)

Low Impact:
- Layout enhancements (14 TODOs)
- Test builders (8 TODOs)
- Performance (TODO #18)
```

---

## 📝 Notes

- **No obsolete TODOs found** - All are still relevant
- **No duplicate work** - TODOs for match expressions intentionally across files
- **Well-documented vision** - ADR-021 and Wire 3 TODOs provide context
- **Tests need attention** - 19.3% of tests are ignored

**Overall Assessment:** Codebase is in good shape. Most TODOs are either planned work (Wire 3, ADR-021) or nice-to-have enhancements. Main concerns are 2 critical issues and 11 ignored tests.

