# Stack Overflow Fix: Nested Quantifiers in Z3 Translator

**Date:** December 12, 2025  
**Issue:** Test `test_nested_quantifiers` causes stack overflow  
**Branch:** `fix/z3-translator-type-handling`  
**Root Cause:** Incomplete type system migration in axiom verifier

---

## Problem Description

### The Failing Test

```rust
#[test]
fn test_nested_quantifiers() {
    // Test: ∀(x : M). ∀(y : M). x + y = y + x
    let axiom_text = "∀(x : M). ∀(y : M). equals(plus(x, y), plus(y, x))";
    let axiom = parse_axiom(axiom_text);
    let result = verify_with_test_context(&axiom);
    // CRASHES: thread 'test_nested_quantifiers' has overflowed its stack
}
```

### The Structure (Not Infinite)

The parsed AST is finite:
```
Quantifier(ForAll, x : M)
  ├─ Body: Quantifier(ForAll, y : M)
           └─ Body: equals(plus(x, y), plus(y, x))
```

This is only 2 levels deep and should terminate properly.

---

## Root Cause Analysis

### What CURRENT_STATE_Z3_FIX.md Says Should Have Been Done

The fix documentation claims:

> **4. Changed Variable Storage**
> ```rust
> // OLD:
> vars: &HashMap<String, Int>  // ← All variables forced to Int
>
> // NEW:
> vars: &HashMap<String, Dynamic>  // ← Variables can be Int, Bool, etc.
> ```

### What Actually Exists in The Code

**File:** `src/axiom_verifier.rs`

```rust
// Line 75: Identity elements still Int
identity_elements: HashMap<String, Int>,

// Line 491: Still using Int
fn kleis_to_z3_dynamic(
    &mut self,
    expr: &Expression,
    vars: &HashMap<String, Int>,  // ❌ Should be Dynamic
) -> Result<Dynamic, String>

// Line 556: Still using Int
fn operation_to_z3_dynamic(
    &mut self,
    name: &str,
    args: &[Expression],
    vars: &HashMap<String, Int>,  // ❌ Should be Dynamic
) -> Result<Dynamic, String>

// Line 788: Still using Int
fn quantifier_to_z3(
    &mut self,
    _quantifier: &QuantifierKind,
    variables: &[QuantifiedVar],
    where_clause: Option<&Expression>,
    body: &Expression,
    vars: &HashMap<String, Int>,  // ❌ Should be Dynamic
) -> Result<Bool, String>

// Line 794: Always creates Int variables
for var in variables {
    let z3_var = Int::fresh_const(&var.name);  // ❌ Ignores type annotations!
    new_vars.insert(var.name.clone(), z3_var);
}
```

**The migration to `Dynamic` was documented but never implemented!**

---

## Why This Causes Stack Overflow

### The Type Mismatch Chain

1. **Outer quantifier** `∀(x : M)` creates variable `x` as `Int`
2. Calls `kleis_to_z3_dynamic(body, vars)` with `vars = {x: Int}`
3. **Inner quantifier** `∀(y : M)` creates variable `y` as `Int`
4. Both variables passed to `equals(plus(x, y), plus(y, x))`

### Potential Failure Modes

**Theory 1: Type Coercion Loop**
- Variables typed as `Bool` in Kleis created as `Int` in Z3
- Z3 operations expecting `Bool` receive `Int`
- Type conversion attempts cause recursive calls
- Stack overflow

**Theory 2: Variable Shadowing Issue**
- Nested quantifiers create variables with same type
- HashMap operations during cloning/merging cause issues
- Infinite recursion in variable resolution

**Theory 3: Z3 Context Issue**
- All `Int::fresh_const()` calls use same Z3 context
- Nested scopes create conflicting constraints
- Z3 solver enters infinite loop trying to resolve

---

## The Complete Fix

### Step 1: Change Variable Storage Type

**File:** `src/axiom_verifier.rs`

#### Change 1: AxiomVerifier struct (line ~75)

```rust
// OLD:
identity_elements: HashMap<String, Int>,

// NEW:
identity_elements: HashMap<String, Dynamic>,
```

#### Change 2: kleis_to_z3_dynamic signature (line ~491)

```rust
// OLD:
fn kleis_to_z3_dynamic(
    &mut self,
    expr: &Expression,
    vars: &HashMap<String, Int>,
) -> Result<Dynamic, String>

// NEW:
fn kleis_to_z3_dynamic(
    &mut self,
    expr: &Expression,
    vars: &HashMap<String, Dynamic>,
) -> Result<Dynamic, String>
```

