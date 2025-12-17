# Learn You Some Kleis

**for Great Mathematical Good!**

*A friendly introduction to the Kleis verification language*

---

## Table of Contents

### Part I: Getting Started

1. **[Introduction](chapters/01-introduction.md)**
   - About this tutorial
   - What is Kleis?
   - What you need to dive in

2. **[Starting Out](chapters/02-starting-out.md)**
   - Numbers and arithmetic
   - Variables and objects
   - Basic operations
   - Comments

3. **[Types and Type Annotations](chapters/03-types.md)**
   - Basic types (ℝ, ℤ, Bool)
   - Type annotations in let bindings
   - Type ascription (Haskell-style)
   - Parametric types

### Part II: Functions and Data

4. **[Functions](chapters/04-functions.md)**
   - Defining functions with `define`
   - Function calls
   - Operators as functions
   - Recursion

5. **[Algebraic Data Types](chapters/05-algebraic-types.md)**
   - Data declarations
   - Constructors
   - Option, Bool, List
   - Recursive types

6. **[Pattern Matching](chapters/06-pattern-matching.md)**
   - Match expressions
   - Wildcards and variables
   - Constructor patterns
   - Nested patterns

### Part III: Mathematical Notation

7. **[Let Bindings](chapters/07-let-bindings.md)**
   - Let expressions
   - Scoping rules
   - Nested bindings
   - The meaning of `in`

8. **[Quantifiers and Logic](chapters/08-quantifiers-logic.md)**
   - Universal quantifier (∀)
   - Existential quantifier (∃)
   - Logical operators (∧, ∨, ¬, ⟹)
   - Comparison operators

9. **[Conditionals](chapters/09-conditionals.md)**
   - If-then-else expressions
   - Guards in functions
   - Combining with pattern matching

### Part IV: Structures and Verification

10. **[Structures](chapters/10-structures.md)**
    - Declaring structures
    - Type parameters
    - Operations
    - Axioms

11. **[Implements and Instances](chapters/11-implements.md)**
    - Implements blocks
    - Where clauses
    - Nested structures
    - Extends

12. **[Z3 and Theorem Proving](chapters/12-z3-verification.md)**
    - What is Z3?
    - Axiom verification
    - SMT solving
    - Proof strategies

### Part V: Practical Kleis

13. **[The REPL](chapters/13-repl.md)**
    - Starting the REPL
    - Evaluating expressions
    - Loading files
    - Debugging

14. **[Building Applications](chapters/14-applications.md)**
    - File organization
    - Importing definitions
    - Best practices
    - Real-world examples

### Appendices

- **[A. Grammar Reference](chapters/appendix-a-grammar.md)**
- **[B. Operator Precedence](chapters/appendix-b-operators.md)**
- **[C. Standard Library](chapters/appendix-c-stdlib.md)**

---

## About Kleis

Kleis is a **verification language** for mathematics and physics. It combines:

- **Mathematical notation** — Write formulas as they appear in textbooks
- **Type safety** — Catch errors before they become disasters  
- **Theorem proving** — Verify your equations are correct
- **Z3 integration** — Leverage world-class SMT solvers

Whether you're a physicist checking tensor equations, a mathematician formalizing proofs, or a student learning calculus — Kleis speaks your language.

---

## Quick Example

```kleis
// Define a mathematical structure
structure Group(G) {
    operation (+) : G → G → G
    operation zero : G
    operation negate : G → G
    
    axiom identity : ∀(x : G). x + zero = x
    axiom inverse  : ∀(x : G). x + negate(x) = zero
    axiom assoc    : ∀(x y z : G). (x + y) + z = x + (y + z)
}

// Use it
define double(x : G) where Group(G) = x + x

// Verify properties
verify : ∀(x : G). double(x) = x + x
```

---

## Ready to Start?

**[Begin with Chapter 1: Introduction →](chapters/01-introduction.md)**

---

*This work is inspired by [Learn You a Haskell](https://learnyouahaskell.github.io/) and [Learn You Some Erlang](https://learnyousomeerlang.com/).*

*Licensed under Creative Commons Attribution-NonCommercial 4.0*

