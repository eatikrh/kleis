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
  #text(size: 17pt, weight: "bold")[The Abstract K-Q Framework: Kernels, Projections, and Null-Space Structure Across Domains]
  
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
      #text(size: 10pt)[Every paper in the Projected Ontology Theory (POT) series uses a kernel $K$ that maps a theory specification to formal expressions, and a projection $Q$ that extracts observables. The gravitational paper uses a logarithmic Green's function. The entanglement paper uses spinor projections. The electrodynamics paper uses the exterior derivative. The Navier-Stokes epilogue uses Biot-Savart. The six K-Q papers use the Feynman integral kernel. This paper names the abstract structure they share. We formalize the domain-independent $(K, Q)$ pair and show that it generates three null spaces: $ker(K)$ (what the kernel does not touch), $ker(Q)$ (what the projection erases), and $ker(Q compose K)$ (what produces no observable). The fundamental inclusion $ker(K) subset.eq ker(Q compose K)$ holds universally. The gap $K^(-1)(ker(Q)) without ker(K)$ --- theory inputs that $K$ processes but $Q$ erases --- is where the six atlas types live. For electrodynamics, the gap is empty: $ker(K) = ker(Q)$ (gauge orbits exhaust the null space). For perturbative QFT, the gap contains six structurally distinct types. Resolution structure (generalizing loop order) enables migration across the $ker(Q)$ boundary. The admissibility boundary separates abelian from non-abelian gauge theories: the Yang-Mills kernel $K_("YM")(A) = d A + A and A$ is not admissible, and the Lie bracket defect forces confinement. Five admissible kernels across five physical domains, plus one non-admissible kernel at the structural boundary, are catalogued and classified. All 24 structural results are machine-verified by Z3.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* K-Q framework, kernel, projection, null space, ker(K), ker(Q), admissibility, Biot-Savart, Feynman integral, exterior derivative, confinement, resolution, migration, Z3]

#v(1em)


= Introduction

The Projected Ontology Theory (POT) series now spans eleven papers across five physical domains: galactic dynamics [AtikRotation], quantum measurement [AtikEntanglement], electrodynamics [AtikEM], fluid mechanics [AtikNS], and perturbative quantum field theory [AtikPhi4, AtikQED, AtikYM, AtikGhost, AtikGauge, AtikAtlas]. Each paper uses a *kernel* $K$ that maps theory inputs to formal expressions, and a *projection* $Q$ that extracts observables from those expressions. The composed map $Q compose K$ takes theory to observables.

But the papers never stated the abstract structure they share. The gravitational kernel is a logarithmic Green's function. The measurement kernel is a spinor projection parameterized by detector angles. The electromagnetic kernel is the exterior derivative $d$ restricted to 1-forms. Biot-Savart maps vorticity to velocity. The Feynman integral kernel maps Lagrangians and diagram topologies to convergent parameter integrands. These are five different mathematical objects in five different physical settings --- yet all serve the same structural role.

This paper formalizes that role. We define the abstract $(K, Q)$ pair and derive three consequences:

+ *Three null spaces.* Every $(K, Q)$ pair generates $ker(K)$, $ker(Q)$, and $ker(Q compose K)$. These live in different spaces and satisfy a universal inclusion: $ker(K) subset.eq ker(Q compose K)$.

+ *The gap.* The set $K^(-1)(ker(Q)) without ker(K)$ --- inputs that $K$ processes but $Q$ erases --- is where null-space structure lives. The preceding atlas [AtikAtlas] found six types there. This paper shows that the gap's existence and size depend on the kernel.

+ *The admissibility boundary.* An admissible kernel (linear, zero-preserving) has well-behaved null spaces with linear structure. The Yang-Mills kernel breaks admissibility, and the Lie bracket defect forces confinement. Electrodynamics sits at the boundary: the unique gauge theory with an admissible kernel.

The classification:

#table(
    columns: 4,
    [*Kernel*], [*Domain*], [*Admissible?*], [*Gap*],
    [$K_("grav")$: log Green's fn], [Galactic dynamics], [Yes], [Open],
    [$K_("meas")$: spinor proj.], [Quantum measurement], [Yes], [Open],
    [$K_("em")$: exterior $d$], [Electrodynamics], [Yes], [Empty],
    [$K_("BS")$: Biot-Savart], [Fluid mechanics], [Yes], [Open],
    [$K_("feyn")$: Feynman integrals], [Perturbative QFT], [Yes], [Six types],
    [$K_("YM")$: $d A + A and A$], [Non-abelian gauge], [*No*], [N/A],
)

Five admissible kernels across five domains. One non-admissible kernel at the structural boundary. The abstract $(K, Q)$ framework unifies them.

= The Abstract (K, Q) Axioms

=== Two maps, three null spaces

The abstract structure is a pair of maps:

$ K : "TheorySpec" arrow.r "FormalExpr" $
$ Q : "FormalExpr" arrow.r "Observable" $

$K$ is the *production kernel*: it takes a theory specification (Lagrangian, vorticity distribution, gauge potential, mass distribution, quantum state) and produces formal expressions (loop integrands, velocity fields, field strengths, gravitational potentials, measurement outcomes). $Q$ is the *observable projection*: it extracts the measurable content from the formal expressions.

The composed map $Q compose K : "TheorySpec" arrow.r "Observable"$ produces observables directly. Three null spaces arise:

$ ker(K) &= { t in "TheorySpec" : K(t) = 0 } $
$ ker(Q) &= { f in "FormalExpr" : Q(f) = 0 } $
$ ker(Q compose K) &= { t in "TheorySpec" : Q(K(t)) = 0 } $

These live in *different spaces*. $ker(K)$ and $ker(Q compose K)$ are subsets of TheorySpec. $ker(Q)$ is a subset of FormalExpr. They cannot be directly compared --- but they are connected by the fundamental inclusion.

=== The fundamental inclusion

*Theorem.* $ker(K) subset.eq ker(Q compose K)$.

*Proof.* If $t in ker(K)$, then $K(t) = 0$, so $Q(K(t)) = Q(0) = 0$, hence $t in ker(Q compose K)$. $square$

This is trivial but structurally important: everything the kernel ignores is necessarily unobservable. The converse is false --- there are theory inputs that $K$ processes (produces nonzero formal expressions) but $Q$ erases (projects to zero observable). These inputs live in the *gap*.

=== Admissibility

An *admissible* kernel satisfies:
+ *Linearity*: $K(a + b) = K(a) + K(b)$
+ *Scalar linearity*: $K(c dot a) = c dot K(a)$
+ *Zero preservation*: $K(0) = 0$

Admissibility ensures that $ker(K)$ is a subspace (closed under addition and scalar multiplication) and that projective equivalence ($a tilde b$ iff $K(a) = K(b)$) is well-defined. All five kernels in the POT catalogue are admissible. The Yang-Mills kernel is not.

Z3-verified: all four kernels (gravitational, measurement, EM, Biot-Savart) have $(K, Q)$ structure (4 results).

= The Three-Space Decomposition

For any $(K, Q)$ pair, the domain TheorySpec decomposes into three regions via $Q compose K$:

$ "TheorySpec" = ker(K) union.sq [K^(-1)(ker(Q)) without ker(K)] union.sq K^(-1)("im"(Q)) $

+ *$ker(K)$*: theory inputs that $K$ does not touch. These produce zero formal expression. In QFT: free fields, total derivatives, the $theta$-term. In EM: there is no such sector (every potential produces a field strength unless it is pure gauge, which is handled by $ker(Q)$).

+ *$K^(-1)(ker(Q)) without ker(K)$*: the *gap*. Theory inputs that $K$ processes (nonzero formal expressions) but $Q$ erases (zero observable). This is where the atlas types live. In QFT: scheme constants, ghosts, unphysical polarizations, anomalous currents, confined states, topological sectors --- the six types from [AtikAtlas].

+ *$K^(-1)("im"(Q))$*: theory inputs that reach observables. The $beta$-function, scattering amplitudes, decay rates, rotation curves, flow measurements --- everything physics actually predicts.

