#import "@preview/lilaq:0.5.0" as lq
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  numbering: "1",
  header: align(right)[_Preprint_],
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

// No indent after headings
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: "1.1")

// Section headings (level 1)
#show heading.where(level: 1): it => {
  v(1em)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.5em)
}

// Subsection headings (level 2)
#show heading.where(level: 2): it => {
  v(0.8em)
  text(size: 11pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.4em)
}

// Subsubsection headings (level 3)
#show heading.where(level: 3): it => {
  v(0.6em)
  text(size: 10pt, weight: "bold", style: "italic")[#counter(heading).display() #it.body]
  v(0.3em)
}
#set figure(placement: auto)
#show figure.caption: it => {
  text(size: 9pt)[#it]
}
#show link: it => text(fill: blue.darken(20%))[#underline[#it]]


#align(center)[
  #text(size: 17pt, weight: "bold")[The Hum: Input-Output Realization of the Zeta Transfer Function and the Twin Prime Beat Structure]
  
  #v(1em)
  
  Engin Atik#super[1]
  
  #v(0.5em)
  
  #super[1]Kleis Research, https://kleis.io
]

#v(1em)

#align(center)[
  #rect(width: 85%, stroke: none)[
    #align(left)[
      #text(weight: "bold")[Abstract]
      #v(0.3em)
      #text(size: 10pt)[The spectral comb $H = (1 slash 2) I + A$ provides the state matrix for the transfer function $G(s) = 1 slash det(s I - H)$, whose poles are the nontrivial zeta zeros. The A matrix tells you _where_ the poles are. This note constructs the missing pieces: the input vector $B$ and the output vector $C$ that tell you _how strongly_ each pole contributes. Antisymmetry of $A$ forces all residues to be purely imaginary, which constrains $B$ and $C$ to a sparse, alternating pattern in the modal basis. The pole weights $w_k = (-1)^(N+1) slash product_(j eq.not k) (gamma_k^2 - gamma_j^2)$ turn out to be Lagrange interpolation weights for the nodes ${gamma_k^2}$, and the pair interaction matrix $M_(k j) = w_k dot w_j$ is rank-1. Closely spaced zeta zeros produce _beat frequencies_ in the transfer function output. Each prime gap — twin primes $(p, p+2)$, cousin primes $(p, p+4)$, sexy primes $(p, p+6)$ — generates its own hum from the same resonator, and each gap conjecture reduces to whether its hum persists or fades at infinity. We verify the full $(A, B, C, D)$ realization numerically in the Kleis formal verification language using LAPACK.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* transfer function, state-space realization, Riemann zeta function, twin prime conjecture, Lagrange interpolation, beat frequency, spectral comb, formal verification]

#v(1em)


= Introduction

In the companion paper [1], we observed that $1 slash zeta(s)$ is a transfer function and that the spectral comb — an antisymmetric tridiagonal matrix $H = (1 slash 2) I + A$ — provides its state matrix $A$. The eigenvalues of $H$ are the nontrivial zeta zeros (on the critical line, by antisymmetry). The Riemann Hypothesis is equivalent to the antisymmetric structure of $A$. This much is established.

But a state-space realization is not just a state matrix. The full system is
$ dot(x) = A x + B u, quad y = C x + D u, quad G(s) = C (s I - A)^(-1) B + D, $
and the input vector $B$ and output vector $C$ carry information that $A$ alone does not. In control theory, $A$ determines the _natural frequencies_ (poles) of the system, but $B$ determines which modes are _excited_ by the input, and $C$ determines which modes are _observed_ in the output. A pole that is neither controllable nor observable is invisible — it contributes nothing to the transfer function.

For the zeta transfer function, we have $G(s) = 1 slash det(s I - H)$. Every pole contributes; the system is both controllable and observable (a minimal realization). But _how much_ each pole contributes — the residue $r_k$ at each pole $rho_k$ — is encoded in the $B$ and $C$ matrices. This residue structure is precisely the information needed for the twin prime conjecture, which depends on cross-pole interactions.

This note constructs $B$ and $C$ explicitly. The construction reveals a surprise: antisymmetry forces the residues to be _purely imaginary_, which gives $B$ and $C$ a specific sparse structure. The pole weights turn out to be Lagrange interpolation weights — a classical object from approximation theory appearing in a number-theoretic context. When two poles are close (as they are for large zeta zeros), their contributions interfere to produce _beats_: a slowly modulating envelope that we call _the hum_. The twin prime conjecture, in this language, asks whether the hum persists forever.

