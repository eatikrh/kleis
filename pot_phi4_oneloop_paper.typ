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
  #text(size: 17pt, weight: "bold")[Renormalization Without Infinities: One-Loop φ⁴ Physics from Convergent Integrals]
  
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
      #text(size: 10pt)[We compute every observable prediction of $phi^4$ scalar field theory at one loop --- the $beta$-function, forward scattering amplitudes at multiple energies, cross-section ratios, and above-threshold imaginary parts --- without invoking dimensional regularization, Laurent series, or any subtraction procedure. We define two operators: the Feynman integral kernel $K$, which maps the Lagrangian and diagram topology to Feynman parameter integrands, and the observable projection $Q$, which maps formal loop expressions to the quotient space of physical predictions. The composed map $Q compose K : cal(L) arrow.r "Observables"$ admits a representation in which no intermediate quantity diverges: each observable is a convergent integral on $[0,1]$ or an algebraic function of kinematic variables. The one-loop self-energy (tadpole $A_0$) lies in $ker(Q)$ --- it is a mass renormalization with no observable content --- and is not computed. The $beta$-function coefficient $3 lambda^2 \/ (16 pi^2)$ is recovered from $integral_0^1 x(1-x) thin d x = 1\/6$, a trivially convergent integral. Forward scattering amplitudes in Minkowski signature below threshold are computed from $integral_0^1 d x thin ln(1 - rho thin x(1-x))$, which is smooth for $rho < 4$. Above threshold, the imaginary part is algebraic: $"Im" thin B_0 = pi sqrt(1 - 4 m^2 \/ s)$, encoding phase space opening at $s = 4m^2$. All 18 numerical results are verified by direct computation, and all 25 structural properties are machine-verified by the Z3 SMT solver, in the Kleis formal verification language. The conclusion: the UV divergences of textbook QFT arise from factoring $Q compose K$ into substeps that compute elements of $ker(Q)$ before projecting them away. The composed map has a convergent representative. The observables were always finite.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* phi-4 theory, one-loop, Feynman parameter integral, convergent integral, beta-function, renormalization, scattering amplitude, unitarity, projected ontology, formal verification, Z3]

#v(1em)


= Introduction

The standard account of one-loop renormalization in quantum field theory runs as follows: compute a loop integral, discover it diverges, regulate the divergence, expand in the regulator, subtract the pole, and obtain a finite answer that agrees with experiment [PS1995, tH1972]. The procedure is spectacularly successful --- and deeply puzzling. At no step does the practitioner explain what the infinity was, why it appeared, or what subtraction means physically.

Volume X of the Projected Ontology Theory (POT) series [AtikX] offered an algebraic interpretation: the $1\/epsilon$ pole in dimensional regularization is a coefficient at a *singular index* of a Laurent series, and the Hadamard admissibility projection extracts the admissible content without subtraction. This interpretation is correct --- but it still works within the dimensional regularization formalism. The present paper asks a sharper question:

*Can the observables be computed without ever encountering infinity?*

The answer is yes. Every observable prediction of $phi^4$ theory at one loop can be expressed as a convergent integral on the unit interval $[0,1]$ or as an algebraic function of kinematic variables. No dimensional regularization is invoked. No Laurent series is written. No pole is identified, and no subtraction is performed. The integrands are smooth and bounded everywhere on $[0,1]$; the integrals are evaluated by an adaptive Runge--Kutta solver (`ode45` in the Kleis language). The results reproduce the textbook predictions exactly.

The key insight is the distinction between *observables* and *bare parameters*:

+ The one-loop self-energy $Sigma = (lambda\/2) A_0(m^2)$ is a mass renormalization. The scalar tadpole $A_0$ diverges because the bare mass is not an observable. The physical mass is defined by measurement. Nothing needs to be computed, regulated, or subtracted.

+ The one-loop scattering amplitude corrections involve the scalar bubble $B_0(p^2; m^2, m^2)$. The observable is the *difference* $B_0(p_1^2) - B_0(p_2^2)$, which has a Feynman parameter representation as a smooth convergent integral on $[0,1]$.

