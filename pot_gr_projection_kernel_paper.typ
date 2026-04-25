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
  #text(size: 17pt, weight: "bold")[Reading General Relativity Through the Projection Kernel: Non-Admissibility, Formulation Fibers, and the Evidentiary Gap]
  
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
      #text(size: 10pt)[Many longstanding problems in general relativity --- the non-localizability of gravitational energy, the problem of time, the status of singularities --- arise from treating intermediate mathematical structures as physical. We show that the Projected Ontology (POT) framework separates what is physical from what is formulation-dependent, and what is empirically established from what remains open.

The technical foundation is a computation: the GR production kernel (tetrad $arrow.r$ curvature via $R = d omega + omega and omega$) is non-admissible, proved by evaluating the admissibility defect $Delta eq.not 0$ on the Schwarzschild tetrad using the Kleis symbolic pipeline. This parallels Yang--Mills ($F = d A + A and A$), with the $omega and omega$ self-coupling as the source.

Three results follow. First, a *Projection Sufficiency Principle*: non-admissibility requires dynamical restoration (a Higgs mechanism) if and only if the essential observables lie in the sector it hides. In Yang--Mills, charges are hidden, so the Higgs is needed. In GR, geometry survives intact --- no restoration is needed. The observable projection $Q$ is not a passive filter but the arbiter of whether non-admissibility has physical consequences.

Second, a *formulation-independence* result: in every formulation of full GR --- Cartan, Palatini, self-coupled spin-2, teleparallel --- the $K$--$Q$ pipeline contains a non-admissible step. The teleparallel formulation is the critical case: its production kernel ($T = d e$) is admissible, but the non-admissibility moves to $Q$. This reveals a *formulation fiber*: properties like energy localizability, $K$-admissibility, and background-dependence vary across physically equivalent formulations. They are fiber artifacts, not physics --- in exactly the sense that the Continuum Hypothesis is independent of ZFC (Volume III). The century-old ``problem of gravitational energy'' is a fiber question mistaken for a physics question.

Third, an *evidentiary gap*: the three cleanest gravitational confirmations (binary pulsar decay, frame-dragging, gravitational waves) are all predictions of linearized GR, which is admissible. The only evidence for the non-admissible $omega and omega$ term is the LIGO merger waveform, extracted via matched filtering against GR-derived templates. The Deser construction shows that non-admissibility is binary --- no consistent theory lies between the admissible linearized theory and the non-admissible full theory --- so the physical status of the nonlinear sector remains genuinely open.

All results are formally verified: 17 by Kleis evaluation, 33 by Z3 structural verification, and 67 by Z3 formulation and truncation analysis.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* general relativity, Cartan geometry, admissible kernel, projected ontology, non-admissibility, connection self-coupling, teleparallel gravity, linearized gravity, diffeomorphism fibers, Weyl tensor, projection sufficiency, Higgs mechanism, singularity annihilation, logarithmic kernel, rotation curves, formulation independence, formal verification, Z3 theorem prover]

#v(1em)


= Introduction

The non-localizability of gravitational energy is one of the oldest puzzles in general relativity. Unlike electromagnetic energy, which is carried by the gauge-invariant field strength $F = d A$, gravitational energy cannot be localized in a coordinate-independent manner: any local expression for it can be transformed to zero at a point by choosing appropriate coordinates. Equivalently, the gravitational energy-momentum ``pseudo-tensor'' is not a true tensor under general coordinate transformations. The standard explanation --- diffeomorphism invariance of the full theory --- is correct but does not pinpoint the algebraic mechanism. Similarly, the problem of time in canonical gravity --- that the Hamiltonian is a pure constraint generating gauge transformations rather than physical evolution --- lacks a structural characterization that connects it to the analogous Yang--Mills phenomenology.

This paper identifies the algebraic mechanism: the non-admissibility of the GR production kernel. In the Projected Ontology (POT) framework, a production kernel $K$ maps a configuration (the tetrad $e^a$, encoding the metric) to curvature (the Riemann tensor $R^a_b$). An observable projection $Q$ then extracts measurable quantities (the Ricci tensor, Einstein tensor, and through the field equations, the energy-momentum tensor). The kernel $K$ is *admissible* if it is linear: $K(e_A + e_B) = K(e_A) + K(e_B)$ and $K(0) = 0$. The admissibility defect $Delta(e_A, e_B) = K(e_A + e_B) - K(e_A) - K(e_B)$ measures how badly linearity fails.

The Cartan curvature formula provides the explicit mechanism:
$ R^a_b = d omega^a_b + omega^a_c and omega^c_b $
This is structurally identical to the Yang--Mills field strength $F = d A + A and A$. The $omega and omega$ term --- the connection self-coupling --- breaks linearity. In the companion paper on Yang--Mills confinement (Volume IV), we showed that $A and A$ produces the Lie bracket $[A, B]$ as the admissibility defect, and derived color confinement as a structural consequence. Here we show that $omega and omega$ produces a nonzero admissibility defect for GR, and derive the three structural consequences listed above.

The key advantage of our approach is that the Cartan pipeline is *computational*: the map tetrad $arrow.r$ connection $arrow.r$ curvature is implemented as a symbolic evaluation chain in Kleis. We do not merely axiomatize non-admissibility --- we compute it, using the Schwarzschild metric as a concrete test case, and isolate the $omega and omega$ term as the source.

The linearized theory provides the control case. When the background is flat, $omega = 0$, so $omega and omega = 0$, and the kernel becomes admissible. This explains why linearized GR has localizable gravitational energy (carried by the Isaacson stress-energy tensor), an external background time (Minkowski), and freely propagating gravitational waves. The transition from linearized to full GR is the transition across the admissibility boundary.

A striking consequence emerges from the Weyl tensor analysis: the Schwarzschild singularity at $r = 0$ --- a genuine curvature singularity where $R_(a b c d) R^(a b c d) arrow.r infinity$ --- lives entirely in $"ker"(Q_("GR"))$. The observable projection maps this divergent curvature to zero ($R_(a b) = 0$). In the POT framework, where physical observables are identified with $"im"(Q)$, the vacuum singularity is not an observable divergence but a structural artifact of the production kernel --- following the same pattern as UV divergence annihilation in quantum field theory.

All results are formally verified: 17 by the Kleis evaluator (symbolic Cartan computations), 33 by the Z3 SMT solver (structural consequences), and 67 by Z3 formulation and truncation analysis.

= GR as a K-Q Pair via Cartan Geometry

We identify the two operators that define GR within the POT framework.

*Production kernel* $K_("GR")$. The production kernel maps a tetrad $e^a$ to the Riemann curvature tensor via Cartan's structure equations in three explicit steps.

*Step 1: Anholonomy coefficients.* Compute the exterior derivatives $d e^a$ and extract the structure coefficients $C^a_(b c)$ defined by:
$ d e^a = 1/2 C^a_(b c) e^b and e^c $
These encode how the tetrad frame fails to be a coordinate basis.

*Step 2: Connection.* The unique torsion-free, metric-compatible connection is:
$ omega^a_b = sum_c 1/2 (C^a_(b c) - C_(b)^(" "a)_(" "c) + C_(c)^(" "a)_(" "b)) e^c $
This is the Koszul formula in the tetrad basis. The connection $omega^a_b$ is a set of $4 times 4$ one-forms, each a linear combination of the tetrad one-forms weighted by the anholonomy coefficients. The nonlinearity enters here: the $C^a_(b c)$ depend on the tetrad components and their derivatives, so $omega$ is a *nonlinear* function of $e$.

*Step 3: Curvature.* The Riemann curvature 2-form is:
$ R^a_b = d omega^a_b + sum_c omega^a_c and omega^c_b $
The first term $d omega$ is linear in $omega$ (hence nonlinear in $e$ through Step 2). The second term $omega and omega$ is *quadratic* in $omega$ --- this is the self-coupling that makes $K_("GR")$ non-admissible, structurally identical to $A and A$ in the Yang--Mills field strength $F = d A + A and A$.

The complete kernel is the composition: $K_("GR")(e) = R(omega(e))$, implemented in Kleis as `compute_riemann(tetrad)`, which chains exterior differentiation (`d1`), the connection solver (`solve_connection`), and wedge products (`wedge`).

