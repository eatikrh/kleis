# Conditionals

## If-Then-Else

The basic conditional expression:

```kleis
if condition then value1 else value2
```

Examples:

```kleis
if x > 0 then "positive" else "non-positive"

if n = 0 then 1 else n * factorial(n - 1)

define abs(x : ℝ) : ℝ = if x ≥ 0 then x else -x
```

## Conditionals Are Expressions

In Kleis, `if-then-else` is an expression that returns a value:

```kleis
let result = if x > 0 then x else -x in
    result * 2

// Both branches must have compatible types!
if True then 42 else "hello"  // ❌ Type error!
```

## Nested Conditionals

```kleis
define sign(x : ℝ) : ℤ =
    if x > 0 then 1
    else if x < 0 then -1
    else 0

define grade(score : ℕ) : String =
    if score ≥ 90 then "A"
    else if score ≥ 80 then "B"
    else if score ≥ 70 then "C"
    else if score ≥ 60 then "D"
    else "F"
```

## Guards vs If-Then-Else

Pattern matching with guards is often cleaner:

```kleis
// With if-then-else
define classify(n : ℤ) : String =
    if n < 0 then "negative"
    else if n = 0 then "zero"
    else "positive"

// With pattern matching and guards
define classify(n : ℤ) : String =
    match n {
        x if x < 0 => "negative"
        0 => "zero"
        _ => "positive"
    }
```

## Piecewise Functions

Mathematicians love piecewise definitions:

```kleis
// Absolute value
define abs(x : ℝ) : ℝ =
    if x ≥ 0 then x else -x

// Heaviside step function
define heaviside(x : ℝ) : ℝ =
    if x < 0 then 0
    else if x = 0 then 0.5
    else 1

// Piecewise polynomial
define f(x : ℝ) : ℝ =
    if x < 0 then x^2
    else if x < 1 then x
    else 2 - x
```

## Boolean Expressions

Conditions can be complex:

```kleis
if x > 0 ∧ y > 0 then "first quadrant"
else if x < 0 ∧ y > 0 then "second quadrant"
else if x < 0 ∧ y < 0 then "third quadrant"
else if x > 0 ∧ y < 0 then "fourth quadrant"
else "on an axis"
```

## Short-Circuit Evaluation

Kleis uses short-circuit evaluation for `∧` and `∨`:

```kleis
// If x = 0, division is never evaluated
if x ≠ 0 ∧ y/x > 1 then "big ratio" else "safe"
```

## What's Next?

Learn about structures for defining mathematical objects!

→ [Next: Structures](./09-structures.md)
