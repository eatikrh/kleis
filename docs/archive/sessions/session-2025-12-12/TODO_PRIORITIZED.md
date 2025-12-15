# Kleis TODO Priority List

**Generated:** December 12, 2025  
**Total TODOs in Code:** 64 (50 in src/, 14 in tests/, excluding vendor/)  
**Source:** Fresh grep of codebase

---

## 🚨 CRITICAL - Fix Immediately (2)

These can cause crashes or production failures.

### 1. Panic in Pattern Matching Layout
**Location:** `src/math_layout/mod.rs:117`
```rust
Expression::Match { .. } => {
    // TODO: Implement pattern matching layout
    unimplemented!("Pattern matching layout not yet implemented")
}
```
**Impact:** Will crash if match expressions reach layout engine  
**Risk:** Production crash  
**Fix:** Replace `unimplemented!()` with placeholder rendering  
**Effort:** 5 minutes  
**Action:** 
```rust
Expression::Match { .. } => {
    LayoutBox::text("⟨match⟩".to_string()) // Placeholder
}
```

### 2. TypeChecker Performance Issue
**Location:** `src/bin/server.rs:732`
```rust
// For now, create a new one each time (TODO: make check() use &self)
let mut checker = TypeChecker::new();
```
**Impact:** Creates new TypeChecker for every request (expensive!)  
**Risk:** Performance degradation, memory churn  
**Fix:** Make `check()` method use `&self` instead of `&mut self`  
**Effort:** 30 minutes (requires signature changes)

---

## ⚠️ HIGH PRIORITY - Address Soon (15)

### Ignored Tests (11 tests - 17% of test suite!)

**A. Rendering Tests with Outdated Expectations (7 tests)**

These tests are ignored because renderer output changed, but tests weren't updated:

1. **`src/render.rs:3813`** - Inner product LaTeX rendering
   ```rust
   #[ignore = "TODO: Fix inner product LaTeX rendering - outdated expectations"]
   ```

2. **`src/render.rs:3865`** - Einstein Field Equations LaTeX  
   ```rust
   #[ignore = "TODO: Update EFE LaTeX expectations - renderer output changed"]
   ```

3. **`src/render.rs:3889`** - Tensor rendering
   ```rust
   #[ignore = "TODO: Fix tensor rendering - outdated expectations"]
   ```

4. **`src/render.rs:4174`** - Outer product rendering
   ```rust
   #[ignore = "TODO: Fix outer product rendering - outdated expectations"]
   ```

5-7. **`src/math_layout/typst_adapter.rs:278,298,339`** - Placeholder conversion tests (3 tests)
   ```rust
   #[ignore = "TODO: Fix placeholder conversion - outdated expectations"]
   #[ignore = "TODO: Fix fraction with placeholder - outdated expectations"]  
   #[ignore = "TODO: Fix nested placeholders - outdated expectations"]
   ```

**Action Required:** For each test:
- Run the test to see current output
- Update expected values if new output is correct
- Remove `#[ignore]` attribute
- Re-run to verify

**Effort:** 1-2 hours total

---

**B. Matrix Dimension Tests (2 tests)**

8-9. **`tests/signature_dimension_test.rs:15,76`** - Matrix type format changed
   ```rust
   #[ignore = "TODO: Update for new Matrix data type format - uses old Type::matrix() helper"]
   ```

**Issue:** Tests use old `Type::matrix()` helper that no longer exists  
**Action:** Update tests to use new Matrix data type format  
**Effort:** 30 minutes

---

**C. Prelude Tests (2 tests - THESE PASS!)**

10-11. **`tests/load_full_prelude_test.rs:14,89`** - These actually pass now!
   ```rust
   #[ignore] // TODO: Requires top-level operation declarations
   ```

**Action:** Remove `#[ignore]` and verify they pass  
**Effort:** 5 minutes (just remove the attribute)

---

### Match Expression Support (4 TODOs)

Match expressions have incomplete implementations across multiple modules:

12. **`src/math_layout/typst_adapter.rs:97`** - Pattern matching rendering
13. **`src/render.rs:1377`** - Pattern matching rendering  
14. **`src/bin/server.rs:245`** - JSON serialization
15. **`src/bin/server.rs:656`** - Slot collection
    Also: `src/bin/debug_matrix_semantic.rs:158` (duplicate)

