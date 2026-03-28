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
  #text(size: 17pt, weight: "bold")[Electrodynamics as a Theorem of Projected Ontology]
  
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
      #text(size: 10pt)[We show that classical electrodynamics can be reduced to a single dynamical equation relating field to sources once expressed in the exterior calculus on an oriented Lorentzian 4-manifold: all remaining structure follows from the nilpotency of the exterior derivative, $d^2 = 0$, and is formally verifiable. Given a smooth oriented 4-manifold $(M, g)$ with Lorentzian signature, the exterior algebra, and the Hodge star operator, we take two axioms --- the definition $F = d A$ (field strength as an exact 2-form) and the inhomogeneous Maxwell equation $d star.op F = star.op J$ (relating the field to sources) --- and derive the complete differential-form structure of classical electrodynamics as a chain of one- and two-step consequences. Gauge invariance ($A arrow.r A + d chi$ leaves $F$ unchanged), the homogeneous Maxwell equation ($d F = 0$, encoding Faraday's law and the absence of magnetic monopoles), and charge conservation ($d(star.op J) = 0$, the continuity equation) all follow from $d^2 = 0$ applied at most twice. We formalize these derivations on the Kleis verification platform, where every axiom --- both the mathematical background structure and the physical content --- is explicit, and every theorem is machine-checked by the Z3 SMT solver. The entire exterior calculus (wedge product, Hodge star, interior product, Lie derivative, de Rham cohomology) is axiomatized in the Kleis standard library. The electromagnetic structures share the same Cartan geometry substrate used in our companion papers on flat galactic rotation curves and quantum entanglement. The potential $A$ is a projection degree of freedom with no ontological weight; monopoles are excluded when $F$ is globally exact (trivial principal bundle); and charge conservation is the algebraic consequence of applying $d$ to $d star.op F = star.op J$, which admits a Noether interpretation as the invariant of the $U(1)$ gauge symmetry under the standard electromagnetic action. The Einstein--Maxwell coupling and the non-abelian Yang--Mills extension $F = d A + A and A$ exhibit a structural parallel with gravity in the Cartan formalism, and both are already axiomatized in the same library. The result is a demonstration that the irreducible physical content of electrodynamics is concentrated in a single equation ($d star.op F = star.op J$); everything else is mathematics.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* Maxwell equations, gauge invariance, formal verification, projected ontology, exterior calculus, Noether theorem, Hodge star, differential forms, Z3 theorem prover, Cartan geometry]

#v(1em)


= Introduction

What are Maxwell's equations? The standard answer is: empirical laws, discovered by Faraday and Maxwell, encoding the behavior of electric and magnetic fields. The mathematical physicist's answer is more refined: they are the unique equations governing a $U(1)$ gauge field on a Lorentzian manifold. But even this answer treats them as *given* --- the starting point of a theory, not its output.

We propose a stronger claim: classical electrodynamics can be reduced to a *single dynamical equation* relating field to sources, once the mathematical infrastructure is made explicit. The infrastructure is: a smooth, oriented 4-dimensional manifold $M$; a Lorentzian metric $g$ with signature $(-,+,+,+)$; the exterior algebra of differential forms with exterior derivative $d$ satisfying $d^2 = 0$; and the Hodge star operator $star.op$ determined by $g$ and the orientation. None of these are physical postulates --- they are the mathematical stage on which physics is performed.

On this stage, we take two axioms: the definition $F = d A$ (field strength as an exact 2-form) and one irreducible physical equation $d star.op F = star.op J$ (the inhomogeneous Maxwell equation relating the field to its sources). From these two statements alone, the entire structure of classical electrodynamics follows:

1. *Gauge invariance*: $A arrow.r A + d chi$ leaves $F$ unchanged (one step from $d^2 = 0$).

2. *Homogeneous Maxwell equation*: $d F = 0$ --- no magnetic monopoles, Faraday's law (one step from $F = d A$ and $d^2 = 0$).

3. *Charge conservation*: $d(star.op J) = 0$, i.e., $partial_mu J^mu = 0$ (two steps: apply $d$ to $d star.op F = star.op J$, use $d^2 = 0$).

