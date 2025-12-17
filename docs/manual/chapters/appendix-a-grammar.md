# Appendix A: Grammar Reference

[← Back to Contents](../index.md)

---

## Overview

This appendix provides a quick reference to Kleis syntax. For the complete formal grammar, see `docs/grammar/kleis_grammar_v07.ebnf`.

---

## Expressions

### Literals

```kleis
42              // Integer
3.14            // Float
"hello"         // String (rarely used)
True, False     // Booleans
```

### Variables and Objects

```kleis
x               // Simple identifier
myVariable      // camelCase
my_var          // snake_case
α, β, γ         // Greek letters
x₁, x₂          // Subscripts
```

### Arithmetic

```kleis
a + b           // Addition
a - b           // Subtraction
a * b           // Multiplication
a / b           // Division
a ^ b           // Exponentiation (right-associative)
-a              // Negation
```

### Comparisons

```kleis
a = b           // Equals
a ≠ b           // Not equals (or: a != b)
a < b           // Less than
a > b           // Greater than
a ≤ b           // Less or equal (or: a <= b)
a ≥ b           // Greater or equal (or: a >= b)
```

### Logic

```kleis
p ∧ q           // And (or: p and q)
p ∨ q           // Or (or: p or q)
¬p              // Not (or: not p)
p ⟹ q          // Implies (or: p => q)
```

### Function Calls

```kleis
f(x)            // Single argument
f(x, y)         // Multiple arguments
f(g(x))         // Nested calls
(+)(a, b)       // Operator as function
```

### Let Bindings

```kleis
let x = 5 in x + x          // Basic
let x : ℝ = 5 in x^2        // With type annotation
let a = 1 in let b = 2 in a + b  // Nested
```

### Type Ascription

```kleis
x : ℝ                       // Variable
(a + b) : ℝ                 // Expression
sqrt(x) : ℝ                 // Function result
```

### Conditionals

```kleis
if condition then expr1 else expr2
```

### Pattern Matching

```kleis
match expr {
    pattern1 => result1
    pattern2 => result2
    ...
}
```

### Quantifiers

```kleis
∀(x : T). body              // Universal (or: forall)
∃(x : T). body              // Existential (or: exists)
∀(x y z : T). body          // Multiple variables
```

### Lambda Expressions

```kleis
\x. body                    // Single parameter
\x y. body                  // Multiple parameters
λx. body                    // Unicode lambda
```

### Lists

```kleis
[]                          // Empty list
[1, 2, 3]                   // List literal
Cons(x, xs)                 // Constructor form
```

---

## Definitions

### Functions

```kleis
define name(params) = body
define name(x : T) : R = body       // With types
define (op)(a, b) = body            // Operator
```

### Data Types

```kleis
data TypeName = Constructor1 | Constructor2 | ...
data Option(T) = None | Some(T)     // Parametric
```

### Structures

```kleis
structure Name(TypeParams) {
    operation opName : Type
    axiom axiomName : Expression
}

structure Name(T) extends Parent(T) {
    // Additional operations/axioms
}
```

### Implementations

```kleis
implements Structure(Type) {
    define operation = implementation
}

implements Structure(Type) where Constraint(Type) {
    ...
}
```

---

## Patterns

```kleis
x               // Variable (binds)
_               // Wildcard (ignores)
42              // Constant
True, False     // Constructors (nullary)
Some(x)         // Constructor with binding
Cons(h, t)      // Multiple bindings
Some(Some(x))   // Nested
(a, b)          // Tuple
```

---

## Types

### Basic Types

```kleis
ℝ               // Real numbers (or: Real)
ℤ               // Integers (or: Int)
ℕ               // Natural numbers (or: Nat)
Bool            // Booleans
```

### Function Types

```kleis
T → U           // Function from T to U
ℝ → ℝ           // Real to real
ℤ → Bool        // Integer to boolean
ℝ → ℝ → ℝ       // Two reals to real (= ℝ → (ℝ → ℝ), right-assoc)
(ℝ → ℝ) → ℝ     // Function to real (higher-order)
(A → B) → List(A) → List(B)  // Generic higher-order
```

### Compound Types

```kleis
(T, U)          // Tuple/product type
List(T)         // Parametric type
Option(T)       // Optional value
Vector(3)       // Type with value parameter
Matrix(m, n, F) // Multiple parameters
```

---

## Comments

```kleis
// Line comment

/* Block comment
   can span multiple
   lines */
```

---

## Operator Precedence

From highest to lowest:

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (highest) | `^` | Right |
| 2 | `*`, `/` | Left |
| 3 | `+`, `-` | Left |
| 4 | `=`, `≠`, `<`, `>`, `≤`, `≥` | None |
| 5 | `¬` | Prefix |
| 6 | `∧` | Left |
| 7 | `∨` | Left |
| 8 (lowest) | `⟹` | Right |

---

## Reserved Keywords

```
data define structure implements extends where
axiom operation theorem let in if then else
match forall exists True False None Some
Nil Cons
```

---

[← Back to Contents](../index.md)

