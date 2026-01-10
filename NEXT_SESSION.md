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
