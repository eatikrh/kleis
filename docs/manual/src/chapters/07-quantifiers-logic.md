# Quantifiers and Logic

## Universal Quantifier (∀)

The universal quantifier expresses "for all":

```kleis example
// Quantified propositions (used inside axioms)
axiom reflexivity : ∀(x : ℝ). x = x
axiom additive_identity : ∀(x : ℝ). x + 0 = x
axiom commutative : ∀(x : ℝ)(y : ℝ). x + y = y + x
```

ASCII alternative: `forall x . ...`

## Existential Quantifier (∃)

The existential quantifier expresses "there exists":

```kleis example
// Existential quantifiers
axiom positive_exists : ∃(x : ℝ). x > 0
axiom sqrt2_exists : ∃(y : ℝ). y * y = 2
axiom distinct_exists : ∃(x : ℝ)(y : ℝ). x ≠ y
```

ASCII alternative: `exists x . ...`

## Combining Quantifiers

Build complex statements:

```kleis example
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

```kleis example
axiom naturals_nonneg : ∀(x : ℕ). x ≥ 0
axiom det_inverse : ∀(M : Matrix(n, n)). det(M * M⁻¹) = 1
```

## The `where` Clause

Add conditions to quantified variables using the `where` keyword:

```kleis
structure Field(F) {
    element zero : F
    element one : F
    operation inverse : F → F
    
    // Multiplicative inverse only for non-zero elements
    axiom multiplicative_inverse:
        ∀(x : F) where x ≠ zero. inverse(x) * x = one
}
```

The `where` clause restricts the domain before the quantified body is evaluated. This is essential for axioms that don't apply universally.

**More examples:**

```kleis
structure Analysis {
    // Division only defined for non-zero denominator
    axiom division: ∀(a : ℝ)(b : ℝ) where b ≠ 0. a / b * b = a
    
    // Logarithm only for positive numbers
    axiom log_exp: ∀(x : ℝ) where x > 0. exp(log(x)) = x
}
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

## Nested Quantifiers (Grammar v0.9)

Quantifiers can appear inside logical expressions:

```kleis
structure Analysis {
    // Quantifier inside conjunction
    axiom bounded_positive: (x > 0) ∧ (∀(y : ℝ). abs(y) <= x)
    
    // Quantifier inside implication
    axiom dense_rationals: ∀(a b : ℝ). a < b → (∃(q : ℚ). a < q ∧ q < b)
    
    // Deeply nested quantifiers
    axiom limit_def: ∀(L : ℝ, ε : ℝ). ε > 0 → 
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x) < δ → abs(f(x) - L) < ε))
}
```

### Epsilon-Delta Limit Definition

The classic analysis definition now parses correctly:

```kleis
structure Limits {
    axiom epsilon_delta: ∀(f : ℝ → ℝ, L a : ℝ). 
        has_limit(f, a, L) ↔ 
        (∀(ε : ℝ). ε > 0 → (∃(δ : ℝ). δ > 0 ∧ 
            (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε)))
}
```

## Function Types in Quantifiers (Grammar v0.9)

Quantify over functions using the arrow type:

```kleis
structure FunctionProperties {
    // Quantify over a function ℝ → ℝ
    axiom continuous: ∀(f : ℝ → ℝ, x : ℝ). 
        is_continuous(f, x)
    
    // Quantify over multiple functions
    axiom composition: ∀(f : ℝ → ℝ, g : ℝ → ℝ). 
        compose(f, g) = λ x . f(g(x))
    
    // Higher-order function types
    axiom curried: ∀(f : ℝ → ℝ → ℝ, a b : ℝ). 
        f = f
}
```

### Topology with Function Types

```kleis
structure Topology {
    axiom continuity: ∀(f : X → Y, V : Set(Y)). 
        is_open(V) → is_open(preimage(f, V))
    
    axiom homeomorphism: ∀(f : X → Y, g : Y → X). 
        (∀(x : X). g(f(x)) = x) ∧ (∀(y : Y). f(g(y)) = y) → 
        bijective(f)
}
```

## Verification with Z3

Kleis uses Z3 to check quantified statements:

```kleis example
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