No additional physical postulates are needed. The potential is an auxiliary projection variable; the field is the only ontologically meaningful object; charge conservation is the algebraic consequence of applying $d$ to the inhomogeneous equation (admitting a Noether interpretation under the standard action); and monopoles are excluded whenever $F$ is globally exact.

This paper is the third in a series on Projected Ontology Theory (POT). The first derived flat galactic rotation curves from the admissible kernel framework without dark matter. The second derived Bell inequality violation and GHZ contextuality from the same kernel structure, reinterpreting quantum entanglement as a projection artifact. The present paper adds the gauge-theoretic sector, showing that the same Cartan geometry substrate that governs gravity also determines electrodynamics. The Einstein--Maxwell coupling $G_(mu nu) + Lambda g_(mu nu) = kappa T^("EM")_(mu nu)$ exhibits a structural parallel with the gravitational sector through shared use of the Cartan exterior calculus, and the non-abelian generalization (Yang--Mills) is already formalized in the library.

Every derivation in this paper is machine-checked. The axioms are explicit, stated in the Kleis verification language, and verified by the Z3 SMT solver. The entire exterior calculus --- wedge product, Hodge star, interior product, Cartan's Magic Formula, de Rham cohomology --- is axiomatized in the Kleis standard library (`stdlib/differential_forms.kleis`). The reader can reproduce every result by running the source file.

= The Projected Ontology Framework

We work within the mathematical framework of Projected Ontology Theory (POT), building on the Cartan geometry pipeline axiomatized in the Kleis standard library. The reader unfamiliar with the full POT axiom set is referred to our companion papers for details; here we summarize only what is needed for the electrodynamics derivation.

== Background Structure

Before stating the two physical axioms, we make explicit the mathematical infrastructure they presuppose. This infrastructure is not physics --- it is the stage on which physics is performed:

1. *Smooth manifold* $M$: A 4-dimensional smooth (i.e., $C^infinity$) manifold serving as the spacetime continuum. All operations below are defined on $M$.

2. *Lorentzian metric* $g$: A non-degenerate symmetric bilinear form of signature $(-,+,+,+)$ on the tangent bundle $T M$. The metric determines causal structure and is required for the Hodge star.

3. *Orientation*: A choice of volume form on $M$, required (together with $g$) to define the Hodge star operator $star.op$.

4. *Exterior algebra*: The graded algebra of differential forms $Omega^p(M)$ for $p = 0, 1, dots, 4$, equipped with the wedge product $and$ and the exterior derivative $d : Omega^p arrow.r Omega^(p+1)$ satisfying the nilpotency $d^2 = 0$.

5. *Hodge star* $star.op$: The isomorphism $star.op : Omega^p(M) arrow.r Omega^(n-p)(M)$ determined by $g$ and the orientation, with the involutivity $star.op star.op alpha = (-1)^(p(n-p) + s) alpha$ where $s$ is the number of negative eigenvalues of $g$ (here $s = 1$).

Items 1--3 are geometric choices. Item 4 is a structural property of smooth manifolds (the nilpotency $d^2 = 0$ is equivalent to the symmetry of mixed partial derivatives). Item 5 is determined by items 2 and 3.

The physical content enters only through the two axioms stated in Section 3. Everything derived in Sections 4--7 follows from these axioms plus the background structure above. The claim of this paper is not that electrodynamics requires *no* assumptions, but that once the mathematical stage is set, a single physical equation ($d star.op F = star.op J$) suffices to determine the rest.

== Cartan Geometry Pipeline

The Kleis standard library implements Élie Cartan's approach to differential geometry using differential forms. The computation pipeline is:

$ "Tetrad" e^a arrow.r "Connection" omega^a_b arrow.r "Curvature" R^a_b arrow.r "Ricci, Einstein" $

Given a tetrad (orthonormal frame) $e^a$ encoding the metric via $g = eta_(a b) e^a times.o e^b$, the torsion-free condition $d e^a + omega^a_b and e^b = 0$ uniquely determines the connection 1-forms $omega^a_b$. The curvature 2-forms are then:

$ R^a_b = d omega^a_b + omega^a_c and omega^c_b $

