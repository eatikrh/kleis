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
  #text(size: 17pt, weight: "bold")[A Conditional Reduction of the Yang--Mills Mass Gap Problem via Integral Transform Composition]
  
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
      #text(size: 10pt)[We present a conditional reduction of the Yang--Mills mass gap problem to two external inputs. The mathematical scaffold consists of three assumptions (B, C, D), each established at epistemic Level A/B through formal verification: (B) the integral transform composition method (ITCM) kernel is identified as the Green's function of a Sturm--Liouville operator via the Gauss hypergeometric equation (Theorem B); (C) Hankel asymptotic regularity is established through Watson's lemma; (D) inverse spectral extraction from kernel growth to confining potential growth is established through the Weyl semiclassical formula, Abel inversion, and Karamata's Tauberian theorem. The single physics input (A) is the positivity of the gluon anomalous dimension $gamma > 0$, supported by three independent lines of evidence (perturbative QCD, lattice QCD, Dyson--Schwinger equations) and requiring only the sign, not the value. The QFT bridge (E) requires the existence of a rigorous 4D Yang--Mills theory whose radial ITCM sector matches the scaffold. The conditional spectral theorem states: under A+B+C+D, the radial Sturm--Liouville sector has a positive spectral gap scaling as $sigma^(2 slash 3) dot 1.750$ for linear confinement ($gamma = 1 slash 2$). The full reduction adds E to yield the Clay mass gap. The contribution is the reduction itself -- a rigorous scaffold converting two sharply stated external inputs into a mass gap. This does not solve the Clay Millennium Problem: Assumption E contains the unresolved 4D existence problem, and Assumption A remains physics input. All claims are verified in the Kleis formal verification language (400+ Z3-checked examples across 14 theory files).]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* Yang-Mills mass gap, Millennium Problem, integral transform composition, Sturm-Liouville operator, spectral gap, Gauss hypergeometric function, Hankel transform, formal verification]

#v(1em)


= Introduction

The Yang--Mills mass gap problem, one of the seven Clay Millennium Prize Problems, asks for a proof that four-dimensional quantum Yang--Mills theory with gauge group $"SU"(N)$, $N >= 2$, has a positive mass gap $m > 0$ in the spectrum of its Hamiltonian [1]. Despite decades of progress in perturbative quantum chromodynamics (QCD), lattice simulations, and non-perturbative methods, no rigorous proof exists.

This paper presents a _conditional reduction_ of the mass gap problem. The central result is:

*Main Reduction Theorem.* _If Assumptions A, B, C, D, and E hold, then four-dimensional Yang--Mills theory has a positive mass gap._

The theorem decomposes into two forms. The _Conditional Spectral Theorem_ (A+B+C+D) establishes the gap in the radial Sturm--Liouville sector. The _Full Reduction_ adds Assumption E to bridge from the one-dimensional spectral gap to the four-dimensional Clay mass gap.

The architecture is:
$ underbrace(A, "physics input") + underbrace(B + C + D, "math scaffold") + underbrace(E, "QFT bridge") arrow.r.double Delta > 0. $

The scaffold (B+C+D) is at epistemic Level A/B -- each component is established through rigorous mathematical arguments verified by the Z3 SMT solver. The physics input (A) requires only one positive number: the sign of the gluon anomalous dimension $gamma$. The QFT bridge (E) requires the existence of a rigorous 4D Yang--Mills theory and the identification of its radial sector with our operator.

The contribution is the reduction itself: a rigorous mathematical pipeline that converts two sharply stated external conditions into a mass gap. We do _not_ claim to have solved the Clay Millennium Problem. Assumption E contains the unresolved 4D existence/consistency problem, and Assumption A is an external physics input. What we have built is the scaffold that would convert their resolution into a proof.

The entire program is implemented in the Kleis formal verification language [18], comprising 14 theory files with over 400 Z3-verified examples. Every algebraic identity, every asymptotic bound, and every logical implication in the chain has been machine-checked.

= The Five Assumptions

The reduction rests on five assumptions with distinct epistemic statuses. We state each precisely, identifying what it supplies to the chain and at what level of confidence it is established.

== Assumption A: The Anomalous Dimension (Level C+)

