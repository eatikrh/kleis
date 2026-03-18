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

A sum type represents alternatives — a value that can be *one of* several different forms.

### The `data` Keyword

In Kleis, you define sum types using the `data` keyword:

```kleis
data TypeName = Constructor1 | Constructor2 | Constructor3
```

**Syntax breakdown:**
- `data` — keyword that introduces a new type definition
- `TypeName` — the name of your new type (starts with uppercase)
- `=` — separates the type name from its constructors
- `Constructor1`, `Constructor2`, etc. — the possible variants (each starts with uppercase)
- `|` — read as "or" — separates the alternatives

### Constructors with Data

Constructors can carry data (fields):

```kleis
data TypeName = Constructor1(field1 : Type1) | Constructor2(field2 : Type2, field3 : Type3)
```

Each constructor acts like a function that creates a value of the type.

### Parameterized Types (Generics)

Types can have parameters, making them work with any type:

```kleis
data Option(T) = Some(value : T) | None
```

Here `T` is a *type parameter*. You can use `Option(ℕ)` for optional natural numbers, `Option(String)` for optional strings, etc. The type is *generic* — it works for any `T`.

### Examples

```kleis
// A shape is a Circle OR a Rectangle OR a Triangle
data Shape = Circle(radius : ℝ) | Rectangle(width : ℝ, height : ℝ) | Triangle(a : ℝ, b : ℝ, c : ℝ)

// An optional value is Some(value) OR None
data Option(T) = Some(value : T) | None

// A result is Ok(value) OR Err(message)
data Result(T, E) = Ok(value : T) | Err(error : E)
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

```kleis
data Expression = 
    ENumber(value : ℝ)
  | EVariable(name : String)
  | EOperation(name : String, args : List(Expression))

// Helper constructors for cleaner syntax
define num(n) = ENumber(n)
define var(x) = EVariable(x)
define e_add(a, b) = EOperation("plus", Cons(a, Cons(b, Nil)))
define e_mul(a, b) = EOperation("times", Cons(a, Cons(b, Nil)))
define e_neg(a) = EOperation("neg", Cons(a, Nil))

define eval_expr(expr, env) =
    match expr {
        ENumber(v) => v
        EVariable(name) => lookup(env, name)
        EOperation("plus", Cons(l, Cons(r, Nil))) => 
            eval_expr(l, env) + eval_expr(r, env)
        EOperation("times", Cons(l, Cons(r, Nil))) => 
            eval_expr(l, env) * eval_expr(r, env)
        EOperation("neg", Cons(e, Nil)) => 
            -eval_expr(e, env)
        _ => 0
    }
```

## What's Next?

Let's dive deeper into pattern matching!

→ [Next: Pattern Matching](./05-pattern-matching.md)