*Observable projection* $Q_("GR")$. The observable projection is the Ricci contraction --- the trace of the Riemann tensor over its first and third indices:
$ R_(a b) = sum_(c=0)^3 R^c_(" " a c b) $
This $4 times 4$ symmetric tensor is the part of the curvature that couples to matter via the Einstein field equations $G_(a b) = R_(a b) - 1/2 R g_(a b) = 8 pi G T_(a b)$. The kernel of $Q$ --- what the trace contraction discards --- is the Weyl tensor $C_(a b c d)$: the 10-component trace-free part of the Riemann tensor encoding tidal forces, gravitational waves, and long-range gravitational effects not determined by local matter.

*Verification.* We verify the pipeline on two metrics:

- *Minkowski* (flat): $K_("GR")$ produces zero curvature, $Q compose K$ produces zero Ricci. This is the trivial case: no gravitational field.

- *Schwarzschild* (curved): $K_("GR")$ produces nonzero curvature components depending on $M$ and $r$, while $Q compose K$ produces zero Ricci ($R_(mu nu) = 0$): the Schwarzschild metric is a vacuum solution. This confirms that the kernel operates correctly on a physically nontrivial input.

*Yang--Mills parallel.* The structural isomorphism is exact:

#table(
  columns: (1fr, 1fr, 1fr),
  [*Property*], [*Yang--Mills*], [*General Relativity*],
  [Kernel formula], [$F = d A + A and A$], [$R = d omega + omega and omega$],
  [Configuration], [Gauge connection $A$], [Tetrad $e^a$],
  [Bundle], [Principal $G$-bundle], [Frame bundle $"SO"(3,1)$],
  [Self-coupling], [$A and A$], [$omega and omega$],
  [Defect], [Lie bracket $[A, B]$], [Connection nonlinearity],
)

= The Admissibility Defect

The admissibility defect $Delta$ measures the failure of linearity:
$ Delta(e_A, e_B) = K(e_A + e_B) - K(e_A) - K(e_B) $
For an admissible kernel, $Delta = 0$ for all inputs. For the GR kernel, we compute $Delta$ on concrete tetrads.

*Test case.* We take $e_A$ = Schwarzschild tetrad (parametric in mass $M$) and $e_B$ = a radial perturbation $epsilon dot.c d r$. The defect component $Delta_(0 1 0 1)$ evaluates to a nonzero symbolic expression in $M$, $r$, and $epsilon$. The expression is complex (involving nested square roots and rational functions of $M$ and $r$), which is expected: the connection solver introduces ratios of tetrad components, and the curvature computation adds quadratic wedge products.

The key observation is that $Delta eq.not 0$: the map from tetrad to curvature is *not* linear. Admissibility requires $Delta(e_A, e_B) = 0$ for *all* inputs --- it is a universally quantified property. A single computed counterexample with $Delta eq.not 0$ is therefore sufficient to refute it. The Schwarzschild defect is that counterexample. This proves non-admissibility of the GR kernel as a mathematical fact, not an assumption: the Cartan pipeline computes $Delta$, and the result is nonzero.

*Source identification.* The defect arises from two sources:

1. *Connection nonlinearity.* The connection $omega^a_b$ depends on the anholonomy coefficients $C^a_(b c)$ extracted from $d e^a$, which involve products of tetrad components and their inverses. The connection solver formula $omega^a_b = 1/2 (C^a_(b c) - C^b_(a c) + C^c_(a b)) e^c$ is linear in the $C$'s, but the $C$'s themselves are nonlinear in the tetrad.

2. *Curvature self-coupling.* Even if the connection were linear in the tetrad, the curvature formula $R = d omega + omega and omega$ adds a quadratic term. This is the same mechanism as Yang--Mills: $F = d A + A and A$.

Both sources contribute, but the second is the *structural* source --- it is the same for GR and Yang--Mills, and it persists even when the connection is treated as a fundamental variable (Palatini formalism).

= Isolating the Self-Coupling Term

To pinpoint the nonlinear contribution, we decompose the curvature into its linear and nonlinear parts:
$ R^a_b = underbrace(d omega^a_b, "linear") + underbrace(sum_c omega^a_c and omega^c_b, "nonlinear") $
We compute each part separately using the Kleis Cartan pipeline.

*Minkowski (flat spacetime).* The connection vanishes: $omega^a_b = 0$ for all $a, b$. Therefore $omega and omega = 0$, and the full curvature $R = d omega + 0 = 0$. This is the *admissible limit*: when the self-coupling vanishes, the kernel is (trivially) linear in the tetrad.

*Schwarzschild (curved spacetime).* The connection is nonzero (e.g., $omega^0_1$ depends on $M/r^2$ and $sqrt(1 - 2 M/r)$). The nonlinear part $omega and omega$ is also nonzero: we compute the $(0,1,0,1)$ component and find a nontrivial expression involving products of connection components. The full curvature $R = d omega + omega and omega$ receives contributions from both terms.

This decomposition makes precise the claim that the $omega and omega$ self-coupling is the source of non-admissibility. In the flat limit ($M arrow.r 0$), $omega arrow.r 0$, the self-coupling vanishes, and the kernel crosses the admissibility boundary into the admissible regime.

The structural parallel is again exact:
- Yang--Mills: $A = 0$ (trivial connection) $arrow.r$ $A and A = 0$ $arrow.r$ admissible limit (abelian electrodynamics)
- GR: $omega = 0$ (flat connection) $arrow.r$ $omega and omega = 0$ $arrow.r$ admissible limit (linearized gravity)

= Structural Consequences

Non-admissibility of the GR kernel produces three structural consequences, all verified by Z3.

*C1. Gravitational energy non-localizability.* For an admissible kernel, the image $K(e)$ is invariant under gauge transformations (diffeomorphisms): the curvature is coordinate-independent. For a non-admissible kernel, the image is *not* invariant on diffeomorphism orbits. The energy extracted from curvature inherits this non-invariance: it depends on the coordinate choice, hence cannot be localized. This is the algebraic root of the pseudo-tensor problem.

The chain is: non-admissible $arrow.r$ image not gauge-invariant $arrow.r$ energy not localizable. The contrapositive holds for linearized GR: admissible $arrow.r$ image gauge-invariant $arrow.r$ Isaacson stress-energy tensor is well-defined.

*C2. Problem of time.* In canonical (Hamiltonian) gravity, the total Hamiltonian is a sum of constraints: $H = N cal(H) + N^i cal(H)_i approx 0$. There is no ``true'' Hamiltonian generating physical time evolution. Non-admissibility forces this: the nonlinear structure of the constraints prevents separation into a ``free'' Hamiltonian and gauge generators. In the admissible (linearized) limit, the Minkowski background provides an external time coordinate, and the linearized Hamiltonian generates physical evolution.

*C3. Admissibility boundary.* Full nonlinear GR is non-admissible. Linearized GR (perturbations around flat space) is admissible. The transition between them is the admissibility boundary --- the same structural boundary that separates non-abelian Yang--Mills (confined) from abelian electrodynamics (free).

#table(
  columns: (1fr, 1fr, 1fr),
  [*Property*], [*Full GR (non-admissible)*], [*Linearized GR (admissible)*],
  [Self-coupling $omega and omega$], [Nonzero], [Zero],
  [Energy localizability], [No (pseudo-tensor)], [Yes (Isaacson)],
  [External time], [No (problem of time)], [Yes (Minkowski background)],
  [Gauge structure], [Nonlinear diffeomorphisms], [Linear perturbations],
  [Gravitational waves], [Coupled to background], [Freely propagating],
)

*C4. Degree-of-freedom counting.* A symmetric $4 times 4$ metric tensor has 10 independent components. Diffeomorphism invariance removes 4 gauge degrees of freedom. The Hamiltonian constraint and 3 momentum constraints remove 4 more. The remaining $10 - 4 - 4 = 2$ physical degrees of freedom are the transverse-traceless modes: the two polarizations of gravitational waves. This counting holds for both the full and linearized theories.

= Diffeomorphism Fibers and Non-Localizability

In the Projected Ontology framework, projection fibers are orbits of the gauge group acting on the configuration space. For Yang--Mills, the fibers are gauge orbits $\{g dot.c A : g in cal(G)\}$. For GR, the fibers are *diffeomorphism orbits*: the set of all tetrads related by coordinate transformations, $\{phi^*(e) : phi in "Diff"(M)\}$.

Two tetrads in the same fiber describe the same physical geometry in different coordinates. The projection $Q compose K$ must produce the same observable for both --- this is the content of general covariance. But the intermediate kernel $K$ (tetrad $arrow.r$ curvature) may or may not be invariant on these fibers.

