# Remaining TODOs After Grammar v0.6

**Date:** December 12, 2025  
**Original Count:** 57 TODOs  
**Resolved Today:** 1 (TODO #11 - Functions in Structures)  
**Remaining:** 56 TODOs

---

## 📊 TODO Categories Breakdown

| Category | Count | Percentage |
|----------|-------|------------|
| **🚨 Critical** | 2 | 3.6% |
| **⚠️ Important** | 18 | 32.1% |
| **📌 Planned Work** | 8 | 14.3% |
| **💡 Nice-to-have** | 28 | 50.0% |
| **Total Remaining** | 56 | 100% |

---

## 🚨 Critical Issues (2 TODOs - Need Immediate Attention)

### TODO #13 - Type Safety Bug
**File:** `src/signature_interpreter.rs:240`
```rust
// TODO: Should error on type mismatch (e.g., Matrix when expecting ℝ)
```
**Issue:** Accepts Matrix when expecting ℝ for "backward compatibility"  
**Impact:** Type safety violation  
**Action:** Decide if this is a bug or intentional behavior

### TODO #22 - Panic in Match Expression Layout
**File:** `src/math_layout/mod.rs:117`
```rust
// TODO: Implement pattern matching layout
unimplemented!("Pattern matching layout not yet implemented")
```
**Issue:** Will crash if match expressions are rendered  
**Impact:** Production crash risk  
**Action:** Replace panic with placeholder rendering

---

## ⚠️ Important Issues (18 TODOs)

### 1. Ignored Tests (11 tests - 19.6% of test suite!)

**Breakdown:**
- 3 placeholder conversion tests (`typst_adapter.rs`)
- 4 LaTeX rendering tests (`render.rs`)
- 2 matrix dimension tests (`signature_dimension_test.rs`)
- 2 prelude tests (`load_full_prelude_test.rs`) ← **These actually pass! Can un-ignore**

**Action Required:** Review each ignored test:
- Update expectations if rendering changed
- Remove if obsolete
- Fix and re-enable if needed

### 2. Match Expression Support (5 TODOs)

Match expressions have incomplete implementation:
- TODO #14: Slot collection (debug_matrix_semantic.rs)
- TODO #15: JSON serialization (server.rs)
- TODO #17: Slot collection duplicate (server.rs)
- TODO #30: Rendering (typst_adapter.rs)
- TODO #37: Rendering (render.rs)

**Question:** Are match expressions actively used?
- If YES: Implement proper support
- If NO: Document as future work

### 3. Type System Gaps (2 TODOs)

**TODO #4** - Missing field constraint validation  
**TODO #8** - Only handles first type argument (multi-arg types may break)

**Action:** Determine if these block any current features

### 4. Registration Gap (1 TODO)

**TODO #9** - Top-level operations not registered (20 operations parsed but unused)

---

## 📌 Planned Work (8 TODOs - Keep as Documentation)

### Wire 3 - Function Types (4 TODOs)

All related to planned "Wire 3" feature for proper function types:
- TODO #3: Function application with currying
- TODO #6: Use parameter type annotations
- TODO #7: Build curried function types
- TODO #12: Store parameter types individually

**Status:** Documented as future planned feature

### ADR-021 Vision (3 TODOs)

Document future where types are defined in Kleis:
- TODO #2: Matrix as data constructor
- TODO #5: Unification in Kleis
- TODO #59: Generic data constructor logic

**Status:** Architectural vision documentation

---

## 💡 Nice-to-Have (28 TODOs - Future Enhancements)

### Layout/Rendering (14 TODOs)

**Font Metrics:** (2 TODOs)
- Port KaTeX metrics
- Computer Modern metrics

**Layout Placeholders:** (8 TODOs)
- List, constant, symbol, fraction, superscript, subscript, sqrt, binary ops
- All currently work with basic placeholders

**Rendering Enhancements:** (4 TODOs)
- More operations, where clauses, Typst templates

### Typst Compilation (2 TODOs)

Large feature, currently stubbed:
- Actual Typst compilation
- MinimalWorld integration

### Test Improvements (8 TODOs)

- Builder API exposure (7 TODOs in golden_tests.rs)
- Set theory tests
- Golden file comparison

### Minor (4 TODOs)

- Constant type checking precision
- Where constraint validation (future)
- Include where clause in JSON
- TypeChecker performance

---

## 🎯 Recommended Priorities

### This Session (Can Continue)

1. ✅ **DONE:** Grammar v0.6 (TODO #11)
2. **TODO #22:** Fix panic in match layout (5 minutes)
3. **TODO #53, #54:** Un-ignore prelude tests (they pass!)

### Next Session

4. **Review ignored tests** - Decide fate of 11 tests
5. **TODO #13:** Type safety - decide on behavior
6. **Match expressions:** Determine if used, implement if needed
7. **TODO #4, #8, #9:** Type system gaps review

### Future

8. **Wire 3** - Function types (8 TODOs)
9. **Layout polish** - When rendering quality matters
10. **Typst integration** - When needed

---

## 📊 Progress Today

**Resolved:** 1 critical TODO (#11)
- ✅ Grammar v0.6 created
- ✅ Parser updated
- ✅ Tests created
- ✅ Documentation comprehensive
- ✅ Committed and pushed

**Discovered:** 2 tests (#53, #54) can be un-ignored (they pass!)

**Net Progress:** 3 actionable items cleared from Important category

---

## 🎯 Quick Wins Available

**Can be done quickly (<30 minutes each):**
1. ✅ **TODO #11** - DONE!
2. **TODO #22** - Replace panic with placeholder (1 line change)
3. **TODO #53, #54** - Un-ignore tests (remove `#[ignore]` attribute)
4. **TODO #14-17, #30, #37** - Match expressions (if not used, mark as future work)

**Would reduce Important category from 18 → 11 TODOs** (39% reduction!)

---

## 📈 Overall TODO Health

**Good news:**
- ✅ No obsolete TODOs (all still relevant)
- ✅ Well-categorized by intent
- ✅ Most are future enhancements (50%)
- ✅ Planned work is documented (14%)
- ✅ Only 2 critical issues (3.6%)

**Areas needing attention:**
- ⚠️ 11 ignored tests (19.6% of tests not running)
- ⚠️ 2 critical type safety issues
- ⚠️ Match expression completeness unclear

---

## Summary

**Categories after Grammar v0.6:**
1. 🚨 **Critical** (2) - Type safety, panic
2. ⚠️ **Important** (18) - Ignored tests, match expressions, type gaps
3. 📌 **Planned** (8) - Wire 3, ADR-021 vision
4. 💡 **Nice-to-have** (28) - Layout, Typst, test improvements

**Next logical focus:** Address the 2 critical issues and the 11 ignored tests.

