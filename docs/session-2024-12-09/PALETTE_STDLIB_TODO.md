# Palette Operations - Stdlib Coverage Analysis

**Date:** December 9, 2024  
**Goal:** Ensure all palette template operations can be type-checked

---

## Operations in Palette (from index.html)

### ✅ Currently Supported (in stdlib)

**Basic Arithmetic:**
- `plus`, `minus`, `scalar_multiply`, `scalar_divide` ✅
- `equals` ✅
- `abs`, `floor`, `ceiling` ✅

**Matrices:**
- `Matrix`, `PMatrix`, `VMatrix`, `BMatrix` ✅ (now data constructors!)
- `transpose` ✅ (in matrices.kleis)
- `add`, `multiply` ✅ (in matrices.kleis)

**Calculus:**
- `int_bounds` ✅ (integral with bounds)
- `sum_bounds`, `prod_bounds` ✅ (summation, product)
- `d_part`, `d_dt` ✅ (partial derivative, time derivative)
- `grad` ✅ (gradient)

**Functions:**
- `sin`, `cos`, `tan` ✅ (in minimal_prelude or prelude)
- `ln`, `log`, `exp` ✅
- `sqrt` ✅

### ⚠️ Needs Adding to Stdlib

**Quantum Operations:** (Priority: Medium)
- `ket` - Dirac ket notation |ψ⟩
- `bra` - Dirac bra notation ⟨ψ|
- `inner` - Inner product ⟨ψ|φ⟩
- `outer` - Outer product |ψ⟩⟨φ|
- `commutator` - Commutator [A, B]
- `expectation` - Expectation value ⟨Ô⟩

**Tensor Operations:** (Priority: Medium)
- `index_mixed` - Mixed tensor indices T^μ_ν
- `christoffel` - Christoffel symbols Γ^μ_νρ
- `riemann` - Riemann tensor R^μ_νρσ

**Integral Transforms:** (Priority: Low - Advanced)
- `fourier_transform` - Fourier transform ℱ[f](ω)
- `inverse_fourier` - Inverse Fourier transform ℱ⁻¹[F](t)
- `laplace_transform` - Laplace transform ℒ[f](s)
- `inverse_laplace` - Inverse Laplace ℒ⁻¹[F](t)
- `convolution` - Convolution (f ∗ g)(t)
- `kernel_integral` - Kernel integral ∫K(x,y)f(y)dy
- `greens_function` - Green's function G(x,x')

**POT Operations:** (Priority: Low - Research)
- `projection` - Π[f](x)
- `modal_integral` - Modal integral ∫f dμ(M)
- `projection_kernel` - K(x, m)
- `causal_bound` - c(x)
- `projection_residue` - Residue[Π, S]
- `modal_space` - ℳ_n
- `spacetime` - ℝ⁴
- `hont` - ℋ_n

**Trigonometric Inverses:** (Priority: High)
- `arcsin`, `arccos`, `arctan` (might be there, needs verification)

**Other:**
- `nth_root` - n-th root ⁿ√x
- `binomial` - Binomial coefficient (n choose k)
- `factorial` - n!
- `lim` - Limit operation
- `vector_bold` - Bold vector notation
- `vector_arrow` - Arrow vector notation
- `parens`, `brackets`, `braces`, `angle_brackets` - Grouping (rendering only)

---

## Recommended Action Plan

### Phase 1: Core Math (High Priority)

Create `stdlib/math_functions.kleis`:
```kleis
structure Transcendental(T) {
    operation sin : T → T
    operation cos : T → T
    operation tan : T → T
    operation arcsin : T → T
    operation arccos : T → T
    operation arctan : T → T
    operation ln : T → T
    operation log : T → T
    operation exp : T → T
}

implements Transcendental(ℝ) {
    operation sin = builtin_sin
    operation cos = builtin_cos
    // ... etc
}

structure Combinatorial {
    operation factorial : ℕ → ℕ
    operation binomial : ℕ × ℕ → ℕ
}

structure Roots(T) {
    operation sqrt : T → T
    operation nth_root : ℕ × T → T
}
```

### Phase 2: Quantum Mechanics (Medium Priority)

Create `stdlib/quantum.kleis`:
```kleis
structure HilbertSpace(T) {
    operation ket : T → Ket(T)
    operation bra : T → Bra(T)
    operation inner : Bra(T) × Ket(T) → ℂ
    operation outer : Ket(T) × Bra(T) → Operator(T)
}

structure Operator(T) {
    operation commutator : Operator(T) × Operator(T) → Operator(T)
    operation expectation : Operator(T) → ℝ
}
```

### Phase 3: Tensor Calculus (Medium Priority)

Create `stdlib/differential_geometry.kleis`:
```kleis
structure TensorField(rank: Nat, dim: Nat) {
    operation index_mixed : TensorField(rank, dim) → ...
}

structure Connection(dim: Nat) {
    operation christoffel : Connection(dim) → Tensor(dim, dim, dim)
}

structure Curvature(dim: Nat) {
    operation riemann : Curvature(dim) → Tensor(dim, dim, dim, dim)
}
```

### Phase 4: Transform Theory (Low Priority)

Create `stdlib/transforms.kleis` with Fourier, Laplace, convolution operations

### Phase 5: POT Operations (Research - Low Priority)

Create `stdlib/pot.kleis` for Projected Ontology Theory operations

---

## Implementation Strategy

1. **Audit existing stdlib** - Check what's already there
2. **Prioritize by usage** - Focus on most-used operations first
3. **Define structures** - Create structure definitions
4. **Add implementations** - For built-in types (ℝ, ℂ, etc.)
5. **Test inference** - Verify palette templates type-check
6. **Document** - Add examples and explanations

---

## Testing Approach

For each operation, test:
```rust
#[test]
fn test_operation_has_type() {
    let checker = TypeChecker::with_stdlib().unwrap();
    let types = checker.types_supporting("operation_name");
    assert!(!types.is_empty(), "operation_name should be defined");
}
```

Create a comprehensive test that goes through ALL palette templates and ensures they type-check.

---

## Benefits

Once complete:
- ✅ All palette templates type-checkable
- ✅ Better error messages for users
- ✅ Type-driven code completion possible
- ✅ Validates operation compatibility
- ✅ Self-documenting through types

---

## Current Status

**Covered:**
- Basic arithmetic ✅
- Matrices ✅
- Basic calculus ✅
- Basic functions ✅

**Missing:**
- Quantum operations (~6 operations)
- Tensor operations (~3 operations)
- Transform operations (~7 operations)
- POT operations (~8 operations)
- Some math functions (factorial, binomial, etc.)

**Estimated work:** 2-4 hours to add all missing operations

---

## Next Session Task

1. Audit stdlib files for existing operations
2. Create list of missing operations with signatures
3. Implement high-priority operations first
4. Test each operation with palette templates
5. Document in stdlib README

**File to create:** `stdlib/advanced.kleis` or separate domain files

---

## Notes

- Most operations are display/rendering only currently
- Adding them to type system enables:
  - Type checking
  - Better error messages
  - Operation compatibility validation
- Can start with structure definitions (abstract)
- Implementations can come later (concrete)