+ The $beta$-function coefficient $3\/(16 pi^2)$ comes from $d B_0 \/ d rho|_(rho=0) = integral_0^1 x(1-x) thin d x = 1\/6$ --- a textbook calculus exercise.

+ Above threshold ($s > 4m^2$), the imaginary part is purely algebraic: $pi sqrt(1 - 4m^2\/s)$.

The structural claim is precise: the composed map $Q compose K : cal(L) arrow.r "Observables"$ --- where $K$ is the Feynman integral kernel and $Q$ is the observable projection --- admits a representation in which no intermediate quantity diverges. The divergence arises only when $K$ is factored into substeps that pass through non-observable intermediates. Section 8 makes this kernel--projection structure explicit.

The paper is organized as follows. Section 2 recalls the Lagrangian and Feynman rules. Section 3 explains why the tadpole is not computed (it lies in $ker(Q)$). Section 4 presents the Feynman parameter integrals and numerical results. Section 5 derives the $beta$-function. Section 6 computes Minkowski forward scattering amplitudes. Section 7 treats above-threshold physics. Section 8 makes the kernel--projection interpretation explicit. Section 9 states what matches textbook QFT, what is reinterpreted, and what is new.

= The Lagrangian and Feynman Rules

Consider $phi^4$ scalar field theory in four dimensions:

$ cal(L) = 1/2 (partial_mu phi)(partial^mu phi) - 1/2 m^2 phi^2 - lambda/(4!) phi^4 $

The Feynman rules follow uniquely from the Lagrangian:

$ "Propagator:" quad Delta(k) = i / (k^2 - m^2 + i epsilon) $
$ "Vertex:" quad V = -i lambda $

These are derived, not chosen. At one loop, two classes of diagrams contribute:

+ *Self-energy (2-point):* The tadpole diagram. One vertex, one propagator closing into a loop. Gives $Sigma = (lambda\/2) A_0(m^2)$, which is momentum-independent.

+ *Vertex correction (4-point):* Bubble diagrams in the $s$, $t$, $u$ channels. Each gives a contribution proportional to $B_0(p^2; m^2, m^2)$, which depends on the momentum transfer.

The self-energy involves the scalar one-point function $A_0$. The vertex correction involves the scalar two-point function $B_0$. Their treatment in this paper is fundamentally different.

= The Tadpole: Mass Renormalization, Not an Observable

The one-loop self-energy is:

$ Sigma = lambda / 2 dot A_0(m^2), quad A_0(m^2) = integral (d^4 k) / ((2 pi)^4) dot 1 / (k^2 + m^2) $

This integral diverges quadratically in $d = 4$. In dimensional regularization, it produces $A_0 = m^2\/(16 pi^2) [1\/epsilon + ("finite constants")]$.

We do not compute this. The reason is not evasion --- it is physics.

The self-energy $Sigma$ shifts the mass parameter: $m^2_("bare") + delta m^2 = m^2_("phys"))$, where $delta m^2 = (lambda\/2) A_0$. The bare mass $m_("bare")$ is a parameter of the Lagrangian with no independent physical meaning. The physical mass $m_("phys")$ is defined by where the propagator pole sits --- an experimental measurement.

The 'divergence' of $A_0$ is not a disease requiring subtraction. It is a signal: *the bare mass is not an observable*. Asking 'what is $A_0$?' is like asking 'what is the absolute potential?' in electrostatics. The question has no physical answer because only *differences* of potential are measurable. Insisting on an answer produces a coordinate-dependent infinity.

In the POT framework (made explicit in Section 8), this has a precise structural meaning. Let $Q$ denote the observable projection --- the map from the space of formal loop expressions to the space of physical predictions. Then $A_0 in ker(Q)$: the tadpole lies in the *null space* of the observable projection. This is not a choice to ignore $A_0$. It is a structural property: any shift $A_0 arrow.r A_0 + c$ can be absorbed into $m^2_("bare")$ without changing any observable prediction. The orbit of $A_0$ under scheme changes is entirely contained in $ker(Q)$.

