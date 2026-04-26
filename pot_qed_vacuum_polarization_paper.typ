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
  #text(size: 17pt, weight: "bold")[Gauge Symmetry Reduces the Null Space: QED Vacuum Polarization from Convergent Integrals]
  
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
      #text(size: 10pt)[We compute the one-loop vacuum polarization of quantum electrodynamics --- the running fine-structure constant $alpha(q^2)$, the QED $beta$-function, and the above-threshold pair-production cross-section --- without invoking dimensional regularization or any subtraction procedure. The Feynman parameter representation $Pi(rho) = (alpha \/ pi) integral_0^1 d x thin x(1-x) ln(1 + rho thin x(1-x))$ is convergent on $[0,1]$ and smooth for all spacelike momenta. The Ward identity $Pi(0) = 0$ is a property of this convergent integrand: at $rho = 0$, the integrand vanishes identically, not by cancellation. This is the decisive structural observation. In the previous paper [AtikPhi4], the $K$--$Q$ framework (Feynman integral kernel $K$, observable projection $Q$) was introduced for $phi^4$ theory, where individual $B_0(rho)$ values are scheme-dependent and only differences lie in $"im"(Q)$. In QED, the Ward identity provides a canonical normalization point: $Pi(0) = 0$ pins the vacuum polarization, making individual $Pi(rho)$ values observable. Gauge symmetry promotes quantities from $ker(Q)$ to $"im"(Q)$. The null space $ker(Q)$ is smaller in gauge theories than in theories without gauge symmetry. We verify 23 structural properties by the Z3 SMT solver and 15 numerical results by direct computation in the Kleis formal verification language. The conclusion: the $K$--$Q$ architecture is not specific to scalar theories. It survives gauge constraints, fermion loops, and the Ward identity --- and is simplified by them.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* QED, vacuum polarization, running coupling, Ward identity, Feynman parameter integral, convergent integral, beta-function, gauge symmetry, projected ontology, formal verification, Z3]

#v(1em)


= Introduction

The previous paper in this series [AtikPhi4] demonstrated that every one-loop observable of $phi^4$ scalar field theory can be computed from convergent Feynman parameter integrals on $[0,1]$, without dimensional regularization. Two operators were introduced: the Feynman integral kernel $K$, which maps the Lagrangian and diagram topology to Feynman parameter integrands, and the observable projection $Q$, which maps formal loop expressions to the quotient space of physical predictions. The composed map $Q compose K$ admits a convergent representative.

A skeptical reader may object: $phi^4$ is the simplest possible case. A scalar theory with no symmetry constraints, no fermions, no gauge invariance. The $K$--$Q$ framework might be a notational convenience for the easy case that collapses when the physics is harder.

This paper answers that objection. We apply the same $K$--$Q$ architecture to QED vacuum polarization at one loop --- a calculation with fermion loops (spinor trace, gamma matrices), gauge symmetry (U(1) invariance), and the Ward identity ($Pi(0) = 0$, enforcing the masslessness of the photon). Every feature absent from $phi^4$ is present here.

The result is the opposite of what the skeptic might expect. The $K$--$Q$ framework is not merely compatible with gauge symmetry --- it is *simplified* by it. The key structural observation:

*In $phi^4$, individual $B_0(rho)$ values are scheme-dependent. Only differences $B_0(rho_1) - B_0(rho_2)$ are observable. In QED, the Ward identity $Pi(0) = 0$ provides a canonical normalization point. Individual $Pi(rho)$ values are scheme-independent and directly observable. Gauge symmetry promotes quantities from $ker(Q)$ to $"im"(Q)$.*

The paper is organized as follows. Section 2 presents the QED Lagrangian and Feynman rules. Section 3 derives the Ward identity from the convergent integrand. Section 4 computes $Pi(rho)$ at multiple momenta from convergent integrals. Section 5 derives the running fine-structure constant. Section 6 extracts the QED $beta$-function. Section 7 treats above-threshold physics. Section 8 makes the $K$--$Q$ gauge analysis explicit, comparing $phi^4$ and QED. Section 9 states what matches, what changes, and what is new.