This is Cartan's second structure equation. The Bianchi identity $d R^a_b + omega^a_c and R^c_b - R^a_c and omega^c_b = 0$ follows from $d^2 = 0$. Contracting gives the Ricci tensor and scalar curvature, hence the Einstein tensor $G_(mu nu)$.

The key point for electrodynamics: the same exterior derivative $d$ and wedge product $and$ that govern spacetime curvature also govern the electromagnetic field. The field strength $F$ is a 2-form, just as the curvature $R^a_b$ is a matrix of 2-forms. The algebraic structures are identical.

== The Exterior Algebra

The derivations in this paper rely on four operations from the exterior algebra, all axiomatized in the Kleis standard library:

1. *Exterior derivative* $d$: maps $p$-forms to $(p+1)$-forms. The fundamental property is $d^2 = 0$ (nilpotency), which encodes the topology of the manifold.

2. *Hodge star* $star.op$: maps $p$-forms to $(n-p)$-forms on an $n$-dimensional manifold with metric signature $s$ (the number of negative eigenvalues). The involutivity relation is:

$ star.op star.op alpha = (-1)^(p(n-p) + s) alpha $

For the electromagnetic 2-form $F$ on a 4D Lorentzian manifold ($p = 2$, $n = 4$, $s = 1$): $star.op star.op F = (-1)^(4 + 1) F = -F$. This sign is essential for correct energy-momentum traces.

3. *Interior product* $iota_X$: contracts a vector field $X$ with a $p$-form, reducing degree by 1. Metric-independent.

4. *Lie derivative* $cal(L)_X$: Cartan's Magic Formula $cal(L)_X = d compose iota_X + iota_X compose d$ connects the algebraic and differential operations. Metric-independent.

The Leibniz rule $d(alpha and beta) = (d alpha) and beta + (-1)^p alpha and (d beta)$ and the graded antisymmetry $alpha and beta = (-1)^(p q) beta and alpha$ complete the algebraic structure. Every identity used in this paper is an axiom in the standard library, verified by Z3.

= The Electromagnetic 2-Form

In the language of differential forms, the electromagnetic field is a 2-form $F$ on a 4-dimensional Lorentzian manifold $(M, g)$ with signature $(-,+,+,+)$. In coordinates:

$ F = E_i thin d x^i and d t + 1/2 B_(i j) thin d x^i and d x^j $

The potential is a 1-form $A = phi thin d t + A_i thin d x^i$, and the 4-current is a 1-form $J = rho thin d t + J_i thin d x^i$ (or equivalently, a 3-form $star.op J$ via the Hodge dual).

The entire content of classical electrodynamics rests on two statements:

== The Two Axioms

*Axiom 1 (Definition).* The field strength is the exterior derivative of the potential:

$ F = d A $

This is a definition, not a dynamical equation. It asserts that the electromagnetic field is an *exact* 2-form.

*Axiom 2 (Physics).* The inhomogeneous Maxwell equation relates the field to its sources via the Lorentzian Hodge star:

$ d star.op F = star.op J $

This single equation encodes both Gauss's law ($nabla dot.op E = rho / epsilon_0$) and Ampère's law with Maxwell's correction ($nabla times B = mu_0 J + mu_0 epsilon_0 partial E / partial t$).

These are the only independent axioms. Everything else --- gauge invariance, the homogeneous Maxwell equation, charge conservation --- is a *derivable theorem*. The remainder of this paper exhibits each derivation and its machine-checked verification.

= Gauge Invariance as Geometric Tautology

The gauge transformation $A arrow.r A' = A + d chi$, where $chi$ is any scalar function (0-form), leaves the field strength invariant:

$ F' = d A' = d(A + d chi) = d A + d(d chi) = d A + 0 = F $

The second equality uses linearity of $d$. The third uses the nilpotency $d^2 = 0$: the exterior derivative of any exact form vanishes. This is not an approximation, not a symmetry imposed by hand, and not a physical assumption. It is a *tautology of the exterior algebra*.

The physical consequence is profound: the potential $A$ is not an observable quantity. It is a *degree of freedom of the mathematical description* --- a choice of representative within an equivalence class of 1-forms that yield the same field $F$. Only $F$ has ontological weight. The potential is an auxiliary projection variable, and gauge 'symmetry' is the statement that the projection is many-to-one.

