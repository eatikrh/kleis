# Kleis Standard Library

**Location:** `stdlib/`  
**Language:** Kleis v0.8 (Advanced pattern matching)  
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

**`types.kleis`** (~260 lines)
- Algebraic data types: Bool, Option, Result, List
- Type system types: Scalar, Vector, Matrix, Complex, Set
- Pattern matching function examples
- Foundation for self-hosting type system

**Status:** ✅ Complete, ✅ Loaded

**`minimal_prelude.kleis`** (~300 lines)
- Basic algebraic structures: Numeric, Arithmetic, Additive
- Implementations for ℝ, ℂ, ℤ
- Basic operations: +, -, *, /, abs, floor, ceiling

**Status:** ✅ Complete, ✅ Loaded

**`matrices.kleis`** (~122 lines)
- Matrix structures: Matrix, MatrixAddable, MatrixMultipliable, SquareMatrix
- Operations: transpose, add, multiply, det, trace, identity
- Block matrix support via polymorphism
- Legacy constructors for backward compatibility

**Status:** ✅ Complete, ✅ Loaded

### Domain-Specific Libraries

**`calculus.kleis`** (~230 lines) ✨ NEW!
- Derivative structures: `Differentiable`, `SmoothFunction`
- Integration: `Integrable` with Fundamental Theorem of Calculus
- Limits: `HasLimit` with limit laws
- Summation: `Summable` with known sums (linear, quadratic)
- Products: `Productable` with factorial
- Vector Calculus: gradient, laplacian, divergence, curl
- Mathematica-style: `D(f,x)`, `Dt(f,x)`, `Integrate`, `Limit`, `Sum`, `Product`

**Status:** ✅ Defined, ⬜ Not yet loaded by default

**`tensors.kleis`** (~280 lines)
- General Relativity tensor operations
- Curvature: Riemann, Ricci, Einstein, Weyl tensors
- Connection: Christoffel symbols, covariant derivative
- Physics: Stress-energy tensor, geodesics, Killing vectors
- Standard metrics: Minkowski, Schwarzschild, Kerr, FLRW
- Tensor products: outer product, wedge product (now enabled!), Lie derivative
- v0.92+: Type-level arithmetic enabled (p + q, n - 1, 2*n)

**Status:** ✅ Defined, ⬜ Not yet loaded by default

**`differential_forms.kleis`** (~350 lines) ✨ NEW!
- **Cartan Calculus:** Full exterior algebra operations
- **Wedge Product (∧):** Antisymmetric tensor product with graded commutativity
- **Exterior Derivative (d):** With d² = 0 axiom (de Rham cohomology)
- **Hodge Star (⋆):** Duality operator for p-forms ↔ (n-p)-forms
- **Interior Product (ι_X):** Contraction with vector fields
- **Cartan's Magic Formula:** ℒ_X = d ∘ ι_X + ι_X ∘ d
- **Physics Applications:**
  - Electromagnetic field as 2-form: dF = 0, d⋆F = ⋆J (Maxwell's equations)
  - Yang-Mills theory: F = dA + A ∧ A (non-abelian gauge fields)
  - Symplectic mechanics: Hamiltonian vector fields, Poisson brackets

**Status:** ✅ Defined, ⬜ Not yet loaded by default

### Optional Libraries (Planned)

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

All code in `stdlib/*.kleis` conforms to **Kleis Grammar v0.8**:

✅ Structure definitions  
✅ Operation declarations  
✅ Axiom syntax  
✅ Implementation blocks  
✅ Polymorphic types with `∀`  
✅ Type annotations with `:`  
✅ Library annotations with `@`  
✅ Algebraic data types with `data` (v0.4)  
✅ Pattern matching with `match` (v0.5)
✅ Derived functions in structures (v0.6)
✅ Mathematica-style calculus: `D()`, `Dt()`, `Limit()` (v0.7)
✅ Pattern guards: `n if n < 0 => ...` (v0.8)
✅ As-patterns: `Cons(h, t) as whole` (v0.8)
✅ Let destructuring: `let Point(x, y) = p in ...` (v0.8)  

---

## Status (December 9, 2025)

**Files Written:** ✅ `types.kleis`, `minimal_prelude.kleis`, `matrices.kleis`, `tensors.kleis`, `differential_forms.kleis`  
**Grammar:** ✅ v0.5 formalized (with pattern matching!)  
**Parser:** ✅ Pattern matching implemented  
**Type Inference:** ✅ Pattern matching type-checks  
**Evaluation:** ✅ Pattern matching evaluates  
**Exhaustiveness:** ✅ Missing case warnings  
**Matrix Operations:** ✅ Working with UI button  
**Recursive Unification:** ✅ Nested types unify correctly  
**Block Matrices:** ✅ Work via polymorphism  

**Files Loaded:** types.kleis, minimal_prelude.kleis, matrices.kleis  
**Files Defined (not loaded):** tensors.kleis

**Pattern Matching Functions:** ✅ Defined in `types.kleis`
- Boolean operations: `not`, `and`, `or`
- Option operations: `isSome`, `isNone`, `getOrDefault`, `mapOption`
- Result operations: `isOk`, `isErr`, `unwrapOr`
- List operations: `isEmpty`, `head`, `tail`
- Meta-level: `isScalarType`, `isVectorType`, `vectorDimension`

**Tensor Operations:** ✅ Defined in `tensors.kleis`
- Curvature tensors: Riemann, Ricci, Einstein, Weyl
- Connection: Christoffel symbols, covariant derivative
- Physics: Stress-energy, geodesics, Killing vectors
- Standard metrics: Minkowski, Schwarzschild, Kerr, FLRW

**Differential Forms:** ✅ Defined in `differential_forms.kleis`
- Cartan calculus: wedge (∧), exterior derivative (d), Hodge star (⋆)
- Interior product: contraction with vector fields
- Cartan's magic formula: ℒ_X = d ∘ ι_X + ι_X ∘ d
- Physics: Maxwell's equations in form language, Yang-Mills, symplectic mechanics  

---

## Next Steps

1. Implement parser for v0.3 syntax
2. Implement stdlib loader
3. Load prelude.kleis at server startup
4. Test type inference with stdlib context
5. Add optional libraries (quantum, pot)

---

**The standard library is now formal Kleis code that defines the type system!** 🎯

