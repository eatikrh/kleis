# NEXT SESSION: Fix User-Defined Parametric Types in Signatures

**Current State:** feature/adr-021-data-types (17 commits, 417/417 tests passing)

**Status:** ✅ ADR-021 Complete, 🎯 Ready to tackle parametric types

---

## Mission: Make SignatureInterpreter Generic

**Goal:** Enable user-defined types with arbitrary arity in operation signatures

**Why Critical:** Without this, users can define types but can't use them in structures!

```kleis
// Users can define this:
data Tensor3D(i: Nat, j: Nat, k: Nat) = Tensor3D(...)

// But can't use it in signatures yet:
structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
  operation sum : Tensor3D(i, j, k) → ℝ  // ← Fails! Unknown type
}
```

---

## The Problem

### Current Weirdness (src/signature_interpreter.rs:313)

```rust
fn interpret_type_expr(&self, type_expr: &TypeExpr) -> Result<Type, String> {
    match type_expr {
        TypeExpr::Named(name) => {
            match name.as_str() {
                "ℝ" => Ok(Type::scalar()),
                "T" => Ok(Type::scalar()),
                _ => Ok(Type::scalar()),   // ← EVERYTHING defaults to Scalar!
            }
        }
        
        TypeExpr::Parametric(name, params) => {
            if name == "Matrix" && params.len() >= 2 {
                // ← Hardcoded arity = 2
                Ok(Type::matrix(rows, cols))
            } else if name == "Vector" && params.len() >= 1 {
                // ← Hardcoded arity = 1
                Ok(Type::vector(dim))
            } else {
                Err(format!("Unknown parametric type: {}", name))
            }
        }
    }
}
```

**Problems:**
- ❌ No registry access
- ❌ Hardcoded arities (1 for Vector, 2 for Matrix)
- ❌ User types default to Scalar (wrong!)
- ❌ Can't handle 0, 3, 4+ parameter types

---

## The Solution: Registry-Based Generic Interpretation

### Key Insight (from your question!)

**"Number of parameters could be variable"** - The DataDef TELLS US the arity!

```rust
let data_def = registry.get_type("Tensor3D")?;
data_def.type_params.len()  // → 3 (data-driven!)
data_def.type_params[i].kind  // → "Nat", "Type", etc.
```

### Architecture Changes

```rust
pub struct SignatureInterpreter {
    bindings: HashMap<String, usize>,
    data_registry: DataTypeRegistry,  // ← ADD THIS!
}

fn interpret_type_expr(
    &self,
    type_expr: &TypeExpr,
    registry: &DataTypeRegistry,  // ← ADD THIS!
) -> Result<Type, String> {
    match type_expr {
        TypeExpr::Named(name) => {
            // 1. Check registry for user types
            if registry.has_type(name) {
                return Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args: vec![],
                });
            }
            
            // 2. Check built-ins
            match name.as_str() {
                "ℝ" => Ok(Type::scalar()),
                "Nat" => Ok(Type::Nat),
                _ => Err(format!("Unknown type: {}", name))
            }
        }
        
        TypeExpr::Parametric(name, param_exprs) => {
            // 1. Look up in registry (GENERIC for ANY arity!)
            if let Some(data_def) = registry.get_type(name) {
                // Validate arity
                let expected = data_def.type_params.len();
                if param_exprs.len() != expected {
                    return Err(format!(
                        "Type {} expects {} params, got {}",
                        name, expected, param_exprs.len()
                    ));
                }
                
                // Interpret each param based on its kind
                let mut args = Vec::new();
                for (param_def, param_expr) in 
                    data_def.type_params.iter().zip(param_exprs) 
                {
                    let arg = match param_def.kind.as_deref() {
                        Some("Nat") => {
                            let n = self.eval_param(param_expr)?;
                            Type::NatValue(n)
                        }
                        Some("String") => {
                            let s = self.eval_string_param(param_expr)?;
                            Type::StringValue(s)
                        }
                        Some("Type") | None => {
                            self.interpret_type_expr(param_expr, registry)?
                        }
                        Some(k) => {
                            return Err(format!("Unknown kind: {}", k));
                        }
                    };
                    args.push(arg);
                }
                
                return Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args,
                });
            }
            
            // 2. Fallback to hardcoded (backward compat)
            if name == "Matrix" && param_exprs.len() >= 2 { ... }
        }
    }
}
```

---

## Implementation Plan

### Step 1: Add Registry Field (30 min)

**Files:** `src/signature_interpreter.rs`

- Add `data_registry: DataTypeRegistry` field
- Update constructor to accept registry
- Thread registry through all methods

### Step 2: Update interpret_type_expr Signature (30 min)

**Files:** `src/signature_interpreter.rs`, `src/type_context.rs`

- Add `registry: &DataTypeRegistry` parameter
- Update all call sites
- Update tests

### Step 3: Implement Generic Named Type Lookup (30 min)

**Files:** `src/signature_interpreter.rs`

```rust
TypeExpr::Named(name) => {
    if registry.has_type(name) {
        return Ok(Type::Data {
            type_name: name.clone(),
            constructor: name.clone(),
            args: vec![],
        });
    }
    // fallback to built-ins...
}
```

**Tests:**
- Simple types: Currency, Bool
- In signatures: `operation rate : Currency → ℝ`

### Step 4: Implement Generic Parametric Type Lookup (1 hour)

**Files:** `src/signature_interpreter.rs`

- Look up DataDef by name
- Get arity from `data_def.type_params.len()`
- Validate param count
- Interpret each param by kind
- Construct Type::Data

**Tests:**
- 1-arity: Vector(n), Option(T)
- 2-arity: Matrix(m,n), Result(T,E)
- 3-arity: Tensor3D(i,j,k)
- 4+-arity: Tensor4D, NdArray

