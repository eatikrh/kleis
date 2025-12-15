# Grammar v0.6 Function Integration - Implementation Plan

**Date:** December 12, 2025  
**Status:** Ready to implement  
**TODO:** #57 - Integrate StructureMember::FunctionDef with Z3 and Evaluator

---

## ✅ What We've Proven

**Tests demonstrate:**
1. ✅ Z3 can compute f(5) = 26 for f(x) = x² + 1
2. ✅ Functions using other functions work (g = 2 * f(5) = 52)
3. ✅ Sequential composition works (f then g)
4. ✅ Pythagorean theorem (a² + b² = c²) computes correctly
5. ✅ Same function works for BOTH compute AND prove

**Conclusion:** "Functions as axioms" is the RIGHT approach! ✅

---

## 🎯 Implementation Requirements

### Two Systems Need Updates

**1. Z3 Axiom Verifier** - For proving with functions
- Load `StructureMember::FunctionDef` as axioms
- Translate `define f(x) = body` to `∀x. f(x) = body`

**2. Evaluator** - For expanding function calls
- Load functions from structures (currently only loads top-level)
- Make structure functions available for symbolic substitution

---

## 📝 Implementation Steps

### Step 1: Update Axiom Verifier (Z3 Integration)

**File:** `src/axiom_verifier.rs` (~line 311)

**Current code:**
```rust
fn load_axioms_recursive(&mut self, members: &[StructureMember]) -> Result<(), String> {
    for member in members {
        match member {
            StructureMember::Axiom { proposition, .. } => {
                // Load axiom ✅
            }
            StructureMember::NestedStructure { members, .. } => {
                // Recurse ✅
            }
            _ => {
                // Ignore Operation, Field
                // ❌ FunctionDef ignored!
            }
        }
    }
}
```

**New code:**
```rust
fn load_axioms_recursive(&mut self, members: &[StructureMember]) -> Result<(), String> {
    for member in members {
        match member {
            StructureMember::Axiom { proposition, .. } => {
                // Load axiom ✅
                let z3_axiom = self.kleis_to_z3_dynamic(proposition, &HashMap::new())?;
                self.solver.assert(&z3_axiom.as_bool()?);
            }
            
            StructureMember::FunctionDef(func_def) => {
                // NEW: Load function definition as axiom
                self.load_function_as_z3_axiom(func_def)?;
            }
            
            StructureMember::NestedStructure { members, .. } => {
                // Recurse ✅
                self.load_axioms_recursive(members)?;
            }
            
            _ => {
                // Operation or Field - not an axiom
            }
        }
    }
    Ok(())
}
```

**Add new method:**
```rust
#[cfg(feature = "axiom-verification")]
fn load_function_as_z3_axiom(&mut self, func_def: &FunctionDef) -> Result<(), String> {
    println!("   📐 Loading function as Z3 axiom: {}", func_def.name);
    
    // 1. Create fresh Z3 variables for parameters
    let mut z3_vars = HashMap::new();
    let mut param_ints = Vec::new();
    
    for param in &func_def.params {
        let z3_var = Int::fresh_const(param);
        param_ints.push(z3_var.clone());
        z3_vars.insert(param.clone(), z3_var.into());
    }
    
    // 2. Translate function body to Z3
    let body_z3 = self.kleis_to_z3_dynamic(&func_def.body, &z3_vars)?;
    let body_int = body_z3.as_int()
        .ok_or_else(|| format!("Function {} body must be Int", func_def.name))?;
    
    // 3. Declare function in Z3
    let func_decl = self.declare_operation(&func_def.name, func_def.params.len());
    
    // 4. Create function application: f(params)
    let ast_args: Vec<&dyn z3::ast::Ast> = param_ints.iter()
        .map(|p| p as &dyn z3::ast::Ast)
        .collect();
    let func_app = func_decl.apply(&ast_args);
    
    // 5. Assert: f(params) = body
    // Z3 treats free variables as universally quantified
    let definition = func_app.eq(&body_int);
    self.solver.assert(&definition);
    
    println!("   ✅ Function {} loaded into Z3", func_def.name);
    Ok(())
}
```

**Estimated:** 50 lines, 30 minutes

---

### Step 2: Update Type Context (Register Functions)

**File:** `src/type_context.rs` (~line 265)

**Current code:**
```rust
fn register_operations_recursive(&mut self, structure_name: &str, members: &[StructureMember]) {
    for member in members {
        match member {
            StructureMember::Operation { name, .. } => {
                self.registry.register_operation(structure_name, name);
            }
            StructureMember::NestedStructure { members, .. } => {
                self.register_operations_recursive(structure_name, members);
            }
            _ => {
                // Field or Axiom
                // ❌ FunctionDef not registered!
            }
        }
    }
}
```

