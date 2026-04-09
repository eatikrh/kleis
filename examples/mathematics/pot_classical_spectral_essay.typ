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
  #text(size: 17pt, weight: "bold")[The Mass Gap as a Classical Spectral Phenomenon: A Reinterpretation]
  
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
      #text(size: 10pt)[We propose a reinterpretation of the Yang--Mills mass gap problem based on the conditional reduction established in Volume VIII of the Projected Ontology Theory (POT) program. The central observation is that the mass gap decomposes into two components of fundamentally different character: a _classical spectral mechanism_, expressed entirely through Sturm--Liouville theory, integral transform asymptotics, and inverse spectral analysis; and a _quantum realization problem_, requiring only a sign condition on an infrared anomalous dimension and the existence of a rigorous four-dimensional Yang--Mills theory. The classical component uses mathematics no later than 1946 (Watson, Weyl, Borg). No quantization, path integrals, gauge fixing, or particle physics appears in the mechanism. Quantum field theory contributes exactly one bit of information -- the sign of a number -- and one structural bridge. This separation reframes the mass gap from a purely quantum mystery into a problem of locating a classical mechanism within a quantum theory. The difficulty was never the mechanism. The difficulty is the realization.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* Yang-Mills mass gap, classical spectral theory, Sturm-Liouville, paradigm shift, mechanism vs realization, projected ontology]

#v(1em)


= Introduction

The Yang--Mills mass gap is traditionally framed as a fundamentally quantum phenomenon: a non-perturbative feature of gauge theory requiring the full machinery of quantum field theory. The Clay Mathematics Institute formulation [1] demands a rigorous 4D Yang--Mills construction satisfying Wightman axioms with a positive mass gap. Decades of work have approached the problem through lattice simulations [2], functional renormalization [3], topological models [4], and AdS/CFT duality.

This essay advances a different perspective, based on the conditional reduction established in the companion paper [5] and the broader Projected Ontology Theory (POT) program [6, 7]:

$ "mass gap" = underbrace("classical spectral mechanism", B + C + D) + underbrace("quantum realization", A + E). $

The key point is not that quantum field theory is irrelevant. It is that its role is _narrower than previously assumed_. The mechanism that produces a spectral gap is classical. The challenge is whether Yang--Mills theory realizes that mechanism.

This separation is not a technicality. It is a reinterpretation of where the problem lives.

= The Classical Spectral Mechanism

Consider a Sturm--Liouville operator on the half-line:
$ L = -d^2 / (d x^2) + V(x), quad x in (0, infinity), quad u(0) = 0. $

If $V(x) arrow +infinity$ as $x arrow infinity$, the Rellich--Molchanov theorem [8] implies: the spectrum is purely discrete, eigenvalues accumulate only at $+infinity$, and the lowest eigenvalue $lambda_1 > 0$. This is a _classical fact_. No quantum field theory is involved.

The reduction program [5] shows that a single scalar condition, $gamma > 0$, propagates through a chain of classical results:

$ gamma > 0 arrow.r.double beta = 1 + gamma > 1 arrow.r.double alpha = gamma > 0 arrow.r.double V(x) tilde x^(2 gamma) arrow +infinity arrow.r.double Delta > 0. $

Each step uses only classical mathematics:

== The ingredients

+ *Watson's lemma* (1918) [9]: transfers IR weight singularity to position-space kernel growth.

+ *Gauss hypergeometric equation* (1812) [10]: identifies the kernel as a Green's function of a Sturm--Liouville operator.

+ *Weyl semiclassical formula* (1911) [11]: connects potential growth to eigenvalue counting.

+ *Karamata's Tauberian theorem* (1930) [12]: inverts the spectral asymptotics.

+ *Abel inversion*: extracts the potential from the eigenvalue counting function.

+ *Borg uniqueness theorem* (1946) [13]: guarantees structural rigidity of the inverse spectral map.

+ *Rellich--Molchanov compactness* [8]: converts confining growth into discrete spectrum.

+ *Darboux transformations* (1882) [14]: generates the confining potential from a smooth superpotential.

The most recent result in this list is from 1946. The chain contains no Feynman diagrams, no path integrals, no gauge fixing, no second quantization, no operator algebras, no lattice discretization, no Monte Carlo simulation. The operator $L$ is a second-order ODE on a half-line with a Dirichlet condition. In the language of 19th-century physics, it is a vibrating string with a growing restoring force.

