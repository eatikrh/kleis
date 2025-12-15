# Z3 and Calculus: Derivatives and Integrals

**Date:** December 12, 2025  
**Question:** Can Z3 evaluate derivatives and integrals?

---

## 🎯 Short Answer: Limited, But Interesting!

**Z3 is NOT a computer algebra system (CAS) like:**
- Mathematica
- SymPy
- Maple
- Maxima

**But Z3 CAN reason about calculus in specific ways!**

---

## What Z3 Can Do

### 1. ✅ **Reason About Properties of Derivatives**

**Example: Prove linearity of differentiation**
```
d/dx (f + g) = df/dx + dg/dx
```

**Z3 approach:**
```smt
; Declare derivative as uninterpreted function
(declare-fun d (Function) Function)

; Assert linearity property
(assert (forall ((f Function) (g Function))
  (= (d (plus f g)) 
     (plus (d f) (d g)))))

; Now Z3 can use this property in proofs!
```

**What this means:**
- ✅ Z3 can verify algebraic properties of derivatives
- ✅ Can prove: d/dx(2f) = 2·df/dx
- ✅ Can reason about differential equations (symbolically)
- ❌ Cannot compute: d/dx(x²) = 2x (needs symbolic math)

### 2. ✅ **Verify Specific Derivative Facts**

**If you tell Z3 the derivatives:**
```smt
; Tell Z3: d/dx(x²) = 2x
(assert (= (d (square x)) (times 2 x)))

; Tell Z3: d/dx(sin x) = cos x  
(assert (= (d (sin x)) (cos x)))

; Now Z3 can use these to prove theorems!
```

**Example proof:**
```
Given: d/dx(x²) = 2x
Prove: d/dx(3x²) = 6x

Proof:
d/dx(3x²) = 3 · d/dx(x²)    [linearity]
          = 3 · 2x          [given fact]
          = 6x              [arithmetic]
✅ QED
```

### 3. ✅ **Reason About Integrals as Axioms**

**Fundamental theorem of calculus:**
```smt
; Define: integral is inverse of derivative
(assert (forall ((f Function) (a Real) (b Real))
  (= (integral (d f) a b)
     (minus (f b) (f a)))))

; Z3 can use this in proofs!
```

**Example:**
```
Given: ∫ₐᵇ f'(x)dx = f(b) - f(a)
Given: f(x) = x²
Given: f'(x) = 2x
Prove: ∫₀⁵ 2x dx = 25

Proof:
∫₀⁵ 2x dx = f(5) - f(0)    [FTC]
          = 25 - 0          [f(x) = x²]
          = 25              ✅
```

---

## ❌ What Z3 Cannot Do

### 1. **Symbolic Differentiation**

**Cannot compute:**
```
d/dx(x² + 3x + 1) = ?
```

Z3 doesn't have symbolic manipulation rules for:
- Power rule
- Chain rule
- Product rule
- Quotient rule

**You'd need:** Computer Algebra System (Mathematica, SymPy)

### 2. **Symbolic Integration**

**Cannot compute:**
```
∫ x² dx = ?
```

Z3 doesn't have integration techniques:
- Substitution
- Integration by parts
- Partial fractions
- Trig substitution

**You'd need:** CAS with integral tables

### 3. **Limit Computation**

**Cannot compute:**
```
lim(x→0) (sin x)/x = ?
```

Z3 doesn't handle limits or infinite processes.

---

## 💡 How Kleis Could Use Z3 for Calculus

### Approach 1: Derivatives as Axioms (What We Can Do)

**In Kleis:**
```kleis
structure Differentiable(F) {
  operation d : F → F  // Derivative operator
  
  // Linearity axiom
  axiom linearity: ∀(f g : F). d(f + g) = d(f) + d(g)
  
  // Power rule as axiom
  axiom power_rule: ∀(n : ℕ). d(x^n) = n × x^(n-1)
}

// Specific derivative facts
define d_x_squared = 2 × x
define d_sin = cos
```

**Z3 can:**
- ✅ Verify properties using these axioms
- ✅ Prove: d/dx(3x²) = 6x (using linearity + power rule)
- ✅ Check consistency of derivative facts

### Approach 2: Precomputed Derivatives (Practical)

**For common functions, store derivative table:**
```kleis
structure StandardFunctions {
  // Function and its derivative
  define d_polynomial(coeffs) = derivative_coeffs(coeffs)
  define d_sin(x) = cos(x)
  define d_cos(x) = negate(sin(x))
  define d_exp(x) = exp(x)
}
```

