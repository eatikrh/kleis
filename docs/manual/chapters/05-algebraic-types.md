# Chapter 5: Algebraic Data Types

[← Previous: Functions](04-functions.md) | [Back to Contents](../index.md) | [Next: Pattern Matching →](06-pattern-matching.md)

---

## What Are Algebraic Data Types?

Algebraic data types (ADTs) let you define your own types by combining simpler types. They're called "algebraic" because they work like algebra:

- **Sum types** — "this OR that" (like addition)
- **Product types** — "this AND that" (like multiplication)

---

## Defining Data Types

Use the `data` keyword:

```kleis
data TypeName = Constructor1 | Constructor2 | ...
```

### Simple Enumeration

```kleis
data Bool = True | False

data Color = Red | Green | Blue

data Direction = North | South | East | West
```

These have multiple **constructors** but no data attached.

### With Data

Constructors can carry data:

```kleis
data Option(T) = None | Some(T)

data Result(T, E) = Ok(T) | Err(E)

data List(T) = Nil | Cons(T, List(T))
```

---

## The Option Type

`Option(T)` represents a value that might not exist:

```kleis
data Option(T) = None | Some(T)
```

- `None` — Nothing there
- `Some(x)` — Contains value `x`

```kleis
// A function that might fail
define safe_sqrt(x) =
    if x < 0 then None
    else Some(sqrt(x))

>>> safe_sqrt(16)
Some(4)

>>> safe_sqrt(-1)
None
```

---

## The Bool Type

```kleis
data Bool = True | False
```

Used for conditions:

```kleis
>>> 5 > 3
True

>>> 2 = 2
True

>>> not(True)
False

>>> True and False
False

>>> True or False
True
```

---

## Lists

```kleis
data List(T) = Nil | Cons(T, List(T))
```

- `Nil` — Empty list
- `Cons(head, tail)` — Element followed by more list

```kleis
// [1, 2, 3] is sugar for:
Cons(1, Cons(2, Cons(3, Nil)))

// List functions
define length(xs) =
    match xs {
        Nil => 0
        Cons(_, tail) => 1 + length(tail)
    }

>>> length([1, 2, 3, 4, 5])
5
```

---

## Binary Trees

```kleis
data Tree(T) = Leaf | Node(T, Tree(T), Tree(T))
```

- `Leaf` — Empty tree
- `Node(value, left, right)` — Value with two subtrees

```kleis
define tree_sum(t) =
    match t {
        Leaf => 0
        Node(v, l, r) => v + tree_sum(l) + tree_sum(r)
    }

let my_tree = Node(1, Node(2, Leaf, Leaf), Node(3, Leaf, Leaf)) in
tree_sum(my_tree)
// = 1 + 2 + 3 = 6
```

---

## Natural Numbers (Peano)

```kleis
data Nat = Zero | Succ(Nat)
```

- `Zero` — The number 0
- `Succ(n)` — Successor of n (i.e., n + 1)

```kleis
// 3 = Succ(Succ(Succ(Zero)))

define nat_add(m, n) =
    match m {
        Zero => n
        Succ(m') => Succ(nat_add(m', n))
    }
```

---

## Pairs and Tuples

```kleis
data Pair(A, B) = MkPair(A, B)
```

Or use built-in tuple syntax:

```kleis
>>> let p = (3, "hello") in p
(3, "hello")

>>> let (x, y) = (1, 2) in x + y
3
```

---

## Type Parameters

Data types can be **parametric** — they work with any type:

```kleis
data Option(T) = None | Some(T)
```

Here `T` is a **type parameter**:

```kleis
Some(5)        : Option(ℤ)
Some(3.14)     : Option(ℝ)
Some(True)     : Option(Bool)
Some([1,2,3])  : Option(List(ℤ))
```

### Multiple Parameters

```kleis
data Either(A, B) = Left(A) | Right(B)

Left(5)        : Either(ℤ, String)
Right("error") : Either(ℤ, String)
```

---

## Recursive Types

Types can refer to themselves:

```kleis
data List(T) = Nil | Cons(T, List(T))
//                              ↑ recursive!

data Tree(T) = Leaf | Node(T, Tree(T), Tree(T))
//                           ↑ recursive twice!
```

This allows infinite structures (computed lazily):

```kleis
// A list: [1, 2, 3, ...]
Cons(1, Cons(2, Cons(3, ...)))
```

---

## Summary

| Type | Definition | Constructors |
|------|------------|--------------|
| `Bool` | `True \| False` | 2 nullary |
| `Option(T)` | `None \| Some(T)` | 1 nullary, 1 unary |
| `List(T)` | `Nil \| Cons(T, List(T))` | 1 nullary, 1 binary |
| `Tree(T)` | `Leaf \| Node(T, Tree(T), Tree(T))` | 1 nullary, 1 ternary |

---

## Exercises

1. **Define** a `Maybe(T)` type (same as `Option`, just different names: `Nothing` and `Just`)

2. **Define** a `Shape` type with constructors `Circle(radius)`, `Rectangle(width, height)`, `Triangle(a, b, c)`

3. **Define** a function `is_some(opt)` that returns `True` if opt is `Some(_)`

4. **Define** `list_head(xs)` that returns `Some(first element)` or `None` if empty

<details>
<summary>Solutions</summary>

```kleis
// 1.
data Maybe(T) = Nothing | Just(T)

// 2.
data Shape = Circle(ℝ) | Rectangle(ℝ, ℝ) | Triangle(ℝ, ℝ, ℝ)

// 3.
define is_some(opt) =
    match opt {
        None => False
        Some(_) => True
    }

// 4.
define list_head(xs) =
    match xs {
        Nil => None
        Cons(h, _) => Some(h)
    }
```

</details>

---

[← Previous: Functions](04-functions.md) | [Back to Contents](../index.md) | [Next: Pattern Matching →](06-pattern-matching.md)