The gap is the structurally interesting region. Its size depends on the kernel:

*Empty gap (EM).* For the electromagnetic kernel $K_("em") = d|_(Omega^1)$, the null space $ker(d)$ consists of closed 1-forms --- the gauge orbits. On a contractible manifold, $ker(d|_(Omega^1)) = d Omega^0$ (exact 0-forms). The gauge orbits exhaust $ker(Q compose K)$: there is nothing that $K$ processes but $Q$ erases. The gap is empty. This is the simplest case.

*Rich gap (QFT).* For the Feynman integral kernel, $ker(K)$ is small (total derivatives, free fields) but $ker(Q)$ contains six structurally distinct types. The gap $K^(-1)(ker(Q)) without ker(K)$ is where all the structure identified in the preceding atlas paper lives. This is the richest case.

Z3-verified: fundamental inclusion, nonempty gap for Feynman, empty gap for EM, three-space decomposition (4 results).

= The Kernel Catalogue

Five admissible kernels instantiate the abstract framework across five physical domains.

=== $K_("grav")$: Logarithmic Green's function (galactic dynamics)

The gravitational kernel maps a modal density (mass distribution from the ontological layer) to a projected gravitational potential. The key structural result [AtikRotation]: a slowly-decaying coherence function produces a logarithmic potential $Phi tilde ln(r)$, which yields flat rotation curves $v(r) = "const"$ without dark matter. The projected mass grows linearly: $M(r) = mu r$. The Baryonic Tully-Fisher relation $M tilde v^4$ follows algebraically.

=== $K_("meas")$: Spinor projection (quantum measurement)

The measurement kernel projects an ontological state (in a 3-dimensional complex Hilbert space) onto a detector-angle-dependent observable. The key structural result [AtikEntanglement]: Bell inequality violation arises not from non-locality but from the many-to-one character of the projection. Two entangled particles share a common pre-image; the projection creates correlations that no local hidden-variable model can reproduce.

=== $K_("em")$: Exterior derivative (electrodynamics)

The electromagnetic kernel $K_("em") = d|_(Omega^1)$ maps gauge potentials (1-forms) to field strengths (2-forms): $F = d A$. The key structural result [AtikEM]: this is the unique gauge theory with an admissible kernel, because $d$ is linear. The nullspace $ker(d|_(Omega^1))$ is exactly the gauge freedom. The nilpotency $d^2 = 0$ means composing $K_("em")$ with itself annihilates everything --- a property unique to this kernel.

=== $K_("BS")$: Biot-Savart (fluid mechanics)

The Biot-Savart kernel maps vorticity distributions to velocity fields: $u(x) = integral K(x - y) times omega(y) d y$ where $K(x) = x \/ (4 pi |x|^3)$. The key structural results [AtikNS]: velocity decays as $1 \/ r^2$, strain as $1 \/ r^3$, and the Taylor remainder as $1 \/ r^4$. These scaling laws control the perturbative analysis of vortex tube interactions in the Navier-Stokes regularity program.

=== $K_("feyn")$: Feynman integral kernel (perturbative QFT)

The Feynman integral kernel maps (Lagrangian, diagram topology) pairs to convergent Feynman parameter integrands on $[0, 1]$. The composed map $Q compose K_("feyn")$ produces observables (the $beta$-function, scattering amplitudes, decay rates) without any intermediate quantity diverging. The key structural result: $ker(Q)$ contains six types [AtikAtlas], including the migratory type (anomalous currents) that demonstrates $ker(Q)$ is not resolution-stable.

Z3-verified: all five kernels admissible; EM nilpotent; Feynman has atlas structure (4 results).

= The Activity Classification

The preceding atlas [AtikAtlas] identified five activity modes for null-space elements. In the abstract framework, these become structural properties of any $(K, Q)$ pair:

=== Inert (Type 1)