Concretely: no cross-section, no $beta$-function coefficient, no threshold behavior, and no scattering amplitude difference depends on the value of $A_0$. Every observable we compute in Sections 4--7 is independent of $A_0$. That is what $A_0 in ker(Q)$ means operationally.

*The physical mass is an input.* In all subsequent computations, we use $m^2 = 1$ (natural units) as a measured parameter.

= Convergent Feynman Parameter Integrals

The observable one-loop corrections enter through the scalar bubble function $B_0(p^2; m^2, m^2)$. In Euclidean space with equal masses, the Feynman parameter representation gives the finite part:

$ F(rho) = integral_0^1 d x thin ln(1 + rho thin x(1-x)), quad rho = p_E^2 / m^2 $

This integral is:

+ *Defined on a compact domain* $[0,1]$.
+ *Smooth everywhere:* the argument $1 + rho x(1-x) > 1$ for all $rho > 0$ and $x in [0,1]$.
+ *Bounded:* the integrand satisfies $0 <= ln(1 + rho x(1-x)) <= ln(1 + rho\/4)$, with the maximum at $x = 1\/2$.
+ *Convergent by inspection:* a continuous function on a compact interval has a finite integral.

No regularization parameter appears. No $epsilon$, no $gamma_E$, no $ln(4 pi)$. The integral *is* the observable content of $B_0$.

We evaluate $F(rho)$ numerically by solving the initial-value problem $d y \/ d x = ln(1 + rho x(1-x))$, $y(0) = 0$, using the adaptive Runge--Kutta method `ode45`. The results at four momentum values:

#table(
    columns: 3,
    [*$rho = p_E^2 \/m^2$*], [*$F(rho)$*], [*Integrand bound*],
    [$0$], [$0$ (exact)], [$0$],
    [$1$], [$0.1520$], [$< ln(5\/4) = 0.223$],
    [$4$], [$0.4928$], [$< ln(2) = 0.693$],
    [$10$], [$0.9312$], [$< ln(3.5) = 1.253$],
)

The observable is the *difference* $Delta F(rho_1, rho_2) = F(rho_1) - F(rho_2)$, which can equivalently be computed as a single integral of a logarithmic ratio:

$ Delta F(rho_1, rho_2) = integral_0^1 d x thin ln ((1 + rho_1 x(1-x)) / (1 + rho_2 x(1-x))) $

We verify: $Delta F(4,1) approx 0.341$ from both methods, with numerical agreement to $6 times 10^(-5)$. The $B_0$ differences telescope: $F(10) - F(0) = [F(10) - F(4)] + [F(4) - F(1)] + [F(1) - F(0)]$ to machine precision ($10^(-16)$).

Verified in Examples 1--10 of `theories/pot_phi4_oneloop_worked.kleis`.

= The β-Function from a Convergent Integral

The one-loop $beta$-function in $phi^4$ theory governs the running of the coupling with energy:

$ beta(lambda) = (3 lambda^2) / (16 pi^2) $

The coefficient $3\/(16 pi^2)$ arises from three Mandelstam channels ($s$, $t$, $u$), each contributing the derivative of $B_0$ with respect to momentum. In the Feynman parameter representation:

$ (d F) / (d rho) = integral_0^1 d x thin (x(1-x)) / (1 + rho thin x(1-x)) $

This is another smooth, bounded, convergent integral on $[0,1]$. The integrand satisfies $0 <= x(1-x)\/(1 + rho x(1-x)) <= 1\/4$ for all $rho >= 0$.

At $rho = 0$ (the infrared renormalization point):

$ (d F) / (d rho) bar_(rho = 0) = integral_0^1 x(1-x) thin d x = 1/6 $

This is a textbook calculus exercise. Three channels $times 1\/6 = 1\/2$, giving $beta = lambda^2 \/ 2 dot 1\/(16 pi^2) dot 2 dot 3 = 3 lambda^2 \/ (16 pi^2)$. At $lambda = 0.1$: $beta approx 3.17 times 10^(-5)$.

