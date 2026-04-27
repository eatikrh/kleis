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
  #text(size: 17pt, weight: "bold")[Non-Unique Factorization of Projection Kernels]
  
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
      #text(size: 10pt)[The Projected Ontology Theory (POT) framework maps configurations to observables through kernel operators. In the existing POT literature, the process from configuration to observable has been factored in two distinct ways: the three-factor decomposition $K = K_("univ") compose K_("dyn") compose K_("rep")$ (used for electrodynamics, quantum entanglement, and flat rotation curves) and the two-step pipeline $K arrow Q$ (used for general relativity and perturbative QFT). We show that these two architectures are not interchangeable --- which factorization is available depends on a single structural predicate: the admissibility of the total kernel. Admissible kernels inhabit a factorial sector where multiple factorizations exist with varying numbers of factors, analogous to factorization in Dedekind domains with non-trivial class group. Non-admissible kernels inhabit an atomic sector where the self-coupling term ($omega and omega$ in GR, $A and A$ in Yang--Mills) acts as an irreducible atom that obstructs decomposition into admissible components. The obstruction theorem is sharp: if $K$ is non-admissible, then no decomposition $K = K_1 compose dots.h compose K_n$ with all $K_i$ admissible exists, because composition of admissible kernels is admissible. The formulation fiber from the GR paper receives a factorization-theoretic interpretation: physically equivalent formulations (Cartan, teleparallel, Palatini) factor the same non-admissible pipeline differently, placing the irreducible atom in $K$ or $Q$ depending on the formulation --- a factorization rearrangement that preserves the total pipeline while varying the decomposition. We formalize these results on the Kleis verification platform with 35 Z3-verified propositions, connecting the POT kernel monoid to the non-unique factorization theory of Geroldinger and Halter-Koch, the Landau example for linear partial differential operators, and categorical factorization systems.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* projection kernel, non-unique factorization, admissible kernel, composition monoid, factorization obstruction, formulation fiber, Geroldinger--Halter-Koch, Landau example, categorical factorization, Projected Ontology Theory, formal verification, Z3]

#v(1em)


= Introduction

The Projected Ontology Theory (POT) framework describes physical observables as projections of flows through kernel operators. A configuration (tetrad, gauge field, spinor state) enters a production kernel $K$, which maps it to an intermediate object (curvature, field strength, correlation); an observable projection $Q$ then extracts the measurable quantity. The pipeline $Q compose K$ is the physical theory.

In the existing POT papers, this pipeline has been factored in two distinct ways.

*Architecture 1.* In the electrodynamics, quantum entanglement, and flat rotation curves papers, the production kernel $K$ is further decomposed as $K = K_("univ") compose K_("dyn") compose K_("rep")$: a universal geometric factor, a dynamical factor, and a representation factor. All three are admissible (linear, zero-preserving, composition-closed), and their composition is therefore also admissible.

*Architecture 2.* In the general relativity (GR) and perturbative QFT papers, the pipeline is factored as a two-step process $K arrow Q$: a production kernel $K$ (the Cartan curvature formula $R = d omega + omega and omega$) followed by an observable projection $Q$ (the Ricci trace contraction). The production kernel is non-admissible.

These are not merely different notational conventions. They represent genuinely different decompositions of the process from configuration to observable, and the question of *which factorization is available for a given physical theory* has not been addressed.

This paper answers that question. The answer is controlled by a single structural predicate: the admissibility of the total kernel. Admissible kernels admit multiple factorizations with varying numbers of factors --- they live in a *factorial sector* with rich decomposition freedom. Non-admissible kernels contain an irreducible self-coupling term (the $omega and omega$ in GR, the $A and A$ in Yang--Mills) that obstructs decomposition into admissible components --- they live in an *atomic sector* where the factorization is constrained.

The obstruction theorem is sharp: composition of admissible kernels is admissible (this is the closure axiom from the POT foundations), so if the total kernel is non-admissible, at least one factor must be non-admissible. This non-admissible factor is an *atom* in the factorization-theoretic sense: it cannot be further decomposed.

The formulation fiber from the GR paper receives a precise interpretation in this language. Physically equivalent formulations of GR (Cartan, teleparallel, Palatini) produce the same total pipeline but factor it differently: Cartan places the non-admissible atom in $K$, teleparallel places it in $Q$. This is a *factorization rearrangement* --- the atom moves between factors without changing the physical content.

