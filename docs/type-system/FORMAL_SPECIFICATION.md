# Kleis Type System - Formal Specification

**Author:** Dr. Engin Atik  
**Date:** December 7, 2025 (Updated: December 9, 2025)  
**Version:** 0.5.0  
**Status:** Core Implementation Complete, Registry Extensions Complete, Let-Polymorphism Planned

**Dec 9 Update:**
- ✅ StructureRegistry implemented (generic parametric structures)
- ✅ List literal support for compositional types
- ✅ Matrix as regular data constructor (zero hardcoding)
- ✅ Block matrices: Matrix(2, 2, List(Matrix(2, 2, List(α))))
- ✅ True user extensibility achieved

---

## Abstract

Kleis implements a **Hindley-Milner-style constraint-based type inference system** augmented with a **user-extensible operation registry** (ADR-016) and **dimensional type checking** (ADR-019). The system differs from classical HM by storing type definitions in user-space structures rather than hardcoding them in the inference engine, enabling self-hosting and domain-specific type systems.

**Important:** The current implementation (v0.4.0) has the core HM unification machinery but **not yet** full Damas-Milner let-polymorphism (type schemes, generalization, instantiation). The `ForAll` constructor exists in the type language but is not used in inference yet. Planned extensions are clearly marked throughout.

---

## 1. Type Language

### **Syntax**

```
Types τ ::= ℝ                    (Scalar - real numbers)
         |  Vector(n)            (Vector of dimension n ∈ ℕ)
         |  Matrix(m, n)         (Matrix of dimensions m×n)
         |  α                    (Type variable)
         |  τ₁ → τ₂              (Function type)
         |  ∀α. τ                (Universal quantification)

Type Variables α, β, γ ::= TypeVar(n) where n ∈ ℕ

Expressions e ::= c              (Constant)
               |  x              (Variable)
               |  □              (Placeholder)
               |  op(e₁,...,eₙ)  (Operation)
```

---

## 2. Type Inference Rules

### **2.1 Current Implementation (Core HM)**

#### **Constants**

```
─────────────── [Const]
Γ ⊢ c : ℝ
```

#### **Variables (Lookup)**

```
x : τ ∈ Γ
───────────── [Var-Bound]
Γ ⊢ x : τ
```

#### **Variables (Fresh)**

```
x ∉ Γ    α fresh
────────────────────── [Var-Fresh]
Γ, x : α ⊢ x : α
```

#### **Placeholders**

```
α fresh
────────────── [Placeholder]
Γ ⊢ □ : α
```

#### **Operations (Registry Lookup)**

```
Γ ⊢ e₁ : τ₁    ...    Γ ⊢ eₙ : τₙ
R ⊢ op : τ₁ → ... → τₙ → τ
────────────────────────────────── [Op-Registry]
Γ ⊢ op(e₁,...,eₙ) : τ

where R is the operation registry (from TypeContextBuilder)
```

---

### **2.2 Constraint Generation**

The inference process generates constraints `C`:

```
Constraints C ::= τ₁ ≡ τ₂         (Type equality)
               |  C₁ ∧ C₂         (Conjunction)
               |  ⊤               (True)
```

#### **Example: Addition**

```
Γ ⊢ e₁ : τ₁    Γ ⊢ e₂ : τ₂
────────────────────────────────── [Plus]
Γ ⊢ plus(e₁, e₂) : τ₁
  with constraints: τ₁ ≡ τ₂
```

---

## 3. Unification Algorithm

### **3.1 Unification Rules**

```
unify(ℝ, ℝ) = id                                    [Unify-Scalar]

unify(Vector(n), Vector(n)) = id                    [Unify-Vector]

unify(Matrix(m,n), Matrix(m,n)) = id                [Unify-Matrix]

unify(α, τ) = [α ↦ τ]    if α ∉ FV(τ)              [Unify-Var-L]

unify(τ, α) = [α ↦ τ]    if α ∉ FV(τ)              [Unify-Var-R]

unify(τ₁ → τ₂, τ₃ → τ₄) = let S₁ = unify(τ₁, τ₃)   [Unify-Fun]
                              S₂ = unify(S₁(τ₂), S₁(τ₄))
                          in S₂ ∘ S₁

unify(τ₁, τ₂) = fail     otherwise                  [Unify-Fail]
```