The derivative $d F \/ d rho$ decreases monotonically with $rho$:

#table(
    columns: 2,
    [*$rho$*], [*$d F \/ d rho$*],
    [$0$], [$0.16667 = 1\/6$],
    [$1$], [$0.13917$],
    [$4$], [$0.09419$],
    [$10$], [$0.05810$],
)

Higher momentum $arrow.r$ weaker running. This is the infrared-free behavior characteristic of $phi^4$ theory (which, unlike non-abelian gauge theories, is not asymptotically free). The entire $beta$-function structure is extracted from convergent integrals.

Verified in Examples 11--12 of `theories/pot_phi4_oneloop_worked.kleis`.

= Minkowski Forward Scattering Amplitudes

The Euclidean integrals of Section 4 describe spacelike momentum transfer. For physical scattering at timelike momenta (below threshold, $s < 4m^2$), the Feynman parameter integral becomes:

$ F_("M") (s) = - integral_0^1 d x thin ln(1 - (s\/m^2) thin x(1-x)) $

For $s < 4m^2$, the argument $1 - (s\/m^2) x(1-x) > 0$ everywhere on $[0,1]$ (minimum at $x = 1\/2$ is $1 - s\/(4m^2) > 0$). The integral is smooth and convergent.

For forward scattering ($theta = 0$), the Mandelstam variables satisfy $t = 0$ and $u = 4m^2 - s$, so the one-loop amplitude correction involves:

$ Sigma_("channels") = F_("M")(s) + F_("M")(0) + F_("M")(4m^2 - s) $

At two energies ($m = 1$, natural units):

#table(
    columns: 4,
    [*Energy*], [*$F_("M")(s)$*], [*$F_("M")(u)$*], [*$Sigma$*],
    [$sqrt(s) = sqrt(2) m$], [$0.429$], [$0.429$], [$0.858$],
    [$sqrt(s) = sqrt(3) m$], [$0.791$], [$0.186$], [$0.977$],
)

The observable is the cross-section ratio between two energies:

$ sigma(sqrt(3) m) / sigma(sqrt(2) m) approx 1 + delta, quad delta approx -7.5 times 10^(-5) $

This is a concrete physical prediction: the forward scattering cross-section at $sqrt(s) = sqrt(3) m$ differs from that at $sqrt(s) = sqrt(2) m$ by a calculable one-loop correction, computed entirely from convergent integrals.

Verified in Examples 13--15 of `theories/pot_phi4_oneloop_worked.kleis`.

= Above-Threshold Physics: Unitarity and Phase Space

For $s > 4m^2$, the two scattered particles can go on shell in the loop. The Feynman parameter integrand $ln(m^2 - s x(1-x))$ passes through zero at:

$ x_(plus.minus) = (1 plus.minus sqrt(1 - 4m^2\/s)) / 2 $

Between $x_-$ and $x_+$, the argument is negative and the logarithm acquires an imaginary part $i pi$. The result is purely algebraic:

$ "Im" thin B_0(s) = pi sqrt(1 - (4 m^2) / s) $

This is not an integral --- it is a square root of a kinematic ratio. It encodes the phase space factor for two-particle production. Key values:

#table(
    columns: 2,
    [*$s \/m^2$*], [*$"Im" thin B_0$*],
    [$4$ (threshold)], [$0$ (exactly)],
    [$4.01$], [$0.157$],
    [$5$], [$1.405$],
    [$8$], [$2.221$],
    [$16$], [$2.721$],
)

The imaginary part opens continuously from zero at threshold $s = 4m^2$, increases monotonically, and approaches $pi$ as $s arrow.r infinity$. By the optical theorem, the imaginary part of the forward amplitude is proportional to the total cross-section:

$ sigma_("tot") tilde.op ("Im" thin M_("forward")(s)) / p_("cm") $

