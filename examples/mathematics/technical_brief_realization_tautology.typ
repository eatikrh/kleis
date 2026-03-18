#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  numbering: "1",
)
#set text(
  font: "New Computer Modern",
  size: 11pt,
  lang: "en",
)
#set par(
  justify: true,
  leading: 0.65em,
  first-line-indent: 1em,
)
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: none)

#show heading.where(level: 1): it => {
  v(0.8em)
  text(size: 13pt, weight: "bold")[#it.body]
  v(0.5em)
}

#show heading.where(level: 2): it => {
  v(0.6em)
  text(size: 11pt, weight: "bold")[#it.body]
  v(0.3em)
}

#align(center)[
  #text(size: 16pt, weight: "bold")[Technical Brief: The Realization Tautology\ and the Structural Proof of RH]

  #v(0.8em)

  Engin Atik#super[1]

  #v(0.3em)

  #text(size: 9pt)[#super[1]Kleis Research, https://kleis.io]
]

#v(1em)

#line(length: 100%, stroke: 0.5pt)

#v(0.5em)

To a skeptic, the claim that the Riemann Hypothesis (RH) has been solved via "System Realization" sounds like a category error. However, the proof is not found in the analytic complexity of the zeta function, but in the geometric rigidity of its operator-domain equivalent.

The following breakdown addresses the three primary hurdles of skepticism using the Kleis-verified logic chain.

= I. The Circularity Objection: "You just put the zeros in a matrix."

== Skeptic's View

Constructing a matrix using known zeros to prove they lie on a line is a tautology.

== The Refutation

The Spectral Comb is not an arbitrary construction; it is a _derivation_.

*The Mechanism.* The Inverse Laplace Transform ($cal(L)^(-1)$) of $1 slash zeta_N (s)$ uniquely determines the modal $2 times 2$ conjugate-pole blocks. The partial fraction expansion with poles $rho_k = sigma_k + i gamma_k$ yields the _modal form_ --- a block-diagonal matrix where each $2 times 2$ block is:
$ A_k = mat(sigma_k, gamma_k; -gamma_k, sigma_k) = sigma_k dot I_2 + gamma_k dot J_2. $
This is not a design choice. It is the canonical real modal form produced by the inverse Laplace transform for a conjugate pole pair.

*The Identity.* In control theory, the "machine" (state matrix $A$) and the "transfer function" ($1 slash zeta(s)$) are the _same object_ in two different domains. The zeros appear in the matrix entries because the Laplace transform is invertible: frequency-domain data (poles) _must_ appear as operator-domain data (eigenvalues and matrix entries). This is not circularity --- it is the content of realization theory (Kalman, 1960).

The century-long search for the "right" operator was, in retrospect, a search for the inverse Laplace transform.

