# Next Session Tasks

## Evaluator Hygiene Plan (Branch: evaluator-hygiene) ✅ DONE

Goal: address capture-avoidance, span preservation, and lazy conditionals with full test coverage.

### Scope
1. **Capture-avoiding substitution for all binders**
   - Filter substitutions under `Quantifier` variables.
   - Filter substitutions in `MatchCase` guards and bodies by pattern-bound vars.
2. **Preserve spans in built-in operations**
   - Keep original `span` when rebuilding `Expression::Operation` for non-user-defined functions.
3. **Lazy conditional evaluation in symbolic `eval_internal`**
   - Evaluate condition only.
   - If condition reduces to `true/false`, evaluate the selected branch.
   - Otherwise return a conditional with unevaluated branches (or weak-head as needed).

### Tests to add
1. **Substitution hygiene**
   - Quantifier: ensure `∀ x . ...` blocks substitution for `x` in body/where.
   - MatchCase: ensure pattern-bound vars are not substituted in guard/body.
2. **Span preservation**
   - Built-in op span survives `eval_internal` when no user-defined function applies.
3. **Lazy conditionals**
   - Branches remain unevaluated when condition is symbolic.
   - Only selected branch evaluates when condition is concrete `true/false`.

### Steps
1. Implement binder-filtering helpers in `Evaluator::substitute`.
2. Preserve span in built-in operation reconstruction.
3. Make conditional evaluation lazy with truthy short-circuit.
4. Add tests covering each behavior.
5. Run test suite for affected modules.

## Type System Gap: Tensor Operation Signatures

The tensor operations used in the Equation Editor (`index_mixed`, `tensor_lower_pair`, etc.) render correctly but lack type signatures in the stdlib. This causes the type checker to return unresolved type variables like `Var(TypeVar(1))` instead of proper tensor types.

**Examples that show this gap:**
- `g_{μν}` - renders correctly but type is `Var(TypeVar(1))`
- `T^i_j` - renders correctly but type is `Var(TypeVar(1))`

**Fix needed:** Add tensor operation signatures to `stdlib/tensors.kleis`:

```kleis
// Hypothetical type signatures for tensor operations
operation index_mixed : (T: Tensor(n), upper: Index, lower: Index) → Tensor(n)
operation tensor_lower_pair : (T: Tensor(n), μ: Index, ν: Index) → Tensor(n)
operation tensor_upper_pair : (T: Tensor(n), μ: Index, ν: Index) → Tensor(n)
```

This would allow the type checker to properly infer tensor types for indexed expressions.

---

## Z3 Context Polymorphism Collision ✅ DONE

**Problem:** Z3 backend caches function declarations by name. Polymorphic ops like `let_simple`
get locked to the first instantiated signature (e.g., `Int × Int → Int`), then later calls
with `Matrix` arguments fail with a sort mismatch.

**Observed in Equation Editor:** `Verify` on matrix `let_simple` after scalar `let_simple`.

**Resolution (current):** Use fresh Z3 context per verification run or monomorphize
operation names in Z3 (`let_simple$Matrix3x3`, etc.). (Decision still open.)

---

## Recent Parser/Type-Inference Fixes ✅ DONE

- **Type param parsing:** keep simple identifiers as `Named` (only arithmetic stays `DimExpr`).
- **Matrix constructor typing:** infer element type from list contents, not `List(T)` itself.
- **Signature interpreter:** evaluate `DimExpr` in Nat params (supports `2^n`, etc.).

---

## Type Constructors + Z3 Impact Plan

Goal: add higher‑kinded type constructors (e.g., `M : Type → Type`) without breaking HM inference or Z3.

### Scope
1. **Kinds in the AST**
   - Introduce `KindExpr` (`Type`, `Nat`, `Kind → Kind`) and store in `TypeParam.kind`.
   - Extend `TypeExpr` to carry kinded params where needed.