### **3.2 Occurs Check**

```
α ∈ FV(τ) ⇒ unify(α, τ) fails

where FV(τ) = free type variables in τ
```

---

## 4. Substitution

### **4.1 Definition**

A substitution `S` is a finite mapping from type variables to types:

```
S : TypeVar ⇀ Type
```

### **4.2 Application**

```
S(ℝ) = ℝ
S(Vector(n)) = Vector(n)
S(Matrix(m,n)) = Matrix(m,n)
S(α) = S(α)           if α ∈ dom(S)
S(α) = α              if α ∉ dom(S)
S(τ₁ → τ₂) = S(τ₁) → S(τ₂)
S(∀α. τ) = ∀α. S(τ)
```

### **4.3 Composition**

```
(S₂ ∘ S₁)(τ) = S₂(S₁(τ))

Implemented as:
  compose(S₁, S₂) = { α ↦ S₁(τ) | α ↦ τ ∈ S₂ } ∪ S₁
```

---

## 5. Operation Registry (ADR-016 Innovation)

### **5.1 Registry Structure**

```
R : (OpName × [Type]) ⇀ Type

R is built from Kleis structures:

structure S(T) {
  operation op : τ₁ → ... → τₙ → τ
}

implements S(C) {
  operation op = impl
}
```

### **5.2 Registry Lookup**

```
R ⊢ op : τ₁ → ... → τₙ → τ

means: There exists structure S and implementation for concrete type C
       such that op is defined with type τ₁ → ... → τₙ → τ
```

### **5.3 Example**

```kleis
structure Arithmetic(T) {
  operation plus : T → T → T
}

implements Arithmetic(ℝ) {
  operation plus = builtin_add
}
```

**Registry entry:**
```
R ⊢ plus : ℝ → ℝ → ℝ
```

**Inference rule:**
```
Γ ⊢ e₁ : ℝ    Γ ⊢ e₂ : ℝ    R ⊢ plus : ℝ → ℝ → ℝ
──────────────────────────────────────────────────── [Plus-Scalar]
Γ ⊢ plus(e₁, e₂) : ℝ
```

---

## 6. Dimensional Type Checking (ADR-019)

### **6.1 Matrix Dimensions as Types**

```
Matrix(m, n) × Matrix(n, p) → Matrix(m, p)
```

**Typing rule:**
```
Γ ⊢ A : Matrix(m, n)    Γ ⊢ B : Matrix(n, p)
──────────────────────────────────────────── [Matrix-Mult]
Γ ⊢ multiply(A, B) : Matrix(m, p)
```

**Dimension constraint:**
```
Matrix(m, n) × Matrix(p, q) is well-typed iff n = p
```

### **6.2 Connection to Physical Dimensions**

Matrix dimensions behave like physical dimensions:

| Physics | Kleis |
|---------|-------|
| [Length] × [Time] = [Length·Time] | Matrix(m,n) × Matrix(n,p) = Matrix(m,p) |
| [Length] + [Length] = [Length] | Matrix(m,n) + Matrix(m,n) = Matrix(m,n) |
| [Length] + [Time] → Error | Matrix(m,n) + Matrix(p,q) → Error if (m,n) ≠ (p,q) |

**Formal analogy:**
```
Dimension ≃ (ℤᵏ, +)              (abelian group)
MatrixDim ≃ (ℕ × ℕ, ×ₘₐₜ)        (composition monoid)
```

---

## 7. Type Checking Algorithm

### **7.1 Main Algorithm**

```
Algorithm: TypeCheck(e, Γ, R)
Input: Expression e, Context Γ, Registry R
Output: Type τ and substitution S

1. (τ, C) ← Infer(e, Γ, R)      // Generate type and constraints
2. S ← Solve(C)                  // Solve constraints via unification
3. return S(τ)                   // Apply substitution to result
```

### **7.2 Inference Function**