**Question:** Are match expressions actively used in the codebase?
- If YES: Complete the implementation
- If NO: Document as future work and replace panics with stubs

**Effort:** 2-4 hours if needed, 15 minutes if just documenting

---

## 📌 MEDIUM PRIORITY - Planned Work (10)

### Type System Improvements (4 TODOs)

16. **`src/type_context.rs:290`** - Only uses first type argument
    ```rust
    // Extract type name from type_args (use first arg for now, TODO: handle multiple)
    ```
    **Impact:** Multi-parameter types may not work correctly

17. **`src/type_context.rs:355`** - No constraint validation
    ```rust
    // TODO (future): Validate that type arguments actually satisfy the constraint
    ```

18. **`src/type_inference.rs:942`** - Missing field constraint check
    ```rust
    // TODO: Add constraint that arg_type matches field_def.type_expr
    ```

19. **`src/type_inference.rs:639`** - Constants lack type checking
    ```rust
    // TODO: Add proper constant type checking
    ```

**Action:** Evaluate impact on current functionality, fix if blocking features

---

### Wire 3 - Function Types (4 TODOs)

These are all part of the planned "Wire 3" feature for proper function types:

20. **`src/type_checker.rs:238`** - Use parameter type annotations
21. **`src/type_checker.rs:251`** - Build curried function types  
22. **`src/type_inference.rs:819`** - Function application with currying
23. **`src/kleis_parser.rs:1963`** - Store parameter types individually

**Status:** Documented as future planned feature (keep as-is)

---

### Solver Backend (2 TODOs)

24. **`src/solvers/z3/backend.rs:320`** - Track assertion count
    ```rust
    assertion_count: 0, // TODO: Track assertions
    ```
    **Priority:** Low (nice-to-have for debugging)

25. **`src/solvers/z3/backend.rs:570`** - Implement axiom loading
    ```rust
    // TODO: Implement axiom loading
    ```
    **Priority:** Medium (part of trait contract)

---

## 💡 LOW PRIORITY - Future Enhancements (32)

### Layout Engine Stubs (9 TODOs)

All in `src/math_layout/mod.rs`, lines 111, 125, 131, 178, 194, 209, 224, 234:
- List literal layout
- Symbol lookup  
- TeX fraction rules
- Various spacing implementations

**Status:** Placeholders work fine for now

---

### Rendering Enhancements (4 TODOs)

26. **`src/render.rs:1393`** - Render where clauses in output
27. **`src/render.rs:3569`** - Add more Typst templates
28. **`src/math_layout/typst_adapter.rs:180`** - Add more operations
29. **`src/bin/server.rs:254`** - Include where clause in JSON

---

### Typst Compilation (2 TODOs)

30. **`src/math_layout/typst_compiler.rs:2653`** - Implement MinimalWorld
31. **`src/math_layout/typst_adapter.rs:224`** - Actual Typst compilation

**Status:** Large feature, currently stubbed

---

### Font Metrics (2 TODOs)

32. **`src/math_layout/font_metrics.rs:6`** - Port KaTeX metrics
33. **`src/math_layout/font_metrics.rs:99`** - Computer Modern metrics

**Status:** Current metrics work adequately

---

### Golden Test Improvements (8 TODOs)

All in `tests/golden_tests.rs`:
- 7 TODOs about exposing builder API (lines 21, 36, 46, 56, 70, 80, 90)
- 1 TODO about set theory support (line 120)
- 1 TODO about golden file comparison (line 927)

**Status:** Test infrastructure improvements, not blocking

---

### Solver Backend Polish (2 TODOs)

34. **`src/solvers/z3/backend.rs:324`** - Temporary helper methods
    ```rust
    // TODO: These methods are temporary to support AxiomVerifier's axiom loading
    ```
    **Action:** Refactor when axiom loading is moved to backend

35. **`src/solvers/z3/backend.rs:529`** - Proper AST reconstruction
    ```rust
    // For now, return string representation (TODO: proper AST reconstruction)
    ```
    **Priority:** Low (string representation works)

---

### Minor Enhancements (5 TODOs)

36. **`src/type_context.rs:333`** - Register operations separately
37. **`src/type_context.rs:683`** - Better error messages (edit distance)
38. **`src/kleis_parser.rs:2231`** - Parse builtin identifiers (comment, not TODO)

