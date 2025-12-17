# Algebraic Data Types

## What Are ADTs?

Algebraic Data Types (ADTs) let you define custom data structures by combining simpler types. There are two main kinds:

- **Product types** — "this AND that" (records, tuples)
- **Sum types** — "this OR that" (variants, enums)

## Product Types

A product type combines multiple values:

```kleis
// A point has an x AND a y
structure Point {
    x : ℝ
    y : ℝ
}

// A person has a name AND an age
structure Person {
    name : String
    age : ℕ
}
```

## Sum Types (Variants)

A sum type represents alternatives:

```kleis
// A shape is a Circle OR a Rectangle OR a Triangle
data Shape {
    Circle(radius : ℝ)
    Rectangle(width : ℝ, height : ℝ)
    Triangle(a : ℝ, b : ℝ, c : ℝ)
}

// An optional value is Some(value) OR None
data Option(T) {
    Some(value : T)
    None
}

// A result is Ok(value) OR Err(message)
data Result(T, E) {
    Ok(value : T)
    Err(error : E)
}
```

## Pattern Matching with ADTs

ADTs shine with pattern matching:

```kleis
define area(shape) =
    match shape {
        Circle(r) => π * r^2
        Rectangle(w, h) => w * h
        Triangle(a, b, c) => 
            let s = (a + b + c) / 2 in
            sqrt(s * (s-a) * (s-b) * (s-c))
    }
```

## Recursive Types

Types can refer to themselves:

```kleis
// A list is either empty (Nil) or a value followed by another list (Cons)
data List(T) {
    Nil
    Cons(head : T, tail : List(T))
}

// A binary tree
data Tree(T) {
    Leaf(value : T)
    Node(left : Tree(T), value : T, right : Tree(T))
}
```

## The Mathematical Perspective

Why "algebraic"?

- **Product types** correspond to multiplication: `Point = ℝ × ℝ`
- **Sum types** correspond to addition: `Option(T) = T + 1`

The number of possible values follows algebra:
- `Bool` has 2 values
- `Bool × Bool` has 2 × 2 = 4 values
- `Bool + Bool` has 2 + 2 = 4 values

## Practical Example: Expression Trees

ADTs are perfect for representing mathematical expressions:

```text
data Expr {
    Const(value : ℝ)
    Var(name : String)
    Add(left : Expr, right : Expr)
    Mul(left : Expr, right : Expr)
    Neg(inner : Expr)
}

define eval(expr, env) =
    match expr {
        Const(v) => v
        Var(name) => lookup(env, name)
        Add(l, r) => eval(l, env) + eval(r, env)
        Mul(l, r) => eval(l, env) * eval(r, env)
        Neg(e) => -eval(e, env)
    }
```

## What's Next?

Let's dive deeper into pattern matching!

→ [Next: Pattern Matching](./05-pattern-matching.md)