2. **Type representation updates**
   - Add a `Type::App(Box<Type>, Box<Type>)` (or `Type::Con` + `Type::App`) for type application.
   - Extend `TypeVar` with optional kind.
3. **Kind checking + inference boundaries**
   - Add kind checking for structure params and `TypeExpr::Parametric`.
   - Keep HM unification first‑order, but kind‑aware (reject ill‑kinded unification).
4. **Signature interpreter updates**
   - Bind type constructor params separately from type params.
   - Unify `TypeExpr::Parametric` against `Type::App` (instead of only `Type::Data`).
5. **Z3 boundary strategy**
   - Choose between:
     - **Monomorphization**: fully instantiate `M(A)` before translation; reject polymorphic SMT goals.
     - **Encoding**: represent `Type` as a first‑order sort and `App(M, A)` as a function; adds axioms for injectivity if needed.

### Z3 interaction policy
- Implement full encoding of `Type`/`App` in Z3 (no monomorphization-only path).

### Touch points
- `kleis_ast.rs`: `TypeParam.kind`, `TypeExpr` (add kind nodes)
- `kleis_parser.rs`: parse `KindExpr` in type params
- `type_inference.rs`: add `Type::App`, kind‑aware unification
- `type_checker.rs`: enforce kind checking on declarations
- `type_context.rs`: replace string‑keyed type lookups with canonicalized type expressions
- `typed_ast.rs`: propagate `Type::App` through typed AST helpers
- `signature_interpreter.rs`: unify `TypeExpr::Parametric` with `Type::App`
- `solvers/z3/*`: ensure types are fully instantiated before translation

### Open questions
1. Do we require explicit kind annotations (`M : Type → Type`) or infer them?
2. Do we allow partial application of type constructors in user code?
3. Should Z3 ever see polymorphic types, or enforce monomorphization at boundary?

---

## Kinding + Type-Level Constraints + Typed Identity Plan

Goal: enforce proper kinds (`Type`, `Nat`, etc.), solve type‑level equalities (e.g., `n = m`), and expose a dimensioned identity element that the type system can infer via context or annotation.

### 1) Proper kinding
- Replace `TypeParam.kind: Option<String>` with `KindExpr` (`Type`, `Nat`, `String`, `Kind → Kind`). ✅ DONE
- Parse kind annotations in structure/type params (e.g., `m: Nat`, `T: Type`, `M: Type → Type`). ✅ DONE
- Add kind checking for:
  - `TypeExpr::Parametric` application arity/kind match
  - `TypeExpr::ForAll` variable kinds
  - `TypeExpr::Named` resolution (fail on ill‑kinded uses)

### 2) Type‑level constraint solving
- Reuse `DimExpr` and add a constraint set for equalities (`n = m`, `p + r = q`).
- Extend unification to emit constraints instead of only positional binding.
- Add a small `DimExpr` solver:
  - normalize (`n + 0 → n`, `n + 1 = m + 1 → n = m`)
  - constant fold (`2+3=5`, `2*n=2*n`)
  - keep symbolic constraints if unsolved
- Surface unsolved constraints as type errors when required by a signature.

### 3) Typed identity (dimension‑carrying)
- Add a structure‑scoped identity element (no top‑level ops):
  - `MatrixUnits(n: Nat, T)` with
    - `left_identity : Matrix(n, n, T)`
    - `right_identity : Matrix(n, n, T)`
    - axioms `left_unit`, `right_unit`, and optional `square_units_equal` (`m=n` case)
- Bind implementations to evaluator builtins (`eye/identity`) via `implements`.
- In usage, rely on **context or explicit annotation**:
  - `left_identity : Matrix(2^n, 2^n, ℂ)`

### 4) Z3 encoding impact
- Encode type‑level Nat constraints as Z3 equalities.
- For identity, prefer axiom‑level characterization:
  - `component(I, i, j) = delta(i, j)` (uses existing Kronecker delta)
- Ensure solver rejects ill‑kinded `Matrix(m,n,T)` uses early.