= QED Lagrangian and Feynman Rules

The QED Lagrangian density is:

$ cal(L)_("QED") = overline(psi) (i gamma^mu D_mu - m) psi - 1/4 F_(mu nu) F^(mu nu) $

where $D_mu = partial_mu + i e A_mu$ is the gauge-covariant derivative. Expanding:

$ cal(L)_("QED") = overline(psi) (i gamma^mu partial_mu - m) psi - e overline(psi) gamma^mu psi A_mu - 1/4 F_(mu nu) F^(mu nu) $

The Feynman rules follow uniquely from the Lagrangian:

$ "Electron propagator:" quad S(p) = i(gamma dot p + m) / (p^2 - m^2 + i epsilon) $
$ "Photon propagator:" quad D_(mu nu)(q) = (-i g_(mu nu)) / (q^2 + i epsilon) quad "(Feynman gauge)" $
$ "Vertex:" quad V^mu = -i e gamma^mu $

QED is a *gauge theory*: the Lagrangian is invariant under local U(1) transformations $psi arrow.r e^(i alpha(x)) psi$, $A_mu arrow.r A_mu - (1\/e) partial_mu alpha$. This gauge invariance constrains the form of radiative corrections via the Ward identity.

At one loop, the photon self-energy (vacuum polarization) is:

$ Pi_(mu nu)(q) = (-1) times "Tr" [(-i e gamma^mu) S(k) (-i e gamma^nu) S(k-q)] $

where the $(-1)$ is the fermion loop sign (from closed fermion loops), and $S(k)$ is the electron propagator. The Dirac trace and momentum integration produce a transverse tensor:

$ Pi_(mu nu)(q) = (q_mu q_nu - g_(mu nu) q^2) times Pi(q^2) $

The transversality $q^mu Pi_(mu nu) = 0$ is exact at all orders --- it is the Ward identity in this channel.

Z3-verified: QED renormalizability, gauge invariance, electron propagator derivation, vacuum polarization from Feynman rules, fermion loop, transversality (6 results in `theories/pot_qed_vacuum_polarization.kleis`).

= The Ward Identity from Convergent Integrals

After evaluating the Dirac trace and performing Feynman parametrization, the scalar vacuum polarization function takes the form:

$ Pi(rho) = alpha / pi integral_0^1 d x thin x(1-x) ln(1 + rho thin x(1-x)) $

where $rho = q_E^2 \/ m^2$ is the Euclidean momentum ratio. This integral is:

+ *Defined on a compact domain* $[0,1]$.
+ *Smooth everywhere:* the argument $1 + rho x(1-x) >= 1$ for all $rho >= 0$.
+ *Bounded:* the integrand satisfies $0 <= x(1-x) ln(1 + rho x(1-x)) <= (1\/4) ln(1 + rho\/4)$.
+ *Vanishes at both endpoints:* $x(1-x) = 0$ at $x = 0$ and $x = 1$.
+ *Convergent by inspection:* a continuous function on a compact interval.

The $x(1-x)$ prefactor is not accidental. It arises from the Dirac trace of the fermion loop --- the spinor structure of QED. It makes the integrand vanish at both endpoints, providing *stronger convergence* than the $phi^4$ integrand $ln(1 + rho x(1-x))$, which is merely zero at the endpoints.

Now consider $rho = 0$:

$ Pi(0) = alpha / pi integral_0^1 d x thin x(1-x) ln(1) = alpha / pi integral_0^1 d x thin x(1-x) times 0 = 0 $

The integrand is *identically zero* at $rho = 0$. Not approximately zero. Not zero after cancellation. Identically zero at every point $x in [0,1]$. This is the Ward identity:

$ Pi(0) = 0 $

The photon remains massless. And this is a property of the *convergent integrand*, not something enforced by regularization. In dimensional regularization, one must verify that the regulator does not break gauge invariance, leading to a careful argument about transversality being preserved in $d$ dimensions. Here, no regulator is present. The convergent representation *is* the gauge-invariant representation.

This is the decisive structural observation of this paper: *the Ward identity is a property of the convergent Feynman parameter integrand.*

