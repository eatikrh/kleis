# ADR-014: Hindley-Milner Type System with Incremental Checking

## Status
Proposed (POC Implemented)

## Date
2024-12-05

## Context

Kleis is a **symbolic mathematics editor** where expressions remain unevaluated. Currently, expressions are **untyped** - the AST can represent mathematically invalid expressions like:
- Adding a matrix to a scalar: `A + 3`
- Differentiating with respect to a matrix: `d/dA[f(x)]`
- Index mismatch in tensor contraction: `g^{μν} T_{μν}`

These errors only manifest at render time or when a human reviews the output.

### The Challenge

We need a type system that:

1. **Verifies symbolic expressions** without requiring evaluation
2. **Supports polymorphism** - same operation (×) works for scalars, vectors, matrices
3. **Handles user-defined types** - mathematical (Matrix) AND domain-specific (PurchaseOrder)
4. **Infers types automatically** - users shouldn't write type annotations everywhere
5. **Provides helpful feedback** - guide users, don't block editing
6. **Tracks dimensions** - Matrix(m,n) × Matrix(n,p) → Matrix(m,p)
7. **Verifies axioms** - check algebraic structure laws hold

### Key Insight from Research

After studying GHC (Glasgow Haskell Compiler), we discovered:

**Type checking works on expression STRUCTURE, not VALUES.**

This means Haskell's Hindley-Milner type inference algorithm works **identically** for symbolic mathematics as it does for evaluated programs. The only difference is what happens AFTER type checking:
- Haskell: types → evaluate → value
- Kleis: types → verify → symbolic expression

## Decision

Kleis will adopt **Hindley-Milner type inference** with the following adaptations:

### 1. Core Type System from Haskell

#### Type Representation
```rust
enum Type {
    // Primitives
    Scalar,              // ℝ, ℂ, ℤ
    Bool,
    String,
    
    // Mathematical structures
    Vector(Box<Dimension>),        // Vector with size
    Matrix(Box<Dimension>, Box<Dimension>), // Matrix m×n
    Tensor(Vec<Dimension>),        // Multi-index tensor
    
    // User-defined types
    Named(String),                 // PurchaseOrder, Invoice, etc.
    Record {
        name: String,
        fields: HashMap<String, Type>,
    },
    
    // Type variables (for polymorphism)
    Var(TypeVar),                  // α, β, γ
    
    // Functions
    Function(Box<Type>, Box<Type>), // T₁ → T₂
    
    // Polymorphic types
    ForAll(TypeVar, Box<Type>),    // ∀α. T
    
    // Constrained types
    Constrained {
        var: TypeVar,
        constraint: Constraint,    // Monoid(α), Numeric(α)
        body: Box<Type>,
    },
}

enum Dimension {
    Const(usize),     // Known: 3
    Var(String),      // Unknown: n
}

enum Constraint {
    IsStructure(String, Type),  // Monoid(T), Numeric(T)
    HasProperty(String, Type),  // Symmetric(M), Hermitian(M)
    DimensionEq(Dimension, Dimension), // m = n
}
```

#### Type Inference Algorithm

Use Hindley-Milner with three phases:

1. **Generate constraints** from expression structure
   ```rust
   // x + y generates:
   // - x : α
   // - y : β
   // - Constraint: α = β
   // - Result: α
   ```

2. **Solve constraints** via unification
   ```rust
   // Unify: α = β
   // Substitute and check consistency
   ```

3. **Generalize** free variables to polymorphic types
   ```rust
   // α → ∀a. Numeric(a) ⇒ a
   ```

### 2. Algebraic Structures as Type Classes

Map Haskell's type classes to algebraic structures:

```kleis
// Define structure (like Haskell class)
structure Monoid(M) {
  element e : M
  operation (•) : M × M → M
  
  axiom identity:
    ∀x ∈ M. e • x = x ∧ x • e = x
    
  axiom associativity:
    ∀x y z ∈ M. (x • y) • z = x • (y • z)
}

// Implement for specific type (like Haskell instance)
implements Monoid(ℤ, +, 0) {
  verify identity    // Kleis checks axioms!
  verify associativity
}
```