**Then:**
- ✅ Z3 can use these in proofs
- ✅ Evaluator can apply them
- ❌ Still can't compute NEW derivatives

### Approach 3: Integration with External CAS

**Hybrid system:**
```rust
// For symbolic calculus, call out to:
fn compute_derivative(expr: &Expression) -> Expression {
    // Option 1: Call SymPy via Python
    // Option 2: Implement simple rules in Rust
    // Option 3: Use existing Rust CAS library
}

// Then give result to Z3 for verification
fn verify_derivative(f: &Expression, df: &Expression) -> bool {
    // Use Z3 to verify the computed derivative satisfies properties
}
```

---

## 🎯 Realistic Example for Kleis

**What we could implement:**

```kleis
structure PolynomialCalculus {
  // Derivative operator (abstract)
  operation d : Polynomial → Polynomial
  
  // Derivative axioms (algebraic properties)
  axiom constant_rule: ∀(c : ℝ). d(const(c)) = const(0)
  axiom power_rule: ∀(n : ℕ). d(x^n) = n × x^(n-1)
  axiom sum_rule: ∀(f g). d(f + g) = d(f) + d(g)
  axiom constant_multiple: ∀(c f). d(c × f) = c × d(f)
  
  // Specific derivatives (precomputed)
  define d_x_squared() = 2 × x
  define d_x_cubed() = 3 × x^2
}
```

**What Z3 could do:**
```kleis
// Prove using the axioms
axiom verify: d(3 × x^2) = 6 × x

// Z3 proof:
d(3 × x²) = 3 × d(x²)        [constant multiple rule]
          = 3 × (2 × x)      [power rule]
          = 6 × x            [arithmetic]
✅ PROVEN!
```

**What Z3 cannot do:**
```kleis
// This requires symbolic computation
define mystery(x) = sin(x^2 + 3x)
// Q: What is d/dx(mystery)?
// A: Z3 cannot compute this! Need CAS.
```

---

## 📊 Comparison

| Operation | Z3 Can Do | Z3 Cannot Do |
|-----------|-----------|--------------|
| **Derivatives** | Verify properties (linearity, etc.) | Compute d/dx(f) symbolically |
| | Use precomputed derivatives | Apply chain rule |
| | Prove: d(f+g) = df + dg | Compute: d/dx(sin(x²)) |
| **Integrals** | Verify FTC holds | Compute ∫ f dx |
| | Reason about definite integrals | Find antiderivatives |
| | Prove: ∫₀¹ 2x dx = 1 (if told f=x²) | Evaluate: ∫ sin(x²) dx |
| **Limits** | ❌ Not supported | ❌ Cannot compute |
| **Taylor Series** | Verify coefficients | ❌ Cannot compute |

---

## 🎯 Recommendation for Kleis

**Tier 1: What Z3 CAN do (Implement this)**
```kleis
// Derivative as abstract operation
structure Differentiable(F) {
  operation d : F → F
  
  // Axioms Z3 can use
  axiom linearity: ∀(f g). d(f + g) = d(f) + d(g)
  axiom product_rule: ∀(f g). d(f × g) = d(f) × g + f × d(g)
  
  // Precomputed derivatives
  define d_polynomial_2(x) = 2 × x
  define d_sin = cos
}

// Z3 can VERIFY and PROVE using these!
```

**Tier 2: What we'd need external CAS for**
- Symbolic differentiation of arbitrary expressions
- Integration (indefinite integrals)
- Limit computation
- Series expansion

---

## ✅ Conclusion

**Your suspicion has some truth:**
- ✅ Z3 can reason **about** derivatives and integrals
- ✅ Z3 can verify properties and prove theorems
- ✅ Z3 can use derivative facts we provide
- ❌ Z3 cannot **compute** derivatives/integrals symbolically
- ❌ Z3 is not a replacement for Mathematica/SymPy

**For Kleis:**
- Use Z3 for **verification and proving**
- Use precomputed derivative tables for common functions
- Consider integrating with external CAS for symbolic calculus
- Focus Z3 on algebraic properties (where it excels!)

**Bottom line:** Z3 is great for **reasoning about calculus**, not **doing calculus**.

---

Would you like me to create a proof-of-concept showing Z3 proving derivative properties using axioms?

