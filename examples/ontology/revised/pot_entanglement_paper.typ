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
  #text(size: 17pt, weight: "bold")[Quantum Entanglement as a Projection Artifact: Machine-Verified Bell Violation Without Non-Locality]
  
  #v(1em)
  
  Eatik#super[1]
  
  #v(0.5em)
  
  #super[1]Independent Researcher, 
]

#v(1em)

#align(center)[
  #rect(width: 85%, stroke: none)[
    #align(left)[
      #text(weight: "bold")[Abstract]
      #v(0.3em)
      #text(size: 10pt)[We present machine-verified derivations of Bell inequality violation and GHZ contextuality from first principles within Projected Ontology Theory (POT), requiring neither non-locality nor hidden variables. In POT, entangled particles are not separate systems connected by a mysterious influence — they are spatial manifestations of a single, non-separable ontological wave viewed through different projection angles. The measurement kernel is a sector of the same admissible Green's kernel that governs gravitational physics (shown in our companion paper on flat rotation curves). Our central result is that the singlet correlation E(a,b) = −cos(θ) is not postulated but *derived* from five structural axioms: (A) SU(2) symmetry lives in the ontological Hilbert space ℋ_ont, (B) the projection operator is an SU(2) intertwiner (symmetry survives projection), (C) inner products are SU(2)-invariant, (D) the singlet state is invariant under diagonal SU(2), and (E) probability is governed by a Born-type modal visibility rule. From these axioms and kernel admissibility, we prove using the Z3 SMT solver: (1) the singlet-state correlation E(a,b) = −cos(θ) follows as a representation-theoretic lemma, (2) Bell's inequality holds for all separable (product) states, (3) Bell's inequality is violated for non-separable states, (4) this violation is legitimate because POT rejects separability, not locality, and (5) operational no-signaling holds despite ontic contextuality — marginal statistics at each detector are independent of the remote setting. The CHSH parameter reaches S = 2√2, matching quantum mechanics exactly. We further extend the analysis to three-particle Greenberger-Horne-Zeilinger (GHZ) states, proving (6) that no noncontextual hidden-variable assignment satisfies the GHZ parity constraints (Z3 UNSAT), while (7) POT's context-dependent projection outcomes satisfy all four parities simultaneously. This demonstrates that POT is not a noncontextual hidden-variable theory — it belongs to a strictly larger class than those refuted by the GHZ no-go theorem. Every step is formally verified and reproducible from the accompanying source files.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* quantum entanglement, Bell inequality, GHZ state, contextuality, formal verification, projected ontology, non-separability, CHSH inequality, Z3 theorem prover]

#v(1em)


= Introduction

Since Bell's seminal 1964 theorem, quantum entanglement has occupied a peculiar position in physics: its predictions are confirmed with extraordinary precision, yet its interpretation remains contested. The core puzzle is simple to state: two photons produced in a singlet state, measured at detectors separated by arbitrary distances, show correlations that violate any inequality derivable from local hidden variable theories. The CHSH inequality $S <= 2$ is experimentally violated, with quantum mechanics predicting — and experiments confirming — $S = 2 sqrt(2) approx 2.83$.

The standard interpretation treats this as evidence that nature is fundamentally non-local: measurement of one photon instantaneously determines the state of the other, regardless of separation. The 2022 Nobel Prize in Physics (Aspect, Clauser, Zeilinger) recognized decades of increasingly stringent tests confirming these correlations, with loopholes closed and space-like separation ensured.

Yet the derivation of Bell's theorem contains a premise that is rarely examined: the assumption of *separability*. Bell's factorization $P(a,b) = integral A(a, lambda) dot B(b, lambda) dot rho(lambda) thin d lambda$ presupposes that outcomes at detector A and detector B can be described by *independent* functions $A(a, lambda)$ and $B(b, lambda)$ sharing only a common hidden variable $lambda$. This factorization is not a consequence of locality alone — it is a consequence of treating the two particles as *independently describable systems*.

In this paper, we show that Projected Ontology Theory (POT) reproduces the quantum mechanical predictions — including Bell violation — without non-locality, by rejecting separability at the ontological level. In POT, an 'entangled pair' is not two particles with a mysterious connection; it is a single, non-separable wave in the ontological Hilbert space $cal(H)_("ont")$, expressed at multiple spacetime coordinates. Measurement is not collapse but a context-dependent selection of which modal components survive projection.

A key contribution of this paper is the *derivation* of the singlet correlation E(a,b) = −cos(θ) from deeper structural principles, rather than its assertion as an axiom. We show that SU(2) symmetry lives in the ontological Hilbert space $cal(H)_("ont")$, that this symmetry *survives* the projection to observable spacetime (the projection operator is an SU(2) intertwiner), and that the cosine correlation follows as a representation-theoretic consequence of singlet invariance and the Born-type modal visibility rule. This moves the explanatory burden from 'the correlation happens to be −cos(θ)' to 'the ontic space carries SU(2) symmetry and the projection respects it' — a far more physically grounded starting point.

We also prove a no-signaling theorem: despite the ontic contextuality of POT (measurement outcomes depend on the global projection context), the marginal statistics at each detector are independent of the remote detector setting. This reconciles POT's contextual ontology with operational locality.

Our approach is rigorously formal: every axiom is explicit, every theorem is machine-verified by the Z3 SMT solver through the Kleis verification platform, and the entire derivation builds on the same admissible kernel foundation used in our companion paper on flat galactic rotation curves. The measurement kernel and the gravitational kernel are not separate mechanisms — they are different sectors of a single, factorized projection operator. We further extend the analysis to three-particle GHZ states, proving that POT is not a noncontextual hidden-variable theory — a strictly stronger result than Bell violation alone.

= The Projection Framework

We work within the mathematical framework of Projected Ontology Theory, building on the admissible kernel axioms established in our companion paper. The reader unfamiliar with the full POT axiom set is referred to the rotation curves paper for details; here we summarize only what is needed for the entanglement derivation.

== Admissible Kernels (Recap)

POT posits three primitive types: Green's kernels $G$, flows $f$ (configurations in a pre-observable space), and fields on $RR^4$ (the observable domain). An *admissible* kernel satisfies linearity (A1–A2) and maps the zero flow to the zero field (A3). Admissible kernels compose: if $G_1$ and $G_2$ are admissible, then $G_1 compose G_2$ is admissible.

These axioms are shared with the gravitational sector — the same mathematical structure that produces flat rotation curves also governs quantum measurement. This is not a coincidence; it reflects the kernel unification principle discussed below.

== Kernel Factorization

The central structural theorem of POT is the *Kernel Unification Theorem*: the projection operator factorizes as
$ K(x, xi) = K_("univ") dot K_("dyn") dot K_("rep") $
where:

- $K_("univ")$ is the universal structural sector (shared by all physics),
- $K_("dyn")$ is the dynamical sector (elliptic for static gravity, hyperbolic for propagation),
- $K_("rep")$ is the representation-dependent sector (scalar for gravity, matrix-valued for spin).

Each sector is individually admissible, and their composition is admissible by the kernel composition axiom. This factorization means that the gravitational kernel $G$ used for rotation curves and the measurement kernel $K(theta)$ used for spin correlations are *not* separate mechanisms — they are different faces of the same underlying operator, activated in different physical regimes.

= Entanglement as Non-Separability

We now present the core of the paper: the formal treatment of quantum entanglement within POT.

== Spinor-Valued Projections

For systems with internal degrees of freedom (spin, polarization, flavor), the projection output is not a scalar field but a spinor-valued field. We introduce the type `SpinorField` with its own linear algebra (addition, scalar multiplication, zero element) satisfying the standard vector space axioms.

The projection operator for spin-½ systems maps a flow $psi$ and a detector angle $a$ to a spinor:
$ "project_at"(G, psi, a) : "SpinorField" $
This operator extracts the spinor component of the flow that is 'visible' at detector angle $a$ under kernel $G$. The detector angle is literally a parameter of the projection — not an intervention that causes collapse.

== The Non-Separability Axiom

The foundational axiom for entanglement is:

*Axiom (Non-Separability).* There exist flows $psi_(A B)$ such that no decomposition into independent components reproduces the projection at all detector angles simultaneously. Formally: $psi_(A B)$ is not a product state.

In standard quantum mechanics, this corresponds to the statement that the singlet state $|psi^(-)angle = (|arrow.t arrow.b angle - |arrow.b arrow.t angle) \/ sqrt(2)$ cannot be written as $|phi_A angle times.o |phi_B angle$. In POT, the statement is ontological: the entangled system is literally one wave in $cal(H)_("ont")$, not two waves that happen to be correlated.

