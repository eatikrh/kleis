# Learn You Some Kleis for Great Good!

> *"Mathematics is the language with which God has written the universe."* — Galileo Galilei

Welcome to **Learn You Some Kleis**, an illustrated guide to the Kleis mathematical language!

## What is Kleis?

Kleis is a **mathematical expression language** designed for:

- 📐 **Symbolic mathematics** — work with expressions, not just numbers
- ✅ **Formal verification** — prove properties with Z3 theorem prover
- 🏗️ **Extensible structures** — define your own mathematical objects
- 🔬 **Scientific computing** — differential geometry, tensor calculus, and more

## Who is This For?

This guide is for anyone who:

- Wants to express mathematical ideas precisely
- Is curious about formal verification
- Enjoys functional programming (Haskell, ML, etc.)
- Works with differential geometry or tensor calculus

## How to Read This Guide

Each chapter builds on the previous ones. We start with the basics:

1. **Types** — the foundation of everything
2. **Functions** — defining computations
3. **Structures** — building mathematical objects

Then we explore advanced features:

4. **Pattern matching** — elegant case analysis
5. **Verification** — proving things with Z3
6. **Applications** — real-world examples

## A Taste of Kleis

Here's what Kleis looks like:

```kleis
// Define a function
define square(x) = x * x

// With type annotation
define double(x : ℝ) : ℝ = x + x

// Create a structure
structure Vector(n : ℕ) {
    axiom dimension : n ≥ 1
    operation dot : Vector(n) → ℝ
}
```

## Getting Started

Ready? Let's dive in!

→ [Start with Chapter 1: Starting Out](./chapters/01-starting-out.md)

---

*This guide is inspired by [Learn You a Haskell](https://learnyouahaskell.github.io/) and [Learn You Some Erlang](https://learnyousomeerlang.com/).*
