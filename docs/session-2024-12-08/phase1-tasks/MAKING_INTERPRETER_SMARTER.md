# Making SignatureInterpreter Smarter

**Date:** December 8, 2024  
**Goal:** Make SignatureInterpreter handle type variables and constraints  
**Impact:** Reduce match statement from ~220 lines to ~50 lines

---

## Current SignatureInterpreter

### **What it CAN do:**

✅ Parse operation signatures from structures  
✅ Extract dimensions from Matrix types  
✅ Substitute into result types

**Example:**
```kleis
structure Matrix(m: Nat, n: Nat, T) {
  operation transpose : Matrix(n, m, T)
}

transpose(Matrix(2, 3)) → Matrix(3, 2) ✓
```

---

### **What it CAN'T do (yet):**

❌ Handle type variables (`Var(TypeVar(0))`)  
❌ Express dimension constraints  
❌ Handle polymorphic operations over different types  
❌ Partial type information  

**Example (fails):**
```rust
plus(Var(0), Scalar) → ???
// Should infer: Var(0) = Scalar, result = Scalar
// Currently: Can't handle Var
```

---

## What Needs to Be Added

### **1. Type Variable Support**

```rust
impl SignatureInterpreter {
    /// Handle type variables in argument types
    /// When we see Var(α), try to infer what it should be
    pub fn handle_type_variable(
        &mut self,
        var: &TypeVar,
        expected: &Type
    ) -> Result<Type, String> {
        // Return the expected type
        // Let the constraint solver handle unification later
        Ok(expected.clone())
    }
}
```

---

### **2. Constraint-Based Matching**

```rust
/// Match operation signature with argument types
/// Handles type variables and generates constraints
pub fn match_signature(
    &mut self,
    signature: &TypeExpr,
    arg_types: &[Type]
) -> Result<(Type, Vec<Constraint>), String> {
    // Parse signature: T → T → T
    // Match with args: [Var(0), Scalar]
    // Generate constraint: Var(0) = Scalar
    // Return: (Scalar, [Var(0) = Scalar])
}
```

---

### **3. Polymorphic Type Matching**

```rust
/// Match a polymorphic signature like: T → T → T
/// With concrete types: [ℝ, ℝ]
/// Or with variables: [Var(0), ℝ]
pub fn match_polymorphic(
    &mut self,
    type_param: &str,  // "T"
    arg_types: &[Type]
) -> Result<Type, String> {
    // Find what T should be by looking at args
    // If all concrete and same: T = that type
    // If has variables: T = most specific known type
}
```

---

## Implementation Plan

### **Phase A: Type Variable Support** (~2-3 hours)

**Goal:** Handle `Var(_)` in argument types

```rust
// Current interpreter panics on Var
// New interpreter gracefully handles it

match arg_type {
    Type::Var(_) => {
        // Don't bind, just note that this is unknown
        // Return a type that's compatible with the signature
    }
    Type::Scalar => {
        // Bind as before
    }
    // ...
}
```

---

### **Phase B: Generic Operation Handler** (~3-4 hours)

**Create a generic handler for T → T → T operations:**

```rust
fn infer_binary_same_type_op(
    &self,
    op_name: &str,
    arg_types: &[Type]
) -> Result<Type, String> {
    if arg_types.len() != 2 {
        return Err(format!("{} requires 2 arguments", op_name));
    }
    
    match (&arg_types[0], &arg_types[1]) {
        // Both concrete and same
        (Type::Scalar, Type::Scalar) => Ok(Type::Scalar),
        (Type::Matrix(m1,n1), Type::Matrix(m2,n2)) if m1==m2 && n1==n2 
            => Ok(Type::Matrix(*m1, *n1)),
        
        // Type variables
        (Type::Var(_), t) | (t, Type::Var(_)) => Ok(t.clone()),
        (Type::Var(_), Type::Var(_)) => Ok(arg_types[0].clone()),
        
        // Mismatch
        _ => Err(format!("{} requires compatible types", op_name))
    }
}
```

