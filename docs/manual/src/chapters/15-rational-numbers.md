# Rational Numbers

Kleis provides complete support for **rational numbers** (ℚ), the field of fractions p/q where p and q are integers and q ≠ 0. Rational arithmetic in Kleis is **exact**—no floating-point approximation errors.

## The Rational Type

Kleis recognizes three equivalent notations for the rational type:

```kleis
define half : ℚ = rational(1, 2)
define third : Rational = rational(1, 3)
define quarter : Q = rational(1, 4)
```

## Constructing Rationals

Use the `rational(p, q)` constructor to create rational numbers:

```kleis
define one_half : ℚ = rational(1, 2)
define two_thirds : ℚ = rational(2, 3)
define negative : ℚ = rational(-3, 4)
```

### Accessors

Extract the numerator and denominator:

```kleis
structure RationalAccessors {
    axiom numer_ex : numer(rational(3, 4)) = 3
    axiom denom_ex : denom(rational(3, 4)) = 4
}
```

## Arithmetic Operations

### Basic Arithmetic

Kleis supports operator overloading for rationals:

```kleis
define sum : ℚ = rational(1, 2) + rational(1, 3)
define diff : ℚ = rational(3, 4) - rational(1, 4)
define prod : ℚ = rational(2, 3) * rational(3, 2)
define quot : ℚ = rational(1, 2) / rational(1, 4)
```

These lower to explicit rational operations:

| Operator | Lowers to |
|----------|-----------|
| `r1 + r2` | `rational_add(r1, r2)` |
| `r1 - r2` | `rational_sub(r1, r2)` |
| `r1 * r2` | `rational_mul(r1, r2)` |
| `r1 / r2` | `rational_div(r1, r2)` |
| `-r` | `neg_rational(r)` |

### Reciprocal and Inverse

```kleis
structure RationalInverse {
    axiom inv_half : rational_inv(rational(1, 2)) = rational(2, 1)
    axiom inv_def : ∀(p q : ℤ). p ≠ 0 ∧ q ≠ 0 → 
        rational_inv(rational(p, q)) = rational(q, p)
}
```

## Derived Operations

Kleis defines several derived operations using conditionals:

### Sign, Absolute Value, Min, Max

```kleis
structure DerivedOps {
    define sign_rational(r : ℚ) : ℤ = 
        if rational_lt(r, rational(0, 1)) then 0 - 1
        else if r = rational(0, 1) then 0
        else 1
    
    define abs_rational(r : ℚ) : ℚ = 
        if rational_lt(r, rational(0, 1)) then neg_rational(r) 
        else r
    
    define min_rational(r1 : ℚ, r2 : ℚ) : ℚ = 
        if rational_le(r1, r2) then r1 else r2
    
    define max_rational(r1 : ℚ, r2 : ℚ) : ℚ = 
        if rational_le(r1, r2) then r2 else r1
    
    define midpoint(r1 : ℚ, r2 : ℚ) : ℚ = 
        rational_div(rational_add(r1, r2), rational(2, 1))
}
```

## Comparison Operations

Rationals are totally ordered:

```kleis
structure RationalOrder {
    axiom trichotomy : ∀(r1 r2 : ℚ). 
        rational_lt(r1, r2) ∨ r1 = r2 ∨ rational_gt(r1, r2)
    
    axiom transitive : ∀(r1 r2 r3 : ℚ). 
        rational_lt(r1, r2) ∧ rational_lt(r2, r3) → rational_lt(r1, r3)
}
```

### Mixed-Type Comparisons

Kleis supports comparing rationals with other numeric types:

```kleis
structure MixedComparisons {
    // Compare ℚ with ℕ
    axiom half_less_one : rational(1, 2) < 1
    
    // Compare ℚ with ℤ
    axiom neg_half_less_zero : rational(-1, 2) < 0
}
```

## Type Promotion

When mixing rationals with other numeric types, Kleis promotes according to:

```
ℕ → ℤ → ℚ → ℝ → ℂ
```

Examples:

```kleis
structure TypePromotion {
    // ℚ + ℤ → ℚ (integer lifted to rational)
    axiom int_plus_rat : rational(1, 2) + 1 = rational(3, 2)
    
    // ℚ + ℝ → ℝ (rational becomes real)
    axiom rat_plus_real : rational(1, 2) + 0.5 = 1.0
}
```

## Field Axioms

Rationals form a **field**—all the familiar algebraic laws hold:

```kleis
structure RationalField {
    // Addition is commutative and associative
    axiom add_comm : ∀(r1 r2 : ℚ). rational_add(r1, r2) = rational_add(r2, r1)
    axiom add_assoc : ∀(r1 r2 r3 : ℚ). 
        rational_add(rational_add(r1, r2), r3) = rational_add(r1, rational_add(r2, r3))
    
    // Additive identity and inverse
    axiom add_identity : ∀(r : ℚ). rational_add(r, rational(0, 1)) = r
    axiom add_inverse : ∀(r : ℚ). rational_add(r, neg_rational(r)) = rational(0, 1)
    
    // Multiplication is commutative and associative
    axiom mul_comm : ∀(r1 r2 : ℚ). rational_mul(r1, r2) = rational_mul(r2, r1)
    axiom mul_assoc : ∀(r1 r2 r3 : ℚ). 
        rational_mul(rational_mul(r1, r2), r3) = rational_mul(r1, rational_mul(r2, r3))
    
    // Multiplicative identity and inverse
    axiom mul_identity : ∀(r : ℚ). rational_mul(r, rational(1, 1)) = r
    axiom mul_inverse : ∀(r : ℚ). r ≠ rational(0, 1) → 
        rational_mul(r, rational_inv(r)) = rational(1, 1)
    
    // Distributive law
    axiom distributive : ∀(r1 r2 r3 : ℚ). 
        rational_mul(r1, rational_add(r2, r3)) = 
        rational_add(rational_mul(r1, r2), rational_mul(r1, r3))
}
```

## Integer Operations

### Floor and Ceiling

Convert rationals to integers:

```kleis
structure FloorCeil {
    // floor: largest integer ≤ r
    axiom floor_def : ∀(r : ℚ). int_to_rational(floor(r)) ≤ r
    
    // ceil: smallest integer ≥ r
    axiom ceil_def : ∀(r : ℚ). r ≤ int_to_rational(ceil(r))
    
    // Examples
    axiom floor_ex : floor(rational(7, 3)) = 2
    axiom ceil_ex : ceil(rational(7, 3)) = 3
}
```

### Integer Division and Modulo

```kleis
structure IntDivMod {
    // Division identity: a = (a div b) * b + (a mod b)
    axiom div_mod_id : ∀(a b : ℤ). b ≠ 0 → 
        a = int_div(a, b) * b + int_mod(a, b)
    
    // Modulo is non-negative for positive divisor
    axiom mod_nonneg : ∀(a b : ℤ). b > 0 → 
        int_mod(a, b) ≥ 0 ∧ int_mod(a, b) < b
}
```

### Greatest Common Divisor

GCD is defined axiomatically:

```kleis
structure GCDAxioms {
    // GCD divides both arguments
    axiom gcd_divides_a : ∀(a b : ℤ). int_mod(a, gcd(a, b)) = 0
    axiom gcd_divides_b : ∀(a b : ℤ). int_mod(b, gcd(a, b)) = 0
    
    // GCD is the greatest such divisor
    axiom gcd_greatest : ∀(a b d : ℤ). 
        (int_mod(a, d) = 0 ∧ int_mod(b, d) = 0) → d ≤ gcd(a, b)
    
    // Euclidean algorithm
    axiom gcd_euclidean : ∀(a b : ℤ). b ≠ 0 → 
        gcd(a, b) = gcd(b, int_mod(a, b))
}
```

## Density Property

Between any two distinct rationals, there's another:

```kleis
structure Density {
    axiom density : ∀(r1 r2 : ℚ). 
        rational_lt(r1, r2) → 
        (∃(r : ℚ). rational_lt(r1, r) ∧ rational_lt(r, r2))
    
    // The midpoint is always between
    axiom midpoint_between : ∀(r1 r2 : ℚ). 
        rational_lt(r1, r2) → 
        rational_lt(r1, midpoint(r1, r2)) ∧ rational_lt(midpoint(r1, r2), r2)
}
```

## Z3 Verification

Z3 maps rationals to its `Real` sort, which provides exact rational arithmetic:

```kleis
structure Z3Example {
    // This theorem is verified by Z3
    axiom half_plus_half : rational_add(rational(1, 2), rational(1, 2)) = rational(1, 1)
    
    // Field properties are automatically verified
    axiom comm_verified : ∀(a b : ℚ). rational_add(a, b) = rational_add(b, a)
}
```

## Common Fractions

The standard library defines convenient names:

```kleis
structure CommonFractions {
    axiom half_def : half = rational(1, 2)
    axiom third_def : third = rational(1, 3)
    axiom quarter_def : quarter = rational(1, 4)
    axiom fifth_def : fifth = rational(1, 5)
    axiom tenth_def : tenth = rational(1, 10)
}
```

## Summary

| Feature | Kleis Support |
|---------|---------------|
| Type notation | `ℚ`, `Rational`, `Q` |
| Construction | `rational(p, q)` |
| Arithmetic | `+`, `-`, `*`, `/`, `-` (negation) |
| Comparison | `<`, `≤`, `>`, `≥`, `=`, `≠` |
| Derived ops | `sign`, `abs`, `min`, `max`, `midpoint` |
| Integer ops | `floor`, `ceil`, `int_div`, `int_mod`, `gcd` |
| Z3 backend | Native Real sort (exact arithmetic) |

See `stdlib/rational.kleis` for the complete axiom set.

## What's Next?

Explore fixed-width binary arithmetic for hardware and low-level verification:

→ [Bit Vectors](16-bit-vectors.md)