The connection to pure mathematics is direct. Non-unique factorization theory, initiated by Carlitz in 1960 and developed systematically by Geroldinger and Halter-Koch, studies rings and monoids where unique factorization into irreducibles fails. The Landau example for linear partial differential operators shows the same operator factored into 2 or 3 factors. Categorical factorization systems formalize the conditions under which morphism decompositions are unique (orthogonal systems) or non-unique (weak systems). The POT kernel monoid sits at the intersection of all three traditions.

All results are formalized on the Kleis verification platform and verified by the Z3 SMT solver. The theory file `pot_kernel_factorization.kleis` contains 35 verified propositions covering the monoid structure, the admissibility obstruction, the factorization dichotomy, the formulation fiber, and the transfer homomorphism.

= The Kernel Composition Monoid

The POT framework is built on a type $cal(K)$ of kernel operators, equipped with a composition operation $compose$ and a distinguished identity element $"id"$. A kernel $K in cal(K)$ maps flows to fields: $K : "Flow" arrow "Field"$. The composition $K_1 compose K_2$ is the kernel whose application is the sequential application of $K_2$ followed by $K_1$.

#strong[Definition 1] (Kernel monoid). The pair $(cal(K), compose)$ is a monoid:

$ K compose "id" = "id" compose K = K #h(2em) "(identity)" $
$ (K_1 compose K_2) compose K_3 = K_1 compose (K_2 compose K_3) #h(2em) "(associativity)" $

A kernel $K$ is *admissible* if it is linear ($K(a + b) = K(a) + K(b)$), respects scalar multiplication ($K(c dot a) = c dot K(a)$), and maps zero to zero ($K(0) = 0$). The admissible kernels form a submonoid $cal(K)_("adm") subset cal(K)$: the identity is admissible, and composition preserves admissibility (the *closure axiom*):

$ "is_admissible"(K_1) and "is_admissible"(K_2) arrow.r.double "is_admissible"(K_1 compose K_2) $

This closure property is the foundation of both Architecture 1 (where all three factors are admissible, so the total is admissible) and the obstruction theorem (where a non-admissible total cannot arise from all-admissible factors).

The closure axiom is verified as propositions M1--M3 and A1--A2 in the theory file: associativity, identity properties, admissibility of the identity, and composition closure for both 2-factor and 3-factor decompositions.

= Two Factorization Architectures

#strong[Architecture 1: Three-factor decomposition.] In the entanglement paper (Volume II) and the electrodynamics paper (Volume III), the production kernel is decomposed as

$ K = K_("univ") compose K_("dyn") compose K_("rep") $

where $K_("univ")$ encodes universal geometry (the Cartan pipeline, exterior derivative structure), $K_("dyn")$ encodes the dynamics specific to the sector (Schrödinger evolution, Maxwell equations), and $K_("rep")$ encodes the representation (spin-1/2, 1-form, scalar). Each factor is declared admissible, and the composition closure axiom guarantees the total $K$ is admissible.

This architecture is formalized in `pot_entanglement_v2.kleis` as the `KernelFactorization` structure with axioms E1--E4: each factor is admissible, and `unified_kernel = compose_kernel(K_univ, compose_kernel(K_dyn, K_rep))`.

#strong[Architecture 2: K--Q pipeline.] In the GR projection kernel paper (Volume IV) and the perturbative QFT papers, the pipeline is

$ "configuration" arrow.r^K "intermediate" arrow.r^Q "observable" $

where $K$ is the production kernel (Cartan curvature $R = d omega + omega and omega$, Feynman propagator, etc.) and $Q$ is the observable projection (Ricci trace, S-matrix element, etc.). The intermediate object (Riemann tensor, propagator) is not directly observable --- only its image under $Q$ is.

In this architecture, $K$ need not be admissible. For full GR, $K$ is non-admissible (the $omega and omega$ self-coupling breaks linearity). For linearized GR, $K$ is admissible (the $omega and omega$ term vanishes). The projection $Q$ may be admissible (Ricci contraction in Cartan) or non-admissible (torsion scalar in teleparallel).

The key observation: Architecture 1 requires all factors to be admissible. Architecture 2 allows non-admissible factors. The question is: what determines which architecture applies?

= The Admissibility Obstruction Theorem

#strong[Theorem 1] (Admissibility obstruction). _If $K$ is non-admissible, then no decomposition $K = K_1 compose dots.h compose K_n$ exists with all $K_i$ admissible._