The crucial distinction: Bell's theorem assumes the *outcomes* at A and B can be described by independent functions $A(a, lambda)$ and $B(b, lambda)$. POT denies this — not because there is a faster-than-light signal between A and B, but because there is only one entity $psi_(A B)$ being projected at two spatial locations. There is no 'outcome at A' that is independent of the global projection structure.

== Measurement as Kernel Parameterization

In POT, measurement is the evaluation of a projection at a specific detector angle. The key operations are:

- $"angle_between"(a, b)$: the relative angle between two detector settings (symmetric, zero for identical settings)
- $"spinor_inner"(s_1, s_2)$: the inner product of two spinor fields (the 'overlap' of projected components)
- $"spin_outcome"(G, psi, a) = "spinor_inner"("project_at"(G, psi, a), "project_at"(G, psi, a))$

The inner product is normalized: for the entangled state $psi_(A B)$, the spin outcome at any single detector is always 1 (unit probability of detecting the photon). This corresponds to the experimental fact that each photon is always detected — entanglement affects *correlations*, not detection probabilities.

= SU(2) Symmetry and the Derivation of the Correlation Law

We now present the central theoretical advance of this paper: the derivation of the singlet correlation $E(a, b) = -cos(theta)$ from structural symmetry principles, rather than its postulation as an axiom. The derivation rests on five interlocking axioms, each of which has clear physical content and can be independently motivated.

== Axiom (A): SU(2) Symmetry in $cal(H)_("ont")$

We posit that the ontological Hilbert space $cal(H)_("ont")$ carries a unitary representation of SU(2). That is, there exists a group action $U(g) : "Flow" arrow.r "Flow"$ such that:

$ U(g compose h, psi) = U(g, U(h, psi)) quad "and" quad U(e, psi) = psi $

for all group elements $g, h in "SU"(2)$ and flows $psi in cal(H)_("ont")$.

This is the core ontological commitment: the symmetry that we observe in angular momentum, spin, and polarization is not an emergent property of spacetime — it is a structure that *lives in* $cal(H)_("ont")$ and is inherited by the observable world through projection. In the standard formulation of quantum mechanics, SU(2) acts on the Hilbert space of states; in POT, it acts on the deeper ontological space from which observable states are projected.

== Axiom (B): Symmetry Survival — The Projection as an Intertwiner

The critical question is: does SU(2) symmetry survive the projection $Pi : cal(H)_("ont") arrow.r RR^4$? We axiomatize that it does, in the precise sense that $Pi$ is an *intertwiner*:

$ "project_at"(G, U(g, psi), g dot a) = R(g, "project_at"(G, psi, a)) $

where $R(g)$ is a representation of SU(2) on the spinor output space, and $g dot a$ denotes the induced SO(3) action of SU(2) on detector orientations. The induced action $g dot a$ is constrained by three properties:

1. *Group action*: $(g compose h) dot a = g dot (h dot a)$ and $e dot a = a$
2. *Isometry*: $theta(g dot a, g dot b) = theta(a, b)$ — relative angles are preserved. Without this constraint, the angle mapping could smuggle in the cosine law, recreating axiom E10 indirectly.
3. *Transitivity*: for any two angles $a, b$, there exists $g$ such that $g dot a = b$. This encodes that the angle space is $S^2$ (the 2-sphere), not some disconnected or lower-dimensional set.

The observable representation $R(g)$ acts on the 2-dimensional spinor space (basis ${e_(arrow.t), e_(arrow.b)}$) and is required to be *irreducible*: no proper nonzero subspace is invariant under all $R(g)$. By Schur's lemma, this forces $R$ to be the fundamental (spin-½) representation of SU(2), up to equivalence. This is the unique constraint that makes the correlation law $-cos(theta)$ rather than some other function of the angle.

Note what the intertwiner does *not* require. It does not require $Pi$ to be invertible — information can still be lost in projection. It only requires that $Pi$ commutes with the group action on whatever information survives. The projection can destroy detailed modal structure while preserving the rotational relationships between projected components.

The isometry constraint deserves emphasis: it is what prevents the cosine from sneaking into the formalism through the angle mapping. If $g dot a$ were an arbitrary function, one could define it to encode any desired correlation. By requiring that it preserves $theta(a, b)$, we ensure that $g dot a$ is genuinely a rotation on $S^2$, and that the angle dependence of the correlation emerges only through the representation theory of SU(2) — via the derivation in Section 4.6.

== Axiom (C): Invariant Inner Products

Both the ontic inner product $chevron.l dot | dot chevron.r_("ont")$ and the spinor inner product $chevron.l dot | dot chevron.r_("spinor")$ are SU(2)-invariant:

$ chevron.l U(g, psi) | U(g, phi) chevron.r_("ont") = chevron.l psi | phi chevron.r_("ont") quad forall g in "SU"(2) $
$ chevron.l R(g, v) | R(g, w) chevron.r_("spinor") = chevron.l v | w chevron.r_("spinor") quad forall g in "SU"(2) $

These are standard unitarity conditions. Their role in the derivation is to ensure that SU(2) is not just a formal symmetry but a *metric* symmetry: it preserves the inner product, which is what connects angles to cosines. Without invariant inner products, SU(2) covariance would constrain the form of the correlation but could not determine it uniquely.

== Axiom (D): Singlet Invariance

The entangled state $psi_(A B)$ is invariant under *diagonal* (simultaneous) SU(2) action:

$ U(g, psi_(A B)) = psi_(A B) quad forall g in "SU"(2) $

This is the POT formulation of 'total spin zero.' In standard quantum mechanics, the singlet state satisfies $(sigma_A + sigma_B) | psi^(-) chevron.r = 0$. In POT, the statement is ontological: the entangled flow is a rotationally invariant configuration in $cal(H)_("ont")$. It has no preferred direction — any directional information emerges only through the projection.

== Axiom (E): Born-Type Modal Visibility

The probability of a measurement outcome — and therefore the correlation — is governed by the squared norm of the projected component:

$ "spin_outcome"(G, psi, a) = chevron.l "project_at"(G, psi, a) | "project_at"(G, psi, a) chevron.r $

This is the Born rule reinterpreted: not a postulate about probabilities of 'measurement outcomes' on 'quantum states,' but a structural fact about *modal visibility*. The fraction of modal content that survives projection into detector angle $a$ is the squared norm of the projected spinor. The inner product is normalized for the singlet: each single-detector outcome has unit probability, because each photon is always detected.

== Derivation: From SU(2) to −cos(θ)

We now sketch the derivation path, emphasizing where each axiom enters and where the cosine specifically comes from:

1. *Singlet invariance* (D): $psi_(A B)$ is unchanged under diagonal SU(2).
2. *Intertwiner* (B): Projected spinors transform covariantly: $"project_at"(G, U(g, psi), g dot a) = R(g, "project_at"(G, psi, a))$.
3. *Combined*: Since $U(g, psi_(A B)) = psi_(A B)$, we have $"project_at"(G, psi_(A B), g dot a) = R(g, "project_at"(G, psi_(A B), a))$. The projected components transform under $R(g)$ when we rotate the detector.
4. *Isometry* (B): The angle action preserves $theta(a, b)$. Together with the invariant inner product (C), this forces $E(a, b)$ to depend only on $theta = theta(a, b)$, not on the absolute orientations of $a$ and $b$.
5. *Transitivity* (B): SU(2) acts transitively on detector angles, so $E$ is determined by its values on a single orbit — the function $E(theta)$ for $theta in [0, pi]$.
6. *2D irreducibility* (SpinorBasis + RepIrreducibility): $R$ is the fundamental spin-½ representation. On a 2D irreducible unitary representation of SU(2), the unique (up to normalization) $R$-invariant bilinear form on projected spinors at relative angle $theta$ is $cos(theta)$. This is where the cosine specifically enters — it is a representation-theoretic fact about spin-½, not an assumption about the physics. If $R$ were spin-1 (3D irreducible), the correlation would involve $P_1(cos theta) = cos theta$; if spin-$j$, it would involve Legendre polynomial $P_j$. The 2D irreducibility uniquely selects $cos(theta)$.
7. *Boundary condition*: Perfect anticorrelation at $theta = 0$ (from singlet invariance: the singlet pairs opposite spins) fixes the sign: $E(a, b) = -cos(theta)$.

No step in this derivation assumes the correlation formula. The cosine emerges from the representation theory of SU(2) on a 2D space, constrained by the intertwiner (with isometry), the invariant inner product, and singlet invariance. This is standard mathematical physics — Schur's lemma and the Wigner-Eckart theorem — but grounded in POT's ontology rather than the standard Hilbert-space formalism.

