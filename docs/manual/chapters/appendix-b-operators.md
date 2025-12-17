# Appendix B: Operator Precedence

[← Back to Contents](../index.md)

---

## Precedence Table

Operators are listed from **highest precedence** (binds tightest) to **lowest** (binds loosest).

| Level | Operators | Description | Associativity |
|-------|-----------|-------------|---------------|
| **1** | `^` | Exponentiation | Right |
| **2** | `*` `/` | Multiplication, Division | Left |
| **3** | `+` `-` | Addition, Subtraction | Left |
| **4** | `=` `≠` `<` `>` `≤` `≥` | Comparisons | Non-associative |
| **5** | `¬` `not` | Logical NOT | Prefix |
| **6** | `∧` `and` | Logical AND | Left |
| **7** | `∨` `or` | Logical OR | Left |
| **8** | `⟹` `=>` | Implication | Right |
| **9** | `:` | Type ascription | Non-associative |

---

## Examples

### Arithmetic

```kleis
2 + 3 * 4       // = 2 + (3 * 4) = 14
2 ^ 3 ^ 2       // = 2 ^ (3 ^ 2) = 2^9 = 512 (right-assoc)
(2 + 3) * 4     // = 20 (parentheses override)
```

### Logic

```kleis
¬p ∧ q          // = (¬p) ∧ q
p ∧ q ∨ r       // = (p ∧ q) ∨ r
p ∨ q ⟹ r      // = (p ∨ q) ⟹ r
```

### Mixed

```kleis
x + 1 = y * 2   // = (x + 1) = (y * 2)
a < b ∧ b < c   // = (a < b) ∧ (b < c)
f(x) : ℝ        // = (f(x)) : ℝ
```

---

## Associativity Explained

### Left-Associative

Operations group from left to right:

```kleis
a - b - c = (a - b) - c

10 - 3 - 2 = (10 - 3) - 2 = 7 - 2 = 5
// NOT: 10 - (3 - 2) = 10 - 1 = 9
```

### Right-Associative

Operations group from right to left:

```kleis
a ^ b ^ c = a ^ (b ^ c)

2 ^ 3 ^ 2 = 2 ^ (3 ^ 2) = 2 ^ 9 = 512
// NOT: (2 ^ 3) ^ 2 = 8 ^ 2 = 64
```

### Non-Associative

Cannot chain without parentheses:

```kleis
a = b = c       // Error! Use: (a = b) ∧ (b = c)
a < b < c       // Error! Use: (a < b) ∧ (b < c)
```

---

## Function Application

Function application has **highest precedence**:

```kleis
f x + y         // = (f(x)) + y, NOT f(x + y)
sin x * 2       // = (sin(x)) * 2
sqrt 16 + 9     // = (sqrt(16)) + 9 = 4 + 9 = 13
```

**Best practice:** Always use parentheses for function calls:

```kleis
f(x) + y        // Clear
sin(x) * 2      // Clear
sqrt(16 + 9)    // sqrt(25) = 5
```

---

## Type Ascription

Type ascription has the **lowest precedence**:

```kleis
a + b : ℝ       // = (a + b) : ℝ
f(x) : ℤ → ℤ    // = (f(x)) : (ℤ → ℤ)
```

---

## Custom Operators

User-defined operators follow these rules:

| Operator | Precedence |
|----------|------------|
| `(⊕)` etc. (additive-looking) | Same as `+` |
| `(⊗)` etc. (multiplicative-looking) | Same as `*` |
| `(∘)` (composition) | High precedence |

---

## Tips

### 1. When in Doubt, Use Parentheses

```kleis
// Ambiguous to readers
p ∧ q ∨ r ⟹ s

// Clear
((p ∧ q) ∨ r) ⟹ s
```

### 2. Break Complex Expressions

```kleis
// Hard to read
a + b * c ^ d - e / f

// Better with let bindings
let power = c ^ d in
let product = b * power in
let quotient = e / f in
a + product - quotient
```

### 3. Match Mathematical Convention

```kleis
// Math: ax² + bx + c
a * x^2 + b * x + c     // Kleis respects standard precedence
```

---

[← Back to Contents](../index.md)