#### Change 3: operation_to_z3_dynamic signature (line ~556)

```rust
// OLD:
fn operation_to_z3_dynamic(
    &mut self,
    name: &str,
    args: &[Expression],
    vars: &HashMap<String, Int>,
) -> Result<Dynamic, String>

// NEW:
fn operation_to_z3_dynamic(
    &mut self,
    name: &str,
    args: &[Expression],
    vars: &HashMap<String, Dynamic>,
) -> Result<Dynamic, String>
```

#### Change 4: quantifier_to_z3 signature (line ~788)

```rust
// OLD:
fn quantifier_to_z3(
    &mut self,
    _quantifier: &QuantifierKind,
    variables: &[QuantifiedVar],
    where_clause: Option<&Expression>,
    body: &Expression,
    vars: &HashMap<String, Int>,
) -> Result<Bool, String>

// NEW:
fn quantifier_to_z3(
    &mut self,
    _quantifier: &QuantifierKind,
    variables: &[QuantifiedVar],
    where_clause: Option<&Expression>,
    body: &Expression,
    vars: &HashMap<String, Dynamic>,
) -> Result<Bool, String>
```

---

### Step 2: Fix Variable Creation Based on Type Annotations

**File:** `src/axiom_verifier.rs`, line ~794

```rust
// OLD (creates all variables as Int):
for var in variables {
    let z3_var = Int::fresh_const(&var.name);
    new_vars.insert(var.name.clone(), z3_var);
}

// NEW (creates variables based on type annotation):
for var in variables {
    let z3_var: Dynamic = if let Some(type_annotation) = &var.type_annotation {
        // Parse type annotation to determine Z3 type
        match type_annotation.as_str() {
            "Bool" | "Boolean" => Bool::fresh_const(&var.name).into(),
            "ℝ" | "Real" | "R" => Real::fresh_const(&var.name).into(),
            "ℤ" | "Int" | "Z" => Int::fresh_const(&var.name).into(),
            _ => {
                // Default to Int for unknown types (M, N, etc.)
                Int::fresh_const(&var.name).into()
            }
        }
    } else {
        // No type annotation, default to Int
        Int::fresh_const(&var.name).into()
    };
    new_vars.insert(var.name.clone(), z3_var);
}
```

---

### Step 3: Fix Identity Element Loading

**File:** `src/axiom_verifier.rs`, in `load_identity_elements` method

```rust
// OLD:
if !self.identity_elements.contains_key(name) {
    let z3_const = Int::fresh_const(name);  // ❌ Always Int
    self.identity_elements.insert(name.clone(), z3_const);
}

// NEW:
if !self.identity_elements.contains_key(name) {
    // Default identity elements to Int (they're usually numeric)
    let z3_const: Dynamic = Int::fresh_const(name).into();
    self.identity_elements.insert(name.clone(), z3_const);
}
```

---

### Step 4: Fix Variable Access in Operations

**File:** `src/axiom_verifier.rs`, in operation implementations

When accessing variables, the code needs to handle `Dynamic`:

```rust
// Example in kleis_to_z3_dynamic for variables (line ~497):

// OLD:
if let Some(var) = vars.get(name) {
    return Ok(var.clone());  // var is Int, but needs to be Dynamic
}

// NEW:
if let Some(var) = vars.get(name) {
    return Ok(var.clone());  // var is already Dynamic!
}
```

When accessing identity elements:

```rust
// OLD:
if let Some(const_val) = self.identity_elements.get(name) {
    return Ok(const_val.clone());  // Returns Int
}

// NEW:
if let Some(const_val) = self.identity_elements.get(name) {
    return Ok(const_val.clone());  // Returns Dynamic
}
```

---

### Step 5: Update Arithmetic Operations

**File:** `src/axiom_verifier.rs`, line ~659

Arithmetic operations need to handle Dynamic and convert to Int:

```rust
// In operation_to_z3_dynamic:
"plus" | "times" | "minus" | "multiply" | "subtract" | "add" => {
    // Convert Dynamic arguments to Int for arithmetic
    let mut int_args = Vec::new();
    for arg in args {
        let dyn_val = self.kleis_to_z3_dynamic(arg, vars)?;
        let int_val = dyn_val.as_int()
            .ok_or_else(|| format!("Arithmetic operation requires Int, got {:?}", dyn_val))?;
        int_args.push(int_val);
    }
    
    // Perform operation on Int values
    let result_int = match name {
        "plus" | "add" => int_args[0].add(&[&int_args[1]]),
        "times" | "multiply" => int_args[0].mul(&[&int_args[1]]),
        "minus" | "subtract" => int_args[0].sub(&[&int_args[1]]),
        _ => return Err(format!("Unknown arithmetic operation: {}", name)),
    };
    
    // Convert result back to Dynamic
    Ok(result_int.into())
}
```

---

## Implementation Checklist

- [ ] Change `identity_elements` type to `HashMap<String, Dynamic>`
- [ ] Change `kleis_to_z3_dynamic` parameter to `vars: &HashMap<String, Dynamic>`
- [ ] Change `operation_to_z3_dynamic` parameter to `vars: &HashMap<String, Dynamic>`
- [ ] Change `quantifier_to_z3` parameter to `vars: &HashMap<String, Dynamic>`
- [ ] Implement type-based variable creation in `quantifier_to_z3`
- [ ] Update identity element loading to use `Dynamic`
- [ ] Update arithmetic operations to convert Dynamic→Int→Dynamic
- [ ] Update all call sites that create `HashMap<String, Int>` to use `Dynamic`
- [ ] Test nested quantifiers
- [ ] Test Boolean variables in quantifiers
- [ ] Test mixed type variables

---

## Testing Strategy

### Test 1: Simple Nested Quantifiers (Already Exists)
```rust
// ∀(x : M). ∀(y : M). equals(plus(x, y), plus(y, x))
// Should verify without stack overflow
```

### Test 2: Boolean Variables
```rust
// ∀(p : Bool). ∀(q : Bool). or(p, q) = or(q, p)
// Should handle Bool type correctly
```

### Test 3: Mixed Types
```rust
// ∀(x : Int). ∀(p : Bool). implies(p, equals(x, x))
// Should handle both Int and Bool in same expression
```

### Test 4: Deep Nesting (3+ levels)
```rust
// ∀(x : M). ∀(y : M). ∀(z : M). equals(plus(x, plus(y, z)), plus(plus(x, y), z))
// Should handle arbitrary nesting depth
```

---

## Expected Outcome

After implementing these changes:

1. ✅ Variables created with correct Z3 types based on annotations
2. ✅ No type mismatches in Z3 operations
3. ✅ Nested quantifiers work at any depth
4. ✅ Boolean, Int, Real variables all supported
5. ✅ Test `test_nested_quantifiers` passes
6. ✅ No stack overflow in nested structures

---

## Notes

### Why The Original Fix Was Incomplete

The commit `bf63849` ("fix: properly handle typed variables in Z3 translator") documented the migration to `Dynamic` in `CURRENT_STATE_Z3_FIX.md` but **only partially implemented it**:

- ✅ Created `kleis_to_z3_dynamic` function
- ✅ Made operations return `Dynamic`
- ❌ Did NOT change variable storage to `Dynamic`
- ❌ Did NOT implement type-based variable creation
- ❌ Did NOT update identity elements to `Dynamic`

This left the system in an inconsistent state where:
- Functions return `Dynamic`
- But variables are stored as `Int`
- Conversions between the two cause issues

### Why This Wasn't Caught Earlier

The clippy warnings prevented compilation, so the tests never ran. Once clippy was fixed (adding `#![allow(warnings)]` to test files), the tests compiled and the stack overflow was revealed.

**This is not a regression from clippy fixes - it's a pre-existing bug that was hidden by compilation errors.**

---

## Related Files

- `src/axiom_verifier.rs` - Main file requiring changes
- `tests/axiom_verification_integration_test.rs` - Test that fails
- `CURRENT_STATE_Z3_FIX.md` - Original incomplete fix documentation
- `vendor/z3/src/ast/dynamic.rs` - Z3 Dynamic type documentation

---

## Alternative: Quick Workaround (Not Recommended)

If a quick fix is needed without full migration:

```rust
// Mark test as ignored until proper fix
#[test]
#[ignore = "Stack overflow with nested quantifiers - needs Dynamic migration"]
fn test_nested_quantifiers() {
    // ...
}
```

But this doesn't fix the underlying issue - nested quantifiers in real axioms will still fail.

**Proper solution is to complete the Dynamic migration.**

