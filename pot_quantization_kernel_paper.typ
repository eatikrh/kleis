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
  #text(size: 17pt, weight: "bold")[Quantization as Projection Kernel: Why Classical and Quantum Are Related by a Kernel]
  
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
      #text(size: 10pt)[Every known quantization scheme --- canonical (Dirac), geometric (Kostant--Souriau), Berezin--Toeplitz, deformation (Kontsevich), BRST, Weyl--Wigner, path integral (Feynman) --- maps a commutative classical algebra to a non-commutative quantum algebra via a structure-preserving map with three universal invariants: (i) bracket preservation on an admissible subalgebra (Dirac's rule), (ii) non-trivial null space (Groenewold--van Hove obstruction), and (iii) image/null decomposition of the classical observable algebra. We identify this as the kernel pattern formalized in Volumes I--X of the Projected Ontology Theory (POT) series. Different quantization schemes realize the image/null decomposition through different algebraic mechanisms: Berezin--Toeplitz via an idempotent Bergman projection ($P^2 = P$), BRST via a nilpotent cohomological operator ($Q^2 = 0$), and deformation quantization via a continuous star-product deformation parameterized by $planck$. What unifies them is not idempotence but the decomposition itself: every scheme partitions the classical algebra into a part that maps faithfully to the quantum algebra (image) and a part that does not (null space). We prove that the Groenewold obstruction is not a defect of quantization but a structural necessity: the null space is non-empty by construction, and the discrepancy between classical and quantum is controlled by $planck$. As a worked example, we apply the quantization kernel to the harmonic oscillator and show that: the discrete spectrum $E_n = planck omega (n + 1\/2)$ is the image; the zero-point energy $planck omega \/2$ is a property of the kernel with no classical analog; and the energy spacing $planck omega$ is the sampling resolution. Volume X proved that infinities are null-space elements of the Hadamard projection; this volume proves that the quantum algebra itself is the image of a quantization kernel applied to the classical algebra. The ontological conclusion: classical and quantum mechanics are related by a structure-preserving kernel whose null space is the uncertainty principle. All 26 structural results are machine-verified by the Z3 SMT solver in the Kleis formal verification language.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* quantization, projection kernel, Poisson bracket, commutator, Dirac correspondence, Groenewold-van Hove theorem, Berezin-Toeplitz, BRST cohomology, deformation quantization, star product, Kontsevich, harmonic oscillator, projected ontology, formal verification, Z3]

#v(1em)


= Introduction

The passage from classical mechanics to quantum mechanics is the oldest open structural question in theoretical physics. Dirac's 1925 insight [Dir1925] identified the core operation: replace the Poisson bracket $\{f, g\}$ with the quantum commutator $[hat(f), hat(g)] \/ (i planck)$, and require that this replacement preserves the Lie algebra structure. The 'quantization problem' has since been attacked from many directions --- geometric quantization (Kostant [Kos1970], Souriau [Sou1970]), Berezin--Toeplitz quantization (Berezin [Ber1975], Bordemann--Meinrenken--Schlichenmaier [BMS1994]), deformation quantization (Bayen et al. [BFFLS1978], Kontsevich [Kon1997]), BRST quantization (Becchi--Rouet--Stora [BRS1976], Tyutin [Tyu1975]), Weyl--Wigner quantization (Weyl [Wey1927], Wigner [Wig1932], Moyal [Moy1949]) --- and in every case the resulting map has the same algebraic anatomy:

+ A *bracket-preserving* map on an admissible subalgebra (Dirac's rule) from a commutative source to a non-commutative image.
+ A *non-trivial null space* (the Groenewold--van Hove obstruction [Gro1946, vH1951]): the kernel is genuinely lossy.
+ An *image/null decomposition* of the classical algebra into a part that quantizes faithfully and a part that does not.

Different schemes realize this decomposition through different algebraic mechanisms --- idempotent projection (Berezin--Toeplitz), nilpotent cohomology (BRST), or continuous deformation (star product) --- but all three invariants are present in every case. This paper argues that this pattern is not a coincidence. It is the kernel pattern formalized in Volumes I--X of the Projected Ontology Theory (POT) series. The same algebraic structure that identifies physical fields as images of Green's function kernels (Volumes I--VI), that identifies renormalization as a projection kernel (Volume VII), and that identifies infinities as null-space elements of the Hadamard projection (Volume X) also identifies quantum mechanics as the image of a quantization kernel applied to classical mechanics.

The paper is organized as follows. Section 2 constructs the quantization kernel and establishes Dirac's correspondence. Section 3 proves the Groenewold obstruction: the kernel has a non-trivial null space. Section 4 shows that Berezin--Toeplitz quantization is literally a projection. Section 5 constructs the BRST cohomological projection. Section 6 presents deformation quantization as a kernel deformation. Section 7 addresses the 'overloaded projection' objection by distinguishing the three algebraic realizations and identifying what actually unifies them. Section 8 applies the framework to the harmonic oscillator. Section 9 gives a structural remark on the analogy with sampling theory. Section 10 connects to Volumes VII and X. Section 11 draws the ontological conclusion.

= The Quantization Kernel

We formalize the quantization map as a kernel in the POT sense.

The classical algebra is the space of smooth functions on a symplectic manifold $(M, omega)$, equipped with:
- Pointwise multiplication: $(f g)(x) = f(x) g(x)$, which is *commutative*.
- Poisson bracket: $\{f, g\} = omega^(i j) partial_i f thin partial_j g$, which satisfies antisymmetry, the Jacobi identity, and the Leibniz rule.

The quantum algebra is the space of operators on a Hilbert space $cal(H)$, equipped with:
- Operator multiplication: $(A B)|psi angle.r = A(B|psi angle.r)$, which is *non-commutative*.
- Commutator: $[A, B] = A B - B A$, which satisfies antisymmetry and the Jacobi identity.

The quantization kernel $Q : C^infinity(M) arrow cal(B)(cal(H))$ maps classical observables to quantum operators. Dirac's canonical quantization rule (1925) requires:

$ Q(\{f, g\}) = 1 / (i planck) [Q(f), Q(g)] $

for observables in the *admissible subalgebra*. For the fundamental pair $(q, p)$ this gives:

$ Q(\{q, p\}) = Q(1) = 1 / (i planck) [hat(q), hat(p)] $

which forces $[hat(q), hat(p)] = i planck$ --- the canonical commutation relation. The bracket structure of classical mechanics is preserved in the image.

*Theorem 1 (Classical commutativity).* The source algebra is commutative: $f g = g f$ for all classical observables $f, g$.

*Theorem 2 (Quantum non-commutativity).* The image algebra is non-commutative: there exist operators $A, B$ with $A B eq.not B A$.

Both are axiomatized in `QuantizationKernel` and verified by Z3.

= The Groenewold Obstruction: Non-Trivial Null Space

Can Dirac's rule hold for *all* classical observables? Groenewold [Gro1946] and van Hove [vH1951] proved the answer is no.

*Theorem 3 (Groenewold--van Hove).* There is no linear map $Q : C^infinity(M) arrow cal(B)(cal(H))$ satisfying $Q(\{f, g\}) = (1 \/ i planck)[Q(f), Q(g)]$ for all $f, g in C^infinity(M)$.

Specifically, Groenewold showed that for the harmonic oscillator phase space $T^* RR$, the identity $\{q^2, p^2\} = 4 q p$ combined with $Q(\{q^2, p^2\}) = (1\/i planck)[Q(q^2), Q(p^2)]$ produces a contradiction with $Q(q p) = (hat(q) hat(p) + hat(p) hat(q))\/2$ at the level of third-order polynomials. The obstruction is inherent: it cannot be removed by any choice of ordering prescription.

In the POT framework, this means the quantization kernel $Q$ has a *non-trivial null space*: the set of classical observables for which bracket preservation fails. This is not a defect --- it is the *structural content* of quantization. The null space is where classical and quantum mechanics genuinely disagree, and the discrepancy is controlled by $planck$.

The parallel with Volume X is precise:

#figure(
  table(
    columns: (auto, auto, auto),
    inset: 8pt,
    align: horizon,
    table.header([], [*Vol X (Hadamard)*], [*Vol XI (Quantization)*]),
    [Source], [Formal series], [Classical observables],
    [Image], [Finite observables], [Quantum operators],
    [Null space], [UV divergences], [Ordering ambiguities],
    [Admissible domain], [$n >= 0$ coefficients], [Linear/quadratic observables],
    [Obstruction], [Laurent poles], [Groenewold--van Hove],
    [Projection], [Hadamard $P$], [Quantization $Q$],
  ),
  caption: [Structural parallel between Volumes X and XI.],
) <vol-x-xi-table>

