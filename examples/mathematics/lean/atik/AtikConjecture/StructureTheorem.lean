/-
  AtikConjecture.StructureTheorem
  --------------------------------
  Lemma 2 (Structure Theorem): If A is a real skew-symmetric matrix,
  then for M = d·I + A, every eigenvalue μ of M satisfies Re(μ) = d.
-/
import Mathlib.LinearAlgebra.Matrix.Defs
import Mathlib.LinearAlgebra.Matrix.ConjTranspose
import Mathlib.LinearAlgebra.Matrix.DotProduct
import Mathlib.Data.Matrix.Mul
import Mathlib.Data.Complex.Basic
import Mathlib.Analysis.Complex.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.LinearCombination

open Matrix Complex
open scoped ComplexOrder

namespace AtikConjecture

variable {n : Type*} [Fintype n] [DecidableEq n]

omit [DecidableEq n] in
private lemma star_dotProduct_self_ne_zero {v : n → ℂ} (hv : v ≠ 0) :
    star v ⬝ᵥ v ≠ 0 := by
  rwa [Ne, dotProduct_star_self_eq_zero]

/-! ### Skew-Hermitian implies purely imaginary eigenvalues -/

omit [DecidableEq n] in
theorem skewHermitian_eigenvalue_conj_neg
    {B : Matrix n n ℂ} (hB : Bᴴ = -B)
    {μ : ℂ} {v : n → ℂ} (hv : v ≠ 0) (heig : B *ᵥ v = μ • v) :
    starRingEnd ℂ μ = -μ := by
  have hinner_pos := star_dotProduct_self_ne_zero hv
  have eq1 : star v ⬝ᵥ (B *ᵥ v) = μ * (star v ⬝ᵥ v) := by
    rw [heig, dotProduct_smul, smul_eq_mul]
  have eq3 : star (B *ᵥ v) = -(star v ᵥ* B) := by
    rw [star_mulVec, hB, vecMul_neg]
  have eq4 : star (B *ᵥ v) ⬝ᵥ v = -(star v ⬝ᵥ (B *ᵥ v)) := by
    rw [eq3, neg_dotProduct, dotProduct_mulVec]
  have eq2 : star (B *ᵥ v) ⬝ᵥ v = starRingEnd ℂ μ * (star v ⬝ᵥ v) := by
    rw [heig, star_smul, smul_dotProduct, smul_eq_mul, starRingEnd_apply]
  have h1 : starRingEnd ℂ μ * (star v ⬝ᵥ v) = -(μ * (star v ⬝ᵥ v)) := by
    rw [← eq2, eq4, eq1]
  have key : (starRingEnd ℂ μ + μ) * (star v ⬝ᵥ v) = 0 := by
    linear_combination h1
  rcases mul_eq_zero.mp key with h | h
  · linear_combination h
  · exact absurd h hinner_pos

omit [DecidableEq n] in
theorem skewHermitian_eigenvalue_re_zero
    {B : Matrix n n ℂ} (hB : Bᴴ = -B)
    {μ : ℂ} {v : n → ℂ} (hv : v ≠ 0) (heig : B *ᵥ v = μ • v) :
    μ.re = 0 := by
  have hconj := skewHermitian_eigenvalue_conj_neg hB hv heig
  have h1 : (starRingEnd ℂ μ).re = μ.re := conj_re μ
  have h2 : (-μ).re = -μ.re := neg_re μ
  have h3 := congrArg Complex.re hconj
  linarith

/-! ### Real skew-symmetric → skew-Hermitian -/

omit [Fintype n] [DecidableEq n] in
theorem real_skewSymmetric_is_skewHermitian
    {A : Matrix n n ℝ} (hA : Aᵀ = -A) :
    (A.map (Complex.ofReal))ᴴ = -(A.map (Complex.ofReal)) := by
  ext i j
  simp only [conjTranspose_apply, map_apply, star_def, conj_ofReal, neg_apply]
  have h := congr_fun (congr_fun hA j) i
  simp only [transpose_apply] at h
  rw [h]
  simp [neg_apply]

/-! ### The Structure Theorem -/

theorem structure_theorem_eigenvector
    {A : Matrix n n ℝ} (hA : Aᵀ = -A) (d : ℝ)
    {μ : ℂ} {v : n → ℂ} (hv : v ≠ 0)
    (heig : ((d : ℂ) • (1 : Matrix n n ℂ) + A.map Complex.ofReal) *ᵥ v = μ • v) :
    μ.re = d := by
  let B := A.map Complex.ofReal
  have hBskew : Bᴴ = -B := real_skewSymmetric_is_skewHermitian hA
  set ν := μ - (d : ℂ) with hν_def
  have hν_eig : B *ᵥ v = ν • v := by
    have h1 : ((d : ℂ) • (1 : Matrix n n ℂ)) *ᵥ v = (d : ℂ) • v := by
      rw [smul_mulVec, one_mulVec]
    rw [add_mulVec, h1] at heig
    have h2 : B *ᵥ v = μ • v - (d : ℂ) • v := by
      linear_combination heig
    rw [h2, hν_def, sub_smul]
  have hre_zero := skewHermitian_eigenvalue_re_zero hBskew hv hν_eig
  simp only [hν_def, sub_re, ofReal_re] at hre_zero
  linarith

end AtikConjecture