### Touch points
- `kleis_ast.rs`: introduce `KindExpr`, update `TypeParam`
- `kleis_parser.rs`: parse `KindExpr` in params
- `type_inference.rs`: carry kinds on `TypeVar`, emit constraints
- `signature_interpreter.rs`: collect/solve `DimExpr` constraints
- `type_context.rs`: propagate constraints through operation lookup
- `solvers/z3/*`: add Nat‑equality constraints and optional delta axioms

## Editor Tensor Format: Update to New Convention ✅ DONE

**Status:** Completed in `feature/migrate-tensor-format` branch (merged).

Migrated old tensor templates (`subsup`, `tensor_mixed`, `tensor_upper_pair`, `tensor_lower_pair`) to use the new format with `kind: 'tensor'` and `indexStructure` metadata. Updated `editor_node_to_expression` to treat tensors as formatting-only (returns base symbol for Z3).

## Differential Geometry Verifier (Long-term Goal)

Currently, Z3 treats tensor symbols as uninterpreted constants (scalars by default). This is sufficient for:
- ✅ Type checking tensor expressions
- ✅ Symbol equality verification

But it cannot verify deep tensor semantics like:
- Einstein's field equations: `G_μν + Λg_μν = κT_μν`
- Tensor transformation rules: `T'^μν = (∂x'^μ/∂x^α)(∂x'^ν/∂x^β)T^αβ`
- Bianchi identity: `∇_[λ R_ρσ]μν = 0`
- Metric contraction: `g^μρ g_ρν = δ^μ_ν`

**Required for full differential geometry verification:**

1. **Tensor axioms in Z3:**
   - Index structure (covariant/contravariant)
   - Metric tensor properties (symmetry, signature)
   - Contraction rules
   - Covariant derivative semantics

2. **Transformation rules:**
   - How tensors transform under coordinate changes
   - Christoffel symbol transformation (non-tensorial)

3. **Curvature tensors:**
   - Riemann tensor symmetries
   - Ricci tensor as contraction
   - Einstein tensor definition

This is a significant undertaking - essentially building a differential geometry verifier on top of Z3.

---

## Literature Survey: Formal Differential Geometry

### Classical Sources

| Source | Approach | Relevance |
|--------|----------|-----------|
| **Bourbaki** (Variétés différentielles) | Abstract, coordinate-free, category-theoretic | Rigorous but hard to encode in SMT |
| **Hilbert** (Grundlagen der Geometrie) | Axiomatic foundations | Influenced all formalization |
| **Courant & Hilbert** (Mathematical Physics) | Concrete tensor index notation | Good for computable axioms |
| **do Carmo** (Riemannian Geometry) | Standard graduate text | Explicit formulas |
| **Misner, Thorne, Wheeler** (Gravitation) | Physics-focused, GR axioms | Index notation reference |

### Proof Assistant Formalizations

#### Isabelle/HOL
- **HOL-Analysis** - Multivariate calculus foundations
- **Analysis-Manifolds** - Smooth manifolds (partial)
- Limited Riemannian geometry work

#### Coq
- **CoRN** - Constructive Real Numbers (foundation)
- **Coquelicot** - Real analysis library
- **GeoCoq** - Tarski's geometry axioms (Euclidean, not Riemannian)
- **UniMath** - Univalent foundations, some manifold work

#### Lean (mathlib) - Most Active
- `Mathlib.Geometry.Manifold` - Smooth manifolds
- `Mathlib.Geometry.Manifold.TangentBundle` - Tangent spaces
- `Mathlib.Geometry.Manifold.ContMDiff` - Smooth maps
- Working toward: connections, curvature (as of 2024)

#### Z3 / SMT Solvers
- **Limited work on geometry**
- Z3 has `RealArith` for real analysis
- **GeoGebra + SMT** - Some Euclidean geometry verification
- **MetiTarski** - Real-valued special functions
- **No known Riemannian geometry work**

### Why SMT is Challenging for Differential Geometry