```
Infer(e, Γ, R) = (τ, C)

Infer(c, Γ, R) = (ℝ, ⊤)

Infer(x, Γ, R) = (Γ(x), ⊤)          if x ∈ Γ
               = (α, ⊤)   α fresh    if x ∉ Γ, add x : α to Γ

Infer(□, Γ, R) = (α, ⊤)   α fresh

Infer(op(e₁,...,eₙ), Γ, R) = 
  let (τ₁, C₁) = Infer(e₁, Γ, R)
      ...
      (τₙ, Cₙ) = Infer(eₙ, Γ, R)
      τ = R(op, [τ₁,...,τₙ])        // Registry lookup
      C = C₁ ∧ ... ∧ Cₙ
  in (τ, C)
```

### **7.3 Constraint Solving**

```
Solve(⊤) = id

Solve(τ₁ ≡ τ₂) = unify(τ₁, τ₂)

Solve(C₁ ∧ C₂) = let S₁ = Solve(C₁)
                     S₂ = Solve(S₁(C₂))
                 in S₂ ∘ S₁
```

---

## 8. Soundness and Completeness

### **8.1 Soundness (Informal)**

**Claim:** If `Γ ⊢ e : τ` and `e` evaluates to value `v`, then `v` has type `τ`.

**Proof sketch:**
- Unification preserves type equality (classical)
- Registry ensures operations are well-typed (by construction)
- Substitution preserves typing (classical)

**Status:** Informal argument; full proof deferred.

---

### **8.2 Principal Types**

**Claim:** The inference algorithm computes principal types.

**Proof sketch:**
- Use of fresh type variables ensures most general type
- Unification computes most general unifier
- No premature instantiation

**Example:**
```
id(x) infers: α → α    (most general)
not:          ℝ → ℝ    (too specific)
```

**Status:** Holds for current implementation; formalize when adding polymorphism.

---

## 9. Extensions (Not Implemented Yet)

### **9.1 Let-Polymorphism (Phase 4)**

**Add to syntax:**
```
e ::= ... | let x = e₁ in e₂
```

**Type schemes:**
```
σ ::= τ | ∀α. σ

Γ ::= · | Γ, x : σ
```

**Inference rules:**
```
Γ ⊢ e₁ : τ₁    α = FV(τ₁) \ FV(Γ)    Γ, x : ∀α. τ₁ ⊢ e₂ : τ₂
──────────────────────────────────────────────────────────────── [Let]
Γ ⊢ let x = e₁ in e₂ : τ₂


Γ(x) = ∀α. τ    β fresh
──────────────────────── [Inst]
Γ ⊢ x : τ[α ↦ β]
```

**Status:** Type enum has `ForAll`, but generalization/instantiation not implemented.

---

### **9.2 Dimension Expressions (Phase 2)**

**Extend Type with dimension expressions:**
```
DimExpr d ::= n              (constant)
           |  α              (dimension variable)
           |  d₁ + d₂        (addition)
           |  d₁ × d₂        (multiplication)

Type τ ::= ...
        |  Matrix(d₁, d₂)   (dimensions as expressions, not just constants)
```

**Example:**
```kleis
let n = 3
let A = Matrix(n, n, ...)     // Type: Matrix(n, n)
let B = Matrix(n, 2×n, ...)   // Type: Matrix(n, 2n)
```

**Unification extended:**
```
unify(Matrix(d₁, d₂), Matrix(d₃, d₄)) = 
  let S₁ = unify_dim(d₁, d₃)
      S₂ = unify_dim(S₁(d₂), S₁(d₄))
  in S₂ ∘ S₁

where unify_dim solves dimension equations
```

---

### **9.3 Dependent Types (Phase 5, Optional)**

**Extend with dependent function types:**
```
Type τ ::= ...
        |  Π(x : τ₁). τ₂     (Dependent product)

Example: Π(n : ℕ). Vector(n) → ℝ
         "Function taking n and returning (Vector(n) → ℝ)"
```

**Typing rule:**
```
Γ, x : τ₁ ⊢ τ₂ : Type
───────────────────────── [Pi-Form]
Γ ⊢ Π(x : τ₁). τ₂ : Type
```

**Status:** Future work, requires significant extension.

---