= Integral Transform Composition as Architecture

A central observation of the POT program, established in Volume VII [7], is that the relevant kernel is not merely computational -- it is _structural_.

The integral transform composition method (ITCM) constructs a transmutation operator $T_w = H_nu^(-1) compose M_w compose H_mu$ from Hankel transforms $H_mu, H_nu$ and a multiplication operator $M_w$. The resulting kernel
$ K(x, y) = integral_0^infinity w(k) J_mu (k x) J_nu (k y) k thin d k $
has $attach(, bl: 2) F_1$ (Gauss hypergeometric) structure inherited from the Sonine--Poisson--Delsarte transmutation [15].

Volume VII [7] established three structural facts:

+ The kernel acts as a _resolvent_: $K(x,y;z) = (L - z I)^(-1)(x,y)$ for a Sturm--Liouville operator $L$ determined by the transform composition (Theorem B, verified in `pot_theorem_b.kleis`).

+ The weight acts as a _spectral measure_: the IR behavior of $w(k)$ encodes the large-$x$ asymptotics of the potential $V$.

+ The Hankel-order asymmetry $mu eq.not nu$ is the _mass gap_: when $mu = nu$, the kernel reduces to the Bessel Green's function with continuous spectrum; when $mu eq.not nu$, the weight singularity forces a confining potential and discrete spectrum.

In this view, integral transform composition is not a technique applied to a problem. It is the _underlying architecture_ from which the operator, the potential, and the spectral gap all emerge. The Green's function is not imposed -- it is _recognized_.

= Where Quantum Field Theory Enters

Quantum field theory appears in exactly two places, and the reduction [5] is transparent about both.

== Input A: The anomalous dimension

The gluon propagator in Yang--Mills theory has an infrared anomalous dimension $gamma$. Three independent lines of evidence -- perturbative QCD [16], lattice QCD [2], and Dyson--Schwinger equations [3] -- support $gamma > 0$.

The reduction uses _only the sign_. Not the value. Not the perturbative expansion. Not the renormalization group flow. One bit of information from quantum field theory: $gamma > 0$.

This is the entire physics input to the classical mechanism.

== Input E: The structural bridge

The existence of a rigorous 4D Yang--Mills theory -- satisfying Wightman axioms [17] or the Osterwalder--Schrader reconstruction [18] -- whose two-point function induces the ITCM kernel and Sturm--Liouville operator identified above.

This is the unresolved constructive QFT problem. It is the content of the Clay Millennium Prize.

Crucially, the _entire mechanism_ connecting $gamma > 0$ to $Delta > 0$ is independent of E. The mechanism is complete without the bridge. What E provides is not the mechanism but the _realization_: the guarantee that Yang--Mills theory actually instantiates the classical spectral structure.

= Mechanism versus Realization

This leads to a conceptual separation that we believe is the central contribution of the POT program:

#align(center)[
#table(
  columns: (auto, auto),
  align: left,
  stroke: 0.5pt,
  inset: 8pt,
  table.header([*Question*], [*Domain*]),
  [_Why_ does a gap exist?], [Classical spectral theory (B+C+D)],
  [_Whether_ Yang--Mills exhibits it], [Quantum field theory (A+E)],
  [_How large_ is the gap?], [Classical (Airy eigenvalues) + physics ($gamma$ value)],
)
]

The mass gap is not intrinsically quantum in its _mechanism_. It is a classical spectral inevitability, contingent on a parameter supplied by QFT and a structural bridge that QFT must provide.

An analogy may clarify. Consider a violin string: a one-dimensional vibrating system with a fixed endpoint and a restoring force (tension). _Why does it produce discrete pitches?_ The mechanism is classical -- the eigenvalues of $-d^2 slash d x^2 + V(x)$ on an interval with Dirichlet boundary conditions are discrete. The _realization_ depends on having a physical string under tension. Nobody would call the discreteness of the harmonic series a "material science phenomenon" simply because a physical string is needed to realize it. The mechanism is spectral theory. The realization is physics. Our operator is precisely this: a one-dimensional Sturm--Liouville equation on a half-line with a Dirichlet condition at the origin. The confining potential $V(x) arrow infinity$ plays the role of tension. The mass gap is the fundamental frequency.

Similarly, the mass gap mechanism is spectral theory. The realization is quantum field theory. The historical difficulty of the mass gap problem arises, we suggest, from conflating the two.

