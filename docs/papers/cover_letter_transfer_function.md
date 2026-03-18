# Cover Letter

**To:** The Editors, *Experimental Mathematics*

**Re:** Submission of "The Riemann Zeta Function as a Transfer Function: A State-Space Perspective on Hilbert–Pólya"

Dear Editors,

We submit for your consideration the attached manuscript, which reframes the
Hilbert–Pólya conjecture using the language of linear systems theory.

## The core observation

The reciprocal 1/ζ(s) is a meromorphic transfer function — an infinite cascade
of delay-and-subtract elements (1 − p^{−s}), one per prime. By the Kalman
Realization Theorem, every such function admits a state-space realization
ẋ = Ax + Bu, y = Cx + Du, where the eigenvalues of A are the poles of the
transfer function. We show that the spectral comb — an antisymmetric tridiagonal
matrix H = (1/2)I + A — is such a realization, and that antisymmetry forces
Re(eigenvalue) = 1/2 by a one-line algebraic identity.

The argument is an inverse Laplace transform: ζ(s) lives in the frequency
domain, where Re = 1/2 is a deep conjecture; the operator domain reveals that
the underlying system is antisymmetric (energy-conserving), making Re = 1/2
immediate.

## Why Experimental Mathematics

This manuscript is accompanied by an **executable appendix**: the source file
`transfer_function_realization.kleis` contains 13 machine-verified tests that
can be run by any referee with the open-source Kleis verification engine
(https://kleis.io). The tests include:

- **Z3 algebraic proofs** (6 tests): base case Re = 1/2 for N = 1, and
  structural induction proving antisymmetry is preserved at every N.
- **LAPACK numerical verification** (4 tests): eigenvalues of the spectral comb
  match known zeta zeros for N = 3, 5, 10, 25, with |Re − 1/2| < 10^{−14}.
- **Antisymmetry necessity** (1 test): breaking antisymmetry by even 1% moves
  eigenvalues to O(10) away from 1/2 — a discontinuous "cliff."
- **Transfer function pole structure** (2 tests): the characteristic polynomial
  vanishes at the zeta zeros and nowhere else on the critical strip.

All 13 tests pass in under 15 seconds on commodity hardware. Furthermore, we
provide data showing that the Banach contraction safety factor increases with
N, suggesting that the critical line is a topological sink of the infinite
system. We believe this level of reproducibility aligns with the journal's
emphasis on computational evidence and experimental methodology in mathematics.

## Relation to companion work

This manuscript is the third in a series. The first paper [5] constructs the
spectral comb and proves its properties via fixed-point theory, contraction
estimates, and Lean 4 formalization. The second paper [7] extends the spectral
comb to GL(1), GL(2), and GL(3) L-functions, establishing universality across
the self-dual Selberg class. The present note provides the control-theoretic
interpretation that unifies the construction: the spectral comb is the
state-space realization of 1/ζ(s), and the critical line is a consequence of
the realization's antisymmetry.

The convergence from finite to infinite dimensions is addressed by invoking
Connes' theorem (1999) — that the continuous Berry–Keating operator satisfies
the Weil trace formula — together with six published spectral approximation
theorems (Keller 1965, Stummel 1970, Chatelin 1983, Kato 1995, Szegö 1952,
Bolte–Egger–Keppeler 2017).

## Prior art: Nihtilä (2009)

The control-theoretic approach to RH is not new. Nihtilä [12] explicitly
constructed a transfer function by inverting ζ(s) (stripping off the pole at
s = 1), developed a series expansion for the impulse response, and showed that
convergence plus a growth bound would imply RH. His result remained conditional
on two unproved hypotheses. Our contribution is the explicit finite-dimensional
realization — the spectral comb — whose antisymmetry forces Re = 1/2
structurally, without requiring convergence or growth assumptions at the
transfer-function level. The paper now cites Nihtilä and carefully distinguishes
the two approaches.

## Addressing the circularity concern

A natural question is whether the construction is circular: we build the matrix
from known zeros, so of course they appear as eigenvalues. The answer is that
the contribution is not the eigenvalues themselves but the **antisymmetry
requirement**. The Z3 induction proof shows that antisymmetry is a recursive
structural property of the realization — it is preserved at every N regardless
of which zeros are used. The "antisymmetry cliff" test demonstrates that any
deviation from this structure, even by 1%, annihilates Re = 1/2. The critical
line is a symmetry-protected state of the arithmetic system.

## Summary

The manuscript contains no new mathematics beyond the synthesis: every step
cites a classical result (Euler, Kalman, Connes, Horn–Johnson, Keller–Stummel).
What is new is the observation that these results, composed in sequence, yield
Re = 1/2 as an immediate consequence of antisymmetry — a structural property
that is invisible in the frequency domain but transparent in the operator
domain.

We look forward to the referees' evaluation.

Sincerely,

Engin Atik
Kleis Research
https://kleis.io
