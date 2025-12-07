# Golden Test Failures - Pre-Existing

**Date:** December 7, 2024  
**Status:** ℹ️ Pre-existing failures (not caused by today's work)  
**Tests:** 2 golden tests failing

---

## Summary

Two golden tests in `tests/golden_tests.rs` are failing:
1. `golden_batch3_completeness::inverse_trig_complete`
2. `golden_batch3_completeness::complete_function_library`

**Important:** These tests were **already failing before our type system changes**.

---

## Verification

### **Test on Previous Commit (HEAD~7)**

```bash
$ git checkout HEAD~7  # Before our session
$ cargo test golden_batch3_completeness
test result: FAILED. 6 passed; 2 failed
```

**Same 2 tests failing!**

### **Test on Current Commit**

```bash
$ git checkout main
$ cargo test golden_batch3_completeness  
test result: FAILED. 6 passed; 2 failed
```

**Same 2 tests failing.**

**Conclusion:** These failures are **pre-existing**, not caused by our type system work.

---

## What These Tests Do

### **Test 1: inverse_trig_complete**

```rust
fn inverse_trig_complete() {
    let samples = kleis::render::collect_samples_for_gallery();
    
    let inv_trig: Vec<&str> = samples
        .iter()
        .filter(|(t, _)| t.contains("Inverse trig"))
        .map(|(_, latex)| latex.as_str())
        .collect();
    
    assert!(!inv_trig.is_empty());  // ← FAILS: No "Inverse trig" samples
    assert!(inv_trig.iter().any(|s| s.contains(r"\arcsin")));
    assert!(inv_trig.iter().any(|s| s.contains(r"\arccos")));
}
```

**Purpose:** Verify that rendering gallery includes inverse trig function samples

**Why it fails:** The gallery (`collect_samples_for_gallery()`) doesn't have any samples tagged "Inverse trig"

**Related to type system?** ❌ No - this is about rendering samples

---

### **Test 2: complete_function_library**

```rust
fn complete_function_library() {
    let samples = kleis::render::collect_samples_for_gallery();
    
    let functions: Vec<&str> = samples
        .iter()
        .filter(|(title, _)| {
            title.contains("sin") || 
            title.contains("cos") ||
            title.contains("log") ||
            title.contains("exp")
        })
        .map(|(t, _)| t)
        .collect();
    
    assert!(
        functions.len() >= 4,  // ← FAILS: Less than 4 function samples
        "Should have comprehensive function support"
    );
}
```

**Purpose:** Verify gallery has at least 4 different function samples

**Why it fails:** The gallery doesn't have enough tagged function samples

**Related to type system?** ❌ No - this is about rendering samples

---

## Impact on Our Work

### **Our Changes:**
- Type inference engine
- Type context builder
- Standard library loading
- Operation registry

### **These Tests:**
- Rendering gallery samples
- LaTeX generation
- Sample collection

**Overlap:** ❌ **NONE**

---

## Why They're Failing

Looking at `src/render.rs`, these are rendering **samples** for the gallery, not type checking functionality.

The tests expect certain samples to be registered with specific titles:
- "Inverse trig" samples (arcsin, arccos)
- At least 4 function samples (sin, cos, log, exp)

**Root cause:** Gallery samples incomplete or not tagged properly.

---

## Recommendation

### **Option 1: Fix the Gallery Samples** (Proper fix)

Add the missing samples to `src/render.rs`:

```rust
// Add to collect_samples_for_gallery()
samples.push(("Inverse trig - arcsin", r"\arcsin(x)".to_string()));
samples.push(("Inverse trig - arccos", r"\arccos(x)".to_string()));
samples.push(("Function - sin", r"\sin(x)".to_string()));
samples.push(("Function - cos", r"\cos(x)".to_string()));
samples.push(("Function - log", r"\log(x)".to_string()));
samples.push(("Function - exp", r"\exp(x)".to_string()));
```

**Time:** 15 minutes  
**Impact:** Fixes the tests properly

---

### **Option 2: Mark Tests as Ignored** (Temporary)

Add `#[ignore]` to these tests until gallery is complete:

```rust
#[test]
#[ignore = "Gallery samples incomplete"]
fn inverse_trig_complete() { ... }

#[test]
#[ignore = "Gallery samples incomplete"]
fn complete_function_library() { ... }
```

**Time:** 5 minutes  
**Impact:** Documents known issue, doesn't block our work

---

### **Option 3: Verify Pre-Existing** (Document only)

Document that these were already failing and are unrelated to type system work.

**Time:** 0 minutes (already done in this document)  
**Impact:** Clear that we didn't break them

---

## My Recommendation

**Use Option 1** - Fix the gallery samples properly.

**Why:**
- Quick fix (15 minutes)
- Improves the gallery
- Makes tests pass
- Professional to fix pre-existing issues when we notice them

---

## Action Plan

1. Add missing samples to `collect_samples_for_gallery()`
2. Run tests to verify they pass
3. Commit as separate fix
4. Continue with clean test suite

**Shall I implement this fix?**

---

**Note:** These failures are **NOT caused by our type system work**. They're pre-existing gallery issues. But we should fix them anyway for completeness.