An element is *inert* if it is stable in $ker(Q)$ (remains there across all resolution levels) and does not couple into $"im"(Q)$ through $Q compose K$. It is passive cargo. Example: scheme-dependent constants ($A_0$, individual $B_0$ values) in perturbative QFT. They exist in the formal expressions but the projection annihilates them.

=== Active (Types 2, 5)

An element is *active* if it is stable in $ker(Q)$ but shapes $"im"(Q)$ through $Q compose K$. It is unobservable in itself, but its presence determines what the observables are. Example: Faddeev-Popov ghosts in non-abelian gauge theory. They are in $ker(Q)$ (unphysical) but their loops determine $beta_0$ (observable). The ghost-activity theorem [AtikGhost] proved this is controlled by $f^(a b c) != 0$.

=== Redistributive (Type 3)

An element is *redistributive* if it absorbs representation variation to stabilize $"im"(Q)$. It does not shape a specific observable but compensates for representation-dependent changes so that the total is representation-invariant. Example: unphysical polarizations of gauge bosons. The gluon loop contribution to $beta_0$ is $xi$-dependent, but the longitudinal/scalar sector absorbs the $xi$ variation so that the total (gluon + ghost) is $xi$-independent.

=== Migratory (Type 4)

An element is *migratory* if it moves from $ker(Q)$ to $"im"(Q)$ across resolution levels. Its membership in $ker(Q)$ is not permanent --- it depends on the resolution at which $K$ operates. Example: the axial current divergence $partial_mu j_5^mu$. At tree level (resolution 0), it vanishes and lies in $ker(Q)$. At one loop (resolution 1), the triangle diagram generates a nonzero value and it moves to $"im"(Q)$. The anomaly is the migration event.

=== Latent (Type 6)

An element is *latent* if it could in principle migrate to $"im"(Q)$ but empirically does not. Example: the vacuum angle $theta$ in QCD. If $theta != 0$, it would produce observable CP violation (neutron electric dipole moment). Empirically, $theta < 10^(-10)$. The strong CP problem, in $(K, Q)$ language, is: why does this element remain inert?

Z3-verified: scheme constants inert; ghosts active; polarizations redistributive; axial divergence migratory (4 results).

= ker(K): What the Kernel Does Not Touch

The six preceding K-Q papers focused entirely on $ker(Q)$ --- the null space of the projection. This paper introduces $ker(K)$ as a separate structural object: the null space of the kernel itself.

$ker(K)$ consists of theory inputs that $K$ maps to zero --- configurations that the production mechanism does not activate. They are invisible not because the projection erases them, but because the kernel never generates them in the first place.

=== Domain-specific examples

*QFT (Feynman kernel).* $ker(K_("feyn"))$ contains:
- *Free fields*: a non-interacting field has propagators but no vertices. $K$ generates no loop diagrams from it.
- *Total derivatives*: they do not contribute to Feynman rules. $K$ does not see them.
- *The $theta$-term*: $cal(L)_theta = theta (g^2) / (32 pi^2) F_(mu nu)^a tilde(F)^(a mu nu)$ is a total derivative perturbatively. $K$ produces no diagrams from it. Non-perturbatively (instantons), it bypasses $K$ entirely.

*EM (exterior derivative).* $ker(d|_(Omega^1))$ consists of closed 1-forms. On a contractible manifold, these are exact: $ker(d|_(Omega^1)) = d Omega^0$. The gauge orbits are exactly $ker(K_("em")))$.

*Fluids (Biot-Savart).* $ker(K_("BS"))$ consists of vorticity distributions that produce zero velocity. Since the Biot-Savart kernel inverts the curl (up to boundary conditions), $ker(K_("BS"))$ is related to irrotational contributions --- potential flow components that the vorticity-based kernel does not couple to.

=== ker(K) vs ker(Q): two different absences

$ker(K)$ and $ker(Q)$ represent structurally different reasons for unobservability:

#table(
    columns: 3,
    [*Property*], [*ker(K)*], [*ker(Q)*],
    [What it contains], [Theory inputs $K$ ignores], [Formal expressions $Q$ erases],
    [Space], [TheorySpec], [FormalExpr],
    [Cause of absence], [Kernel blindness], [Projection erasure],
    [Structural role], [Outside the mechanism], [Inside the mechanism but projected out],
)