= Implications for the Mass Gap Problem

If this reinterpretation is correct, it has consequences for how the field allocates intellectual effort.

== The problem is relocated, not solved

The Clay Millennium Problem asks for both the construction of a rigorous Yang--Mills theory _and_ a proof that it has a mass gap. The reduction separates these: the mass gap follows from the construction (via A+E) plus the classical scaffold (B+C+D). The open problem is therefore:

_Does a rigorous 4D Yang--Mills theory exist that realizes the classical spectral mechanism?_

This is a different question from: _Why does a gap exist once such a mechanism is present?_ The second question is answered by classical spectral theory. The first remains open.

== Decades of work addressed the frame, not the mechanism

Much of the literature on confinement -- dual superconductor models [19], center vortex models [20], Gribov copies [21] -- attempts to explain _why_ QCD confines. These are investigations of the _frame_ (how Yang--Mills theory realizes confinement) rather than the _mechanism_ (why confinement produces a gap once present).

The reduction suggests that the mechanism was always available in the classical literature. The spectral gap of a confining Sturm--Liouville operator is a theorem, not a conjecture. What was missing was not a calculation but a _viewpoint_: the recognition that the ITCM kernel carries Sturm--Liouville structure, and that Hankel-order asymmetry is the spectral gap in disguise.

== No new physics was needed

Every mathematical ingredient in the reduction predates quantum field theory. Watson's lemma (1918), Weyl's asymptotic law (1911), the Gauss hypergeometric function (1812), Darboux transformations (1882), Borg's uniqueness theorem (1946), Sturm--Liouville theory itself (1836). The anomalous dimension $gamma > 0$ is measured, not derived. The only genuinely quantum element is Assumption E -- the existence problem -- which is precisely the part left open.

The mass gap was never a quantum mystery. It was a classical spectral phenomenon that _appears_ in a quantum theory.

= Epistemological Shift

A secondary contribution is methodological. The reduction is built with explicit assumptions (A--E), labeled confidence levels (Level A through Level C/D), formal verification of logical steps (400+ Z3-checked examples across 14 Kleis theory files [22]), and explicit falsifiability conditions for each assumption.

This enforces a discipline where gaps are _identified_, not obscured; claims are conditional when they must be; and the reader knows exactly what would break the argument. The contribution is not only a chain of reasoning but a _transparent structure_ of reasoning.

The traditional mode of mathematical physics -- plausible narrative punctuated by "it can be shown that" -- is replaced by machine-verified logic with assumption tracking. Whether or not the mass gap reduction survives scrutiny, this methodology stands on its own: a demonstration that formal verification can discipline speculative mathematical physics without sterilizing it.

= Conclusion

The Yang--Mills mass gap problem can be reframed as:

$ "mass gap" = underbrace("classical spectral mechanism", "Sturm--Liouville, Watson, Weyl, Borg") + underbrace("quantum realization", "anomalous dimension + 4D existence"). $

The classical component is fully analyzable with established mathematics. The remaining difficulty lies in constructing and validating the quantum field theory that realizes it.

This does not solve the Clay Millennium Problem. It _relocates_ it. The mechanism that produces a spectral gap was never quantum. It was classical, available since the 1940s, distributed across spectral theory, asymptotic analysis, and inverse problems. What was missing was the architectural insight -- provided by the ITCM framework and the POT program -- that these classical pieces compose into a single pipeline from $gamma > 0$ to $Delta > 0$.

The mass gap was not hiding in quantum field theory. It was hiding in the spectral theory of second-order ODEs on a half-line. Quantum field theory is the _theater_ in which the gap appears. Classical spectral theory is the _script_.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] A. Jaffe and E. Witten, 'Quantum Yang--Mills theory,' Clay Mathematics Institute Millennium Problem description (2000).]

#par(hanging-indent: 1.5em)[\[2\] C. Morningstar and M. Peardon, 'The glueball spectrum from an anisotropic lattice study,' Phys. Rev. D 60 (1999), 034509.]

#par(hanging-indent: 1.5em)[\[3\] R. Alkofer and L. von Smekal, 'The infrared behavior of QCD Green's functions,' Phys. Reports 353 (2001), 281--465.]

#par(hanging-indent: 1.5em)[\[4\] G. 't Hooft, 'On the phase transition towards permanent quark confinement,' Nuclear Phys. B 138 (1978), 1--25.]

