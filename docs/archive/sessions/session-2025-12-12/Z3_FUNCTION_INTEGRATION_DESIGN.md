# Z3 Integration for Function Definitions (TODO #57)

**Date:** December 12, 2025  
**Issue:** Z3 doesn't know about `define` statements in structures  
**Example:** `define (-)(x, y) = x + negate(y)` in Ring structure

---

## 🔍 Current State

### What Works ✅

**Top-level functions:**
```kleis
define double(x) = x + x
```
✅ Loaded by `TypeChecker.load_function_definitions()`  
✅ Available for type inference  
❌ But NOT available to Z3

**Uninterpreted functions (operations):**
```kleis
structure Semigroup(S) {
  operation (•) : S × S → S
}
```
✅ Declared in Z3 as uninterpreted function  
✅ Z3 reasons about using only axioms  
✅ Works perfectly for abstract operations

### What Doesn't Work ❌

**Functions inside structures (Grammar v0.6):**
```kleis
structure Ring(R) {
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
}

axiom test: ∀(a b : R). (a - b) + b = a
```
❌ Parser sees `define` but doesn't register it  
❌ Z3 treats `(-)` as uninterpreted (no definition!)  
❌ Can't prove axioms that use derived operations

---

## 🎯 Three Approaches to Translate Functions to Z3

### Option 1: Function Definitions as Axioms ⭐ **RECOMMENDED**

**Concept:** Translate `define f(x, y) = expr` to `∀(x y). f(x, y) = expr` as Z3 assertion

**Example:**
```kleis
// Kleis code:
structure Ring(R) {
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
}

// Translates to Z3:
1. Declare uninterpreted function: (-) : R × R → R
2. Assert axiom: ∀(x y : R). (x - y) = x + negate(y)
```

**Implementation:**
```rust
// In axiom_verifier.rs
fn load_function_definition(&mut self, func_def: &FunctionDef) -> Result<(), String> {
    // 1. Declare the function as uninterpreted
    let func_decl = self.declare_operation(&func_def.name, func_def.params.len());
    
    // 2. Create fresh Z3 variables for parameters
    let mut vars = HashMap::new();
    for (i, param) in func_def.params.iter().enumerate() {
        let z3_var = Int::fresh_const(&format!("{}_{}", param, i));
        vars.insert(param.clone(), z3_var.into());
    }
    
    // 3. Translate body to Z3
    let body_z3 = self.kleis_to_z3_dynamic(&func_def.body, &vars)?;
    
    // 4. Create function application: f(x, y)
    let param_z3s: Vec<&dyn Ast> = vars.values()
        .map(|d| d as &dyn Ast)
        .collect();
    let func_app = func_decl.apply(&param_z3s);
    
    // 5. Assert: ∀ params. f(params) = body
    let definition = func_app._eq(&body_z3);
    self.solver.assert(&definition);
    
    Ok(())
}
```

**Pros:**
- ✅ Simple and direct translation
- ✅ Z3 treats it like any other axiom
- ✅ Works with existing infrastructure
- ✅ Handles nested function calls naturally
- ✅ Compatible with uninterpreted functions

**Cons:**
- ⚠️ Adds an axiom per function (increases proof complexity)
- ⚠️ Universal quantifiers might slow Z3 on complex functions

---

### Option 2: Substitution/Macro Expansion

**Concept:** Replace `f(x, y)` calls with the function body inline during translation

**Example:**
```kleis
define (-)(x, y) = x + negate(y)

// When we see: a - b
// Expand to: a + negate(b)
// Before translating to Z3
```

**Implementation:**
```rust
// Store function definitions in a registry
struct FunctionRegistry {
    functions: HashMap<String, (Vec<String>, Expression)>
}

// During translation:
fn kleis_to_z3_dynamic(&mut self, expr: &Expression, vars: &HashMap) -> Result<Dynamic, String> {
    match expr {
        Expression::Operation { name, args } => {
            // Check if this is a defined function
            if let Some((params, body)) = self.function_registry.get(name) {
                // Substitute parameters with arguments in body
                let substituted = substitute(body, params, args);
                // Translate the expanded body
                return self.kleis_to_z3_dynamic(&substituted, vars);
            }
            
            // Otherwise treat as operation...
        }
    }
}
```