The gap $K^(-1)(ker(Q)) without ker(K)$ consists of theory inputs that $K$ *does* process but $Q$ *does* erase. This is where all the interesting structure lives --- and where the two null spaces interact.

= The Admissibility Boundary

Not all kernels are admissible. The Yang-Mills kernel

$ K_("YM")(A) = d A + A and A $

violates linearity:

$ K_("YM")(A + B) = d(A + B) + (A + B) and (A + B) = K_("YM")(A) + K_("YM")(B) + A and B + B and A $

The admissibility defect $Delta(A, B) = K(A + B) - K(A) - K(B) = A and B + B and A = [A, B]$ is the Lie bracket.

The electrodynamics paper [AtikEM] identified this boundary precisely: electrodynamics is the *unique* gauge theory with an admissible projection kernel, because $U(1)$ is abelian and the quadratic self-interaction $A and A$ vanishes. Non-abelian gauge theories ($S U(N)$, $N >= 2$) necessarily break admissibility.

The break is structurally meaningful. The confinement paper [AtikConfinement] showed that the Lie bracket defect forces *fiber non-invariance*: color charge becomes a fiber-dependent predicate under the non-admissible kernel, making it unobservable as an asymptotic state. This is a structural necessary condition for confinement --- derived from the kernel's non-linearity, not from QCD-specific dynamics.

The admissibility restoration paper [AtikRestoration] showed that a non-admissible kernel can become *effectively* admissible through coupling to an additional field that compensates the Lie bracket defect. This is the structural role of the Higgs field in the electroweak sector: it restores admissibility for $S U(2) times U(1)$, making weak charges observable despite the non-abelian gauge algebra.

=== The landscape

The admissibility boundary partitions gauge theories into three structural classes:

#table(
    columns: 3,
    [*Class*], [*Kernel*], [*Consequence*],
    [Admissible], [$K_("em") = d$], [Gauge orbits $=$ $ker(K)$; charges observable],
    [Non-admissible], [$K_("YM") = d + [dot, dot]$], [Lie bracket defect; confinement],
    [Restored], [$K_("YM") + phi$-coupling], [Effective admissibility; masses + observable charges],
)

Z3-verified: Yang-Mills non-admissible; defect is Lie bracket; forces confinement; EM unique admissible gauge (4 results).

= Resolution and Migration

The atlas paper [AtikAtlas] discovered that $ker(Q)$ is not resolution-stable: the chiral anomaly moves an element from $ker(Q)$ to $"im"(Q)$ across loop orders. In the abstract framework, "loop order" generalizes to *resolution level* --- the approximation depth at which $K$ operates.

=== Resolution is domain-specific

Each kernel has its own notion of resolution:

#table(
    columns: 3,
    [*Kernel*], [*Domain*], [*Resolution*],
    [$K_("feyn")$], [QFT], [Loop order (tree, one-loop, two-loop, ...)],
    [$K_("BS")$], [Fluids], [Scale separation / Reynolds number regime],
    [$K_("grav")$], [Gravity], [Distance scale / mass shell],
    [$K_("em")$], [EM], [Perturbation order in external field],
)

The abstract statement is: at resolution $n$, the kernel $K$ generates formal expressions up to depth $n$. The projection $Q$ may classify some of these as unobservable at depth $n$ but observable at depth $n + 1$. If this happens, the element *migrates* from $ker(Q)$ to $"im"(Q)$.

=== Stability vs. migration

An element is *resolution-stable* if its membership in $ker(Q)$ does not depend on the resolution level. Types 1--3 from the atlas are stable: scheme constants, ghosts, and unphysical polarizations remain in $ker(Q)$ at all loop orders.

An element is *migratory* if its membership changes. Type 4 is migratory: the axial current divergence $partial_mu j_5^mu$ is in $ker(Q)$ at tree level (zero, contributes nothing) but exits $ker(Q)$ at one loop (nonzero, determines $pi^0 arrow gamma gamma$). The anomaly is the migration event.