Hierarchy:
```
Magma
  └─ Semigroup (associativity)
      └─ Monoid (identity)
          └─ Group (inverse)
              └─ AbelianGroup (commutativity)
                  └─ Ring (two operations)
                      └─ Field (division)
                          └─ VectorSpace
                              └─ HilbertSpace
```

### 3. Non-Intrusive Type Checking States

Type checking runs **incrementally** and never blocks the user. Five states:

#### 🔴 Error - Clear Violation
```kleis
A + 3  // where A : Matrix(3,3)
🔴 "Cannot add Matrix(3,3) to Scalar"
💡 "Did you mean A + 3·I?"
```
**State:** `TypeState::Error { message, suggestion }`

#### 🟡 Incomplete - Has Placeholders
```kleis
x + □
🟡 "Type: α + α → α (fill placeholder)"
```
**State:** `TypeState::Incomplete { partial_type, missing }`

#### 🟢 Polymorphic - Valid but Generic
```kleis
x + y  // no context
🟢 "Type: α + α → α (polymorphic)"
ℹ️ "Valid for any Numeric type"
```
**State:** `TypeState::Polymorphic { type_scheme, variables }`

#### 🔵 Concrete - Fully Resolved
```kleis
1 + 2
🔵 "Type: ℝ + ℝ → ℝ"
```
**State:** `TypeState::Concrete { ty }`

#### ⚪ Unknown - No Context
```kleis
x  // no definition
⚪ "Type: unknown (no context)"
```
**State:** `TypeState::Unknown`

### 4. Context Management

```rust
struct EditorTypeContext {
    // Built-in mathematical types
    builtin_types: HashMap<String, Type>,  // ℝ, ℂ, Vector, Matrix
    
    // User-defined types (math AND non-math)
    user_types: HashMap<String, TypeDefinition>,  // PurchaseOrder, Invoice
    
    // Variable bindings in current scope
    variables: HashMap<String, Type>,  // x : ℝ, order : PurchaseOrder
    
    // Type inference engine
    inference: TypeInference,
}
```

Context initialized with:
- Mathematical types: `ℝ, ℂ, ℕ, Vector(n), Matrix(m,n), Tensor(...)`
- Common operations: `+, -, ×, /, ∂, ∫, ∇`
- Algebraic structures: `Monoid, Group, Ring, Field, VectorSpace`

Users can add:
- Custom types: `PurchaseOrder, Particle, Contract`
- Custom operations: `calculateTotal, isOverdue`
- Custom structures: `Serializable, Comparable`

### 5. Mathematical Notation Syntax

Use mathematical symbols, not programming syntax:

| Haskell | Kleis |
|---------|-------|
| `f :: Int -> Int` | `f : ℤ → ℤ` |
| `forall a. a -> a` | `∀a. a → a` |
| `class Monoid` | `structure Monoid` |
| `instance Monoid` | `implements Monoid` |

Preserve Kleis' multiple equality types:
- `define` - Definition (by fiat)
- `assert` - Algebraic equality (provable)
- `equiv` - Structural equivalence (isomorphic)
- `approx` - Approximate equality (numerical)

## Consequences

### Positive

1. **Type Safety for Symbolic Math**
   - Catch errors before rendering: `Matrix + Scalar` → ERROR
   - Verify dimensional consistency: `Matrix(3,4) × Matrix(5,2)` → ERROR
   - Track tensor index types: `g^{μν} T_{μν}` → ERROR (should be `g^{μν} T_μν`)

2. **Automatic Type Inference**
   - Users write: `define f(x) = x²`
   - Kleis infers: `f : ℝ → ℝ`
   - No verbose annotations needed

3. **Polymorphic Operations**
   - Single `×` operation for scalars, dot product, matrix multiply
   - Type system dispatches correctly based on inferred types
   - Same operation works across domains (math, business, physics)

4. **Universal Applicability**
   - Mathematical types: `Vector, Matrix, Tensor`
   - Business types: `PurchaseOrder, Invoice`
   - Physics types: `Particle, Force, Field`
   - Same type system machinery for all domains

5. **Non-Intrusive UX**
   - Type checking never blocks editing
   - Real-time feedback with 5 clear states
   - Helpful suggestions, not just errors
   - Progressive refinement: Unknown → Polymorphic → Concrete