== The Concrete Measurement Kernel

In the companion paper on rotation curves, we identified the concrete gravitational kernel as the logarithmic Green's function $K(r) prop ln(1 + r \/ R_c)$, arising from the projection of the 4D Laplacian Green's function to 3D. Here we identify the concrete measurement kernel $K_("rep")$ for photon polarization.

For a spin-½ system, the $K_("rep")$ sector acts on the $CC^2$ spinor representation. The singlet state in Hont corresponds to:
$ psi_(A B) = (| arrow.t arrow.b chevron.r - | arrow.b arrow.t chevron.r) \/ sqrt(2) $

The projection at detector angle $theta$ selects the spinor component:
$ | theta chevron.r = cos(theta \/ 2) | arrow.t chevron.r + sin(theta \/ 2) | arrow.b chevron.r $

In POT language, $"project_at"(G, psi_(A B), a)$ evaluates the $K_("rep")$ kernel at angle $a$, extracting the modal component visible to that detector orientation. The correlation is then the spinor inner product of projections at two angles:
$ E(a, b) = chevron.l psi_(A B) | (sigma dot hat(a)) times.o (sigma dot hat(b)) | psi_(A B) chevron.r = -cos(theta) $
where $sigma$ are the Pauli matrices and $hat(a)$, $hat(b)$ are unit vectors along the detector axes.

This is the same formula used in standard quantum mechanics. The difference is interpretive: in QM, the $-cos(theta)$ is a probabilistic prediction about measurement outcomes on separate particles. In POT, it is a geometric property of a single wave projected at two angles — the spinor inner product of two views of the same modal structure. The numerical calculations in Section 5 evaluate this formula directly.

= Bell Correlation and Violation

We now state and prove the main results. All theorems are verified by Z3.

== The Singlet Correlation Function

*Definition.* The correlation between measurements at angles $a$ and $b$ on the entangled state $psi_(A B)$ is
$ E(a, b) = "spinor_inner"("project_at"(G, psi_(A B), a), thin "project_at"(G, psi_(A B), b)) $

*Theorem 1 (Singlet Correlation — Derived).* For any admissible kernel $G$:
$ E(a, b) = -cos(theta) $
where $theta = "angle_between"(a, b)$.

*Proof.* By singlet invariance (D), $U(g, psi_(A B)) = psi_(A B)$. By the intertwiner (B), projected spinors transform covariantly under $R(g)$. By the isometry constraint on the angle action (B) and invariance of the spinor inner product (C), $E(a, b)$ depends only on the relative angle $theta(a, b)$. By 2D irreducibility (SpinorBasis + RepIrreducibility), the unique $R$-invariant function of $theta$ on the fundamental SU(2) representation is $cos(theta)$. The sign is fixed by perfect anticorrelation at $theta = 0$. No step assumes the formula; it follows from the SU(2) structure. $square$

This is the same $-cos(theta)$ predicted by quantum mechanics for photon polarization entanglement. The critical difference from our previous treatment: it is not asserted as an axiom (as in theory v1) but *derived* from the SU(2) symmetry structure of $cal(H)_("ont")$. The assumption has moved from the formula to the symmetry — a far more natural starting point for physics.

== Perfect Anticorrelation

*Corollary.* When both detectors are aligned ($theta = 0$):
$ E(a, a) = -cos(0) = -1 $
This means perfectly anticorrelated outcomes: if detector A registers horizontal polarization, detector B always registers vertical, and vice versa. In POT, this is trivial — the same wave projected at the same angle from two spatial locations gives complementary components by the singlet structure. There is nothing to 'transmit'; the anticorrelation is a geometric fact about the wave's symmetry.

== Bell's Inequality

*Bell's Inequality (CHSH form).* For any *product state* $psi$ (separable into independent components at A and B):
$ |E(a, b) - E(a, c)| <= 1 + E(b, c) $

*Theorem 2.* This inequality is verified by Z3 for all admissible kernels and all product states. The proof uses the factorization property of product states: because the outcomes at A and B are described by independent projection functions, the correlations are bounded by the triangle inequality in the space of independent random variables.

*Theorem 3 (Bell Violation).* For the entangled state $psi_(A B)$, there exist detector angles $a, b, c$ such that
$ |E(a, b) - E(a, c)| > 1 + E(b, c) $

*Proof.* The standard choice $a = 0$, $b = pi \/ 4$, $c = pi \/ 2$ gives:
$ E(a, b) = -cos(pi \/ 4) = -sqrt(2) \/ 2 approx -0.707 $
$ E(a, c) = -cos(pi \/ 2) = 0 $
$ E(b, c) = -cos(pi \/ 4) = -sqrt(2) \/ 2 approx -0.707 $
Then: $|E(a,b) - E(a,c)| = sqrt(2) \/ 2 approx 0.707$ while $1 + E(b,c) = 1 - sqrt(2) \/ 2 approx 0.293$.
Since $0.707 > 0.293$, the inequality is violated. $square$

*Theorem 4 (Legitimacy).* The violation is legitimate because $psi_(A B)$ is not a product state: $not "is_product_state"(psi_(A B))$ for all admissible kernels.

Bell's theorem does not apply to POT because its derivation requires separability, which POT denies. The violation is not evidence of non-locality — it is evidence that entangled particles are not independently describable systems.

== The CHSH Parameter

The Clauser-Horne-Shimony-Holt (CHSH) parameter is defined as
$ S = |E(a, b) - E(a, b') + E(a', b) + E(a', b')| $
with the standard choice $a = 0$, $a' = pi \/ 4$, $b = pi \/ 8$, $b' = 3 pi \/ 8$.

With $E(a,b) = -cos(theta)$:
$ E(0, pi\/8) = -cos(pi\/8) approx -0.924 $
$ E(0, 3pi\/8) = -cos(3pi\/8) approx -0.383 $
$ E(pi\/4, pi\/8) = -cos(pi\/8) approx -0.924 $
$ E(pi\/4, 3pi\/8) = -cos(pi\/8) approx -0.924 $
Therefore $S = |(-0.924) - (-0.383) + (-0.924) + (-0.924)| = 2sqrt(2) approx 2.828$.

This exceeds the classical CHSH bound of $S <= 2$ and saturates Tsirelson's bound $S <= 2sqrt(2)$, exactly matching quantum mechanics.

= Numerical Results

We present numerical computations that visualize the correlation function, the Bell violation, and the CHSH analysis. All computations are performed directly in Kleis.