The abstract framework asks: does migration occur in non-QFT domains? Does fluid mechanics have an analogue of the chiral anomaly --- a quantity that is unobservable at one scale but becomes observable at a finer scale? Does the gravitational kernel have resolution-dependent $ker(Q)$ structure? These are open questions that the abstract framework makes precise.

Z3-verified: Feynman kernel has resolution; scheme constants stable; ghosts stable; axial divergence migrates (4 results).

= Implications

=== The framework is not QFT-specific

The K-Q papers were written in the language of perturbative QFT. This paper shows that the structure they found --- null spaces, activity modes, migration, admissibility boundaries --- is domain-independent. Five physical domains, five different mathematical kernels, one abstract framework. The six atlas types are QFT instantiations of general structural phenomena.

=== ker(K) is a new structural object

The preceding papers studied only $ker(Q)$. This paper introduces $ker(K)$ and shows that the two null spaces have different structural origins (kernel blindness vs. projection erasure) and interact through the gap $K^(-1)(ker(Q)) without ker(K)$. The gap's size --- empty for EM, rich for QFT --- is a structural diagnostic of the $(K, Q)$ pair.

=== The admissibility boundary has physical content

Admissibility is not a mathematical convenience. It separates observable from confined charges, determines whether gauge orbits exhaust the null space, and controls whether the Higgs mechanism is structurally necessary. The Yang-Mills defect $Delta = [A, B]$ is the Lie algebra itself, making confinement a consequence of non-linearity rather than strong coupling.

=== Resolution structure generalizes loop order

"Loop order" is QFT's name for a domain-independent concept: the approximation depth at which the kernel operates. Migration (Type 4) is the structural phenomenon that occurs when $ker(Q)$ depends on resolution. The abstract framework makes this precise and asks whether migration occurs in other domains.

=== What this framework does and does not claim

This framework claims: every POT paper is an instance of the $(K, Q)$ structure; the atlas types are consequences of the abstract axioms; the admissibility boundary separates structurally distinct classes of gauge theories; and resolution structure is domain-independent.

This framework does not claim: that the five activity modes are exhaustive (there may be others), that every domain has all six atlas types (the gap may be empty), or that migration occurs outside QFT (this is an open question the framework poses, not answers).

The abstract $(K, Q)$ framework is a structural lens. It does not compute --- it classifies. And the classification reveals that the structure found in perturbative QFT is not an accident of that domain but an instance of something more general.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[AtikRotation\] Atik, E. Flat galactic rotation curves from projected ontology without dark matter. POT VUFT Series, Vol. I. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[AtikEntanglement\] Atik, E. Quantum entanglement as a projection artifact: machine-verified Bell violation without non-locality. POT VUFT Series, Vol. II. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[AtikEM\] Atik, E. Electrodynamics as a theorem of projected ontology. POT VUFT Series, Vol. III. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[AtikConfinement\] Atik, E. Confinement as fiber non-invariance: the admissibility boundary in projected ontology. POT VUFT Series, Vol. IV. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[AtikRestoration\] Atik, E. Admissibility restoration: the structural necessity of symmetry-breaking fields. POT VUFT Series, Vol. V. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[AtikNS\] Atik, E. Epilogue: the kernel and the fluid --- Navier-Stokes regularity as projected ontology. POT VUFT Series. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[AtikPhi4\] Atik, E. Renormalization without infinities: one-loop $phi^4$ physics from convergent integrals. POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikQED\] Atik, E. Gauge symmetry reduces the null space: QED vacuum polarization from convergent integrals. POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikYM\] Atik, E. The null space is not inert: Yang-Mills vacuum polarization from convergent integrals. POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikGhost\] Atik, E. Ghost-mediated null-space activity: a structural theorem from the K-Q framework. POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikGauge\] Atik, E. Gauge dependence and the boundary of ghost activity. POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikAtlas\] Atik, E. The structural atlas of ker(Q). POT VUFT Series. *Preprint* (2026).]