6. **Axiom Verification**
   - Check algebraic laws: `Monoid(ℤ, +, 0)` verifies associativity
   - Check business rules: `PurchaseOrder.total = subtotal + tax`
   - Unique to Kleis (Haskell doesn't verify axioms)

7. **Proven Technology**
   - Hindley-Milner has 40+ years of research
   - Production-tested in Haskell, OCaml, F#, Rust
   - Well-understood algorithms and properties

### Negative

1. **Implementation Complexity**
   - Need full unification algorithm
   - Need constraint solver
   - Need type error reporting with suggestions
   - Estimated: 3000+ lines of code

2. **Performance Concerns**
   - Type inference can be O(n²) in worst case
   - Need caching for real-time feedback
   - Large documents may have latency
   - Mitigation: Incremental checking, debouncing (300ms)

3. **Learning Curve**
   - Users need to understand type errors
   - Polymorphic types may be confusing initially
   - Documentation and examples critical
   - Mitigation: Clear error messages, visual indicators

4. **Type Error Messages**
   - Can be cryptic ("Cannot unify α with β")
   - Need human-friendly translations
   - Example: "Cannot add Matrix(3,3) to Scalar" instead of "Cannot unify Matrix(3,3) with Scalar"

### Neutral

1. **Breaking Changes**
   - Existing expressions may show type errors
   - Migration needed for incompatible expressions
   - Can start with warnings-only mode

2. **Optional vs Required**
   - Should type annotations be required or optional?
   - Decision: Optional by default, required for exports/libraries

## Alternatives Considered

### Alternative 1: No Type System (Status Quo)

**Pros:**
- Simple implementation
- No learning curve
- Fast (no inference)

**Cons:**
- No error detection
- Can't verify dimensional consistency
- Can't support polymorphic operations
- Can't verify axioms
- Errors only caught at render time or by humans

**Rejected:** Doesn't support Kleis' vision of verified mathematics.

### Alternative 2: Simple Type Checking (No Inference)

Require explicit type annotations everywhere:

```kleis
define f(x : ℝ) : ℝ = x²
define A : Matrix(3, 3) = ...
```

**Pros:**
- Simpler implementation
- Explicit types aid understanding

**Cons:**
- Verbose and tedious
- Doesn't match mathematical writing style
- Still doesn't support polymorphism well

**Rejected:** Too verbose for practical use.

### Alternative 3: Dependent Type Theory (Coq/Agda Style)

Full dependent types with proof obligations:

```kleis
define f(x : ℝ) : {y : ℝ | y > 0} = x²
```

**Pros:**
- Maximum expressiveness
- Can express complex constraints
- Proof-carrying code

**Cons:**
- Extremely complex to implement
- High learning curve
- May require user to write proofs
- Overkill for most use cases

**Rejected:** Too heavyweight. May revisit for Phase 4.

## Implementation Phases

### Phase 1: Core Type Inference (Weeks 1-2)
- ✅ Basic type representation (Scalar, Vector, Matrix)
- ✅ Type variables and substitution
- ✅ Unification algorithm
- ✅ Constraint generation for basic operations (+, -, ×, /)
- ✅ Working POC (`src/type_inference.rs`)

**Status:** ✅ Complete (POC working)

### Phase 2: Operation Coverage (Weeks 3-4)
- ⬜ Polymorphic multiplication rules
- ⬜ Vector operations (dot, cross, norm)
- ⬜ Matrix operations (det, trace, transpose)
- ⬜ Calculus operations (∂, ∫, ∇)
- ⬜ Tensor index checking

### Phase 3: Algebraic Structures (Weeks 5-6)
- ⬜ Structure definitions (Monoid, Group, Ring, Field)
- ⬜ Structure hierarchy and inheritance
- ⬜ Instance resolution (`implements` keyword)
- ⬜ Constraint solving for structures

### Phase 4: Incremental Checking & UX (Weeks 7-8)
- ⬜ TypeState enum (Error, Incomplete, Polymorphic, Concrete, Unknown)
- ⬜ EditorTypeContext with real-time checking
- ⬜ Visual feedback (colored indicators, tooltips)
- ⬜ API endpoints (`/api/type_check`)
- ⬜ Frontend integration

### Phase 5: User-Defined Types (Weeks 9-10)
- ⬜ Record types (struct-like)
- ⬜ Sum types (enum-like)
- ⬜ Type aliases
- ⬜ Field access type checking
- ⬜ Support for non-mathematical domains (PurchaseOrder, etc.)

### Phase 6: Dependent Types (Weeks 11-12)
- ⬜ Dimension tracking (Matrix(m,n) with type-level dimensions)
- ⬜ Type-level computation
- ⬜ Dimension constraint solving
- ⬜ Index compatibility checking for tensors

### Phase 7: Axiom Verification (Weeks 13-14)
- ⬜ Axiom representation in type system
- ⬜ Symbolic axiom checking
- ⬜ Instance verification (`verify` keyword)
- ⬜ Violation reporting with counterexamples

## Rationale

### Why Hindley-Milner?

1. **Proven Algorithm** - 40+ years of research and production use
2. **Automatic Inference** - Minimal user annotations
3. **Principal Types** - Always finds most general type
4. **Works Symbolically** - Doesn't require evaluation
5. **Supports Polymorphism** - Single operation, multiple types

### Why from Haskell?

1. **Most mature implementation** - GHC is production-quality
2. **Type classes** map directly to algebraic structures
3. **Higher-kinded types** enable generic operations (Functor, Monad)
4. **Extensive research** on extensions (dependent types, GADTs)
5. **Well-documented** algorithms and design decisions

### Why Incremental States?

1. **Non-blocking UX** - Users can continue editing during type errors
2. **Progressive refinement** - Unknown → Polymorphic → Concrete
3. **Helpful feedback** - Visual indicators guide users
4. **Matches workflow** - Build expression step-by-step

### Why Support User-Defined Types?

1. **Universal verification** - Same system for math AND business
2. **Domain flexibility** - Not just for mathematics
3. **Future-proof** - Enables Kleis to verify ANY structured domain
4. **Consistent experience** - Same type system everywhere

## Examples

### Example 1: Mathematical Expression

```kleis
// User builds expression visually:
Step 1: x + □
  State: 🟡 Incomplete
  Type: α + α → α (fill placeholder)

Step 2: x + 1
  State: 🟢 Polymorphic
  Type: α + α → α (inferred x : α)

Step 3: Context adds "x : ℝ"
  State: 🔵 Concrete
  Type: ℝ + ℝ → ℝ
```

### Example 2: Type Error Detection

```kleis
// User types: A + 3 (where A : Matrix(3,3))
State: 🔴 Error
Message: "Cannot add Matrix(3,3) to Scalar"
Suggestion: "Did you mean A + 3·I?"

// User fixes: A + 3·I
State: 🔵 Concrete
Type: Matrix(3,3)
```

### Example 3: Polymorphic Operation

```kleis
// Define: double(x) = x + x
Inferred type: ∀a. Numeric(a) ⇒ a → a

// Use with different types:
double(5)        → Type: ℝ (concrete)
double(v)        → Type: Vector(3) (where v : Vector(3))
double(A)        → Type: Matrix(3,3) (where A : Matrix(3,3))

// All valid! Same operation, multiple types
```

### Example 4: User-Defined Business Type

```kleis
// Define business type
structure PurchaseOrder {
  orderId : String
  items : List(LineItem)
  total : Money
  
  axiom has_items: length(items) > 0
  axiom total_correct: total = Σᵢ items[i].lineTotal
}

// Use in expression
define order : PurchaseOrder = ...
define total_amount = order.total

State: 🔵 Concrete
Type: Money
```

### Example 5: Dimensional Safety

```kleis
// Matrix multiplication with dimension checking
define A : Matrix(3, 4)
define B : Matrix(4, 2)
define C = A × B

State: 🔵 Concrete
Type: Matrix(3, 2) ✓

// Dimension mismatch
define D : Matrix(5, 2)
define bad = A × D

State: 🔴 Error
Message: "Matrix dimensions incompatible: (3,4) × (5,2)"
Info: "Inner dimensions must match: 4 ≠ 5"
```

## Proof of Concept Status

**Location:** `src/type_inference.rs`, `examples/type_inference_demo.rs`

**Working:**
- ✅ Type representation (Scalar, Vector, Matrix, Var, Function)
- ✅ Type inference for basic operations
- ✅ Constraint generation
- ✅ Unification algorithm
- ✅ Substitution and solving

**Demo Results:**
```
Example 1: Const("42") → Inferred type: ℝ
Example 3: x + 1 → Inferred type: ℝ
Example 4: x + y → Inferred type: α1 (polymorphic!)
Example 8: (x + 1) / 2 → Inferred type: ℝ
```

## Related Decisions

- **ADR-002**: Evaluation vs Simplification - Type checking happens before either
- **ADR-013**: Paper-Level Scope Hierarchy - Type context follows scope rules
- **POC**: Type inference working on current Kleis AST

## References

### Papers
- "Principal type-schemes for functional programs" - Damas & Milner (1982)
- "A Theory of Type Polymorphism in Programming" - Milner (1978)
- "Typing Haskell in Haskell" - Mark P. Jones (1999)
- "OutsideIn(X)" - Vytiniotis et al. (2011) - GHC's actual algorithm

### Implementation References
- GHC source: `~/Documents/git/cee/haskell/ghc/compiler/GHC/Tc/`
- "Typing Haskell in Haskell": https://gist.github.com/chrisdone/0075a16b32bfd4f62b7b

### Kleis Documentation
- `docs/type-system/HASKELL_TYPE_SYSTEM_LESSONS.md`
- `docs/type-system/TYPE_SYSTEM_SIMPLIFIED.md`
- `docs/type-system/TYPES_FOR_SYMBOLIC_MATH.md`
- `docs/type-system/HASKELL_TYPES_FOR_SYMBOLIC_MATH.md`
- `docs/type-system/USER_DEFINED_TYPES.md`
- `docs/type-system/INCREMENTAL_TYPE_CHECKING.md`
- `docs/type-system/TYPE_INFERENCE_POC.md`

## Open Questions

1. **Type Annotation Syntax**
   - Should we use `:` or `::` for type annotations?
   - Decision: Use `:` (mathematical convention)

2. **Error Recovery**
   - How aggressive should we be in suggesting fixes?
   - Should we auto-fix obvious errors?
   - Decision: Suggest but don't auto-fix (user control)

3. **Performance Threshold**
   - What's acceptable latency for type checking?
   - Decision: < 100ms for real-time, < 1s for thorough

4. **Axiom Verification Depth**
   - Full theorem proving or pattern matching?
   - Decision: Start with pattern matching, expand later

5. **Type Inference for Templates**
   - Should palette templates have pre-computed types?
   - Decision: Yes, cache common template types

## Migration Strategy

### Stage 1: Opt-In (Months 1-2)
- Type checking available but optional
- No breaking changes
- Users can enable via flag: `@enable_types`
- Warning mode only

### Stage 2: Warnings (Months 3-4)
- Type checking enabled by default
- Shows warnings for type errors
- Doesn't prevent rendering
- Allows users to adapt

### Stage 3: Errors (Months 5-6)
- Type checking enforced
- Prevents invalid expressions
- Full integration with editor
- Migration guide for fixing errors

## Success Criteria

1. ✅ Type inference working for 90%+ of mathematical expressions
2. ✅ < 100ms type checking latency for real-time feedback
3. ✅ < 5% false positive rate (incorrect type errors)
4. ✅ User-defined types working for 3+ domains (math, business, physics)
5. ✅ Visual feedback integrated in editor
6. ✅ 90%+ of users find type checking helpful (survey)

## Conclusion

Kleis will adopt Hindley-Milner type inference from Haskell, adapted for symbolic mathematics with:

- **Type inference** - Automatic, minimal annotations
- **Algebraic structures** - Monoid, Group, Ring, Field hierarchy
- **Incremental checking** - Non-blocking, helpful feedback
- **Universal types** - Mathematical AND user-defined domains
- **Axiom verification** - Unique to Kleis

This positions Kleis as: **"Haskell's type system for mathematicians, not programmers."**

The POC is working. Next: expand operation coverage and integrate with visual editor.