#strong[Proof.] By the closure axiom, the composition of admissible kernels is admissible. If all $K_i$ are admissible, then $K_1 compose dots.h compose K_n$ is admissible. But $K$ is non-admissible. Contradiction. $square$

Despite its simplicity, this theorem has far-reaching consequences for the structure of physical theories.

#strong[Corollary 1] (Non-admissible atom). _Every non-admissible kernel $K$ contains at least one irreducible non-admissible factor --- an atom in the factorization-theoretic sense._

An atom is a kernel that admits no non-trivial factorization: it cannot be decomposed into non-identity factors. The $omega and omega$ term in GR and the $A and A$ term in Yang--Mills are such atoms. They are the irreducible building blocks of non-admissibility.

#strong[Corollary 2] (Architecture selection). _Architecture 1 (three-factor with all admissible) is available if and only if the total kernel is admissible. Architecture 2 (K--Q pipeline with non-admissible K) is forced when the total kernel is non-admissible._

This explains the empirical pattern in the POT literature: electrodynamics, entanglement, and flat rotation curves use Architecture 1 because their kernels are admissible; GR and Yang--Mills use Architecture 2 because their kernels are non-admissible.

The obstruction and its consequences are verified as propositions A1--A5 and K1--K6 in the theory file.

= The Factorization Dichotomy

The obstruction theorem partitions the kernel monoid $cal(K)$ into two sectors:

*Factorial sector* $cal(K)_("fac")$: admissible kernels. These admit multiple factorizations with varying numbers of factors. The exterior derivative $d$ on 1-forms can be viewed as a single operator (factorization length 1), as the composition $K_("univ") compose K_("em")$ (length 2), or as $K_("univ") compose K_("dyn") compose K_("rep")$ (length 3). The maximum factorization length exceeds 1. This is the POT analog of factorization in a Dedekind domain with non-trivial class group, where the same element admits factorizations of different lengths.

*Atomic sector* $cal(K)_("atom")$: non-admissible kernels. These contain an irreducible non-admissible atom. The atom has factorization length 1 (it cannot be decomposed). The full GR kernel $K_("GR") = R = d omega + omega and omega$ and the Yang--Mills kernel $K_("YM") = d A + A and A$ are themselves atoms: they are irreducible in the admissible factorization monoid.

#strong[Proposition 1] (Sectors are exhaustive and disjoint).
$ cal(K) = cal(K)_("fac") union.sq cal(K)_("atom") $
_Every kernel belongs to exactly one sector. Membership is determined by the admissibility predicate._

The dichotomy is verified as propositions D1--D5 in the theory file. Proposition D4 establishes exhaustiveness; D5 establishes that atomic sector membership implies the presence of a non-admissible atom.

The dichotomy connects to the classification results in non-unique factorization theory. In the language of Geroldinger and Halter-Koch, the factorial sector contains elements with *sets of lengths* $L(K)$ having $|L(K)| > 1$ (multiple factorization lengths), while atoms have $L(K) = {1}$. The *elasticity* $rho(K) = max L(K) slash min L(K)$ measures the degree of non-uniqueness: $rho = 1$ for atoms (unique length), $rho > 1$ for factorial elements.

= The Landau Analogy

The factorization dichotomy in POT kernels has a precise parallel in the theory of linear partial differential operators (LPDOs). The Landau example demonstrates that the same LPDO can be factored in fundamentally different ways:

$ L = (D_x + 1 + frac(1, x + c(y))) compose (D_x + 1 - frac(1, x + c(y))) compose (D_x + x D_y) $
$ = (D_(x x) + x D_(x y) + D_x + (2 + x) D_y) compose (D_x + 1) $

The first factorization has *three factors*; the second has *two*. The second-order factor in the two-factor decomposition is irreducible --- it cannot be further decomposed. This is exactly the POT pattern: the same physical pipeline (total operator $L$) admits a three-factor and a two-factor decomposition, with the two-factor version containing an irreducible component.

Shemyakova (2010) proved a refinement theorem for LPDOs: if an operator $L$ has a left factor $F_1$ and a right factor $F_2$ with coprime principal symbols ($ gcd("Sym"(F_1), "Sym"(F_2)) = 1$), then a three-factor decomposition $L = F_1 compose M compose F_2$ exists. Conversely, when coprimality fails, the refinement may not exist --- the two-factor decomposition may be the *only* one available.