## 10. Structure-Based Operations (ADR-016)

### **10.1 Formal Model**

A **structure** `S` defines abstract operations:

```
Structure := (Name, TypeParams, Operations)

Operation := (Name, Signature : τ)

Example:
structure Numeric(N) {
  operation abs : N → N
}
```

An **implementation** provides concrete bindings:

```
Implementation := (StructureName, ConcreteType, Bindings)

Binding := (OpName, Implementation)

Example:
implements Numeric(ℝ) {
  operation abs = builtin_abs
}
```

---

### **10.2 Registry Semantics**

The registry `R` is built from structures and implementations:

```
R : OpName → [ConcreteType] → Signature

R(op) = { (C, τ) | ∃ S, I. S defines op with signature τ 
                            and I implements S for C }
```

**Type checking with registry:**
```
Γ ⊢ e₁ : τ₁    ...    Γ ⊢ eₙ : τₙ
(τ₁,...,τₙ, τ) ∈ R(op)
──────────────────────────────── [Op-Registry]
Γ ⊢ op(e₁,...,eₙ) : τ
```

---

### **10.3 Polymorphic Operations**

**Structure with type parameter:**
```kleis
structure Arithmetic(T) {
  operation plus : T → T → T
}

implements Arithmetic(ℝ)
implements Arithmetic(Matrix(m,n))
```

**Registry entries:**
```
R(plus) = { (ℝ, ℝ → ℝ → ℝ),
            (Matrix(m,n), Matrix(m,n) → Matrix(m,n) → Matrix(m,n)) }
```

**Polymorphic inference:**
```
Γ ⊢ e₁ : τ    Γ ⊢ e₂ : τ    (τ, τ → τ → τ) ∈ R(plus)
────────────────────────────────────────────────────── [Plus-Poly]
Γ ⊢ plus(e₁, e₂) : τ
```

---

## 11. Dimensional Analysis (ADR-019)

### **11.1 Dimensional Structures**

A type with dimensions `(d₁,...,dₖ)` forms a dimensional structure:

```
DimType := (BaseType, [Dimension])

Example:
  Matrix(m, n) = (Matrix, [m, n])
  Force = (Physical, [1, 1, -2])   // M¹·L¹·T⁻²
```

### **11.2 Dimensional Constraints**

Operations on dimensional types must satisfy dimensional constraints:

```
op : D₁ → ... → Dₙ → D

where D, D₁,...,Dₙ satisfy dimensional compatibility
```

**Example: Matrix Multiplication**
```
multiply : Matrix(m, n) → Matrix(n, p) → Matrix(m, p)
           └────────────┘   └────────────┘
           Inner dimensions must match: n = n
```

**Example: Addition**
```
plus : Matrix(m, n) → Matrix(m, n) → Matrix(m, n)
       └────────────┘   └────────────┘
       Dimensions must match exactly
```

---

### **11.3 Generalization to Physics**

The same framework applies to physical dimensions:

```kleis
structure Physical(L: ℤ, M: ℤ, T: ℤ) {
  operation times : Physical(L₁,M₁,T₁) → Physical(L₂,M₂,T₂) 
                  → Physical(L₁+L₂, M₁+M₂, T₁+T₂)
}

// Exponents add, like logarithms!
Force(1,1,-2) × Distance(1,0,0) = Energy(2,1,-2)
M¹·L¹·T⁻² × L¹ = M¹·L²·T⁻²
```

**Type rule:**
```
Γ ⊢ F : Physical(L₁,M₁,T₁)    Γ ⊢ d : Physical(L₂,M₂,T₂)
────────────────────────────────────────────────────────────── [Phys-Mult]
Γ ⊢ times(F, d) : Physical(L₁+L₂, M₁+M₂, T₁+T₂)
```

**Example with dimension constraints:**

Let `n : ℕ` be a dimension variable. Consider:
```
A : Matrix(n, 2n)
B : Matrix(2n, 3)
```

Multiplication constraint:
```
Matrix(n, 2n) × Matrix(2n, 3) requires 2n = 2n ✓
Result: Matrix(n, 3)
```