1. Tensors are higher-order (functions on functions)
2. Index manipulation is symbolic, not arithmetic
3. Covariant derivatives involve limits/continuity
4. Coordinate transformations are diffeomorphisms (infinite-dimensional)

### Kleis Engineering Approach (Z3-focused)

**Key Decision:** We are NOT building a proof assistant. Isabelle/Lean/Coq require manual proof writing (Isar, tactics). Kleis takes an **engineering approach**:

1. **Encode proven theorems as Z3 axioms** - pre-load established DG identities
2. **User enters formulas** - e.g., POT Projection kernel expressions
3. **Z3 checks automatically** - does this formula violate any axiom?
4. **Report violations** - VALID / INVALID with counterexample

**Prior Work:**
- Isabelle solver backend exists (hibernated) - requires Isar proofs
- Lean/Coq would have same problem

**Z3 Encoding Strategy:**

1. **Tensor index algebra** - tractable, pure symbolic
2. **Metric axioms** - symmetry, inverse, signature as assertions
3. **Curvature identities** - Bianchi, Ricci symmetry as constraints
4. **Covariant derivative** - challenging, may need approximations

**Focus on algebraic identities first** (Bianchi, symmetries) - these are decidable in Z3.

---

## Progress: TensorComponents Structure ✅ DONE

**Problem Found:** Axioms in `stdlib/tensors.kleis` use `component`, `component3`, `component4` but these were NOT declared as operations. Z3 defaulted to `Int → Int` signatures.

**Fix Applied:** Added `TensorComponents(dim: Nat)` structure with proper type signatures:

```kleis
structure TensorComponents(dim: Nat) {
    operation component  : Tensor(0, 2, dim, ℝ) → Nat → Nat → ℝ
    operation component3 : Tensor(1, 2, dim, ℝ) → Nat → Nat → Nat → ℝ
    operation component4 : Tensor(1, 3, dim, ℝ) → Nat → Nat → Nat → Nat → ℝ
}
```

**Result:** Z3 can now properly type-check and apply:
- `metric_symmetric` axiom
- `riemann_antisym_34` axiom  
- `riemann_bianchi_1` axiom (First Bianchi identity)
- `christoffel_symmetric` axiom

---

## Missing Tensor Operations (Priority Order)

### HIGH Priority - Core Index Algebra

| Operation | Signature | Why Needed |
|-----------|-----------|------------|
| `delta` | `Nat → Nat → ℝ` | Kronecker delta: `g^{μρ} g_{ρν} = δ^μ_ν` |
| `symmetrize` | `Tensor → List(Nat) → Tensor` | `g_{(μν)} = g_{μν}`, Young tableaux |
| `antisymmetrize` | `Tensor → List(Nat) → Tensor` | `R_{[μνρσ]}`, differential forms |

**Axioms enabled:**
```kleis
// Metric inverse identity
axiom metric_inverse_identity : ∀ g : Tensor(0, 2, dim, ℝ) .
    ∀ μ : Nat . ∀ ν : Nat .
    sum_over(λ ρ . times(component(inverse(g), μ, ρ), component(g, ρ, ν)), 0, dim) = delta(μ, ν)

// Metric symmetry via symmetrize
axiom metric_is_symmetric : ∀ g : Tensor(0, 2, dim, ℝ) .
    symmetrize(g, [0, 1]) = g

// Riemann first pair antisymmetry
axiom riemann_antisym_12 : ∀ R : Tensor(0, 4, dim, ℝ) .
    antisymmetrize(R, [0, 1]) = R
```

### MEDIUM Priority - Covariant Calculus

| Operation | Signature | Why Needed |
|-----------|-----------|------------|
| `nabla_component` | `Tensor(1,3) → Nat → ... → ℝ` | Second Bianchi: `∇_{[λ} R_{ρσ]μν} = 0` |
| `levi_civita` | `Nat → Nat → Nat → Nat → ℝ` | ε-tensor for volume, duality |
| `contract_indices` | `Tensor → Nat → Nat → Tensor` | Specific index pair contraction |

