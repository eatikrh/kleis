# Quantifiers and Logic

## Universal Quantifier (∀)

The universal quantifier expresses "for all":

```kleis
∀ x . x = x                      // Everything equals itself
∀ x . x + 0 = x                  // Zero is additive identity
∀ x . ∀ y . x + y = y + x        // Addition is commutative
```

ASCII alternative: `forall x . ...`

## Existential Quantifier (∃)

The existential quantifier expresses "there exists":

```kleis
∃ x . x > 0                      // Some number is positive
∃ y . y * y = 2                  // Square root of 2 exists
∃ x . ∃ y . x ≠ y                // At least two things exist
```

ASCII alternative: `exists x . ...`

## Combining Quantifiers

Build complex statements:

```kleis
-- Every number has a successor
∀ n : ℕ . ∃ m : ℕ . m = n + 1

-- Density of rationals: between any two reals is a rational
∀ x : ℝ . ∀ y : ℝ . x < y → ∃ q : ℚ . x < q ∧ q < y
```

## Logical Connectives

### Conjunction (∧ / and)

```kleis
x > 0 ∧ x < 10     // x is between 0 and 10
True ∧ False       // False
```

### Disjunction (∨ / or)

```kleis
x = 0 ∨ x = 1      // x is 0 or 1
True ∨ False       // True
```

### Implication (→ / implies)

```kleis
x > 0 → sqrt(x) ∈ ℝ    // If positive, has real square root
P → Q                   // If P then Q
```

### Negation (¬ / not)

```kleis
¬(x = 0)           // x is not zero
¬True              // False
```

### Biconditional (↔ / iff)

```kleis
x = 0 ↔ x * x = 0  // x is zero iff x² is zero
```

## Type Constraints in Quantifiers

Restrict the domain:

```kleis
∀ x : ℕ . x ≥ 0                  // All naturals are non-negative
∀ M : Matrix(n, n) . det(M * M⁻¹) = 1   // Determinant property
```

## Using Quantifiers in Axioms

Quantifiers are essential in structure axioms:

```kleis
structure Group(G) {
    operation e : G                      // Identity element
    operation mul : G × G → G
    operation inv : G → G
    
    axiom identity : ∀ x : G . mul(e, x) = x ∧ mul(x, e) = x
    axiom inverse : ∀ x : G . mul(x, inv(x)) = e
    axiom associative : ∀ x : G . ∀ y : G . ∀ z : G .
        mul(mul(x, y), z) = mul(x, mul(y, z))
}
```

## Verification with Z3

Kleis uses Z3 to check quantified statements:

```kleis
-- Z3 can verify this is always true:
verify ∀ x : ℝ . x + 0 = x

-- Z3 can find a counterexample for this:
verify ∀ x : ℝ . x > 0
-- Counterexample: x = -1
```

## Truth Tables

| P | Q | P ∧ Q | P ∨ Q | P → Q | ¬P |
|---|---|-------|-------|-------|-----|
| T | T |   T   |   T   |   T   |  F  |
| T | F |   F   |   T   |   F   |  F  |
| F | T |   F   |   T   |   T   |  T  |
| F | F |   F   |   F   |   T   |  T  |

## What's Next?

Learn about conditional expressions!

→ [Next: Conditionals](./08-conditionals.md)
