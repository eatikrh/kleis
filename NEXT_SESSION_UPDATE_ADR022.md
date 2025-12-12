# Next Session: Update ADR-022 Generic Translator Code

**Date Created:** December 12, 2024  
**Priority:** Documentation Accuracy  
**File:** `docs/adr/adr-022-z3-integration-for-axiom-verification.md`

---

## Issue

The generic translator code example in ADR-022 is **outdated** and shows the old implementation that caused the stack overflow.

**Location:** https://github.com/eatikrh/kleis/blob/main/docs/adr/adr-022-z3-integration-for-axiom-verification.md

Section: "Generic Translator"

---

## Current (Outdated) Code in ADR

```rust
fn kleis_to_z3(expr: &Expression) -> Result<Bool> {
    match expr {
        Expression::Operation { name, args } => {
            match name.as_str() {
                "plus" => Int::add(...),
                "times" => Int::mul(...),
                "logical_and" => Bool::and(...),
                // Extensible - just add more cases!
            }
        }
        Expression::Quantifier { variables, body, .. } => {
            // Create fresh Z3 variables
            // Translate body recursively
        }
    }
}
```

**Problems with this code:**
1. ❌ Returns `Result<Bool>` - forces everything to Bool!
2. ❌ No variable storage shown - how are variables tracked?
3. ❌ This was the OLD signature that caused stack overflow
4. ❌ Doesn't show Dynamic type system

---

## What It Should Show (After December 12, 2024 Fix)

```rust
fn kleis_to_z3_dynamic(
    expr: &Expression,
    vars: &HashMap<String, Dynamic>  // Variables can be Int, Bool, Real, etc.
) -> Result<Dynamic> {  // Returns Dynamic, not Bool!
    match expr {
        Expression::Object(name) => {
            // Return actual variable (Dynamic type)
            if let Some(var) = vars.get(name) {
                return Ok(var.clone());
            }
            // Check identity elements...
        }
        
        Expression::Operation { name, args } => {
            match name.as_str() {
                "equals" => {
                    let left = self.kleis_to_z3_dynamic(&args[0], vars)?;
                    let right = self.kleis_to_z3_dynamic(&args[1], vars)?;
                    Ok(left.eq(&right).into())  // Returns Bool as Dynamic
                }
                "plus" => {
                    let left = self.kleis_to_z3_dynamic(&args[0], vars)?;
                    let right = self.kleis_to_z3_dynamic(&args[1], vars)?;
                    // Handle Int, Real, and mixed types
                    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
                        Ok(Int::add(&[&l, &r]).into())
                    } else if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
                        Ok(Real::add(&[&l, &r]).into())
                    } else {
                        // Convert mixed Int/Real to Real
                        let l = left.as_real().or_else(|| left.as_int().map(|i| i.to_real()));
                        let r = right.as_real().or_else(|| right.as_int().map(|i| i.to_real()));
                        Ok(Real::add(&[&l?, &r?]).into())
                    }
                }
                "logical_and" => {
                    let left = self.kleis_to_z3_dynamic(&args[0], vars)?;
                    let right = self.kleis_to_z3_dynamic(&args[1], vars)?;
                    let l = left.as_bool().ok_or("and requires Bool")?;
                    let r = right.as_bool().ok_or("and requires Bool")?;
                    Ok(l.and(&[&r]).into())  // Returns Bool as Dynamic
                }
                // Extensible - just add more cases!
                _ => {
                    // Uninterpreted function for unknown operations
                    let func_decl = self.declare_operation(name, args.len());
                    Ok(func_decl.apply(&ast_args))
                }
            }
        }
        
        Expression::Quantifier { variables, body, .. } => {
            // Create fresh Z3 variables based on type annotations
            let mut new_vars = vars.clone();
            for var in variables {
                let z3_var: Dynamic = if let Some(ty) = &var.type_annotation {
                    match ty.as_str() {
                        "Bool" => Bool::fresh_const(&var.name).into(),
                        "R" | "Real" => Real::fresh_const(&var.name).into(),
                        _ => Int::fresh_const(&var.name).into(),
                    }
                } else {
                    Int::fresh_const(&var.name).into()
                };
                new_vars.insert(var.name.clone(), z3_var);
            }
            
            // Translate body with new variables
            let body_z3 = self.kleis_to_z3_dynamic(body, &new_vars)?;
            Ok(body_z3)
        }
    }
}
```

---

## Key Points to Update

1. **Function name:** `kleis_to_z3` → `kleis_to_z3_dynamic`
2. **Return type:** `Result<Bool>` → `Result<Dynamic>`
3. **Variable storage:** Show `HashMap<String, Dynamic>` parameter
4. **Type-based creation:** Show how variables get correct types
5. **Mixed type handling:** Show Int+Real conversion
6. **Dynamic conversions:** Show `.as_int()`, `.as_bool()`, `.as_real()`

---

## Context: Why This Matters

### The Old Code Caused Stack Overflow

The example in ADR-022 shows the **problematic pattern** that led to:
- Stack overflow in nested quantifiers
- Type mismatches (Real variables forced to Int)
- Incorrect variable handling

### The Fix (December 12, 2024)

Complete migration to Dynamic type system:
- Variables stored as `Dynamic`
- Type-based creation from annotations
- Mixed type arithmetic (Int+Real → Real)
- Nested quantifiers work at any depth

**PR #8 merged:** Complete Dynamic migration

---

## Files to Update

1. **docs/adr/adr-022-z3-integration-for-axiom-verification.md**
   - Update "Generic Translator" section
   - Show correct `kleis_to_z3_dynamic` signature
   - Add note about Dynamic type system
   
2. **Consider adding note:**
   - Original implementation had limitations
   - Fixed in December 12, 2024 (PR #8)
   - See Z3_DYNAMIC_FIX_COMPLETE.md for details

---

## Related Documentation

- `STACK_OVERFLOW_FIX.md` - Why the old code failed
- `Z3_DYNAMIC_FIX_COMPLETE.md` - Complete fix documentation
- `src/axiom_verifier.rs` - Current implementation (source of truth)

---

**Action for next session:** Update ADR-022 generic translator example to reflect actual working implementation.