Unitarity is manifest: the imaginary part is a smooth, monotone function that vanishes exactly at threshold.

Verified in Examples 16--17 of `theories/pot_phi4_oneloop_worked.kleis`.

= Kernel--Projection Interpretation of the One-Loop Map

The preceding sections computed every one-loop observable of $phi^4$ theory from convergent integrals. This section names the structural operators that make the computation work, and identifies precisely where divergences live and why they are absent from the observable path.

=== The kernel $K$

The *kernel* $K$ is the Feynman integral operator. It maps the Lagrangian and a diagram topology to a Feynman parameter integrand:

$ K : (cal(L), Gamma) arrow.r.long f_(Gamma)(x; rho) $

For the bubble diagram with equal masses, $K$ produces the integrand $ln(1 + rho thin x(1-x))$ on $[0,1]$, parameterized by the external momentum ratio $rho = p^2\/m^2$. For the tadpole, $K$ produces $A_0(m^2)$, a momentum-independent constant that depends on the regularization.

$K$ encodes the loop structure: propagators, vertices, symmetry factors, momentum routing, and Feynman parametrization. It is determined entirely by the Lagrangian and the diagram topology.

=== The observable projection $Q$

The *observable projection* $Q$ maps the space of formal loop expressions to the space of physical predictions. It is defined by the equivalence relation: two loop expressions are equivalent if they yield the same values for all measurable quantities (cross-sections, decay rates, phase shifts).

Concretely, $Q$ acts as follows:

+ $Q(A_0) = 0$. The tadpole is in $ker(Q)$. Any shift $A_0 arrow.r A_0 + c$ is absorbed into the bare mass. No observable depends on $A_0$.

+ $Q(B_0(rho))$ is not a number --- it is an equivalence class. Individual $B_0(rho)$ values are scheme-dependent ($gamma_E$, $ln(4 pi)$ depend on the regularization). But the equivalence class $[B_0(rho)]$ determines all observables via *differences*.

+ $Q(B_0(rho_1) - B_0(rho_2)) = B_0(rho_1) - B_0(rho_2)$. Differences are scheme-independent. They are in $"im"(Q)$ --- they are observables.

+ $Q(d B_0 \/ d rho) = d B_0 \/ d rho$. The $beta$-function derivative is scheme-independent at leading order. It is in $"im"(Q)$.

+ $Q("Im" B_0(s)) = "Im" B_0(s)$. The imaginary part above threshold is scheme-independent and algebraic. It is in $"im"(Q)$.

=== The observable structure

The projection $Q$ organizes the space of one-loop expressions into two classes:

$ker(Q)$ contains $A_0$ and the scheme-dependent parts of individual $B_0$ values (the constants $gamma_E$, $ln(4 pi mu^2\/m^2)$ that depend on the renormalization scheme). These are the quantities that textbook QFT computes and then subtracts. They carry no physical information.

Representatives of $"im"(Q)$ contain $B_0$ differences, the $beta$-function, threshold behavior, and all cross-section predictions. These are the quantities we computed in Sections 4--7. Each has a convergent integral or algebraic representative.

We do not claim a canonical direct-sum splitting of the full space of loop expressions. The operative structure is the *quotient*: observables are equivalence classes under scheme changes, and $Q$ is the quotient map. What matters is that each equivalence class in $"im"(Q)$ has a convergent representative --- not that the decomposition is unique.

=== Why the composed map is finite

The textbook factorization passes through $ker(Q)$:

$ cal(L) arrow.r^(K_1) A_0(epsilon) arrow.r^(K_2) "Laurent series" arrow.r^("subtract") "finite value" $

Each step is well-defined, but the intermediate $A_0(epsilon)$ diverges as $epsilon arrow.r 0$. The divergence lives in $ker(Q)$ --- it is a scheme artifact that the subtraction removes.

The composed map $Q compose K$ bypasses $ker(Q)$ entirely:

$ cal(L) arrow.r^(Q compose K) integral_0^1 d x thin ln ((1 + rho_1 x(1-x)) / (1 + rho_2 x(1-x))) $

