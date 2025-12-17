# Chapter 9: Conditionals

[← Previous: Quantifiers](08-quantifiers-logic.md) | [Back to Contents](../index.md) | [Next: Structures →](10-structures.md)

---

## If-Then-Else

The basic conditional expression:

```kleis
if condition then consequent else alternative
```

### Basic Examples

```kleis
>>> if True then 1 else 0
1

>>> if False then 1 else 0
0

>>> if 5 > 3 then "yes" else "no"
"yes"
```

### As an Expression

Unlike many languages, `if` is an **expression** that returns a value:

```kleis
let result = if x > 0 then x else -x in result

define abs(x) = if x < 0 then -x else x

define sign(x) =
    if x > 0 then 1
    else if x < 0 then -1
    else 0
```

---

## Nested Conditionals

```kleis
define classify(n) =
    if n < 0 then "negative"
    else if n = 0 then "zero"
    else "positive"

define grade(score) =
    if score >= 90 then "A"
    else if score >= 80 then "B"
    else if score >= 70 then "C"
    else if score >= 60 then "D"
    else "F"
```

---

## Conditionals vs Pattern Matching

Often, pattern matching is cleaner:

```kleis
// With if-then-else
define describe_option(opt) =
    if opt = None then "nothing"
    else "something"

// With match (better!)
define describe_option(opt) =
    match opt {
        None => "nothing"
        Some(_) => "something"
    }
```

### When to Use Each

**Use `if-then-else` for:**
- Numeric comparisons: `if x > 0 then ...`
- Boolean expressions: `if is_valid(x) then ...`
- Simple two-way branches

**Use `match` for:**
- Destructuring data types
- Multiple cases
- Extracting values from constructors

---

## Combining with Other Features

### With Let

```kleis
define quadratic_roots(a, b, c) =
    let disc = b^2 - 4*a*c in
    if disc < 0 then None
    else if disc = 0 then Some(-b / (2*a))
    else Some((-b + sqrt(disc)) / (2*a))
```

### With Pattern Matching

```kleis
define safe_div(a, b) =
    if b = 0 then None
    else Some(a / b)

define process_result(result) =
    match result {
        None => 0
        Some(x) => if x > 100 then 100 else x
    }
```

---

## Guards in Functions

Kleis supports guards for function definitions:

```kleis
define abs(x)
    | x < 0  = -x
    | x >= 0 = x

define fib(n)
    | n = 0 = 0
    | n = 1 = 1
    | n > 1 = fib(n-1) + fib(n-2)
```

Guards are checked top-to-bottom. First matching guard wins.

---

## Boolean Expressions

### Short-Circuit Evaluation

`∧` and `∨` short-circuit:

```kleis
// If first is False, second is not evaluated
False ∧ expensive_computation()

// If first is True, second is not evaluated
True ∨ expensive_computation()
```

### Common Patterns

```kleis
// Check before accessing
if is_some(opt) then get_value(opt) else default

// Combine conditions
if x > 0 ∧ x < 100 then "in range" else "out of range"

// Negate
if ¬is_empty(list) then head(list) else None
```

---

## Exercises

1. **Write** `max(a, b)` using if-then-else

2. **Write** `clamp(value, min, max)` that keeps value in range [min, max]

3. **Write** `fizzbuzz(n)` that returns:
   - "fizzbuzz" if n divisible by both 3 and 5
   - "fizz" if divisible by 3
   - "buzz" if divisible by 5
   - the number otherwise

4. **Rewrite** this using match:
   ```kleis
   if b = True then 1 else 0
   ```

<details>
<summary>Solutions</summary>

```kleis
// 1.
define max(a, b) = if a > b then a else b

// 2.
define clamp(value, min, max) =
    if value < min then min
    else if value > max then max
    else value

// 3.
define fizzbuzz(n) =
    if mod(n, 15) = 0 then "fizzbuzz"
    else if mod(n, 3) = 0 then "fizz"
    else if mod(n, 5) = 0 then "buzz"
    else n

// 4.
match b {
    True => 1
    False => 0
}
```

</details>

---

## Summary

- `if condition then x else y` is an expression
- Both branches must have compatible types
- Nest conditionals with `else if`
- Use pattern matching when destructuring data
- Guards provide another way to express conditions

---

[← Previous: Quantifiers](08-quantifiers-logic.md) | [Back to Contents](../index.md) | [Next: Structures →](10-structures.md)