Axiomatized in `QuantizationKernel` (Dirac + Groenewold) and verified by Z3.

= Berezin--Toeplitz: Quantization Is Literally Projection

On a compact Kähler manifold $(M, omega, J)$, the Berezin--Toeplitz quantization scheme makes the projection structure fully explicit. Let $L^(times.circle m)$ denote the $m$-th tensor power of the prequantum line bundle, and let $H^0(M, L^(times.circle m))$ be the space of holomorphic sections. The Bergman projection

$ P_m : L^2(M, L^(times.circle m)) arrow H^0(M, L^(times.circle m)) $

is the orthogonal projection onto holomorphic sections. Its integral kernel is the Bergman kernel $K_m(x, y)$, from which the Toeplitz operator is constructed:

$ T_m(f) = P_m compose M_f : H^0 arrow H^0 $

where $M_f$ is multiplication by the classical observable $f$. The quantum operator associated to $f$ is $T_m(f)$.

The Bergman projection $P_m$ satisfies the POT kernel axioms:

+ *Idempotent*: $P_m^2 = P_m$. Applying the projection twice is the same as applying it once.
+ *Image = quantum Hilbert space*: holomorphic sections $s in H^0$ are fixed: $P_m(s) = s$.
+ *Non-trivial null space*: non-holomorphic sections are projected: $P_m(s) eq.not s$ for $s in.not H^0$.

Bordemann, Meinrenken, and Schlichenmaier [BMS1994] proved that as $m arrow infinity$:

$ T_m(f) T_m(g) - T_m(f g) = O(1 \/m) $
$ m [T_m(f), T_m(g)] - i T_m(\{f, g\}) = O(1 \/m) $

which establishes that Berezin--Toeplitz quantization recovers the Dirac correspondence in the semiclassical limit (with $planck tilde 1\/m$).

The ontological reading: the Bergman kernel *is* the quantization kernel. The classical algebra lives on $L^2$. The quantum algebra lives on $H^0$. The passage from classical to quantum is orthogonal projection --- literally the same mathematical operation as the Hadamard projection of Volume X, applied in a different space.

Axiomatized in `BerezinToeplitz` and verified by Z3 (4 results).

= BRST Cohomology: Physical States as Cohomological Projection

In gauge quantum field theory, the full Fock space includes unphysical ghost degrees of freedom introduced by gauge fixing [FP1967]. The BRST operator $Q$ (Becchi--Rouet--Stora [BRS1976], Tyutin [Tyu1975]) is a nilpotent operator acting on the extended Fock space:

$ Q^2 = 0 $

This nilpotency defines a cohomological projection. The physical Hilbert space is the BRST cohomology:

$ cal(H)_("phys") = "Ker"(Q) \/ "Im"(Q) $

The structure is:
- *BRST-closed states*: $Q|psi angle.r = 0$ (candidates for physical states).
- *BRST-exact states*: $|psi angle.r = Q|chi angle.r$ (pure gauge, unphysical).
- *Physical states*: closed but not exact.

Because $Q^2 = 0$, every exact state is automatically closed ($"Im"(Q) subset.eq "Ker"(Q)$), so the quotient is well-defined. The image of the BRST projection is the physical Hilbert space; the null space (exact states) contains the gauge artifacts.

This is the POT kernel pattern applied to gauge theories:
- $"Image"(Q^("BRST"))$ = physical states (gauge-invariant).
- $"Null"(Q^("BRST"))$ = gauge artifacts (ghosts, longitudinal photons).

The BRST projection is not idempotent in the same sense as the Bergman projection (it is nilpotent, $Q^2 = 0$, rather than $Q^2 = Q$), but the *cohomological quotient* it defines is a genuine projection from the extended space to the physical subspace. The two kinds of projection --- idempotent and cohomological --- are unified in the POT framework as different realizations of the image/null decomposition.

Axiomatized in `BRSTProjection` and verified by Z3 (4 results).

= Deformation Quantization: Star Product as Kernel Deformation

