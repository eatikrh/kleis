# Complex Numbers

Kleis has first-class support for complex numbers (ℂ), enabling symbolic reasoning about complex arithmetic, verification of identities, and theorem proving in complex analysis.

## Natural Arithmetic Syntax ✨ NEW

**Kleis now supports natural arithmetic operators for complex numbers!**

You can write expressions using `+`, `-`, `*`, `/` with complex numbers, just like you would with real numbers:

```kleis
// Natural syntax (NEW!)
:verify complex(1, 2) + complex(3, 4) = complex(4, 6)
// ✅ Valid

:verify complex(1, 2) * complex(3, 4) = complex(-5, 10)
// ✅ Valid

// The classic: 3 + 4i
:verify 3 + 4*i = complex(3, 4)
// ✅ Valid

// Mixed real and complex
:verify 5 + complex(1, 2) = complex(6, 2)
// ✅ Valid
```

Kleis automatically converts these to the appropriate complex operations via **semantic lowering**:

| You Write | Kleis Translates To |
|-----------|---------------------|
| `z1 + z2` | `complex_add(z1, z2)` |
| `z1 - z2` | `complex_sub(z1, z2)` |
| `z1 * z2` | `complex_mul(z1, z2)` |
| `z1 / z2` | `complex_div(z1, z2)` |
| `r + z` (ℝ + ℂ) | `complex_add(complex(r, 0), z)` |
| `-z` | `neg_complex(z)` |

This works transparently in the REPL and for verification.

## The Imaginary Unit

The imaginary unit `i` is predefined in Kleis:

```kleis
// i is the square root of -1
define i_squared = i * i
// Result: complex(-1, 0)  — that's -1!
```

In the REPL, you can verify this fundamental property:

```kleis
:verify i * i = complex(-1, 0)
// ✅ Valid

// Or using the explicit function:
:verify complex_mul(i, i) = complex(-1, 0)
// ✅ Valid
```

### Scoping Rules for `i`

The imaginary unit `i` is a global constant. However, it can be shadowed by:

1. **Quantified variables** with explicit type annotations
2. **Lambda parameters**

| Expression | Type | Explanation |
|------------|------|-------------|
| `i` | Complex | Global imaginary unit |
| `i + 1` | Complex | Uses global `i` |
| `i * i` | Complex | `i² = -1` |
| `λ x . x + i` | Complex | Uses global `i` in body |
| `∀(i : ℝ). i + 1` | Scalar | Quantifier `i : ℝ` shadows global |
| `∀(i : ℕ). i + 0` | Nat | Quantifier `i : ℕ` shadows global |
| `λ i . i + 1` | Scalar | Lambda param shadows global |

**Best practice:** Avoid using `i` as a variable name to prevent confusion with the imaginary unit. Use descriptive names like `idx`, `index`, or `iter` for loop-like variables.

```kleis
// Clear: using i as imaginary unit
:verify ∀(z : ℂ). z * i = complex(neg(im(z)), re(z))
// ✅ Valid

// Clear: using idx as index variable
:verify ∀(idx : ℕ). idx + 0 = idx
// ✅ Valid
```

## Creating Complex Numbers

**Method 1: Using arithmetic (recommended)**

```kleis
define z1 = 3 + 4*i           // 3 + 4i
define z2 = 1 - 2*i           // 1 - 2i
define pure_real = 5 + 0*i    // 5 (a real number)
define pure_imag = 0 + 3*i    // 3i (pure imaginary)
```

**Method 2: Using the `complex(re, im)` constructor**

```kleis
// complex(real_part, imaginary_part)
define z1 = complex(3, 4)       // 3 + 4i
define z2 = complex(1, -2)      // 1 - 2i
define pure_real = complex(5, 0)     // 5 (a real number)
define pure_imag = complex(0, 3)     // 3i (pure imaginary)
```

## Extracting Parts

Use `re` and `im` to extract the real and imaginary parts:

```kleis
define z = complex(3, 4)

// Extract real part
define real_part = re(z)        // 3

// Extract imaginary part  
define imag_part = im(z)        // 4
```

Verification examples:

```kleis
:verify re(complex(7, 8)) = 7
// ✅ Valid

:verify im(complex(7, 8)) = 8
// ✅ Valid

:verify ∀(a : ℝ)(b : ℝ). re(complex(a, b)) = a
// ✅ Valid
```