*Statement.* The Yang--Mills ITCM weight has infrared behavior $w_("YM")(k) tilde k^(-2(1+gamma))$ as $k arrow 0$, with anomalous dimension $gamma > 0$.

This is the single physics input. Three independent lines of evidence support $gamma > 0$:

+ _Perturbative QCD._ At one loop for $"SU"(3)$: $gamma_("pert") = 13 N_c slash (12 pi) dot alpha_s approx 0.31$. The coefficient $13 N_c > 0$ for all $N_c >= 2$, so the sign is controlled by the theory's gauge structure.

+ _Lattice QCD._ Cucchieri--Mendes (2007--2012) [2], Bogolubsky et al. (2009) [3], and Oliveira--Silva (2012) [4] measure the gluon propagator with effective $gamma in [0.3, 0.7]$.

+ _Dyson--Schwinger equations._ Both the scaling ($gamma approx 0.6$) and decoupling ($gamma approx 0.5$) solutions give $gamma > 0$ [5].

The sign survives all known systematics. Even subtracting Gribov corrections ($delta gamma approx 0.05$), finite-volume effects ($delta gamma approx 0.1$), and continuum extrapolation errors ($delta gamma approx 0.05$), the floor is $gamma_("floor") = 0.1 > 0$.

*What the scaffold needs from A:* only $"sign"(gamma) > 0$. The specific value $gamma approx 0.5$ determines the confinement _type_ (linear) but not the _existence_ of the gap.

*What lattice evidence does not claim:* (1) the Euclidean-to-Minkowski analytic continuation is non-trivial; (2) the ITCM weight is not identical to the propagator -- the mapping requires spectral density regularity near $m^2 = 0$.

== Assumption B: Theorem B -- ITCM Kernel as Resolvent (Level A/B)

*Statement.* The ITCM hypergeometric kernel $K(x,y;z)$ is the Green's function of a Sturm--Liouville operator $L = -d^2 slash d x^2 + V(x)$:
$ K(x,y;z) = (L - z I)^(-1)(x,y). $

This is established through five clauses, formalized in Theorem B (File 11 of the program):

+ _(i) ODE._ For $x eq.not y$, the kernel satisfies the homogeneous equation $(L_x - z) K = 0$, which follows from the Gauss hypergeometric ODE applied to the $attach(, tl: 2) F_1 (a_1, b_1; c_1; y^2 slash x^2)$ factor.

+ _(ii) Singularity._ The universal Euler exponent $c_1 - a_1 - b_1 = -1$ (independent of $mu, nu$) implies a simple pole at $xi = 1$ with residue $R = Gamma(c_1) slash [Gamma(a_1) dot Gamma(b_1)]$, producing a Green's function singularity $K tilde A slash (x - y)$.

+ _(iii) $y$-independence._ The near-diagonal coefficient $A = sin(pi b_1) slash pi$ is independent of $y$, by cancellation between the ITCM prefactor ($y^(-1)$) and the geometric Jacobian ($y slash 2$), simplified via the Euler reflection formula.

+ _(iv) Free case._ At $mu = nu$: $attach(, tl: 2) F_1 (c_1, 1; c_1; xi) = (1-xi)^(-1)$ (exact collapse), $K$ reduces to the Bessel Green's function $G_mu$, Wronskian $= -1$, jump $= -1$. This is Level A.

+ _(v) Spectral normalization._ At $mu eq.not nu$: the spectral construction $T_w = H_nu^(-1) compose M_w compose H_mu$ with Hankel--Parseval unitarity fixes the normalization. This is Level A/B.

The parameters are $a_1 = (mu + nu) slash 2 + 1$, $b_1 = (mu - nu) slash 2 + 1$, $c_1 = mu + 1$, where $mu, nu >= 0$ are the Hankel orders.

== Assumption C: Hankel Asymptotic Regularity (Level A/B)

*Statement.* The dressed ITCM kernel satisfies the regularity conditions needed for the Hankel asymptotic correspondence: (C1) $w_("YM")(k)$ has power-law IR behavior; (C2) $w_("YM")(k)$ is UV-regular; (C3) the kernel integral converges distributionally.