**Pros:**
- ✅ No additional axioms (no proof complexity increase)
- ✅ Direct semantics (what you see is what you get)
- ✅ Fast for simple functions
- ✅ No Z3 quantifiers needed

**Cons:**
- ❌ Code duplication (each call site expands fully)
- ❌ Exponential blowup with nested function calls
- ❌ Harder to debug (expanded expressions are large)
- ❌ Requires substitution engine

---

### Option 3: Z3 RecFuncDecl (Recursive Functions)

**Concept:** Use Z3's built-in recursive function declarations

**Example:**
```rust
// For: define factorial(n) = if n <= 1 then 1 else n * factorial(n-1)
let fact = RecFuncDecl::new(&ctx, "fact", &[&int_sort], &int_sort);
let n = Int::new_const(&ctx, "n");
let body = /* ... recursive definition ... */;
fact.add_def(&[&n], &body);
```

**Pros:**
- ✅ Native Z3 support
- ✅ Handles recursion correctly
- ✅ Optimized by Z3 internally

**Cons:**
- ❌ Only for recursive functions
- ❌ More complex API
- ❌ Not needed for simple derived operations

---

## 🎯 Recommended Solution: Option 1 (Functions as Axioms)

**Why Option 1 is best for Kleis:**

### 1. Mathematical Correctness
Function definitions ARE axioms in algebraic structures:
```
define (-)(x, y) = x + negate(y)
≡
axiom subtraction_def: ∀(x y : R). (x - y) = x + negate(y)
```

This is exactly how mathematicians think about derived operations!

### 2. Consistency with Existing Design
We already translate axioms to Z3 assertions:
```rust
StructureMember::Axiom { proposition, .. } => {
    let z3_axiom = self.kleis_to_z3_dynamic(proposition, &HashMap::new())?;
    self.solver.assert(&z3_axiom.as_bool().unwrap());
}
```

Functions are just definitional axioms!

### 3. Handles Edge Cases
- ✅ Nested function calls work (each is an axiom)
- ✅ Mutual recursion works (both are axioms)
- ✅ Compatible with uninterpreted operations
- ✅ Can be overridden in `implements` blocks

### 4. Minimal Implementation
Just extend the existing pattern-matching in two places:
1. `axiom_verifier.rs:load_axioms_recursive()` - Handle `FunctionDef`
2. `type_context.rs:register_operations_recursive()` - Register function names

---

## 📝 Implementation Plan

### Step 1: Update Axiom Verifier

**File:** `src/axiom_verifier.rs` (line ~311)

**Current:**
```rust
match member {
    StructureMember::Axiom { proposition, .. } => { /* load */ }
    StructureMember::NestedStructure { members, .. } => { /* recurse */ }
    _ => { /* ignore */ }
}
```

**New:**
```rust
match member {
    StructureMember::Axiom { proposition, .. } => { 
        // Load axiom into Z3
        let z3_axiom = self.kleis_to_z3_dynamic(proposition, &HashMap::new())?;
        self.solver.assert(&z3_axiom.as_bool()?);
    }
    
    StructureMember::FunctionDef(func_def) => {
        // NEW: Translate function definition as axiom
        // define f(x, y) = body
        // becomes: ∀(x y). f(x, y) = body
        self.load_function_as_axiom(func_def)?;
    }
    
    StructureMember::NestedStructure { members, .. } => {
        self.load_axioms_recursive(members)?;
    }
    
    _ => { /* Operation or Field */ }
}
```

### Step 2: Implement Function-to-Axiom Translator

