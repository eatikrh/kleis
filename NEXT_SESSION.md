# Next Session Tasks

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