The POT analog: the three-factor Architecture 1 exists when the component kernels operate on "coprime" domains (universal geometry, dynamics, representation are structurally independent). Architecture 2 (K--Q) is forced when the production and projection cannot be further decomposed --- when the non-admissible atom resists refinement.

The Landau example shows that non-unique factorization of operators is not an anomaly but a structural feature of composition monoids that lack unique factorization domains. The POT kernel monoid exhibits the same phenomenon, but with a physical interpretation: the choice of factorization determines which intermediate quantities are visible (K_dyn or K_rep in Architecture 1; the intermediate curvature in Architecture 2).

= The Formulation Fiber as Factorization Rearrangement

The GR projection kernel paper established the *formulation fiber*: physically equivalent formulations of GR (Cartan, teleparallel, Palatini, spin-2) produce the same physical predictions but differ in structural properties such as energy localizability. Properties that vary across physically equivalent formulations are *fiber artifacts*.

The factorization-theoretic interpretation is now clear. Each formulation defines a different factorization of the *same* total pipeline:

#table(
    columns: (auto, auto, auto, auto),
    inset: 6pt,
    [*Formulation*], [*K admissible?*], [*Q admissible?*], [*Atom location*],
    [Cartan], [No ($omega and omega$)], [Yes (trace)], [In $K$],
    [TEGR], [Yes ($T = d e$)], [No (torsion scalar)], [In $Q$],
    [Palatini], [No ($Gamma and Gamma$)], [Yes (trace)], [In $K$],
    [Spin-2 full], [No (self-coupling)], [Yes (trace)], [In $K$],
    [Spin-2 free], [Yes (linearized)], [Yes (trace)], [No atom],
)

The non-admissible atom is *invariant* across formulations: every full GR formulation has a non-admissible step in the pipeline (Obstruction Theorem applied to the total pipeline). But the *location* of the atom (in $K$ or in $Q$) is a *factorization fiber artifact*. Cartan and TEGR are physically equivalent but place the atom in different factors.

This is precisely a *factorization rearrangement*: the same irreducible element appears in different positions in different factorizations of the same element. In the language of Smertnig (2025), the *class group* of the factorization encodes which factorizations are available: formulations with the same class group element produce the same physics.

The exception is linearized GR (spin-2 free): both $K$ and $Q$ are admissible, so no atom exists. The pipeline is in the factorial sector. This is the formulation-independent way to see that linearized GR is structurally different from full GR: it lives in a *different sector* of the factorization dichotomy.

These results are verified as propositions F1--F7 in the theory file. F4 is the key result: physically equivalent formulations have different K-admissibility (factorization rearrangement). F6 establishes that pipeline non-admissibility is invariant (the atom is preserved).

= Transfer Homomorphisms and the Class Group

Non-unique factorization theory provides a systematic tool for studying factorization in monoids where unique factorization fails: the *transfer homomorphism* (Halter-Koch, building on Narkiewicz and Kummer--Dedekind theory).

#strong[Definition 2] (Transfer homomorphism). A monoid homomorphism $theta : H arrow B$ is a transfer homomorphism if:
1. $B = theta(H) dot B^times$ (surjectivity up to units).
2. If $theta(a) = b_1 b_2$ in $B$, then $a = a_1 a_2$ in $H$ with $theta(a_i) approx b_i$ (factorization lifting).

A transfer homomorphism preserves all factorization-theoretic invariants: sets of lengths, elasticity, catenary degree. It allows one to study factorizations in $H$ by studying factorizations in a simpler monoid $B$.

For the POT kernel monoid, we identify two natural transfers:

*Binary transfer:* $theta_1 : cal(K) arrow {0, 1}$, mapping $K$ to 0 if admissible, 1 if not. This preserves the factorization dichotomy: factorial sector maps to 0, atomic sector maps to 1. It is the coarsest transfer that captures the obstruction theorem.

*Formulation transfer:* $theta_2 : cal(F) arrow cal(K) slash tilde.op$, mapping formulations to physical equivalence classes. This forgets the atom location (which is a fiber artifact) while preserving the pipeline non-admissibility. Two formulations in the same class have the same physics but may have different factorizations of the pipeline.

The *class group* $"Cl"(theta_2)$ of the formulation transfer encodes the degrees of freedom in choosing a factorization. For GR, the class group is non-trivial: it contains the Cartan class (atom in $K$) and the TEGR class (atom in $Q$). For electrodynamics, the class group is trivial (unique factorization sector).

