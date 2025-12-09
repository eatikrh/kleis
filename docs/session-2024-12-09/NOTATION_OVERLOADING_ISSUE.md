# Notation Overloading Issue - Multiply Operations

**Date:** December 9, 2024  
**Status:** Needs implementation  
**Priority:** High (UX issue)

---

## Problem

**User Perspective:** 
```
Matrix(2,3) × Matrix(3,3) × Matrix(3,2) = Matrix(2,2)  ✓ Correct math!
```

**Current Behavior (varies!):**

**Sometimes:**
```
✗ Type Error: 'T' mismatch
  scalar_multiply requires both args to be same type
  Got: Matrix(3,3) and Matrix(3,2)
```

**Other times:**
```
✗ Unknown operation: 'cross'
  Hint: cross is not defined in any loaded structure
```

**Root Cause:** The `×` symbol maps inconsistently:
- Sometimes → `"scalar_multiply"` (wrong for matrices of different dimensions)
- Sometimes → `"cross"` (wrong - that's for 3D vector cross products!)
- Should → `"multiply"` (matrix multiplication)

---

## The UX Problem

**Notation overloading is fundamental to mathematics:**

| Symbol | Context | Operation | Structure |
|--------|---------|-----------|-----------|
| `×` | Scalars | Scalar multiply | `Arithmetic(T)` |
| `×` | Matrices | Matrix multiply | `MatrixMultipliable(m,n,p,T)` |
| `×` | Vectors (3D) | Cross product | `VectorSpace(V)` |
| `×` | Sets | Cartesian product | `Set(T)` |

**Users think visually** - they see `×` and expect the "right" multiplication based on what they're multiplying.

---

## Current Architecture

**Parser/Frontend:** Maps LaTeX/visual notation to operation names
- `\times` or `*` → `"scalar_multiply"` (sometimes!)
- `\times` or `×` → `"cross"` (sometimes!)
- **Inconsistent mapping depending on context!**

**Type System:** Checks if operation is valid for argument types
- `scalar_multiply` requires `Arithmetic(T)` with `T → T → T`
- For matrices: requires **same** dimensions

**Problem:** Parser picks operation name **before** knowing types!

---

## Solution Options

### Option 1: Type-Driven Operation Resolution (Recommended)

**Add resolution layer between parsing and type checking:**

```rust
// In type_checker.rs or new operation_resolver.rs:
fn resolve_operation_name(
    operation_name: &str,
    arg_types: &[Type]
) -> String {
    match operation_name {
        "scalar_multiply" | "times" => {
            // Check argument types
            if all_matrices(arg_types) {
                "multiply".to_string()  // Matrix multiplication
            } else if all_vectors_3d(arg_types) {
                "cross".to_string()  // Cross product
            } else {
                "scalar_multiply".to_string()  // Scalar multiplication
            }
        }
        _ => operation_name.to_string()
    }
}
```

**Call site:**
```rust
// Before type inference:
let resolved_name = resolve_operation_name(&op_name, &arg_types);
// Then use resolved_name for type checking
```

**Pros:**
- Natural for users (just use `×`)
- Type-driven (correct for mathematics)
- Backend handles it (frontend stays simple)

**Cons:**
- Need to infer arg types before resolving operation
- Slight complexity in resolution logic

---

### Option 2: Unified Polymorphic Operation

**Define one multiply that works for everything:**

```kleis
structure Multiplicative(T, U, R) {
    operation multiply : T → U → R
}

implements Multiplicative(ℝ, ℝ, ℝ) {
    operation multiply = scalar_multiply
}

implements Multiplicative(Matrix(m, n, T), Matrix(n, p, T), Matrix(m, p, T)) {
    operation multiply = matrix_multiply
}

implements Multiplicative(Vector(3, T), Vector(3, T), Vector(3, T)) {
    operation multiply = cross_product
}
```

**Pros:**
- Clean type theory
- One operation name for all contexts
- Extensible to user-defined types

**Cons:**
- Changes existing structure definitions
- More complex type signatures
- Need 3 type parameters (input1, input2, output)

---

### Option 3: Frontend Type-Aware Templates

**Frontend inspects types before picking operation:**

```javascript
function getMultiplyOperation(leftType, rightType) {
    if (isMatrix(leftType) && isMatrix(rightType)) {
        return 'multiply';  // Matrix multiplication
    } else if (isVector3D(leftType) && isVector3D(rightType)) {
        return 'cross';  // Cross product
    } else {
        return 'scalar_multiply';  // Scalar multiplication
    }
}
```

**Pros:**
- Backend stays simple
- Frontend has full control

**Cons:**
- Frontend needs type inference (complex!)
- Duplicates logic from backend
- Not practical for interactive editing

---

## Recommended Approach

**Option 1: Type-Driven Resolution in Backend**

### Implementation Plan

1. **Create `operation_resolver.rs`:**
   ```rust
   pub fn resolve_overloaded_operation(
       op_name: &str,
       args: &[Expression],
       inference: &mut TypeInference,
       context_builder: Option<&TypeContextBuilder>
   ) -> Result<String, String>
   ```

2. **Add resolution for common overloads:**
   - `*`, `times`, `scalar_multiply` → Check types, pick right operation
   - `+`, `plus` → Already works (Arithmetic(T) is polymorphic)
   - `/`, `divide` → Check if matrix needs special handling

3. **Integrate into `infer_operation()`:**
   ```rust
   fn infer_operation(...) -> Result<Type, String> {
       // Step 1: Infer argument types
       let arg_types = infer_all_args(...)?;
       
       // Step 2: Resolve operation name based on types
       let resolved_name = resolve_overloaded_operation(name, &arg_types)?;
       
       // Step 3: Type check with resolved name
       context_builder.infer_operation_type(&resolved_name, &arg_types, ...)
   }
   ```

4. **Test cases:**
   - `2 * 3` → `scalar_multiply` ✓
   - `Matrix(2,3) * Matrix(3,2)` → `multiply` ✓
   - `vec * vec` → `dot` or `cross` based on dimension

---

## Benefits

**For Users:**
- Natural mathematical notation
- Just use `×` - system figures it out
- No need to remember different operation names

**For Type System:**
- Still type-safe (checks happen after resolution)
- Leverages existing structure definitions
- Clean separation of concerns

---

## Implementation Estimate

**Time:** 1-2 hours

**Files to modify:**
- `src/operation_resolver.rs` (new file, ~100 lines)
- `src/type_inference.rs` (integrate resolver)
- `tests/operation_resolution_test.rs` (new tests)

**Test cases:**
- Scalar × Scalar
- Matrix × Matrix (compatible dimensions)
- Matrix × Matrix (incompatible dimensions - should still error)
- Vector × Vector (dot vs cross)

---

## Alternative: Document Current Behavior

If we don't implement resolution, we should:

1. **Error message improvement:**
   ```
   ✗ Matrix multiplication requires 'multiply' operation, not 'scalar_multiply'
   
   Hint: Matrix(3,3) × Matrix(3,2) should use:
     - multiply: for matrix multiplication
     - scalar_multiply: only for matrices of SAME dimensions
   
   Did you mean to use matrix multiplication?
   ```

2. **Frontend button labels:**
   - Separate buttons: "× (scalar)" vs "• (matrix mult)"
   - Or: Smart button that changes based on context

---

## Related Issues

**Similar overloading in math:**
- `+` : addition vs union vs concatenation
- `-` : subtraction vs set difference  
- `·` : dot product vs scalar mult vs function composition
- `^` : exponentiation vs wedge product vs XOR

**All need context-aware resolution for natural UX.**

---

## Next Session TODO

1. Implement Option 1 (type-driven resolution)
2. Start with multiply overloading (most critical)
3. Add tests for dimensional compatibility
4. Extend to other overloaded operations
5. Document the resolution strategy

**Goal:** Users can write natural mathematical notation, system handles the rest!

---

## Immediate Fix Needed

**Frontend template mapping is broken!**

The `×` button (or cross template) should generate:
```javascript
// For matrix context:
{ Operation: { name: 'multiply', args: [...] } }  // NOT 'cross'!

// For 3D vectors:
{ Operation: { name: 'cross', args: [...] } }

// For scalars:
{ Operation: { name: 'scalar_multiply', args: [...] } }
```

**Location:** `static/index.html` - astTemplates or button handlers

---

## Current Workaround

**For testing:** Manually construct with correct operation name
- Matrix multiplication: use `"multiply"` operation name
- Avoid using `×` button for matrices until fixed