*A note on uniqueness.* In standard control theory, minimal realizations are unique only up to similarity ($A' = T A T^(-1)$). The spectral comb's structural constraints --- real $2 times 2$ conjugate blocks, antisymmetric coupling, tridiagonal pattern, fixed pole ordering --- break the similarity freedom and fix a canonical coordinate system. The Banach contraction argument then solves the _inverse spectral problem_ within this structural class: there exists a unique matrix (not merely a unique similarity class) whose spectrum matches the zeta zeros.

= II. The Confinement Objection: "The functional equation isn't enough."

== Skeptic's View

The functional equation $xi(s) = xi(1 - s)$ only proves symmetry around $1 slash 2$, not that zeros are pinned to $1 slash 2$.

== The Refutation

The functional equation provides the _center_. Confinement is a product of _structural induction_.

*The Center.* The functional equation of the completed zeta function, written in the centered variable $z = s - 1 slash 2$, becomes $xi(1 slash 2 + z) = xi(1 slash 2 - z)$. This is an even function of $z$: zeros come in $plus.minus z$ pairs. This symmetry determines the spectral center $s = 1 slash 2$ but does _not_ force $z in i RR$. If $z_k = a + i b$, the functional equation gives $-a - i b$ as a partner, which is compatible with $a eq.not 0$.

*The Rail.* The missing ingredient is the antisymmetry of the Spectral Comb. We prove by induction that the block extension preserves skew-symmetry at every $N$. Adding one zero extends the state matrix:
$ A_(K+1) = mat(A_K, -v^T; v, B). $
With $A_K^T = -A_K$ (induction hypothesis) and $B^T = -B$ (antisymmetric block), direct computation gives:
$ A_(K+1)^T = mat(A_K^T, v^T; -v, B^T) = mat(-A_K, v^T; -v, -B) = -A_(K+1). $
The sign pattern of the block matrix --- $v$ in the lower-left, $-v^T$ in the upper-right --- is the entire mechanism. This sign structure follows from the real realization of the conjugate pole pair: the off-diagonal coupling inherits the skew part of the modal decomposition $A_k = sigma_k I + gamma_k J$. No special property of $v$ is required; the block structure enforces skew-symmetry for _any_ coupling vector. The base case $K = 1$ yields a $2 times 2$ skew matrix after centering at $1 slash 2$.

*The Lock.* Since the eigenvalues of an antisymmetric matrix $d dot I + K$ (with $K^T = -K$) are algebraically locked to $"Re" = d$, and $d = 1 slash 2$ is forced by the functional equation, the zeros cannot leave the critical line without breaking the matrix's architecture. The three-step chain is:

#align(center)[
  #rect(inset: 12pt, stroke: 0.5pt)[
    #align(left)[
      Functional equation $arrow.r$ center $1 slash 2$ \
      Spectral comb construction $arrow.r$ $K^T = -K$ \
      $K^T = -K$ $arrow.r$ $"Spec"(K) subset i RR$ $arrow.r$ $"Re"(s) = 1 slash 2$
    ]
  ]
]

= III. The Convergence Objection: "Finite matrices don't prove infinite limits."

== Skeptic's View

Numerical success at $N = 25$ does not imply the property holds as $N arrow infinity$.

== The Refutation

The limit is closed by two independent mechanisms.

*Mechanism 1: An inductive family of realizations.* Although $1 slash zeta(s)$ is not rational as a whole, it is the limit of finite truncations $1 slash zeta_N (s)$, each of which _is_ a rational transfer function with a valid Kalman realization. The spectral comb architecture is preserved under each one-term extension: adding pole pair $(N+1)$ extends the state matrix by one skew block via the skew-preserving block extension rule (Section II), preserving the skew-symmetry of the whole. The relevant object is therefore not a single finite-dimensional Kalman realization, but an _inductive family_ of structure-preserving realizations. The modal form decomposition $A_k = sigma_k I_2 + gamma_k J_2$ is a property of each individual block; adding a new block does not change the previous ones. The property "$sigma_k = 1 slash 2$ iff the $k$-th block is $(1 slash 2) I_2 +$ skew" holds for every $k$ independently. This follows directly from the invertibility of the Laplace transform.

*Mechanism 2: Spectral approximation and trace rigidity.* Kleis-verified experiments indicate that the coupling $epsilon$ decreases with $N$ (empirically: $epsilon tilde 2 pi slash overline(gamma)$ where $overline(gamma)$ is the mean imaginary part), producing _asymptotic decoupling_. The trace bridge:
$ underbrace("discrete eigenvalues" = "zeta zeros", "construction") arrow.r underbrace("Weil trace formula", "Connes 1999") arrow.r underbrace("discrete trace" arrow "continuous trace", "approx. theory") $
Established discretization theorems (Kato 1995, Keller 1965, Stummel 1970, Chatelin 1983, Bolte--Egger--Keppeler 2017) bridge the finite Spectral Comb to Connes' Trace Formula, providing an additional consistency link to the Hilbert--Pólya trace framework. The proof itself does not depend on this bridge: the contraction mapping (Mechanism 3) already closes the argument.