In the POT framework, this has a precise interpretation: $A$ is a choice of *how* to project from the ontological space to the observable field. Different choices of $A$ correspond to different projections, but the observable field $F$ --- the quantity that couples to charged matter --- is the same for all of them. Gauge invariance is not a feature of electrodynamics; it is a structural property of exact forms on smooth manifolds.

= No Magnetic Monopoles: A Topological Theorem

The homogeneous Maxwell equation is:

$ d F = 0 $

This encodes two classical statements: $nabla dot.op B = 0$ (no magnetic monopoles) and $nabla times E = -partial B / partial t$ (Faraday's law). The derivation from Axiom 1 is immediate:

$ d F = d(d A) = d^2 A = 0 $

The non-existence of magnetic monopoles is therefore *not* an independent law of physics. It is a one-step consequence of the definition $F = d A$ and the nilpotency $d^2 = 0$.

The deeper statement involves de Rham cohomology. A $p$-form $alpha$ is *closed* if $d alpha = 0$ and *exact* if $alpha = d beta$ for some $(p-1)$-form $beta$. Since $d^2 = 0$, every exact form is closed: $d(d beta) = 0$. The converse is not always true --- it depends on the topology of the manifold.

The second de Rham cohomology group $H^2_("dR")(M)$ measures the obstruction: it is the space of closed 2-forms modulo exact 2-forms. If $H^2_("dR")(M) = 0$, then *every* closed 2-form is exact, and any field strength $F$ satisfying $d F = 0$ must be expressible as $F = d A$ for some potential $A$.

Magnetic monopoles would correspond to $F$ being closed but *not* exact --- a nontrivial element of $H^2_("dR")(M)$. On $RR^4$ (or any contractible manifold), $H^2_("dR") = 0$, and every closed 2-form is exact. The non-existence of monopoles is then a theorem, not a postulate.

*Important caveat.* Our Axiom 1 ($F = d A$) is stronger than merely requiring $d F = 0$. It asserts that $F$ is *globally exact* --- that a global potential 1-form $A$ exists on all of $M$. In the language of fiber bundles, this amounts to working on a *trivial* $U(1)$ principal bundle. On a nontrivial bundle (e.g., over $RR^3 backslash {0}$), $F$ can be closed but not globally exact, and the Dirac monopole lives precisely in this cohomological gap.

The distinction matters: our derivations hold for the trivial-bundle case, which covers standard Minkowski-spacetime electrodynamics. The generalization to nontrivial bundles --- where $A$ is a connection on a principal $U(1)$-bundle rather than a globally defined 1-form --- requires replacing the de Rham cohomology argument with characteristic classes (Chern numbers), and monopole charge becomes quantized by the Dirac condition. This generalization does not invalidate our framework; it extends it to a topologically richer setting where Axiom 1 is replaced by the weaker requirement that $F$ be closed.

= Energy-Momentum Conservation

The electromagnetic stress-energy tensor $T^("EM")_(mu nu)$ encodes the energy density, momentum density, and stress of the electromagnetic field. Its conservation $partial_mu T^(mu nu)_("EM") = 0$ (in the absence of sources) is the formal statement of energy-momentum conservation for the field.

In the Cartan formalism, this conservation law is not an additional postulate. It follows from the Bianchi identity. For the Riemann curvature 2-form:

$ d R^a_b + omega^a_c and R^c_b - R^a_c and omega^c_b = 0 $

This identity, which itself is a consequence of $d^2 = 0$ applied to Cartan's second structure equation, implies the contracted Bianchi identity $nabla_mu G^(mu nu) = 0$ for the Einstein tensor. Combined with the Einstein--Maxwell field equation $G_(mu nu) + Lambda g_(mu nu) = kappa T^("EM")_(mu nu)$, this yields $nabla_mu T^(mu nu)_("EM") = 0$.

The Poynting vector $S = E times B / mu_0$ and the electromagnetic energy density $u = (epsilon_0 E^2 + B^2 / mu_0) / 2$ are components of $T^("EM")_(mu nu)$. Their conservation --- the Poynting theorem --- is not an independent result but a component of the tensorial conservation law that follows from the Bianchi identity.

The Lie derivative provides the infinitesimal version. For a timelike vector field $xi^mu$ (representing time evolution), Cartan's Magic Formula gives:

$ cal(L)_xi F = d(iota_xi F) + iota_xi (d F) = d(iota_xi F) $

The last equality uses $d F = 0$ (derived in Section 5). The energy flux along $xi$ is then entirely determined by the contraction $iota_xi F$ and its exterior derivative --- no additional structure needed beyond what the exterior algebra provides.

= Charge Conservation: The Noether Closing Argument

The derivation of charge conservation from the two axioms is a two-step chain:

*Step 1.* From Axiom 2: $d star.op F = star.op J$.

*Step 2.* Apply the exterior derivative $d$ to both sides:

$ d(d star.op F) = d(star.op J) $

The left side is $d^2(star.op F) = 0$ by nilpotency. Therefore:

$ d(star.op J) = 0 $

In components, this is the continuity equation $partial_mu J^mu = 0$: charge is locally conserved.

This result is not a separate law. It is a two-step algebraic consequence of the inhomogeneous Maxwell equation and $d^2 = 0$. No additional axiom is needed.

Note what this derivation does *not* require: it does not invoke an action functional, a variational principle, or the Noether theorem. The conservation law $d(star.op J) = 0$ follows purely from the algebraic identity $d^2 = 0$ applied to Axiom 2.

*The Noether interpretation.* The algebraic derivation above has a physical reading through Noether's theorem, which provides deeper context. The standard electromagnetic action is:

$ S = integral (1/2 F and star.op F + A and star.op J) $

The gauge transformation $A arrow.r A + d chi$ is a $U(1)$ symmetry of this action (Section 4), and Noether's theorem associates a conserved current to every continuous symmetry. The conserved current associated with gauge invariance is precisely $J^mu$. Charge conservation is therefore the *Noether invariant* of the $U(1)$ gauge symmetry.

The two perspectives are complementary:

- *Algebraic*: $d(star.op J) = 0$ follows from $d^2 = 0$ in two steps. No action needed.
- *Variational*: $partial_mu J^mu = 0$ is the Noether current of the $U(1)$ gauge symmetry of the action $S$.

The algebraic derivation is logically prior (it requires fewer assumptions), but the Noether interpretation explains *why* this conservation law exists: it is the shadow of gauge symmetry.

Charge is not an arbitrary 'stuff' assigned to particles. It is a geometric quantity forced into existence by the structure of the exterior algebra. The source term $J$ in $d star.op F = star.op J$ is not a free choice --- it is constrained by $d(star.op J) = 0$.

This is the closing argument of the paper: two axioms, the background structure of the Lorentzian manifold, and the nilpotency of $d$ yield the complete classical electrodynamics. The field equations, the gauge symmetry, the topological structure, and the conservation laws are all derivable. The only irreducible physical content is concentrated in a single equation: $d star.op F = star.op J$.

= Einstein--Maxwell Coupling

The electromagnetic field is not isolated; it couples to gravity through the stress-energy tensor. The Einstein--Maxwell field equation is:

$ G_(mu nu) + Lambda g_(mu nu) = kappa T^("EM")_(mu nu) $

where $G_(mu nu)$ is the Einstein tensor, $Lambda$ the cosmological constant, $kappa = 8 pi G / c^4$ Einstein's gravitational constant, and $T^("EM")_(mu nu)$ the electromagnetic stress-energy tensor.

In the Kleis standard library, this equation is axiomatized in the `EinsteinMaxwell` structure, which also enforces the symmetry properties: $F_(mu nu) = -F_(nu mu)$ (antisymmetry of the field tensor), $T^("EM")_(mu nu) = T^("EM")_(nu mu)$ (symmetry of the stress-energy tensor), and $g_(mu nu) = g_(nu mu)$ (symmetry of the metric).

The coupling has a structural parallel in the Cartan formalism. Both gravity and electromagnetism are built from the *same* mathematical operations:

- *Gravity*: the curvature 2-form $R^a_b = d omega^a_b + omega^a_c and omega^c_b$ is a matrix of 2-forms built from the connection $omega$ via the exterior derivative $d$ and wedge product $and$.

- *Electromagnetism*: the field strength 2-form $F = d A$ is a single 2-form built from the potential $A$ via the same exterior derivative $d$.

The connection $omega^a_b$ and the potential $A$ are both 1-forms. The curvature $R^a_b$ and the field strength $F$ are both 2-forms. The Bianchi identity $d R + omega and R - R and omega = 0$ and the homogeneous Maxwell equation $d F = 0$ are both consequences of $d^2 = 0$.

We emphasize that this is a *structural parallel*, not a derivation of electrodynamics from gravity or vice versa. The Einstein--Maxwell equation couples the two sectors, but the coupling is an additional axiom (relating $G_(mu nu)$ to $T^("EM")_(mu nu)$), not a consequence of the exterior calculus alone. What *is* a consequence of the shared algebraic structure is that the conservation law $nabla_mu T^(mu nu)_("EM") = 0$ follows from the Bianchi identity $nabla_mu G^(mu nu) = 0$ once the coupling is assumed.

This structural parallel connects to our companion papers: the same Cartan geometry infrastructure (exterior derivative, wedge product, curvature forms) underlies both the gravitational sector (rotation curves, lensing) and the electromagnetic sector. The operations are shared; the physical content is sector-specific.

= The Yang--Mills Generalization

Electrodynamics is the $U(1)$ gauge theory. The natural generalization to non-abelian gauge groups --- Yang--Mills theory --- replaces the abelian field strength $F = d A$ with:

$ F = d A + A and A $

The additional $A and A$ term (absent in electrodynamics because the $U(1)$ Lie algebra is abelian) encodes the self-interaction of the gauge field. The Bianchi identity becomes the gauge-covariant version:

$ D_A F = d F + [A, F] = 0 $

where $D_A$ is the gauge-covariant exterior derivative.

This structure is *already formalized* in the Kleis standard library as the `YangMillsForm` structure, with the axioms:

- $F = d A + A and A$ (curvature definition)
- $D_A F = 0$ (Bianchi identity)

The formal pattern is identical to the Cartan geometry of gravity:

- *Gravity*: Connection $omega$, curvature $R = d omega + omega and omega$, Bianchi identity $D_omega R = 0$.
- *Yang--Mills*: Connection $A$, curvature $F = d A + A and A$, Bianchi identity $D_A F = 0$.

The gauge group determines the physics: $U(1)$ gives electrodynamics, $S U(2) times U(1)$ gives the electroweak theory, and $S U(3)$ gives quantum chromodynamics. But the mathematical structure --- connection, curvature, Bianchi identity --- is the same in every case.

This observation sets the stage for a verified treatment of the Standard Model gauge sector within the POT framework. The Kleis standard library already contains the axioms; what remains is to derive the specific physical consequences for each gauge group, following the same derivation pattern used in this paper for $U(1)$.

= Discussion and Conclusion

We have shown that, given the background structure of a smooth oriented Lorentzian 4-manifold with its exterior algebra and Hodge star, the complete differential-form structure of classical electrodynamics follows from two axioms and the nilpotency $d^2 = 0$. The derivation chain is short:

#table(
  columns: (auto, auto, auto),
  inset: 8pt,
  align: (left, left, left),
  [*Theorem*], [*Derivation*], [*Steps from axioms*],
  [Gauge invariance], [$d(A + d chi) = d A + d^2 chi = d A$], [1 step],
  [Homogeneous Maxwell ($d F = 0$)], [$d F = d(d A) = d^2 A = 0$], [1 step],
  [Charge conservation ($d(star.op J) = 0$)], [$d(star.op J) = d(d star.op F) = d^2(star.op F) = 0$], [2 steps],
  [No monopoles], [Topological: $H^2_("dR")(RR^4) = 0$], [Cohomology],
  [Poynting / energy conservation], [Bianchi identity $+$ Einstein--Maxwell], [Structural],
)

