# Next Session: Quick Wins (1.5 hours total)

**Date:** December 9, 2024 (Evening) or Next Session  
**Status:** Ready to start immediately  
**Goal:** Two easy, high-value features

---

## 🎯 Quick Win #1: Uncomment Remaining Stdlib Functions (30 min)

**File:** `stdlib/types.kleis`

**Current status:** We uncommented 6 functions (not, and, or, isSome, isNone, isEmpty)

**Remaining commented functions to enable:**

### Task 1a: Option Functions (10 min)

Find and uncomment:
```kleis
// define getOrDefault : Option(T) × T → T
// define getOrDefault(opt, default) = match opt {
//   None => default
//   Some(x) => x
// }
```

### Task 1b: List Functions (10 min)

Find and uncomment:
```kleis
// define head : List(T) → Option(T)
// define head(list) = match list {
//   Nil => None
//   Cons(h, _) => Some(h)
// }

// define tail : List(T) → Option(List(T))
// define tail(list) = match list {
//   Nil => None
//   Cons(_, t) => Some(t)
// }
```

### Task 1c: Test (10 min)

```bash
# Verify stdlib loads
cargo test --lib type_checker::tests::test_with_stdlib_still_works -- --nocapture

# Quick smoke test
cargo test --lib
```

**Expected result:** All tests pass, new functions available

**Success criteria:**
- ✅ Functions uncommented
- ✅ Stdlib loads without errors
- ✅ All tests still pass

---

## 🎯 Quick Win #2: Add Math Functions Stdlib (1 hour)

**File:** Create `stdlib/math_functions.kleis`

### Task 2a: Create File (40 min)

```kleis
// Inverse Trigonometric Functions
structure InverseTrig(T) {
    operation arcsin : T → T
    operation arccos : T → T
    operation arctan : T → T
    operation arctan2 : T → T → T
}

implements InverseTrig(ℝ) {
    operation arcsin = builtin_arcsin
    operation arccos = builtin_arccos
    operation arctan = builtin_arctan
    operation arctan2 = builtin_arctan2
}

// Hyperbolic Functions
structure Hyperbolic(T) {
    operation sinh : T → T
    operation cosh : T → T
    operation tanh : T → T
    operation asinh : T → T
    operation acosh : T → T
    operation atanh : T → T
}

implements Hyperbolic(ℝ) {
    operation sinh = builtin_sinh
    operation cosh = builtin_cosh
    operation tanh = builtin_tanh
    operation asinh = builtin_asinh
    operation acosh = builtin_acosh
    operation atanh = builtin_atanh
}

// Combinatorics
structure Combinatorics {
    operation factorial : ℕ → ℕ
    operation binomial : ℕ → ℕ → ℕ
    operation permutation : ℕ → ℕ → ℕ
}

implements Combinatorics {
    operation factorial = builtin_factorial
    operation binomial = builtin_binomial
    operation permutation = builtin_permutation
}

// Special Functions
structure SpecialFunctions(T) {
    operation gamma : T → T
    operation digamma : T → T
    operation beta : T → T → T
    operation erf : T → T
    operation erfc : T → T
}

implements SpecialFunctions(ℝ) {
    operation gamma = builtin_gamma
    operation digamma = builtin_digamma
    operation beta = builtin_beta
    operation erf = builtin_erf
    operation erfc = builtin_erfc
}

// Logarithms and Exponentials (extended)
structure LogarithmExtended(T) {
    operation log10 : T → T
    operation log2 : T → T
    operation exp2 : T → T
    operation expm1 : T → T  // exp(x) - 1 (for small x)
    operation log1p : T → T  // log(1 + x) (for small x)
}

implements LogarithmExtended(ℝ) {
    operation log10 = builtin_log10
    operation log2 = builtin_log2
    operation exp2 = builtin_exp2
    operation expm1 = builtin_expm1
    operation log1p = builtin_log1p
}

// Rounding Functions (extended)
structure RoundingExtended(T) {
    operation round : T → T
    operation trunc : T → T
    operation frac : T → T  // Fractional part
}

implements RoundingExtended(ℝ) {
    operation round = builtin_round
    operation trunc = builtin_trunc
    operation frac = builtin_frac
}
```