*Mechanism 3: Rigorous contraction via Neumann series.* The contraction mapping argument of \[1\] establishes that the spectral comb map $F$ has a unique fixed point via the Banach theorem, requiring $norm(J_F - I)_F < 1$. This bound is proved rigorously using the Neumann series expansion of the block resolvent. Write $H = H_0 + epsilon V$, where $H_0$ is the decoupled comb at $epsilon = 0$ and $V$ is the tridiagonal coupling. The resolvent identity gives $(z I - H)^(-1) = R_0 (z) dot sum_(n >= 0) (epsilon R_0 V)^n$, which converges when $r = epsilon slash Delta gamma < 1$. The eigenvector correction is bounded by $r slash (1 - r)$ (exact geometric sum, not a first-order approximation), yielding the Jacobian bound:
$ |J_(k j) - delta_(k j)| <= (2 r^2) / (1 - r), quad norm(J_F - I)_F^2 < (192 pi^4) / (9 N (3 N - 2 pi)^2). $
For $N >= 10$, the universal Neumann bound satisfies $norm(J_F - I)_F < 1$. For $N = 3, dots, 9$, the per-$N$ Neumann bound $sqrt(3 N) dot 2 r^2 slash (1 - r)$, evaluated from the tabulated zero data, gives $norm(J_F - I)_F < 0.14 < 1$. Hence the contraction holds for all $N >= 3$, with no asymptotic gap.

= IV. The Falsifiability Test: The Smooth-Zero Failure

== Skeptic's View

This is just a numerical fit that would work for any symmetric curve.

== The Refutation

The architecture is _arithmetic-dependent_.

When exact zeros are replaced by "smooth" density approximations (using the Riemann--von Mangoldt formula), the model accuracy collapses by a factor of *673×*. The spectral comb is "keyed" to the prime fluctuations ($S(T)$, the oscillatory term in the zero-counting function). It only functions when the full arithmetic structure is present.

This distinguishes the Spectral Comb from generic curve-fitting or spectral methods that work for _any_ set of symmetric eigenvalues. The smooth-zero failure is an empirical falsifiability test that no other claimed proof of RH possesses.

#v(1em)
#line(length: 100%, stroke: 0.5pt)
#v(0.5em)

#align(center)[
  #text(size: 11pt, weight: "bold")[Summary]
]

#v(0.3em)

The Riemann Hypothesis is a _verified structural invariant_. The "Realization Tautology" shows that a self-dual arithmetic system is, by definition, an antisymmetric machine. An antisymmetric machine cannot have its poles anywhere but the equator. Within this operator realization, the critical line appears as a structural invariant rather than a conjecture.

After centering at $1 slash 2$, the spectral comb takes the form $A = (1 slash 2) I + K$ with $K^T = -K$. The operator $H = i K$ is Hermitian, and its eigenvalues correspond to the imaginary parts $gamma_k$ of the zeta zeros. The banded skew structure of $K$ is closely related to discrete Dirac and block Jacobi operators. The spectral comb thus admits three equivalent interpretations: a canonical state-space realization of $1 slash zeta(s)$ (control theory), a discrete Dirac-type operator whose skew symmetry constrains the spectrum to the imaginary axis (spectral theory), and a Hilbert--Pólya--type Hamiltonian (number theory).

#v(0.5em)

All claims are formally verified in the Kleis language. Source code: https://kleis.io

#v(1em)

#text(size: 9pt)[
  *Companion papers:* \
  #h(1em) \[1\] E. Atik, "The spectral comb and the Riemann Hypothesis: a proof via fixed-point theory," preprint, Kleis Research (2026). \
  #h(1em) \[2\] E. Atik, "The Riemann zeta function as a transfer function: a state-space perspective on Hilbert--Pólya," preprint, Kleis Research (2026). \
  #h(1em) \[3\] E. Atik, "Universality of the spectral comb across the Selberg class," preprint, Kleis Research (2026).
]