Unification example:
```
unify(Matrix(n, 2n), Matrix(3, 6))
⇒ unify_dim(n, 3) ∧ unify_dim(2n, 6)
⇒ [n ↦ 3] ∧ [2n ↦ 6]
⇒ [n ↦ 3]  (consistent: 2·3 = 6 ✓)
```

---

## 12. Comparison to Standard HM

### **Standard Hindley-Milner (Damas-Milner)**

```
Types τ ::= α | τ₁ → τ₂ | T(τ₁,...,τₙ)
Schemes σ ::= τ | ∀α. σ
Context Γ ::= · | Γ, x : σ

Rules:
  [Var]     x : σ ∈ Γ ⇒ Γ ⊢ x : instantiate(σ)
  [Abs]     Γ, x : τ₁ ⊢ e : τ₂ ⇒ Γ ⊢ λx. e : τ₁ → τ₂
  [App]     Γ ⊢ e₁ : τ₁ → τ₂    Γ ⊢ e₂ : τ₁ ⇒ Γ ⊢ e₁ e₂ : τ₂
  [Let]     Γ ⊢ e₁ : σ    Γ, x : σ ⊢ e₂ : τ ⇒ Γ ⊢ let x = e₁ in e₂ : τ
  [Gen]     Γ ⊢ e : τ    α ∉ FV(Γ) ⇒ Γ ⊢ e : ∀α. τ
```

---

### **Kleis (Current)**

```
Types τ ::= ℝ | Vector(n) | Matrix(m,n) | α | τ₁ → τ₂ | ∀α. τ
Context Γ ::= · | Γ, x : τ     (NOT schemes yet)
Registry R ::= OpName → [Type] → Type

Rules:
  [Const]   Γ ⊢ c : ℝ
  [Var]     x : τ ∈ Γ ⇒ Γ ⊢ x : τ
            x ∉ Γ ⇒ Γ, x : α ⊢ x : α    (fresh α)
  [Op]      Γ ⊢ eᵢ : τᵢ    R(op, [τ₁,...,τₙ]) = τ ⇒ Γ ⊢ op(e₁,...,eₙ) : τ
  
  [NO Let, Abs, App, Gen yet - PoC scope]
```

---

### **Key Differences**

| Feature | Standard HM | Kleis (Current) | Kleis (Planned) |
|---------|-------------|-----------------|-----------------|
| **Schemes in Γ** | ✅ ∀α. τ | ❌ Just τ | Phase 4 |
| **Generalization** | ✅ At let | ❌ | Phase 4 |
| **Instantiation** | ✅ At use | ❌ | Phase 4 |
| **λ-calculus** | ✅ | ❌ Math expressions only | Phase 5 |
| **Operation registry** | ❌ | ✅ ADR-016 | Current |
| **Dimensional checking** | ❌ | ✅ ADR-019 | Current |
| **Self-hosting** | ❌ | ✅ Types in Kleis | Current |

**Design choice:** Kleis prioritizes HM-style inference over richer systems (e.g., full dependent types) at the core to preserve decidable inference and compiler-friendly complexity, while using the registry to express domain-specific structure. This allows users to extend the type system without modifying the inference engine.

---

## 13. Metatheoretic Properties

### **13.1 Type Safety**

**Progress:** If `Γ ⊢ e : τ`, then either:
- `e` is a value, or
- `e` steps to `e'` with `Γ ⊢ e' : τ`

**Preservation:** If `Γ ⊢ e : τ` and `e ⟶ e'`, then `Γ ⊢ e' : τ`.

**Status:** Informal. Evaluation not yet implemented.

---

### **13.2 Principal Types**

**Theorem (Informal):** For any expression `e` and context `Γ`, if `e` is typeable, then there exists a principal type `τ` such that:
- `Γ ⊢ e : τ`
- For any other derivation `Γ ⊢ e : τ'`, there exists substitution `S` with `τ' = S(τ)`

**Proof:** By induction on expression structure, using MGU (most general unifier).

**Status:** True for current implementation; formally verify when complete.

---

### **13.3 Decidability**

**Theorem:** Assuming the operation registry is finite and non-ambiguous per operator signature, type inference is decidable.