*Admissible kernel (linearized GR).* When the kernel is admissible, the curvature is invariant on diffeomorphism orbits: $K(phi^*(e)) = K(e)$ for all diffeomorphisms $phi$. The image is constant on each fiber. Any predicate on the image --- including gravitational energy --- is therefore fiber-invariant, hence a genuine diffeomorphism-invariant observable. This is why linearized GR admits the Isaacson stress-energy tensor.

*Non-admissible kernel (full GR).* When the kernel is non-admissible, there exist diffeomorphisms $phi$ and tetrads $e$ such that $K(phi^*(e)) eq.not K(e)$: the curvature components change across the fiber. Any predicate that depends on these components --- including gravitational energy density --- is fiber-non-invariant.

By the Main Theorem of projection fiber theory (proved in Volume III): *a predicate is independent of the projection if and only if it is non-invariant on fibers*. Applied to GR:

$ "non-admissible" K arrow.r "image varies on fibers" arrow.r "energy is fiber-non-invariant" arrow.r "energy is independent of observables" $

This *is* non-localizability, derived from the fiber structure. The gravitational energy pseudo-tensor is not a defect of our formalism --- it is a structural necessity: the kernel's non-admissibility forces energy to be invisible to the projection.

The parallel with Yang--Mills confinement is now exact:
- *Yang--Mills:* color charge is non-invariant on gauge fibers $arrow.r$ independent of gauge-invariant observables $arrow.r$ confined
- *GR:* gravitational energy is non-invariant on diffeomorphism fibers $arrow.r$ independent of diffeomorphism-invariant observables $arrow.r$ non-localizable

Both are instances of the same structural theorem, applied to different fiber bundles.

*Gribov analog.* For admissible $K$, diffeomorphism orbits are affine subspaces, so a global coordinate fixing (harmonic gauge, $square.stroked x^mu = 0$) selects one representative per orbit. For non-admissible $K$, orbits are nonlinear manifolds, and no single coordinate system covers the entire solution space. This is the GR analog of the Gribov problem in Yang--Mills: just as there is no global gauge fixing for non-abelian gauge fields, there is no global coordinate system for general spacetimes. The Schwarzschild coordinate singularity at $r = 2 M$ --- requiring Kruskal--Szekeres extension --- is a concrete instance.

*Observable hierarchy.* The fiber structure determines the minimum order of observables:
- *Admissible (linearized):* The metric perturbation $h_(a b)$ is itself gauge-invariant at linear order --- it is an order-1 observable.
- *Non-admissible (full):* Only nonlinear curvature invariants survive as diffeomorphism-invariant observables: the Ricci scalar $R$ (order 2 in derivatives), the Kretschmer scalar $R_(a b c d) R^(a b c d)$ (order 4), the Weyl invariant $C_(a b c d) C^(a b c d)$ (order 4). The metric itself is not an observable.

All fiber-structure results (F1--F11) are Z3-verified.

= Admissibility and the Gravitational Kernels

In Volume V, we showed that the Higgs mechanism in the electroweak sector can be understood as *admissibility restoration*: the Higgs field compensates the Lie bracket defect in the SU(2) kernel, making weak charges observable. Given the structural identity between GR and Yang--Mills at the kernel level ($omega and omega$ versus $A and A$), one might expect a gravitational analog of the Higgs mechanism to exist. We argue that no such mechanism is needed, and that the reason illuminates a fundamental distinction between the two theories.

*The same algebra, different consequences.* Both kernels are non-admissible for the same algebraic reason: quadratic self-coupling. But the gauge symmetries act on different objects:

- In Yang--Mills, the gauge symmetry is *internal*: SU(2) and SU(3) act on an internal charge space (color, weak isospin). These are properties of matter fields that live *in* spacetime. When non-admissibility confines these charges, it hides something inside spacetime. Without the Higgs, weak charges would be invisible --- the theory would predict unobservable $W$ and $Z$ bosons.

- In GR, the gauge symmetry is *external*: diffeomorphisms act on spacetime *itself*. Gravity is not a field in spacetime --- it IS spacetime. Non-admissibility makes gravitational energy non-localizable, but it does not hide the observable effects of gravity. Tidal forces, gravitational lensing, gravitational waves, frame-dragging --- all are directly measured. You cannot confine spacetime. You are already in it.

*What GR does explain.* GR is supported by several classes of observation, of varying directness:

- *Binary pulsar decay*: The Hulse--Taylor pulsar energy loss matches the quadrupole formula to 0.2% accuracy over decades of observation. This is a direct measurement --- the pulse arrival times shift cumulatively, with no large systematics to subtract.
- *Frame-dragging*: Gravity Probe B reported Lense--Thirring precession at $37.2 plus.minus 7.2$ mas/yr, consistent with GR's prediction of 39.2 mas/yr. The measurement required extensive modeling of systematic effects (electrostatic patch torques) that exceeded the signal itself. LAGEOS satellite tracking yields consistent results after subtracting Earth's gravitational multipole contributions.
- *Gravitational waves*: LIGO detected coincident strain signals in two detectors consistent with a binary inspiral waveform. The signal extraction uses matched filtering --- cross-correlation against GR-derived templates --- so the detection and the theory are not fully independent.

No ``gravitational Higgs'' was needed for nature to produce these observables, with the binary pulsar providing the most model-independent confirmation.

*What GR does not explain.* Galaxy rotation curves are flat: orbital velocity $v(r)$ remains approximately constant far beyond the visible matter distribution. The GR kernel (and its Newtonian limit) predicts Keplerian decline $v(r) prop r^(-1\/2)$, in sharp disagreement with observation. The standard resolution is to postulate dark matter --- unobserved mass that modifies $T_(mu nu)$ to produce the observed curves. But this means GR requires invisible matter to match galactic observations, a move that is structurally analogous to adding an unobserved field.

In Volume III, we showed that a different admissible kernel --- the logarithmic projection kernel $K_("log")$ with Green's function $G(r, r') = -(kappa \/ 2 pi) ln |r - r'|$ --- produces $v(r) = "const"$ directly, without dark matter. The logarithmic kernel is admissible (linear), so it does not raise the Higgs question. But it predicts a different gravitational potential ($ln r$ instead of $1\/r$) and does not produce frame-dragging or gravitational waves.

The situation is therefore: GR is correct for strong-field, dynamical phenomena (waves, frame-dragging, binary pulsars), while the logarithmic kernel matches galactic rotation curves without auxiliary hypotheses. Neither kernel, alone, accounts for all gravitational observations.

*The admissibility dilemma.* The success of the logarithmic kernel creates a genuine interpretive tension. The kernel that explains flat rotation curves is admissible. Could this be evidence that the universe prefers admissible kernels --- that non-admissibility in GR is, after all, an obstruction that nature corrects, perhaps through some restoration mechanism we have not yet identified?

The answer is not clear, and we must be honest about this. The evidence pulls in two directions:

*For* a preference for admissibility: the logarithmic kernel (admissible) explains rotation curves that the GR kernel (non-admissible) cannot without dark matter.

*Against* a preference for admissibility: the Newtonian kernel is also admissible, and it too fails at rotation curves --- it predicts Keplerian decline, just like GR's weak-field limit. Admissibility alone does not select the correct kernel. The logarithmic kernel succeeds not because it is admissible, but because it has a specific Green's function ($ln |r - r'|$) that produces the right potential. Newton's kernel has a different Green's function ($1\/|r - r'|$) that produces the wrong one.

This means we face a four-kernel landscape:

#table(
  columns: (1fr, 1fr, 1fr, 1fr, 1fr),
  [*Property*], [*Newton*], [*Logarithmic*], [*Linearized GR*], [*Full GR*],
  [Admissible?], [Yes], [Yes], [Yes], [No],
  [Rotation curves?], [No (Keplerian)], [Yes (flat)], [No (Keplerian)], [No (needs dark matter)],
  [Frame-dragging?], [No], [No], [Yes], [Yes],
  [Gravitational waves?], [No], [No], [Yes], [Yes],
  [Binary pulsar?], [No], [No], [Yes], [Yes],
)

*A critical observation.* The three observations commonly cited as confirming full GR --- binary pulsar decay, frame-dragging, and gravitational waves --- are all predictions of *linearized* GR. The quadrupole formula for gravitational radiation is derived in the linearized/post-Newtonian regime. The Lense--Thirring effect comes from the gravitomagnetic potential $h_(0 i)$ of the linearized metric. The gravitational wave equation is a linearized Einstein equation. None of these observations requires the $omega and omega$ self-coupling that makes full GR non-admissible.