## Type Ascriptions with ℂ

Type ascriptions tell Kleis (and Z3) that a variable is a complex number. The syntax is `: ℂ` (or `: Complex`).

### Quantifier Variables

The most common use is in universal quantifiers:

```kleis
// z is a complex variable
:verify ∀(z : ℂ). conj(conj(z)) = z
// ✅ Valid

// Multiple complex variables
:verify ∀(z1 : ℂ)(z2 : ℂ). z1 + z2 = z2 + z1
// ✅ Valid

// Mixed types: real and complex
:verify ∀(r : ℝ)(z : ℂ). r + z = complex(r + re(z), im(z))
// ✅ Valid
```

When you write `∀(z : ℂ)`, the Z3 backend creates a symbolic complex variable with unknown real and imaginary parts. This lets Z3 reason about **all possible** complex numbers.

### Definition Annotations

You can annotate definitions for clarity:

```kleis
define z1 : ℂ = complex(1, 2)
define z2 : ℂ = 3 + 4*i
define origin : ℂ = complex(0, 0)
```

### Why Type Ascriptions Matter

Without type information, Z3 wouldn't know how to handle operations:

```kleis
// With `: ℂ`, Z3 knows z is complex and creates appropriate constraints
:verify ∀(z : ℂ). z * complex(1, 0) = z
// ✅ Valid

// Z3 can reason symbolically about the real and imaginary parts
:verify ∀(z : ℂ). re(z) * re(z) + im(z) * im(z) = abs_squared(z)
// ✅ Valid
```

### Equivalent Type Names

These are all equivalent:

| Syntax | Description |
|--------|-------------|
| `: ℂ` | Unicode symbol (recommended) |
| `: Complex` | Full name |
| `: C` | Short ASCII alternative |

## Arithmetic Operations

### Addition and Subtraction

```kleis
define z1 = 1 + 2*i    // 1 + 2i
define z2 = 3 + 4*i    // 3 + 4i

// Addition: (1 + 2i) + (3 + 4i) = 4 + 6i
define sum = z1 + z2

// Subtraction: (1 + 2i) - (3 + 4i) = -2 - 2i
define diff = z1 - z2
```

Verify concrete arithmetic:

```kleis
// Natural syntax
:verify (1 + 2*i) + (3 + 4*i) = 4 + 6*i
// ✅ Valid

:verify (5 + 3*i) - (2 + 1*i) = 3 + 2*i
// ✅ Valid

// Explicit function syntax (also works)
:verify complex_add(complex(1, 2), complex(3, 4)) = complex(4, 6)
// ✅ Valid
```

### Multiplication

Complex multiplication follows the rule: `(a + bi)(c + di) = (ac - bd) + (ad + bc)i`

```kleis
define z1 = 1 + 2*i    // 1 + 2i
define z2 = 3 + 4*i    // 3 + 4i

// (1 + 2i)(3 + 4i) = 3 + 4i + 6i + 8i² = 3 + 10i - 8 = -5 + 10i
define product = z1 * z2
```

Verification:

```kleis
// Natural syntax
:verify (1 + 2*i) * (3 + 4*i) = complex(-5, 10)
// ✅ Valid

// The fundamental property: i² = -1
:verify i * i = complex(-1, 0)
// ✅ Valid

// Multiplication by i rotates 90°
:verify ∀(z : ℂ). z * i = complex(neg(im(z)), re(z))
// ✅ Valid (where neg is negation)
```

### Division

```kleis
define z1 = 1 + 0*i    // 1
define z2 = 0 + 1*i    // i

// 1 / i = -i
define quotient = z1 / z2
```

Verification:

```kleis
// Natural syntax
:verify (1 + 0*i) / (0 + 1*i) = 0 - 1*i
// ✅ Valid

// Explicit function syntax
:verify complex_div(complex(1, 0), complex(0, 1)) = complex(0, -1)
// ✅ Valid
```

### Negation

```kleis
define z = complex(3, 4)
define neg_z = neg_complex(z)    // -3 - 4i
```

```kleis
:verify neg_complex(complex(3, 4)) = complex(-3, -4)
// ✅ Valid

:verify ∀(z : ℂ). complex_add(z, neg_complex(z)) = complex(0, 0)
// ✅ Valid
```

