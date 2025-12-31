# Appendix: Operators

This appendix covers all operators in the Kleis language. For built-in functions, see [Built-in Functions](./builtin-functions.md). For numerical linear algebra, see [LAPACK Functions](./lapack.md).

## Arithmetic Operators

| Operator | Unicode | Name | Example | Result |
|----------|---------|------|---------|--------|
| `+` | | Addition | `3 + 4` | `7` |
| `-` | | Subtraction | `10 - 3` | `7` |
| `*` | `×` | Multiplication | `6 × 7` | `42` |
| `/` | `÷` | Division | `15 / 3` | `5` |
| `^` | | Exponentiation | `2 ^ 10` | `1024` |
| `-` (unary) | | Negation | `-5` | `-5` |
| `·` | | Dot product | `a · b` | scalar |

## Comparison Operators

| Operator | Unicode | Name | Example |
|----------|---------|------|---------|
| `=` | | Equality | `x = y` |
| `==` | | Equality (alt) | `x == y` |
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
| `implies` | `→` `⇒` `⟹` | Implication | `P → Q` |
| `iff` | `↔` `⇔` `⟺` | Biconditional | `P ↔ Q` |
| `&&` | | Conjunction (alt) | `P && Q` |
| `\|\|` | | Disjunction (alt) | `P \|\| Q` |

> **Note:** All Unicode variants for implication (`→`, `⇒`, `⟹`) and biconditional (`↔`, `⇔`, `⟺`) are equivalent.

## Postfix Operators

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `!` | Factorial | `5!` | `120` |
| `ᵀ` | Transpose | `Aᵀ` | transposed matrix |
| `†` | Dagger/Adjoint | `A†` | conjugate transpose |
| `′` | Prime | `f′` | derivative notation |
| `″` | Double prime | `f″` | second derivative |
| `‴` | Triple prime | `f‴` | third derivative |
| `⁺` | Superscript plus | `A⁺` | pseudo-inverse |
| `⁻` | Superscript minus | `A⁻` | inverse notation |

## Prefix Operators

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `-` | Negation | `-x` | negated value |
| `∇` | Gradient/Del | `∇f` | gradient of f |
| `∫` | Integral | `∫f` | integral of f |
| `¬` | Logical not | `¬P` | negation of P |

## Big Operators (v0.95)

Kleis supports big operator syntax for summations, products, integrals, and limits:

| Operator | Name | Syntax | Translates to |
|----------|------|--------|---------------|
| `Σ` | Summation | `Σ(from, to, body)` | `sum_bounds(body, from, to)` |
| `Π` | Product | `Π(from, to, body)` | `prod_bounds(body, from, to)` |
| `∫` | Integral | `∫(lower, upper, body, var)` | `int_bounds(body, lower, upper, var)` |
| `lim` | Limit | `lim(var, target, body)` | `lim(body, var, target)` |

### Examples

```kleis
// Sum from i=1 to n
Σ(1, n, i^2)

// Product from k=1 to 5
Π(1, 5, k)

// Integral from 0 to 1
∫(0, 1, x^2, x)

// Limit as x approaches 0
lim(x, 0, sin(x)/x)
```

## Custom Mathematical Operators

Kleis recognizes many Unicode mathematical symbols as custom binary operators:

| Category | Operators |
|----------|-----------|
| **Products** | `•` (bullet), `∘` (composition), `⊗` (tensor), `⊙` (Hadamard) |
| **Sums** | `⊕` (direct sum), `⊞` (squared plus), `⨁` (n-ary) |
| **Set-like** | `∪` (union), `∩` (intersection), `⊔` (join), `⊓` (meet) |
| **Other** | `⊛`, `⊘`, `⊚`, `⊝`, `⊟`, `⊠`, `⊡`, `△`, `▽` |

> **Note:** These are parsed as operators but their semantics depend on context. Define their behavior in structures.

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
| 1 | `↔` `⇔` `⟺` (biconditional) | Left |
| 2 | `→` `⇒` `⟹` (implication) | Right |
| 3 | `∨` `or` `\|\|` | Left |
| 4 | `∧` `and` `&&` | Left |
| 5 | `¬` `not` | Prefix |
| 6 | `=` `≠` `<` `>` `≤` `≥` | Non-associative |
| 7 | `+` `-` | Left |
| 8 | `*` `/` `×` `·` | Left |
| 9 | `^` (power) | Right |
| 10 | `-` (unary negation) | Prefix |
| 11 | `!` `ᵀ` `†` `′` (postfix) | Postfix |
| 12 | Function application | Left |

## Examples

### Arithmetic Precedence

```kleis
define ex1 = 2 + 3 * 4        // 14 (not 20)
define ex2 = (2 + 3) * 4      // 20
define ex3 = 2 ^ 3 ^ 2        // 512 (= 2^9, right associative)
define neg_sq(x) = -x^2       // -(x^2), not (-x)^2
```

### Logical Precedence

```kleis
define logic1(P, Q, R) = P ∧ Q ∨ R        // (P ∧ Q) ∨ R
define logic2(P, Q, R) = P → Q → R        // P → (Q → R)
define logic3(P, Q) = ¬P ∧ Q              // (¬P) ∧ Q
```

### Postfix with Power

```kleis
n!^2        // (n!)^2 - factorial first, then square
Aᵀᵀ         // (Aᵀ)ᵀ = A - transpose twice
```

### Type Expressions

```kleis
ℝ → ℝ → ℝ        // ℝ → (ℝ → ℝ) (curried binary function)
(ℝ → ℝ) → ℝ      // Higher-order: takes function, returns value
ℝ × ℝ → ℝ        // Takes pair, returns value
```

## See Also

- [Built-in Functions](./builtin-functions.md) - List, string, matrix operations
- [LAPACK Functions](./lapack.md) - Numerical linear algebra
- [Complex Numbers](../chapters/14-complex-numbers.md) - Complex number operations