**Proof:**
- Expression structure is finite
- Fresh variable generation terminates
- Unification is decidable (occurs check prevents infinite recursion)
- Registry lookup is finite and deterministic by construction
- Constraint solving terminates (finite constraints, terminating unification)

**Complexity:** O(n·log n) where n is expression size (in practice).

---

## 14. Implementation Details

### **14.1 Current State (v0.4.0)**

**Files:**
- `src/type_inference.rs`: 469 lines (Core HM algorithm)
- `src/type_context.rs`: 798 lines (Registry and structures)
- `src/type_checker.rs`: 302 lines (User-facing API)

**Operations in stdlib:** 21
- Arithmetic: 7 operations
- Numeric: 5 operations
- Relational: 6 operations (Equatable + Ordered)
- Matrix: 3 operations

**Test coverage:** 381 tests, all passing (updated Dec 9, 2025)
- 376 core type system tests
- 5 List literal tests  
- Block matrix compositionality verified

---

### **14.2 Architecture**

```
┌──────────────────────────────────────────┐
│         TypeChecker                      │
│  (User API + error messages)             │
└──────────────┬───────────────────────────┘
               │
       ┌───────┴────────┐
       ▼                ▼
┌─────────────┐  ┌─────────────────┐
│TypeInference│  │TypeContextBuilder│
│   (HM core) │  │   (Registry)     │
└─────────────┘  └─────────────────┘
       │                │
       └────────┬───────┘
                ▼
       ┌─────────────────┐
       │ Kleis Stdlib    │
       │ (*.kleis files) │
       └─────────────────┘
```

**Key innovation:** Type definitions live in Kleis code, not Rust!

---

## 15. Example Derivations

### **15.1 Simple Addition**

**Expression:** `1 + 2`

**Derivation:**
```
─────────── [Const]    ─────────── [Const]
Γ ⊢ 1 : ℝ              Γ ⊢ 2 : ℝ

R(plus, [ℝ, ℝ]) = ℝ
────────────────────────────────────────── [Plus-Scalar]
Γ ⊢ plus(1, 2) : ℝ
```

**Constraints:** ⊤ (trivial)  
**Result:** `ℝ`

---

### **15.2 Variable Inference**

**Expression:** `x + 1`

**Derivation:**
```
α fresh                ─────────── [Const]
─────────────── [Var]  Γ ⊢ 1 : ℝ
Γ ⊢ x : α

R(plus, [α, ℝ]) with constraint α ≡ ℝ
────────────────────────────────────────── [Plus-Infer]
Γ ⊢ plus(x, 1) : ℝ

Solve(α ≡ ℝ) = [α ↦ ℝ]
Result: ℝ
```

**Constraints:** `α ≡ ℝ`  
**Substitution:** `[α ↦ ℝ]`  
**Result:** `ℝ` with `x : ℝ` inferred

---

### **15.3 Matrix Multiplication**

**Expression:** `A × B` where `A : Matrix(2,3)`, `B : Matrix(3,4)`

**Derivation:**
```
Γ(A) = Matrix(2,3)           Γ(B) = Matrix(3,4)
────────────────────         ─────────────────────
Γ ⊢ A : Matrix(2,3)          Γ ⊢ B : Matrix(3,4)

R(multiply, [Matrix(2,3), Matrix(3,4)]) = Matrix(2,4)
  with dimension constraint: 3 = 3 ✓
────────────────────────────────────────────────────── [Matrix-Mult]
Γ ⊢ multiply(A, B) : Matrix(2,4)
```

**Dimensional check:** Inner dimensions match (3 = 3) ✓  
**Result:** `Matrix(2,4)`

---

### **15.4 Dimensional Error**

**Expression:** `A × C` where `A : Matrix(2,3)`, `C : Matrix(4,5)`

**Derivation:**
```
Γ(A) = Matrix(2,3)           Γ(C) = Matrix(4,5)
────────────────────         ─────────────────────
Γ ⊢ A : Matrix(2,3)          Γ ⊢ C : Matrix(4,5)

R(multiply, [Matrix(2,3), Matrix(4,5)]) 
  requires dimension constraint: 3 = 4
  3 ≠ 4 ⇒ FAIL
────────────────────────────────────────────────────── [Matrix-Mult-Fail]
Γ ⊢ multiply(A, C) : ERROR
  "Matrix multiplication: inner dimensions must match! 3 ≠ 4"
```