The only claimed evidence for the nonlinear regime is the specific waveform shape during the final merger cycles in LIGO data, where numerical relativity (full nonlinear GR) produces templates that linearized theory cannot. But this is precisely the measurement with the most model-dependent extraction: the signal is identified through matched filtering against templates built from the nonlinear theory.

This shifts the landscape significantly. Three admissible kernels (Newton, logarithmic, linearized GR) account for different subsets of gravitational observations. Linearized GR is the most successful admissible kernel: it explains frame-dragging, gravitational waves, and binary pulsar decay --- everything except rotation curves. The logarithmic kernel explains rotation curves but not the dynamical phenomena. The only evidence that requires going *beyond* admissible kernels into the non-admissible full GR is the LIGO merger waveform, which is the least model-independent of all the observations.

We cannot conclude that the universe systematically prefers admissible kernels. But we can observe that the evidence for the *non-admissible* part of GR --- the $omega and omega$ term, the very structure that creates the non-admissibility --- is thinner than commonly presented. The cleanest measurements do not distinguish linearized from full GR.

*The patching problem.* The situation is in fact more uncomfortable than a three-way competition. In Volume III, we introduced a core radius $R_c$ and defined the logarithmic kernel's domain as $r >= R_c$, with the inner region ($r < R_c$) governed by the standard baryonic mass distribution --- which is to say, by the Newtonian potential. The matching condition at $R_c$ fixes $kappa = 2 pi \/ R_c$ and produces the flat outer rotation curve. But this is a *patch*: two different kernels stitched together at a boundary, not a single unified kernel that reproduces both the inner rise and the outer plateau from one structure.

This is structurally analogous to MOND (Milgrom, 1983), which introduces an acceleration scale $a_0$ and applies a modified force law below it while retaining Newton above. Both approaches succeed empirically in their respective outer regimes; both involve a transition parameter ($R_c$ or $a_0$); and neither derives a single kernel valid at all scales from first principles. The POT formulation has the advantage that its transition is spatial ($R_c$) rather than dynamical ($a_0$), and the logarithmic kernel has a concrete Green's function interpretation. But the structural similarity is real, and we should not overclaim: the flat rotation curve result is a regime-specific success, not a demonstration that the logarithmic kernel is the universal gravitational kernel.

*The role of the observable projection.* What we *can* say is that the Higgs question --- does non-admissibility hide essential observables? --- has a clear answer for GR:

#table(
  columns: (1fr, 1fr, 1fr),
  [*Property*], [*Yang--Mills*], [*GR*],
  [Gauge symmetry acts on], [Internal charge space], [Spacetime itself],
  [What enters $ker(Q)$], [Charges (color, weak isospin)], [Energy density],
  [What remains in im($Q$)], [Not enough --- charges missing], [Geometry intact],
  [Non-admissibility hides essential observables?], [*Yes*], [*No*],
)

In Yang--Mills, non-admissibility pushes charges into $ker(Q)$, and $"im"(Q)$ without the Higgs is missing the weak sector entirely. The Higgs moves charges from $ker(Q)$ to $"im"(Q)$, restoring observability. In GR, non-admissibility pushes energy density into $ker(Q)$, but the geometry --- tidal forces, geodesic deviation, wave strain --- stays in $"im"(Q)$. Non-admissibility does not hide the geometric observables that GR correctly predicts.

But the rotation curve problem does not fit neatly into either the Higgs question or the kernel-range question. It is not that non-admissibility *hides* the flat rotation curves --- the Newtonian kernel is admissible and also fails. It is that the *specific form* of the GR kernel (and its Newtonian limit) produces the wrong potential at galactic scales. Whether this points to a different kernel, a modification of GR, or genuinely invisible matter remains an open question --- one that the POT framework identifies precisely but does not resolve.

*Linearization and the status of the nonlinear term.* Writing $g_(mu nu) = eta_(mu nu) + h_(mu nu)$ and linearizing produces an admissible kernel $K_("eff")(h)$. The standard view treats this as an approximation --- a human simplification of the full nonlinear theory. But the evidentiary situation is less clear than this framing suggests. The cleanest gravitational observations (binary pulsar, frame-dragging, gravitational waves) are all predictions of the linearized theory. The only claimed evidence for the nonlinear $omega and omega$ term is the specific merger waveform in LIGO data, extracted through matched filtering against templates built from the nonlinear theory. Whether the linearized kernel is an approximation to the full theory, or the full theory is a mathematical extension of the physical kernel, is an open question.

*What we can and cannot conclude.* The distinction between internal and external gauge symmetry is structural: Yang--Mills gauge symmetry acts on internal charge space (can confine charges), while GR gauge symmetry acts on spacetime itself (cannot confine geometry). This explains why non-admissibility requires a Higgs in Yang--Mills but does not hide geometric observables in GR.

But whether the universe ultimately operates through admissible kernels at all scales --- whether the success of the logarithmic kernel at galactic scales is a clue --- is an open question. POT provides the language to state the question precisely: three gravitational kernels, two admissible and one not, each correct in a different regime, none complete. The framework does not tell us which principle selects the right kernel in each regime. That remains to be discovered.

Results R1--R4 verify the formal linearization claims in Z3.

= Formulation Independence of Non-Admissibility

The analysis so far uses the Cartan formulation of GR: the production kernel $K$ maps a tetrad $e^a$ to the Riemann curvature via $R = d omega + omega and omega$, and the observable projection $Q$ extracts the Ricci tensor. In this formulation, $K$ is non-admissible (due to $omega and omega$) and $Q$ is admissible (Ricci contraction is linear). A natural question arises: is non-admissibility a property of *GR itself*, or of the *Cartan formulation specifically*? If a different formulation has an admissible $K$, the claims of this paper would need revision.

We analyze four formulations. All produce the same field equations and the same physical predictions (except the linearized theory, which is an approximation).

*1. Cartan* (this paper). $K$: tetrad $arrow.r$ connection $arrow.r$ curvature $R = d omega + omega and omega$. $Q$: Riemann $arrow.r$ Ricci. $K$ non-admissible, $Q$ admissible. Non-admissibility is in $K$.

*2. Spin-2 field on flat spacetime* (Fierz--Pauli, Deser 1970). The free theory --- a massless spin-2 field on Minkowski background --- is exactly linearized GR: $K$ maps the perturbation $h_(mu nu)$ to the linearized Riemann tensor. This $K$ is admissible (linear in $h$). Requiring consistent self-coupling --- gravity must couple to its own energy --- introduces cubic, quartic, and higher terms. Deser showed the infinite series sums to the full Einstein--Hilbert action. The fully self-coupled $K$ is non-admissible. $Q$ remains admissible (Ricci contraction). Non-admissibility is in $K$.

The Deser construction has a fine structure that reveals the nature of the admissibility boundary. Label the truncation of the self-coupling series at order $n$. At $n = 0$ (free spin-2), $K$ is admissible and the theory is self-consistent. At $n = infinity$ (full GR), $K$ is non-admissible and the theory is again self-consistent --- the infinite sum closes on itself, restoring diffeomorphism invariance. But at any finite $n >= 1$, the truncation is *both non-admissible and inconsistent*: the self-coupling at order $n$ generates source terms at order $n + 1$ that the truncation cannot account for, breaking gauge invariance. The admissibility boundary between linearized and full GR is not a threshold that can be approached gradually --- it is a *gap*. Non-admissibility is binary: any nonzero coupling order makes $K$ non-admissible. And every finite truncation with self-coupling is physically inconsistent. Only the full infinite sum restores consistency, but at the price of non-admissibility. Self-gravitation (coupling to gravitational energy) and non-admissibility are inseparable: you cannot have one without the other. This also clarifies the status of post-Newtonian approximations (1PN, 2PN, etc.): they are finite truncations used as calculational tools, borrowing their validity from the full non-admissible theory, not self-consistent field theories in their own right.

*3. Teleparallel Equivalent of GR (TEGR)*. This is the critical case. The Weitzenböck connection is flat: curvature vanishes identically. The gravitational field strength is *torsion* $T^a = d e^a$ (in the Weitzenböck gauge where the spin connection vanishes). The production kernel $K$: tetrad $arrow.r$ torsion is *admissible* --- $d e^a$ is linear in $e$. But the observable projection $Q$ extracts the torsion scalar $T = S_a^(mu nu) T^a_(mu nu)$, which is quadratic in torsion. $Q$ is *non-admissible*. The non-admissibility moves from $K$ to $Q$: same physics, different pipeline.