### LOW Priority - Convenience

| Operation | Signature | Why Needed |
|-----------|-----------|------------|
| `permute_indices` | `Tensor → List(Nat) → Tensor` | Index reordering |
| `trace_free` | `Tensor → Tensor` | Weyl tensor construction |
| `dual` | `Tensor → Tensor` | Hodge dual for tensors |

---

## Implementation Status

- [x] `component`, `component3`, `component4` - ✅ DONE
- [x] `delta` - ✅ DONE (KroneckerDelta structure)
- [x] `symmetrize2` - ✅ DONE (IndexSymmetrization structure)
- [x] `antisymmetrize2` - ✅ DONE (IndexAntisymmetrization structure)
- [x] `is_symmetric`, `is_antisymmetric` - ✅ DONE (predicate operations)
- [x] `trace2`, `contract2` - ✅ DONE (EinsteinSummation structure)
- [x] `covariant_divergence` - ✅ DONE (ContractedBianchi structure)
- [x] `metric_inv` - ✅ DONE (MetricInverse structure)
- [x] `raise_vec`, `lower_vec` - ✅ DONE (IndexRaiseLower structure)
- [x] `epsilon4` - ✅ DONE (LeviCivita structure)
- [ ] `nabla_component` - PENDING (requires D(f,x) grammar integration)
- [ ] Partial derivatives - PENDING (uses D(f,x) / Dt(f,x) per grammar v0.96)

---

## Progress: Metric Inverse & Index Operations ✅ DONE

Added metric inverse and index raising/lowering axioms:

### MetricInverse(dim)
- `metric_inv : Tensor(0,2,dim,ℝ) → Tensor(2,0,dim,ℝ)`
- `metric_inverse_identity` : g^μρ g_ρν = δ^μ_ν
- `metric_inv_symmetric` : g^μν = g^νμ
- `metric_inv_involutive` : (g^{-1})^{-1} = g

### IndexRaiseLower(dim)
- `raise_vec`, `lower_vec` operations
- `raise_lower_identity` : lower(raise(T)) = T
- `lower_raise_identity` : raise(lower(T)) = T

### MetricTrace(dim)
- `metric_trace_is_dim` : g^μν g_μν = dim

### LeviCivita(dim)
- `epsilon4 : Nat → Nat → Nat → Nat → ℝ`
- `epsilon_antisym_01`, `epsilon_antisym_12`, `epsilon_antisym_23`
- `epsilon_repeated_zero` : ε(μ,μ,ν,ρ) = 0

**Total tensor tests: 55 passed, 5 ignored**

---

## Remaining Gaps (Future Work)

| Component | Status | Blocker |
|-----------|--------|---------|
| Christoffel axioms | ❌ | Parser limitation (see below) |
| Riemann axioms | ❌ | Parser limitation (see below) |
| Ricci/Einstein axioms | ❌ | Parser limitation (see below) |

### Parser Support Confirmed ✅ (Grammar v0.98)

**Update (2026-01-09):** The Kleis parser **already supports** parametric types in axiom quantifiers! This was discovered and documented as grammar v0.98.

```kleis
// ✅ NOW WORKS (always did, just undocumented)
axiom ricci_symmetric : ∀ Ric : Tensor(0, 2, dim, ℝ) .
    component(Ric, μ, ν) = component(Ric, ν, μ)

// ✅ Also works
axiom matrix_commute : ∀ A : Matrix(m, n, ℝ) . ∀ B : Matrix(m, n, ℝ) .
    plus(A, B) = plus(B, A)
```

**Tests added to verify:** `test_parse_quantifier_parametric_type_no_paren`, `test_parse_quantifier_parametric_type_with_paren`, `test_parse_quantifier_matrix_type`

The parser's `parse_type_annotation_for_quantifier` function handles nested parentheses correctly.

### What Already Works