Z3-verified: scalar function extraction, Ward identity $Pi(0) = 0$, transversality, convergence, boundedness, endpoint vanishing (6 results).

= Convergent Feynman Parameter Integrals

We evaluate the bare integral (without the $alpha\/pi$ prefactor):

$ I(rho) = integral_0^1 d x thin x(1-x) ln(1 + rho thin x(1-x)) $

at multiple Euclidean momenta, using the adaptive Runge--Kutta solver `ode45`:

#table(
    columns: 4,
    [*$rho = q_E^2 \/m^2$*], [*$I(rho)$*], [*$Pi(rho) = (alpha\/pi) I(rho)$*], [*Integrand bound*],
    [$0$], [$0$ (exact)], [$0$ (Ward)], [$0$],
    [$1$], [$0.03022$], [$7.02 times 10^(-5)$], [$< 0.0558$],
    [$4$], [$0.09664$], [$2.24 times 10^(-4)$], [$< 0.1733$],
    [$10$], [$0.17989$], [$4.18 times 10^(-4)$], [$< 0.3132$],
    [$100$], [$0.49944$], [$1.16 times 10^(-3)$], [$< 0.8145$],
)

Each $Pi(rho)$ value is *individually observable* --- unlike $phi^4$, where only differences are scheme-independent. The reason: the Ward identity pins $Pi(0) = 0$, providing a canonical reference point. There is no additive scheme-dependent constant to worry about.

Consistency checks:

+ *Single integral vs. separate integrals:* $I(10) - I(1)$ computed as a single integral $integral_0^1 d x thin x(1-x) ln[(1+10x(1-x))\/(1+x(1-x))]$ agrees with the difference of separate integrals to $5 times 10^(-6)$.

+ *Telescoping:* $I(10) = [I(10) - I(4)] + [I(4) - I(1)] + I(1)$ to machine precision.

+ *Monotonicity:* $I(1) < I(4) < I(10) < I(100)$. More momentum $arrow.r$ more vacuum polarization $arrow.r$ more charge screening.

Verified in Examples 1--10 of `theories/pot_qed_vacuum_polarization_worked.kleis`.

= The Running Fine-Structure Constant

The running fine-structure constant is:

$ alpha(q^2) = alpha / (1 - Pi(q^2)) $

Since $Pi(rho) > 0$ for $rho > 0$, the denominator $1 - Pi(q^2) < 1$, and $alpha(q^2) > alpha$: the effective charge *increases* with momentum. This is *charge screening*: at short distances, virtual electron--positron pairs partially screen the bare charge, so the measured charge is smaller at long distances (low momenta) than at short distances (high momenta).

#table(
    columns: 4,
    [*$rho$*], [*$Pi(rho)$*], [*$alpha(q^2)$*], [*$Delta alpha \/alpha$*],
    [$0$], [$0$], [$1\/137.036$], [$0$],
    [$1$], [$7.02 times 10^(-5)$], [$0.007298$], [$7.0 times 10^(-5)$],
    [$100$], [$1.16 times 10^(-3)$], [$0.007306$], [$1.16 times 10^(-3)$],
)

At $rho = 100$ ($|q| approx 10 m_e$), the running is $Delta alpha \/alpha approx 0.12%$. This is a small effect because $alpha approx 1\/137$ is small. At much higher energies ($rho tilde.op 10^6$, corresponding to $|q| tilde.op m_Z$), the accumulated effect becomes $alpha(m_Z^2) approx 1\/128$ --- a well-measured quantity at LEP.

The direction of running is the opposite of QCD ($alpha_s$ decreases at high momentum, asymptotic freedom) and $phi^4$ ($lambda$ decreases at high momentum, infrared freedom). QED has charge screening; QCD has anti-screening. Both are computed from convergent Feynman parameter integrals with the same structural method.

Verified in Examples 7--8 of `theories/pot_qed_vacuum_polarization_worked.kleis`.

= The QED β-Function from a Convergent Integral

Differentiating $I(rho)$ under the integral sign:

$ (d I) / (d rho) = integral_0^1 d x thin ([x(1-x)]^2) / (1 + rho thin x(1-x)) $