*4. Palatini formulation*. The connection $Gamma^lambda_(mu nu)$ is treated as an independent variable, not derived from the metric. The Riemann tensor is still $R = d Gamma + Gamma and Gamma$: the $Gamma and Gamma$ self-coupling persists. $K$ is non-admissible. $Q$ (Ricci contraction) is admissible. Varying the action with respect to $Gamma$ recovers the Levi-Civita connection and the standard metric formulation. Non-admissibility is in $K$.

#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  [*Formulation*], [*$K$ admissible?*], [*$Q$ admissible?*], [*Non-admissibility in*],
  [Cartan], [No ($omega and omega$)], [Yes (Ricci trace)], [$K$],
  [Spin-2 free], [Yes (linear in $h$)], [Yes], [Nowhere],
  [Spin-2 full], [No (self-coupling)], [Yes], [$K$],
  [TEGR], [Yes ($T = d e$, linear)], [No ($T$ scalar quadratic)], [$Q$],
  [Palatini], [No ($Gamma and Gamma$)], [Yes], [$K$],
)

*The invariant.* Every formulation of full (non-linearized) GR has at least one non-admissible step in the $K$--$Q$ pipeline. The formulation determines *where* it appears --- in $K$ for Cartan, Palatini, and the self-coupled spin-2 theory; in $Q$ for TEGR --- but it cannot be eliminated. Non-admissibility is a property of GR itself, not of any particular choice of variables.

The sole exception is the free spin-2 theory (linearized GR), in which both $K$ and $Q$ are admissible. But this is not physically equivalent to full GR --- it is an approximation that discards the self-coupling.

*Implication for the Projection Sufficiency Principle.* The claim stated earlier --- that the obstruction in GR resides in $K$, not in $Q$ --- is formulation-dependent. In the Cartan formulation it is true. In TEGR it is reversed: $K$ is admissible and $Q$ carries the non-admissibility. The formulation-independent statement is: *the $K$--$Q$ pipeline for GR necessarily contains a non-admissible step; changing variables can move it between $K$ and $Q$, but cannot remove it.*

This is a stronger result than the Cartan-specific claim. It means that any attempt to reformulate GR so that both $K$ and $Q$ are admissible must change the physics --- as linearization does, losing the self-coupling and the strong-field effects it predicts.

*Observational versus structural comparison.* All full GR formulations are physically equivalent: they make the same predictions. For the weak-field observables --- frame-dragging, gravitational waves, binary pulsar decay --- the linearized theory agrees with the full formulations (these are linearized GR results). All formulations fail at flat rotation curves. The discriminator is the *strong-field regime*: only the full formulations predict specific merger waveforms (the nonlinear chirp and ringdown). The linearized theory has no strong-field regime. The *structural* comparison reveals the pipeline differences:

#table(
  columns: (1.3fr, 1fr, 1fr, 1fr, 1fr, 1fr),
  [*Property*], [*Cartan*], [*Spin-2 free*], [*Spin-2 full*], [*TEGR*], [*Palatini*],
  [Admissible $K$?], [No], [Yes], [No], [Yes], [No],
  [Admissible $Q$?], [Yes], [Yes], [Yes], [No], [Yes],
  [Non-admissibility in], [$K$], [Nowhere], [$K$], [$Q$], [$K$],
  [Energy localizable?], [No], [Yes], [No], [Yes], [No],
  [Merger waveforms?], [Yes], [No], [Yes], [Yes], [Yes],
  [Background needed?], [No], [Yes], [Yes], [No], [No],
  [Dynamical variable], [Tetrad $e^a$], [$h_(mu nu)$], [$h_(mu nu)$], [Tetrad $e^a$], [Connection $Gamma$],
  [Equiv. to full GR?], [Yes], [No], [Yes], [Yes], [Yes],
)

The energy-localizability row is the structural fingerprint of $K$-admissibility: when $K$ is admissible (TEGR, linearized), its image is gauge-invariant, and gravitational energy density can be defined. When $K$ is non-admissible (Cartan, Palatini, spin-2 full), the image varies across diffeomorphism orbits, and gravitational energy is non-localizable. Result V3 is striking: Cartan and TEGR are physically equivalent, yet energy is localizable in TEGR and not in Cartan --- the same physics, formulated so that the pipeline's non-admissibility sits in a different place, yields different intermediate structure while agreeing on all observables.

*The formulation fiber.* This observation has a deeper consequence. Apply the POT framework at a meta-level: the ``configurations'' are the formulations themselves, the ``projection'' maps formulations to physical predictions, and the ``fiber'' is the set of formulations producing identical physics. A property that is constant on fibers is *physical*; a property that varies across fibers is a *fiber artifact*. Energy localizability varies across the fiber $brace.l$Cartan, TEGR$brace.r$: Cartan says energy is not localizable, TEGR says it is, yet both produce the same field equations, the same gravitational waves, the same frame-dragging. Therefore energy localizability is a fiber artifact --- it is not a property of gravity, but a property of the formulation. Similarly, $K$-admissibility, $Q$-admissibility, and background-dependence are fiber properties. The only admissibility property that is physical (constant on all fibers) is pipeline non-admissibility: every full GR formulation agrees that the $K$--$Q$ pipeline contains a non-admissible step. The century-old ``problem of gravitational energy'' --- whether gravitational energy can be localized --- is, in the language of projection fiber theory, a question about where one sits in the formulation fiber, not a question about physics. This is an instance of the general principle established in Volume III: a statement is independent of a theory if and only if it varies across models within the same projection fiber. Energy localizability varies across the formulation fiber, so it is *independent* of the physics of GR --- undecidable from the field equations alone, exactly as the Continuum Hypothesis is undecidable from the axioms of ZFC.

All 67 results --- 49 formulation comparisons and 18 truncation-ladder properties --- are Z3-verified in the companion theory file.

= The Content of ker(Q): The Weyl Tensor

The observable projection $Q_("GR")$ extracts the Ricci tensor from the Riemann curvature. What does it discard? The answer is the *Weyl tensor* $C_(a b c d)$ --- the trace-free part of the Riemann tensor that encodes tidal forces, gravitational radiation, and the long-range gravitational field.

In four dimensions, the Riemann tensor has 20 independent components. The Ricci tensor has 10. The remaining 10 components form the Weyl tensor:
$ C_(a b c d) = R_(a b c d) - ("Ricci terms") $
The Weyl tensor is the content of $"ker"(Q_("GR")):$ it is curvature that is *not* determined by the local energy-momentum tensor.

*Computation: Schwarzschild.* For the Schwarzschild vacuum ($T_(a b) = 0$), the Einstein equations give $R_(a b) = 0$: the Ricci tensor vanishes. But the Riemann tensor is *nonzero* --- it produces the tidal forces that stretch and compress freely falling objects. Therefore: $"im"(K_("GR")) subset.eq "ker"(Q_("GR"))$ for vacuum. *All* of the curvature that $K$ produces is projected away by $Q$.

We compute this explicitly using the Kleis Cartan pipeline: the Weyl component $W^0_("  1 01")$ for Schwarzschild is nonzero (depending on $M$ and $r$), confirming that the vacuum Riemann tensor has nontrivial content in $"ker"(Q_("GR"))$.

*Minkowski.* For flat spacetime, the Riemann tensor vanishes entirely: $W^0_("  1 01") = 0$. The kernel of $Q$ is empty for flat space --- there is nothing to project away.

*Physical interpretation.* The Weyl tensor is responsible for:
- *Tidal forces:* the relative acceleration of nearby geodesics (geodesic deviation)
- *Gravitational waves:* the two transverse-traceless polarizations $h_+$ and $h_times$ are Weyl modes
- *Gravitational lensing:* light deflection by massive objects (even in vacuum)
- *Black hole hair:* the Schwarzschild and Kerr metrics are entirely characterized by their Weyl tensor

All of these are physical effects that are *not* constrained by local matter --- they are the content of the kernel of $Q$. In POT language: $"ker"(Q_("GR"))$ is non-empty and physically rich, containing the degrees of freedom that propagate freely through vacuum.

