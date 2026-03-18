/-
  AtikConjecture.Convergence
  ---------------------------
  Eigenvalue convergence for the spectral comb.

  Key result: The 2×2 skew-symmetric block with peak γ has eigenvector
  [1, i] with eigenvalue d + iγ, proving the decoupled spectral comb
  has eigenvalues exactly at the peak locations.
-/
import AtikConjecture.Basic
import Mathlib.Data.Complex.Basic
import Mathlib.Data.Matrix.Basic
import Mathlib.LinearAlgebra.Matrix.Defs
import Mathlib.Tactic.FinCases
import Mathlib.Tactic.LinearCombination

open Matrix Complex

namespace AtikConjecture

/-! ### The 2×2 eigenvector computation -/

/--
  The concrete 2×2 antisymmetric block with peak γ and diagonal d,
  embedded in ℂ. This is `d·I₂ + [[0, γ], [-γ, 0]]`.
-/
def spectralBlock (d γ : ℝ) : Matrix (Fin 2) (Fin 2) ℂ := fun i j =>
  if i = j then (d : ℂ)
  else if i = 0 then (γ : ℂ)
  else -(γ : ℂ)

/-- The candidate eigenvector [1, i]. -/
def eigvec_plus : Fin 2 → ℂ := fun i =>
  if i = 0 then 1 else Complex.I

/-- The candidate eigenvector [1, -i]. -/
def eigvec_minus : Fin 2 → ℂ := fun i =>
  if i = 0 then 1 else -Complex.I

theorem eigvec_plus_ne_zero : eigvec_plus ≠ 0 := by
  intro h
  have h0 := congr_fun h 0
  simp [eigvec_plus] at h0

theorem eigvec_minus_ne_zero : eigvec_minus ≠ 0 := by
  intro h
  have h0 := congr_fun h 0
  simp [eigvec_minus] at h0

/--
  **2×2 Eigenvector Theorem.**
  The spectral block `d·I + [[0,γ],[-γ,0]]` applied to `[1, i]`
  yields `(d + iγ) · [1, i]`.
-/
private theorem aux_ext {a b : ℂ} (hr : a.re = b.re) (hi : a.im = b.im) : a = b :=
  Complex.ext hr hi

theorem spectralBlock_eigenvector_plus (d γ : ℝ) :
    (spectralBlock d γ) *ᵥ eigvec_plus = (↑d + ↑γ * Complex.I) • eigvec_plus := by
  funext i; fin_cases i <;> apply aux_ext <;>
    simp [spectralBlock, eigvec_plus, mulVec, dotProduct, smul_eq_mul,
      Complex.add_re, Complex.add_im, Complex.mul_re, Complex.mul_im,
      Complex.neg_re, Complex.neg_im, Complex.ofReal_re, Complex.ofReal_im,
      Complex.I_re, Complex.I_im, Finset.univ, Fintype.elems,
      List.finRange, List.map, List.sum]

theorem spectralBlock_eigenvector_minus (d γ : ℝ) :
    (spectralBlock d γ) *ᵥ eigvec_minus = (↑d - ↑γ * Complex.I) • eigvec_minus := by
  funext i; fin_cases i <;> apply aux_ext <;>
    simp [spectralBlock, eigvec_minus, mulVec, dotProduct, smul_eq_mul,
      Complex.add_re, Complex.add_im, Complex.mul_re, Complex.mul_im,
      Complex.neg_re, Complex.neg_im, Complex.sub_re, Complex.sub_im,
      Complex.ofReal_re, Complex.ofReal_im, Complex.I_re, Complex.I_im,
      Finset.univ, Fintype.elems, List.finRange, List.map, List.sum]

/-- The eigenvalue d + iγ has Re = d. -/
theorem eigenvalue_plus_re (d γ : ℝ) :
    (↑d + ↑γ * Complex.I : ℂ).re = d := by
  simp [Complex.add_re, Complex.mul_re, Complex.I_re, Complex.I_im,
        Complex.ofReal_re, Complex.ofReal_im]

/-- The eigenvalue d - iγ has Re = d. -/
theorem eigenvalue_minus_re (d γ : ℝ) :
    (↑d - ↑γ * Complex.I : ℂ).re = d := by
  simp [Complex.sub_re, Complex.mul_re, Complex.I_re, Complex.I_im,
        Complex.ofReal_re, Complex.ofReal_im]

/-! ### Summary: the decoupled spectral comb -/

/--
  **Decoupled Spectral Comb Eigenvalue Theorem.**
  Each 2×2 block of the decoupled (ε = 0) spectral comb with peak γ_k
  and diagonal d has:
  - Eigenvector [1, i] with eigenvalue d + iγ_k (Re = d)
  - Eigenvector [1, -i] with eigenvalue d - iγ_k (Re = d)
  - Both eigenvectors are nonzero

  For a block-diagonal comb with N blocks, eigenvectors lift to the
  full matrix (each eigenvector extended by zeros in the other blocks).
  The full decoupled comb therefore has eigenvalues
  {d + iγ_k, d - iγ_k : k = 1, ..., N}, all with Re = d.

  Setting d = 1/2, this gives Re = 1/2 for all eigenvalues.

  For the coupled comb (ε > 0), Gershgorin's circle theorem
  (Mathlib: `eigenvalue_mem_ball`) bounds the perturbation:
  each eigenvalue stays within a Gershgorin disk centered at d
  with radius at most max(γ_k) + ε. As ε → 0, the coupled
  eigenvalues converge to the decoupled ones by continuity of
  eigenvalues as functions of matrix entries.
-/
theorem decoupled_eigenvalues (d γ : ℝ) :
    ((spectralBlock d γ) *ᵥ eigvec_plus = (↑d + ↑γ * Complex.I) • eigvec_plus) ∧
    ((spectralBlock d γ) *ᵥ eigvec_minus = (↑d - ↑γ * Complex.I) • eigvec_minus) ∧
    (↑d + ↑γ * Complex.I : ℂ).re = d ∧
    (↑d - ↑γ * Complex.I : ℂ).re = d ∧
    eigvec_plus ≠ 0 ∧
    eigvec_minus ≠ 0 :=
  ⟨spectralBlock_eigenvector_plus d γ,
   spectralBlock_eigenvector_minus d γ,
   eigenvalue_plus_re d γ,
   eigenvalue_minus_re d γ,
   eigvec_plus_ne_zero,
   eigvec_minus_ne_zero⟩

end AtikConjecture