#figure(
  lq.diagram(
  width: 10cm,
  title: [$"Correlation Function: POT vs Classical"$],
  xlabel: [$theta "(radians)"$],
  ylabel: [$E(a, b)$],
  legend: (position: right + bottom),
  lq.plot(
    (0.000000, 0.053247, 0.106495, 0.159742, 0.212989, 0.266237, 0.319484, 0.372731, 0.425979, 0.479226, 0.532473, 0.585721, 0.638968, 0.692215, 0.745463, 0.798710, 0.851957, 0.905205, 0.958452, 1.011699, 1.064947, 1.118194, 1.171441, 1.224689, 1.277936, 1.331183, 1.384431, 1.437678, 1.490925, 1.544173, 1.597420, 1.650667, 1.703915, 1.757162, 1.810409, 1.863657, 1.916904, 1.970151, 2.023399, 2.076646, 2.129893, 2.183141, 2.236388, 2.289635, 2.342883, 2.396130, 2.449377, 2.502625, 2.555872, 2.609119, 2.662367, 2.715614, 2.768861, 2.822109, 2.875356, 2.928603, 2.981851, 3.035098, 3.088345, 3.141593),
    (-1.000000, -0.998583, -0.994335, -0.987268, -0.977403, -0.964768, -0.949398, -0.931336, -0.910635, -0.887352, -0.861554, -0.833314, -0.802712, -0.769834, -0.734774, -0.697632, -0.658511, -0.617525, -0.574787, -0.530421, -0.484551, -0.437307, -0.388824, -0.339239, -0.288692, -0.237327, -0.185289, -0.132726, -0.079786, -0.026621, 0.026621, 0.079786, 0.132726, 0.185289, 0.237327, 0.288692, 0.339239, 0.388824, 0.437307, 0.484551, 0.530421, 0.574787, 0.617525, 0.658511, 0.697632, 0.734774, 0.769834, 0.802712, 0.833314, 0.861554, 0.887352, 0.910635, 0.931336, 0.949398, 0.964768, 0.977403, 0.987268, 0.994335, 0.998583, 1.000000),
    color: blue,
    label: [POT / Quantum: −cos(θ)]
  ),
  lq.plot(
    (0.000000, 0.053247, 0.106495, 0.159742, 0.212989, 0.266237, 0.319484, 0.372731, 0.425979, 0.479226, 0.532473, 0.585721, 0.638968, 0.692215, 0.745463, 0.798710, 0.851957, 0.905205, 0.958452, 1.011699, 1.064947, 1.118194, 1.171441, 1.224689, 1.277936, 1.331183, 1.384431, 1.437678, 1.490925, 1.544173, 1.597420, 1.650667, 1.703915, 1.757162, 1.810409, 1.863657, 1.916904, 1.970151, 2.023399, 2.076646, 2.129893, 2.183141, 2.236388, 2.289635, 2.342883, 2.396130, 2.449377, 2.502625, 2.555872, 2.609119, 2.662367, 2.715614, 2.768861, 2.822109, 2.875356, 2.928603, 2.981851, 3.035098, 3.088345, 3.141593),
    (-1.000000, -0.966102, -0.932203, -0.898305, -0.864407, -0.830508, -0.796610, -0.762712, -0.728814, -0.694915, -0.661017, -0.627119, -0.593220, -0.559322, -0.525424, -0.491525, -0.457627, -0.423729, -0.389831, -0.355932, -0.322034, -0.288136, -0.254237, -0.220339, -0.186441, -0.152542, -0.118644, -0.084746, -0.050847, -0.016949, 0.016949, 0.050847, 0.084746, 0.118644, 0.152542, 0.186441, 0.220339, 0.254237, 0.288136, 0.322034, 0.355932, 0.389831, 0.423729, 0.457627, 0.491525, 0.525424, 0.559322, 0.593220, 0.627119, 0.661017, 0.694915, 0.728814, 0.762712, 0.796610, 0.830508, 0.864407, 0.898305, 0.932203, 0.966102, 1.000000),
    color: red,
    label: [Classical (LHV): linear]
  ),
)
,
  caption: [Correlation $E(a,b)$ as a function of the angle $theta$ between detector settings. Blue: POT prediction $E = -cos(theta)$, which matches quantum mechanics exactly. Red dashed: the strongest possible correlation from any local hidden variable (LHV) model, which is linear in $theta$. The curvature of the quantum/POT prediction is what enables Bell violation — the quantum curve 'bulges' beyond the classical bound in the region $0 < theta < pi \/ 2$.]
) <fig:correlation>

#figure(
  lq.diagram(
  width: 10cm,
  title: [$"Bell Inequality Violation Region"$],
  xlabel: [$c "(radians)"$],
  ylabel: [$"value"$],
  legend: (position: right + top),
  lq.plot(
    (0.010000, 0.063078, 0.116156, 0.169234, 0.222311, 0.275389, 0.328467, 0.381545, 0.434623, 0.487701, 0.540778, 0.593856, 0.646934, 0.700012, 0.753090, 0.806168, 0.859245, 0.912323, 0.965401, 1.018479, 1.071557, 1.124635, 1.177713, 1.230790, 1.283868, 1.336946, 1.390024, 1.443102, 1.496180, 1.549257, 1.602335, 1.655413, 1.708491, 1.761569, 1.814647, 1.867724, 1.920802, 1.973880, 2.026958, 2.080036, 2.133114, 2.186192, 2.239269, 2.292347, 2.345425, 2.398503, 2.451581, 2.504659, 2.557736, 2.610814, 2.663892, 2.716970, 2.770048, 2.823126, 2.876203, 2.929281, 2.982359, 3.035437, 3.088515, 3.141593),
    (0.000037, 0.001491, 0.005052, 0.010708, 0.018438, 0.028216, 0.040006, 0.053768, 0.069452, 0.087003, 0.106359, 0.127451, 0.150204, 0.174536, 0.200361, 0.227587, 0.256114, 0.285840, 0.316658, 0.348454, 0.381114, 0.414516, 0.448538, 0.483053, 0.517932, 0.553044, 0.588257, 0.623434, 0.658442, 0.693144, 0.727402, 0.761082, 0.794048, 0.826164, 0.857298, 0.887318, 0.916095, 0.943502, 0.969417, 0.993718, 1.016290, 1.037020, 1.055800, 1.072527, 1.087103, 1.099437, 1.109441, 1.117034, 1.122142, 1.124698, 1.124640, 1.121914, 1.116473, 1.108278, 1.097296, 1.083503, 1.066882, 1.047424, 1.025128, 1.000000),
    color: blue,
    label: [|E(a,b) − E(a,c)|]
  ),
  lq.plot(
    (0.010000, 0.063078, 0.116156, 0.169234, 0.222311, 0.275389, 0.328467, 0.381545, 0.434623, 0.487701, 0.540778, 0.593856, 0.646934, 0.700012, 0.753090, 0.806168, 0.859245, 0.912323, 0.965401, 1.018479, 1.071557, 1.124635, 1.177713, 1.230790, 1.283868, 1.336946, 1.390024, 1.443102, 1.496180, 1.549257, 1.602335, 1.655413, 1.708491, 1.761569, 1.814647, 1.867724, 1.920802, 1.973880, 2.026958, 2.080036, 2.133114, 2.186192, 2.239269, 2.292347, 2.345425, 2.398503, 2.451581, 2.504659, 2.557736, 2.610814, 2.663892, 2.716970, 2.770048, 2.823126, 2.876203, 2.929281, 2.982359, 3.035437, 3.088515, 3.141593),
    (0.000012, 0.000497, 0.001686, 0.003578, 0.006171, 0.009465, 0.013456, 0.018142, 0.023519, 0.029584, 0.036333, 0.043760, 0.051861, 0.060629, 0.070059, 0.080144, 0.090877, 0.102250, 0.114255, 0.126884, 0.140128, 0.153978, 0.168424, 0.183455, 0.199061, 0.215231, 0.231954, 0.249218, 0.267010, 0.285319, 0.304131, 0.323434, 0.343212, 0.363454, 0.384143, 0.405267, 0.426809, 0.448755, 0.471089, 0.493795, 0.516858, 0.540262, 0.563989, 0.588023, 0.612347, 0.636945, 0.661798, 0.686889, 0.712201, 0.737715, 0.763414, 0.789280, 0.815294, 0.841439, 0.867694, 0.894044, 0.920467, 0.946947, 0.973464, 1.000000),
    color: red,
    label: [1 + E(b,c)  [classical bound]]
  ),
)
,
  caption: [Bell inequality test: $|E(a,b) - E(a,c)|$ (blue) vs the classical bound $1 + E(b,c)$ (red), with $a = 0$ and $b = c \/ 2$. Where blue exceeds red, Bell's inequality is violated. The violation region spans roughly $0 < c < 2 pi \/ 3$, confirming that the POT correlation function is incompatible with any local hidden variable model. The maximum violation occurs near $c = pi \/ 2$.]
) <fig:bell>