The irreducible physical content is concentrated in a single dynamical equation: $d star.op F = star.op J$. This is the *only* statement that connects the geometry (the field $F$) to the physical world (the source current $J$). Everything else is either mathematical infrastructure (manifold, metric, exterior algebra) or a derivable consequence thereof.

Several features of this result merit comment.

*The role of $d^2 = 0$.* The nilpotency of the exterior derivative is not a physical assumption. It is a structural property of the exterior algebra on any smooth manifold, equivalent to the symmetry of mixed partial derivatives (Schwarz's theorem). Every derivation in this paper reduces, ultimately, to this single identity. Gauge invariance, the homogeneous Maxwell equation, and charge conservation are all shadows of $d^2 = 0$ projected through different algebraic paths.

*Machine verification as methodology.* The Kleis platform makes it possible to treat physics derivations as executable programs: the axioms are inputs, the derivations are computations, and the theorems are outputs checked by an independent solver. This is not merely a presentation choice. It enforces a discipline that informal derivations do not: every step must be justified by an explicit axiom, every axiom must be stated, and the dependency graph --- which theorem follows from which axiom in how many steps --- is a machine-checkable artifact. The derivation table above is the output of this process, not a human summary of it. We believe this represents a productive mode of interaction between formal methods and theoretical physics: not replacing physical intuition, but making its logical structure auditable.

*What is and is not claimed.* The formulation of electrodynamics in terms of differential forms is well known; the key references are Flanders, Misner--Thorne--Wheeler, Frankel, and Nakahara. Our contribution is not the mathematics, which is standard, but: (a) the isolation of a *minimal* axiom set and a complete *dependency graph* showing which results follow from which axioms in how many steps; (b) the *machine verification* of every derivation, making the axiom dependencies checkable rather than informal; and (c) the embedding of this verified structure into the Projected Ontology framework alongside gravity and quantum entanglement.

*Comparison with the standard presentation.* Textbooks typically present Maxwell's equations as four independent laws (Gauss, Gauss for magnetism, Faraday, Ampère--Maxwell), then derive gauge invariance and charge conservation as consequences. Our presentation inverts this: we start with the definition $F = d A$ and the single inhomogeneous equation $d star.op F = star.op J$, and derive everything else. The 'four laws' are a redundant encoding of two independent statements.

*The Lorentzian Hodge star.* The sign in the involutivity relation $star.op star.op alpha = (-1)^(p(n-p) + s) alpha$ depends on the metric signature $s$ (number of negative eigenvalues). For the electromagnetic 2-form on Minkowski spacetime, $star.op star.op F = -F$. Getting this sign wrong would reverse the sign of the energy-momentum tensor, predicting negative energy densities. The parameterization of the Hodge star by the signature integer $s$ is therefore not a mathematical nicety but a physical necessity.

*Connection to the POT series.* This paper is the third in a series demonstrating that the Kleis standard library axiomatizes the mathematical structures from which physical laws can be derived as theorems. The first paper derived flat rotation curves from the admissible kernel framework. The second derived Bell violation from the same kernel structure. The present paper derives electrodynamics from the exterior calculus and Cartan geometry --- mathematical structures that *also* live in the kernel framework. The Einstein--Maxwell coupling makes the connection explicit: the same anholonomy coefficients that determine spacetime curvature also constrain the electromagnetic field.

*Toward verified unified field theory.* The Yang--Mills generalization $F = d A + A and A$ is already axiomatized. The formal structure is identical to gravity (connection, curvature, Bianchi identity). The gauge group determines the physics; the mathematics is universal. A verified treatment of the Standard Model gauge sector --- $S U(3) times S U(2) times U(1)$ --- is within reach using the same derivation patterns and the same verification infrastructure.

We conclude with a philosophical observation. In the Projected Ontology framework, the distinction between 'law' and 'theorem' depends on whether a statement carries irreducible physical content or follows from mathematical structure. For electrodynamics, the inventory is clear: one dynamical equation ($d star.op F = star.op J$), one definitional axiom ($F = d A$), and a background manifold. Everything else --- gauge invariance, the homogeneous Maxwell equation, charge conservation, the absence of monopoles on trivial bundles --- is theorem. The universe does not 'obey' four independent Maxwell equations; it satisfies one dynamical constraint on a Lorentzian manifold, and the rest is the compiled output of the exterior algebra.

= Appendix A: Standard Library Axiom Inventory

The following table lists every axiom from the Kleis standard library used in the derivations of this paper.

#table(
  columns: (auto, auto, auto),
  inset: 6pt,
  align: (left, left, left),
  [*Axiom*], [*Structure / File*], [*Statement*],
  [$d^2 = 0$], [`ExteriorDerivative` / `differential_forms.kleis`], [$d(d alpha) = 0$ for any $p$-form $alpha$],
  [Leibniz rule], [`ExteriorDerivative`], [$d(alpha and beta) = (d alpha) and beta + (-1)^p alpha and (d beta)$],
  [Hodge involutivity], [`HodgeStar` / `differential_forms.kleis`], [$star.op star.op alpha = (-1)^(p(n-p)+s) alpha$],
  [Lorentzian Hodge], [`LorentzianHodge`], [$star.op star.op alpha = (-1)^(p(n-p)+1) alpha$ (specialization with $s=1$)],
  [Interior nilpotency], [`InteriorProduct`], [$iota_X iota_X = 0$],
  [Cartan Magic Formula], [`CartanCalculus`], [$cal(L)_X = d compose iota_X + iota_X compose d$],
  [$F = d A$], [`ElectromagneticForm`], [Field strength from potential (Axiom 1)],
  [$d star.op F = star.op J$], [`ElectromagneticForm`], [Inhomogeneous Maxwell (Axiom 2)],
  [Exact $arrow.r.double$ Closed], [`DeRhamCohomology`], [$d beta = 0 arrow.r.double d(d beta) = 0$ (exact implies closed)],
  [$F$ antisymmetry], [`FieldTensorProperties` / `maxwell.kleis`], [$F_(mu nu) = -F_(nu mu)$],
  [$T^("EM")$ symmetry], [`EinsteinMaxwell` / `maxwell.kleis`], [$T^("EM")_(mu nu) = T^("EM")_(nu mu)$],
  [Einstein--Maxwell], [`EinsteinMaxwell`], [$G_(mu nu) + Lambda g_(mu nu) = kappa T^("EM")_(mu nu)$],
  [Bianchi identity], [`CurvatureForm` / `cartan_geometry.kleis`], [$d R^a_b + omega^a_c and R^c_b - R^a_c and omega^c_b = 0$],
  [Yang--Mills curvature], [`YangMillsForm` / `differential_forms.kleis`], [$F = d A + A and A$],
  [Yang--Mills Bianchi], [`YangMillsForm`], [$D_A F = 0$],
)



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[cartan1923\] É. Cartan, "Sur les variétés à connexion affine et la théorie de la relativité généralisée," Annales scientifiques de l'É.N.S., 1923.]

