# Integrating Haskell's Type System into Kleis

**Date:** December 2024  
**Status:** Design Document  
**Consolidated from:** 5 source documents

---

## Table of Contents

1. [Why Haskell for Symbolic Math](#why-haskell-for-symbolic-math)
2. [Core Concepts](#core-concepts)
3. [What to Adopt](#what-to-adopt)
4. [What to Adapt](#what-to-adapt)
5. [Hindley-Milner Simplified](#hindley-milner-simplified)
6. [Studying GHC Source Code](#studying-ghc-source-code)

---

## Why Haskell for Symbolic Math

### The Critical Insight

**Type checking works on expression STRUCTURE, not VALUES.**

This means Haskell's type inference algorithm works **identically** for symbolic mathematics as it does for evaluated programs.

### Comparison

| System | Evaluation | Type Checking | Result |
|--------|------------|---------------|--------|
| Haskell | ✅ Yes | ✅ Yes | Values |
| Mathematica | ❌ Symbolic | ❌ No | Can produce nonsense |
| **Kleis** | **❌ Symbolic** | **✅ Yes** | **Type-safe symbols** |

### Example

**Haskell (evaluates):**
```haskell
double x = x + x
-- Type inference: Num a => a -> a
-- Evaluation: double 5 = 10
```

**Kleis (symbolic):**
```kleis
define double(x) = x + x
// Type inference: Numeric(a) ⇒ a → a (SAME ALGORITHM!)
// No evaluation: double(y) stays as "2y" (symbolic)
```

**Key Point:** The type inference algorithm is IDENTICAL! Only difference is evaluation vs keeping symbolic.

---

## Core Concepts

### 1. Type Classes = Algebraic Structures

**Haskell:**
```haskell
class Monoid a where
  mempty :: a
  mappend :: a -> a -> a
```

**Kleis:**
```kleis
structure Monoid(M) {
  element e : M
  operation (•) : M × M → M
  
  axiom identity: ∀x. e • x = x ∧ x • e = x
  axiom associativity: ∀x y z. (x•y)•z = x•(y•z)
}
```

**Mapping:**
- Type class → `structure`
- Instance → `implements`
- Laws (comments) → `axiom` (verifiable!)

### 2. Hindley-Milner Inference

**Algorithm:**
1. Assign type variables to unknowns
2. Generate constraints from usage
3. Unify constraints
4. Generalize free variables

**Example:**
```kleis
define f(x) = x + x

// Step 1: x : α
// Step 2: (+) requires: α ~ β, Numeric(α)
// Step 3: Unify: α := β
// Step 4: Generalize: ∀a. Numeric(a) ⇒ a → a
```

**Works for symbolic expressions!**

### 3. Higher-Kinded Types

**Enable generic operations:**

```kleis
structure Functor(F) {
  operation map : ∀A B. (A → B) → F(A) → F(B)
}

implements Functor(List)
implements Functor(Maybe)
implements Functor(Vector)  // Math structure!
```

### 4. GADTs for Type-Safe AST

**Prevent nonsense at construction:**

```rust
enum TypedExpr<T> {
    Add<T>(TypedExpr<T>, TypedExpr<T>) : TypedExpr<T>,
    ScalarMul<V>(TypedExpr<Scalar>, TypedExpr<V>) : TypedExpr<V>,
}

// This won't compile:
// Add(scalar, vector)  // Type error!
```

### 5. Dependent Types for Dimensions

**Track dimensions in types:**

```kleis
Matrix(3,4) × Matrix(4,2) → Matrix(3,2) ✓
Matrix(3,4) × Matrix(3,5) → ERROR ✗
```

### 6. Phantom Types for Properties

**Track properties without runtime cost:**

```kleis
Matrix<ℝ, n, n, Symmetric>
// "Symmetric" is phantom - no runtime representation
// But type system knows eigenvalues are real!
```

---

## What to Adopt

### 1. Type Inference Algorithm ✅
- Hindley-Milner for automatic type inference
- Minimal annotations needed

### 2. Type Classes → Algebraic Structures ✅
- Monoid, Group, Ring, Field hierarchy
- Inheritance with `extends`
- Instance resolution

### 3. Polymorphic Types ✅
```kleis
define id : ∀T. T → T
define map : ∀A B. (A → B) → List(A) → List(B)
```

### 4. Constrained Polymorphism ✅
```kleis
define sum : ∀T. Monoid(T) ⇒ List(T) → T
```

### 5. Higher-Kinded Types ✅
```kleis
structure Functor(F : * → *) { ... }
```

### 6. Type-Safe AST (GADTs) ✅
```rust
enum TypedExpr<T> { ... }
```

---

## What to Adapt

### 1. Mathematical Notation

**Haskell:** Programming syntax
```haskell
f :: Int -> Int
forall a. a -> a
```

**Kleis:** Mathematical symbols
```kleis
f : ℤ → ℤ
∀a. a → a
```

### 2. Multiple Equality Types

**Haskell:** One `=`

**Kleis:** Four equalities
```kleis
define c = 3e8         // Definition
assert (x+y)² == x² + 2xy + y²  // Algebraic
equiv Vector(ℝ³) ~ ℝ³  // Structural
approx π ≈ 3.14        // Numerical
```

### 3. Axiom Verification

**Haskell:** Laws are comments (not checked)

**Kleis:** Axioms are first-class (verified!)
```kleis
structure Monoid(M) {
  axiom identity: ∀x. e • x = x
}

implements Monoid(ℤ, +, 0) {
  verify identity  // Kleis checks this!
}
```

### 4. No Evaluation

**Haskell:** Types → evaluate → value

**Kleis:** Types → verify → symbolic
```kleis
define f(x) = x²
// Stays as "x²" forever (no evaluation)
```

---

## Hindley-Milner Simplified

### The Core Algorithm

```
Given: Expression to type-check
Output: Type (or type error)

1. Assign fresh type variables to unknowns
2. Generate constraints from operations
3. Solve constraints via unification
4. Apply solution to get final type
5. Generalize free variables to ∀
```

### Example: Type Inference for `double`

```kleis
define double(x) = x + x

// Step 1: Assign variables
x : α (fresh type variable)

// Step 2: Generate constraints
(+) : Numeric(β) ⇒ β → β → β
x : α
x : α
Constraints: α ~ β, Numeric(α)

// Step 3: Unify
α ~ β  ==>  α := β
Constraint remains: Numeric(α)

// Step 4: Apply solution
Result type: α → α where Numeric(α)

// Step 5: Generalize
Final type: ∀a. Numeric(a) ⇒ a → a
```

### The Three Core Operations

#### 1. Unification

**Make two types equal:**

```rust
unify(α, Int) = {α := Int}
unify(α → β, Int → γ) = {α := Int, β := γ}
unify(Matrix(m,n), Matrix(3,4)) = {m := 3, n := 4}
```

#### 2. Substitution

**Replace variables:**

```rust
apply({α := Int}, α → α) = Int → Int
apply({m := 3, n := 4}, Matrix(m,n)) = Matrix(3,4)
```

#### 3. Generalization

**Turn variables into ∀:**

```rust
generalize(α → α) = ∀a. a → a
generalize(α → β) = ∀a b. a → b
```

### Why It Works for Symbolic Math

**Type inference looks at STRUCTURE:**
- What operations are used?
- How are they connected?
- What are the constraints?

**Type inference does NOT look at VALUES:**
- What is x?
- What is the result of x²?
- What's the numerical answer?

**Therefore:** Works perfectly for symbolic expressions!

---

## Studying GHC Source Code

### Repository

**Location:** `~/Documents/git/cee/haskell/ghc/`  
**Size:** 25,705 files + 33 submodules

### Key Directories

```
compiler/GHC/
├── Tc/                    # Type Checker (most important!)
│   ├── Solver.hs         # ⭐ Constraint solver (Hindley-Milner core)
│   ├── Gen/
│   │   └── Expr.hs       # ⭐ Expression type inference
│   ├── Instance/
│   │   └── Class.hs      # ⭐ Type class resolution
│   └── Utils/
│       └── Unify.hs      # ⭐ Unification algorithm
│
└── Core/
    ├── Type.hs           # ⭐ Type representation
    └── TyCon.hs          # Type constructors
```

### Warning: GHC Code is Cryptic!

GHC's code has:
- 30+ years of optimization
- Every edge case imaginable
- Performance hacks
- Backwards compatibility

**Don't try to learn from it directly!**

### Better Resources to Start

1. **"Typing Haskell in Haskell"** by Mark P. Jones
   - https://gist.github.com/chrisdone/0075a16b32bfd4f62b7b
   - ~500 lines, actually readable!
   - Implements full Hindley-Milner

2. **"Write You a Haskell"** Tutorial
   - http://dev.stephendiehl.com/fun/006_hindley_milner.html
   - Step-by-step implementation

3. **Algorithm W** (Original paper)
   - 5 pages, elegant, simple

### Learning Path

**Week 1:** Read simplified resources  
**Week 2:** Implement minimal version  
**Week 3:** Look at GHC for specific features  

**Use GHC as reference, not tutorial!**

---

## What Kleis Should Do

### Phase 1: Core Type System
```rust
enum Type {
    Scalar, Vector(usize), Matrix(usize, usize),
    Var(TypeVar), Function(Box<Type>, Box<Type>),
}

fn infer(expr: &Expression) -> Result<Type, TypeError>
fn unify(t1: &Type, t2: &Type) -> Result<Substitution, TypeError>
```

### Phase 2: Algebraic Structures
```kleis
structure Monoid(M) { ... }
structure Group(G) extends Monoid(G) { ... }
implements Monoid(ℤ, +, 0)
```

### Phase 3: User-Defined Types
```kleis
structure PurchaseOrder {
  orderId : String
  total : Money
  axiom positive_total: total > 0
}
```

### Phase 4: Axiom Verification
```kleis
verify Monoid(ℤ, +, 0)
// Check: associativity, identity ✓
```

---

## Key Differences: Haskell vs Kleis

| Feature | Haskell | Kleis |
|---------|---------|-------|
| **Type inference** | ✅ Yes | ✅ Yes (same algorithm) |
| **Type classes** | ✅ Yes | ✅ Structures (similar) |
| **Notation** | Programming | **Mathematical** |
| **Evaluation** | Yes | **No (symbolic)** |
| **Axiom verification** | No | **Yes (unique!)** |
| **Multiple equalities** | No | **Yes (4 types)** |

---

## Conclusion

**YES! We adopt Haskell's entire type system!**

Because type checking verifies STRUCTURE, not VALUES, it works perfectly for symbolic mathematics.

### What We Get:
- ✅ Type inference
- ✅ Polymorphism
- ✅ Algebraic structures
- ✅ Dimensional safety

### What We Add:
- ✅ Mathematical notation
- ✅ Axiom verification
- ✅ Symbolic focus

**Haskell discovered type theory for programming. We apply it to mathematics!**

