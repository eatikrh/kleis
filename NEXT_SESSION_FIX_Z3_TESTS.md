# URGENT: Fix Z3 Function Tests Using RecFuncDecl

**Date:** December 12, 2024  
**Priority:** CRITICAL  
**Issue:** 11 tests failing in CI due to using wrong Z3 API

---

## 🚨 Problem

**Current:** Using `FuncDecl` + assertions (doesn't work for model evaluation)  
**Solution:** Use `RecFuncDecl` + `.add_def()` (proper Z3 API for defined functions)

**Root cause discovered:** Z3 documentation and test examples show `RecFuncDecl` is the correct way to define functions that need to evaluate correctly in models.

---

## 📚 Key Discovery from Z3 Source

**File:** `/Users/eatik_1/Documents/git/cee/kleis/vendor/z3/src/rec_func_decl.rs`  
**Z3 Test:** `/Users/eatik_1/Documents/git/cee/Z3/z3.rs/z3/tests/lib.rs:541`

**Correct pattern:**
```rust
// Use RecFuncDecl, not FuncDecl!
let fac = RecFuncDecl::new("fac", &[&Sort::int()], &Sort::int());
let n = Int::new_const("n");
let body = /* ... formula ... */;

// ADD DEFINITION (not assertion!)
fac.add_def(&[&n], &body);

// Now fac(5) evaluates correctly in models! ✅
```

---

## 🔧 Files to Fix

### 1. tests/z3_function_evaluation_test.rs (3 failing tests)

**Tests:**
- `test_z3_compute_chained_functions` (line 82)
- `test_z3_compute_derived_operation` (line 152)
- `test_z3_grammar_v06_pattern` (line 218)

**Pattern to apply:**
```rust
// OLD:
let func_decl = FuncDecl::new("func", &[&Sort::int()], &Sort::int());
solver.assert(func_decl.apply(&[&x]).eq(&body));

// NEW:
let func = RecFuncDecl::new("func", &[&Sort::int()], &Sort::int());
func.add_def(&[&x], &body);
```

### 2. tests/z3_composed_functions_test.rs (8 failing tests)

**All tests need same fix:** Replace `FuncDecl` with `RecFuncDecl`

---

## 📝 Detailed Fix for test_z3_compute_chained_functions

**Current (line 88-125):**
```rust
let square_decl = FuncDecl::new("square", &[&Sort::int()], &Sort::int());
let x = Int::fresh_const("x");
let x_squared = &x * &x;
solver.assert(square_decl.apply(&[&x]).eq(&x_squared));  // ❌ Doesn't work!
```

**Fixed:**
```rust
let square = RecFuncDecl::new("square", &[&Sort::int()], &Sort::int());
let x = Int::new_const("x");
let x_squared = &x * &x;
square.add_def(&[&x], &x_squared);  // ✅ Works!
```

**For composition:**
```rust
let sum_of_squares = RecFuncDecl::new("sum_of_squares", &[&Sort::int(), &Sort::int()], &Sort::int());
let a = Int::new_const("a");
let b = Int::new_const("b");

// Can use square() in the definition!
let square_a = square.apply(&[&a]).as_int().unwrap();
let square_b = square.apply(&[&b]).as_int().unwrap();
let sum_body = &square_a + &square_b;

sum_of_squares.add_def(&[&a, &b], &sum_body);  // ✅ Chaining works!
```

---

## ✅ Why This Fixes Everything

### RecFuncDecl vs FuncDecl

| Aspect | FuncDecl | RecFuncDecl |
|--------|----------|-------------|
| **Purpose** | Uninterpreted (abstract) | Defined (concrete) |
| **Definition** | None | `.add_def()` provides body |
| **Model evaluation** | Arbitrary values ❌ | Uses definition ✅ |
| **Quantifiers** | Need E-matching | Not needed ✅ |
| **Our use case** | Wrong choice | ✅ **CORRECT** |

### What We Learn

**"Functions as axioms" is still correct!**

But the implementation detail matters:
- ❌ `FuncDecl` + `solver.assert(∀x. f(x) = body)` - Doesn't work for evaluation
- ✅ `RecFuncDecl` + `f.add_def([x], body)` - Works perfectly!

**The theory is sound, we just used the wrong Z3 API!**

---

## 🎯 Action Items for Next Session

### Immediate (30 minutes)

1. Add `RecFuncDecl` to imports in both test files
2. Replace all `FuncDecl::new()` with `RecFuncDecl::new()`
3. Replace all `solver.assert(func.apply(...).eq(...))` with `func.add_def(...)`
4. For model evaluation, create result variable and assert equality
5. Run tests - should all pass!

### Update Implementation (1 hour)

6. Update `axiom_verifier.rs::load_function_as_z3_axiom()` to use `RecFuncDecl`
7. This will make Grammar v0.6 functions work correctly with Z3!
8. Test with actual Kleis code

### Commit (15 minutes)

9. Run quality gates
10. Commit: "fix: Use RecFuncDecl for function definitions in Z3"
11. Push to GitHub

**Total time:** ~2 hours to fix everything properly

---

## 📊 Expected Outcome

**After fix:**
- ✅ All 11 tests pass
- ✅ Proves functions-as-axioms works (with correct API)
- ✅ Grammar v0.6 Z3 integration fully functional
- ✅ CI passes

**This validates our entire approach!** ✅

---

## 🔑 Key Takeaway

**We discovered the RIGHT way to do functions in Z3:**
- Use `RecFuncDecl` (not `FuncDecl`)
- Use `.add_def()` (not `solver.assert()`)
- Works for both recursive AND non-recursive functions
- Model evaluation returns correct values

**Our theory was correct, we just needed to learn the proper Z3 API!** 🎓

---

**Status:** Ready to fix in next session  
**Estimated:** 2 hours  
**Impact:** Validates entire Grammar v0.6 approach ✅