Deformation quantization (Bayen, Flato, Frønsdal, Lichnerowicz, and Sternheimer [BFFLS1978]) reformulates quantum mechanics directly on phase space, without Hilbert spaces or operators, by deforming the classical product:

$ f star g = f g + sum_(k=1)^infinity planck^k B_k(f, g) $

where $B_k$ are bidifferential operators (determined by the Poisson structure). The key property:

$ f star g - g star f = i planck \{f, g\} + O(planck^2) $

Kontsevich [Kon1997] proved that a star product exists on *every* Poisson manifold (his formality theorem), settling a 20-year conjecture. The bidifferential operators $B_k$ are themselves integral kernels --- they define the quantization via convolution.

The star product demonstrates the kernel structure of quantization:
- At $planck = 0$: $f star g = f g$ (commutative, classical).
- At $planck > 0$: $f star g eq.not g star f$ in general (non-commutative, quantum).
- The deformation parameter $planck$ controls the 'distance from classical.'

In the POT language: the star product is a one-parameter family of kernels $Q_planck$ interpolating between the identity kernel ($planck = 0$, classical) and the quantum kernel ($planck > 0$). The 'quantization' is the deformation itself; the classical limit is the trivial projection.

*Theorem 4.* The commutative limit of the star product recovers the classical product: if $f star g = g star f$ for all $f, g$, then $f star g = f g$.

Axiomatized in `DeformationQuantization` and verified by Z3 (3 results).

= Three Realizations of the Image/Null Decomposition

A legitimate objection: the three quantization schemes discussed in Sections 4--6 use different algebraic mechanisms. Are we simply calling everything a 'projection'? We address this directly.

The three mechanisms are genuinely different:

+ *Idempotent* (Berezin--Toeplitz): $P^2 = P$. The Bergman projection is an orthogonal projection in the functional-analytic sense. Image and null space are orthogonal complements in $L^2$.
+ *Cohomological* (BRST): $Q^2 = 0 eq.not Q$. The BRST operator is nilpotent, not idempotent. The 'projection' is the passage to the cohomology $"Ker"(Q) \/ "Im"(Q)$, which is a quotient, not an orthogonal complement.
+ *Deformative* (star product): neither $P^2 = P$ nor $Q^2 = 0$. The star product is a continuous one-parameter deformation. The 'projection' is the limiting map $planck arrow 0$ that recovers the classical product.

What unifies them is *not* a shared algebraic operation. It is a shared *structural pattern*: each scheme decomposes the classical observable algebra into two parts --- a part that maps faithfully to the quantum algebra (the image) and a part where the classical-quantum correspondence breaks down (the null space). The decomposition satisfies, in every case:

+ The image algebra is non-commutative (quantum).
+ The bracket is preserved on the admissible part of the source (Dirac's rule holds on a subalgebra).
+ The null space is non-trivial (Groenewold--van Hove obstruction).

We formalize this as follows. A *quantization kernel* is a linear map $Q : cal(A)_("cl") arrow cal(A)_("qu")$ together with a decomposition $cal(A)_("cl") = cal(A)_("adm") plus.circle cal(A)_("null")$ such that:
- $Q$ restricted to $cal(A)_("adm")$ is a Lie algebra homomorphism (bracket-preserving).
- $Q$ restricted to $cal(A)_("null")$ is *not* a Lie algebra homomorphism.
- $cal(A)_("null") eq.not {0}$ (the obstruction is non-trivial).

This definition does not require idempotence. Berezin--Toeplitz, BRST, and deformation quantization are all instances. The contribution of this paper is not to claim they are 'the same operation,' but to identify that they share these three invariants --- and that these invariants are the same ones that appear in the POT kernel pattern of Volumes I--X.

All three invariants are axiomatized and verified by Z3 in the theory file.

= Worked Example: The Harmonic Oscillator

As a concrete end-to-end demonstration, we apply the quantization kernel to the harmonic oscillator --- the simplest system that exhibits all three structural invariants.

=== The classical system

The classical Hamiltonian is:

$ H(q, p) = p^2 / (2m) + (m omega^2 q^2) / 2 $

Phase-space orbits are ellipses of area $A(E) = 2 pi E \/ omega$. The energy $E$ is *continuous*: any non-negative real value is allowed. The fundamental Poisson bracket is $\{q, p\} = 1$.

=== The quantum system (image of the kernel)

The quantization kernel maps $H arrow.r hat(H)$ with eigenvalues:

$ E_n = planck omega (n + 1\/2), quad n = 0, 1, 2, dots $

This is the *image* of $Q$ applied to the classical Hamiltonian. The three structural invariants and their consequences:

*1. Energy levels are discrete.* $E_0 = 0.5$, $E_1 = 1.5$, $E_2 = 2.5$, $E_3 = 3.5$, $E_4 = 4.5$ (in natural units $planck = omega = 1$). Verified by computation.

*2. Zero-point energy is a quantum artifact.* $E_0 = planck omega \/ 2 = 0.5 > 0$, while the classical minimum energy is $E = 0$ (oscillator at rest). The zero-point energy has *no classical analog* --- it is a property of the quantization kernel, not the classical source. The uncertainty principle $Delta q thin Delta p >= planck \/ 2$ prevents the oscillator from occupying the phase-space origin.

*3. Energy spacing is constant (sampling resolution).* $Delta E = E_(n+1) - E_n = planck omega = 1.0$ for all $n$. This is the 'sampling resolution' of the quantization kernel: the minimum energy difference the quantum system can distinguish. Below this resolution, energy differences lie in the null space.

*4. Dirac's correspondence is preserved.* The fundamental bracket maps: $\{q, p\} = 1 arrow.r [hat(q), hat(p)] \/ (i planck) = 1$. The kernel preserves the Lie algebra structure.

*5. Phase-space area is quantized.* $Delta A = A(E_(n+1)) - A(E_n) = 2 pi planck = h$. The phase space is partitioned into cells of area $h$ (Planck's constant). The minimum cell area is $A(E_0) = pi planck > 0$: the Nyquist limit of the quantization kernel.

=== The classical limit

As $n arrow infinity$: $Delta E \/ E_n = 1 \/ (n + 1\/2) arrow 0$. At $n = 100$, the relative spacing is below 1% --- the discrete spectrum is effectively continuous. The quantization kernel approaches the identity at large quantum numbers: classical mechanics is the limit of trivial projection.

All 7 numerical results are verified by direct computation in `theories/pot_quantization_kernel_worked.kleis`.

= Structural Remark: Correspondence with Discrete-Time Control

We note a structural analogy --- not a physical identification --- between the quantization kernel and discretization in digital control theory.

In hybrid control systems, a continuous-time plant is controlled by a discrete-time controller through an analog-to-digital converter (ADC). The ADC samples the continuous signal at period $T$, producing a discrete-time representation. The correspondence is algebraic:

#figure(
  table(
    columns: (auto, auto, auto),
    inset: 8pt,
    align: horizon,
    table.header([], [*Quantization*], [*Digital Control*]),
    [Continuous source], [Phase space $C^infinity(M)$], [Continuous plant $G(s)$],
    [Discrete image], [Hilbert space $cal(H)$], [Discrete model $G(z)$],
    [Resolution parameter], [$planck$], [Sampling period $T$],
    [Information loss], [Uncertainty principle], [Aliasing],
    [Transform pair], [Poisson $arrow$ commutator], [Laplace $arrow$ Z-transform],
    [Minimum cell], [$Delta q thin Delta p >= planck\/2$], [Nyquist: $f_("max") < 1\/(2T)$],
  ),
  caption: [Structural analogy between quantization and discrete-time control.],
) <sampling-table>

Both are instances of the same algebraic pattern: a linear, structure-preserving map from a continuous algebra to a discrete algebra, with a resolution parameter controlling the information loss and a fundamental limit on distinguishable states.

We emphasize that this is a *structural observation*, not a physical claim. We are not asserting that quantization 'is' sampling, or that $planck$ 'is' a sampling period. The observation is that the same structural pattern (bracket/transform preservation on an admissible domain, non-trivial null space, resolution limit) arises independently in two different engineering/physical contexts. This universality is evidence that the projection kernel pattern is a fundamental structural phenomenon, not an accident of quantum mechanics.

= Connection to Volumes VII and X

The quantization kernel completes a three-volume arc within the POT series:

*Volume VII: Renormalization as projection kernel.* The composite QFT kernel $K_("QFT") = "FP" compose K_("ren") compose K_("path")$ maps sources to finite observables. The finite-part operator FP is a projection: it extracts admissible content and discards UV divergences.

*Volume X: Infinities as null-space elements.* The Hadamard projection $P$ on formal power series decomposes every amplitude into Image($P$) (finite observables) and Null($P$) (UV divergences, curvature singularities). The infinities are representational artifacts.

*Volume XI (this paper): Quantum mechanics as projection image.* The quantization kernel $Q$ maps classical observables to quantum operators. Image($Q$) is the quantum algebra. Null($Q$) is the Groenewold obstruction.

The pattern is identical across all three:

#figure(
  table(
    columns: (auto, auto, auto, auto),
    inset: 8pt,
    align: horizon,
    table.header([], [*Vol VII*], [*Vol X*], [*Vol XI*]),
    [Kernel], [$K_("QFT")$], [$P$ (Hadamard)], [$Q$ (quantization)],
    [Source], [QFT source $J$], [Formal series], [Classical $C^infinity(M)$],
    [Image], [Finite observable], [Admissible series], [Quantum $cal(B)(cal(H))$],
    [Null space], [Divergent intermediate], [UV poles / singularities], [Ordering ambiguity],
    [Admissible], [Regularized values], [$n >= 0$ coefficients], [Linear/quadratic obs.],
    [Decomposition], [$"FP"^2 = "FP"$], [$P^2 = P$], [Image/Null (Sec. 7)],
  ),
  caption: [The POT kernel pattern across three volumes.],
) <pot-pattern-table>

