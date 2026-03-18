# Pattern Matching

## The Power of Match

Pattern matching is one of Kleis's most powerful features. It lets you destructure data and handle different cases elegantly:

```kleis
define describe(n) =
    match n {
        0 => 0
        1 => 1
        _ => 2
    }
```

## Basic Patterns

### Literal Patterns

Match exact values:

```kleis
define describe_literal(x) =
    match x {
        0 => "zero"
        1 => "one"
        42 => "the answer"
        _ => "something else"
    }
```

### Variable Patterns

Bind matched values to names:

```kleis
define sum_point(point) =
    match point {
        Point(x, y) => x + y
    }
```

### Wildcard Pattern

The underscore `_` matches anything:

```kleis
define describe_pair(pair) =
    match pair {
        (_, 0) => "second is zero"
        (0, _) => "first is zero"
        _ => "neither is zero"
    }
```

## Nested Patterns

Patterns can be nested arbitrarily:

```kleis
define sum_tree(tree) =
    match tree {
        Leaf(v) => v
        Node(Leaf(l), v, Leaf(r)) => l + v + r
        Node(left, v, right) => v + sum_tree(left) + sum_tree(right)
    }
```

## Guards

Add conditions to patterns with `if`:

```kleis
define sign(n) =
    match n {
        x if x < 0 => "negative"
        x if x > 0 => "positive"
        _ => "zero"
    }
```

## As-Patterns

Bind the whole match while also destructuring:

```kleis
define filter_head(list) =
    match list {
        Cons(h, t) as whole => 
            if h > 10 then whole
            else t
        Nil => Nil
    }
```

## Pattern Matching in Let

Destructure directly in let bindings:

```kleis
define distance_squared(origin) =
    let Point(x, y) = origin in x^2 + y^2

define sum_first_two(triple) =
    let (first, second, _) = triple in first + second
```

## Pattern Matching in Function Parameters

With lambda expressions now available, you can combine them with match:

```kleis
// Pattern matching with lambdas
define fst = λ pair . match pair { (a, _) => a }
define snd = λ pair . match pair { (_, b) => b }
```

**Alternative workaround:**

```kleis
define fst(pair) = 
    match pair {
        (a, _) => a
    }
```

## Exhaustiveness

Kleis checks that your patterns cover all cases:

```kleis
// ⚠️ Warning: non-exhaustive patterns
define incomplete(opt) =
    match opt {
        Some(x) => x
    }

// ✓ Complete
define complete(opt) =
    match opt {
        Some(x) => x
        None => 0
    }
```

## Real-World Example: Symbolic Differentiation

Pattern matching makes symbolic math elegant:

```kleis
define diff(expr, var) =
    match expr {
        Const(_) => Const(0)
        
        Var(name) => 
            if name = var then Const(1)
            else Const(0)
        
        Add(f, g) => 
            Add(diff(f, var), diff(g, var))
        
        Mul(f, g) =>
            Add(Mul(diff(f, var), g), 
                Mul(f, diff(g, var)))
        
        Neg(f) => 
            Neg(diff(f, var))
    }
```

> **Note:** This `diff` function **computes** derivatives by pattern matching on expression trees. Kleis also provides `D(f, x)` and `Dt(f, x)` operations in `stdlib/calculus.kleis` for **verifying** derivative properties with Z3. See [Applications: Symbolic Differentiation](./13-applications.md#symbolic-differentiation) for a detailed comparison.

## What's Next?

Learn about let bindings for local definitions!

→ [Next: Let Bindings](./06-let-bindings.md)