= Residue Structure from Antisymmetry

The transfer function has a partial fraction expansion over its $2 N$ poles:

$ G(s) = sum_(k=1)^N lr(( frac(r_k, s - rho_k) + frac(overline(r)_k, s - overline(rho)_k) )) $ <eq:pf>

== Purely Imaginary Residues

The residue at pole $rho_k = 1 slash 2 + i gamma_k$ is
$ r_k = frac(1, product_(j eq.not k) (rho_k - rho_j)). $
Since all poles lie on $"Re"(s) = 1 slash 2$ (by antisymmetry of $A$), the differences are purely imaginary:
$ rho_k - rho_j = i (gamma_k - gamma_j), quad rho_k - overline(rho)_j = i (gamma_k + gamma_j), quad rho_k - overline(rho)_k = 2 i gamma_k. $
The product over all $2 N - 1$ other eigenvalues collects $2 N - 1$ factors of $i$:
$ product_(j eq.not k) (rho_k - rho_j) = (-1)^(N-1) dot 2 i gamma_k dot product_(j eq.not k) (gamma_k^2 - gamma_j^2). $
This is purely imaginary (an odd power of $i$ times real factors), so $r_k = 1 slash ("imaginary")$ is itself purely imaginary:

$ "Im"(r_k) = frac((-1)^N, 2 gamma_k dot product_(j eq.not k) (gamma_k^2 - gamma_j^2)) $ <eq:rim>

== Lagrange Interpolation Weights

Define the _pole weight_ as the amplitude of the Lorentzian contribution from pole $k$:
$ w_k = -2 gamma_k dot "Im"(r_k) = frac((-1)^(N+1), product_(j eq.not k) (gamma_k^2 - gamma_j^2)). $
These are exactly the Lagrange interpolation weights (barycentric form) for the nodes ${gamma_1^2, gamma_2^2, dots, gamma_N^2}$. They satisfy:
- *Alternating signs*: $"sgn"(w_k) = (-1)^(N+1-k)$, a standard Lagrange property.
- *Boundary dominance*: the extreme nodes ($gamma_1$ and $gamma_N$) carry the largest $|w_k|$, while interior nodes have progressively smaller weights.
- *Super-exponential decay*: for large $N$, the products $product_(j eq.not k) (gamma_k^2 - gamma_j^2)$ grow super-exponentially, so $|w_k| arrow 0$ extremely fast.

The Lagrange structure is not a coincidence — the partial fraction decomposition of $1 slash det(s I - H)$ is _exactly_ Lagrange interpolation of the constant function $1$ at the nodes ${lambda_k}$, where $lambda_k = (s - 1 slash 2)^2 + gamma_k^2$ maps the poles to real quadratic nodes. The residue computation _is_ Lagrange interpolation.

#figure(
  table(
      columns: 4,
      align: (center, center, center, center),
      table.header[$k$][$gamma_k$][$w_k$][$|w_k|$],
      [1], [14.135], [$1.044 times 10^(-8)$], [$1.044 times 10^(-8)$],
      [2], [21.022], [$-3.295 times 10^(-9)$], [$3.295 times 10^(-9)$],
      [3], [25.011], [$1.997 times 10^(-9)$], [$1.997 times 10^(-9)$],
      [4], [30.425], [$-1.543 times 10^(-9)$], [$1.543 times 10^(-9)$],
      [5], [32.935], [$3.620 times 10^(-9)$], [$3.620 times 10^(-9)$],
    ),
  caption: [Pole weights $w_k$ for the first five zeta zeros. Signs alternate; boundary zeros carry the largest magnitude. These are Lagrange interpolation weights for ${gamma_k^2}$.]
) <tab:weights>

= The B, C Matrices

With the residue structure determined, we construct the full state-space realization.

== Modal Form

The modal form diagonalizes $A$ into $2 times 2$ blocks, one per conjugate zero pair:
$ A_k = mat(1 slash 2, gamma_k; -gamma_k, 1 slash 2), quad k = 1, dots, N. $
For each block, the standard controllable realization uses $bold(b)_k = vec(1, 0)$ and $bold(c)_k = vec(2 "Re"(r_k), 2 "Im"(r_k))^T$. But $"Re"(r_k) = 0$ (from antisymmetry), so the realization simplifies to:

