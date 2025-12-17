# Chapter 1: Introduction

[← Back to Contents](../index.md) | [Next: Starting Out →](02-starting-out.md)

---

## About This Tutorial

So you've decided to learn Kleis. Excellent choice! This tutorial will take you from zero to hero — well, at least from zero to "can write and verify mathematical expressions without setting your computer on fire."

We assume you know:
- Basic programming concepts (variables, functions)
- Some mathematics (algebra, maybe a little calculus)
- How to use a terminal

We don't assume you know:
- Haskell, OCaml, or any functional language
- Type theory or formal methods
- SMT solvers or theorem provers

If you've used Haskell or ML before, you'll feel right at home. If not, don't worry — we'll explain everything.

---

## What is Kleis?

**Kleis** (pronounced "klice", rhymes with "dice") is a verification language for mathematics.

Think of it as a programming language that:

1. **Speaks math** — Write `∀(x : ℝ). x + 0 = x` instead of `forall x in R: x + 0 == x`
2. **Checks your work** — It will tell you if your equations are wrong
3. **Proves things** — Not just checks, but mathematically *proves*

### Why Would I Want That?

Have you ever:

- Spent hours debugging a physics simulation only to discover you forgot a minus sign?
- Written a paper with an equation that looked right but was subtly wrong?
- Wished you could just ask a computer "is this equation valid?"

Kleis solves these problems.

### A Taste of Kleis

Here's what Kleis code looks like:

```kleis
// Define a simple function
define square(x) = x * x

// Use mathematical notation
define norm(v : Vector(3)) = sqrt(v[0]^2 + v[1]^2 + v[2]^2)

// Express mathematical laws
axiom commutativity : ∀(x y : ℝ). x + y = y + x

// Let Kleis verify them
structure Ring(R) {
    operation (+) : R → R → R
    operation (*) : R → R → R
    
    axiom add_comm : ∀(a b : R). a + b = b + a
    axiom mul_assoc : ∀(a b c : R). (a * b) * c = a * (b * c)
}
```

---

## The Kleis Philosophy

### 1. Mathematical Notation is Good

Other languages make you write `forall x in Real` or `for all x : Real`. 

Kleis lets you write `∀(x : ℝ)` because that's what mathematicians write. Your code should look like your equations.

### 2. Types Prevent Bugs

You can't add a scalar to a matrix in Kleis. You can't pass a 3-vector where a 4-vector is expected. The type system catches these errors *before* you run anything.

### 3. Verification is Better Than Testing

Tests check specific cases: "does `f(5)` return `25`?"

Verification proves general facts: "does `f(x) = x * x` for *all* x?"

Kleis uses **Z3**, a world-class theorem prover from Microsoft Research, to verify your mathematical statements.

### 4. Abstraction Through Structures

Kleis lets you define abstract mathematical structures — groups, rings, vector spaces — and prove properties that hold for *any* instance of that structure.

---

## What You Need to Dive In

### Required

- **Rust toolchain** — Kleis is written in Rust. You'll need `cargo` to build it.
- **Z3** — The theorem prover. Install via your package manager.

```bash
# macOS
brew install z3

# Ubuntu/Debian
sudo apt install z3

# Windows (with chocolatey)
choco install z3
```

### Building Kleis

```bash
git clone https://github.com/eatikrh/kleis.git
cd kleis
cargo build --release
```

### Running the REPL

```bash
cargo run --bin repl
```

You should see:

```
Kleis REPL v0.1.0
Type :help for commands, :quit to exit
>>> 
```

Try typing:

```kleis
>>> 2 + 2
4
>>> let x = 5 in x * x
25
```

Congratulations! You're running Kleis.

---

## A Note on Notation

Throughout this tutorial, we'll use mathematical symbols like:

| Symbol | Meaning | Keyboard alternative |
|--------|---------|---------------------|
| `∀` | For all | `forall` |
| `∃` | There exists | `exists` |
| `∧` | And | `and` |
| `∨` | Or | `or` |
| `¬` | Not | `not` |
| `⟹` | Implies | `=>` |
| `ℝ` | Real numbers | `Real` |
| `ℤ` | Integers | `Int` |
| `ℕ` | Natural numbers | `Nat` |

Kleis accepts both! Use whichever you prefer.

---

## Summary

- Kleis is a verification language for mathematics
- It uses mathematical notation directly
- It can prove properties, not just test them
- It's built on solid type theory and theorem proving foundations

Now let's actually write some Kleis code!

---

[← Back to Contents](../index.md) | [Next: Starting Out →](02-starting-out.md)

