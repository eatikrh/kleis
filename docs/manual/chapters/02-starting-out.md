# Chapter 2: Starting Out

[← Previous: Introduction](01-introduction.md) | [Back to Contents](../index.md) | [Next: Types →](03-types.md)

---

## Numbers and Arithmetic

Let's start with the basics. Open your Kleis REPL and try some arithmetic:

```kleis
>>> 2 + 3
5

>>> 10 - 4
6

>>> 7 * 8
56

>>> 15 / 3
5

>>> 2 ^ 10
1024
```

Nothing surprising here. Kleis supports all the usual arithmetic operations.

### Operator Precedence

Operators follow standard mathematical precedence:

1. `^` (exponentiation) — highest, right-associative
2. `*`, `/` (multiplication, division)
3. `+`, `-` (addition, subtraction) — lowest

```kleis
>>> 2 + 3 * 4
14         // Not 20! Multiplication first.

>>> 2 ^ 3 ^ 2
512        // 2^(3^2) = 2^9, because ^ is right-associative

>>> (2 + 3) * 4
20         // Parentheses override precedence
```

### Negative Numbers

Use the `-` prefix for negative numbers:

```kleis
>>> -5
-5

>>> -5 + 10
5

>>> 3 * -2
-6
```

---

## Variables and Objects

In Kleis, we distinguish between **constants** and **objects** (variables).

### Constants

Constants are literal values:

```kleis
42          // Integer constant
3.14        // Floating-point constant
"hello"     // String constant (less common in math)
```

### Objects (Variables)

Objects are symbolic names that can be bound to values:

```kleis
>>> let x = 5 in x + x
10

>>> let pi = 3.14159 in 2 * pi * 10
62.8318
```

Variables follow standard naming rules:
- Start with a letter or underscore
- Contain letters, numbers, underscores
- Case-sensitive (`x` and `X` are different)

```kleis
>>> let myVariable = 10 in myVariable
10

>>> let x_1 = 5 in let x_2 = 3 in x_1 + x_2
8
```

### Greek Letters

Kleis fully supports Greek letters — essential for mathematical notation:

```kleis
>>> let α = 1 in let β = 2 in α + β
3

>>> let Δx = 0.001 in Δx * 1000
1

>>> let θ = 3.14159 / 4 in θ
0.7853975
```

---

## Basic Operations

Beyond arithmetic, Kleis provides mathematical functions:

### Built-in Functions

```kleis
>>> abs(-5)
5

>>> sqrt(16)
4

>>> sin(0)
0

>>> cos(0)
1

>>> exp(1)
2.71828...

>>> log(exp(1))
1
```

### Function Calls

Functions are called with parentheses:

```kleis
>>> max(3, 7)
7

>>> min(3, 7)
3

>>> pow(2, 10)
1024
```

### Defining Your Own Functions

Use `define` to create functions:

```kleis
>>> define square(x) = x * x
Function 'square' defined

>>> square(5)
25

>>> square(square(2))
16
```

Functions can take multiple parameters:

```kleis
>>> define add(a, b) = a + b
Function 'add' defined

>>> add(3, 4)
7

>>> define hypotenuse(a, b) = sqrt(a^2 + b^2)
Function 'hypotenuse' defined

>>> hypotenuse(3, 4)
5
```

---

## Comments

Comments help document your code:

### Line Comments

```kleis
// This is a line comment
define area(r) = 3.14159 * r^2  // Circle area
```

### Block Comments

```kleis
/* 
   This is a block comment.
   It can span multiple lines.
   Useful for longer explanations.
*/
define volume(r) = (4/3) * 3.14159 * r^3
```

---

## Putting It Together

Let's write a small program that calculates the distance between two points:

```kleis
// Define a function to calculate Euclidean distance
define distance(x1, y1, x2, y2) =
    let dx = x2 - x1 in
    let dy = y2 - y1 in
    sqrt(dx^2 + dy^2)

// Test it
>>> distance(0, 0, 3, 4)
5

>>> distance(1, 1, 4, 5)
5
```

Here we used:
- `define` to create a function
- `let ... in` to introduce local variables
- `sqrt` and `^` for the calculation
- Comments to explain what we're doing

---

## Exercises

Try these in the REPL:

1. **Calculate** `(1 + 2 + 3 + 4 + 5)^2`

2. **Define** a function `cube(x)` that returns `x^3`

3. **Define** a function `average(a, b)` that returns the average of two numbers

4. **Calculate** the area of a circle with radius 7 (use `let pi = 3.14159 in ...`)

5. **Define** a function `quadratic(a, b, c, x)` that evaluates `ax² + bx + c`

<details>
<summary>Solutions</summary>

```kleis
// 1.
>>> (1 + 2 + 3 + 4 + 5)^2
225

// 2.
>>> define cube(x) = x^3
>>> cube(3)
27

// 3.
>>> define average(a, b) = (a + b) / 2
>>> average(10, 20)
15

// 4.
>>> let pi = 3.14159 in pi * 7^2
153.93791

// 5.
>>> define quadratic(a, b, c, x) = a * x^2 + b * x + c
>>> quadratic(1, 0, -4, 2)  // x² - 4 at x=2
0
```

</details>

---

## Summary

In this chapter, you learned:

- Basic arithmetic operations (`+`, `-`, `*`, `/`, `^`)
- Operator precedence follows math conventions
- Variables are introduced with `let ... in`
- Greek letters work naturally
- Functions are defined with `define name(params) = body`
- Comments use `//` for lines, `/* */` for blocks

Next, we'll explore Kleis's type system!

---

[← Previous: Introduction](01-introduction.md) | [Back to Contents](../index.md) | [Next: Types →](03-types.md)