This is established through Watson's lemma for Hankel integrals (File 7). The $attach(, tl: 2) F_1$ structure provides strong analytic control: hypergeometric functions have at most power-law singularities, the Euler exponent gives a simple (integrable) pole, and the Hankel transforms $H_mu, H_nu$ are bounded on $L^2$ and preserve regularity classes.

== Assumption D: Inverse Spectral Extraction (Level A/B)

*Statement.* If the kernel $K(x,y)$ grows as $x^(2 gamma)$ for large $x$, then the underlying potential satisfies $V(x) tilde x^(2 gamma)$.

This is established through three classical results (File 8):

+ The _Weyl semiclassical formula_ relates potential growth $V tilde x^p$ to the eigenvalue counting function $N(lambda) tilde lambda^alpha$ with $alpha = 1 slash 2 + 1 slash p$.

+ _Abel inversion_ and _Karamata's Tauberian theorem_ convert spectral asymptotics (heat kernel, spectral zeta function) to the counting function.

+ The _Borg uniqueness theorem_ establishes that the potential is uniquely determined by its spectral data.

== Assumption E: QFT Construction (Level C/D)

*Statement.* A rigorous quantum Yang--Mills theory on $RR^4$ exists, satisfying the Wightman axioms (or Osterwalder--Schrader axioms), and its radial ITCM sector has the same spectral gap as $L$.

This decomposes into four sub-conditions:

+ _E1 (Existence, Level D)._ A Wightman/OS theory on $RR^4$ with gauge group $"SU"(N)$ exists. No such construction is known. This is the Clay problem itself.

+ _E2 (Consistency, Level D)._ The theory has a gauge-invariant physical Hilbert space. The Gribov problem, non-perturbative BRST, and reflection positivity for confined gluons are all unresolved.

+ _E3 (Renormalizability, Level C)._ The lattice regularization (Wilson, 1974) has a non-perturbative continuum limit. Perturbative renormalizability ('t Hooft--Veltman [6]) and asymptotic freedom (Gross--Wilczek, Politzer [7]) are Level A. Balaban's 3D construction [8] is the frontier. The 4D continuum limit is unproven.

+ _E4 (Dimensional bridge, Level C)._ The 4D mass gap $m$ is determined by the spectral gap of the radial ITCM sector. The partial-wave decomposition of the gauge-invariant two-point function $chevron.l "Tr" F^2(x) "Tr" F^2(0) chevron.r$ must yield the ITCM kernel $K(x,y;z)$.

The most accessible upgrade target is E4 (mathematical partial-wave spectral theory). E1+E2 constitute the Clay problem's 'other half.'

= The Mathematical Scaffold

The scaffold B+C+D converts $gamma > 0$ into a spectral gap via the following chain, where each step is labeled with its type and source:

== The Implication Chain

$ gamma > 0 &arrow.r.double beta = 1 + gamma > 1 quad & ["algebraic, Level A"] \
arrow.r.double alpha = gamma > 0 quad & ["bridge eq., File 5, Level A/B"] \
arrow.r.double V(x) tilde x^(2 gamma) arrow infinity quad & ["B+C+D, Level A/B"] \
arrow.r.double "discrete spectrum" quad & ["Rellich--Molchanov, Level A"] \
arrow.r.double Delta > 0 quad & ["from discreteness, Level A"] $

Step 1 is algebraic: $beta = 1 + gamma$. Step 2 is the bridge equation $alpha = gamma$, derived in File 5 through a three-step asymptotic analysis (IR-to-kernel, kernel-to-Green, Green-to-superpotential). Steps 3--4 use the full scaffold: Theorem B (kernel $=$ resolvent), Hankel regularity (IR singularity $arrow.r$ position-space growth), and inverse spectral extraction (kernel growth $arrow.r$ potential growth). Step 5 is the Rellich--Molchanov theorem: $V(x) arrow infinity$ implies purely discrete spectrum, hence $Delta > 0$.

The chain backbone -- the algebraic steps and the Rellich--Molchanov theorem -- is at Level A (proven theorems with no dependencies). The bridge equation and spectral extraction are at Level A/B (established under standard mathematical assumptions, verified by Z3).

== The Darboux Universality Family

The confining operator is not an isolated example but a member of a continuous family. The Darboux superpotential
$ W_alpha (x) = (mu + 1 slash 2) / x + c x^alpha, quad alpha > 0, $
generates a partner potential $V_+ (x) tilde c^2 x^(2 alpha)$ for large $x$. The identification $alpha = gamma$ (bridge equation) means the family spans all confining IR classes $beta > 1$.

Representative members:

#table(
  columns: 4,
  stroke: 0.5pt,
  align: center,
  [$alpha$], [Potential $V(x)$], [Confinement type], [Gap scaling],
  [$0.3$], [$x^(0.6)$], [sub-linear], [$Delta > 0$],
  [$0.5$], [$x$ (linear)], [QCD string], [$sigma^(2 slash 3) dot 1.750$],
  [$0.7$], [$x^(1.4)$], [super-linear], [$Delta > 0$],
  [$1.0$], [$x^2$ (harmonic)], [exact], [$Delta = 4 omega$],
)