**Then arithmetic becomes:**
```rust
"plus" | "minus" | "times" | "divide" | ... => {
    self.infer_binary_same_type_op(op_name, arg_types)
}
```

**7 cases → 1 helper function!**

---

### **Phase C: Dimension-Aware Operations** (~2-3 hours)

**Extract dimension checking:**

```rust
fn check_dimensions_match(
    op_name: &str,
    t1: &Type,
    t2: &Type
) -> Result<(), String> {
    match (t1, t2) {
        (Type::Matrix(m1,n1), Type::Matrix(m2,n2)) => {
            if m1 != m2 || n1 != n2 {
                return Err(format!(
                    "{}: dimensions must match! {}×{} ≠ {}×{}",
                    op_name, m1, n1, m2, n2
                ));
            }
            Ok(())
        }
        _ => Ok(())
    }
}
```

**Then use it:**
```rust
"add" => {
    check_dimensions_match("add", &arg_types[0], &arg_types[1])?;
    self.infer_binary_same_type_op("add", arg_types)
}
```

---

## Expected Reduction

### **Before:**

```rust
match op_name {
    "transpose" => { /* 10 lines */ }
    "add" => { /* 30 lines */ }
    "multiply" => { /* 24 lines */ }
    "det" | "determinant" => { /* 25 lines */ }
    "trace" => { /* 15 lines */ }
    "plus" | "minus" | ... => { /* 35 lines */ }
    "abs" | "floor" | "sqrt" => { /* 10 lines */ }
    "power" | "sup" | "sub" => { /* 15 lines */ }
    "derivative" | "integral" | ... => { /* 8 lines */ }
    "int_bounds" => { /* 12 lines */ }
    "equals" | "not_equals" => { /* 10 lines */ }
    "less_than" | ... => { /* 18 lines */ }
    _ => { /* fallback */ }
}

Total: ~220 lines in match
```

---

### **After Refactoring:**

```rust
// Helper functions (extracted, reusable)
fn infer_binary_same_type_op(...) { /* 20 lines */ }
fn check_dimensions_match(...) { /* 15 lines */ }
fn check_square_matrix(...) { /* 10 lines */ }

match op_name {
    // Matrix-specific (still need special handling)
    "transpose" | "add" | "multiply" => {
        // Dimension checks + generic handler
        /* ~5 lines each = 15 lines total */
    }
    
    "det" | "trace" => {
        check_square_matrix(&arg_types[0])?;
        /* ~5 lines */
    }
    
    // All arithmetic in one case!
    "plus" | "minus" | "times" | "divide" | ... => {
        infer_binary_same_type_op(op_name, arg_types)
        /* 1 line! */
    }
    
    // All numeric in one case!
    "abs" | "floor" | "sqrt" | "power" | "sup" | "sub" => {
        infer_unary_or_binary_numeric(op_name, arg_types)
        /* 1 line! */
    }
    
    // Special cases remain
    "equals" | "not_equals" => { /* 5 lines */ }
    "less_than" | ... => { /* 10 lines */ }
    "derivative" | "integral" | "int_bounds" => { /* 10 lines */ }
    
    _ => { /* fallback */ }
}

Helper functions: ~45 lines
Match statement: ~50 lines
Total: ~95 lines (was 220)
```

**Reduction: ~125 lines (~57% smaller!)** ✓

---

## Work Estimate

**Phase A:** Type variable support (2-3 hours)  
**Phase B:** Generic handlers (3-4 hours)  
**Phase C:** Dimension checking (2-3 hours)  
**Testing:** (1-2 hours)

**Total: 8-12 hours (~1-1.5 days)**

---

## Let's Do It!

This will make the code:
- ✅ Cleaner (57% smaller)
- ✅ More maintainable (helper functions)
- ✅ More reusable (extract patterns)
- ✅ True ADR-016 (less hardcoding)

**Ready to start Phase A!** 🚀