#par(hanging-indent: 1.5em)[\[5\] E. Atik, 'A conditional reduction of the Yang--Mills mass gap problem via integral transform composition,' preprint, Kleis Research (2026). Volume VIII of the POT series.]

#par(hanging-indent: 1.5em)[\[6\] E. Atik, 'Confinement as fiber non-invariance: the admissibility boundary in projected ontology,' preprint, Kleis Research (2026). Volume IV of the POT series.]

#par(hanging-indent: 1.5em)[\[7\] E. Atik, 'Renormalization as projected ontology: the theory that was never divergent,' preprint, Kleis Research (2026). Volume VII of the POT series.]

#par(hanging-indent: 1.5em)[\[8\] F. Rellich, 'Halbbeschrankte gewohnliche Differentialoperatoren zweiter Ordnung,' Math. Ann. 122 (1951), 343--368. A. M. Molchanov, 'On conditions for discreteness of the spectrum of self-adjoint differential equations of the second order,' Trudy Moskov. Mat. Obshch. 2 (1953), 169--199.]

#par(hanging-indent: 1.5em)[\[9\] G. N. Watson, 'A Treatise on the Theory of Bessel Functions,' 2nd ed., Cambridge University Press (1944).]

#par(hanging-indent: 1.5em)[\[10\] C. F. Gauss, 'Disquisitiones generales circa seriem infinitam,' Commentationes Societatis Regiae Scientiarum Gottingensis 2 (1812). See also: F. W. J. Olver et al., 'NIST Digital Library of Mathematical Functions,' Ch. 15.]

#par(hanging-indent: 1.5em)[\[11\] H. Weyl, 'Das asymptotische Verteilungsgesetz der Eigenwerte linearer partieller Differentialgleichungen,' Math. Ann. 71 (1912), 441--479.]

#par(hanging-indent: 1.5em)[\[12\] J. Karamata, 'Sur un mode de croissance reguliere. Theoremes fondamentaux,' Bull. Soc. Math. France 61 (1933), 55--62.]

#par(hanging-indent: 1.5em)[\[13\] G. Borg, 'Eine Umkehrung der Sturm--Liouvilleschen Eigenwertaufgabe,' Acta Math. 78 (1946), 1--96.]

#par(hanging-indent: 1.5em)[\[14\] G. Darboux, 'Sur une proposition relative aux equations lineaires,' C. R. Acad. Sci. Paris 94 (1882), 1456--1459.]

#par(hanging-indent: 1.5em)[\[15\] S. M. Sitnik and E. L. Shishkina, 'Transmutations, Singular and Fractional Differential Equations with Applications to Mathematical Physics,' Academic Press (2020).]

#par(hanging-indent: 1.5em)[\[16\] D. J. Gross and F. Wilczek, 'Ultraviolet behavior of non-abelian gauge theories,' Phys. Rev. Lett. 30 (1973), 1343--1346. H. D. Politzer, 'Reliable perturbative results for strong interactions?' Phys. Rev. Lett. 30 (1973), 1346--1349.]

#par(hanging-indent: 1.5em)[\[17\] R. F. Streater and A. S. Wightman, 'PCT, Spin and Statistics, and All That,' Princeton University Press (1964).]

#par(hanging-indent: 1.5em)[\[18\] K. Osterwalder and R. Schrader, 'Axioms for Euclidean Green's functions,' Comm. Math. Phys. 31 (1973), 83--112; 42 (1975), 281--305.]

#par(hanging-indent: 1.5em)[\[19\] S. Mandelstam, 'Vortices and quark confinement in non-abelian gauge theories,' Phys. Reports 23 (1976), 245--249. G. 't Hooft, 'Topology of the gauge condition and new confinement phases in non-abelian gauge theories,' Nucl. Phys. B 190 (1981), 455--478.]

#par(hanging-indent: 1.5em)[\[20\] L. Del Debbio, M. Faber, J. Greensite, and S. Olejnik, 'Center dominance and Z(2) vortices in SU(2) lattice gauge theory,' Phys. Rev. D 55 (1997), 2298--2306.]

#par(hanging-indent: 1.5em)[\[21\] V. N. Gribov, 'Quantization of non-abelian gauge theories,' Nucl. Phys. B 139 (1978), 1--19.]

#par(hanging-indent: 1.5em)[\[22\] Kleis: a formal verification language for mathematics and engineering. https://kleis.io]