**New code:**
```rust
fn register_operations_recursive(&mut self, structure_name: &str, members: &[StructureMember]) {
    for member in members {
        match member {
            StructureMember::Operation { name, .. } => {
                self.registry.register_operation(structure_name, name);
            }
            
            StructureMember::FunctionDef(func_def) => {
                // NEW: Register function as available operation
                self.registry.register_operation(structure_name, &func_def.name);
            }
            
            StructureMember::NestedStructure { members, .. } => {
                self.register_operations_recursive(structure_name, members);
            }
            
            _ => {
                // Field or Axiom
            }
        }
    }
}
```

**Estimated:** 5 lines, 5 minutes

---

### Step 3: Update Evaluator (Load Structure Functions)

**File:** `src/evaluator.rs` (add new method)

**New method:**
```rust
/// Load function definitions from structure members
pub fn load_structure_functions(&mut self, structure: &StructureDef) -> Result<(), String> {
    self.load_structure_functions_recursive(&structure.members)
}

fn load_structure_functions_recursive(&mut self, members: &[StructureMember]) -> Result<(), String> {
    use crate::kleis_ast::StructureMember;
    
    for member in members {
        match member {
            StructureMember::FunctionDef(func_def) => {
                // Load function for symbolic expansion
                self.load_function_def(func_def)?;
            }
            
            StructureMember::NestedStructure { members, .. } => {
                // Recurse into nested structures
                self.load_structure_functions_recursive(members)?;
            }
            
            _ => {
                // Operation, Field, Axiom - not functions
            }
        }
    }
    Ok(())
}
```

**Integration point (in TypeChecker or StructureRegistry):**
```rust
// When loading structures, also load their functions
for item in &program.items {
    if let TopLevel::StructureDef(structure) = item {
        evaluator.load_structure_functions(structure)?;
    }
}
```

**Estimated:** 30 lines, 20 minutes

---

### Step 4: Add Tests

**File:** `tests/grammar_v06_function_z3_integration_test.rs` (new)

**Test cases:**
```rust
#[test]
fn test_ring_subtraction_z3_proof() {
    // Load Ring structure with define (-)(x, y) = x + negate(y)
    // Verify axiom: ∀a. (a - a) = zero
}

#[test]
fn test_field_division_z3_proof() {
    // Load Field structure with define (/)(x, y) = x * inverse(y)
    // Verify axiom: ∀a b. b ≠ 0 ⇒ (a / b) * b = a
}

#[test]
fn test_structure_function_evaluation() {
    // Load structure with function
    // Use evaluator to expand function call
}
```

**Estimated:** 100 lines, 30 minutes

---

## 📋 Complete Implementation Checklist

**Phase 1: Core Integration (~1 hour)**
- [ ] Add `StructureMember::FunctionDef` case to `axiom_verifier.rs::load_axioms_recursive()`
- [ ] Implement `load_function_as_z3_axiom()` method in axiom_verifier
- [ ] Add `StructureMember::FunctionDef` case to `type_context.rs::register_operations_recursive()`
- [ ] Test with Ring subtraction example

**Phase 2: Evaluator Integration (~30 minutes)**
- [ ] Add `load_structure_functions()` to evaluator.rs
- [ ] Add `load_structure_functions_recursive()` helper
- [ ] Integrate with structure loading pipeline
- [ ] Test symbolic expansion

**Phase 3: Testing & Validation (~1 hour)**
- [ ] Create `grammar_v06_function_z3_integration_test.rs`
- [ ] Test Ring subtraction proof
- [ ] Test Field division proof
- [ ] Test evaluator expansion
- [ ] Run all tests (600+)
- [ ] Run quality gates

**Phase 4: Documentation (~30 minutes)**
- [ ] Update TODO #57 status
- [ ] Document the integration approach
- [ ] Add examples to grammar docs
- [ ] Update session notes

**Total Estimated Time:** 3 hours

---

## 🎯 Expected Outcome

**After implementation:**

```kleis
structure Ring(R) {
  operation (+) : R × R → R
  operation negate : R → R
  
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)  // ← Will work with Z3!
  
  axiom subtraction_identity: ∀(a : R). (a - a) = zero
  // ✅ Z3 can prove this using the definition!
}
```

**What will work:**
- ✅ Z3 knows about `(-)` definition
- ✅ Can prove axioms using derived operations
- ✅ Can compute concrete values: `7 - 3 = 4`
- ✅ Evaluator can expand: `a - b` → `a + negate(b)`

---

## 🚀 Ready to Implement

**All prerequisites met:**
- ✅ Grammar v0.6 complete and pushed
- ✅ Tests prove the approach works
- ✅ Clear implementation path
- ✅ Estimated at 3 hours total work

**Shall I proceed with the implementation?**