Propositions T1--T2 verify that the transfer preserves pipeline non-admissibility (T1) and forgets atom location (T2).

= Connections to Pure Mathematics

The factorization structure of the POT kernel monoid connects to several established mathematical traditions.

*Non-unique factorization in monoids and rings* (Geroldinger and Halter-Koch, 2006). The foundational theory develops invariants --- elasticity, catenary degree, sets of lengths --- for studying how factorizations in cancellative monoids can differ. Our kernel composition monoid $cal(K)$ is a (generally non-commutative, non-cancellative) monoid; extending the Geroldinger--Halter-Koch invariants to this setting is a well-defined mathematical program. The recent work of Smertnig (2025) on non-commutative Dedekind prime rings provides the modern tools.

*Factorization of differential operators* (Shemyakova, 2010; Tsarev, 2005). The Landau example and its generalizations show that LPDOs can have factorizations with different numbers of irreducible factors, that coprimality of symbols controls refinement, and that invariant theory characterizes when factorizations exist. The POT admissibility predicate plays the role of the symbol coprimality condition.

*Categorical factorization systems* (Freyd and Kelly, 1972; Bousfield, 1977). In a category $cal(C)$, a factorization system $(E, M)$ decomposes every morphism $f$ as $f = m compose e$ with $e in E$, $m in M$. If the system is *orthogonal*, the factorization is unique up to isomorphism. If it is *weak*, the factorization is non-unique. The K--Q pipeline is an $(E, M)$-factorization with $E =$ "production kernels" and $M =$ "observable projections." The formulation fiber shows this system is weak, not orthogonal: the same total morphism admits multiple (E, M)-decompositions.

*Factorization algebras in QFT* (Costello and Gwilliam, 2016). Factorization algebras organize local-to-global information about observables in quantum field theory. The POT kernel factorization is complementary: it organizes the pipeline from configuration to observable, while Costello--Gwilliam organizes the spatial assembly of local observables into global ones.

*Wiener--Hopf factorization* (Gohberg and Krein, 1958). The factorization of operator-valued functions on the half-line is essentially unique under analyticity conditions. The POT case differs: no analyticity condition constrains the factorization, so non-uniqueness is generic.

= Discussion

*What the factorization dichotomy tells us about physics.* The partition of the kernel monoid into factorial and atomic sectors is not merely an algebraic classification --- it corresponds to a structural divide in physics. Admissible theories (electrodynamics, linearized gravity, quantum measurement) have clean factorizations: one can identify universal geometric content, dynamical content, and representational content as separate, admissible stages. Non-admissible theories (full GR, Yang--Mills) resist such decomposition: the self-coupling term binds the stages together into an irreducible whole.

*The atom as physics.* The non-admissible atom ($omega and omega$ in GR, $A and A$ in YM) is not a mathematical inconvenience --- it is the carrier of essential physics. In Yang--Mills, the atom generates confinement (color charges become unobservable). In GR, the atom generates energy non-localizability and the problem of time. Removing the atom (linearization) restores admissibility but loses these features. The atom is the price of self-interaction.

*Why two architectures coexist.* The two architectures are not competing descriptions of the same structure. They are structurally distinct, each applicable in its own sector. Architecture 1 (three-factor, all admissible) is the natural language for the factorial sector: it decomposes the kernel into independent, recombinable components. Architecture 2 (K--Q pipeline) is the natural language for the atomic sector: it accepts the non-admissible atom as given and focuses on separating production from observation. Attempting to force Architecture 1 on a non-admissible kernel fails (Obstruction Theorem). Attempting to force Architecture 2 on an admissible kernel works but is unnecessarily restrictive.

*The formulation fiber as non-uniqueness.* The central result of the GR projection kernel paper --- that energy localizability is a formulation fiber artifact --- is now seen as an instance of non-unique factorization. The atom (non-admissible step) is invariant; its location (in $K$ or $Q$) varies across formulations; properties that depend on the location (energy localizability) are therefore factorization-dependent, not physical. This resolves the century-old debate about gravitational energy: it is neither localizable nor non-localizable; the question is ill-posed because it depends on the factorization.

*Open questions.* Several mathematical questions remain. First, what is the elasticity of the POT kernel monoid? For admissible kernels, the maximum factorization length depends on the richness of the available sub-decompositions. Second, does the kernel monoid have a Krull structure, and if so, what is its class group? Third, can the categorical factorization system (production, projection) be made orthogonal by imposing additional physical conditions (e.g., requiring the image of $K$ to be the smallest space containing all physically relevant intermediate quantities)?