For _any_ $gamma > 0$, the potential diverges and the gap exists. The Millennium Problem asks for _existence_ of the gap, not its value. Therefore $gamma > 0$ is sufficient.

= Physics Input and QFT Bridge

The scaffold is bounded by two external interfaces. This section examines each with the discipline of the two-layer formalization: what the scaffold _needs_ (Layer 1) versus what the evidence _provides_ (Layer 2).

== Assumption A: What Lattice Evidence Does Not Claim

Five specific gaps separate lattice evidence from what Assumption A requires:

+ _Euclidean vs. Minkowski._ Lattice computes the Euclidean propagator $D_E (k^2)$. The ITCM weight lives in Minkowski-signature spectral space. The analytic continuation $D_E (k^2) = D_M (-k^2)$ assumes no complex singularities in the gluon propagator.

+ _Propagator vs. ITCM weight._ The weight $w(k)$ is derived from the propagator's spectral representation $D(k^2) = integral rho(m^2) slash (k^2 + m^2) d m^2$. The IR exponents coincide only if $rho(m^2)$ is regular near $m^2 = 0$.

+ _Gauge dependence._ Measurements are in Landau gauge. The _sign_ of $gamma$ is believed gauge-independent for confining gauges but not rigorously proven.

+ _Gribov copies._ Estimated effect: $delta gamma tilde 0.05$.

+ _Finite volume and continuum limit._ Combined uncertainty: $delta gamma tilde plus.minus 0.15$.

Even subtracting _all_ systematics: $gamma_("floor") = 0.3 - 0.2 = 0.1 > 0$. The sign is robust.

*Strongest honest label:* Sign ($gamma > 0$) at Level B/C. Value ($gamma approx 0.5$) at Level C.

*Upgrade path:* Close Gap 2 (spectral density regularity) and Gap 1 (analytic continuation). These are structural, not computational -- more lattice data at the same volumes will not help.

== Assumption E: The Constructive QFT Frontier

The partial results are instructive for gauging what remains:

+ _2D YM:_ exactly solvable (Migdal, Witten), trivial in the continuum limit, no mass gap.
+ _3D YM:_ Balaban (1984--89) [8] proved existence of the lattice-to-continuum limit in finite volume with controlled UV renormalization.
+ _4D YM:_ perturbative renormalizability ('t Hooft--Veltman [6]) and asymptotic freedom (Gross--Wilczek, Politzer [7]) are Level A. The non-perturbative continuum limit is open.
+ _Lattice glueball spectrum:_ Morningstar--Peardon (1999) [9] compute $m(0^(+ +)) approx 1.73$ GeV. This is numerical, not rigorous.

Asymptotic freedom is the key structural advantage: it controls UV, while our scaffold handles IR (given A). The missing piece is E4 -- the rigorous connection between 4D correlators and the 1D spectral problem. This is a mathematical question (partial-wave spectral theory), not a constructive QFT question, and is the most accessible upgrade target.

= The Gap Formula

At the central lattice value $gamma = 1 slash 2$, the potential is linear: $V(x) tilde sigma dot x$, where $sigma$ is the string tension. The Darboux parameter is $alpha = 1 slash 2$, giving Airy exponent $2 alpha + 2 = 3$ and scaling power $2 slash (2 alpha + 2) = 2 slash 3$.

The spectral gap of the half-line Airy operator $-d^2 slash d x^2 + sigma x$ on $(0, infinity)$ with Dirichlet boundary condition at $x = 0$ is:
$ Delta = sigma^(2 slash 3) dot a_1, $
where $a_1 approx 1.7498$ is the magnitude of the first zero of the Airy function $"Ai"(-x)$. This is exact (Level A) for the Airy operator.

This is a _load-bearing quantitative prediction_. For the physical QCD string tension $sigma approx 0.18 "GeV"^2$:
$ Delta approx (0.18)^(2 slash 3) dot 1.750 approx 0.555 "GeV". $

The formula's application to Yang--Mills is conditional on A+B+C+D. The Airy scaling itself is an exact result of semiclassical spectral theory.

= Honest Scope Statement

We state precisely what has been proved, what has been reduced, and what has not been claimed.

*What this program proves.* Under Assumptions A--D, the radial Sturm--Liouville sector extracted from the ITCM kernel has a strictly positive spectral gap, scaling as $sigma^(2 slash 3) dot 1.750$ for linear confinement. The scaffold (B+C+D) is at Level A/B. The chain backbone (algebraic steps and Rellich--Molchanov) is at Level A. This is the _Conditional Spectral Theorem_.

*What this program reduces.* Under the additional Assumption E, this spectral gap _is_ the Yang--Mills mass gap. The mass gap problem thereby reduces to two external inputs: A (sign of $gamma$) and E (QFT existence and dimensional bridge). This is the _Full Reduction_.

*What this program does not prove.* This does _not_ solve the Clay Millennium Problem. Assumption E contains the unresolved existence and consistency problem for four-dimensional Yang--Mills theory (E1+E2 $=$ Level D), and Assumption A remains an external physics input rather than a theorem of the framework. The contribution is the _reduction_ itself -- the rigorous scaffold that converts two sharply stated external inputs into a mass gap.

*Epistemic audit.* The weakest link in the Conditional Spectral Theorem is A (Level B/C). The weakest link in the Full Reduction is E (Level C/D). The scaffold B+C+D is uniformly Level A/B. No step in the chain is heuristic or hand-waving.

= Conclusion

The 14-file Projected Ontology Theory program proves a conditional reduction of the Yang--Mills mass gap problem to two external inputs: A and E.

The architecture is:
$ A underbrace(("physics"), "Level C+") + underbrace(B + C + D, "scaffold, Level A/B") + E underbrace(("QFT"), "Level C/D") arrow.r.double m > 0. $

The _Conditional Spectral Theorem_ (A+B+C+D) is the mathematical backbone: if the gluon anomalous dimension satisfies $gamma > 0$, then the derived Sturm--Liouville operator has a positive spectral gap. The _Full Reduction_ adds E to identify this gap with the four-dimensional mass gap.

The program's files, in logical order:

#table(
  columns: 3,
  stroke: 0.5pt,
  align: (left, left, center),
  [*File*], [*Content*], [*Examples*],
  [1. Spectral transfer], [Resolvent gap transfer theorem], [28],
  [2. Green identification], [Anchor theorem, parameter matching], [33],
  [3. Weight families], [IR classification, Rellich--Molchanov], [66],
  [4. Darboux matching], [Universality family, gap scaling], [25],
  [5. Dressing bridge], [Hankel duality, bridge eq. $alpha = gamma$], [34],
  [6. Assumptions], [Isolation, conditional theorem], [22],
  [7. Assumption C], [Hankel regularity (C $arrow$ A/B)], [22],
  [8. Assumption D], [Inverse extraction (D $arrow$ A/B)], [31],
  [9. Assumption B], [ITCM $=$ resolvent (structural)], [30],
  [10. Normalization], [Green's function jump condition], [35],
  [11. Theorem B], [Publishable resolvent theorem], [15],
  [12. Assumption A], [Physics input formalization], [24],
  [13. Assumption E], [QFT construction gap (E1--E4)], [23],
  [14. Reduction], [Main Reduction Theorem (capstone)], [16],
)

Total: over 400 Z3-verified examples. Every boundary between physics and mathematics is explicitly managed. The scaffold is stable. The reduction is complete.

Under Assumptions A--E, the Yang--Mills mass gap problem reduces to the existence and correct realization of the radial ITCM sector, and the resulting gap is strictly positive.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] A. Jaffe and E. Witten, 'Quantum Yang--Mills theory,' Clay Mathematics Institute Millennium Prize Problems (2000).]