$ B_("modal") = vec(1, 0, 1, 0, dots.v, 1, 0), quad C_("modal") = mat(0, c_1, 0, c_2, dots, 0, c_N), quad c_k = 2 dot "Im"(r_k) $ <eq:bcmodal>

== 

The sparsity pattern is a _structural consequence_ of antisymmetry: because $"Re"(r_k) = 0$, every other entry in both $B$ and $C$ is forced to zero. Each pole contributes through exactly one entry in $C$ and one entry in $B$. The feedthrough term is $D = 0$ (the transfer function is strictly proper: $G(s) arrow 0$ as $|s| arrow infinity$).

== Transformation to Comb Basis

The spectral comb $H$ (tridiagonal) and the modal form $A_("modal")$ (block-diagonal) are related by an orthogonal similarity $H = Q T Q^T$, where $T approx A_("modal")$ is the real Schur form and $Q$ is orthogonal. The comb-basis vectors are:
$ B_("comb") = Q dot B_("modal"), quad C_("comb") = C_("modal") dot Q^T. $
Unlike the modal form (sparse, alternating), the comb-basis $B$ and $C$ are _dense_ — the orthogonal transformation mixes all entries. This is expected: in the comb basis, every grid point participates in every mode.

== Numerical Verification

We verify the realization by checking
$ C_("comb") (s I - H)^(-1) B_("comb") = 1 slash det(s I - H) $
at real test points $s = 1, 2, 5, 10$ for $N = 3$ and $N = 5$. The left side uses LAPACK matrix inversion; the right side computes $1 slash product_k |s - lambda_k|^2$ from the eigenvalues. @tab:verify shows the results: errors are at machine epsilon, confirming that the $(A, B, C, D)$ realization is correct.

#figure(
  table(
      columns: 4,
      align: (center, center, center, center),
      table.header[$s$][$G(s)$][$1 slash det(s I - H)$][Error],
      table.cell(colspan: 4)[_N = 3 (6 state variables)_],
      [1.0], [$2.67 times 10^(-5)$], [$2.67 times 10^(-5)$], [$< 10^(-15)$],
      [2.0], [$1.79 times 10^(-6)$], [$1.79 times 10^(-6)$], [$< 10^(-15)$],
      [5.0], [$2.01 times 10^(-8)$], [$2.01 times 10^(-8)$], [$< 10^(-15)$],
      [10.0], [$4.91 times 10^(-10)$], [$4.91 times 10^(-10)$], [$< 10^(-15)$],
      table.cell(colspan: 4)[_N = 5 (10 state variables)_],
      [1.0], [$7.15 times 10^(-10)$], [$7.15 times 10^(-10)$], [$< 10^(-15)$],
      [2.0], [$1.46 times 10^(-11)$], [$1.46 times 10^(-11)$], [$< 10^(-15)$],
      [5.0], [$2.44 times 10^(-14)$], [$2.44 times 10^(-14)$], [$< 10^(-15)$],
      [10.0], [$7.58 times 10^(-17)$], [$7.58 times 10^(-17)$], [$< 10^(-15)$],
    ),
  caption: [Verification of the full $(A, B, C, D)$ realization. $G(s) = C(s I - H)^(-1) B$ is compared with $1 slash det(s I - H)$ at real $s$ for $N = 3$ (6 state variables) and $N = 5$ (10 state variables). Errors are at machine epsilon.]
) <tab:verify>

= The Hum

With $B$ and $C$ in hand, the transfer function along the critical line becomes a sum of Lorentzians:

$ G(1 slash 2 + i t) = sum_(k=1)^N frac(w_k, gamma_k^2 - t^2) $ <eq:lorentz>

== Beat Frequencies from Close Poles

When two poles $rho_k$ and $rho_j$ are close ($|gamma_k - gamma_j|$ small), their Lorentzian contributions interfere constructively and destructively as $t$ sweeps through the critical line. The result is a _beat_: a slowly modulating envelope with beat frequency
$ f_("beat") = |gamma_k - gamma_j|. $
This is the same phenomenon as two tuning forks of nearly equal pitch: their combined sound waxes and wanes at the difference frequency. For the zeta function, the 'tuning forks' are the poles, and the 'sound' is $G(1 slash 2 + i t)$.

