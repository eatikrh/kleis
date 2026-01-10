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

<<<<<<< HEAD
## Editor Tensor Format: Update to New Convention
=======
## Editor Tensor Format: Update to New Convention ✅ DONE
>>>>>>> ddfd404 (feat: migrate tensor templates to new format with indexStructure)

**Status:** Completed in `feature/migrate-tensor-format` branch.

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

<<<<<<< HEAD
**Files to update:**
- `static/index.html` - Update `subsup` template to use new tensor format
- `src/solvers/z3/backend.rs` - Add handling for `kind: 'tensor'` operations (treat as formatting-only, return base type)
=======
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
>>>>>>> ddfd404 (feat: migrate tensor templates to new format with indexStructure)
