# Transcendental Functions: Type System vs Backend Limitations

**Date:** December 12, 2024  
**Context:** Solver abstraction layer refactoring  
**Status:** Documented behavior (not a bug)

---

## 🎯 Summary

**Type System:** ✅ Accepts transcendental functions on matrices (mathematically correct!)  
**Z3 Backend:** ⚠️ Treats as uninterpreted functions (symbolic, not evaluated)  
**Equation Editor:** Uses polymorphic definitions (allows matrix inputs)

---

## 📐 Mathematical Background

### Matrix Exponentials Are Real!

In control theory and differential equations:

```
e^(A - sI)    // Resolvent / Transfer function
e^(At)        // State transition matrix  
sin(M)        // Via Taylor series: sin(M) = M - M³/3! + M⁵/5! - ...
cos(M)        // Via Taylor series: cos(M) = I - M²/2! + M⁴/4! - ...
```

**These are mathematically valid operations!**

Used in:
- Control theory (stability analysis)
- Differential equations (matrix ODEs)
- Quantum mechanics (time evolution: e^(-iHt))
- General relativity (exponential map on manifolds)

---

## 🏗️ Architecture: Two Definitions

### Concrete Definition (prelude.kleis)

```kleis
operation sin : ℝ → ℝ
operation cos : ℝ → ℝ
operation exp : ℝ → ℝ
```

**Purpose:** Scalar-only transcendentals  
**When used:** When type is known to be Scalar

### Polymorphic Definition (math_functions.kleis)

```kleis
structure Trigonometric(T) {
    operation sin : T → T
    operation cos : T → T
}

structure Exponential(T) {
    operation exp : T → T
}

implements Trigonometric(ℝ) { ... }
implements Exponential(ℝ) { ... }
```

**Purpose:** Generic transcendentals (works for ℝ, ℂ, Matrix, etc.)  
**When used:** Type inference finds this first (loaded after prelude)  
**Effect:** Allows `sin(Matrix)`, `exp(Matrix)`, etc.

---

## 💻 Current Behavior

### Equation Editor Example

**User Input:**
```
sin(Matrix(2, 2, [a11, a12, a21, a22]))
```

**Type Inference:**
1. Finds `Trigonometric(T)` structure
2. Matches `sin : T → T`
3. Unifies `T = Matrix(2, 2, ℝ)`
4. Returns type: `Matrix(2, 2, ℝ)`

**Result:** ✅ Type checks successfully

### What Z3 Does

**When this reaches Z3 backend:**

```rust
// In Z3Backend::translate_operation()
"sin" => {
    // No native translator for sin!
    // Falls back to uninterpreted function
    let func_decl = self.declare_uninterpreted("sin", 1);
    Ok(func_decl.apply(&[arg]))  // Returns: sin(Matrix(...)) [symbolic]
}
```

**Result:** Symbolic expression, not evaluated

**From capabilities.toml:**
```toml
# sin is NOT in the native operations list
# Z3 has NO support for transcendental functions
# Treated as uninterpreted function
```

---

## 🎯 This Is By Design!

### Type System (Kleis)

**Should accept:** `sin(Matrix)`, `exp(Matrix)`, etc.

**Why:**
- Mathematically valid operations
- Users in control theory need this
- Type system should allow valid mathematics

### Backend (Z3)

**Current status:** Treats as uninterpreted

**Why:**
- Z3 is for theorem proving, not numeric computation
- SMT solvers don't have transcendental theories
- Even `sin(Scalar)` is uninterpreted in Z3!

**What this means:**
- Can prove properties using axioms
- Can't compute numeric values
- Returns symbolic expressions

---

## 📊 Coverage Analysis

### Z3 Native Operations (15)

```
Arithmetic: plus, minus, times, negate
Comparison: equals, lt, gt, leq, geq  
Boolean: and, or, not, implies
```

### Uninterpreted (118+ operations)

```
Transcendentals: sin, cos, tan, exp, ln, sqrt, ...
Matrix ops: matrix_multiply, transpose, determinant, ...
Tensor ops: contract, wedge, riemann, ...
Quantum ops: bra, ket, commutator, ...
```

**This is correct for theorem proving!**

---

## 🚀 Future: Numeric Backends

When we add numeric computation backends:

```rust
// Future CASBackend (Symbolic computation)
impl SolverBackend for CASBackend {
    fn supports_operation(&self, op: &str) -> bool {
        match op {
            "sin" | "cos" | "exp" | "matrix_exp" => true,  // ✅ Can evaluate!
            ...
        }
    }
}
```

**With CAS backend:**
- ✅ Can compute `sin(Matrix)` using Taylor series
- ✅ Can compute `exp(A - sI)` for control theory
- ✅ Returns numeric results, not symbolic

---

## 📝 Related Files

**Type Definitions:**
- `stdlib/prelude.kleis` - Concrete transcendentals (`sin : ℝ → ℝ`)
- `stdlib/math_functions.kleis` - Polymorphic transcendentals (`sin : T → T`)

**Type Checking:**
- `src/type_checker.rs:120-123` - Loads math_functions.kleis
- `src/signature_interpreter.rs:240` - Accepts Matrix for ℝ (documented now)

**Solver Backend:**
- `src/solvers/z3/capabilities.toml` - Lists only 15 native operations
- `src/solvers/z3/translators/` - No transcendental translators
- Z3 treats all transcendentals as uninterpreted functions

---

## ✅ Conclusion

**This is NOT a bug, it's a FEATURE with LIMITATIONS:**

1. ✅ **Type system is correct** - Allows matrix transcendentals (mathematically valid)
2. ⚠️ **Z3 backend limitation** - Can't evaluate, treats as symbolic
3. 📖 **User expectation** - Need to document this limitation
4. 🚀 **Future improvement** - Add CAS backend for numeric evaluation

**No code changes needed.** Just awareness that:
- Equation editor accepts mathematically valid expressions
- Z3 provides symbolic reasoning, not numeric computation
- Future backends can add evaluation capabilities

---

**See Also:**
- [Solver Abstraction Architecture](../solver-abstraction/ARCHITECTURE.md)
- ADR-022: Z3 Integration (current backend)
- ADR-023: Solver Abstraction Layer (enables multiple backends)