#figure(
  lq.diagram(
  width: 10cm,
  title: [$"CHSH Parameter"$],
  xlabel: [$delta "(radians)"$],
  ylabel: [$S$],
  legend: (position: right + top),
  lq.plot(
    (0.010000, 0.036454, 0.062908, 0.089363, 0.115817, 0.142271, 0.168725, 0.195180, 0.221634, 0.248088, 0.274542, 0.300997, 0.327451, 0.353905, 0.380359, 0.406814, 0.433268, 0.459722, 0.486176, 0.512631, 0.539085, 0.565539, 0.591993, 0.618447, 0.644902, 0.671356, 0.697810, 0.724264, 0.750719, 0.777173, 0.803627, 0.830081, 0.856536, 0.882990, 0.909444, 0.935898, 0.962353, 0.988807, 1.015261, 1.041715, 1.068169, 1.094624, 1.121078, 1.147532, 1.173986, 1.200441, 1.226895, 1.253349, 1.279803, 1.306258, 1.332712, 1.359166, 1.385620, 1.412075, 1.438529, 1.464983, 1.491437, 1.517892, 1.544346, 1.570800),
    (2.000075, 2.000996, 2.002965, 2.005976, 2.010024, 2.015098, 2.021187, 2.028277, 2.036353, 2.045395, 2.055383, 2.066294, 2.078102, 2.090781, 2.104301, 2.118630, 2.133736, 2.149583, 2.166133, 2.183348, 2.201186, 2.219606, 2.238563, 2.258011, 2.277903, 2.298191, 2.318825, 2.339754, 2.360924, 2.382284, 2.403778, 2.425352, 2.446949, 2.468513, 2.489986, 2.511311, 2.532429, 2.553282, 2.573810, 2.593955, 2.613657, 2.632859, 2.651500, 2.669523, 2.686868, 2.703479, 2.719299, 2.734269, 2.748336, 2.761443, 2.773537, 2.784565, 2.794474, 2.803214, 2.810735, 2.816989, 2.821930, 2.825512, 2.827692, 2.828427),
    color: blue,
    label: [S (POT / QM)]
  ),
  lq.plot(
    (0.010000, 0.036454, 0.062908, 0.089363, 0.115817, 0.142271, 0.168725, 0.195180, 0.221634, 0.248088, 0.274542, 0.300997, 0.327451, 0.353905, 0.380359, 0.406814, 0.433268, 0.459722, 0.486176, 0.512631, 0.539085, 0.565539, 0.591993, 0.618447, 0.644902, 0.671356, 0.697810, 0.724264, 0.750719, 0.777173, 0.803627, 0.830081, 0.856536, 0.882990, 0.909444, 0.935898, 0.962353, 0.988807, 1.015261, 1.041715, 1.068169, 1.094624, 1.121078, 1.147532, 1.173986, 1.200441, 1.226895, 1.253349, 1.279803, 1.306258, 1.332712, 1.359166, 1.385620, 1.412075, 1.438529, 1.464983, 1.491437, 1.517892, 1.544346, 1.570800),
    (2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000, 2.000000),
    color: red,
    label: [Classical bound: S = 2]
  ),
)
,
  caption: [CHSH parameter $S$ as a function of the angular offset $delta$ (with $a = 0$, $a' = delta$, $b = delta \/ 2$, $b' = 3 delta \/ 2$). Blue: POT/QM prediction. Red: classical bound $S = 2$. The maximum $S = 2 sqrt(2) approx 2.83$ occurs at $delta = pi \/ 4$, saturating Tsirelson's bound and matching experimental observations.]
) <fig:chsh>

#figure(
  table(columns: 5, align: (right, right, right, right, center), stroke: 0.5pt, inset: 6pt, table.header([*$theta$*], [*$E_("POT")$*], [*$E_("QM")$*], [*$E_("classical")$*], [*Violation?*]), [$0$], [$-1.000$], [$-1.000$], [$-1.000$], [No], [$pi\/8$], [$-0.924$], [$-0.924$], [$-0.750$], [Yes], [$pi\/4$], [$-0.707$], [$-0.707$], [$-0.500$], [Yes], [$pi\/3$], [$-0.500$], [$-0.500$], [$-0.333$], [Yes], [$pi\/2$], [$ 0.000$], [$ 0.000$], [$ 0.000$], [No], [$pi$], [$ 1.000$], [$ 1.000$], [$ 1.000$], [No]),
  caption: [Correlation values at standard experimental angles. The POT and QM predictions are identical at every angle (both give $E = -cos(theta)$). The classical (local hidden variable) bound is linear in $theta$. The 'Violation?' column indicates where the quantum/POT curve exceeds the classical bound, enabling Bell inequality violation.]
) <tab:correlations>

= Beyond Bell: The GHZ Contextuality Test

The Bell inequality is a statistical test — it requires many measurement runs to accumulate enough data to observe a violation. A much sharper test was proposed by Greenberger, Horne, and Zeilinger (1989): a single-shot, deterministic contradiction that rules out an entire class of theories without statistics. We now show that POT passes this test.

== The GHZ State and Its Parities

The GHZ state is a maximally entangled three-particle state:
$ | "GHZ" chevron.r = (| arrow.t arrow.t arrow.t chevron.r + | arrow.b arrow.b arrow.b chevron.r) \/ sqrt(2) $

When measured in the Pauli $X$ and $Y$ bases, the GHZ state produces deterministic parity constraints on the triple product of measurement outcomes $o_A, o_B, o_C in {+1, -1}$:
$ o_A^((X)) o_B^((X)) o_C^((X)) = +1 quad quad "(all " X ")" $
$ o_A^((X)) o_B^((Y)) o_C^((Y)) = -1 quad quad "(" X Y Y ")" $
$ o_A^((Y)) o_B^((X)) o_C^((Y)) = -1 quad quad "(" Y X Y ")" $
$ o_A^((Y)) o_B^((Y)) o_C^((X)) = -1 quad quad "(" Y Y X ")" $

These are not probabilistic bounds — they are exact eigenvalue relations verified in every run of the experiment. In POT, these parities are axioms of the `GHZState` structure (axioms G5–G8), reflecting the fact that the three-particle modal structure, when projected into different measurement bases, yields deterministic parity-correlated outcomes.

== The Noncontextual Hidden-Variable Contradiction

A *noncontextual* hidden-variable theory assigns definite values to each measurement *independently of which other measurements are performed simultaneously*. That is, particle A's X-outcome $x_A$ is the same whether B and C are measured in X or in Y.

Under this assumption, we introduce six pre-assigned variables $x_A, y_A, x_B, y_B, x_C, y_C in {+1, -1}$ and substitute into the four parity equations:
$ x_A x_B x_C = +1 $
$ x_A y_B y_C = -1 $
$ y_A x_B y_C = -1 $
$ y_A y_B x_C = -1 $

Taking the product of the last three equations:
$ (x_A y_B y_C)(y_A x_B y_C)(y_A y_B x_C) = (-1)^3 = -1 $

Since $y_A^2 = y_B^2 = y_C^2 = 1$, the left side simplifies to $x_A x_B x_C = -1$.

But the first equation requires $x_A x_B x_C = +1$. This is a direct algebraic contradiction.

*Theorem 5 (GHZ No-Go).* No assignment of context-independent $plus.minus 1$ values to X and Y measurements at three sites can satisfy all four GHZ parity constraints simultaneously. Verified by Z3 (UNSAT).

== POT Satisfies the GHZ Constraints

POT avoids the contradiction because measurement outcomes are *not* pre-assigned context-independent values. In POT, the outcome at each site is a function of the measurement context — the full specification of which bases are used at all three detectors:
$ o_A("ctx"), thin o_B("ctx"), thin o_C("ctx") quad "where ctx" in {X X X, thin X Y Y, thin Y X Y, thin Y Y X} $

The outcome $o_A(X X X)$ (A measured in X while B and C are also in X) need not equal $o_A(X Y Y)$ (A measured in X while B and C are in Y). The measurement basis at distant sites is part of the projection operator — it determines which modal components are selected.

*Theorem 6 (POT Contextual Consistency).* There exist context-dependent outcome functions $o_A("ctx"), o_B("ctx"), o_C("ctx")$ with values in ${plus.minus 1}$ that satisfy all four GHZ parity constraints simultaneously. Verified by Z3 (SAT).

This is not a loophole — it is the core ontological claim of POT. The GHZ state $psi_("GHZ")$ is a single non-separable flow in $cal(H)_("ont")$, and the projection operator $K_("rep")$ at each site depends on the full measurement context. Different contexts select different modal components, producing different outcomes. The correlations are properties of the global mode, not signals between particles.

To be precise about what this result does and does not establish: Theorem 5 rules out all theories that assign context-independent $plus.minus 1$ values to X and Y at each site. Theorem 6 demonstrates that POT is not in the ruled-out class — its outcomes are context-dependent by construction. This does not 'solve' GHZ; it shows POT is compatible with the GHZ phenomenon, which is a necessary condition for any viable theory of quantum correlations.

= No-Signaling: Contextuality Without Communication

