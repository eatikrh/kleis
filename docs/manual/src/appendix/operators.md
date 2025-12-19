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
| `implies` | `→` `⇒` `⟹` | Implication | `P → Q` |
| `iff` | `↔` `⇔` `⟺` | Biconditional | `P ↔ Q` |

> **Note:** All three Unicode variants for implication and biconditional are 
> equivalent. Use whichever matches your notation conventions.

## Set Operators

> **Note:** Set operators are parsed as custom operators but don't have special semantic meaning yet. Use function-call syntax for set operations: `member(x, S)`, `union(A, B)`, etc.

| Operator | Unicode | Name | Status |
|----------|---------|------|--------|
| `∪` | | Union | Custom operator (no special semantics) |
| `∩` | | Intersection | Custom operator (no special semantics) |
| `∈` | | Membership | ❌ Not implemented - use `member(x, S)` |
| `∉` | | Non-membership | ❌ Not implemented |
| `⊆` | | Subset | ❌ Not implemented - use `subset(A, B)` |
| `⊇` | | Superset | ❌ Not implemented |

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
| 3 | `∨` `or` | Left |
| 4 | `∧` `and` | Left |
| 5 | `¬` `not` | Prefix |
| 6 | `=` `≠` `<` `>` `≤` `≥` | Non-associative |
| 7 | `+` `-` | Left |
| 8 | `*` `/` `×` `·` | Left |
| 9 | `^` (power) | Right |
| 10 | `-` (unary negation) | Prefix |
| 11 | `!` `ᵀ` `†` (postfix) | Postfix |
| 12 | Function application | Left |

> **Note:** Field access (`.`) is NOT implemented. Use function-call syntax: `field(object)` instead of `object.field`.

## Examples

### Arithmetic

```kleis
define ex1 = 2 + 3 * 4        // 14 (not 20)
define ex2 = (2 + 3) * 4      // 20
define ex3 = 2 ^ 3 ^ 2        // 512 (= 2^9, right associative)
define neg_sq(x) = -x^2       // -(x^2), not (-x)^2
```

### Logical

```kleis
define logic1(P, Q, R) = P ∧ Q ∨ R        // (P ∧ Q) ∨ R
define logic2(P, Q, R) = P → Q → R        // P → (Q → R) (right associative)
define logic3(P, Q) = ¬P ∧ Q              // (¬P) ∧ Q
```

### Type Expressions

```kleis
ℝ → ℝ → ℝ        // ℝ → (ℝ → ℝ) (curried binary function)
(ℝ → ℝ) → ℝ      // Higher-order: takes function, returns value
ℝ × ℝ → ℝ        // Takes pair, returns value
```

## Built-in Functions

### Mathematical Functions

```kleis
sqrt(x)          // Square root
abs(x)           // Absolute value
floor(x)         // Round down
ceil(x)          // Round up
round(x)         // Round to nearest
min(x, y)        // Minimum
max(x, y)        // Maximum
```

### Trigonometric Functions

```kleis
sin(x)   cos(x)   tan(x)
asin(x)  acos(x)  atan(x)
sinh(x)  cosh(x)  tanh(x)
```

### Exponential and Logarithmic

```kleis
exp(x)           // e^x
ln(x)            // Natural logarithm
log(x)           // Base-10 logarithm
log(b, x)        // Logarithm base b
```

### Constants

```kleis
π                // Pi (3.14159...)
e                // Euler's number (2.71828...)
i                // Imaginary unit
```

## Reserved Keywords

The following words are reserved and cannot be used as variable or function names:

### Control Flow

| Keyword | Purpose | Example |
|---------|---------|---------|
| `if` | Conditional start | `if x > 0 then ...` |
| `then` | Conditional consequence | `if P then Q else R` |
| `else` | Conditional alternative | `if P then Q else R` |
| `match` | Pattern matching | `match x { ... }` |
| `let` | Local binding | `let x = 5 in ...` |
| `in` | Binding body | `let x = 5 in x + 1` |

### Functions

| Keyword | Purpose | Example |
|---------|---------|---------|
| `lambda` | Anonymous function (ASCII) | `lambda x . x + 1` |
| `λ` | Anonymous function (Unicode) | `λ x . x + 1` |
| `define` | Function definition | `define f(x) = x^2` |

### Quantifiers

| Keyword | Purpose | Example |
|---------|---------|---------|
| `forall` | Universal quantifier (ASCII) | `forall x . P(x)` |
| `∀` | Universal quantifier (Unicode) | `∀(x : ℝ). x = x` |
| `exists` | Existential quantifier (ASCII) | `exists x . P(x)` |
| `∃` | Existential quantifier (Unicode) | `∃(x : ℝ). x > 0` |

### Logical Operators (keyword form)

| Keyword | Purpose | Example |
|---------|---------|---------|
| `and` | Logical conjunction | `P and Q` |
| `or` | Logical disjunction | `P or Q` |
| `not` | Logical negation | `not P` |

### Definitions

| Keyword | Purpose | Example |
|---------|---------|---------|
| `structure` | Algebraic structure | `structure Group(G) { ... }` |
| `implements` | Structure implementation | `implements Group(ℤ) { ... }` |
| `data` | Algebraic data type | `data Option(T) = Some(T) \| None` |
| `type` | Type alias | `type Point = (ℝ, ℝ)` |
| `operation` | Operation declaration | `operation add : G × G → G` |
| `element` | Element declaration | `element zero : G` |
| `axiom` | Axiom declaration | `axiom identity : ...` |

### Modifiers and Clauses

| Keyword | Purpose | Example |
|---------|---------|---------|
| `import` | File import | `import "stdlib/prelude.kleis"` |
| `over` | Type constraint | `structure V over Field(F) { ... }` |
| `extends` | Structure extension | `structure Ring extends Group { ... }` |
| `as` | Pattern alias | `match x { y as Some(_) => ... }` |

### Commands

| Keyword | Purpose | Example |
|---------|---------|---------|
| `verify` | Verification directive | `:verify P ∧ Q` |

> **Note:** Using a reserved keyword as a variable name will cause a parse error.
> For example, `let lambda = 5` fails because `lambda` is reserved for anonymous functions.