**New method in axiom_verifier.rs:**
```rust
#[cfg(feature = "axiom-verification")]
fn load_function_as_axiom(&mut self, func_def: &FunctionDef) -> Result<(), String> {
    println!("   📐 Loading function definition as axiom: {}", func_def.name);
    
    // 1. Declare function as uninterpreted in Z3
    self.declare_operation(&func_def.name, func_def.params.len());
    
    // 2. Create fresh Z3 variables for parameters
    let mut z3_vars = HashMap::new();
    let mut z3_param_asts = Vec::new();
    
    for param in &func_def.params {
        let z3_var = Int::fresh_const(param);
        z3_param_asts.push(z3_var.clone());
        z3_vars.insert(param.clone(), z3_var.into());
    }
    
    // 3. Translate function body to Z3
    let body_z3 = self.kleis_to_z3_dynamic(&func_def.body, &z3_vars)?;
    
    // 4. Create function application: f(x, y)
    let func_decl = self.declare_operation(&func_def.name, func_def.params.len());
    let param_refs: Vec<&dyn Ast> = z3_param_asts.iter()
        .map(|p| p as &dyn Ast)
        .collect();
    let func_app = func_decl.apply(&param_refs);
    
    // 5. Assert: f(x, y) = body
    // This is implicitly universally quantified by Z3
    let definition_axiom = func_app._eq(&body_z3);
    self.solver.assert(&definition_axiom.as_bool()?);
    
    println!("   ✅ Function {} registered in Z3", func_def.name);
    Ok(())
}
```

### Step 3: Update Type Context Registration

**File:** `src/type_context.rs` (line ~265)

**Current:**
```rust
_ => {
    // Field or Axiom - no operation to register
}
```

**New:**
```rust
StructureMember::FunctionDef(func_def) => {
    // Register function name as available operation
    self.registry.register_operation(structure_name, &func_def.name);
}
_ => {
    // Field or Axiom
}
```

### Step 4: Test It!

**New test:**
```rust
#[test]
fn test_z3_derived_operation_proof() {
    let code = r#"
    structure Ring(R) {
      operation (+) : R × R → R
      operation negate : R → R
      element zero : R
      
      // Derived operation
      operation (-) : R × R → R
      define (-)(x, y) = x + negate(y)
      
      // Test axiom using derived operation
      axiom subtraction_inverse:
        ∀(x : R). (x - x) = zero
    }
    "#;
    
    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().unwrap();
    
    let registry = StructureRegistry::new();
    // ... register Ring ...
    
    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    let result = verifier.verify_axiom(&subtraction_inverse_expr).unwrap();
    
    assert_eq!(result, VerificationResult::Valid);
    // ✅ Z3 should prove this using the definition of (-)!
}
```

---

## 📊 Comparison of Approaches

| Aspect | Option 1: Axioms | Option 2: Substitution | Option 3: RecFuncDecl |
|--------|------------------|------------------------|----------------------|
| **Correctness** | ✅ Mathematically sound | ✅ Correct | ✅ Correct |
| **Z3 Proof complexity** | ⚠️ Adds axioms | ✅ No extra axioms | ⚠️ Complex |
| **Implementation** | ✅ Simple (~50 lines) | ⚠️ Need substitution engine | ❌ Complex API |
| **Debugging** | ✅ Easy (axioms visible) | ⚠️ Expanded expressions large | ⚠️ Black box |
| **Recursion** | ⚠️ May not terminate | ❌ Stack overflow | ✅ Native support |
| **Performance** | ✅ Good for simple functions | ⚠️ Exponential blowup | ✅ Optimized |
| **Fits Kleis design** | ✅✅ Perfect match | ⚠️ Different semantics | ❌ Overkill |

---

## 🎯 Recommendation: Option 1 (Functions as Axioms)

### Why This is The Right Choice

**1. Mathematical Correctness:**
In algebra, derived operations ARE definitional axioms:
```
Subtraction in rings: x - y ≡ x + (-y)    [definition]
Division in fields: x / y ≡ x × y⁻¹       [definition]
```

These are literally called "definitional axioms" in mathematics!

**2. Consistency with Kleis Design:**
```kleis
structure Ring(R) {
  // Primitive operations
  operation (+) : R × R → R
  operation negate : R → R
  
  // Derived operation (definitional axiom!)
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
  
  // Regular axiom
  axiom associativity: ∀(x y z). (x + y) + z = x + (y + z)
}
```

Both `define` and `axiom` become Z3 assertions - symmetric and clean!

**3. Simple Implementation:**
- Extend existing `load_axioms_recursive()` with 1 new case
- Reuse all existing infrastructure
- ~50 lines of code