= Conclusion

The POT framework describes physics through kernel operators that map configurations to observables. This paper has shown that the factorization of these kernels --- how the pipeline from configuration to observable is decomposed into stages --- is governed by a structural dichotomy controlled by the admissibility predicate.

*The obstruction.* Non-admissible kernels cannot be decomposed into admissible factors. The self-coupling terms $omega and omega$ (GR) and $A and A$ (Yang--Mills) are irreducible atoms in the composition monoid.

*The dichotomy.* Admissible kernels live in a factorial sector with multiple factorizations (Architecture 1: $K_("univ") compose K_("dyn") compose K_("rep")$). Non-admissible kernels live in an atomic sector where the atom constrains the decomposition (Architecture 2: $K arrow Q$).

*The fiber.* Physically equivalent formulations of a theory factor the same total pipeline differently. The atom moves between $K$ and $Q$. Properties that depend on the atom's location are factorization artifacts, not physics.

*The mathematics.* The kernel monoid connects to non-unique factorization theory (Geroldinger--Halter-Koch), LPDO factorization (Landau, Shemyakova), categorical factorization systems (Freyd--Kelly), and factorization algebras in QFT (Costello--Gwilliam). The POT case is distinguished by its physical interpretation: different factorizations correspond to different ways of reading a theory, and factorization artifacts correspond to questions that depend on how we read, not on what the theory says.

35 propositions have been verified by Z3 on the Kleis platform, covering the monoid structure (M1--M3), the admissibility obstruction (A1--A5), the kernel classification (K1--K6), the factorization dichotomy (D1--D5), the formulation fiber (F1--F7), the transfer homomorphism (T1--T2), the elasticity properties (E1--E3), and cross-architecture compatibility (C1--C4).



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[Geroldinger2006\] A. Geroldinger and F. Halter-Koch, Non-Unique Factorizations: Algebraic, Combinatorial and Analytic Theory, Chapman and Hall/CRC, 2006.]

#par(hanging-indent: 1.5em)[\[Smertnig2025\] D. Smertnig, Divide and Transfer: Non-Unique Factorizations Beyond Commutativity, arXiv:2602.06222, 2025.]

#par(hanging-indent: 1.5em)[\[Shemyakova2010\] E. Shemyakova, Refinement of Two-Factor Factorizations of a Linear Partial Differential Operator of Arbitrary Order and Dimension, arXiv:1001.2004, 2010.]

#par(hanging-indent: 1.5em)[\[Landau1903\] E. Landau, Über die Zerlegung definiter Funktionen in Quadrate, Archiv der Mathematik und Physik, 1903.]

#par(hanging-indent: 1.5em)[\[Costello2016\] K. Costello and O. Gwilliam, Factorization Algebras in Quantum Field Theory, Volume 1, Cambridge University Press, 2016.]

#par(hanging-indent: 1.5em)[\[GohbergKrein1958\] I. C. Gohberg and M. G. Krein, Systems of Integral Equations on a Half Line with Kernels Depending on the Difference of Arguments, Uspekhi Matematicheskikh Nauk 13(2), 1958.]

#par(hanging-indent: 1.5em)[\[FreydKelly1972\] P. Freyd and G. M. Kelly, Categories of Continuous Functors, I, Journal of Pure and Applied Algebra 2(3), 1972.]

#par(hanging-indent: 1.5em)[\[WienerHopf\] N. Wiener and E. Hopf, Über eine Klasse singulärer Integralgleichungen, Sitzungsberichte der Preußischen Akademie der Wissenschaften, 1931.]

#par(hanging-indent: 1.5em)[\[AtikEntanglement\] E. Atik, Quantum Entanglement as a Projection Artifact: Bell Violation from Admissible Kernels, Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[AtikElectrodynamics\] E. Atik, Electrodynamics as a Theorem of Projected Ontology, Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[AtikGR\] E. Atik, Reading General Relativity Through the Projection Kernel: Non-Admissibility, Formulation Fibers, and the Evidentiary Gap, Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[AtikRotation\] E. Atik, Flat Galactic Rotation Curves from Projected Ontology Theory, Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[AtikYM\] E. Atik, Non-Abelian Gauge Confinement as Admissibility Restoration, Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[Kleis\] E. Atik, Kleis: A Verification Platform for Mathematical Knowledge Production, https://kleis.io, 2025.]


