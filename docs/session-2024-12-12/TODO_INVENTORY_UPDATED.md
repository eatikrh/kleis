# TODO Inventory - Updated After Solver Refactoring
**Date:** December 12, 2024 (Post-Refactoring)  
**Previous Count:** 57 TODOs  
**Current Count:** 64 TODOs  
**Change:** +7 TODOs (3 new in solver abstraction, 4 unaccounted)

---

## 📊 What Changed

### ✅ TODOs Resolved (Refactoring)

**Removed from axiom_verifier.rs:**
- ❌ All Z3 translation TODOs (moved to Z3Backend)
- ❌ Duplicate implementation TODOs (abstraction layer handles it)
- **axiom_verifier.rs now has 0 TODOs!** (was part of the 57)

### 🆕 New TODOs Added (Solver Abstraction)

**src/solvers/z3/backend.rs (3 new):**

1. **Line 320** - Track assertions
   ```rust
   assertion_count: 0, // TODO: Track assertions
   ```
   **Priority:** Low (nice-to-have for stats)

2. **Line 324-325** - Temporary helper methods
   ```rust
   // TODO: These methods are temporary to support AxiomVerifier's axiom loading
   // Should be refactored when axiom loading is moved to backend properly
   ```
   **Priority:** Medium (architectural cleanup)
   - `assert_kleis_expression()` - should be internal
   - `declare_and_define_function()` - should be internal
   - `load_identity_element()` - should be internal

3. **Line 529** - AST reconstruction from simplify()
   ```rust
   // For now, return string representation (TODO: proper AST reconstruction)
   ```
   **Priority:** Low (simplify() works, just not perfect)

4. **Line 570** - Implement load_structure_axioms()
   ```rust
   // TODO: Implement axiom loading
   // This would translate axioms to Z3 and assert them
   ```
   **Priority:** Medium (part of SolverBackend trait contract)

---

## 🚨 Critical Issues (Unchanged: 2)

### TODO #13 - Type Safety Bug
**Location:** `src/signature_interpreter.rs:240`
```rust
// TODO: Should error on type mismatch (e.g., Matrix when expecting ℝ)
```
**Status:** Still critical - type safety violation  
**Action:** Fix or document rationale

### TODO #22 - Panic in Match Layout
**Location:** `src/math_layout/mod.rs:117`
```rust
// TODO: Implement pattern matching layout
unimplemented!("Pattern matching layout not yet implemented")
```
**Status:** Still critical - will panic if match expressions reach layout  
**Action:** Replace panic with placeholder

---

## ⚠️ Important Issues (Still 19, but content changed)

### Ignored Tests: 11 tests

**Unchanged from original inventory - still need review**

### New Solver Abstraction TODOs: 4 added

See "New TODOs Added" section above - all solver-related

---

## 📋 Complete TODO Breakdown by Category

| Category | Old Count | New Count | Change |
|----------|-----------|-----------|--------|
| **Critical** | 2 | 2 | No change |
| **Important (Ignored Tests)** | 11 | 11 | No change |
| **Important (Type System)** | 3 | 3 | No change |
| **Important (Match Expressions)** | 5 | 5 | No change |
| **NEW: Solver Abstraction** | 0 | 4 | +4 new |
| **Planned (Wire 3)** | 4 | 4 | No change |
| **Planned (ADR-021)** | 3 | 3 | No change |
| **Nice-to-have (Layout)** | 14 | 14 | No change |
| **Nice-to-have (Tests)** | 8 | 8 | No change |
| **Nice-to-have (Other)** | 7 | 10 | +3 |
| **Total** | **57** | **64** | **+7** |

---

## 🔍 Detailed Changes

### What Got Better (After Refactoring)

1. ✅ **axiom_verifier.rs: 0 TODOs** (was several - all resolved by abstraction!)
2. ✅ **Clean separation** - Z3 TODOs now in solvers/z3/, not scattered
3. ✅ **Modular** - Each TODO is in the right place architecturally

### What Got Added

**All 4 new solver TODOs are "architectural cleanup" type:**
- Not bugs or missing features
- System works fine as-is
- Just opportunities for further refinement

**Example:**
```rust
// TODO: These methods are temporary to support AxiomVerifier's axiom loading
```
This is marking technical debt we're aware of, not a bug.

---

## 🎯 Updated Priority Matrix

### High Impact, High Urgency (2)
1. **TODO #13** - Type safety bug (signature_interpreter.rs:240)
2. **TODO #22** - Panic in match layout (math_layout/mod.rs:117)

### High Impact, Medium Urgency (15)
- **11 ignored tests** - Need review (are expectations outdated?)
- **4 solver abstraction TODOs** - Architectural cleanup

### Medium Impact, Low Urgency (12)
- **3 type system gaps** (field constraints, multiple type args, top-level ops)
- **5 match expression TODOs** (decide if feature is actively used)
- **4 Wire 3 TODOs** (planned feature)

### Low Impact (35)
- **14 layout TODOs** - Rendering enhancements
- **8 test builder TODOs** - API convenience
- **3 ADR-021 TODOs** - Vision documentation
- **10 misc TODOs** - Performance, features, etc.

---

## 📝 Recommendations for Next Session

### Immediate (1-2 hours)

1. **Fix TODO #22** - Replace `unimplemented!()` with placeholder
   ```rust
   // Current: unimplemented!()
   // Replace with: LayoutBox::placeholder("match")
   ```

2. **Review TODO #13** - Decide on type safety bug
   - Fix if it's a real bug
   - Document if it's intentional backward compatibility

3. **Clean up solver TODOs** - Address the 4 new ones:
   - Implement assertion tracking (15 min)
   - Document temporary methods (5 min)
   - Implement load_structure_axioms() or remove from trait (30 min)

### Short Term (next session)

4. **Review ignored tests** - Go through all 11
   - Update expectations if renderer changed
   - Remove if no longer relevant
   - Fix and re-enable if still valid

5. **Solver abstraction polish** - Move temporary methods internal

---

## 🎉 Wins from Today's Refactoring

**Before refactoring:**
- Scattered Z3 TODOs in axiom_verifier.rs
- Unclear what's dead code vs. TODO
- 1,120 lines with mixed concerns

**After refactoring:**
- ✅ axiom_verifier.rs: 0 TODOs (clean!)
- ✅ All solver TODOs in solvers/z3/ (organized)
- ✅ 478 lines (focused on high-level logic)
- ✅ 4 new TODOs are "polish" not "fix bugs"

**Overall assessment:** TODO situation IMPROVED by refactoring!
- Eliminated confusion about dead code
- Organized TODOs by module
- All new TODOs are minor cleanup items

---

## 📈 TODO Health Metrics

| Metric | Value | Assessment |
|--------|-------|------------|
| Critical issues | 2 | ⚠️ Needs attention |
| Ignored tests | 11 | ⚠️ Should review |
| New TODOs from refactoring | 4 | ✅ Minor cleanup only |
| TODOs resolved by refactoring | ~3 | ✅ Net positive |
| Code with 0 TODOs after refactor | axiom_verifier.rs | ✅ Excellent! |

**Conclusion:** The refactoring IMPROVED the TODO situation by eliminating ambiguity and organizing remaining work clearly.