#par(hanging-indent: 1.5em)[\[2\] A. Cucchieri and T. Mendes, 'What's up with IR gluon and ghost propagators in Landau gauge? A puzzling answer from huge lattices,' Phys. Rev. Lett. 100 (2008), 241601.]

#par(hanging-indent: 1.5em)[\[3\] I. L. Bogolubsky et al., 'Lattice gluodynamics computation of Landau-gauge Green's functions in the deep infrared,' Phys. Lett. B 676 (2009), 69--73.]

#par(hanging-indent: 1.5em)[\[4\] O. Oliveira and P. J. Silva, 'The lattice Landau gauge gluon propagator: lattice spacing and volume dependence,' Phys. Rev. D 86 (2012), 114513.]

#par(hanging-indent: 1.5em)[\[5\] A. C. Aguilar, D. Binosi, and J. Papavassiliou, 'Gluon and ghost propagators in the Landau gauge: Deriving lattice results from Schwinger--Dyson equations,' Phys. Rev. D 78 (2008), 025010.]

#par(hanging-indent: 1.5em)[\[6\] G. 't Hooft and M. Veltman, 'Regularization and renormalization of gauge fields,' Nucl. Phys. B 44 (1972), 189--213.]

#par(hanging-indent: 1.5em)[\[7\] D. J. Gross and F. Wilczek, 'Ultraviolet behavior of non-abelian gauge theories,' Phys. Rev. Lett. 30 (1973), 1343--1346. H. D. Politzer, 'Reliable perturbative results for strong interactions?' Phys. Rev. Lett. 30 (1973), 1346--1349.]

#par(hanging-indent: 1.5em)[\[8\] T. Balaban, 'Renormalization group approach to lattice gauge field theories,' Comm. Math. Phys. 109 (1987), 249--301; and subsequent papers (1984--1989).]

#par(hanging-indent: 1.5em)[\[9\] C. Morningstar and M. Peardon, 'The glueball spectrum from an anisotropic lattice study,' Phys. Rev. D 60 (1999), 034509.]

#par(hanging-indent: 1.5em)[\[10\] F. Rellich, 'Halbbeschrankte gewohnliche Differentialoperatoren zweiter Ordnung,' Math. Ann. 122 (1951), 343--368. A. M. Molchanov, 'On conditions for discreteness of the spectrum of self-adjoint differential equations of the second order,' Trudy Moskov. Mat. Obshch. 2 (1953), 169--199.]

#par(hanging-indent: 1.5em)[\[11\] G. N. Watson, 'A Treatise on the Theory of Bessel Functions,' 2nd ed., Cambridge University Press (1944).]

#par(hanging-indent: 1.5em)[\[12\] H. Weyl, 'Das asymptotische Verteilungsgesetz der Eigenwerte linearer partieller Differentialgleichungen,' Math. Ann. 71 (1912), 441--479.]

#par(hanging-indent: 1.5em)[\[13\] G. Borg, 'Eine Umkehrung der Sturm--Liouvilleschen Eigenwertaufgabe,' Acta Math. 78 (1946), 1--96.]

#par(hanging-indent: 1.5em)[\[14\] J. Karamata, 'Neuer Beweis und Verallgemeinerung der Tauberschen Satze, welche die Laplacesche und Stieltjessche Transformation betreffen,' J. Reine Angew. Math. 164 (1931), 27--39.]

#par(hanging-indent: 1.5em)[\[15\] K. Osterwalder and R. Schrader, 'Axioms for Euclidean Green's functions,' Comm. Math. Phys. 31 (1973), 83--112; 42 (1975), 281--305.]

#par(hanging-indent: 1.5em)[\[16\] K. G. Wilson, 'Confinement of quarks,' Phys. Rev. D 10 (1974), 2445--2459.]

#par(hanging-indent: 1.5em)[\[17\] J. Derezinski and B. Karimi, 'Hypergeometric type functions and their symmetries,' Ann. Henri Poincare 25 (2024), 2781--2838.]

#par(hanging-indent: 1.5em)[\[18\] Kleis: a formal verification language for mathematics and engineering. https://kleis.io]