The right-hand side is a convergent integral on $[0,1]$. No intermediate diverges. The map $Q compose K$ admits a representation --- the Feynman parameter representation restricted to observable combinations --- in which no intermediate quantity diverges.

This is the precise claim. Not 'the infinities are removed,' but: *the map from Lagrangian to observables admits a representation that never passes through divergent intermediates. The divergences of textbook QFT arise from factoring this map into substeps that compute elements of $ker(Q)$ before projecting them away.*

=== Summary of the K--Q structure

#table(
    columns: 3,
    [*Quantity*], [*Space*], [*Representation*],
    [$A_0$], [$ker(Q)$], [Diverges in $d=4$. Not computed.],
    [Individual $B_0(rho)$], [Mixed], [Scheme-dependent. Not an observable by itself.],
    [$B_0(rho_1) - B_0(rho_2)$], [$"im"(Q)$], [Convergent integral on $[0,1]$.],
    [$d B_0 \/ d rho$], [$"im"(Q)$], [Convergent integral on $[0,1]$.],
    [$"Im" B_0(s)$], [$"im"(Q)$], [Algebraic: $pi sqrt(1-4m^2\/s)$.],
    [$sigma(s_1)\/sigma(s_2)$], [$"im"(Q)$], [Ratio of convergent integrals.],
)

Every entry in $"im"(Q)$ has a finite representation. Every divergence in textbook QFT lives in $ker(Q)$. The projection $Q$ is not a subtraction procedure --- it is the structural recognition that observables form a quotient space, and the quotient has a convergent representative.

= What Matches, What Changes, What Is New

=== What matches textbook $phi^4$

Every numerical result: the $beta$-function coefficient $3\/(16 pi^2)$, the Feynman parameter integrals, the forward scattering amplitudes, the threshold behavior, and the imaginary parts above threshold. No standard result is modified. The numbers are the same; the route to them is different.

=== What POT reinterprets

The *meaning* of the UV divergence. In standard QFT, the divergence of $A_0(m^2)$ is a disease that must be cured by adding counterterms. In POT, the divergence is a *signal* that the bare mass is not an observable. The physical mass is defined by measurement. The 'infinity' is not in the theory --- it is in the question.

More precisely: the standard procedure computes non-observable quantities ($A_0$, bare parameters) in $d = 4$, obtains infinities, and then cancels them by subtraction. The procedure presented here never computes non-observable quantities. It proceeds directly from the Lagrangian to the observable (Feynman parameter integrals, cross-section ratios) without passing through any divergent intermediate.

=== What POT claims beyond standard renormalization

Three things:

+ *The composed map $Q compose K$ is finite.* The map from Lagrangian to observables (Section 8) admits a representation --- Feynman parameter integrals restricted to $"im"(Q)$ --- in which no intermediate quantity diverges. The integrands are bounded on $[0,1]$. The integrals converge. No regularization is needed. The divergence is still 'there' in the sense that the factored route through $ker(Q)$ encounters it. But the composed route does not. The precise claim is not 'infinities are removed' but: *the map $Q compose K$ admits a convergent representative.*

+ *The divergence is in the factorization, not the theory.* The textbook route factors $Q compose K$ into substeps that compute elements of $ker(Q)$ --- the tadpole $A_0$, individual $B_0$ values with scheme-dependent constants --- before projecting them away by subtraction. This factorization passes through divergent intermediates. The composed operator does not. The divergence is an artifact of premature factorization.

+ *$Q$ is not a subtraction procedure.* In textbook QFT, renormalization is described as 'subtracting counterterms.' In the $K$--$Q$ framework (Section 8), $Q$ is a structural projection onto the quotient space of observables. It is not applied after the divergence appears; it determines which computations to perform in the first place. The elements of $"im"(Q)$ have convergent representatives. One need not construct a divergent expression in order to project it away.

