/-
  AtikConjecture.Basic
  --------------------
  Definitions and basic properties of skew-symmetric (antisymmetric) matrices.

  A real matrix A is skew-symmetric iff Aᵀ = -A, equivalently ∀ i j, A j i = -A i j.
-/
import Mathlib.LinearAlgebra.Matrix.Defs
import Mathlib.Data.Matrix.Basic
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

open Matrix

namespace AtikConjecture

variable {n : Type*} {α : Type*}

/-- A matrix is skew-symmetric if its transpose equals its negation. -/
def IsSkewSymmetric [Neg α] (A : Matrix n n α) : Prop :=
  Aᵀ = -A

section BasicProperties

variable [AddGroup α]

theorem IsSkewSymmetric.transpose_eq {A : Matrix n n α} (h : IsSkewSymmetric A) :
    Aᵀ = -A :=
  h

theorem IsSkewSymmetric.entry {A : Matrix n n α} (h : IsSkewSymmetric A) (i j : n) :
    A j i = -(A i j) := by
  have := congr_fun (congr_fun h i) j
  simp [transpose_apply] at this
  exact this

theorem isSkewSymmetric_iff_entry (A : Matrix n n α) :
    IsSkewSymmetric A ↔ ∀ i j, A j i = -(A i j) := by
  constructor
  · exact fun h i j => h.entry i j
  · intro h
    ext i j
    simp [transpose_apply, h i j]

theorem IsSkewSymmetric.neg {A : Matrix n n α} (h : IsSkewSymmetric A) :
    IsSkewSymmetric (-A) := by
  rw [IsSkewSymmetric, transpose_neg, h.transpose_eq, neg_neg]

end BasicProperties

section AddCommGroup

variable [AddCommGroup α]

theorem IsSkewSymmetric.add {A B : Matrix n n α}
    (hA : IsSkewSymmetric A) (hB : IsSkewSymmetric B) :
    IsSkewSymmetric (A + B) := by
  rw [IsSkewSymmetric, transpose_add, hA.transpose_eq, hB.transpose_eq, neg_add]

theorem IsSkewSymmetric.zero : IsSkewSymmetric (0 : Matrix n n α) := by
  simp [IsSkewSymmetric, transpose_zero]

end AddCommGroup

section DiagZero

variable [NonAssocRing α] [NoZeroDivisors α] [CharZero α]

open CharZero in
theorem IsSkewSymmetric.diag_eq_zero
    {A : Matrix n n α} (h : IsSkewSymmetric A) (i : n) :
    A i i = 0 := by
  have := h.entry i i
  exact eq_neg_self_iff.mp this

end DiagZero

section Scalar

variable [Ring α]

theorem IsSkewSymmetric.smul_left {A : Matrix n n α} (h : IsSkewSymmetric A) (c : α) :
    IsSkewSymmetric (c • A) := by
  rw [IsSkewSymmetric, transpose_smul, h.transpose_eq, smul_neg]

end Scalar

end AtikConjecture