*Where is the obstruction?* In the Cartan formulation, $Q_("GR")$ is well-defined and computable: it is the Ricci contraction, and we verified it on Schwarzschild ($R_(mu nu) = 0$) and Minkowski ($R_(mu nu) = 0$). The non-admissibility resides in $K$: the $omega and omega$ self-coupling. However, as shown in Section 8, this assignment is formulation-dependent. In the teleparallel equivalent of GR, $K$ (torsion $T = d e$) is admissible and the non-admissibility moves to $Q$ (the torsion scalar is quadratic). The formulation-independent statement is: the $K$--$Q$ pipeline necessarily contains a non-admissible step. For the Weyl tensor analysis here, the relevant point is that $Q$ (Ricci contraction) is admissible in the Cartan formulation, so $"ker"(Q)$ is well-defined and computable regardless of where the pipeline's non-admissibility resides.

= Prior Work

The algebraic and structural phenomena identified here have been studied from various perspectives. We highlight connections to the most relevant prior work and clarify where our contribution lies.

*Curiel (2019).* Curiel proved rigorously that no gravitational stress-energy tensor satisfying natural physical conditions can exist in GR. Our result reaches the same conclusion by a different route: we identify the *algebraic mechanism* (the $omega and omega$ defect) that prevents energy localization, and we connect it to the Yang--Mills confinement chain through the shared non-admissibility structure. Curiel's work establishes the impossibility; ours explains the algebraic root and places it in a cross-domain framework.

*De Vuyst, Eccles, Hoehn, and Kirklin (2024).* These authors identified ``linearization instabilities'' in GR: solutions of the linearized theory that do not integrate to exact solutions of the full nonlinear theory. In our framework, linearization instabilities are precisely the *admissibility boundary*: the transition from admissible (linearized) to non-admissible (full) kernel. Solutions that exist in the admissible regime but not in the non-admissible regime are structural artifacts of the boundary crossing. Our characterization is algebraic (the vanishing or non-vanishing of $omega and omega$) rather than geometric, providing a complementary viewpoint.

*Ashtekar (1986) and the connection formulation.* Ashtekar reformulated GR as a gauge theory with SO(3,1) (or SL(2,$bb(C)$)) connection variables, bringing GR closer to Yang--Mills in mathematical structure. Our work uses the same Cartan connection framework but extracts a different conclusion: the admissibility defect. Ashtekar's formulation enables loop quantum gravity; our framework classifies the kernel $K$ structurally, without committing to a specific quantization program.

*Trautman and Sardanashvily: metric as gravitational Higgs.* Trautman (1979) and later Sardanashvily proposed that the metric tensor in GR plays a role analogous to the Higgs field: it breaks the GL(4,$bb(R)$) symmetry of the frame bundle to SO(3,1). Our linearization-as-restoration analysis (Section 7) formalizes a related but distinct claim: the *background metric* $eta$ restores admissibility by absorbing the $omega and omega$ defect, paralleling how the Higgs restores admissibility by absorbing the Lie bracket defect. The key difference we identify is fragility: the GR restoration is a non-dynamical choice, not a physical mechanism.

*ADM formalism (1959).* Our degree-of-freedom counting $10 - 4 - 4 = 2$ reproduces the result of the ADM (Arnowitt--Deser--Misner) canonical formulation of GR, which identifies the Hamiltonian and momentum constraints and establishes that the gravitational field has exactly 2 propagating degrees of freedom: the transverse-traceless polarizations $h_+$ and $h_times$. LIGO's detection of gravitational waves (GW150914, 2015) reported a signal consistent with exactly 2 polarizations, using matched filtering against GR-derived waveform templates. Our framework arrives at the same count from the POT structure: the 4 gauge DOF are fiber directions (diffeomorphism orbits), the 4 constraints are $"ker"(Q)$ content (projected away), and the 2 surviving modes are $"im"(Q compose K)$ --- what passes through both production and projection.

*Logarithmic potential and flat rotation curves.* In a companion paper, we showed that a logarithmic Green's function $K(r) tilde log(r)$ reproduces flat galactic rotation curves without dark matter. This raises the question: can the logarithmic kernel be derived from the GR weak-field limit? The answer is no. The GR weak-field kernel is the Newtonian $1/r$ potential, producing Keplerian falloff $v tilde 1/sqrt(r)$. The logarithmic correction requires physics beyond the standard linearization. Our framework makes this precise: the GR kernel (via Cartan) gives $1/r$ in the admissible limit; any departure from $1/r$ requires either a modification of $K$ (e.g., additional fields, modified gravity) or a non-perturbative effect from the non-admissible regime.

= Singularity Annihilation: Vacuum Singularities in ker(Q)

The Schwarzschild singularity at $r = 0$ is a genuine curvature singularity: the Kretschmer scalar $R_(a b c d) R^(a b c d) arrow.r infinity$. Yet $Q_("GR")$ maps this divergent curvature to *zero*: $R_(a b) = 0$ everywhere, including at the singularity. The entire divergence lives in $"ker"(Q_("GR"))$ --- the Weyl tensor sector. $Q$ annihilates the infinity --- not in the sense that the Riemann tensor ceases to diverge (it does not; the curvature invariant $R_(a b c d) R^(a b c d) arrow.r infinity$ remains), but in the precise sense that the observable projection maps the divergent input to a finite (in fact, zero) output. The divergence is real in the production layer; it is absent from the observable layer.

The mechanism is identical to the annihilation of UV divergences in QFT. In the companion $phi^4$ paper (Volume I), we showed that one-loop divergences in Feynman integrals live in $"ker"(Q)$: the divergent parts are scheme-dependent and do not contribute to physical observables. The observable projection $Q$ extracts only the finite, scheme-independent residue. Here, $Q_("GR")$ performs the same operation on the Schwarzschild singularity: it extracts the Ricci tensor (the part coupled to matter), which is zero, discarding the divergent Weyl tensor (the part encoding tidal forces).

*Singularity classification by $Q$.* The key structural distinction is between singularities that live in $"ker"(Q)$ and those that live in $"im"(Q)$:

#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  [*Singularity type*], [*Ricci*], [*Weyl*], [*$Q$ annihilates?*],
  [Vacuum (Schwarzschild, Kerr)], [$R_(a b) = 0$], [Diverges], [*Yes* --- in $"ker"(Q)$],
  [Matter (collapsing star)], [$R_(a b) arrow.r infinity$], [Diverges], [*No* --- in $"im"(Q)$],
  [Cosmological (Big Bang)], [$R_(a b) arrow.r infinity$], [May diverge], [*No* --- in $"im"(Q)$],
)

For vacuum singularities, the entire curvature is Weyl: $R_(a b c d) = C_(a b c d)$ when $R_(a b) = 0$. The observable projection sees nothing but ``vacuum'' ($T_(a b) = 0$). The singularity --- the divergent tidal stretching, the infinite curvature --- is invisible to $Q$. It is a *structural artifact of the production kernel*, not a feature of the observable output.

To be precise: the Schwarzschild singularity at $r = 0$ is *not* a coordinate artifact --- the curvature invariant $R_(a b c d) R^(a b c d) arrow.r infinity$ is diffeomorphism-invariant and cannot be removed by coordinate choice (unlike the coordinate singularity at $r = 2 M$, which is removable). What the POT framework establishes is a different and stronger claim: *the singularity is a projection artifact*. The divergent curvature is geometrically real, but it resides entirely in $"ker"(Q_("GR"))$ --- the sector that the observable projection discards. The shift is not from geometry to coordinates, but from geometry to the production--observation distinction: the singularity is produced by $K$ but erased by $Q$. A subtlety: in the teleparallel formulation (Section 8), $K$ is admissible and gravitational energy is localizable, so the singularity corresponds to a well-defined infinite energy concentration --- not merely undefined divergent curvature. $Q$ still annihilates it (the field equations still yield vacuum), but what is being annihilated is richer in TEGR than in Cartan. Whether the singularity is ``just geometry'' or ``localized infinite energy'' is a formulation fiber property (Section 8); its annihilation by $Q$ is not.

*The observability criterion.* Within Projected Ontology Theory, physical observables are identified with $"im"(Q)$: a quantity is physically accessible if and only if it survives the observable projection. This criterion formalizes standard practice in field theory, where gauge-dependent quantities (Yang--Mills), coordinate-dependent quantities (GR), and scheme-dependent quantities (QFT renormalization) are not regarded as physical. Consequently, divergences that lie entirely in $"ker"(Q)$ do not correspond to observable singularities but to structural features of the production kernel.