This is another smooth, bounded, convergent integral on $[0,1]$. At $rho = 0$:

$ (d I) / (d rho) bar_(rho = 0) = integral_0^1 d x thin [x(1-x)]^2 = integral_0^1 d x thin (x^2 - 2 x^3 + x^4) = 1/3 - 1/2 + 1/5 = 1/30 $

This is a textbook calculus exercise. The numerical result from `ode45` agrees: $0.03333$ with error $< 10^(-16)$.

The QED $beta$-function at one loop is:

$ beta(alpha) = (2 alpha^2) / (3 pi) $

The coefficient $2\/(3 pi)$ comes from the derivative $d Pi \/ d (ln q^2) = (alpha \/pi) times m^2 times d I \/ d rho$ and the standard identification of the $beta$-function with the momentum dependence of the coupling. The integral $1\/30$ provides the crucial numerical input.

The derivative decreases with $rho$:

#table(
    columns: 2,
    [*$rho$*], [*$d I \/ d rho$*],
    [$0$], [$0.03333 = 1\/30$],
    [$4$], [$0.01812$],
)

Higher momentum $arrow.r$ weaker rate of increase. The running is logarithmic, as expected from the perturbative $beta$-function.

Verified in Examples 11--12 of `theories/pot_qed_vacuum_polarization_worked.kleis`.

= Above-Threshold Physics: Pair Production

For timelike $q^2 > 4 m^2$ (above the $e^+ e^-$ pair-production threshold), the vacuum polarization acquires an imaginary part:

$ "Im" thin Pi(s) = alpha / 3 (1 + (2 m^2) / s) sqrt(1 - (4 m^2) / s) $

where $s = q^2\/m^2$. This is purely algebraic --- no integration needed. It encodes the rate at which a virtual photon can create an electron--positron pair, weighted by the phase space factor $sqrt(1 - 4m^2\/s)$ and the spin factor $(1 + 2m^2\/s)$.

Key values:

#table(
    columns: 3,
    [*$s\/m^2$*], [*$"Im" thin Pi$*], [*Physical interpretation*],
    [$4$ (threshold)], [$0$ (exactly)], [Pairs barely created at rest],
    [$5$], [$1.52 times 10^(-3)$], [Phase space opening],
    [$100$], [$2.43 times 10^(-3)$], [Approaching $alpha\/3$],
    [$infinity$], [$alpha\/3 = 2.43 times 10^(-3)$], [Asymptotic value],
)

At threshold $s = 4m^2$: the square root vanishes, $"Im" thin Pi = 0$. Above threshold, the imaginary part grows, and at high energy approaches $alpha\/3$ asymptotically.

By the optical theorem, the imaginary part of the vacuum polarization is related to the total cross-section for $e^+ e^- arrow.r "hadrons"$ (or, at lowest order, $e^+ e^- arrow.r mu^+ mu^-$):

$ sigma(e^+ e^- arrow.r mu^+ mu^-) = (4 pi alpha^2) / (3 s) (1 + (2 m_mu^2) / s) sqrt(1 - (4 m_mu^2) / s) $

This is the Born cross-section multiplied by kinematic factors --- all computed from the same algebraic expression that appears as $"Im" thin Pi$.

Verified in Examples 13--14 of `theories/pot_qed_vacuum_polarization_worked.kleis`.

= The K--Q Architecture with Gauge Symmetry

This section makes explicit the structural comparison between the $K$--$Q$ framework in $phi^4$ (previous paper) and QED (this paper). The same operators appear; what changes is the *size* of the null space.

=== The kernel $K$ for QED

The kernel $K$ maps the QED Lagrangian and the bubble diagram topology to the Feynman parameter integrand:

$ K : (cal(L)_("QED")\, "bubble") arrow.r.long x(1-x) ln(1 + rho thin x(1-x)) $