**4. Handles Edge Cases:**
```kleis
// Nested function calls work:
define square(x) = x * x
define fourth(x) = square(square(x))

// Z3 gets:
// ∀x. square(x) = x * x
// ∀x. fourth(x) = square(square(x))
// Can prove: fourth(2) = 16 ✅
```

---

## 🔧 Implementation Details

### Z3 Translation Strategy

**Kleis:**
```kleis
define (-)(x, y) = x + negate(y)
```

**Z3 SMT-LIB equivalent:**
```smt
; Declare uninterpreted function
(declare-fun minus (Int Int) Int)

; Assert definition as axiom
(assert (forall ((x Int) (y Int))
  (= (minus x y) (plus x (negate y)))))
```

**Z3 Rust API:**
```rust
// 1. Declare function
let minus = FuncDecl::new("minus", &[&Sort::int(), &Sort::int()], &Sort::int());

// 2. Create quantified variables
let x = Int::fresh_const("x");
let y = Int::fresh_const("y");

// 3. Build: minus(x, y) = plus(x, negate(y))
let lhs = minus.apply(&[&x, &y]);
let rhs = /* translate: x + negate(y) */;
let definition = lhs._eq(&rhs);

// 4. Assert (implicitly universally quantified in Z3)
solver.assert(&definition);
```

### Handling in Proof Context

**When proving:**
```kleis
axiom test: ∀(a b : R). (a - b) + b = a
```

**Z3 automatically uses the definition:**
```
Given: ∀(x y). (x - y) = x + negate(y)   [from define]
Given: ∀(x). x + negate(x) = zero        [from Ring axioms]
Given: ∀(x). x + zero = x                [from Ring axioms]

Prove: ∀(a b). (a - b) + b = a

Steps:
1. (a - b) + b = (a + negate(b)) + b     [by definition of (-)]
2.             = a + (negate(b) + b)      [by associativity]
3.             = a + zero                 [by inverse axiom]
4.             = a                        [by identity axiom]
✅ QED
```

---

## 🚨 Potential Issues and Solutions

### Issue 1: Circular Definitions

**Problem:**
```kleis
define f(x) = g(x)
define g(x) = f(x)
```

**Solution:**
- Dependency analysis before loading
- Reject circular definitions at parse/check time
- OR: Let Z3 handle it (may time out)

### Issue 2: Non-terminating Recursion

**Problem:**
```kleis
define loop(x) = loop(x + 1)
```

**Solution:**
- Current approach: Z3 treats as uninterpreted with axiom
- Z3 may not terminate if it tries to expand infinitely
- Detect direct recursion and warn or use RecFuncDecl

### Issue 3: Multiple Parameters with Different Types

**Problem:**
```kleis
define scale(s : ℝ, v : Vector(n)) = s · v
```

**Solution:**
- Use Z3 polymorphic sorts (Real, Int, Array)
- OR: Keep using Int sort (current approach - works!)
- Types are abstracted in uninterpreted functions

---

## 📋 Implementation Checklist

**Phase 1: Basic Integration (~1 hour)**
- [ ] Add `StructureMember::FunctionDef` case to `load_axioms_recursive()`
- [ ] Implement `load_function_as_axiom()` method
- [ ] Register function names in type context
- [ ] Add test for simple derived operation

**Phase 2: Robustness (~2 hours)**
- [ ] Handle nested function calls
- [ ] Detect circular definitions
- [ ] Add comprehensive tests
- [ ] Document in ADR or design doc

**Phase 3: Advanced (future)**
- [ ] Recursive functions using RecFuncDecl
- [ ] Polymorphic function types
- [ ] Function inlining optimization

---

## 🎯 Estimated Effort

**Minimal viable:** 50 lines of code + 1 test = ~30 minutes  
**Production ready:** 150 lines + 5 tests + docs = ~2 hours  
**Complete feature:** With recursion support = ~4 hours

---

## ✅ Recommendation

**Start with Option 1 (Functions as Axioms):**

1. Simple to implement
2. Mathematically correct
3. Consistent with Kleis design
4. Handles 90% of use cases
5. Can add Option 3 (RecFuncDecl) later for recursion if needed

**Next step:** Implement the basic integration in `axiom_verifier.rs` and test with Ring subtraction.

---

Would you like me to implement this now?