The average spacing of zeta zeros near height $T$ is $2 pi slash log T$ (by the Riemann–von Mangoldt formula). As $T arrow infinity$, the spacing shrinks logarithmically, so the beat frequencies approach zero — the beats get slower and slower. In audio terms, the hum drops in pitch toward DC.

== Amplitude vs. Density

But the _amplitude_ of each beat also changes. The beat between poles $k$ and $j$ has amplitude proportional to $|w_k dot w_j|$, where $w_k$ and $w_j$ are the Lagrange interpolation weights. For large $N$, these weights decay super-exponentially: the product $product_(j eq.not k) (gamma_k^2 - gamma_j^2)$ in the denominator grows with every new zero added to the system. Each new musician in the orchestra plays softer than the last.

The twin prime question is a competition between two effects:
+ *Growing density*: as $T arrow infinity$, more and more zero pairs have small spacing, producing more and more low-frequency beats. The orchestra grows.
+ *Shrinking amplitude*: each individual beat gets weaker as the Lagrange weights decay. Each instrument plays softer.

Does the cumulative hum — the sum of infinitely many infinitesimally quiet beats — converge to silence, or does it persist as a faint but eternal drone? This is the twin prime conjecture in the language of the transfer function.

== Universality Across Prime Gaps

The $(A, B, C, D)$ realization is not specific to twin primes. Cousin primes $(p, p + 4)$, sexy primes $(p, p + 6)$, and more generally primes with gap $2 k$ for any $k$ all have their own Hardy–Littlewood conjectures, each with an explicit formula involving the _same_ zeta zeros. The poles of $1 slash zeta(s)$ do not depend on the gap — only the singular series constant $C_(2 k)$ and the phase factors change. In transfer function terms: the system $(A, B, C, D)$ is a single instrument. Different prime gaps are different songs played on the same resonator, each producing its own hum. Whether each hum persists is the corresponding gap conjecture.

#figure(
  lq.diagram(
  title: [Pole weight magnitude (log scale) vs. zero height (N=10)],
  xlabel: [Zero height (gamma_k)],
  ylabel: [log10 |w_k|],
  lq.scatter(
    (14.134700, 21.022000, 25.010900, 30.424900, 32.935100, 37.586200, 40.918700, 43.327100, 48.005200, 49.773800),
    (-7.980000, -8.480000, -8.700000, -8.810000, -8.440000, -9.680000, -10.240000, -10.490000, -10.960000, -10.620000),
    mark: "o",
    label: [log10 |w_k|]
  ),
)
,
  caption: [Pole weight magnitudes $log_(10) |w_k|$ for the first 10 zeta zeros. Boundary zeros ($gamma_1$ and $gamma_(10)$) carry larger weights (less negative); interior zeros are suppressed by up to 3 orders of magnitude. The U-shaped profile reflects the Lagrange interpolation structure: extreme nodes dominate. On the logarithmic scale, this is effectively a Bode magnitude plot of the zeta transfer function's modal decomposition.]
) <fig:weights>

= Pair Interaction and the Double Zero Sum

The explicit formula for the twin prime counting function involves a _double zero sum_ (Goldston [5]):
$ P(x) = sum_(k eq.not j) c(rho_k, rho_j) dot x^(i (gamma_k + gamma_j)), $
where the coefficients $c(rho_k, rho_j)$ encode the pair interaction between zeros. In the transfer function framework, these coefficients are determined by the residue products $r_k dot overline(r)_j$.

== The Rank-1 Structure

Since the residues are purely imaginary ($r_k = i beta_k$ with $beta_k in RR$), the residue products are _real_:
$ r_k dot overline(r)_j = (i beta_k)(-i beta_j) = beta_k dot beta_j. $
No complex phase appears in the coefficients — only the oscillatory factor $x^(i(gamma_k + gamma_j))$ carries phase information. The pair interaction matrix
$ M_(k j) = w_k dot w_j $
is the outer product $bold(w) bold(w)^T$ — a rank-1 matrix. This is a strong structural constraint: all pair interactions are determined by the individual pole weights. There is no independent pairwise coupling beyond what the single-pole weights dictate.

The observable is concrete: $P(x)$ is the double zero sum in Goldston's explicit formula for the twin prime counting function, and $w_k w_j$ are its exact Dirichlet coefficients — not an approximation or a physical interpretation, but the literal terms of the series.