The extra $x(1-x)$ prefactor (compared to $phi^4$'s $ln(1 + rho x(1-x))$) arises from the Dirac trace of the fermion loop. It is not a choice --- it is determined by the spinor structure of QED.

=== The observable projection $Q$ for QED

$Q$ maps formal loop expressions to the space of physical predictions. As in $phi^4$, $Q$ is defined by the equivalence relation: two expressions are equivalent if they yield the same values for all measurable quantities.

The action of $Q$ on QED one-loop expressions:

+ $Q(Sigma_("electron")(p)) = 0$ for the mass renormalization part. The electron self-energy mass shift, like $phi^4$'s tadpole $A_0$, lies in $ker(Q)$. It is absorbed into the physical electron mass.

+ $Q(Z_2 - 1) = 0$. The wavefunction renormalization is not individually observable --- it cancels in the Ward identity $Z_1 = Z_2$.

+ $Q(Pi(rho)) = Pi(rho)$. *This is the decisive difference from $phi^4$.* Individual $Pi(rho)$ values are observable, because $Pi(0) = 0$ is fixed by the Ward identity. There is no additive constant to absorb.

=== The structural comparison

#table(
    columns: 4,
    [*Quantity*], [*$phi^4$ status*], [*QED status*], [*Why different*],
    [$A_0$ or $Sigma_("mass")$], [$ker(Q)$], [$ker(Q)$], [Mass renormalization in both],
    [Individual loop function], [Mixed], [$"im"(Q)$], [Ward identity pins $Pi(0) = 0$],
    [Differences], [$"im"(Q)$], [$"im"(Q)$ (redundant)], [Automatic from individual values],
    [$beta$-function], [$"im"(Q)$], [$"im"(Q)$], [Same status in both],
    [Imaginary part], [$"im"(Q)$], [$"im"(Q)$], [Same status in both],
)

The table makes visible what gauge symmetry does to the $K$--$Q$ structure:

*Gauge symmetry reduces $ker(Q)$.*

In $phi^4$, the individual $B_0(rho)$ values contain a scheme-dependent additive constant ($gamma_E$, $ln(4 pi mu^2\/m^2)$). The entire orbit of this constant under scheme changes is in $ker(Q)$. To get an observable, one must take a difference that cancels this constant.

In QED, the Ward identity eliminates this degree of freedom. The statement $Pi(0) = 0$ is not a renormalization *condition* (a choice) --- it is a *consequence of gauge invariance*. It holds in every scheme. The additive constant is fixed to zero by symmetry. Individual $Pi(rho)$ values are therefore scheme-independent and observable.

This means $ker(Q)$ is *smaller* in QED than in $phi^4$. The items that were in $ker(Q)$ (additive constants in the loop function) have been *promoted* to $"im"(Q)$ by gauge symmetry. The null space has shrunk; the observable space has grown.

=== Why $Q compose K$ is finite for QED

The composed map $Q compose K$ for QED produces:

$ Q compose K : cal(L)_("QED") arrow.r.long Pi(rho) = alpha/pi integral_0^1 d x thin x(1-x) ln(1 + rho thin x(1-x)) $

This is a convergent integral on $[0,1]$. No intermediate quantity diverges. The integrand is smooth, bounded, and vanishes at both endpoints. The Ward identity is manifest ($Pi(0) = 0$ from $ln(1) = 0$).

The textbook factorization passes through $ker(Q)$:

$ cal(L)_("QED") arrow.r^(K_1) Pi_("bare")(epsilon) arrow.r^("subtract") Pi_("ren")(q^2) $

where $Pi_("bare")$ diverges as $epsilon arrow.r 0$. The subtraction removes the $1\/epsilon$ pole (which is in $ker(Q)$) and fixes $Pi(0) = 0$ as a renormalization condition. In the convergent representation, the subtraction is unnecessary: $Pi(0) = 0$ is automatic.

The sentence this paper exists to establish: *Gauge symmetry does not complicate the $K$--$Q$ framework. It simplifies it. The Ward identity reduces $ker(Q)$ by providing a canonical normalization point. What required a difference in $phi^4$ is an individual value in QED. The composed map $Q compose K$ has a convergent representative in which the Ward identity holds by construction.*

= What Matches, What Changes, What Is New

=== What matches textbook QED

