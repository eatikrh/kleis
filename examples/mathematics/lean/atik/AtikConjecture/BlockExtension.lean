/-
  AtikConjecture.BlockExtension
  ------------------------------
  Lemma 1 (Block Extension): Extending a skew-symmetric matrix by a
  skew-symmetric block with skew-symmetric coupling produces a
  skew-symmetric matrix.

  This is the formal gap identified in the k-induction proof of the
  Atik Conjecture. The spectral comb at size 2N is extended to
  2(N+1) by appending a 2×2 antisymmetric block. We prove that this
  block construction preserves skew-symmetry for arbitrary index types.
-/
import AtikConjecture.Basic
import Mathlib.Data.Matrix.Block

open Matrix

namespace AtikConjecture

variable {n m : Type*} {α : Type*} [AddGroup α]

/--
  The off-diagonal coupling condition `Cᵀ = -B` is equivalent to `Bᵀ = -C`.
-/
theorem coupling_skew_iff {B : Matrix n m α} {C : Matrix m n α} :
    Cᵀ = -B ↔ Bᵀ = -C := by
  constructor
  · intro h
    have h1 : C = -(Bᵀ) := by
      have := congrArg Matrix.transpose h
      rwa [transpose_transpose, transpose_neg] at this
    rw [h1, neg_neg]
  · intro h
    have h1 : B = -(Cᵀ) := by
      have := congrArg Matrix.transpose h
      rwa [transpose_transpose, transpose_neg] at this
    rw [h1, neg_neg]

/--
  A block matrix `fromBlocks A B C D` is skew-symmetric if:
  - `A` is skew-symmetric (top-left block)
  - `D` is skew-symmetric (bottom-right block)
  - `Cᵀ = -B` (the off-diagonal blocks are skew-paired)
-/
theorem fromBlocks_isSkewSymmetric
    {A : Matrix n n α} {B : Matrix n m α} {C : Matrix m n α} {D : Matrix m m α}
    (hA : IsSkewSymmetric A) (hD : IsSkewSymmetric D) (hCB : Cᵀ = -B) :
    IsSkewSymmetric (fromBlocks A B C D) := by
  unfold IsSkewSymmetric
  rw [fromBlocks_transpose, fromBlocks_neg]
  have hBC : Bᵀ = -C := coupling_skew_iff.mp hCB
  rw [hA.transpose_eq, hCB, hBC, hD.transpose_eq]

/--
  Specialized version for the spectral comb inductive step:
  extending a skew-symmetric matrix `A` (the existing 2N×2N comb)
  by a skew-symmetric matrix `D` (the new 2×2 block) with coupling
  specified by a single matrix `B : Matrix n m α` with the lower-left
  block being `-Bᵀ`.
-/
theorem spectralComb_extension_isSkewSymmetric
    {A : Matrix n n α} {B : Matrix n m α} {D : Matrix m m α}
    (hA : IsSkewSymmetric A) (hD : IsSkewSymmetric D) :
    IsSkewSymmetric (fromBlocks A B (-Bᵀ) D) := by
  apply fromBlocks_isSkewSymmetric hA hD
  rw [transpose_neg, transpose_transpose]

end AtikConjecture