The unification is not metaphorical. The same structural pattern --- bracket preservation on an admissible domain, non-trivial null space, image/null decomposition --- applies in each case. The three volumes together establish that the POT kernel pattern governs: (1) how finite observables emerge from divergent intermediates (Vol VII), (2) why infinities are representational artifacts (Vol X), and (3) why the quantum world is a projected image of the classical world (Vol XI).

= The Ontological Conclusion

The quantization kernel establishes a structural claim:

$ "Quantum algebra" = "Image"(Q) $
$ "Classical excess" = "Null"(Q) $

The quantum algebra corresponds to the image of a structure-preserving map applied to the classical observable algebra. The classical features that do not survive --- commutative products, precise phase-space localization, bracket relations beyond the admissible subalgebra --- lie in the null space of $Q$.

We are careful about what this does and does not assert. It does *not* assert that classical mechanics is 'correct' and quantum mechanics is derived from it. It does *not* assert that quantum mechanics is merely a filtered version of a more fundamental classical reality. What it asserts is that the *relationship* between the two is a kernel --- a structure-preserving map with non-trivial null space --- and that every known quantization scheme instantiates this relationship through the same three invariants (bracket preservation, non-trivial null space, image/null decomposition), regardless of the specific algebraic mechanism used to realize it.

The uncertainty principle, in this framework, is not a mysterious limitation on measurement. It is the *non-trivial null space* of the quantization kernel: the mathematical statement that $Q$ is not an isomorphism. The minimum phase-space cell $Delta q thin Delta p >= planck \/ 2$ is the kernel's resolution limit.

The zero-point energy of the harmonic oscillator ($planck omega \/ 2$) is not a property of the classical system or the quantum system alone. It is a property of the *kernel itself* --- an artifact of the map. Just as the Hadamard projection in Volume X reveals what was admissible without introducing new content, the quantization kernel reveals which classical observables survive as quantum operators without introducing new physics.

We state this as a formal postulate, extending the observability postulate of Volume X:

*Postulate (Quantization as Kernel).* The passage from classical to quantum mechanics is a structure-preserving kernel $Q$ whose image is the quantum algebra and whose null space is the classical excess. The three invariants --- bracket preservation on an admissible subalgebra, non-trivial null space, and image/null decomposition --- are universal across all known quantization schemes.

This completes the POT program through eleven volumes:

- *Volumes I--VI*: Physical laws are properties of admissible kernels.
- *Volume VII*: Renormalization is a projection kernel.
- *Volume VIII*: Conditional reduction of the Yang--Mills mass gap via ITCM.
- *Volume IX*: Yang--Mills vacuum stability as a classical spectral property.
- *Volume X*: All infinities are projection singularities (Hadamard projection).
- *Volume XI*: Quantum mechanics is the image of a quantization kernel.

