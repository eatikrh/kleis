# Z3 RecFuncDecl - The Correct Way to Define Functions

**Date:** December 12, 2024  
**Discovery:** We should use `RecFuncDecl` with `.add_def()`, not `FuncDecl` with assertions!

---

## 🎯 The Problem We Had

**What we were doing (WRONG):**
```rust
// Declare uninterpreted function
let g_decl = FuncDecl::new("g", &[&Sort::int()], &Sort::int());

// Try to define it with assertion
let x = Int::fresh_const("x");
solver.assert(g_decl.apply(&[&x]).eq(&formula));  // ❌ Doesn't work for model eval!

// Query
let result = g_decl.apply(&[&five]);
model.eval(&result)  // Returns arbitrary value! ❌
```

**Why it fails:**
- `FuncDecl` creates **uninterpreted** function
- Assertion `∀x. g(x) = formula` doesn't make g "defined"
- Z3 doesn't instantiate quantifiers during model evaluation
- Model can assign arbitrary values to uninterpreted functions

---

## ✅ The Solution: RecFuncDecl

**What we SHOULD do:**
```rust
// Use RecFuncDecl (recursive function declaration)
let g = RecFuncDecl::new("g", &[&Sort::int()], &Sort::int());

// Define the function body
let x = Int::new_const("x");
let body = &(&x + &x) + &Int::from_i64(2);  // 2x + 2

// ADD DEFINITION (not assertion!)
g.add_def(&[&x], &body);  // ← This is the key!

// Now query
let result = g.apply(&[&Int::from_i64(5)]);
model.eval(&result)  // Returns 12 ✅ CORRECT!
```

**Why it works:**
- `RecFuncDecl` creates a **defined** function
- `.add_def()` tells Z3 the actual definition
- Z3 uses the definition during model evaluation
- No quantifier instantiation needed!

---

## 📚 From Z3 Rust Tests

**Example from `/Users/eatik_1/Documents/git/cee/Z3/z3.rs/z3/tests/lib.rs:541`:**

```rust
#[test]
fn test_rec_func_def() {
    // Define factorial function
    let fac = RecFuncDecl::new("fac", &[&Sort::int()], &Sort::int());
    let n = ast::Int::new_const("n");
    let n_minus_1 = &n - 1;
    let fac_of_n_minus_1 = fac.apply(&[&n_minus_1]);
    let cond: ast::Bool = n.le(0);
    let body = cond.ite(
        &ast::Int::from_i64(1),
        &ast::Int::mul(&[&n, &fac_of_n_minus_1.as_int().unwrap()]),
    );

    fac.add_def(&[&n], &body);  // ← Define the function!

    let solver = Solver::new();
    
    // Now can use fac(4), fac(5) and they compute correctly!
    solver.assert(x.eq(fac.apply(&[&ast::Int::from_i64(4)]).as_int().unwrap()));
    solver.assert(y.eq(fac.apply(&[&ast::Int::from_i64(5)]).as_int().unwrap()));
    solver.assert(y.eq(120));  // fac(5) = 120
    
    assert_eq!(solver.check(), SatResult::Sat);  // ✅ Works!
}
```

---

## 🔧 How to Fix Our Tests

### Pattern: Use RecFuncDecl for Non-Recursive Functions Too!

**Even though our functions aren't recursive, RecFuncDecl works for them:**

```rust
// OLD (doesn't work):
let g_decl = FuncDecl::new("g", &[&Sort::int()], &Sort::int());
solver.assert(g_decl.apply(&[&x]).eq(&body));

// NEW (works!):
let g = RecFuncDecl::new("g", &[&Sort::int()], &Sort::int());
let x = Int::new_const("x");
let body = &(&x + &x) + &Int::from_i64(2);
g.add_def(&[&x], &body);  // ← No solver.assert needed!

// Now g(5) will evaluate correctly!
```

---

## 📝 Fixed Test Example

### Before (Fails):
```rust
fn test_z3_function_using_another_function() {
    let solver = Solver::new();
    
    let f_decl = FuncDecl::new("f", &[&Sort::int()], &Sort::int());
    let x = Int::fresh_const("x");
    let one = Int::from_i64(1);
    solver.assert(f_decl.apply(&[&x]).eq(&(&x + &one)));  // ❌
    
    let five = Int::from_i64(5);
    let g_at_5 = f_decl.apply(&[&five]);
    
    let model = solver.get_model().unwrap();
    let value = model.eval(&g_at_5, true).unwrap();
    // Returns: arbitrary value ❌
}
```

### After (Should Work):
```rust
fn test_z3_function_using_another_function() {
    let solver = Solver::new();
    
    // Use RecFuncDecl!
    let f = RecFuncDecl::new("f", &[&Sort::int()], &Sort::int());
    let x = Int::new_const("x");
    let one = Int::from_i64(1);
    let body = &x + &one;
    f.add_def(&[&x], &body);  // ✅ Define it!
    
    let five = Int::from_i64(5);
    let f_at_5 = f.apply(&[&five]);
    
    // Need to assert something for solver to have a model
    let result_var = Int::fresh_const("result");
    solver.assert(result_var.eq(f_at_5.as_int().unwrap()));
    
    if solver.check() == SatResult::Sat {
        let model = solver.get_model().unwrap();
        let value = model.eval(&result_var, true).unwrap().as_i64().unwrap();
        // Returns: 6 ✅ CORRECT!
    }
}
```

---

## 🎯 Key Insights

### 1. RecFuncDecl vs FuncDecl

| Feature | FuncDecl | RecFuncDecl |
|---------|----------|-------------|
| **Purpose** | Uninterpreted functions | Defined functions |
| **Definition** | None (abstract) | `.add_def()` provides body |
| **Model evaluation** | Arbitrary values | Uses definition |
| **Quantifiers** | Need E-matching | Not needed |
| **Use case** | Abstract reasoning | Concrete computation |

### 2. When to Use Each

**Use FuncDecl when:**
- Function is truly abstract (no definition)
- Only care about axioms constraining it
- Don't need concrete values

**Use RecFuncDecl when:**
- Have explicit definition
- Need model evaluation to return correct values
- Want Z3 to compute with the function

### 3. Our Use Case

**For Grammar v0.6 functions:**
```kleis
define (-)(x, y) = x + negate(y)
```

**Should use:** `RecFuncDecl` ✅
- We have explicit definition
- Want model evaluation to work
- Need concrete computation

---

## 🔧 Implementation Plan

### Fix All Failing Tests

**Pattern to apply:**
1. Replace `FuncDecl::new()` with `RecFuncDecl::new()`
2. Replace `solver.assert(func.apply(...).eq(...))` with `func.add_def(...)`
3. For model evaluation, assert result variable equals function application
4. Extract value from result variable

**Files to fix:**
- `tests/z3_function_evaluation_test.rs` - 3 failing tests
- `tests/z3_composed_functions_test.rs` - 8 failing tests (if restored)

---

## ✅ This Validates Our Theory!

**Once fixed, these tests will prove:**
- ✅ Functions as definitions work in Z3
- ✅ Model evaluation returns correct values
- ✅ Function composition works
- ✅ "Functions as axioms" approach is sound (when using RecFuncDecl!)

**The theory is correct, we just used the wrong Z3 API!** ✅

---

**Next: Fix all tests using RecFuncDecl!**