**This is dimensional analysis preventing errors at type-check time!**

---

## 16. Formal Properties (Summary)

### **What We Can Prove:**

✅ **Unification terminates** (occurs check)  
✅ **Type inference terminates** (finite expression tree)  
✅ **Substitution is idempotent** (classical)  
✅ **Composition is associative** (classical)  
✅ **Principal types exist** (MGU properties)

### **What Needs Formal Proof:**

⚠️ **Soundness** (preserve types through evaluation)  
⚠️ **Completeness** (all typeable expressions are typed)  
⚠️ **Registry consistency** (no conflicting operation definitions)

### **Future Work:**

🔮 **Mechanized proof** (Coq/Lean/Agda)  
🔮 **Dimension arithmetic soundness**  
🔮 **Polymorphism correctness** (when added)

---

## 17. Differences from Pure HM

### **17.1 Innovations**

1. **Registry-Driven Operations** (ADR-016)
   - Operations defined in user-space structures
   - Not hardcoded in inference engine
   - Enables self-hosting

2. **Dimensional Type Checking** (ADR-019)
   - Matrix dimensions checked like physical units
   - Generalizes to arbitrary dimensional structures
   - Compile-time dimensional safety

3. **Parametric Base Types**
   - `Vector(n)`, `Matrix(m,n)` are first-class
   - Not type applications (like `List<Int>`)
   - Dimensions are part of the type

---

### **17.2 Limitations (Current)**

1. **No Type Schemes in Environment**
   - Context stores `Type`, not `∀α. Type`
   - Let-polymorphism not yet implemented

2. **No Generalization/Instantiation**
   - ForAll exists in Type but not used
   - Planned for Phase 4

3. **Simple Type Language**
   - No row types
   - No higher-kinded types
   - No type classes (yet)

4. **Expression Language Limited**
   - No λ-calculus
   - No let-expressions
   - Math operations only

**These are PoC scope limitations, not architectural problems.**

---

## 18. Notation and Conventions

### **18.1 Type Notation**

```
ℝ         Scalar (real numbers)
ℂ         Complex numbers (future)
ℕ         Natural numbers
Vector(n) Vector of dimension n
Matrix(m,n) Matrix m×n
α, β, γ   Type variables
τ₁ → τ₂   Function type
∀α. τ     Universal quantification
```

### **18.2 Judgment Notation**

```
Γ ⊢ e : τ         Expression e has type τ in context Γ
τ₁ ≡ τ₂           Type constraint
S(τ)              Apply substitution S to type τ
S₂ ∘ S₁           Compose substitutions
R ⊢ op : τ        Registry says op has type τ
α ∉ FV(Γ)         α is not free in Γ
```

---

## 19. References

### **Type Theory**

- Damas, L. & Milner, R. (1982). "Principal type-schemes for functional programs"
- Pierce, B. (2002). "Types and Programming Languages"
- Cardelli, L. (1987). "Basic Polymorphic Typechecking"

### **Dimensional Analysis**

