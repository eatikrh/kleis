# Quantifiers and Logic

## Universal Quantifier (∀)

The universal quantifier expresses "for all":

```kleis
// Quantified propositions (used inside axioms)
axiom reflexivity : ∀(x : ℝ). x = x
axiom additive_identity : ∀(x : ℝ). x + 0 = x
axiom commutative : ∀(x : ℝ)(y : ℝ). x + y = y + x
```

ASCII alternative: `forall x . ...`

## Existential Quantifier (∃)

The existential quantifier expresses "there exists":

```kleis
// Existential quantifiers
axiom positive_exists : ∃(x : ℝ). x > 0
axiom sqrt2_exists : ∃(y : ℝ). y * y = 2
axiom distinct_exists : ∃(x : ℝ)(y : ℝ). x ≠ y
```

ASCII alternative: `exists x . ...`

## Combining Quantifiers

Build complex statements:

```kleis
// Every number has a successor
axiom successor : ∀(n : ℕ). ∃(m : ℕ). m = n + 1

// Density of rationals: between any two reals is a rational
axiom density : ∀(x : ℝ)(y : ℝ). x < y → ∃(q : ℚ). x < q ∧ q < y
```

## Logical Connectives

### Conjunction (∧ / and)

```kleis
define in_range(x) = x > 0 ∧ x < 10     // x is between 0 and 10
define false_example = True ∧ False     // False
```

### Disjunction (∨ / or)

```kleis
define is_binary(x) = x = 0 ∨ x = 1    // x is 0 or 1
define true_example = True ∨ False     // True
```

### Implication (→ / implies)

```kleis
define positive_square(x) = x > 0 → x * x > 0   // If positive, square is positive
define implication(P, Q) = P → Q                // If P then Q
```

### Negation (¬ / not)

```kleis
define nonzero(x) = ¬(x = 0)     // x is not zero
define not_true = ¬True          // False
```

### Biconditional (↔ / iff)

```kleis
define zero_iff_square_zero(x) = x = 0 ↔ x * x = 0  // x is zero iff x² is zero
```

## Type Constraints in Quantifiers

Restrict the domain:

```kleis
axiom naturals_nonneg : ∀(x : ℕ). x ≥ 0
axiom det_inverse : ∀(M : Matrix(n, n)). det(M * M⁻¹) = 1
```

## Using Quantifiers in Axioms

Quantifiers are essential in structure axioms:

```kleis
structure Group(G) {
    e : G                      // Identity element
    operation mul : G × G → G
    operation inv : G → G
    
    axiom identity : ∀(x : G). mul(e, x) = x ∧ mul(x, e) = x
    axiom inverse : ∀(x : G). mul(x, inv(x)) = e
    axiom associative : ∀(x : G)(y : G)(z : G).
        mul(mul(x, y), z) = mul(x, mul(y, z))
}
```

## Verification with Z3

Kleis uses Z3 to check quantified statements:

```kleis
// Z3 can verify this is always true:
axiom add_zero : ∀(x : ℝ). x + 0 = x

// Z3 can find a counterexample for this:
axiom all_positive : ∀(x : ℝ). x > 0
// Z3 finds counterexample: x = -1
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