+ *Universality.* The same strategy --- identify $"im"(Q)$, express its elements as Feynman parameter integrals, compute --- applies to every perturbative diagram in every renormalizable QFT. The Feynman parametrization is universal; only the Lagrangian parameters and diagram topology change. What is specific to this paper is $phi^4$; what is general is the principle that $"im"(Q)$ has convergent integral representatives.

=== Relationship to the Hadamard projection (Volumes VII and X)

Volume X of the POT series [AtikX] established the Hadamard admissibility projection as the structural mechanism for extracting observables from divergent Laurent series. Within the dimensional regularization formalism, the projection annihilates singular-index coefficients (poles) and preserves admissible-index coefficients (finite parts). This is a correct theorem about the Laurent representation, verified by Z3.

The present paper does not contradict Volume X --- it demonstrates its deepest implication. Volume VII stated: 'the divergence is in the factorization, not the theory.' Volume X proved that within dim reg, projection extracts the observable structurally. This paper shows that the dim reg formalism is *itself* unnecessary: the observables have direct convergent representations that never pass through a divergent intermediate.

The three results form a hierarchy of increasing strength:

+ *Volume VII:* Divergences are factorization artifacts (theoretical claim).
+ *Volume X:* Within dim reg, projection = MS subtraction (structural theorem).
+ *This paper:* Observables computable without dim reg at all (operational demonstration).

Each level entails the next. If observables have convergent integral representations (this paper), then any procedure that obtains the same values by factoring through a divergent intermediate must have a structural projection that extracts the finite part (Volume X), and the divergence must be an artifact of the factorization (Volume VII).

There is one refinement. Volume X applies the Hadamard projection to $A_0$ and obtains the finite part $c_0 = m^2\/(16 pi^2)(1 - gamma_E + ln(4 pi))$. This paper identifies $A_0$ as a mass renormalization and does not compute it. These are compatible: $c_0$ is the MS-scheme value of the self-energy, which contains the scheme-dependent constants $gamma_E$ and $ln(4 pi)$. It is finite but not physical --- it requires a renormalization condition (a measured mass) to become an observable. The Hadamard projection correctly extracts the MS-scheme answer; the present paper correctly observes that the MS-scheme answer is not the final observable.

The Hadamard projection is thus not wrong --- it is *scaffolding*. It served a necessary purpose: demonstrating that within the standard formalism, the extraction of observables is a structural projection, not an ad hoc subtraction. But once the deeper insight is established --- that observables have direct convergent representations --- the scaffolding can be removed. One need not construct a divergent intermediate in order to project it away.

The decisive sentence: *The map $Q compose K : cal(L) arrow.r "Observables"$ admits a representation in which no intermediate quantity diverges. The UV divergences of textbook QFT are artifacts of factoring this map into substeps that compute elements of $ker(Q)$ before projecting them away.*



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[PS1995\] Peskin, M. E. and Schroeder, D. V. *An Introduction to Quantum Field Theory.* Addison-Wesley (1995).]

#par(hanging-indent: 1.5em)[\[tH1972\] 't Hooft, G. and Veltman, M. Regularization and renormalization of gauge fields. *Nucl. Phys. B* 44, 189--213 (1972).]

#par(hanging-indent: 1.5em)[\[PV1979\] Passarino, G. and Veltman, M. One-loop corrections for $e^+ e^-$ annihilation into $mu^+ mu^-$ in the Weinberg model. *Nucl. Phys. B* 160, 151--207 (1979).]

#par(hanging-indent: 1.5em)[\[tHV1979\] 't Hooft, G. and Veltman, M. Scalar one-loop integrals. *Nucl. Phys. B* 153, 365--401 (1979).]

#par(hanging-indent: 1.5em)[\[AtikX\] Atik, E. Projection singularities: why physics has no infinities. Volume X of the POT VUFT Series. *Preprint* (2026).]

#par(hanging-indent: 1.5em)[\[AtikVII\] Atik, E. Renormalization as projected ontology: the theory that was never divergent. Volume VII of the POT VUFT Series. *Preprint* (2026).]


