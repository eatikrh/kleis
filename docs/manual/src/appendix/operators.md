# Appendix B: Operators

## Arithmetic Operators

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `+` | Addition | `3 + 4` | `7` |
| `-` | Subtraction | `10 - 3` | `7` |
| `*` | Multiplication | `6 * 7` | `42` |
| `/` | Division | `15 / 3` | `5` |
| `^` | Exponentiation | `2 ^ 10` | `1024` |
| `-` (unary) | Negation | `-5` | `-5` |

## Comparison Operators

| Operator | Unicode | Name | Example |
|----------|---------|------|---------|
| `=` | | Equality | `x = y` |
| `!=` | `≠` | Inequality | `x ≠ y` |
| `<` | | Less than | `x < y` |
| `>` | | Greater than | `x > y` |
| `<=` | `≤` | Less or equal | `x ≤ y` |
| `>=` | `≥` | Greater or equal | `x ≥ y` |

## Logical Operators

| Operator | Unicode | Name | Example |
|----------|---------|------|---------|
| `and` | `∧` | Conjunction | `P ∧ Q` |
| `or` | `∨` | Disjunction | `P ∨ Q` |
| `not` | `¬` | Negation | `¬P` |
| `implies` | `→` | Implication | `P → Q` |
| `iff` | `↔` | Biconditional | `P ↔ Q` |

## Set Operators

| Operator | Unicode | Name | Example |
|----------|---------|------|---------|
| `in` | `∈` | Membership | `x ∈ S` |
| `notin` | `∉` | Non-membership | `x ∉ S` |
| `subset` | `⊆` | Subset | `A ⊆ B` |
| `superset` | `⊇` | Superset | `A ⊇ B` |
| `union` | `∪` | Union | `A ∪ B` |
| `intersect` | `∩` | Intersection | `A ∩ B` |

## Type Operators

| Operator | Name | Example |
|----------|------|---------|
| `→` | Function type | `ℝ → ℝ` |
| `×` | Product type | `ℝ × ℝ` |
| `:` | Type annotation | `x : ℝ` |

## Precedence Table

From lowest to highest precedence:

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 | `↔` | Left |
| 2 | `→` (logical) | Right |
| 3 | `∨` | Left |
| 4 | `∧` | Left |
| 5 | `¬` | Prefix |
| 6 | `=` `≠` `<` `>` `≤` `≥` | None |
| 7 | `∪` `∩` | Left |
| 8 | `+` `-` | Left |
| 9 | `*` `/` | Left |
| 10 | `^` | Right |
| 11 | `-` (unary) | Prefix |
| 12 | `.` (field access) | Left |

## Examples

### Arithmetic

```kleis
2 + 3 * 4        -- 14 (not 20)
(2 + 3) * 4      -- 20
2 ^ 3 ^ 2        -- 512 (= 2^9, right associative)
-x^2             -- -(x^2), not (-x)^2
```

### Logical

```kleis
P ∧ Q ∨ R        -- (P ∧ Q) ∨ R
P → Q → R        -- P → (Q → R) (right associative)
¬P ∧ Q           -- (¬P) ∧ Q
```

### Type Expressions

```kleis
ℝ → ℝ → ℝ        -- ℝ → (ℝ → ℝ) (curried binary function)
(ℝ → ℝ) → ℝ      -- Higher-order: takes function, returns value
ℝ × ℝ → ℝ        -- Takes pair, returns value
```

## Built-in Functions

### Mathematical Functions

```kleis
sqrt(x)          -- Square root
abs(x)           -- Absolute value
floor(x)         -- Round down
ceil(x)          -- Round up
round(x)         -- Round to nearest
min(x, y)        -- Minimum
max(x, y)        -- Maximum
```

### Trigonometric Functions

```kleis
sin(x)   cos(x)   tan(x)
asin(x)  acos(x)  atan(x)
sinh(x)  cosh(x)  tanh(x)
```

### Exponential and Logarithmic

```kleis
exp(x)           -- e^x
ln(x)            -- Natural logarithm
log(x)           -- Base-10 logarithm
log(b, x)        -- Logarithm base b
```

### Constants

```kleis
π                -- Pi (3.14159...)
e                -- Euler's number (2.71828...)
i                -- Imaginary unit
```