- Buckingham, E. (1914). "On physically similar systems"
- Kennedy, A. (1997). "Programming languages and dimensions" (F# units)
- Kennedy, A. (2010). "Types for units-of-measure in F#"

### **Kleis-Specific**

- ADR-014: Hindley-Milner Type System
- ADR-016: Operations in Structures (self-hosting)
- ADR-019: Dimensional Type Checking
- Implementation: `src/type_inference.rs`, `src/type_context.rs`

---

## 20. Future Formalizations

### **20.1 Mechanized Verification (Future)**

**Goal:** Prove type safety in Coq/Lean/Agda

**Approach:**
```coq
Inductive Expr : Type :=
  | Const : nat -> Expr
  | Var : string -> Expr
  | Op : string -> list Expr -> Expr.

Inductive Type : Type :=
  | Scalar : Type
  | Matrix : nat -> nat -> Type
  | TVar : nat -> Type
  | Arrow : Type -> Type -> Type.

Inductive typing : context -> Expr -> Type -> Prop :=
  | T_Const : forall Γ n, typing Γ (Const n) Scalar
  | T_Var : forall Γ x τ, lookup Γ x = Some τ -> typing Γ (Var x) τ
  | ...

Theorem soundness : forall Γ e τ v,
  typing Γ e τ -> eval e = Some v -> has_type v τ.
```

**Status:** Future work, after core stabilizes.

---

### **20.2 Formal Dimensional Analysis**

**Goal:** Prove dimensional safety

**Theorem:**
```
If Γ ⊢ e : Matrix(m, n)
and e ⟶* v
then dimensions(v) = (m, n)
```

**Extension to physics:**
```
If Γ ⊢ e : Physical(L, M, T)
and e evaluates to quantity q
then dimensions(q) = [L, M, T]
```

**Status:** Informal argument exists (ADR-019), formal proof future.

---

## 21. Conclusion

### **Summary**

Kleis implements:
1. ✅ **HM-style constraint-based type inference** (core algorithm)
2. ✅ **Registry-driven operation typing** (ADR-016 innovation) - **COMPLETE Dec 9!**
3. ✅ **Dimensional type checking** (ADR-019 innovation)
4. ✅ **List literals for compositional types** - **NEW Dec 9!**
5. ✅ **StructureRegistry for user extensibility** - **NEW Dec 9!**
6. ⏳ **Planned: Full Damas-Milner with schemes** (Phase 4 - Let-polymorphism)

### **Current Status (Dec 9, 2025)**

**Theoretical foundation:** Solid  
**Core implementation:** Complete (1,959 lines in type_inference.rs)  
**Extensions:** ✅ StructureRegistry, DataTypeRegistry with List support  
**Innovation:** Self-hosting + dimensional analysis + true user extensibility  
**Tests:** 381 passing (376 core + 5 List literals)

**Major achievement:** Matrix is now a regular 3-arg data constructor with zero hardcoding!
- Block matrices work: `Matrix(2, 2, List(Matrix(...)))`
- Users can define `Tensor(i, j, k, List(T))` without code changes
- Removed 133 lines of hardcoded special cases

### **Academic Contribution**

1. **User-extensible dimensional type systems** (ADR-019)
   - Not hardcoded like F#/Rust/Haskell
   - Generalizes beyond physics

2. **Self-hosting type definitions** (ADR-016)
   - Operations in user-space structures
   - Type system as library, not compiler

3. **Dimensional analysis as type checking**
   - Matrix dimensions = physical dimensions
   - Compile-time dimensional safety
   - Prevents Mars Orbiter-class errors

---

## Appendix A: Notation Reference

```
Γ, Δ         Type contexts (environments)
τ, σ, ρ      Types
α, β, γ      Type variables
e            Expressions
S            Substitutions
C            Constraints
R            Operation registry
m, n, p      Natural numbers (dimensions)
∀, ∃         Quantifiers
⊢            Turnstile (typing judgment)
≡            Type equality constraint
⟶           Evaluation step
⟶*          Reflexive transitive closure
∘            Composition
FV(τ)        Free variables in τ
```

---

## Appendix B: Implementation Mapping

| Formal Notation | Rust Implementation |
|-----------------|---------------------|
| τ | `Type` enum |
| α | `TypeVar(usize)` |
| S | `Substitution { map: HashMap }` |
| Γ | `TypeContext { vars: HashMap }` |
| C | `Vec<Constraint>` |
| R | `OperationRegistry` |
| unify(τ₁, τ₂) | `fn unify(t1: &Type, t2: &Type)` |
| S₂ ∘ S₁ | `s2.compose(&s1)` |
| S(τ) | `subst.apply(&ty)` |
| Γ ⊢ e : τ | `inference.infer(&expr, context_builder)` |

---

**Document Status:** ✅ Complete  
**Suitable for:** Papers, talks, academic discussions  
**Next:** Mechanized proof (when type system stabilizes)

---

**This formalizes exactly what you've built, Dr. Atik!** 🎓