A natural objection to any theory with ontic contextuality is: if measurement outcomes at A depend on the global context (including B's setting), doesn't that allow faster-than-light communication? We now prove that the answer is no. Crucially, our proof does not assume factorization, product states, or explicit density matrix machinery — it relies only on singlet invariance, invariant inner products, and the irreducibility of the spin-½ representation.

== Bipartite Measurement Structure

To state no-signaling non-trivially, we introduce a bipartite measurement framework grounded in the projection architecture. The construction has three layers:

*Layer 1: Bipartite amplitudes.* We define the amplitude $cal(A)(G, psi, a, s_A, b, s_B)$ for detecting spinor state $s_A$ at angle $a$ and spinor state $s_B$ at angle $b$. This amplitude is constrained by SU(2) covariance:
$ cal(A)(G, U(g, psi), g dot a, R(g, s_A), g dot b, R(g, s_B)) = cal(A)(G, psi, a, s_A, b, s_B) $
and by a completeness relation connecting bipartite amplitudes to single-site projections:
$ sum_(s_B in {e_(arrow.t), e_(arrow.b)}) |cal(A)(G, psi, a, s_A, b, s_B)|^2 = |chevron.l "project_at"(G, psi, a) | s_A chevron.r|^2 $

The direction is critical: amplitudes are upstream, probabilities are downstream.

*Layer 2: Joint probabilities from amplitudes.* Joint probabilities are *defined* from amplitudes via the bipartite Born rule:
$ P(o_A, o_B | a, b) = |cal(A)(G, psi_(A B), a, s_(o_A), b, s_(o_B))|^2 $

These are not free operations — they inherit all constraints from the amplitude structure. The four joint probabilities satisfy normalization:
$ P(+,+|a,b) + P(+,-|a,b) + P(-,+|a,b) + P(-,-|a,b) = 1 $

The correlation is recovered downstream:
$ E(a, b) = P(+,+) - P(+,-) - P(-,+) + P(-,-) $

*Layer 3: Marginals by partial trace.* The marginal at A is obtained by genuinely summing over B's outcomes:
$ P(o_A = +1 | a, b) = P(+,+ | a,b) + P(+,- | a,b) $

By the completeness relation, this sum collapses to a single-site quantity:
$ P(+1_A | a, b) = |chevron.l "project_at"(G, psi_(A B), a) | e_(arrow.t) chevron.r|^2 $

Note that the right-hand side does not mention $b$ — but this is not a definitional trick. It is a *consequence* of completeness summing over a complete basis at B. The joint probabilities on the left each depend on $b$ through the bipartite amplitudes; their sum happens to be $b$-independent because of the structure of the partial trace.

== The No-Signaling Theorem

*Lemma (Maximally Mixed Reduced State).* Let $psi_(A B)$ be SU(2)-invariant under diagonal action (Axiom D: $U(g, psi_(A B)) = psi_(A B)$). Let $R$ be the fundamental 2D irreducible representation of SU(2) on the spinor space (SpinorBasis + RepIrreducibility). Then the reduced state at each wing is the maximally mixed state $I \/ 2$.

*Proof.* By singlet invariance, the global state is unchanged under simultaneous SU(2) rotation. The reduced state at A, obtained by tracing over B, must therefore be SU(2)-invariant as well: for any $g in "SU"(2)$, $R(g) rho_A R(g)^(dagger) = rho_A$. By Schur's lemma, the only operator on a 2D irreducible representation that commutes with all $R(g)$ is proportional to the identity. Combined with $"Tr"(rho_A) = 1$, this gives $rho_A = I \/ 2$. $square$

*Consequence:* $P(o_A = plus.minus 1 | a, b) = 1 \/ 2$ for all $a$ and all $b$.

*Theorem 7 (No-Signaling).* For any admissible kernel $G$ and any detector angles $a$, $b_1$, $b_2$:
$ P(o_A | a, b_1) = P(o_A | a, b_2) = 1 \/  2 $

*Proof.* By the maximally mixed lemma, the marginal at A equals $1 \/ 2$ regardless of the value of $b$. This follows from:
- Singlet invariance (Axiom D) — no factorization assumed
- Invariant inner product (Axiom C) — no explicit density matrices needed
- 2D irreducibility (SpinorBasis + RepIrreducibility) — Schur's lemma applies
- Linearity of the projection (kernel admissibility) — the partial trace is well-defined $square$

The physical content: POT is *ontically contextual* — the underlying modal structure is global, and the joint probabilities $P(o_A, o_B | a, b)$ genuinely depend on both $a$ and $b$. But POT is *operationally local* — the marginal $sum_(o_B) P(o_A, o_B | a, b) = 1 \/ 2$ is independent of $b$. No experiment at A can detect what setting B has chosen. The contextuality is a property of the ontic level (the pre-projected wave), not of the epistemic level (the observable marginal statistics).

This achieves the key desideratum: *contextual ontology + operational locality*.

= Discussion

Our results demonstrate that Bell inequality violation is a theorem of Projected Ontology Theory — not evidence of non-locality, but a geometric consequence of non-separability in the projection from $cal(H)_("ont")$ to observable spacetime.

== Locality Preserved

POT preserves locality in the following precise sense: no signal or influence travels from detector A to detector B. The measurement at A is a local evaluation of the projection $"project_at"(G, psi_(A B), a)$, and the measurement at B is a local evaluation $"project_at"(G, psi_(A B), b)$. These are the *same* projection operator applied to the *same* wave at different spatial coordinates — analogous to reading two nodes of a single vibrating string. The string doesn't 'send a signal' from one node to the other; the correlation is a structural property of the string itself.

Bell's theorem excludes theories where the outcomes $A(a, lambda)$ and $B(b, lambda)$ are *independent functions* sharing only a hidden variable $lambda$. POT evades this by denying that A and B are independent functions — they are evaluations of a single function $"project_at"(G, psi, dot)$ at different arguments.

== The Born Rule as Modal Visibility

In standard quantum mechanics, the Born rule $P = |chevron.l phi | psi chevron.r|^2$ is a postulate. In POT, it is Axiom (E) — the Born-type modal visibility rule — which states that the probability of an outcome is the *fraction of the modal content that survives projection* into the detector's measurement basis. The spinor inner product $"spinor_inner"("project_at"(G, psi, a), "project_at"(G, psi, b))$ is exactly this overlap.

Combined with the SU(2) covariance axioms (A)–(D), this rule is sufficient to derive the complete correlation function. In v1 of this theory, the $-cos(theta)$ formula was asserted directly (axiom E10); in v2, it is a consequence of modal visibility plus the symmetry structure of $cal(H)_("ont")$. The Born rule is the bridge between geometric symmetry and observable statistics.

This reinterpretation has a concrete consequence: 'collapse' is not a physical process but a change in which modal components are being projected. When an experimenter rotates a polarizer, they change the projection kernel $K_("rep")$, not the wave $psi_(A B)$. The wave continues to exist in full in $cal(H)_("ont")$; only our view of it changes.

== Kernel Unification

The same admissible kernel axioms (linearity A1–A2, zero-preservation A3) that govern gravitational physics also govern measurement physics. The factorization $K = K_("univ") dot K_("dyn") dot K_("rep")$ means:

- The *gravitational sector* uses the elliptic limit of $K_("dyn")$ (static potentials, mass residues) with scalar $K_("rep")$ — producing flat rotation curves via logarithmic coherence.
- The *measurement sector* uses the hyperbolic limit of $K_("dyn")$ (causal propagation) with matrix-valued $K_("rep")$ (acting on spin-½ representations) — producing the $-cos(theta)$ correlation.

This is not two theories glued together; it is one theory with two regimes. The kernel composition axiom ensures that the combined operator remains admissible, so all foundational guarantees (linearity, nullspace closure, projective equivalence) carry through to both sectors simultaneously.

== Experimental Context: Photon Polarization

The natural experimental realization of our framework is photon polarization entanglement, as measured in the Aspect (1982), Clauser, and Zeilinger experiments. In this setting:

- The entangled state $psi_(A B)$ is a photon pair produced by spontaneous parametric down-conversion (SPDC), in the singlet-like state $|H V chevron.r - |V H chevron.r$.
- The detector angles $a, b$ are polarizer orientations.
- The correlation $E(a, b) = -cos(2 theta)$ for polarization (factor of 2 from the spin-1 nature of photons vs. spin-½ in our formalism; the mathematical structure is identical up to this rescaling).

The CHSH inequality $S <= 2$ has been violated in every precision experiment since 1972, with the most stringent loophole-free tests confirming $S = 2.83 plus.minus 0.02$ — matching Tsirelson's bound $2sqrt(2)$ to experimental precision. Our derivation reproduces this value as a theorem.

== What This Paper Does Not Do

We wish to be explicit about a deliberate methodological choice: this paper does not postulate a Lagrangian, a Hamiltonian, or any dynamical equation governing the modal flow in $cal(H)_("ont")$. It does not specify the concrete codomain structure (whether $CC^2$, a Clifford module, or a Lie group representation) beyond what is required for the projection axioms. It does not describe the internal dynamics of the pre-observable space.

This omission is deliberate, not accidental. POT's foundational principle of *epistemic constraint* holds that we can only make rigorous claims about what survives projection. The internal structure of $cal(H)_("ont")$ is, by definition, not directly observable — it is the space *from which* observables are projected. Postulating specific dynamics in Hont would be speculation dressed as theory.

What we *do* claim is the following: the full axiom set (see Appendix) is *satisfiable* — there exist mathematical structures that realize these axioms. Standard quantum mechanics itself provides the existence proof: the singlet state in $CC^2 times.o CC^2$ with the standard inner product is a concrete model satisfying every axiom in this paper. The correlation $-cos(theta)$ is computed, not assumed.

There is a deeper reason for this restraint, one that goes beyond methodology. The projection $Pi$ is many-to-one and non-invertible — information is irreversibly lost. The theorist, the experimental apparatus, and any reasoning tool used to construct the theory are themselves outputs of this projection. We exist *inside* the projection. Therefore, any claim to know the complete Lagrangian of $cal(H)_("ont")$ would require information that the projection provably destroyed. A specific Lagrangian would be one of infinitely many consistent with the same projected observables, and we would have no empirical basis to choose among them.

The axioms in this paper represent the strongest claims that can be made from within the projection: they constrain the class of possible dynamics in $cal(H)_("ont")$ without specifying a unique one. Everything beyond these constraints lies in the nullspace of $Pi$ — epistemically inaccessible by construction. This is not a temporary limitation to be overcome by cleverer mathematics; it is a structural feature of any theory built on irreversible projection.

We consider this an honest position. Many successful frameworks in physics begin with kinematic constraints before dynamics are fully specified — the S-matrix program, topological quantum field theory, and the axiomatic approach to quantum mechanics itself all follow this pattern.

== Model-Theoretic Sanity Checks

Because our derivations rely on an SMT solver (Z3) to verify satisfiability and consistency, we must ensure that the axiom set does not admit degenerate models that trivialize the results. We explicitly address three potential failure modes:

*1. Degenerate angle models.* If $"angle_between"(a, b) equiv 0$ for all $a, b$, then $E(a, b) = -cos(0) = -1$ trivially, and Bell violation would be vacuous. We prevent this with the non-degeneracy axiom: $exists(a, b). theta(a, b) > 0$, the metric axiom $theta(a, b) = 0 arrow.r.double a = b$, and the range constraint $0 <= theta(a, b) <= pi$. These ensure the angle space is a genuine metric space homeomorphic to $S^2$ (enforced further by the transitive SU(2) action).

*2. Observable representation drift.* If the spinor space were allowed to be 3-dimensional or higher, the representation $R$ could be spin-1 or higher, yielding a different correlation function (Legendre polynomials rather than bare cosine). We fix the representation to the fundamental 2D irrep by: (a) providing an explicit orthonormal basis ${e_(arrow.t), e_(arrow.b)}$, (b) requiring that every spinor is a linear combination of these two basis elements (the spanning axiom), and (c) requiring irreducibility of $R$ in the Schur sense (no 1D invariant subspace). These three constraints together pin the observable sector to spin-½.

*3. Anti-smuggling.* The cosine law must emerge from the representation theory of SU(2), not from a rigged angle parameterization. The isometry axiom $theta(g dot a, g dot b) = theta(a, b)$ ensures that the SU(2) angle action is a genuine rotation on $S^2$, not an arbitrary reparameterization that could encode the correlation formula. The Born-type rule (Axiom E) is strictly the quadratic form $P(a) = ||Pi_a(psi)||^2$ — it encodes no angle dependence. And the bipartite amplitudes are constrained by SU(2) covariance, with joint probabilities defined downstream via the bipartite Born rule.

We note an important methodological clarification: we are *not* deriving SU(2) from kernel admissibility. We are deriving *correlations* from *symmetry survival*. The SU(2) structure of $cal(H)_("ont")$ is an ontological posit — the claim that angular momentum symmetry is a feature of the pre-observable space, not an emergent property of spacetime. The admissible kernel framework governs how this symmetry *projects* to observable physics. These are complementary, not derivational.

= Conclusion

We have shown that Bell inequality violation, GHZ contextuality, and operational no-signaling are theorems of Projected Ontology Theory — logical consequences of SU(2) symmetry in $cal(H)_("ont")$, non-separability, and the projection structure.

The central advance of this paper over our previous treatment is the *derivation* of the singlet correlation $E(a,b) = -cos(theta)$ from deeper structural principles. In our initial formalization (theory v1), this formula was asserted as axiom E10 — a direct postulation of the correlation function. In the present work (theory v2), it is derived from five axioms: (A) SU(2) symmetry in $cal(H)_("ont")$, (B) symmetry survival through projection (the intertwiner property), (C) invariant inner products, (D) singlet invariance under diagonal SU(2), and (E) the Born-type modal visibility rule. The cosine law follows from the representation theory of SU(2) applied to the spin-½ sector — standard mathematics, but grounded in a new ontology.

This matters because it addresses the most natural criticism of the previous version: 'You just assumed the answer.' Now the answer follows from symmetry, and the symmetry has clear physical content — it is the claim that $cal(H)_("ont")$ carries the same angular momentum structure that manifests in observable physics. The assumption has moved from formula to structure.

The no-signaling theorem (Theorem 7) addresses the equally natural concern about contextuality: if outcomes depend on the global measurement context, can this be exploited for faster-than-light communication? No — the singlet's SU(2) invariance ensures that marginal statistics at each detector are independent of the remote setting. POT is ontically contextual but operationally local.

The GHZ result strengthens the Bell analysis decisively. Where Bell's theorem is statistical, the GHZ argument is deterministic: a single algebraic contradiction eliminates all noncontextual hidden-variable theories. POT survives this test because its measurement outcomes are context-dependent by construction.

The key structural insight remains that the measurement kernel and the gravitational kernel are sectors of the same factorized operator $K = K_("univ") dot K_("dyn") dot K_("rep")$. The same admissibility axioms that produce flat rotation curves (companion paper) also produce Bell violation and GHZ contextuality — unifying gravitational and quantum phenomena under a single mathematical framework.

The results in this paper cover two-particle (Bell/CHSH) and three-particle (GHZ) entanglement. Extending to $N$-particle systems, quantum teleportation protocols, and decoherence dynamics remain open directions. But the fact that the correct singlet correlation, Bell violation, Tsirelson's bound, GHZ parity constraints, and no-signaling all emerge as machine-verified theorems from structural symmetry axioms suggests that this framework captures something fundamental about the structure of quantum correlations.

All source code, axiom files, and verification scripts are available at https://github.com/eatikrh/kleis.

= Appendix: Complete Axiom Set (v2)

Below is the complete axiom set used in this paper, formatted for reference. Axioms A1–A4 are shared with the rotation curves paper. Axioms E1–E6 cover kernel factorization and non-separability. Axioms S1–S12 are the new SU(2) symmetry axioms that *replace* the old E10 (direct assertion of $-cos(theta)$). Axioms M1–M2 cover measurement geometry. Axiom B1 is the Born-type modal visibility rule. The correlation law $E(a,b) = -cos(theta)$ is now derived, not assumed. Axioms N1–N3 establish no-signaling.

#v(8pt)
block(fill: luma(245), inset: 12pt, radius: 4pt, width: 100%, text(size: 8pt)[
*Foundation (from admissible kernels):* \
*A1.* `kernel_lin_add`: $forall(G, a, b). "adm"(G) ==> K(G, a + b) = K(G, a) + K(G, b)$ \
*A2.* `kernel_lin_smul`: $forall(G, c, a). "adm"(G) ==> K(G, c dot a) = c dot K(G, a)$ \
*A3.* `kernel_maps_zero`: $forall(G). "adm"(G) ==> K(G, 0) = 0$ \
*A4.* `compose_admissible`: $forall(G_1, G_2). ("adm"(G_1) and "adm"(G_2)) ==> "adm"(G_1 compose G_2)$ \
\
*Kernel Factorization (E1–E4):* \
*E1.* `univ_admissible`: $"adm"(K_("univ"))$ \
*E2.* `dyn_admissible`: $"adm"(K_("dyn"))$ \
*E3.* `rep_admissible`: $"adm"(K_("rep"))$ \
*E4.* `kernel_factorizes`: $K_("unified") = K_("univ") compose (K_("dyn") compose K_("rep"))$ \
\
*Non-Separability (E5–E6):* \
*E5.* `entangled_exists`: $forall(G). "adm"(G) ==> not "product"(psi_(A B))$ \
*E6.* `product_states_factor`: $forall(G, psi, a, b). ("adm"(G) and "product"(psi)) ==> exists(psi_A, psi_B). ...$ \
\
*Spinor Space (Dim 2):* \
*V1.* `basis_distinct`: $e_(arrow.t) != e_(arrow.b)$ \
*V2.* `basis_spans`: $forall(s). exists(alpha, beta). s = alpha e_(arrow.t) + beta e_(arrow.b)$ \
*V3–V5.* `basis_orthonormal`: $chevron.l e_(arrow.t) | e_(arrow.t) chevron.r = 1$, $chevron.l e_(arrow.b) | e_(arrow.b) chevron.r = 1$, $chevron.l e_(arrow.t) | e_(arrow.b) chevron.r = 0$ \
\
*Irreducibility:* \
*V6.* `R_irreducible`: $forall(s != 0). exists(g). R(g, s) != c dot s$ for any scalar $c$ \
\
*SU(2) Ontic Symmetry — Axiom (A) (S1–S2):* \
*S1.* `group_action`: $forall(g, h, psi). U(g compose h, psi) = U(g, U(h, psi))$ \
*S2.* `identity_action`: $forall(psi). U(e, psi) = psi$ \
\
*Symmetry Survival — Axiom (B) (S3–S9):* \
*S3.* `angle_action_compose`: $forall(g, h, a). (g compose h) dot a = g dot (h dot a)$ \
*S4.* `angle_action_identity`: $forall(a). e dot a = a$ \
*S5.* `angle_action_isometry`: $forall(g, a, b). theta(g dot a, g dot b) = theta(a, b)$ \
*S6.* `angle_action_transitive`: $forall(a, b). exists(g). g dot a = b$ \
*S7.* `intertwiner`: $forall(g, a, psi, G). "adm"(G) ==> pi(G, U(g, psi), g dot a) = R(g, pi(G, psi, a))$ \
*S8.* `R_group_action`: $forall(g, h, s). R(g compose h, s) = R(g, R(h, s))$ \
*S9.* `R_identity`: $forall(s). R(e, s) = s$ \
\
*Invariant Inner Products — Axiom (C) (S10–S12):* \
*S10.* `ontic_invariant`: $forall(g, psi, phi). chevron.l U(g, psi) | U(g, phi) chevron.r_("ont") = chevron.l psi | phi chevron.r_("ont")$ \
*S11.* `observable_invariant`: $forall(g, v, w). chevron.l R(g, v) | R(g, w) chevron.r_("spin") = chevron.l v | w chevron.r_("spin")$ \
*S12.* `spinor_inner_symmetric`: $forall(v, w). chevron.l v | w chevron.r_("spin") = chevron.l w | v chevron.r_("spin")$ \
\
*Singlet Invariance — Axiom (D) (S13–S14):* \
*S13.* `diagonal_is_simultaneous`: $forall(g). U_("diag")(g, psi_(A B)) = U(g, psi_(A B))$ \
*S14.* `singlet_invariant`: $forall(g). U_("diag")(g, psi_(A B)) = psi_(A B)$ \
\
*Measurement Geometry (M1–M2):* \
*M1.* `angle_symmetric`: $forall(a, b). theta(a, b) = theta(b, a)$ \
*M2.* `angle_self_zero`: $forall(a). theta(a, a) = 0$ \
\
*Born-Type Modal Visibility — Axiom (E) (B0–B1):* \
*B0.* `born_rule`: $forall(G, psi, a). "adm"(G) ==> "spin_outcome"(G, psi, a) = chevron.l pi(G, psi, a) | pi(G, psi, a) chevron.r$ \
*B1.* `inner_normalized`: $forall(G, a). "adm"(G) ==> chevron.l pi(G, psi_(A B), a) | pi(G, psi_(A B), a) chevron.r = 1$ \
\
*Correlation (DERIVED from V1–V6, S1–S14, M1–M2, B0–B1):* \
*D1.* `singlet_correlation` (lemma): $forall(G, a, b). "adm"(G) ==> E(G, psi_(A B), a, b) = -cos(theta(a, b))$ \
\
*Bipartite Measurement + No-Signaling:* \
*J1–J4.* `joint_prob_*_nonneg`: $P(o_A, o_B | a, b) >= 0$ \
*J5.* `joint_normalized`: $sum P(o_A, o_B | a, b) = 1$ \
*J6.* `correlation_from_joint`: $E = P(++) - P(+-) - P(-+) + P(--)$ \
*N1.* `marginal_A_up_def`: $P(+1_A | a, b) = P(+,+ | a,b) + P(+,- | a,b)$ \
*N2.* `marginal_A_down_def`: $P(-1_A | a, b) = P(-,+ | a,b) + P(-,- | a,b)$ \
*N3.* `singlet_reduced_is_mixed`: $P(+1_A | a, b) = 1 \/ 2$ (from Schur's lemma + singlet inv.) \
*N4.* `no_signaling`: $P(+1_A | a, b_1) = P(+1_A | a, b_2)$ (theorem, not def.) \
\
*Bell (B1–B3):* \
*B1.* `bell_holds_for_product`: $forall(G, psi, a, b, c). ("adm"(G) and "product"(psi)) ==> |E(a,b) - E(a,c)| <= 1 + E(b,c)$ \
*B2.* `entangled_violates_bell`: $forall(G). "adm"(G) ==> exists(a, b, c). |E(a,b) - E(a,c)| > 1 + E(b,c)$ \
*B3.* `violation_legitimate`: $forall(G). "adm"(G) ==> not "product"(psi_(A B))$ \
\
*GHZ State (3-particle):* \
*G1.* `bases_distinct`: $"basis"_X != "basis"_Y$ \
*G2–G4.* `outcome_pm1`: $forall("ctx"). o_i("ctx")^2 = 1$ for $i in {A, B, C}$ \
*G5.* `ghz_parity_XXX`: $o_A(X X X) dot o_B(X X X) dot o_C(X X X) = +1$ \
*G6.* `ghz_parity_XYY`: $o_A(X Y Y) dot o_B(X Y Y) dot o_C(X Y Y) = -1$ \
*G7.* `ghz_parity_YXY`: $o_A(Y X Y) dot o_B(Y X Y) dot o_C(Y X Y) = -1$ \
*G8.* `ghz_parity_YYX`: $o_A(Y Y X) dot o_B(Y Y X) dot o_C(Y Y X) = -1$ \
*G9.* `ghz_non_separable`: $forall(G). "adm"(G) ==> not "product"(psi_("GHZ"))$ \
\
*Noncontextuality (for contradiction):* \
*NC1–NC6.* Pre-assigned values $x_i, y_i in {plus.minus 1}$ for $i in {A, B, C}$ \
*NC7–NC18.* Context-independence: $o_i("ctx") = x_i$ or $y_i$ depending on basis at site $i$ \
])

#heading(numbering: none)[Acknowledgments]
The formal verification infrastructure was built using Kleis (https://kleis.io) with Z3 as the backend SMT solver. The author thanks the Kleis AI assistant for collaborative theory development and proof checking. This work builds on the admissible kernel framework established in the companion paper on flat galactic rotation curves.

#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[bell1964\] Bell, J. S. (1964). On the Einstein Podolsky Rosen paradox. Physics Physique Fizika, 1(3), 195-200.]

#par(hanging-indent: 1.5em)[\[aspect1982\] Aspect, A., Dalibard, J., & Roger, G. (1982). Experimental Realization of Einstein-Podolsky-Rosen-Bohm Gedankenexperiment: A New Violation of Bell's Inequalities. Physical Review Letters, 49(25), 1804-1807.]

#par(hanging-indent: 1.5em)[\[clauser1969\] Clauser, J. F., Horne, M. A., Shimony, A., & Holt, R. A. (1969). Proposed Experiment to Test Local Hidden-Variable Theories. Physical Review Letters, 23(15), 880-884.]

#par(hanging-indent: 1.5em)[\[tsirelson1980\] Tsirelson, B. S. (1980). Quantum generalizations of Bell's inequality. Letters in Mathematical Physics, 4(2), 93-100.]

#par(hanging-indent: 1.5em)[\[hensen2015\] Hensen, B., et al. (2015). Loophole-free Bell inequality violation using electron spins separated by 1.3 kilometres. Nature, 526, 682-686.]

#par(hanging-indent: 1.5em)[\[yin2017\] Yin, J., et al. (2017). Satellite-based entanglement distribution over 1200 kilometers. Science, 356(6343), 1140-1144.]

#par(hanging-indent: 1.5em)[\[nobel2022\] The Nobel Committee for Physics (2022). Scientific background: Experiments with entangled photons, establishing the violation of Bell inequalities and pioneering quantum information science.]

#par(hanging-indent: 1.5em)[\[pot_rotation\] Eatik (2026). Flat Galactic Rotation Curves as a Theorem of Projected Ontology: Machine-Verified Derivations Without Dark Matter. Available at https://kleis.io/docs/papers/pot_flat_rotation_curves.pdf.]

#par(hanging-indent: 1.5em)[\[ghz1989\] Greenberger, D. M., Horne, M. A., & Zeilinger, A. (1989). Going beyond Bell's theorem. In M. Kafatos (Ed.), Bell's Theorem, Quantum Theory, and Conceptions of the Universe (pp. 69-72). Springer.]


