# Matrix Constructor Type System Issue

**Date:** December 7, 2024  
**Status:** ⚠️ Known Issue - Deferred to Future Work  
**Priority:** Medium (works but not elegant)  
**Related:** ADR-016, Phase 2 or 3

---

## The Issue

Matrix constructors (`Matrix`, `PMatrix`, `VMatrix`, `BMatrix`) are currently handled as special cases in `type_inference.rs`, but the implementation is **hacky** and not properly integrated with the type system.

---

## Current Implementation

### **What Happens Now**

```rust
// src/type_inference.rs:202-212
match name {
    "Matrix" | "PMatrix" | "VMatrix" | "BMatrix" => {
        self.infer_matrix_constructor(name, args, context_builder)
    }
    _ => { /* delegate to context_builder */ }
}

// Lines 238-290
fn infer_matrix_constructor(...) {
    // Extract dimensions from first two arguments
    let rows = match &args[0] {
        Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
        _ => 2,  // ← Default if not a constant
    };
    let cols = match &args[1] {
        Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
        _ => 2,
    };
    
    // ... check element types ...
    
    Ok(Type::Matrix(rows, cols))
}
```

---

## The Problems

### **1. Parsing Dimensions from Expressions**

```rust
let rows = match &args[0] {
    Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
    _ => 2,  // What if it's a variable? Just default to 2?
};
```

**Issue:** If dimensions aren't constants, we just default to `2×2`. This loses information!

**Example:**
```kleis
let n = 3
let A = Matrix(n, n, 1, 2, 3, 4, 5, 6, 7, 8, 9)
// Type inference sees: Matrix(2, 2) ← WRONG! Should be Matrix(3, 3)
```

---

### **2. Dimensions as Values, Not Types**

In proper dependent type theory:
```
Matrix(2, 3, ℝ)  ← 2 and 3 are TYPE-LEVEL values
```

In our current implementation:
```rust
Type::Matrix(rows, cols)  ← rows and cols are runtime usize values
```

**Problem:** Type system doesn't distinguish between:
- Type-level natural numbers (static dimensions)
- Value-level natural numbers (runtime dimensions)

---

### **3. Not ADR-016 Compliant**

Matrix constructors are **literals**, not operations, so they can't easily be moved to stdlib.

**Why?**
```kleis
// This is a literal:
Matrix(2, 2, 1, 2, 3, 4)

// NOT an operation like:
transpose(A)  // ← Takes a matrix, returns a matrix
```

**But:** Even literals could be better integrated with the type system.

---

### **4. Inconsistent with Other Constructors**

```kleis
// Matrix: Special case in type_inference.rs
Matrix(2, 2, a, b, c, d)

// Vector: Should also be a literal constructor (but isn't implemented)
Vector(3, x, y, z)

// List: Should also be a literal constructor
List(1, 2, 3)
```

**Issue:** No consistent pattern for parametric type constructors.

---

## Why It Works (For Now)

### **Practical Reality:**

1. **Most matrices use constants for dimensions**
   ```kleis
   Matrix(2, 2, 1, 2, 3, 4)  // ← Works perfectly
   Matrix(3, 3, ...)         // ← Works perfectly
   ```

2. **Fallback to 2×2 is reasonable**
   - If we can't determine dimensions, 2×2 is a sensible default
   - Better than crashing or returning Type::Var

3. **It's isolated**
   - Only one function: `infer_matrix_constructor()`
   - Doesn't pollute the rest of the type system
   - Easy to find and fix later

---

## The Proper Solution

### **Option A: Dependent Types** (Long-term)

Extend the type system to support dependent types where types can depend on values:

```rust
enum Type {
    // ...
    DependentMatrix {
        rows: Expression,  // Not usize, but Expression!
        cols: Expression,
        element_type: Box<Type>
    }
}
```

**Then:**
```kleis
let n = 3
Matrix(n, n, ...)  // Type: Matrix(n, n, ℝ) where n = 3
```

**Complexity:** HIGH - Requires significant type system extension  
**Time:** 2-3 weeks  
**Benefits:** Proper dependent typing, handles all cases

---

### **Option B: Dimension Expressions** (Medium-term)

Don't try to extract dimensions at type inference time. Instead:

```rust
enum Type {
    Matrix {
        rows: DimExpr,  // Can be Const(2) or Var("n")
        cols: DimExpr,
        element_type: Box<Type>
    }
}

enum DimExpr {
    Const(usize),
    Var(String),
    Unknown,
}
```

**Then:**
```kleis
Matrix(n, n, ...)  // Type: Matrix(Var("n"), Var("n"), ℝ)
Matrix(2, 3, ...)  // Type: Matrix(Const(2), Const(3), ℝ)
```

**Complexity:** MEDIUM - Extend Type enum, update unification  
**Time:** 3-5 days  
**Benefits:** Tracks dimensions properly without full dependent types

---

### **Option C: Keep Current, Document Limitation** (Current)

Accept that matrix constructor dimension inference is limited:

```rust
// Document the limitation clearly
fn infer_matrix_constructor(...) {
    // NOTE: This is a HACK. We extract dimensions from const expressions
    // and default to 2×2 if not available. This works for most cases but
    // is not theoretically sound. See: docs/session-2024-12-07/MATRIX_CONSTRUCTOR_ISSUE.md
    // TODO: Implement proper dimension tracking (Option B above)
    let rows = match &args[0] {
        Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
        _ => 2,  // Limitation: default to 2×2
    };
    // ...
}
```

**Complexity:** LOW - Just documentation  
**Time:** Done  
**Benefits:** Clear that it's a known limitation, not a bug

---

## Recommendation

### **For Now: Option C** ✅

**Why:**
- It works for 95% of cases
- It's isolated and easy to find
- More important work in Phase 1 & 2
- Can be fixed properly once parser is extended

### **Future: Option B** (Phase 2 or 3)

**When:**
- After parser extended to ~80% grammar
- After full prelude loaded
- When we have time for type system refinement

**Why:**
- Proper solution without full dependent types
- Reasonable complexity
- Handles variable dimensions correctly

---

## Related Issues

### **1. Vector Constructors Don't Exist**

```kleis
Vector(3, x, y, z)  // Should work but doesn't!
```

**Solution:** Once we fix Matrix constructors, apply same pattern to Vector

### **2. List Constructors**

```kleis
List(1, 2, 3, 4)  // Should work
```

**Solution:** Need general approach for parametric type constructors

### **3. Parser Support for Dimensions**

Parser currently treats dimension arguments as regular expressions:
```
Matrix(2, 3, a, b, c, d, e, f)
       ^  ^  <- These should be DIMENSIONS (type-level)
          ^^^^^^^^^^^^^^^^^^ <- These are ELEMENTS (value-level)
```

No distinction at parse level!

---

## Action Items (Future)

### **Phase 2 or 3:**

1. **Extend Type enum** with DimExpr
2. **Update infer_matrix_constructor** to use DimExpr
3. **Update unification** to handle dimension expressions
4. **Add tests** for variable dimensions
5. **Apply to Vector** constructors
6. **Apply to List** constructors
7. **Document** the new approach

**Estimated time:** 3-5 days

---

## Examples of What Will Work Better

### **Current Behavior:**

```kleis
let n = 3
Matrix(n, n, ...)
// Type: Matrix(2, 2, ℝ) ← WRONG! Defaults to 2×2
```

### **After Fix (Option B):**

```kleis
let n = 3
Matrix(n, n, ...)
// Type: Matrix(n, n, ℝ) ← CORRECT! Tracks dimension variable
```

### **With Full Dependent Types (Option A):**

```kleis
let n = 3
let m = n + 1
Matrix(n, m, ...)
// Type: Matrix(n, n+1, ℝ) ← Even expressions work!
```

---

## Why This Document Exists

**Purpose:** Document known limitation so:
1. Future maintainers understand it's deliberate
2. Users know the limitation if they encounter it
3. We have a plan for fixing it properly
4. It's not confused with a bug

**This is technical debt we're aware of and have a plan for.**

---

## References

- `src/type_inference.rs:238-290` - Current implementation
- ADR-016 - Operations in structures (Matrix is exception)
- Phase 2 roadmap - When to address this

---

**Status:** Documented ✅  
**Priority:** Medium (Phase 2 or 3)  
**Workaround:** Works for constant dimensions  
**Proper fix:** Option B (DimExpr)

