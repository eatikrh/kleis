# Task 1.3: TypeContextBuilder Analysis

**Date:** December 8, 2025  
**Goal:** Analyze current state and plan improvements  

---

## Current State

### **File:** `src/type_context.rs`
**Lines:** 846 total

### **Function:** `infer_operation_type()` (lines 381-641)
**Size:** 260 lines  
**Issue:** Many hardcoded match cases

---

## Operations by Category

### **Matrix Operations (Hardcoded)**

| Operation | Lines | Uses SignatureInterpreter? | Notes |
|-----------|-------|----------------------------|-------|
| transpose | 423-431 | ✅ YES | Good example! |
| add | 433-464 | 🟡 Partial | Dimension check + interpreter |
| multiply | 466-489 | ❌ NO | Fully hardcoded |
| det/determinant | 491-514 | 🟡 Partial | Squareness check + interpreter |
| trace | 516-525 | ❌ NO | Fully hardcoded |

**Issue:** Should all use SignatureInterpreter consistently!

---

### **Arithmetic Operations (Added Yesterday)**

| Operation | Lines | Status |
|-----------|-------|--------|
| plus, minus, times, divide, etc. | 527-550 | Hardcoded but necessary |

**Note:** These handle type variables, can't easily use interpreter yet

---

### **Numeric Operations**

| Operation | Lines | Status |
|-----------|-------|--------|
| abs, floor, sqrt | 552-562 | Simple passthrough |
| power, sup, sub | 564-578 | Type variable handling |

---

### **Calculus Operations**

| Operation | Lines | Status |
|-----------|-------|--------|
| derivative, integral, d_dx, partial | 580-584 | Stub (just return Scalar) |
| int_bounds | 586-595 | Basic implementation |

---

### **Relational Operations**

| Operation | Lines | Status |
|-----------|-------|--------|
| equals, not_equals | 597-607 | Return RHS type |
| less_than, greater_than, etc. | 609-628 | Matrix rejection + scalar handling |

---

## Problems Identified

### **1. Inconsistent Use of SignatureInterpreter**

Only `transpose` and partially `add`/`det` use it. Should be used for ALL matrix operations!

### **2. Duplicate Dimension Checking Logic**

```rust
// This pattern repeats:
match (&arg_types[0], &arg_types[1]) {
    (Type::Matrix(m, n), Type::Matrix(p, q)) => {
        if n != p {
            return Err("dimensions don't match");
        }
        // ... compute result
    }
}
```

Should be abstracted!

### **3. No Fallback to Signature**

Operations not in the big match return error:
```rust
_ => Err(format!("Operation '{}' found but type inference not implemented", op_name))
```

Should try SignatureInterpreter as fallback!

---

## Improvement Plan

### **Step 1: Use SignatureInterpreter for Matrix Ops**

**Target operations:**
- multiply (currently 24 lines → should be ~10)
- trace (currently 10 lines → should be ~5)

### **Step 2: Add Fallback Logic**

```rust
// After all special cases, try SignatureInterpreter
_ => {
    // Try to use signature interpreter as fallback
    if let Some(structure_name) = structure_for_operation(op_name) {
        let structure = get_structure(structure_name)?;
        let mut interpreter = SignatureInterpreter::new();
        interpreter.interpret_signature(structure, op_name, arg_types)
    } else {
        Err(format!("Unknown operation: {}", op_name))
    }
}
```

### **Step 3: Extract Dimension Checking**

```rust
fn check_matrix_dimensions_match(
    op_name: &str,
    m1: usize, n1: usize,
    m2: usize, n2: usize
) -> Result<(), String> {
    if m1 != m2 || n1 != n2 {
        return Err(format!(
            "{}: dimensions must match! {}×{} ≠ {}×{}",
            op_name, m1, n1, m2, n2
        ));
    }
    Ok(())
}
```

### **Step 4: Better Error Messages**

Include:
- What operation was attempted
- What types were provided
- What types are supported
- Suggestions

---

## Expected Improvements

### **Before:**
- 260 lines in `infer_operation_type()`
- 5 hardcoded matrix operations
- Inconsistent use of SignatureInterpreter
- Repetitive code

### **After:**
- ~150 lines (100 fewer!)
- Consistent SignatureInterpreter usage
- Helper functions for common checks
- Fallback logic for extensibility

---

## Next Steps

1. ✅ Analysis complete
2. Refactor `multiply` to use SignatureInterpreter
3. Refactor `trace` to use SignatureInterpreter
4. Add fallback logic
5. Test everything
6. Commit

**Let's start!** 🚀