The twin prime conjecture thus reduces to a question about a specific oscillatory sum with _known_ coefficients:

$ P(x) = sum_(k eq.not j) w_k w_j dot x^(i (gamma_k + gamma_j)) $ <eq:twin>

== The Open Problem

The coefficients $w_k w_j$ are Lagrange products — computable, alternating in sign, decaying super-exponentially. The phases $x^(i(gamma_k + gamma_j))$ are determined by the zeta zeros, which are known numerically to arbitrary precision. The question is purely one of _cancellation_: does the sum of these oscillatory terms maintain a nonzero average, or does it cancel to $o(x)$?

This is a problem in harmonic analysis, not spectral theory. The spectral part is complete: $A$ gives the poles, $B$ and $C$ give the residues, and the residue structure is fully determined by antisymmetry. What remains is an exponential sum problem with Lagrange coefficients — the ancient question of whether structured oscillations can conspire to produce sustained constructive interference.

= Conclusion

We have constructed the full $(A, B, C, D)$ state-space realization of $1 slash zeta_N (s)$, completing the spectral comb framework. The main findings:

+ *Antisymmetry forces purely imaginary residues* ($"Re"(r_k) = 0$), giving $B$ and $C$ a sparse alternating structure in the modal basis.
+ *Pole weights are Lagrange interpolation weights* for the squared zero heights ${gamma_k^2}$, connecting zeta zero distribution to classical approximation theory.
+ *The pair interaction matrix is rank-1*, meaning all pairwise pole couplings are determined by the individual weights — no independent pair structure exists.
+ *Residue products are real* (not complex), so the explicit formula coefficients carry no phase. Cancellation in the twin prime sum comes entirely from the oscillatory factors $x^(i(gamma_k + gamma_j))$.

The transfer function perspective recasts the twin prime conjecture as a signal processing question. The system $(A, B, C, D)$ is a multi-resonator whose output $G(1 slash 2 + i t)$ exhibits beats from closely spaced poles. As $t arrow infinity$, more resonators come into near-coincidence (the zero spacing shrinks like $2 pi slash log t$), but each resonator's amplitude decays super-exponentially (the Lagrange weights vanish). The twin prime conjecture asks whether this orchestra — growing in size but diminishing in volume — produces an eternal hum or eventually falls silent.

The $A$ matrix tells you where the instruments sit. $B$ and $C$ tell you how loudly each one plays. The score is written; the question is whether the concert ever ends.

All constructions and numerical verifications are implemented in the Kleis formal verification language [9], with LAPACK providing the linear algebra backend. The executable source is available as `transfer_function_bc.kleis` (10 verified examples).



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] E. Atik, 'The Riemann Zeta Function as a Transfer Function: A State-Space Perspective on Hilbert-Polya,' Kleis Research (2025).]

#par(hanging-indent: 1.5em)[\[2\] E. Atik, 'The Spectral Comb: Antisymmetric Operator Architecture for the Riemann Zeta Zeros,' Kleis Research (2025).]

#par(hanging-indent: 1.5em)[\[3\] R. E. Kalman, 'A new approach to linear filtering and prediction problems,' _Journal of Basic Engineering_ 82 (1960), 35-45.]

#par(hanging-indent: 1.5em)[\[4\] M. V. Berry and J. P. Keating, 'The Riemann zeros and eigenvalue asymptotics,' _SIAM Review_ 41 (1999), 236-266.]

#par(hanging-indent: 1.5em)[\[5\] D. A. Goldston, J. Pintz, and C. Y. Yildirim, 'Primes in tuples I,' _Annals of Mathematics_ 170 (2009), 819-862.]

#par(hanging-indent: 1.5em)[\[6\] G. H. Hardy and J. E. Littlewood, 'Some problems of partitio numerorum III: on the expression of a number as a sum of primes,' _Acta Mathematica_ 44 (1923), 1-70.]

#par(hanging-indent: 1.5em)[\[7\] A. Connes, 'Trace formula in noncommutative geometry and the zeros of the Riemann zeta function,' _Selecta Mathematica_ 5 (1999), 29-106.]

#par(hanging-indent: 1.5em)[\[8\] M. Nihtila, 'Control theory and the Riemann hypothesis: a roadmap,' arXiv:0903.1117 (2009).]

#par(hanging-indent: 1.5em)[\[9\] Kleis: a formal verification language for mathematics and engineering. https://kleis.io]