Every numerical result: the vacuum polarization values $Pi(rho)$ at multiple momenta, the running of $alpha$, the $beta$-function coefficient $2\/(3 pi)$, the above-threshold imaginary part, and the pair-production cross-section formula. No standard result is modified. The numbers match Peskin & Schroeder Section 7.5.

=== What the two papers establish together

The $phi^4$ paper [AtikPhi4] introduced the $K$--$Q$ framework for the simplest case: a scalar theory with no gauge symmetry, where only $B_0$ differences are observable. This paper extends the framework to QED, where gauge symmetry (the Ward identity) promotes individual $Pi(rho)$ values to observables.

Together, the two papers demonstrate:

+ *$K$--$Q$ is not theory-specific.* The same kernel--projection architecture applies to both $phi^4$ and QED, with different Lagrangians and different diagram content, but the same structural decomposition.

+ *Gauge symmetry interacts predictably with $K$--$Q$.* The Ward identity reduces $ker(Q)$ --- it does not introduce new complications. The framework is simplified, not strained.

+ *Fermion loops are handled.* The Dirac trace produces the $x(1-x)$ prefactor, which strengthens convergence. The spinor structure is absorbed into the Feynman parameter integrand without difficulty.

=== What is new

Three structural claims that go beyond the $phi^4$ paper:

+ *The Ward identity is a property of the convergent integrand.* At $rho = 0$, the integrand $x(1-x) ln(1 + 0) = 0$ identically. No regularization is needed to see this. The photon mass is zero because the convergent representative says so, not because a regulator preserves gauge invariance.

+ *Gauge symmetry reduces $ker(Q)$.* The additive constant that makes individual $B_0(rho)$ scheme-dependent in $phi^4$ is fixed to zero by the Ward identity in QED. This is a structural feature of the $K$--$Q$ decomposition: gauge symmetry shrinks the null space.

+ *Individual values become observable.* In $phi^4$, only differences (quotients) of $B_0$ are in $"im"(Q)$. In QED, individual $Pi(rho)$ values are in $"im"(Q)$. The observable space is *larger* when gauge symmetry is present. This is the opposite of the naive expectation that gauge invariance constrains and complicates.

=== The generality claim

The convergent Feynman parameter method is not restricted to these two examples. The Feynman parametrization is universal: it applies to every one-loop diagram in every renormalizable QFT. What changes between theories is the Lagrangian (determining the vertex structure and trace algebra) and the diagram topology (determining the number of propagators and the momentum routing). The $K$--$Q$ structure --- convergent representative for $"im"(Q)$, non-observable quantities in $ker(Q)$, composed map $Q compose K$ finite --- is a consequence of the Feynman parameter representation itself.

The decisive sentence, now established in two independent calculations: *the map $Q compose K : cal(L) arrow.r "Observables"$ admits a representation in which no intermediate quantity diverges. The UV divergences of textbook QFT are artifacts of factoring this map into substeps that compute elements of $ker(Q)$ before projecting them away.*

What $phi^4$ established in the scalar sector, QED now confirms in the gauge sector. The architecture survives.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[PS1995\] Peskin, M. E. and Schroeder, D. V. *An Introduction to Quantum Field Theory.* Addison-Wesley (1995).]

#par(hanging-indent: 1.5em)[\[tH1972\] 't Hooft, G. and Veltman, M. Regularization and renormalization of gauge fields. *Nucl. Phys. B* 44, 189--213 (1972).]

#par(hanging-indent: 1.5em)[\[PV1979\] Passarino, G. and Veltman, M. One-loop corrections for $e^+ e^-$ annihilation into $mu^+ mu^-$ in the Weinberg model. *Nucl. Phys. B* 160, 151--207 (1979).]

#par(hanging-indent: 1.5em)[\[AtikPhi4\] Atik, E. Renormalization without infinities: one-loop $phi^4$ physics from convergent integrals. Volume X Supplement of the POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikX\] Atik, E. Projection singularities: why physics has no infinities. Volume X of the POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[JL1950\] Jost, R. and Luttinger, J. M. Vacuum polarization and $e^4$ charge renormalization for electrons. *Helv. Phys. Acta* 23, 201 (1950).]


