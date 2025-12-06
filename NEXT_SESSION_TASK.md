# NEXT SESSION: Remove Hardcoded Matrix Logic from type_inference.rs

**CRITICAL TASK:** Fix ADR-016 violation in `src/type_inference.rs` lines 207-396

**Status:** SignatureInterpreter exists and works! Just need to wire it up properly.

---

## The Problem (MUST FIX)

**File:** `src/type_inference.rs`  
**Lines:** 207-396 (approximately)  
**Issue:** Hardcoded matrix-specific type rules violate ADR-016

**Specific hardcoded sections:**
1. Lines 207-245: `"scalar_multiply" | "times"` - Matrix multiplication logic
2. Lines 297-333: `"multiply"` - Matrix multiplication (duplicate!)
3. Lines 335-361: `"add"` - Matrix addition
4. Lines 363-376: `"transpose"` - Transpose logic
5. Lines 378-391: `"det" | "determinant"` - Determinant logic
6. Lines 393-406: `"trace"` - Trace logic

**All these need to be DELETED and replaced with delegation to TypeContextBuilder!**

---

## What Already Works ✅

**SignatureInterpreter (src/signature_interpreter.rs):**
- ✅ Created and tested
- ✅ Reads operation signatures from structures
- ✅ Interprets `Matrix(n, m, T)` correctly
- ✅ Test passing: `test_interpret_transpose_signature`

**TypeContextBuilder (src/type_context.rs):**
- ✅ Has `infer_operation_type()` method
- ✅ Uses SignatureInterpreter for transpose, add, det
- ✅ Can read from stdlib/matrices.kleis

**The infrastructure is READY!**

---

## The Solution (Step by Step)

### Step 1: Make infer_operation() delegate to context_builder

**Current code (src/type_inference.rs line ~191):**
```rust
fn infer_operation(&mut self, name: &str, args: &[Expression]) -> Result<Type, String> {
    match name {
        "plus" | "minus" => { /* OK - basic arithmetic */ }
        
        "scalar_multiply" | "times" => {
            // ❌ DELETE lines 207-245 (hardcoded matrix logic)
        }
        
        "multiply" => {
            // ❌ DELETE lines 297-333
        }
        
        "add" => {
            // ❌ DELETE lines 335-361
        }
        
        "transpose" => {
            // ❌ DELETE lines 363-376
        }
        
        // etc...
        
        _ => { /* unknown */ }
    }
}
```

**Replace with:**
```rust
fn infer_operation(
    &mut self,
    name: &str,
    args: &[Expression],
    context_builder: Option<&crate::type_context::TypeContextBuilder>,  // ADD THIS
) -> Result<Type, String> {
    match name {
        // Keep basic arithmetic (truly primitive)
        "plus" | "minus" | "scalar_divide" | "frac" | "sqrt" | "sup" | "power" => {
            /* existing logic - these are primitive */
        }
        
        // Everything else: delegate to context_builder!
        _ => {
            if let Some(builder) = context_builder {
                // Infer argument types first
                let arg_types: Vec<Type> = args.iter()
                    .map(|arg| self.infer(arg, context_builder))  // Recursive call
                    .collect::<Result<Vec<_>, _>>()?;
                
                // Delegate to builder!
                builder.infer_operation_type(name, &arg_types)
            } else {
                // No context builder - return unknown
                for arg in args {
                    self.infer(arg, context_builder)?;
                }
                Ok(self.context.fresh_var())
            }
        }
    }
}
```

### Step 2: Thread context_builder through call chain

**Update `infer()` signature:**
```rust
// OLD:
pub fn infer(&mut self, expr: &Expression) -> Result<Type, String>

// NEW:
pub fn infer(
    &mut self,
    expr: &Expression,
    context_builder: Option<&crate::type_context::TypeContextBuilder>,
) -> Result<Type, String>
```

**Update the match in `infer()`:**
```rust
Expression::Operation { name, args } => {
    self.infer_operation(name, args, context_builder)  // Pass it through!
}
```

### Step 3: Update infer_and_solve()

```rust
// OLD:
pub fn infer_and_solve(&mut self, expr: &Expression) -> Result<Type, String>

// NEW:
pub fn infer_and_solve(
    &mut self,
    expr: &Expression,
    context_builder: Option<&crate::type_context::TypeContextBuilder>,
) -> Result<Type, String> {
    let ty = self.infer(expr, context_builder)?;  // Pass it through
    let subst = self.solve()?;
    Ok(subst.apply(&ty))
}
```

### Step 4: Update TypeChecker.check()

```rust
pub fn check(&mut self, expr: &Expression) -> TypeCheckResult {
    match self.inference.infer_and_solve(expr, Some(&self.context_builder)) {
        // ✅ NOW context_builder is passed!
        ...
    }
}
```

### Step 5: Delete hardcoded matrix logic

**Delete these entire match arms:**
- `"scalar_multiply" | "times"` (lines ~212-245) → **DELETE**
- `"multiply"` (lines ~297-333) → **DELETE**
- `"add"` (lines ~335-361) → **DELETE**
- `"transpose"` (lines ~363-376) → **DELETE**
- `"det" | "determinant"` (lines ~378-391) → **DELETE**
- `"trace"` (lines ~393-406) → **DELETE**
- Matrix construction (lines ~408-430) → **KEEP but move to context_builder**

**Total: Delete ~180 lines of hardcoded logic!**

---

## Testing Checklist

After changes, verify:

```bash
# 1. Library tests
cargo test --lib
# Should pass: 280 tests

# 2. Matrix type inference
cargo run --bin test_matrix_type_inference
# Should pass all 7 tests

# 3. Format
cargo fmt

# 4. Server builds
cargo build --bin server --release

# 5. Live demo
# Start server, create matrix, see type feedback
```

---

## Files to Modify

1. **src/type_inference.rs** (~200 lines changed)
   - Add `context_builder` parameter to methods
   - Delete hardcoded matrix logic
   - Keep only primitive operations

2. **src/type_checker.rs** (~5 lines changed)
   - Pass `Some(&self.context_builder)` to inference

3. **src/type_context.rs** (maybe ~20 lines)
   - Finish signature interpreter for multiply, trace
   - Handle matrix construction

---

## Expected Result

**After this refactor:**
- ✅ ZERO hardcoded matrix logic in type_inference.rs
- ✅ ALL rules read from stdlib/matrices.kleis
- ✅ Pure ADR-016 compliance
- ✅ All tests passing
- ✅ Live demo still works

**Time estimate:** 30-60 minutes focused work

---

## Current Git State

**Branch:** main  
**Commits ahead:** 12  
**Last commit:** f00917a "feat: Signature interpreter for add and det operations"  
**Tests:** 280 passing  
**Tag ready:** v0.3.0-type-inference (but wait for pure compliance!)

---

## Success Criteria

When done, `src/type_inference.rs` should have:
- ✅ Basic arithmetic only (plus, minus, divide, sqrt, power)
- ✅ Generic delegation: `builder.infer_operation_type(name, &arg_types)`
- ✅ NO matrix-specific code
- ✅ NO hardcoded type rules

**This is achievable in 30-60 minutes with fresh context!**

---

**Ready for new session. This is the final push to pure ADR-016!** 🚀


