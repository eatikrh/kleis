# Pattern Matching

## The Power of Match

Pattern matching is one of Kleis's most powerful features. It lets you destructure data and handle different cases elegantly:

```kleis
define describe(n : ℤ) : String =
    match n {
        0 => "zero"
        1 => "one"
        _ => "many"
    }
```

## Basic Patterns

### Literal Patterns

Match exact values:

```kleis
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
match point {
    Point(x, y) => x + y  // x and y are bound
}
```

### Wildcard Pattern

The underscore `_` matches anything:

```kleis
match pair {
    (_, 0) => "second is zero"
    (0, _) => "first is zero"
    _ => "neither is zero"
}
```

## Nested Patterns

Patterns can be nested arbitrarily:

```kleis
match tree {
    Leaf(v) => v
    Node(Leaf(l), v, Leaf(r)) => l + v + r  // Both children are leaves
    Node(left, v, right) => v + sum(left) + sum(right)
}
```

## Guards

Add conditions to patterns with `if`:

```kleis
match n {
    x if x < 0 => "negative"
    x if x > 0 => "positive"
    _ => "zero"
}
```

## As-Patterns

Bind the whole match while also destructuring:

```kleis
match list {
    Cons(h, t) as whole => 
        if h > 10 then whole
        else t
}
```

## Pattern Matching in Let

Destructure directly in let bindings:

```kleis
let Point(x, y) = origin in
    x^2 + y^2

let (first, second, _) = triple in
    first + second
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
match opt {
    Some(x) => x
    // Missing: None case!
}

// ✓ Complete
match opt {
    Some(x) => x
    None => 0
}
```

## Real-World Example: Symbolic Differentiation

Pattern matching makes symbolic math elegant:

```kleis
define diff(expr : Expr, var : String) : Expr =
    match expr {
        Const(_) => Const(0)
        
        Var(name) => 
            if name = var then Const(1)
            else Const(0)
        
        Add(f, g) => 
            Add(diff(f, var), diff(g, var))
        
        Mul(f, g) =>  // Product rule
            Add(Mul(diff(f, var), g), 
                Mul(f, diff(g, var)))
        
        Neg(f) => 
            Neg(diff(f, var))
    }
```

## What's Next?

Learn about let bindings for local definitions!

→ [Next: Let Bindings](./06-let-bindings.md)