#par(hanging-indent: 1.5em)[\[nakahara2003\] M. Nakahara, Geometry, Topology and Physics, 2nd ed. CRC Press, 2003.]

#par(hanging-indent: 1.5em)[\[mtw1973\] C. W. Misner, K. S. Thorne, and J. A. Wheeler, Gravitation. W. H. Freeman, 1973.]

#par(hanging-indent: 1.5em)[\[noether1918\] E. Noether, "Invariante Variationsprobleme," Nachrichten von der Gesellschaft der Wissenschaften zu Göttingen, 1918.]

#par(hanging-indent: 1.5em)[\[yangmills1954\] C. N. Yang and R. L. Mills, "Conservation of isotopic spin and isotopic gauge invariance," Physical Review, vol. 96, no. 1, 1954.]

#par(hanging-indent: 1.5em)[\[flanders1963\] H. Flanders, Differential Forms with Applications to the Physical Sciences. Dover, 1963.]

#par(hanging-indent: 1.5em)[\[frankel2011\] T. Frankel, The Geometry of Physics: An Introduction, 3rd ed. Cambridge University Press, 2011.]

#par(hanging-indent: 1.5em)[\[atik2025rotation\] E. Atik, "Flat galactic rotation curves from projected ontology," Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[atik2025entanglement\] E. Atik, "Quantum entanglement as a projection artifact: Machine-verified Bell violation without non-locality," Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[atik2025lensing\] E. Atik, "Gravitational lensing in projected ontology theory," Kleis Research, 2025.]

#par(hanging-indent: 1.5em)[\[kleis2025\] Kleis verification platform, https://kleis.io, 2025.]


