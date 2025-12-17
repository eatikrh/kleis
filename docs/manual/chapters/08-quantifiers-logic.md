# Chapter 8: Quantifiers and Logic

[← Previous: Let Bindings](07-let-bindings.md) | [Back to Contents](../index.md) | [Next: Conditionals →](09-conditionals.md)

---

## Universal Quantifier (∀)

"For all" — a statement that holds for every value:

```kleis
∀(x : ℝ). x + 0 = x
```

Read as: "For all real numbers x, x plus zero equals x."

### Syntax

```kleis
∀(variable : Type). expression

// Or ASCII:
forall(variable : Type). expression
```

### Examples

```kleis
// Identity laws
∀(x : ℝ). x + 0 = x
∀(x : ℝ). x * 1 = x

// Commutativity
∀(x y : ℝ). x + y = y + x
∀(x y : ℝ). x * y = y * x

// Associativity
∀(x y z : ℝ). (x + y) + z = x + (y + z)
```

### Multiple Variables

```kleis
// Two variables
∀(x y : ℝ). x + y = y + x

// Three variables
∀(a b c : ℝ). (a + b) + c = a + (b + c)

// Mixed types
∀(v : Vector(3)). ∀(s : ℝ). norm(s * v) = abs(s) * norm(v)
```

---

## Existential Quantifier (∃)

"There exists" — at least one value satisfies the property:

```kleis
∃(x : ℝ). x * x = 2
```

Read as: "There exists a real number x such that x squared equals 2."

### Syntax

```kleis
∃(variable : Type). expression

// Or ASCII:
exists(variable : Type). expression
```

### Examples

```kleis
// Square root of 2 exists
∃(x : ℝ). x^2 = 2

// Every number has an additive inverse
∀(x : ℝ). ∃(y : ℝ). x + y = 0

// Intermediate value theorem (informally)
∀(f : ℝ → ℝ). ∀(a b : ℝ).
    f(a) < 0 ∧ f(b) > 0 ⟹ ∃(c : ℝ). f(c) = 0
```

---

## Logical Operators

Kleis supports standard logical operators:

| Symbol | ASCII | Meaning | Example |
|--------|-------|---------|---------|
| `∧` | `and` | Conjunction | `p ∧ q` |
| `∨` | `or` | Disjunction | `p ∨ q` |
| `¬` | `not` | Negation | `¬p` |
| `⟹` | `=>` | Implication | `p ⟹ q` |

### Conjunction (AND)

Both must be true:

```kleis
>>> True ∧ True
True

>>> True ∧ False
False

>>> (5 > 3) ∧ (2 < 4)
True
```

### Disjunction (OR)

At least one must be true:

```kleis
>>> True ∨ False
True

>>> False ∨ False
False

>>> (5 > 10) ∨ (2 < 4)
True
```

### Negation (NOT)

Flips the truth value:

```kleis
>>> ¬True
False

>>> ¬False
True

>>> ¬(5 > 10)
True
```

### Implication

"If p then q":

```kleis
>>> True ⟹ True
True

>>> True ⟹ False
False

>>> False ⟹ True
True    // Vacuously true!

>>> False ⟹ False
True    // Vacuously true!
```

**Important:** `False ⟹ anything` is always true. This is "vacuous truth."

---

## Operator Precedence

From highest to lowest:

1. `¬` (not) — binds tightest
2. `∧` (and)
3. `∨` (or)
4. `⟹` (implies) — binds loosest

```kleis
// This:
¬p ∧ q ∨ r ⟹ s

// Means:
((¬p) ∧ q) ∨ r) ⟹ s

// Use parentheses for clarity:
(¬p ∧ q) ∨ (r ⟹ s)
```

---

## Comparison Operators

| Symbol | ASCII | Meaning |
|--------|-------|---------|
| `=` | `=` | Equals |
| `≠` | `!=` | Not equals |
| `<` | `<` | Less than |
| `>` | `>` | Greater than |
| `≤` | `<=` | Less or equal |
| `≥` | `>=` | Greater or equal |

```kleis
>>> 5 = 5
True

>>> 5 ≠ 3
True

>>> 3 < 5
True

>>> 5 ≤ 5
True

>>> 5 ≥ 10
False
```

---

## Combining Everything

Real mathematical statements combine quantifiers and logic:

```kleis
// Group inverse axiom
∀(x : G). ∃(y : G). x * y = e ∧ y * x = e

// Distributivity
∀(a b c : R). a * (b + c) = a * b + a * c

// Definition of limit
∀(ε : ℝ). ε > 0 ⟹ ∃(δ : ℝ). δ > 0 ∧
    ∀(x : ℝ). abs(x - a) < δ ⟹ abs(f(x) - L) < ε

// Continuity
∀(x : ℝ). ∀(ε : ℝ). ε > 0 ⟹
    ∃(δ : ℝ). δ > 0 ∧
    ∀(y : ℝ). abs(y - x) < δ ⟹ abs(f(y) - f(x)) < ε
```

---

## In Structures and Axioms

Quantifiers are essential for defining mathematical axioms:

```kleis
structure Group(G) {
    operation (*) : G → G → G
    operation e : G
    operation inv : G → G
    
    axiom identity : ∀(x : G). x * e = x ∧ e * x = x
    axiom inverse : ∀(x : G). x * inv(x) = e ∧ inv(x) * x = e
    axiom assoc : ∀(x y z : G). (x * y) * z = x * (y * z)
}
```

---

## De Morgan's Laws

Useful for simplifying expressions:

```kleis
// ¬(p ∧ q) ≡ ¬p ∨ ¬q
// ¬(p ∨ q) ≡ ¬p ∧ ¬q

// Negating quantifiers:
// ¬(∀x. P(x)) ≡ ∃x. ¬P(x)
// ¬(∃x. P(x)) ≡ ∀x. ¬P(x)
```

---

## Exercises

1. **Write** the commutative law for multiplication

2. **Write** "there exists a real number whose square is 4"

3. **Write** the transitivity of less-than

4. **Evaluate** `(True ∧ False) ∨ True`

5. **Write** "for all x, if x > 0 then x² > 0"

<details>
<summary>Solutions</summary>

```kleis
// 1.
∀(x y : ℝ). x * y = y * x

// 2.
∃(x : ℝ). x^2 = 4

// 3.
∀(a b c : ℝ). (a < b) ∧ (b < c) ⟹ (a < c)

// 4.
(True ∧ False) ∨ True
= False ∨ True
= True

// 5.
∀(x : ℝ). x > 0 ⟹ x^2 > 0
```

</details>

---

## Summary

- `∀(x : T). P` — for all x of type T, P holds
- `∃(x : T). P` — there exists x of type T where P holds
- Logic: `∧` (and), `∨` (or), `¬` (not), `⟹` (implies)
- Comparisons: `=`, `≠`, `<`, `>`, `≤`, `≥`
- Precedence: `¬` > `∧` > `∨` > `⟹`

---

[← Previous: Let Bindings](07-let-bindings.md) | [Back to Contents](../index.md) | [Next: Conditionals →](09-conditionals.md)