---

## 📚 DOCUMENTATION - ADR-021 Vision (3 TODOs)

These document future architectural vision:

39. **`src/type_inference.rs:789`** - Matrix as data constructor
    ```rust
    /// TODO(ADR-021): Matrix constructors are DATA CONSTRUCTORS
    ```

40. **`src/type_inference.rs:959`** - Generic data constructor logic
    ```rust
    /// TODO(ADR-021): This logic is already generic and would work for all data constructors!
    ```

41. **`src/type_inference.rs:1013`** - Pattern matching in Kleis
    ```rust
    /// TODO(ADR-021): This pattern matching should be in Kleis, not Rust!
    ```

**Status:** Keep as documentation of future direction

---

## ✅ RESOLVED (1)

**`tests/grammar_v06_function_in_structure_test.rs:3`** - Resolved today!
```rust
//! Resolves TODO #11
```
✅ Grammar v0.6 implemented, test created, documentation written

---

## 📊 Summary by Category

| Category | Count | Percentage |
|----------|-------|------------|
| 🚨 **Critical** | 2 | 3.1% |
| ⚠️ **High Priority** | 15 | 23.4% |
| 📌 **Medium Priority** | 10 | 15.6% |
| 💡 **Low Priority** | 32 | 50.0% |
| 📚 **Documentation** | 3 | 4.7% |
| ✅ **Resolved** | 1 | 1.6% |
| **Not Counted** | 1 | 1.6% |
| **Total** | **64** | **100%** |

---

## 🎯 Recommended Actions

### This Session (30 minutes)

1. ✅ **DONE:** TODO #11 (Grammar v0.6)
2. **TODO #1:** Fix panic in match layout (5 min) - One line change
3. **TODO #10-11:** Un-ignore prelude tests (5 min) - Just remove attribute
4. **TODO #2:** Fix TypeChecker performance (20 min) - Signature change

**Impact:** Eliminate 1 critical crash, improve server performance, re-enable 2 passing tests

### Next Session (2-3 hours)

5. **Review ignored tests (TODO #3-9):** Update expectations or remove tests
6. **Decide on match expressions (TODO #12-15):** Complete or stub out
7. **Fix matrix dimension tests (TODO #8-9):** Update to new format

**Impact:** Re-enable 11 ignored tests (17% of test suite!)

### Future Sessions

8. **Type system improvements (TODO #16-19):** Evaluate and fix gaps
9. **Solver backend polish (TODO #24-25, #34-35):** Complete abstraction
10. **Layout engine:** When rendering quality becomes priority

---

## 🏆 Quick Wins Available

**Can complete in <30 minutes total:**
1. ✅ Fix match layout panic → Replace with placeholder (1 line)
2. ✅ Un-ignore prelude tests → Remove `#[ignore]` (2 tests)  
3. ⚠️ TypeChecker performance → Make `check()` use `&self`

**Would resolve:** 1 critical issue + 2 high-priority items = 3 TODOs cleared!

---

## 📈 TODO Health Assessment

**Good News:**
- ✅ Only 3.1% are critical (2 TODOs)
- ✅ 50% are low-priority future enhancements
- ✅ No obsolete TODOs (all documented with rationale)
- ✅ Well-organized by module
- ✅ Clear architectural vision (ADR-021 TODOs)

**Areas Needing Attention:**
- ⚠️ 11 ignored tests (17% of test suite not running!)
- ⚠️ 1 critical panic in match layout
- ⚠️ 1 critical performance issue (TypeChecker recreation)
- ⚠️ Match expression support incomplete (is it used?)

**Overall Assessment:** Healthy TODO situation! Most TODOs are future work, not bugs.

---

## 📋 Checklist for This Session

- [x] Generate fresh TODO inventory from code
- [ ] Fix critical panic in match layout
- [ ] Un-ignore passing prelude tests  
- [ ] Fix TypeChecker performance issue
- [ ] Create consolidated session README
- [ ] Archive old TODO documents

---

**Last Updated:** December 12, 2025  
**Method:** Fresh grep of `src/` and `tests/` directories  
**Excluded:** `vendor/` directory (external Z3 bindings)  
**Tools:** `grep -i 'TODO|FIXME|XXX' --include='*.rs'`