### Step 5: Add eval_string_param Helper (30 min)

**Files:** `src/signature_interpreter.rs`

```rust
fn eval_string_param(&self, expr: &TypeExpr) -> Result<String, String> {
    match expr {
        TypeExpr::Named(s) => Ok(s.clone()),
        TypeExpr::Var(s) => Ok(s.clone()),
        _ => Err("Expected string parameter".to_string()),
    }
}
```

### Step 6: Update All Callers (30 min)

**Files:** `src/type_context.rs`, `src/signature_interpreter.rs`

- Thread registry through interpret_signature
- Update TypeContextBuilder to pass registry
- Fix compilation errors

### Step 7: Testing (1 hour)

- Add tests for Currency, Bool (simple)
- Add tests for Option(ℝ), Tensor3D(2,3,4)
- Test arity validation
- Test kind handling
- Run full test suite

---

## Expected Results

### Before Fix
```kleis
data Currency = USD | EUR
structure Tradeable(C) {
  operation rate : C → ℝ
}
implements Tradeable(Currency) {  
  // Currency interpreted as Scalar! ❌
}
```

### After Fix
```kleis
data Currency = USD | EUR
structure Tradeable(C) {
  operation rate : C → ℝ
}
implements Tradeable(Currency) {  
  // Currency correctly interpreted as Type::Data { "Currency", ... } ✅
}

// Now users can define:
data Tensor3D(i: Nat, j: Nat, k: Nat) = Tensor3D(...)
structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
  operation sum : Tensor3D(i, j, k) → ℝ  // ✅ Works!
}
```

---

## Files to Modify

| File | Changes | Complexity |
|------|---------|------------|
| `src/signature_interpreter.rs` | Add registry field, update interpret_type_expr | HIGH |
| `src/type_context.rs` | Thread registry to interpreter | MEDIUM |
| `tests/signature_dimension_test.rs` | Add user type tests | LOW |
| New test file | Test arbitrary arity | LOW |

**Estimated lines:** ~200-300 lines changed  
**Estimated time:** 3-4 hours total

---

## Success Criteria

After implementation:
- ✅ Simple user types work (Currency, Bool)
- ✅ Parametric user types work (Option(T))
- ✅ Arbitrary arity works (0, 1, 2, 3, 4+)
- ✅ Mixed kinds work (Nat + Type parameters)
- ✅ All 417+ tests still pass
- ✅ No more hardcoded Matrix/Vector special cases (or minimal)
- ✅ interpret_type_expr is clean and generic

---

## Testing Plan

### Test 1: Simple User Type
```rust
#[test]
fn test_simple_user_type_in_signature() {
    let mut checker = TypeChecker::new();
    checker.load_data_types("data Currency = USD | EUR").unwrap();
    checker.load_kleis("
        structure Tradeable(C) {
            operation rate : C → ℝ
        }
        implements Tradeable(Currency) {
            operation rate = builtin_rate
        }
    ").unwrap();
    
    assert!(checker.type_supports_operation("Currency", "rate"));
}
```

### Test 2: Parametric User Type
```rust
#[test]
fn test_parametric_user_type() {
    let mut checker = TypeChecker::new();
    checker.load_data_types("data Option(T) = None | Some(T)").unwrap();
    // Check that Option(ℝ) works in signatures
}
```

### Test 3: 3-Parameter Type
```rust
#[test]
fn test_tensor3d_type() {
    let mut checker = TypeChecker::new();
    checker.load_data_types("data Tensor3D(i: Nat, j: Nat, k: Nat) = Tensor3D(...)").unwrap();
    checker.load_kleis("
        structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
            operation sum : Tensor3D(i, j, k) → ℝ
        }
        implements Tensor3DOps(10, 20, 30) {
            operation sum = builtin_sum
        }
    ").unwrap();
    
    assert!(checker.type_supports_operation("Tensor3D(10, 20, 30)", "sum"));
}
```

---

## Documentation Reference

See these files for complete analysis:
- `docs/session-2024-12-08/USER_DEFINED_TYPES_IN_SIGNATURES.md`
- `docs/session-2024-12-08/ARBITRARY_ARITY_TYPES.md`

These contain:
- Complete problem statement
- 4 levels of complexity
- 3 solution approaches
- Concrete examples
- Implementation pseudocode

---

## Current Branch Status

**Branch:** `feature/adr-021-data-types`  
**Commits:** 17  
**Tests:** 417/417 passing ✅  
**Quality:** All checks pass ✅  
**Docs:** Comprehensive ✅  

**Ready to:**
1. Push current work to GitHub (backup)
2. Continue on same branch with parametric type fix
3. Complete the full ADR-021 vision

---

## What Makes This "The Right Way"

**Data-Driven:** Arity and kinds from DataDef (not hardcoded)  
**Generic:** One algorithm handles ALL arities  
**Extensible:** Users can define ANY parametric structure  
**Clean:** No special cases, no weirdness

Once fixed, `interpret_type_expr` becomes a beautiful, generic, extensible function that respects the registry - exactly what ADR-021 envisioned!

---

## Next Session First Action

1. Review `USER_DEFINED_TYPES_IN_SIGNATURES.md`
2. Review `ARBITRARY_ARITY_TYPES.md`
3. Start Step 1: Add `data_registry` field to SignatureInterpreter
4. Implement Phase 1-3 (3-4 hours)
5. Test thoroughly
6. Commit and celebrate! 🎉

**Then:** Merge to main with full user-defined parametric type support!

---

**Status:** 🎯 Clear mission for next session  
**Docs:** Complete analysis ready  
**Branch:** Clean and tested  
**Goal:** Make Kleis truly extensible for ANY user-defined parametric types!