For matter singularities, the Ricci tensor diverges along with the Weyl tensor: both $"im"(Q)$ and $"ker"(Q)$ contain divergent content. $Q$ does not annihilate these. The physical energy-momentum $T_(a b) arrow.r infinity$ is a genuine observable divergence.

*Structural parallel.* Vacuum singularities in GR occupy exactly the same structural position as:
- *UV divergences in QFT:* live in $"ker"(Q)$, renormalized away, do not affect observables
- *Ghost fields in Yang--Mills:* live in $"ker"(Q)$, cancel unphysical degrees of freedom
- *Color charge:* non-invariant on gauge fibers, invisible to gauge-invariant observables

By the Main Theorem of projection fiber theory, a quantity that lives in $"ker"(Q)$ is *independent* of the observable projection. The vacuum singularity satisfies this: it is a feature of the production kernel's output that the projection erases.

*Implication for singularity resolution.* If vacuum singularities are $"ker"(Q)$ phenomena, and if the POT framework treats $"ker"(Q)$ content as structurally analogous to gauge artifacts and scheme-dependent quantities, then a natural question arises: *is the vacuum singularity a structural artifact of the classical kernel $K_("GR")$?* In QFT, UV divergences in $"ker"(Q)$ disappear when one uses a better definition of $K$ (lattice regularization, dimensional regularization). Could the Schwarzschild singularity disappear with a better (non-perturbative, quantum) definition of $K_("GR")$? This would give the widespread expectation in quantum gravity --- that singularities are resolved by quantum effects --- a precise structural home: *singularity resolution means finding a kernel $K$ where the divergent Weyl component does not arise*.

All singularity results (S1--S6) are Z3-verified.

= Discussion

The central result of this paper is that General Relativity and Yang--Mills theory share the same non-admissibility mechanism: quadratic self-coupling in the curvature formula. This is not an analogy --- it is a structural identity:

$ R = d omega + omega and omega quad quad quad quad F = d A + A and A $

Both are curvature 2-forms of connections on fiber bundles (the frame bundle $"SO"(3,1)$ for GR, the principal $G$-bundle for Yang--Mills). The $omega and omega$ and $A and A$ terms break kernel admissibility in exactly the same way, producing nonzero admissibility defects that propagate through the same structural chain to physical consequences.

Yet the physical consequences differ:

- *Yang--Mills non-admissibility* $arrow.r$ color confinement (charges unobservable) $arrow.r$ Higgs needed
- *GR non-admissibility* $arrow.r$ energy non-localizability $arrow.r$ but geometry observable $arrow.r$ no restoration needed

*The re-interpretation of $Q$.* In the Yang--Mills paper (Volume V), we presented the observable projection $Q$ as primarily an algebraic filter: it takes the output of $K$ and extracts the observable content. The obstruction was located entirely in $K$, and $Q$ was treated as passive. The present paper forces a deeper reading of $Q$'s role.

The critical observation is this: *the same non-admissible algebra produces different needs for restoration depending on what $Q$ projects onto*. In Yang--Mills, $Q$ needs charges. Non-admissibility confines them. Therefore $"im"(Q)$ is incomplete, and a restoring mechanism (the Higgs) must move charges from $"ker"(Q)$ back into $"im"(Q)$. In GR, $Q$ needs geometry. Non-admissibility hides energy density, but geometry --- tidal forces, wave strain, geodesic deviation --- remains in $"im"(Q)$. The projection is already complete. No restoration is needed.

This means $Q$ is not merely a passive filter. It determines *whether non-admissibility is a problem*. The same algebraic defect in $K$ is catastrophic or benign depending on what $Q$ requires. Put differently: the algebra of $K$ is the same, but $Q$ decides whether the universe needs to respond with a dynamical mechanism.

We can state this as a principle:

#align(center)[#box(stroke: 0.5pt, inset: 8pt)[
*Projection Sufficiency Principle:* Non-admissibility of $K$ requires a dynamical restoration mechanism if and only if the essential observables of the theory lie in the sector that non-admissibility pushes into $"ker"(Q)$.
]]

For Yang--Mills: essential observables include charges $arrow.r$ charges confined by non-admissibility $arrow.r$ restoration required. For GR: essential observables are geometric $arrow.r$ geometry survives non-admissibility $arrow.r$ no restoration required.

A subtlety: the Projection Sufficiency Principle refers to the $K$--$Q$ pipeline as a whole. As shown in Section 8, the location of non-admissibility within the pipeline is formulation-dependent: in the Cartan formulation it resides in $K$; in the teleparallel formulation it resides in $Q$. The principle applies regardless of where the non-admissible step sits, because it asks whether the *composite* pipeline $Q compose K$ hides essential observables.

This reframes the relationship between the Higgs mechanism and GR. The question is not ``where is the gravitational Higgs?'' The question is ``does $Q_("GR")$ need one?'' For the Higgs question specifically --- does non-admissibility hide essential observables? --- the answer is no. The binary pulsar, frame-dragging measurements, and gravitational wave detections are all consistent with geometric predictions of GR. Non-admissibility does not confine geometric observables in the way it confines charges in Yang--Mills.

But the flat rotation curve problem introduces a deeper uncertainty. The admissible logarithmic kernel (Volume III) explains galactic rotation curves without dark matter, where the non-admissible GR kernel cannot. Read one way, this is evidence that the universe prefers admissible kernels --- that non-admissibility in GR is a genuine obstruction. But the Newtonian kernel is also admissible and also fails at rotation curves, producing Keplerian decline just like GR's weak-field limit. Admissibility does not select the correct kernel; the specific form of the Green's function does. We therefore cannot conclude that the universe systematically honors or violates admissibility. The data underdetermines the principle.

*$Q$ as the arbiter of physical consequences.* This interpretation promotes $Q$ from a passive algebraic operation to the object that determines the *physical meaning* of non-admissibility. $K$ produces the same kind of defect in both theories. But:

- $Q_("YM")$ + non-admissible $K_("YM")$ = *incomplete observables* $arrow.r$ dynamical fix needed
- $Q_("GR")$ + non-admissible $K_("GR")$ = *complete observables* $arrow.r$ no fix needed

The universe responds to the combination $(K, Q)$, not to $K$ alone. The Higgs is not a response to non-admissibility per se --- it is a response to *insufficient projection*. Where projection is sufficient, non-admissibility is tolerated.

*$Q_("GR")$ is concrete and computable.* Unlike in quantum gravity --- where the nature of the observable projection is an open question tied to the measurement problem --- $Q_("GR")$ in classical GR is fully determined: it is the Ricci contraction. We computed $Q compose K$ on Schwarzschild and obtained $R_(mu nu) = 0$, confirming that the projection works correctly. The kernel of $Q$ is the Weyl tensor, which we computed explicitly.

*The intermediate layer is not observable.* The Riemann tensor is the *intermediate* of the GR pipeline: it is the output of $K$ (production) and the input to $Q$ (observation). Stating that ``the Riemann tensor diverges at $r = 0$'' is a statement about this intermediate layer --- not about $"im"(Q)$. In the POT framework, attributing physical reality to the intermediate is a category error: it is structurally equivalent to asking what an electron is doing between measurements, what momentum a virtual photon carries, or what value a gauge field takes before gauge fixing. Only $"im"(Q)$ corresponds to physical observables. The curvature invariant $R_(a b c d) R^(a b c d) arrow.r infinity$ is a property of the intermediate layer. The observable output is $Q$'s result: $R_(a b) = 0$. Vacuum. Nothing diverges in the projection.

*Admissibility as a classification principle.* The admissibility boundary organizes physical theories:

- *Admissible:* Electrodynamics ($K = d$), linearized GR, Newton gravity, the logarithmic flat-rotation kernel
- *Non-admissible:* Yang--Mills ($K = d + A and A$), full nonlinear GR ($K: e arrow.r d omega + omega and omega$)

The admissible theories do not raise the question of restoration at all --- their projections are automatically well-behaved. Among the non-admissible theories, the Projection Sufficiency Principle determines whether a Higgs-like mechanism is required: it is required for Yang--Mills (insufficient projection) but not for GR (sufficient projection).

*The evidentiary status of non-admissibility in GR.* A further complication deserves emphasis. The three gravitational observations cited above --- binary pulsar decay, frame-dragging, and gravitational waves --- are all predictions of *linearized* GR, which is admissible. The quadrupole formula is a linearized/post-Newtonian result. The Lense--Thirring effect is derived from the gravitomagnetic potential $h_(0 i)$ of the linearized metric. The gravitational wave equation is the linearized Einstein equation. The only claimed observation that requires the nonlinear $omega and omega$ self-coupling is the specific merger waveform shape in LIGO data, extracted via matched filtering against templates derived from the nonlinear theory. The evidence for the non-admissible part of GR --- the very structure this paper analyzes --- is therefore thinner than commonly presented. The cleanest gravitational measurements do not distinguish between linearized and full GR.

