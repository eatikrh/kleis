# Chapter 6: Pattern Matching

[← Previous: Algebraic Types](05-algebraic-types.md) | [Back to Contents](../index.md) | [Next: Let Bindings →](07-let-bindings.md)

---

## What is Pattern Matching?

Pattern matching lets you **destructure** data and handle different cases elegantly.

Instead of:
```
if x is None then ... else if x is Some then ...
```

You write:
```kleis
match x {
    None => ...
    Some(value) => ...
}
```

---

## Basic Syntax

```kleis
match expression {
    pattern1 => result1
    pattern2 => result2
    ...
}
```

The expression is compared against each pattern top-to-bottom. The first match wins.

---

## Matching Constructors

### Bool

```kleis
define describe_bool(b) =
    match b {
        True => "yes"
        False => "no"
    }

>>> describe_bool(True)
"yes"
```

### Option

```kleis
define unwrap_or(opt, default) =
    match opt {
        None => default
        Some(x) => x
    }

>>> unwrap_or(Some(5), 0)
5

>>> unwrap_or(None, 0)
0
```

### List

```kleis
define is_empty(xs) =
    match xs {
        Nil => True
        Cons(_, _) => False
    }

define head(xs) =
    match xs {
        Nil => None
        Cons(h, _) => Some(h)
    }

define tail(xs) =
    match xs {
        Nil => Nil
        Cons(_, t) => t
    }
```

---

## Pattern Types

### Variable Patterns

Bind a name to the matched value:

```kleis
match Some(42) {
    Some(x) => x + 1    // x = 42
}
// Result: 43
```

### Wildcard Pattern

`_` matches anything but doesn't bind:

```kleis
match Some(42) {
    Some(_) => "got something"
    None => "got nothing"
}
```

### Constructor Patterns

Match specific constructors:

```kleis
match result {
    Ok(value) => value
    Err(msg) => error(msg)
}
```

### Nested Patterns

Patterns can be nested:

```kleis
match nested_option {
    None => "outer none"
    Some(None) => "inner none"
    Some(Some(x)) => x
}

match list {
    Nil => "empty"
    Cons(_, Nil) => "one element"
    Cons(_, Cons(_, Nil)) => "two elements"
    Cons(_, Cons(_, _)) => "three or more"
}
```

### Constant Patterns

Match literal values:

```kleis
define factorial(n) =
    match n {
        0 => 1
        n => n * factorial(n - 1)
    }

define describe_number(n) =
    match n {
        0 => "zero"
        1 => "one"
        2 => "two"
        _ => "many"
    }
```

### Tuple Patterns

```kleis
match pair {
    (0, y) => y
    (x, 0) => x
    (x, y) => x + y
}
```

---

## Exhaustiveness

Kleis checks that your patterns cover all cases:

```kleis
// Warning: Non-exhaustive patterns
define bad(b) =
    match b {
        True => 1
        // Missing: False!
    }
```

Always cover all constructors, or use a wildcard:

```kleis
define good(b) =
    match b {
        True => 1
        False => 0
    }

// Or with wildcard:
define also_good(b) =
    match b {
        True => 1
        _ => 0
    }
```

---

## Recursive Functions with Match

Pattern matching is perfect for recursive data:

```kleis
define length(xs) =
    match xs {
        Nil => 0
        Cons(_, tail) => 1 + length(tail)
    }

define sum(xs) =
    match xs {
        Nil => 0
        Cons(h, t) => h + sum(t)
    }

define map(f, xs) =
    match xs {
        Nil => Nil
        Cons(h, t) => Cons(f(h), map(f, t))
    }

define filter(pred, xs) =
    match xs {
        Nil => Nil
        Cons(h, t) =>
            if pred(h) then Cons(h, filter(pred, t))
            else filter(pred, t)
    }
```

---

## Trees with Match

```kleis
define tree_size(t) =
    match t {
        Leaf => 0
        Node(_, l, r) => 1 + tree_size(l) + tree_size(r)
    }

define tree_depth(t) =
    match t {
        Leaf => 0
        Node(_, l, r) => 1 + max(tree_depth(l), tree_depth(r))
    }

define tree_contains(t, value) =
    match t {
        Leaf => False
        Node(v, l, r) =>
            if v = value then True
            else tree_contains(l, value) or tree_contains(r, value)
    }
```

---

## Guards (Future)

Pattern matching can include guards for additional conditions:

```kleis
define classify(n) =
    match n {
        n if n < 0 => "negative"
        0 => "zero"
        n if n > 0 => "positive"
    }
```

---

## Common Idioms

### Safe Operations

```kleis
define safe_head(xs) =
    match xs {
        Nil => None
        Cons(h, _) => Some(h)
    }

define safe_div(a, b) =
    match b {
        0 => None
        _ => Some(a / b)
    }
```

### Result Handling

```kleis
define process(result) =
    match result {
        Ok(value) => compute(value)
        Err(e) => log_error(e)
    }
```

### Default Values

```kleis
define get_or_default(opt, default) =
    match opt {
        None => default
        Some(x) => x
    }
```

---

## Exercises

1. **Write** `list_last(xs)` that returns `Some(last element)` or `None`

2. **Write** `list_reverse(xs)` using pattern matching

3. **Write** `option_map(f, opt)` that applies f if Some, returns None otherwise

4. **Write** `tree_leaves(t)` that counts the number of Leaf nodes

<details>
<summary>Solutions</summary>

```kleis
// 1.
define list_last(xs) =
    match xs {
        Nil => None
        Cons(h, Nil) => Some(h)
        Cons(_, t) => list_last(t)
    }

// 2. Using a helper function (or lambda when available)
define reverse_helper(acc, xs) =
    match xs {
        Nil => acc
        Cons(h, t) => reverse_helper(Cons(h, acc), t)
    }
define list_reverse(xs) = reverse_helper(Nil, xs)

// With lambda (coming soon!):
// define list_reverse(xs) =
//     let aux = \acc xs. match xs { Nil => acc | Cons(h,t) => aux(Cons(h,acc), t) }
//     in aux(Nil, xs)

// 3.
define option_map(f, opt) =
    match opt {
        None => None
        Some(x) => Some(f(x))
    }

// 4.
define tree_leaves(t) =
    match t {
        Leaf => 1
        Node(_, l, r) => tree_leaves(l) + tree_leaves(r)
    }
```

</details>

---

## Summary

- `match expr { patterns }` destructures data
- Patterns: variables, wildcards `_`, constructors, constants, tuples
- Patterns can be nested: `Some(Some(x))`
- Kleis checks exhaustiveness
- Perfect for recursive data structures

---

[← Previous: Algebraic Types](05-algebraic-types.md) | [Back to Contents](../index.md) | [Next: Let Bindings →](07-let-bindings.md)