### Task 2b: Load in TypeChecker (10 min)

**File:** `src/type_checker.rs`

In the `with_stdlib()` method, add after loading quantum:

```rust
// Load math functions
let math_fns = include_str!("../stdlib/math_functions.kleis");
checker
    .load_kleis(math_fns)
    .map_err(|e| format!("Failed to load stdlib/math_functions.kleis: {}", e))?;
```

### Task 2c: Test (10 min)

```bash
# Test parsing
cargo test --lib kleis_parser::tests

# Test stdlib loading
cargo test --lib type_checker::tests::test_with_stdlib_still_works -- --nocapture

# Full test suite
cargo test --lib
```

**Success criteria:**
- ✅ File parses correctly
- ✅ Stdlib loads without errors
- ✅ All tests still pass
- ✅ Functions available in type checker

---

## 🎯 Execution Plan

### Step-by-Step

```bash
# 1. Uncomment stdlib functions (30 min)
# Open: stdlib/types.kleis
# Find: // define getOrDefault
# Find: // define head
# Find: // define tail
# Uncomment and fix formatting
# Test: cargo test --lib

# 2. Create math functions (40 min)
# Create: stdlib/math_functions.kleis
# Copy template from above
# Add all structures and implementations

# 3. Load in type checker (10 min)  
# Edit: src/type_checker.rs
# Add load_kleis() call for math_functions
# Test: cargo test --lib

# 4. Quality checks (10 min)
cargo fmt
cargo clippy --all-targets --all-features
cargo test

# 5. Commit (5 min)
git add stdlib/
git commit -m "feat: add remaining stdlib functions

- Uncommented getOrDefault, head, tail
- Added math_functions.kleis with:
  * Inverse trig (arcsin, arccos, arctan)
  * Hyperbolic (sinh, cosh, tanh)
  * Combinatorics (factorial, binomial)
  * Special functions (gamma, erf, bessel)

All tests pass."
```

---

## ✅ Success Criteria

After both tasks:

**Functionality:**
- ✅ 9+ new functions available (6 already done + 3 uncommented)
- ✅ 30+ math operations type-checkable
- ✅ Palette coverage significantly improved

**Testing:**
- ✅ All tests pass (413+)
- ✅ Stdlib loads cleanly
- ✅ No regressions

**Code Quality:**
- ✅ cargo fmt clean
- ✅ cargo clippy no new warnings
- ✅ Functions properly documented

---

## 🎊 Expected Outcome

**After 1.5 hours:**

Users can now use:
```kleis
// Boolean logic
not(True)              // ✅ Already works
and(True, False)       // ✅ Already works

// Option handling  
getOrDefault(opt, 0)   // ✅ NEW
head([1, 2, 3])        // ✅ NEW (returns Some(1))

// Math functions
arcsin(0.5)            // ✅ NEW
sinh(x)                // ✅ NEW
factorial(5)           // ✅ NEW  
gamma(3.5)             // ✅ NEW
```

**Kleis stdlib significantly expanded with minimal effort!** 🚀

---

## 📝 Notes

### Why These Are Easy

1. **No new Rust code** - Just Kleis definitions
2. **Pattern is clear** - Copy existing structures
3. **No parser changes** - Existing syntax works
4. **Low risk** - Just adding, not modifying

### Why These Are Valuable

1. **User-requested** - People need these functions
2. **High usage** - Basic math operations
3. **Demonstrates power** - Extensibility in action
4. **Foundation** - Other features build on these

### Potential Issues

**None expected!** Both tasks are straightforward.

If you hit any issues:
- Parsing errors → Check syntax against grammar
- Loading errors → Check structure name conflicts
- Type errors → Verify implementations reference correct structures

---

## 🔜 After These Quick Wins

**Next session priorities:**
1. Physical constants palette (2-3 hours) - Today's discovery
2. Parser extension for complex implements (4-6 hours) - Architectural
3. Integration tests (2-3 hours) - Validation

**But first: Knock out these two easy wins!** 💪

---

**Ready to start?** Open `stdlib/types.kleis` and look for commented-out functions!