- `D(f,x)` and `Dt(f,x)` - Full calculus support in `stdlib/calculus.kleis`
- Matrix axioms (using uninterpreted functions)
- Tensor component axioms (symmetry, Bianchi, etc.)
- Einstein field equations (component form)

---

## Progress: Einstein Equation Axioms ✅ DONE

Added dimension-agnostic axiomatization of Einstein's field equations:

### EinsteinSummation(dim)
Abstractly models index contraction without explicit sums (Z3 doesn't handle higher-order functions):
- `trace2 : Tensor(1,1,dim,ℝ) → ℝ` - trace operation
- `contract2 : Tensor(0,2,dim,ℝ) → Tensor(2,0,dim,ℝ) → ℝ` - metric contraction
- Axioms: `trace_additive`, `delta_self_contract`, `trace_zero`

### RicciTensorDefinition(dim)
- `ricci_symmetric` - R_μν = R_νμ
- `ricci_from_flat` - flat Riemann → zero Ricci

### RicciScalarDefinition(dim)
- `ricci_scalar_flat` - flat space has R = 0

### EinsteinTensorDefinition(dim)
- `einstein_symmetric` - G_μν is symmetric
- `einstein_flat` - flat space has G_μν = 0

### ContractedBianchi(dim)
The key identity that guarantees energy-momentum conservation:
- `covariant_divergence : Tensor(0,2,dim,ℝ) → Nat → ℝ`
- `einstein_divergence_free` - ∇^μ G_μν = 0

### EinsteinFieldEquationsAxiom(dim)
The field equations themselves:
- `field_equation_components` - G_μν + Λg_μν = κT_μν
- `vacuum_field_equation` - T_μν = 0 → G_μν = -Λg_μν

### EnergyMomentumConservation(dim)
- `stress_energy_conserved` - ∇^μ T_μν = 0

**Total new tests:** 7 tests for Einstein equation structures

---

## Physics Domains: Current vs. Future

### Currently Implemented ✅

| Domain | stdlib File | Tests |
|--------|-------------|-------|
| General Relativity | `tensors.kleis` | 60+ |
| Electromagnetism | `maxwell.kleis` | 10 |
| Fluid Dynamics | `fluid_dynamics.kleis` | 20 |
| Cosmology | `cosmology.kleis` | 12 |
| Solid Mechanics | `solid_mechanics.kleis` | 11 |

**Total: 100+ physics verification tests**

### Concrete Task: Expand Summations Before Z3 ✅ DONE

**Problem:** Z3 is first-order and can't handle `sum_over(λ ρ . ...)` directly. But Kleis CAN handle lambdas.

**Solution:** Pre-expand summations in the Z3 backend when bounds are concrete.

**Status:** ✅ Implemented in `feature/z3-tensor-contraction` branch

**Implementation:** `src/solvers/z3/backend.rs` - `try_expand_sum_over()` function

```rust
fn try_expand_sum_over(
    &mut self,
    lambda_arg: &Expression,
    start_arg: &Expression,
    end_arg: &Expression,
    vars: &HashMap<String, Dynamic>,
) -> Result<Option<Dynamic>, String>
```

**What This Enables:**

```kleis
// sum_over(λ ρ . g(μ,ρ) * g_inv(ρ,ν), 0, 4) 
// → g(μ,0)*g_inv(0,ν) + g(μ,1)*g_inv(1,ν) + g(μ,2)*g_inv(2,ν) + g(μ,3)*g_inv(3,ν)
```

**Features:**
- Handles concrete integer bounds only (falls back to uninterpreted for symbolic)
- Limits to 64 terms to prevent explosion
- Proper variable substitution respecting shadowing (lambdas, let, quantifiers)

**Tests Added:**
- `test_sum_over_expansion_simple`: λ i . i from 0 to 4 = 6 ✅
- `test_sum_over_expansion_with_multiplication`: λ i . 2*i ✅
- `test_sum_over_tensor_contraction`: g(μ,ρ) * g_inv(ρ,ν) pattern ✅
- `test_sum_over_empty_range`: empty range = 0 ✅
- `test_einstein_field_equations_chain`: Full verification chain ✅
  - Ricci tensor: R_μν = Σ_ρ R(ρ, μ, ρ, ν)
  - Ricci scalar: R = Σ_μ Σ_ν g^μν R_μν
  - Einstein tensor: G_μν = R_μν - ½ R g_μν
  - Field equations: G_μν + Λ g_μν = κ T_μν
  - Vacuum solution: G_μν = -Λ g_μν

---

## Progress: Transcendental Derivative Axioms ✅ DONE

Added `TranscendentalDerivatives(F)` structure to `stdlib/calculus.kleis`:

| Category | Axioms |
|----------|--------|
| Trigonometric | D_sin, D_cos, D_tan |
| Inverse Trig | D_arcsin, D_arccos, D_arctan |
| Exponential | D_exp, D_ln, D_log |
| Power | D_power_general (f^g) |
| Square Root | D_sqrt |
| Hyperbolic | D_sinh, D_cosh, D_tanh |

**These enable:**
- Schwarzschild metric (1/r terms, sqrt)
- Conformal factors (exp, ln)
- Spherical coordinates (sin, cos)
- FLRW cosmology (scale factor a(t))

---

## Progress: Manual Documentation ✅ DONE

Clarified the difference between computational and axiomatic differentiation:

| Function | Type | Where | What |
|----------|------|-------|------|
| `diff(expr, var)` | Computational | Evaluator | Pattern matches on AST, returns derivative |
| `D(f, x)` / `Dt(f, x)` | Axiomatic | Z3 | Declares properties for verification |

**Files Updated:**
- `docs/manual/src/chapters/13-applications.md` - Full comparison section
- `docs/manual/src/chapters/05-pattern-matching.md` - Cross-reference note

---

## Future Physics Domains 🎯

| Domain | Key Equations | Difficulty | Notes |
|--------|---------------|------------|-------|
| **Gas Dynamics** | Rankine-Hugoniot shocks, isentropic flow, Mach relations | Easy | Extends fluid_dynamics.kleis |
| **Kaluza-Klein** | 5D metric → gravity + EM unification | Medium | Unifies Maxwell + Einstein! |
| **Quantum Mechanics** | Schrödinger equation, commutators [x,p]=iℏ | Medium | Requires complex numbers |
| **Thermodynamics** | Maxwell relations, Gibbs-Duhem | Easy | Partial derivatives |
| **Heat Transfer** | Fourier's law, heat equation | Easy | Parabolic PDE |
| **Acoustics** | Wave equation, impedance matching | Easy | Hyperbolic PDE |
| **Optics** | Snell's law, Fresnel equations | Easy | EM at interfaces |
| **Elastodynamics** | Wave propagation in solids | Medium | Extends solid_mechanics.kleis |
| **Piezoelectricity** | Coupled electro-mechanical equations | Medium | Tensor coupling |
| **String Theory** | Polyakov action, Virasoro algebra | Hard | Conformal field theory |

### Kaluza-Klein Priority

Kaluza-Klein is especially interesting because it would **unify our existing Maxwell and Einstein implementations**:

```
5D metric g_AB (A,B = 0,1,2,3,5):
├── g_μν → 4D gravity (Einstein)
├── g_μ5 → Electromagnetism (Maxwell)  
└── g_55 → Scalar field (dilaton)
```

This demonstrates the power of the tensor verification framework!

---

## Progress: Cartan Geometry (Computational) ✅ PARTIAL

Implemented computational Cartan calculus for curvature tensor computation using tetrads and exterior algebra.

### Files Created

- `stdlib/symbolic_diff.kleis` - Computational symbolic differentiation (`diff` function)
- `stdlib/cartan_geometry.kleis` - Cartan geometry structures (axiomatic)
- `stdlib/cartan_compute.kleis` - Computational implementation
- `tests/symbolic_diff_test.rs` - 23 tests for `diff` and `simplify`
- `tests/cartan_compute_test.rs` - 19 passing tests, 3 ignored

### What Works ✅

| Feature | Status |
|---------|--------|
| `diff(expr, var)` - symbolic derivative | ✅ |
| `simplify(expr)` - algebraic simplification | ✅ |
| `d0(f)` - exterior derivative of 0-form | ✅ |
| `d1(ω)` - exterior derivative of 1-form | ✅ |
| `wedge(α, β)` - wedge product | ✅ |
| `minkowski_tetrad_forms` | ✅ |
| `schwarzschild_tetrad_forms(M)` | ✅ |
| `d_tetrad(e)` - derivative of tetrad | ✅ |
| `solve_levi_civita(e, de, η)` - connection 1-forms | ✅ |

### What Doesn't Work Yet ❌

| Feature | Problem |
|---------|---------|
| `compute_curvature(ω)` | Expression explosion - `R = dω + ω∧ω` creates huge ASTs |

### Root Cause: Expression Explosion

The curvature computation involves:
1. `d1(ω^a_b)` - differentiates each component of connection (16 derivatives × 4 coords = 64 terms)
2. `ω^a_c ∧ ω^c_b` - wedge products (16 × 16 = 256 terms per sum)
3. Sum over index c - another 4× factor

Total: thousands of nested `Add`, `Mul`, `Pow` nodes that the `simplify` function can't reduce fast enough.

### Required Optimizations 🔧

1. **Lazy Evaluation** - Don't expand until needed
2. **Better Simplification** - Pattern-based algebraic rules
3. **Sparse Representation** - Most tetrad/connection components are zero
4. **Memoization** - Cache computed derivatives

### Workaround (Current)

For now, curvature tests are `#[ignore]`. The connection solver works correctly for both Minkowski (all zeros) and Schwarzschild (non-trivial).

## HOF Derivatives: Proper Mathematical Formulation

**Branch:** `feature/hof-derivatives`

**Insight:** The derivative is a **higher-order function**:

```
D : (ℝ → ℝ) → (ℝ → ℝ)
```

It takes a function and returns a function. The `Expr` AST in `symbolic_diff.kleis` was an unnecessary detour — the derivative should operate directly on Kleis lambdas.

### Current Status

Created `stdlib/calculus_hof.kleis` with the proper HOF formulation, but hit parser/evaluator limitations:

1. **Parser doesn't support double application** `f(x)(y)`
   - `D(lambda x. x*x)(3)` fails with "Expected identifier"
   - Need to recognize `(expr)(args)` as function application

2. **Evaluator HOF return not callable**
   - `let f' = D(f) in f'(x)` fails
   - `D(f)` returns something, but it's not a callable lambda

### Workaround

Using `eval_at(D(f), x)` as a placeholder for `D(f)(x)` in axioms.

### Required Fixes

1. **Parser:** Add rule for `(expr)(args)` application syntax
2. **Evaluator:** Ensure HOF-returned lambdas are callable
3. **Then:** Remove `eval_at` workaround, use natural `D(f)(x)` syntax

### Why This Matters

With working HOF:
- No need for `Expr` AST for symbolic differentiation
- Derivatives work on native Kleis lambdas
- Axioms become cleaner: `D(sin) = cos` instead of pattern matching
- Cartan geometry can use `D` directly on coordinate functions


### Update: No Parser Fix Needed!

`D(f, x)` and `D(f)(x)` are **isomorphic** via currying:

```
(A → B → C) ≅ (A × B → C)
```

So `D : (F → F) → F → F` used as `D(f, x)` works with current syntax.

The `eval_at` workaround was unnecessary. Removed it.

Current `calculus_hof.kleis`:
```kleis
structure Derivative(F : Field) {
    operation D : (F → F) → F → F
    
    axiom D_additive: ∀(f : (F → F))(g : (F → F))(x : F). 
        D(plus_fn(f, g), x) = D(f, x) + D(g, x)
    // ...
}
```

Clean, no workarounds, no parser changes needed.

