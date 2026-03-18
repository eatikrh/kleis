/-
  AtikConjecture.VarianceReduction
  ---------------------------------
  The variance reduction: if Σ σ_k = N/2 and Σ σ_k² = N/4,
  then σ_k = 1/2 for all k.

  This is the algebraic core of the spectral identity argument.
-/

import Mathlib.Algebra.BigOperators.Fin
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.BigOperators
import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum

open Finset BigOperators

/-- If Σ (xₖ - μ)² = 0, then xₖ = μ for all k. -/
theorem all_eq_of_sum_sq_deviation_zero {n : ℕ} (x : Fin n → ℝ) (μ : ℝ)
    (h : ∑ i : Fin n, (x i - μ) ^ 2 = 0) :
    ∀ (i : Fin n), x i = μ := by
  intro i
  have h_nonneg : ∀ (j : Fin n), 0 ≤ (x j - μ) ^ 2 := fun j => sq_nonneg _
  have h_le : (x i - μ) ^ 2 ≤ ∑ j : Fin n, (x j - μ) ^ 2 :=
    Finset.single_le_sum (fun j _ => h_nonneg j) (Finset.mem_univ i)
  have h_le_zero : (x i - μ) ^ 2 ≤ 0 := by linarith
  have h_eq : (x i - μ) ^ 2 = 0 := le_antisymm h_le_zero (h_nonneg i)
  have h_diff : x i - μ = 0 := by
    rwa [sq_eq_zero_iff] at h_eq
  linarith

/-- **RH from moments.**
    If Σ σ_k = N/2 and Σ σ_k² = N/4, then σ_k = 1/2 for all k. -/
theorem rh_from_moments {n : ℕ} (σ : Fin n → ℝ)
    (h_sum : ∑ i : Fin n, σ i = ↑n / 2)
    (h_sum_sq : ∑ i : Fin n, σ i ^ 2 = ↑n / 4) :
    ∀ (i : Fin n), σ i = 1 / 2 := by
  apply all_eq_of_sum_sq_deviation_zero σ (1 / 2)
  -- Expand (σ i - 1/2)² = σ i² + (1/4 - σ i)
  have expand : ∀ (i : Fin n), (σ i - 1 / 2) ^ 2 = σ i ^ 2 + ((1 : ℝ) / 4 - σ i) := by
    intro i; ring
  simp_rw [expand]
  rw [Finset.sum_add_distrib]
  -- Split (1/4 - σ i) = 1/4 + (-σ i)
  have split : ∀ (i : Fin n), (1 : ℝ) / 4 - σ i = (1 : ℝ) / 4 + (-σ i) := by
    intro i; ring
  simp_rw [split]
  rw [Finset.sum_add_distrib, Finset.sum_neg_distrib]
  simp only [Finset.sum_const, Finset.card_fin, nsmul_eq_mul]
  linarith

/-- **Spectral identity reduction.**
    Structure theorem (Re = 1/2) + trace formula matching → RH. -/
theorem spectral_identity_implies_rh
    {n : ℕ} (σ : Fin n → ℝ) (re_eig : Fin n → ℝ)
    (h_structure : ∀ (i : Fin n), re_eig i = 1 / 2)
    (h_trace_1 : ∑ i : Fin n, σ i = ∑ i : Fin n, re_eig i)
    (h_trace_2 : ∑ i : Fin n, σ i ^ 2 = ∑ i : Fin n, re_eig i ^ 2) :
    ∀ (i : Fin n), σ i = 1 / 2 := by
  have h_re_sum : ∑ i : Fin n, re_eig i = ↑n / 2 := by
    have : ∑ i : Fin n, re_eig i = ∑ _i : Fin n, (1 : ℝ) / 2 :=
      Finset.sum_congr rfl (fun i _ => h_structure i)
    rw [this, Finset.sum_const, Finset.card_fin, nsmul_eq_mul]; ring
  have h_re_sum_sq : ∑ i : Fin n, re_eig i ^ 2 = ↑n / 4 := by
    have : ∑ i : Fin n, re_eig i ^ 2 = ∑ _i : Fin n, ((1 : ℝ) / 2) ^ 2 :=
      Finset.sum_congr rfl (fun i _ => by rw [h_structure])
    rw [this, Finset.sum_const, Finset.card_fin, nsmul_eq_mul]; norm_num; ring
  exact rh_from_moments σ (by linarith) (by linarith)