### Inverse

```kleis
define z = complex(0, 1)         // i
define inv = complex_inverse(z)   // 1/i = -i
```

```kleis
:verify complex_inverse(complex(0, 1)) = complex(0, -1)
// ✅ Valid

// z * (1/z) = 1 (for non-zero z)
:verify ∀(z : ℂ). z ≠ complex(0, 0) ⟹ complex_mul(z, complex_inverse(z)) = complex(1, 0)
// ✅ Valid
```

## Complex Conjugate

The conjugate of `a + bi` is `a - bi`:

```kleis
define z = complex(3, 4)
define z_bar = conj(z)    // 3 - 4i
```

Verification:

```kleis
:verify conj(complex(2, 3)) = complex(2, -3)
// ✅ Valid

// Double conjugate is identity
:verify ∀(z : ℂ). conj(conj(z)) = z
// ✅ Valid

// Conjugate of product
:verify ∀(z1 : ℂ)(z2 : ℂ). conj(complex_mul(z1, z2)) = complex_mul(conj(z1), conj(z2))
// ✅ Valid

// Conjugate of sum
:verify ∀(z1 : ℂ)(z2 : ℂ). conj(complex_add(z1, z2)) = complex_add(conj(z1), conj(z2))
// ✅ Valid
```

## Magnitude Squared

The squared magnitude `|z|² = re(z)² + im(z)²`:

```kleis
define z = complex(3, 4)
define mag_sq = abs_squared(z)    // 3² + 4² = 25
```

```kleis
:verify abs_squared(complex(3, 4)) = 25
// ✅ Valid

:verify ∀(z : ℂ). abs_squared(z) = re(z) * re(z) + im(z) * im(z)
// ✅ Valid
```

Note: Full magnitude `|z| = √(re² + im²)` requires square root, which is not yet supported.

## Field Properties

Complex numbers form a field. Kleis can verify all field axioms:

### Commutativity

```kleis
// Addition is commutative
:verify ∀(z1 : ℂ)(z2 : ℂ). complex_add(z1, z2) = complex_add(z2, z1)
// ✅ Valid

// Multiplication is commutative
:verify ∀(z1 : ℂ)(z2 : ℂ). complex_mul(z1, z2) = complex_mul(z2, z1)
// ✅ Valid
```

### Associativity

```kleis
// Addition is associative
:verify ∀(z1 : ℂ)(z2 : ℂ)(z3 : ℂ). 
    complex_add(complex_add(z1, z2), z3) = complex_add(z1, complex_add(z2, z3))
// ✅ Valid

// Multiplication is associative
:verify ∀(z1 : ℂ)(z2 : ℂ)(z3 : ℂ). 
    complex_mul(complex_mul(z1, z2), z3) = complex_mul(z1, complex_mul(z2, z3))
// ✅ Valid
```

### Identity Elements

```kleis
// Additive identity: z + 0 = z
:verify ∀(z : ℂ). complex_add(z, complex(0, 0)) = z
// ✅ Valid

// Multiplicative identity: z * 1 = z
:verify ∀(z : ℂ). complex_mul(z, complex(1, 0)) = z
// ✅ Valid
```

### Distributive Law

```kleis
:verify ∀(z1 : ℂ)(z2 : ℂ)(z3 : ℂ). 
    complex_mul(z1, complex_add(z2, z3)) = 
        complex_add(complex_mul(z1, z2), complex_mul(z1, z3))
// ✅ Valid
```

## Embedding Real Numbers

Real numbers embed into complex numbers with imaginary part 0:

```kleis
define r = 5
define z = complex(r, 0)    // 5 + 0i = 5

// Extracting real from embedded real
:verify ∀(a : ℝ). re(complex(a, 0)) = a
// ✅ Valid

:verify ∀(a : ℝ). im(complex(a, 0)) = 0
// ✅ Valid
```

Adding real and imaginary parts:

```kleis
:verify ∀(x : ℝ)(y : ℝ). complex_add(complex(x, 0), complex(0, y)) = complex(x, y)
// ✅ Valid
```

## The Multiplication Formula

The explicit formula for complex multiplication:

```kleis
// (a + bi)(c + di) = (ac - bd) + (ad + bc)i
:verify ∀(a : ℝ)(b : ℝ)(c : ℝ)(d : ℝ). 
    complex_mul(complex(a, b), complex(c, d)) = complex(a*c - b*d, a*d + b*c)
// ✅ Valid
```

## Mixing Symbolic and Concrete

Kleis can reason about mixed expressions:

```kleis
// Symbolic z plus concrete value
:verify ∀(z : ℂ). complex_add(z, complex(0, 0)) = z
// ✅ Valid

// Symbolic z times concrete i
:verify ∀(z : ℂ). complex_mul(z, i) = complex_add(z, complex(0, 1))
// This checks if multiplying by i equals adding i (it doesn't!)
// ❌ Invalid — as expected!

// Correct: multiplying by i rotates
:verify ∀(a : ℝ)(b : ℝ). complex_mul(complex(a, b), i) = complex(-b, a)
// ✅ Valid (rotation by 90°)
```

## The Fundamental Theorem

The defining property of complex numbers:

```kleis
// i² = -1
:verify complex_mul(i, i) = complex(-1, 0)
// ✅ Valid

// More explicitly
:verify complex_mul(complex(0, 1), complex(0, 1)) = complex(-1, 0)
// ✅ Valid
```

## Convention: Loop Indices

When using `Sum` or `Product` with complex numbers, **avoid using `i` as a loop index**:

```kleis
// GOOD: use k, j, n, m as loop indices
Sum(k, complex_mul(complex(1, 0), pow(z, k)), 0, n)

// BAD: i as loop index clashes with imaginary i
Sum(i, complex_mul(i, pow(z, i)), 0, n)   // Which 'i' is which?
```

The convention is:
- `k`, `j`, `n`, `m` — loop indices
- `i` — imaginary unit

## Complete Example: Verifying Complex Identities

Here's a complete session verifying multiple complex number properties:

```kleis
// Define some complex numbers
define z1 : ℂ = complex(1, 2)
define z2 : ℂ = complex(3, 4)

// Compute operations
define sum = complex_add(z1, z2)
define product = complex_mul(z1, z2)
define i_squared = complex_mul(i, i)

// Structure with axioms
structure ComplexTest {
    axiom i_squared_minus_one : complex_mul(i, i) = complex(-1, 0)
    axiom conj_involution : ∀(z : ℂ). conj(conj(z)) = z
    axiom add_commutes : ∀(a : ℂ)(b : ℂ). complex_add(a, b) = complex_add(b, a)
}
```

## Operation Reference

| Operation | Natural Syntax | Explicit Syntax | Description |
|-----------|----------------|-----------------|-------------|
| Create | `a + b*i` | `complex(a, b)` | Create a + bi |
| Real part | — | `re(z)` | Extract real part |
| Imaginary part | — | `im(z)` | Extract imaginary part |
| Add | `z1 + z2` | `complex_add(z1, z2)` | z1 + z2 |
| Subtract | `z1 - z2` | `complex_sub(z1, z2)` | z1 - z2 |
| Multiply | `z1 * z2` | `complex_mul(z1, z2)` | z1 × z2 |
| Divide | `z1 / z2` | `complex_div(z1, z2)` | z1 / z2 |
| Negate | `-z` | `neg_complex(z)` | -z |
| Inverse | — | `complex_inverse(z)` | 1/z |
| Conjugate | — | `conj(z)` | Complex conjugate |
| Magnitude² | — | `abs_squared(z)` | \|z\|² |

## Current Limitations

| Feature | Status | Notes |
|---------|--------|-------|
| Operator overloading | ✅ | `z1 + z2`, `3 + 4*i` work! |
| Magnitude `abs(z)` | ❌ | Requires sqrt |
| Transcendentals | ❌ | `exp`, `log`, `sin`, `cos` |
| Polar form | ❌ | `(r, θ)` |
| Euler's formula | ❌ | `e^{iθ} = cos(θ) + i·sin(θ)` |

## What's Next?

Complex numbers enable reasoning about:
- Signal processing (Fourier transforms)
- Quantum mechanics (wave functions)
- Control theory (transfer functions)
- Complex analysis (contour integrals)

→ [Next: Grammar Reference](../appendix/grammar.md)