The chain from Volume I to Volume XI traces a single structural principle: the observable universe is the image of projection kernels. What lies outside the image --- divergences, singularities, ordering ambiguities, classical determinism --- is not 'subtracted,' 'resolved,' or 'lost.' It was never in the image. It is the null space.

All 26 structural results (19 Z3-verified axioms + 7 computational verifications) are machine-verified in the Kleis formal verification language.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[Dir1925\] Dirac, P. A. M. The fundamental equations of quantum mechanics. *Proc. R. Soc. Lond. A* 109, 642--653 (1925).]

#par(hanging-indent: 1.5em)[\[Gro1946\] Groenewold, H. J. On the principles of elementary quantum mechanics. *Physica* 12, 405--460 (1946).]

#par(hanging-indent: 1.5em)[\[vH1951\] van Hove, L. Sur certaines représentations unitaires d'un groupe infini de transformations. *Mém. Acad. Roy. Belg.* 26, 1--102 (1951).]

#par(hanging-indent: 1.5em)[\[Kos1970\] Kostant, B. Quantization and unitary representations. *Lectures in Modern Analysis and Applications III*, 87--208. Springer (1970).]

#par(hanging-indent: 1.5em)[\[Sou1970\] Souriau, J.-M. *Structure des systèmes dynamiques.* Dunod, Paris (1970). English translation: *Structure of Dynamical Systems*, Birkhäuser (1997).]

#par(hanging-indent: 1.5em)[\[Ber1975\] Berezin, F. A. Quantization. *Math. USSR Izvestija* 8, 1109--1165 (1975).]

#par(hanging-indent: 1.5em)[\[BMS1994\] Bordemann, M., Meinrenken, E., and Schlichenmaier, M. Toeplitz quantization of Kähler manifolds and $"gl"(N)$, $N arrow infinity$ limits. *Commun. Math. Phys.* 165, 281--296 (1994).]

#par(hanging-indent: 1.5em)[\[BFFLS1978\] Bayen, F., Flato, M., Frønsdal, C., Lichnerowicz, A., and Sternheimer, D. Deformation theory and quantization. I & II. *Ann. Phys.* 111, 61--110 & 111--151 (1978).]

#par(hanging-indent: 1.5em)[\[Kon1997\] Kontsevich, M. Deformation quantization of Poisson manifolds. *Lett. Math. Phys.* 66, 157--216 (2003). Preprint q-alg/9709040 (1997).]

#par(hanging-indent: 1.5em)[\[BRS1976\] Becchi, C., Rouet, A., and Stora, R. Renormalization of gauge theories. *Ann. Phys.* 98, 287--321 (1976).]

#par(hanging-indent: 1.5em)[\[Tyu1975\] Tyutin, I. V. Gauge invariance in field theory and statistical physics in operator formalism. *Lebedev preprint* FIAN-39 (1975).]

#par(hanging-indent: 1.5em)[\[Wey1927\] Weyl, H. Quantenmechanik und Gruppentheorie. *Z. Phys.* 46, 1--46 (1927).]

#par(hanging-indent: 1.5em)[\[Wig1932\] Wigner, E. P. On the quantum correction for thermodynamic equilibrium. *Phys. Rev.* 40, 749--759 (1932).]

#par(hanging-indent: 1.5em)[\[Moy1949\] Moyal, J. E. Quantum mechanics as a statistical theory. *Math. Proc. Camb. Philos. Soc.* 45, 99--124 (1949).]

#par(hanging-indent: 1.5em)[\[FP1967\] Faddeev, L. D. and Popov, V. N. Feynman diagrams for the Yang--Mills field. *Phys. Lett. B* 25, 29--30 (1967).]

#par(hanging-indent: 1.5em)[\[Atik2025\] Atik, E. Projected ontology: physical laws as properties of admissible kernels. Volumes I--VI of the POT VUFT Series. *Preprint* (2025--2026).]

#par(hanging-indent: 1.5em)[\[Atik2026a\] Atik, E. Renormalization as projected ontology: the theory that was never divergent. Volume VII of the POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[Atik2026d\] Atik, E. Projection singularities: why physics has no infinities. Volume X of the POT VUFT Series. *Preprint* (2026).]