*Formulation independence.* The non-admissibility result is not an artifact of the Cartan variables. We analyze four formulations of GR (Cartan, Palatini, self-coupled spin-2, teleparallel) and find that every one has a non-admissible step in the $K$--$Q$ pipeline. The teleparallel formulation is the illuminating case: the production kernel ($T = d e$, torsion) is admissible, but the non-admissibility moves to the observable projection (the torsion scalar is quadratic in $T$). The non-admissibility cannot be eliminated; it can only be moved between $K$ and $Q$. This strengthens the paper's claims from the Cartan-specific ``$K$ is non-admissible'' to the formulation-independent ``the $K$--$Q$ pipeline is non-admissible.''

*What this paper does not do.* We do not derive the Einstein field equations from first principles, nor do we resolve the problem of time dynamically, nor do we provide a concrete quantum kernel $K$ that resolves the Schwarzschild singularity. These require physical content beyond the algebraic framework developed here. What we establish is the structural layer beneath: the non-admissibility of the GR $K$--$Q$ pipeline (shown to be formulation-independent), the role of $Q$ as the arbiter of whether non-admissibility requires dynamical correction, and the classification of singularities by their position relative to $"ker"(Q)$.

= Conclusion

This paper is not primarily about the non-admissibility of GR --- it is about how to read GR correctly: how to distinguish what is physical from what is formulation-dependent, and what is empirically established from what remains open. The $K$--$Q$ framework provides the coordinate system for this classification.

*The computation.* The GR production kernel is non-admissible: the admissibility defect $Delta eq.not 0$ on the Schwarzschild tetrad, with the $omega and omega$ self-coupling as the source. This parallels the $A and A$ term in Yang--Mills. The Weyl tensor is the explicit content of $"ker"(Q_("GR"))$, and the Schwarzschild singularity lives entirely within it --- a production artifact that the observable projection annihilates.

*The projection question.* The same algebraic non-admissibility produces different physical consequences depending on what $Q$ requires. In Yang--Mills, $Q$ needs charges, non-admissibility hides them, and the Higgs is required. In GR, $Q$ needs geometry, geometry survives non-admissibility, and no restoration is needed. The Projection Sufficiency Principle: non-admissibility requires dynamical restoration if and only if the essential observables lie in the sector it hides. This makes $Q$ the arbiter, not a passive filter.

*The formulation question.* Non-admissibility is a property of GR itself, not of Cartan variables. In every full formulation --- Cartan, Palatini, self-coupled spin-2, teleparallel --- the $K$--$Q$ pipeline contains a non-admissible step. But *where* it sits (in $K$ or $Q$), whether energy is localizable, and whether a background is needed are formulation fiber properties: they vary across physically equivalent formulations and are therefore not physical. The century-old problem of gravitational energy is a fiber question --- undecidable from the field equations --- in the same formal sense as the Continuum Hypothesis is undecidable from ZFC.

*The empirical question.* The three cleanest gravitational confirmations --- binary pulsar decay, frame-dragging, gravitational waves --- are predictions of linearized GR, which is admissible. The only evidence for the nonlinear $omega and omega$ self-coupling is the LIGO merger waveform, the most model-dependent of all gravitational measurements. The Deser construction shows that non-admissibility is binary: no consistent theory lies between the admissible linearized theory ($n = 0$) and the non-admissible full theory ($n = infinity$). The physical necessity of the nonlinear sector is not yet cleanly established.

*What the framework provides.* The $K$--$Q$ pipeline, combined with projection fiber theory, yields a three-way classification:

- *Physical content* (constant on formulation fibers): pipeline non-admissibility, field equations, observable predictions.
- *Formulation artifacts* (varying on fibers): energy localizability, $K$-admissibility, $Q$-admissibility, background-dependence.
- *Open questions* (empirically underdetermined): whether nature realizes the non-admissible sector, whether admissible kernels operate at all scales.

This classification --- projection, fiber, evidence --- applies beyond GR. Singularities, renormalization, gauge artifacts, and the Higgs mechanism all submit to the same analysis: the $K$--$Q$ framework does not resolve these questions, but it tells you which kind of question you are asking.

All results are formally verified: 17 by Kleis evaluation of the Cartan pipeline, 33 by Z3 structural verification, and 67 by Z3 formulation and truncation analysis.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[ADM1959\] Arnowitt, R., Deser, S., and Misner, C. W. Dynamical structure and definition of energy in general relativity. *Phys. Rev.* 116, 1322 (1959).]

#par(hanging-indent: 1.5em)[\[Ash1986\] Ashtekar, A. New variables for classical and quantum gravity. *Phys. Rev. Lett.* 57, 2244 (1986).]

#par(hanging-indent: 1.5em)[\[LIGO2016\] Abbott, B. P. et al. (LIGO/Virgo). Observation of gravitational waves from a binary black hole merger. *Phys. Rev. Lett.* 116, 061102 (2016).]

#par(hanging-indent: 1.5em)[\[Cur2019\] Curiel, E. On the existence of spacetime structure. *Brit. J. Phil. Sci.* 69, 447--483 (2019).]

#par(hanging-indent: 1.5em)[\[DEHK2024\] De Vuyst, J., Eccles, S., Hoehn, P. A., and Kirklin, J. Gravitational constraints on a lightlike boundary. arXiv:2409.09308 (2024).]

#par(hanging-indent: 1.5em)[\[Tra1979\] Trautman, A. Fiber bundles, gauge fields, and gravitation. In *General Relativity and Gravitation*, Vol. 1, A. Held, ed. (Plenum, 1979).]

#par(hanging-indent: 1.5em)[\[Sar1992\] Sardanashvily, G. On the geometry of spontaneous symmetry breaking and Higgs mechanism in gravitation. *J. Math. Phys.* 33, 1546 (1992).]

#par(hanging-indent: 1.5em)[\[Atik-IV\] Atik, E. Confinement as fiber non-invariance: a projection-theoretic derivation. Volume IV of the POT Series. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[Atik-V\] Atik, E. Admissibility restoration and the necessity of the Higgs mechanism. Volume V of the POT Series. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[Atik-III\] Atik, E. Independence as non-invariance: detecting undecidability via projection fibers in SMT-backed shadow theories. Volume III of the POT Series. *Preprint* (2025). DOI: 10.13140/RG.2.2.22374.18243.]

#par(hanging-indent: 1.5em)[\[Atik-X\] Atik, E. Projection singularities: why physics has no infinities. Volume X of the POT Series. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[Atik-VII\] Atik, E. The abstract K-Q framework: kernels, projections, and null-space structure across domains. Volume VII of the POT Series. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[Atik-RC\] Atik, E. Flat galactic rotation curves from a logarithmic Green's function. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[Atik-I\] Atik, E. Projected Ontology Theory: a kernel-projection framework for quantum field theory. Volume I of the POT Series. *Preprint* (2025).]

#par(hanging-indent: 1.5em)[\[Mil1983\] Milgrom, M. A modification of the Newtonian dynamics as a possible alternative to the hidden mass hypothesis. *Astrophys. J.* 270, 365 (1983).]

#par(hanging-indent: 1.5em)[\[GPB2011\] Everitt, C. W. F. et al. Gravity Probe B: Final results of a space experiment to test general relativity. *Phys. Rev. Lett.* 106, 221101 (2011).]

#par(hanging-indent: 1.5em)[\[HT1975\] Hulse, R. A. and Taylor, J. H. Discovery of a pulsar in a binary system. *Astrophys. J.* 195, L51 (1975).]

#par(hanging-indent: 1.5em)[\[Isa1968\] Isaacson, R. A. Gravitational radiation in the limit of high frequency. II. Nonlinear terms and the effective stress tensor. *Phys. Rev.* 166, 1272 (1968).]

#par(hanging-indent: 1.5em)[\[Des1970\] Deser, S. Self-interaction and gauge invariance. *Gen. Relativ. Gravit.* 1, 9 (1970).]

#par(hanging-indent: 1.5em)[\[Mal2013\] Maluf, J. W. The teleparallel equivalent of general relativity. *Annalen Phys.* 525, 339 (2013).]


