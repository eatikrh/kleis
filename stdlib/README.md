# Kleis Standard Library

**Location:** `stdlib/`  
**Language:** Kleis v0.5 (with pattern matching)  
**Purpose:** Bootstrap the type system with fundamental mathematical structures

---

## Overview

The Kleis standard library is written **in Kleis itself** (self-hosting). It provides:

- Algebraic structure hierarchy (Monoid → Group → Ring → Field)
- Implementations for built-in types (ℝ, ℂ, ℤ, Vector, Matrix)
- Common mathematical operations (dot, cross, det, trace, ∂, ∫, ∇)
- Mathematical constants (π, e, i, φ)

---

## Files

### Core Library (Always Loaded)

**`prelude.kleis`** (~500 lines)
- Algebraic hierarchy: Semigroup, Monoid, Group, AbelianGroup, Ring, Field
- Vector space structure
- Implementations: Field(ℝ), Field(ℂ), Ring(ℤ), VectorSpace(Vector)
- Vector operations: dot, cross, norm
- Matrix operations: det, trace, transpose, (×)
- Calculus operations: d/dx, ∂/∂x, ∇, ∫
- Constants: π, e, i, φ, √2

**Status:** ✅ Defined, ⬜ Parser support needed

### Optional Libraries (Import on Demand)

**`quantum.kleis`** (planned)
- HilbertSpace structure
- Bra-ket notation: ⟨·|·⟩
- Operators: †, [·,·] (commutator)
- Quantum states and measurements

**`pot.kleis`** (planned)
- ModalSpace structure
- Projection operators: Π
- Projection kernel: K(x,m)
- Residue operations
- Hont (Hilbert Ontology)

**`linear_algebra.kleis`** (planned)
- Extended matrix operations
- Eigenvalues, eigenvectors
- Matrix decompositions (SVD, QR, LU)
- Special matrices (Hermitian, Unitary, Orthogonal)

---

## How It's Loaded

### Server Startup

```rust
// When Kleis server starts
let mut ctx = EditorTypeContext::core();  // Primitives only

// Load standard library (self-hosting!)
let prelude = include_str!("../stdlib/prelude.kleis");
ctx.load_kleis_definitions(prelude)?;

// Context now has:
// - 7 algebraic structures
// - 47 operations  
// - 8 constants
// - 12 implementations
```

### Optional Imports (User Choice)

```kleis
// In user's document
import std.quantum
import std.pot
```

---

## Why Self-Hosting?

### Benefits

✅ **Visible** - Users can read the stdlib source  
✅ **Modifiable** - Users can extend or override  
✅ **Consistent** - Same syntax as user code  
✅ **Self-documenting** - Stdlib IS the documentation  
✅ **Testable** - Can verify stdlib axioms  
✅ **Extensible** - Easy to add new structures  

### What's Hardcoded (Minimal)

Only these are in Rust (`src/type_inference.rs`):
- Primitive types: `Scalar, Bool, String, Nat`
- Type constructors: `Vector(n), Matrix(m,n), List(T)`
- Unification algorithm
- Constraint solving

**Everything else is Kleis code!**

---

## Example: How `a + b` Gets Type-Checked

### 1. Stdlib Loaded

```kleis
// From prelude.kleis:
structure Numeric(T) {
  operation (+) : T × T → T
  ...
}

implements Numeric(ℝ)
implements Numeric(ℂ)
implements Numeric(Vector(n))
```

### 2. Context Built

```rust
// After loading prelude.kleis:
operation_registry.types_supporting("+") = [ℝ, ℂ, Vector(n), Matrix(m,n)]
```

### 3. User Types Expression

```kleis
a + b
```

### 4. Type Inference

```rust
// Query: which types support (+)?
candidates = [ℝ, ℂ, Vector(n), Matrix(m,n)]  // From stdlib!

// Generate constraint:
a : α where α ∈ candidates
b : α

// Result:
Type: ∀α. Numeric(α) ⇒ α
```

### 5. User Sees Feedback

```
🟢 Type: α where Numeric(α)
Possible types: ℝ, ℂ, Vector(n), Matrix(m,n)
```

---

## Adding New Structures (User Extensions)

### User Creates: `workspace/my_types.kleis`

```kleis
// User-defined business type
structure Money {
  amount : ℝ
  currency : String
  
  supports {
    (+) : Money × Money → Money
    (×) : ℝ × Money → Money
  }
  
  axiom non_negative: amount ≥ 0
  axiom same_currency: ∀(m₁ m₂ : Money). m₁ + m₂ requires m₁.currency = m₂.currency
}

implements Monoid(Money) {
  element zero = Money { amount: 0, currency: "USD" }
  operation (+)(m₁, m₂) = Money {
    amount: m₁.amount + m₂.amount,
    currency: m₁.currency  // Assumes same currency
  }
}

// Now Money works with generic Monoid operations!
define total : List(Money) → Money
define total(amounts) = fold(amounts, (+), zero)
```

### Loading User Types

```rust
// Load user workspace
ctx.load_kleis_definitions(&user_workspace)?;

// Now type system knows about Money!
// Expression: order.total + tax
// Type: Money + Money → Money ✓
```

---

## Grammar Conformance

All code in `stdlib/*.kleis` conforms to **Kleis Grammar v0.5**:

✅ Structure definitions  
✅ Operation declarations  
✅ Axiom syntax  
✅ Implementation blocks  
✅ Polymorphic types with `∀`  
✅ Type annotations with `:`  
✅ Library annotations with `@`  
✅ Algebraic data types with `data` (v0.4)  
✅ Pattern matching with `match` (v0.5)  

---

## Status

**Defined:** ✅ `types.kleis`, `prelude.kleis`, `matrices.kleis` written  
**Grammar:** ✅ v0.5 formalized (with pattern matching!)  
**Parser:** ✅ Pattern matching implemented  
**Type Inference:** ✅ Pattern matching type-checks  
**Evaluation:** ✅ Pattern matching evaluates  
**Exhaustiveness:** ✅ Missing case warnings  
**Loader:** ⬜ Needs implementation  

**Pattern Matching Functions:** ✅ Added to `types.kleis`
- Boolean operations: `not`, `and`, `or`
- Option operations: `isSome`, `isNone`, `getOrDefault`, `mapOption`
- Result operations: `isOk`, `isErr`, `unwrapOr`
- List operations: `isEmpty`, `head`, `tail`
- Meta-level: `isScalarType`, `isVectorType`, `vectorDimension`  

---

## Next Steps

1. Implement parser for v0.3 syntax
2. Implement stdlib loader
3. Load prelude.kleis at server startup
4. Test type inference with stdlib context
5. Add optional libraries (quantum, pot)

---

**The standard library is now formal Kleis code that defines the type system!** 🎯

