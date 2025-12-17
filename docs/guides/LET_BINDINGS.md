# Let Bindings in Kleis

**Date:** December 17, 2025  
**Status:** Implemented (v0.7 grammar)

## Syntax

```kleis
let <name> [ : <type> ] = <value> in <body>
```

**Examples:**
```kleis
let x = 5 in x + x                    // Simple binding
let x : ℝ = 5 in x^2                  // With type annotation
let a = 1 in let b = 2 in a + b       // Nested bindings
```

---

## What Does `in` Mean?

The `in` keyword separates the **binding** from the **scope where that binding is available**.

```
let x = 5 in x + x
│        │   └───── body: x is available here
│        └──── "in" = "within the following expression"
└──────────────── binding: x = 5
```

**Read it as:** "Let x equal 5, **in** (the expression) x + x"

---

## Scoping Rules

### Basic Scope

The bound variable is only available **after** the `in` keyword:

```kleis
let x = 5 in x + x
             ─────
             └── x is in scope here (evaluates to 10)
```

### Nested Let Bindings

Each `in` introduces a new scope that includes all outer bindings:

```kleis
let x = 5 in let y = 3 in x * y
│            │            └───── both x and y in scope
│            └──── y = 3 (x is also in scope here)
└──────────────── x = 5
```

**Evaluation:**
```
let x = 5 in let y = 3 in x * y
→ let y = 3 in 5 * y            // substitute x = 5
→ 5 * 3                         // substitute y = 3
→ 15
```

### Variable Shadowing

Inner bindings can shadow outer ones:

```kleis
let x = 5 in let x = 10 in x    // evaluates to 10, not 5
```

The inner `x = 10` shadows the outer `x = 5` within its scope.

---

## Pure Substitution Semantics

`let x = e₁ in e₂` means: **substitute e₁ for every occurrence of x in e₂**

This is **pure functional semantics** - there's no mutation, just substitution.

| Expression | Substitution | Result |
|------------|--------------|--------|
| `let x = 5 in x + x` | Replace x with 5 | `5 + 5 = 10` |
| `let x = 2 in x^3` | Replace x with 2 | `2^3 = 8` |
| `let r = 3 in π * r^2` | Replace r with 3 | `π * 9 ≈ 28.27` |

---

## Mathematical Equivalent

Let bindings are equivalent to mathematical "where" notation, just with the definition first:

| Kleis (definition first) | Math (definition last) |
|--------------------------|------------------------|
| `let x = 5 in x^2` | x² **where** x = 5 |
| `let r = 3 in π * r^2` | πr² **where** r = 3 |
| `let a = 1 in let b = 2 in a + b` | a + b **where** a = 1, b = 2 |

The difference is just **order** - Kleis puts the binding first, math puts it last.

---

## Type Annotations (v0.7)

You can optionally annotate the type of the bound variable:

```kleis
let x : ℝ = 5 in x^2                         // Simple type
let v : Vector(3) = data in norm(v)          // Parametric type
let f : ℝ → ℝ = abs in f(x)                  // Function type
```

This is useful for:
- **Documentation** - making the intended type explicit
- **Disambiguation** - when the type isn't obvious from context
- **Type checking** - verifying the value has the expected type

---

## The Body Can Be Any Expression

After `in`, you can have **any valid expression** - as elaborate as you want.

The grammar makes this clear:

```ebnf
letBinding ::= "let" identifier "=" expression "in" expression
                                                     └────────┘
                                                     ANY expression!
```

**Simple arithmetic:**
```kleis
let x = 5 in x + x
```

**Nested let bindings:**
```kleis
let x = 5 in 
    let y = 3 in 
        let z = 2 in 
            x * y + z
```

**Conditionals:**
```kleis
let x = input in 
    if x > 0 then x * 2 else negate(x)
```

**Pattern matching:**
```kleis
let result = compute(data) in
    match result {
        Ok(value) => value * 2
        Err(_) => 0
    }
```

**Complex mathematical expressions:**
```kleis
let a = coefficients[0] in
    let b = coefficients[1] in
        let c = coefficients[2] in
            let discriminant = b^2 - 4*a*c in
                if discriminant >= 0 then
                    (negate(b) + sqrt(discriminant)) / (2 * a)
                else
                    NaN
```

It's **expressions all the way down** - the body of a let is just another expression, which can itself contain more lets, conditionals, match expressions, or anything else.

---

## Examples

### Simple Calculations

```kleis
// Quadratic formula: compute discriminant
let discriminant = b^2 - 4*a*c in sqrt(discriminant)

// Circle area
let r = 5 in π * r^2
```

### Named Intermediate Values

```kleis
define quadratic(a, b, c, x) = 
    let x2 = x * x in 
    a * x2 + b * x + c
```

### With Pattern Matching

```kleis
define safe_divide(x, y) =
    let result = if y == 0 then None else Some(x / y) in
    match result {
        None => 0
        Some(v) => v
    }
```

### Nested Computations

```kleis
define euclidean_distance(x1, y1, x2, y2) =
    let dx = x2 - x1 in
    let dy = y2 - y1 in
    sqrt(dx^2 + dy^2)
```

---

## Z3 Integration

Let bindings are translated to Z3 with proper variable scoping:

```kleis
let x = 5 in x + 3
```

Becomes (conceptually):
```z3
(let ((x 5)) (+ x 3))   ; Z3 SMT-LIB syntax
```

The Z3 backend:
1. Evaluates the value expression
2. Extends the variable context with the new binding
3. Evaluates the body in the extended context

---

## Comparison to Other Languages

| Language | Syntax | Notes |
|----------|--------|-------|
| **Kleis** | `let x = 5 in x + x` | ML/Haskell style |
| **Haskell** | `let x = 5 in x + x` | Identical |
| **OCaml** | `let x = 5 in x + x` | Identical |
| **Rust** | `{ let x = 5; x + x }` | Statement, not expression |
| **Python** | `(x := 5) + x` | Walrus operator (different semantics) |
| **Math** | `x + x where x = 5` | Definition comes last |

Kleis follows the **ML/Haskell tradition** of pure, expression-based let bindings.

---

## Grammar (EBNF)

From `docs/grammar/kleis_grammar_v07.ebnf`:

```ebnf
letBinding
    ::= "let" identifier [ typeAnnotation ] "=" expression "in" expression
    ;

typeAnnotation ::= ":" type ;
```

---

## See Also

- [Kleis Grammar v0.7](../grammar/kleis_grammar_v07.md) - Full grammar specification
- [Pattern Matching](./PATTERN_MATCHING.md) - Using match with let bindings
- [Conditionals](./CONDITIONALS.md) - if/then/else expressions

