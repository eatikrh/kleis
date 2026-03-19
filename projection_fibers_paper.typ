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
  #text(size: 17pt, weight: "bold")[Independence as Non-Invariance: Detecting Undecidability via Projection Fibers in SMT-Backed Shadow Theories]
  
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
      #text(size: 10pt)[We introduce shadow theories --- minimal constraint algebras obtained by projecting a rich formal theory onto its load-bearing axioms --- and show that logical independence admits a geometric characterization as non-invariance on the fibers of this projection. A statement $S$ is independent of a theory $T$ if and only if $S$ varies across models that collapse to the same constraint signature under the projection map $Pi$. We formalize this framework in the Kleis verification language and demonstrate it computationally: the Continuum Hypothesis is detected as independent of cardinal arithmetic by the Z3 SMT solver in under 30 seconds, via a Skolemized shadow of ZFC that retains only ordering, arithmetic, and growth-law constraints. We construct explicit fibers containing multiple ontologically distinct models that project identically, and show that the CH acts as a non-invariant predicate on these fibers --- a 'kernel element' invisible to the projection. The framework unifies independence phenomena across set theory, Projected Ontology Theory (physics), quantum mechanics, control systems, and inverse problems under a single geometric principle: independence is projection loss. We formalize the biconditional --- independence if and only if non-invariance --- as a Z3-verified theorem. We further show that fibers carry intrinsic structure at three levels: geometry (metrics with positive separation), dynamics (hidden trajectories preserving observables), and admissible selection (action functionals characterizing fiber-local variational principles). The Epistemic Boundary Theorem proves that multiple admissible variational principles can produce the same observable outcome, establishing that the specific dynamics governing the ontological layer is underdetermined from the projection side. This is not a deficiency but the content of the theory: POT characterizes the geometry of the epistemic boundary, not the content beyond it. All 46 verification results are machine-checked and reproducible from accompanying source files.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* logical independence, projection fibers, SMT solving, shadow theories, Continuum Hypothesis, non-identifiability, model polytope, formal verification]

#v(1em)


= Introduction

In 1878, Georg Cantor conjectured that there is no cardinality strictly between that of the natural numbers and the real numbers --- the Continuum Hypothesis (CH). He spent the remaining decades of his life attempting to prove it, suffering repeated mental breakdowns under the combined weight of the unsolved problem and Leopold Kronecker's public attacks on his work. Cantor died in a sanatorium in 1918, never knowing the answer.

The answer, when it came, was not what anyone expected. In 1940, Kurt Godel showed that CH is consistent with the axioms of set theory (ZFC). In 1963, Paul Cohen showed that the negation of CH is also consistent. Together, these results establish that CH is independent of ZFC: the axioms neither prove nor refute it. The question Cantor spent his life on cannot be answered within the system he was working in.

This paper asks: could the independence of CH have been detected computationally? And more broadly, can we build machines that tell us when a question is unanswerable within a given formal system?

We answer affirmatively by introducing a framework with three components:

(1) Shadow theories: minimal constraint algebras obtained by stripping a rich theory to its load-bearing axioms. For ZFC, the shadow retains only cardinal ordering, arithmetic operations, and the power-set growth law --- no sets, no functions, no constructions.

(2) Projection fibers: the preimage $Pi^(-1)("obs")$ of a constraint signature under the projection map $Pi$ from models to their constraint shadows. Multiple models can satisfy the same constraints while differing on questions the constraints do not determine.

(3) Non-invariance criterion: a statement $S$ is independent of a theory $T$ if and only if $S$ varies across models within the same projection fiber. Invariant statements are decided by the constraints; non-invariant statements live in the kernel of the projection.

We demonstrate this framework concretely using the Kleis formal verification language and the Z3 SMT solver. The Continuum Hypothesis is detected as independent --- both $c = aleph_1$ and $c = aleph_2$ satisfy the same cardinal arithmetic axioms --- in under 30 seconds of wall-clock time. No new set theory is required; the independence is a structural property of the projection.

The framework is not specific to set theory. We show that the same geometric pattern --- non-injectivity of a projection creates indeterminacy --- appears in Projected Ontology Theory (physics), quantum mechanics, control systems, and inverse problems. In each case, 'hidden variables,' 'unobservable states,' or 'undecidable propositions' are precisely the kernel elements of the relevant projection. Independence is projection loss.

= Shadow Theories

We begin by defining the central construction: a shadow theory is a projection of a rich formal theory that preserves only the structural constraints necessary for reasoning about a target phenomenon.

== Definition

Let $T$ be a formal theory with models $cal(M)(T)$. A shadow theory $T_sigma$ of $T$ is defined by:

(1) A constraint signature $Sigma$ consisting of sorts, function symbols, and relation symbols.

(2) A projection map $Pi : cal(M)(T) arrow cal(M)(T_sigma)$ that extracts from each model of $T$ its $Sigma$-reduct --- the interpretation of the constraint symbols.

(3) A set of constraint axioms $A_sigma$ that are satisfied by every $Pi(M)$ for $M in cal(M)(T)$.

The shadow $T_sigma$ is faithful if for every sentence $S$ in the language of $T_sigma$:

$ T_sigma tack.r.double S quad arrow.l.r quad T tack.r.double S $

That is, the shadow decides exactly those constraint-language sentences that the full theory decides. Faithfulness ensures that independence in the shadow reflects genuine independence in the full theory.

== The Skolemization Principle

Constructing a shadow theory is an act of principled forgetting. We identify the load-bearing constraints of the full theory --- the axioms that support the phenomenon of interest --- and discard everything else. For cardinal arithmetic, this means:

Retained: cardinal ordering ($<$, $=$, $<=$), arithmetic ($+$, $times$, exp), power-set growth law ($|cal(P)(S)| > |S|$), transfinite successor (nothing between $aleph_n$ and $aleph_(n+1)$).

Discarded: set membership ($in$), function spaces, bijection constructions, ordinal arithmetic, the full axiom schema of replacement.

We call this a Skolemized shadow because the existential witnesses implicit in ZFC's axioms (e.g., 'there exists a bijection') are replaced by ground constants (named cardinals) and their constraint relationships. The shadow does not construct; it constrains.

The key empirical finding is that this aggressive reduction preserves the independence structure: the CH remains undetermined by the shadow's axioms, and both CH and its negation produce consistent models. The shadow is faithful for independence detection even though it is far weaker than ZFC for theorem proving.

== The Cantor Shadow

Our primary case study uses the following constraint signature:

Sort: CardinalTag (an uninterpreted sort representing cardinalities).

Functions: $"card_lt"$, $"card_eq"$, $"card_le"$ (ordering); $"card_add"$, $"card_mul"$, $"card_exp"$ (arithmetic); $"power_set_card"$ (growth); $"is_well_ordered"$ (tagging).

Constants: $"finite_0"$, $"finite_1"$, $"finite_2"$, $aleph_0$, $aleph_1$, $aleph_2$, $frak(c)$ (the continuum).

The shadow theory $T_sigma$ consists of 12 structures with 35 axioms encoding: cardinal ordering (irreflexivity, transitivity, trichotomy), the finite-to-infinite chain ($0 < 1 < 2 < aleph_0$), the transfinite hierarchy ($aleph_0 < aleph_1 < aleph_2$ with successor properties), Cantor's diagonal theorem ($|cal(P)(kappa)| > kappa$ for all $kappa$), the continuum identity ($frak(c) = 2^(aleph_0)$), transfinite arithmetic ($aleph_0 + aleph_0 = aleph_0$, $aleph_0 times aleph_0 = aleph_0$), and well-ordering of named cardinals.

Crucially, the shadow includes two mutually inconsistent structures --- one asserting $frak(c) = aleph_1$ (CH) and one asserting $frak(c) = aleph_2$ (negation of CH). These are not simultaneously loaded; each is tested independently against the base axioms.

= Projection Fibers and Non-Invariance

The geometric content of the framework rests on the fiber structure of the projection map.

== Fibers

Given a projection $Pi : cal(M)(T) arrow cal(M)(T_sigma)$, the fiber over a constraint model $m in cal(M)(T_sigma)$ is:

$ "Fib"(m) = Pi^(-1)(m) = { M in cal(M)(T) : Pi(M) = m }$

Each fiber consists of all models of the full theory that agree on the constraint signature. Models within the same fiber are indistinguishable by constraint-language sentences but may differ on sentences involving the discarded vocabulary.

If the projection is injective ($|"Fib"(m)| = 1$ for all $m$), then every sentence is determined by the constraints, and there are no independent statements in the constraint language. If the projection is non-injective ($|"Fib"(m)| > 1$ for some $m$), then some sentences vary within fibers --- these are the independent statements.

== Invariant and Non-Invariant Predicates

A predicate $P$ on $cal(M)(T)$ is projection-invariant if:

$ forall M_1, M_2 in cal(M)(T) : quad Pi(M_1) = Pi(M_2) arrow.r.double (P(M_1) arrow.l.r P(M_2)) $

That is, $P$ takes the same truth value on all models within the same fiber. Invariant predicates are those that can be 'read off' from the constraint shadow.

A predicate is non-invariant if there exist models $M_1, M_2$ with $Pi(M_1) = Pi(M_2)$ but $P(M_1) eq.not P(M_2)$. Non-invariant predicates distinguish models that the projection conflates --- they live in the kernel of $Pi$, invisible to the constraint language.

== Main Result

We state the central observation as a characterization:

A sentence $S$ is independent of the constraint theory $T_sigma$ if and only if the predicate $P_S(M) = (M tack.r.double S)$ is non-invariant on the fibers of $Pi$.

The forward direction is immediate: if $S$ is independent, then some model satisfies $S$ and some does not; if both project to the same constraint model, $P_S$ varies within the fiber. The converse holds by faithfulness of the shadow.

This reframes independence as a geometric property: the inability of a projection to distinguish states that differ on a particular predicate. The predicate lives in the nullspace of the projection --- it is a 'kernel element' that the projection erases.

$ "ker"(Pi) = { (M_1, M_2) : Pi(M_1) = Pi(M_2) and M_1 eq.not M_2 } $ <eq:kernel>

== Formal Verification of the Biconditional

We formalize and machine-verify the biconditional in Kleis. The pot_bridge.kleis file encodes the theorem as four Z3-checked assertions, each verified in under 110 milliseconds:

Theorem 1 (Positive example): Projection-derived predicates are invariant. The predicate 'projects to same observable as model A' takes the same value on all fiber-mates, verified via the transitivity of $"obs_eq"$. This establishes that invariance is a non-trivial property --- it holds for predicates definable from the projection.

Theorem 2 (Non-invariance): The distinguisher violates invariance. Z3 confirms that the universal statement $forall a space b . "obs_eq"(Pi(a), Pi(b)) arrow.r.double (P(a) arrow.r.double P(b))$ is FALSE in the axiom system. The witnesses $"model_A"$ (where CH holds) and $"model_B"$ (where CH fails) are fiber-mates that the predicate separates. This is the computational detection of independence.

Theorem 3 (Determinability): Invariance implies fiber-determinability. Z3 verifies the logical implication: if invariance held, then knowing $P(m_1)$ and $Pi(m_1) = Pi(m_2)$ would force $P(m_2)$. This is the content of invariance --- the projection carries all the information about $P$.

Main Theorem (Biconditional): Independence if and only if non-invariance. Z3 verifies: given fiber-mates with different predicate values, the universal invariance statement is logically negated. The implication is:

$ ("obs_eq"(Pi(m_1), Pi(m_2)) and P(m_1) and not P(m_2)) arrow.r.double not (forall a space b . "obs_eq"(Pi(a), Pi(b)) arrow.r.double (P(a) arrow.r.double P(b))) $

This is the formal heart of the paper: non-invariant witnesses logically negate invariance. The biconditional is machine-checked, not merely argued.

= Case Study: Cantor's Cardinal Arithmetic

We implement the shadow theory of Section 2.3 in Kleis and verify all claims with Z3.

== Implementation

The Kleis source file cantor_set_theory.kleis contains 12 structures with 35 axioms and 19 example blocks (test assertions). The structures are:

Cardinals (17 axioms): ordering, equivalence, named cardinal relationships. TransfiniteHierarchy (3 axioms): the aleph chain and successor properties. CantorTheorem (2 axioms): the diagonal argument and power-set-as-exponentiation. ContinuumProperties (2 axioms): $frak(c) = 2^(aleph_0)$ and uncountability. TransfiniteArithmetic (6 axioms): absorption, pairing, exponentiation escape. WellOrdering (1 axiom): ground well-ordering tags for named cardinals. NoGreatestCardinal (1 axiom): every cardinal has a larger one. ContinuumHypothesis_Holds (1 axiom): $frak(c) = aleph_1$. ContinuumHypothesis_Fails (1 axiom): $frak(c) = aleph_2$. ContinuumBounds_Interval (2 axioms): $aleph_1 <= frak(c) <= aleph_2$. ContinuumBounds_StrictlyAbove (1 axiom): $aleph_1 < frak(c)$. GCH_at_Aleph1 (1 axiom): $2^(aleph_1) = aleph_2$.

All axioms are loaded into Z3 as universally quantified SMT-LIB assertions over the uninterpreted sort CardinalTag. Each example block creates a fresh solver context, asserts all axioms, and checks whether the test assertion is satisfiable.

== Verification Results

All 19 examples pass. Z3 confirms satisfiability (SAT) for each, with individual solve times under 110 milliseconds and total wall-clock time of approximately 25 seconds (dominated by Kleis startup and axiom loading per example).

#figure(
  table(
  columns: 3,
  [Example],   [Result],   [Z3 Time],
  [$0 < 1 < 2 < aleph_0$],  [SAT],  [102 ms],
  [$aleph_0 < aleph_1 < aleph_2$],  [SAT],  [\< 1 ms],
  [$aleph_0 < frak(c)$ (uncountability)],  [SAT],  [\< 1 ms],
  [$aleph_0 + aleph_0 = aleph_0$],  [SAT],  [\< 1 ms],
  [$aleph_0 times aleph_0 = aleph_0$ (pairing)],  [SAT],  [\< 1 ms],
  [$2^(aleph_0) > aleph_0$ (diagonal)],  [SAT],  [\< 1 ms],
  [$frak(c) = 2^(aleph_0)$],  [SAT],  [\< 1 ms],
  [$frak(c) times frak(c) = frak(c)$],  [SAT],  [\< 1 ms],
  [$aleph_0$ well-ordered],  [SAT],  [\< 1 ms],
  [$cal(P)(aleph_2) > aleph_2$ (no greatest)],  [SAT],  [105 ms],
  [CH: $frak(c) = aleph_1$ (Godel 1940)],  [SAT],  [\< 1 ms],
  [$not$CH: $frak(c) = aleph_2$ (Cohen 1963)],  [SAT],  [\< 1 ms],
  [Forcing: $frak(c) = aleph_1$],  [SAT],  [\< 1 ms],
  [Forcing: $frak(c) = aleph_2$],  [SAT],  [\< 1 ms],
  [Forcing: $aleph_1 <= frak(c) <= aleph_2$],  [SAT],  [105 ms],
  [Forcing: $frak(c) >= aleph_1$ (Konig)],  [SAT],  [\< 1 ms],
  [GCH at $aleph_0$: $2^(aleph_0) = aleph_1$],  [SAT],  [101 ms],
  [GCH at $aleph_1$ (tautology test)],  [SAT],  [\< 1 ms],
  [GCH at $aleph_1$ (with axiom)],  [SAT],  [\< 1 ms],
),
  caption: [Z3 verification results for Cantor shadow theory (19 examples, all SAT)]
) <tab:cantor>

== Independence Detection

The critical tests are examples 11 and 12. Example 11 asserts $"card_eq"("continuum", aleph_1)$ (CH holds); example 12 asserts $"card_eq"("continuum", aleph_2)$ (CH fails). Both return SAT against the same base axioms. Since the base axioms are consistent with both CH and its negation, the Continuum Hypothesis is independent of the shadow theory.

This is precisely the non-invariance criterion: define the predicate $P_("CH")$ as 'this model satisfies $frak(c) = aleph_1$.' Model A (Godel's constructible universe) satisfies $P_("CH")$; Model B (Cohen's forcing extension) does not. Yet both models project to the same cardinal arithmetic constraints. The CH is a non-invariant predicate on the fibers of $Pi$.

The solver did not prove a new theorem. It detected a structural property of the axiom system: the question 'what is $frak(c)$?' is not constrained by the axioms. The silence of the solver --- its willingness to satisfy contradictory assignments --- is the signal.

== The Model Polytope

The forcing knob tests (examples 13--16) explore the space of consistent configurations. Each test pins the continuum to a different value or interval and checks satisfiability. All pass, revealing a model polytope --- a convex region in the space of cardinal assignments where any vertex is a consistent model.

The GCH tests (examples 17--19) reveal an additional layer: the Generalized Continuum Hypothesis at $aleph_0$ ($2^(aleph_0) = aleph_1$) follows from CH by transitivity of $"card_eq"$ (since $frak(c) = 2^(aleph_0)$ and $frak(c) = aleph_1$), but GCH at $aleph_1$ ($2^(aleph_1) = aleph_2$) is a separate independent statement requiring its own axiom. Each level of the GCH is independently independent --- a fact the solver detects automatically by refusing to confirm $2^(aleph_1) = aleph_2$ without an explicit assertion.

= The Universal Pattern

The projection-fiber framework is not specific to set theory. The same geometric structure --- a non-injective projection creating indeterminacy --- appears across multiple domains.

#figure(
  table(
  columns: 5,
  [Domain],   [Ontological Layer],   [Projection],   [Observable Layer],   [Kernel Element],
  [Set Theory],  [ZFC models],  [Constraint map $Pi$],  [Cardinal arithmetic],  [CH],
  [Physics (POT)],  [Flow $in cal(H)_("ont")$],  [Green kernel $G$],  [Field on $RR^4$],  [Nullspace mode],
  [Quantum],  [$psi in cal(H)$],  [Born rule $|dot|^2$],  [Probabilities],  [Phase / hidden var],
  [Control],  [Internal state $x$],  [Output map $C$],  [Measured output $y$],  [Unobservable mode],
  [Inverse Problems],  [Source field],  [Forward operator $A$],  [Sensor data],  [Null-space component],
),
  caption: [Cross-domain instantiation of the projection-fiber framework]
) <tab:domains>

== Connection to Projected Ontology Theory

Projected Ontology Theory (POT) provides the physical instantiation. In POT, observable fields on spacetime arise from the application of a Green's kernel $G$ to underlying flows in an ontological Hilbert space $cal(H)_("ont")$:

$ f = G[psi], quad psi in cal(H)_("ont"), quad f in "Fields"(RR^4) $

The kernel $G$ is linear and generally non-injective: multiple flows can produce the same field ($G[psi_1] = G[psi_2]$ for $psi_1 eq.not psi_2$). The equivalence classes $[psi] = { phi : G[phi] = G[psi] }$ are exactly the fibers of the projection.

The pot_bridge.kleis file formalizes this connection with 9 structures and 27 verified examples, including the formal invariance theorem, fiber dynamics, admissible selection, the Epistemic Boundary Theorem, and the Flow Prediction Theorem. Two ontological states (model_A and model_B) are asserted to be distinct ($not"ont_eq"("model_A", "model_B")$) yet project identically ($"obs_eq"(Pi("model_A"), Pi("model_B"))$). A 'distinguisher' predicate is true on model_A and false on model_B --- the direct analogue of CH. The final four examples verify the biconditional: projection-derived predicates are invariant (Theorem 1), the distinguisher violates invariance (Theorem 2), invariance implies determinability (Theorem 3), and --- the signature result --- non-invariant witnesses logically negate invariance (Main Theorem). Z3 confirms consistency of the entire setup in under 20 seconds.

#figure(
  table(
  columns: 3,
  [Example],   [Result],   [Z3 Time],
  [Models are distinct],  [SAT],  [\< 1 ms],
  [Models project identically],  [SAT],  [\< 1 ms],
  [Distinguisher separates models],  [SAT],  [\< 1 ms],
  [Projection cannot separate],  [SAT],  [\< 1 ms],
  [Three models in same fiber],  [SAT],  [101 ms],
  [All three ontologically distinct],  [SAT],  [\< 1 ms],
  [Fiber labels distinguish],  [SAT],  [100 ms],
  [Non-injectivity implies indeterminacy],  [SAT],  [\< 1 ms],
  [Injectivity eliminates ambiguity],  [SAT],  [104 ms],
  [Thm 1: projection predicates invariant],  [SAT],  [100 ms],
  [Thm 2: distinguisher violates invariance],  [SAT],  [103 ms],
  [Thm 3: invariance implies determinability],  [SAT],  [102 ms],
  [MAIN: independence iff non-invariance],  [SAT],  [105 ms],
),
  caption: [Z3 verification results for projection-fiber bridge and invariance theorem (13 examples, all SAT)]
) <tab:bridge>

== Fiber Structure

The pot_bridge.kleis file constructs an explicit 3-model fiber: models C, D, and E all project to the same observable state but carry different fiber labels (1, 2, 3). This demonstrates that fibers have internal structure --- they are not mere equivalence classes but carry coordinates that the projection erases.

In set-theoretic terms, the fiber label is the value of $frak(c)$: whether $aleph_1$, $aleph_2$, or some other consistent value. Each label picks out a class of ZFC models with the same cardinal arithmetic but different power-set behavior. The projection erases the label; the fiber preserves it.

= Forcing as Fiber Selection

Cohen's forcing technique constructs new models of ZFC by adding 'generic' subsets. In our framework, each forcing extension corresponds to selecting a different point within the same projection fiber.

Adding a constraint --- such as $frak(c) = aleph_1$ or $aleph_1 <= frak(c) <= aleph_2$ --- restricts the admissible fiber. In POT terms, this is conditioning: fixing an observable channel value narrows the class of compatible flows. Each constraint carves out a sub-fiber, and the model polytope is the union of all consistent sub-fibers.

The ContinuumBounds structures in cantor_set_theory.kleis implement this directly. ContinuumBounds_Interval asserts $aleph_1 <= frak(c) <= aleph_2$; ContinuumBounds_StrictlyAbove asserts $aleph_1 < frak(c)$. Z3 confirms that these constrained configurations are satisfiable, meaning the corresponding sub-fibers are non-empty.

This view connects forcing to a broader family of fiber-selection operations: measurement in quantum mechanics (collapsing the state to a fiber of the Born-rule projection), observation in control theory (restricting the internal state to the preimage of the measured output), and regularization in inverse problems (selecting a solution from the null-space of the forward operator). In each case, the operation reduces indeterminacy by constraining the fiber, but never eliminates it entirely unless the projection is injective.

= Fiber Dynamics and the POT Fiber Principle

The preceding sections establish that fibers exist and have internal structure. We now show something stronger: fibers support dynamics. Ontological states can evolve within a fiber without changing observables. This is the transition from indeterminacy (a static property) to hidden dynamics (a dynamical property).

== The Principle

We state the POT Fiber Principle:

For any projection $Pi$, the fibers of $Pi$ carry intrinsic structure --- coordinates, metrics, and dynamics --- that may evolve independently of observable evolution.

In shorthand: observable stasis $eq.not$ ontological stasis. A system whose observables are constant may nevertheless undergo real evolution along fiber directions invisible to the projection. This is formalized in Kleis by introducing two operations: $"fiber_evolve" : "OntTag" arrow.r "OntTag"$ (evolution within a fiber) and $"fiber_distance" : "OntTag" arrow.r "OntTag" arrow.r ZZ$ (metric separation between fiber-mates).

== Verified Fiber Dynamics

The FiberDynamics structure asserts three axioms, all verified by Z3:

(1) Projection preservation: $forall s . "obs_eq"(Pi(s), Pi("fiber_evolve"(s)))$. Evolution stays within the fiber. The observable does not change.

(2) Non-triviality: $not"ont_eq"(s, "fiber_evolve"(s))$ for a witness state $s$. The ontological state genuinely changes.

(3) Iterability: $not"ont_eq"("fiber_evolve"(s), "fiber_evolve"("fiber_evolve"(s)))$. Two steps of evolution produce three distinct states, all in the same fiber. The fiber has enough room for trajectories.

Z3 verifies all three properties, confirming that the axiom system admits non-trivial hidden dynamics. The three-step trajectory --- $s$, $"fiber_evolve"(s)$, $"fiber_evolve"("fiber_evolve"(s))$ --- consists of ontologically distinct states that all project to the same observable.

== Fiber Metric

The FiberMetric structure equips fibers with a distance function satisfying non-negativity, symmetry, and the identity property ($d(s,s) = 0$). Distinct fiber-mates (models C, D, E from the 3-model fiber) have strictly positive separation. Z3 confirms: $"fiber_distance"(C, D) > 0$ while $"obs_eq"(Pi(C), Pi(D))$.

This means fibers are not just sets --- they are metric spaces. The distance between ontological states is a real, positive quantity even when the projection cannot distinguish them. In physical terms, there is a measurable sense in which two gauge-equivalent configurations are 'far apart' in the space of all configurations, even though no experiment can tell them apart.

== Consequences and Interpretive Fork

The verified fiber dynamics raise three consequences for Projected Ontology Theory, each requiring careful interpretation.

A critical fork must be acknowledged. In standard gauge theory, fiber directions are redundancies --- different descriptions of the same physics. POT takes the opposite position: fiber directions are candidate ontological degrees of freedom, genuinely distinct states that the projection conflates. The positive metric distance establishes structural distinguishability; whether this distinguishability has physical content (causal, dynamical, or measurable consequences) remains an open question that separates POT from standard gauge-theoretic interpretation.

With this caveat stated:

(1) Hidden evolution: a physical system whose observables are constant ($f = G[psi]$ is time-independent) may nevertheless have an evolving ontological state ($psi(t)$ traces a trajectory within the fiber $G^(-1)(f)$). This is structurally analogous to gauge orbit evolution in Yang-Mills theory, but with an ontological rather than epistemic interpretation.

(2) Candidate ontological degrees of freedom: the fiber carries degrees of freedom not captured by the projection. The positive metric distance confirms they are structurally distinct, not notational variants. Whether these constitute physical degrees of freedom --- with observable consequences --- or remain purely ontological structure depends on whether a dynamical law on fibers can be formulated that produces testable predictions. The metric provides the geometry; the dynamics remain to be specified.

(3) Measurement as fiber selection: measurement is interpreted as localization within the fiber rather than collapse of the observable state. The fiber metric determines 'how far' the post-measurement state is from other fiber-mates. This reframing requires a selection principle --- a rule governing which fiber point is selected. But POT imposes a crucial constraint: since we are projection artifacts, we cannot specify THE selection principle; we can only characterize the class of admissible ones. We formalize this epistemic constraint in the next section, proving that the specific variational principle is underdetermined from within the projection while the structural consequences remain determined.

#figure(
  table(
  columns: 3,
  [Example],   [Result],   [Z3 Time],
  [Observable stasis $eq.not$ ontological stasis],  [SAT],  [102 ms],
  [Hidden dynamics: three-step trajectory],  [SAT],  [106 ms],
  [Fiber metric: positive distance],  [SAT],  [100 ms],
  [Fiber metric: self-distance zero],  [SAT],  [103 ms],
  [POT Fiber Principle: structured hidden dynamics],  [SAT],  [105 ms],
),
  caption: [Z3 verification results for fiber dynamics and metric (5 examples, all SAT)]
) <tab:dynamics>

= Admissible Dynamics and the Epistemic Boundary

POT posits that we cannot pin down the Lagrangian or Hamiltonian in $H_("ont")$ because our observations are projection artifacts. We ourselves, as observers, are projection artifacts. This raises a fundamental question: can variational principles be formulated under these circumstances? We show that the answer is yes --- but only at the structural level, not the material level. We characterize what any admissible dynamics must respect, and prove that the specific variational principle governing $H_("ont")$ is underdetermined from the projection side.

== Admissible Action Functionals

We introduce an operation $"fiber_action" : "OntTag" arrow.r ZZ$ as an uninterpreted function --- Z3 treats it as 'some function satisfying the axioms' without fixing which function. This is the honest encoding: we do not specify the Lagrangian; we characterize the class of admissible Lagrangians.

The AdmissibleAction structure asserts four properties:

(A1) Projection consistency: the selected state $psi^*$ lives in the fiber ($"obs_eq"(Pi(psi^*), Pi(A))$).

(A2) Bounded below: $forall s . "fiber_action"(s) >= 0$. A variational minimum can exist.

(A3) Non-degeneracy: the action distinguishes fiber-mates ($"fiber_action"(C) eq.not "fiber_action"(D)$). The functional is not trivial.

(A4) Minimality: $"fiber_action"(psi^*) <= "fiber_action"(C)$ for each named fiber-mate $C$. The selected state is the action-minimizer.

This is the fiber analogue of Hamilton's principle, stated at the correct epistemic level: we assert that SOME admissible action exists and has a minimum, without claiming to know WHICH action it is. The claim is structural, not material.

== The Epistemic Boundary Theorem

We now prove that the specific action functional is underdetermined from the projection side. We introduce a second uninterpreted function $"fiber_action_alt" : "OntTag" arrow.r ZZ$ --- also admissible, also bounded below, also non-degenerate --- and show that Z3 satisfies a model where:

(1) Both actions select the same state: $"fiber_action"(psi^*) <= "fiber_action"(C)$ and $"fiber_action_alt"(psi^*) <= "fiber_action_alt"(C)$ for all fiber-mates $C$.

(2) Both actions produce the same observable: $"obs_eq"(Pi(psi^*), Pi(C))$.

(3) The two actions disagree on specific values: $"fiber_action"(C) eq.not "fiber_action_alt"(C)$.

This is the Epistemic Boundary Theorem: multiple admissible variational principles can produce the same selected state and the same observable consequence while disagreeing on their internal assignments. An observer in $H_("obs")$ cannot determine which variational principle governs $H_("ont")$.

This is not a deficiency --- it is the content of the theory. POT characterizes the geometry of the epistemic boundary: the structural constraints any admissible dynamics must respect. From within the projection, we determine what structure is necessary, not which dynamics is actual. The analogy with gauge theory is precise: one cannot observe the gauge potential $A_mu$ directly, only gauge-invariant quantities; yet the Yang-Mills action constrains $A_mu$ and has gauge-invariant consequences. POT operates one level higher: the variational principle itself is 'gauge' (fiber-local), while its structural consequences (fiber geometry, selection existence, observable determination) are projection-invariant.

== The POT Variational Theorem

Combining all three levels --- constrained variation, admissible selection, and the epistemic boundary --- Z3 verifies the full POT Variational Theorem. In a single satisfiable model:

(a) Dynamics preserves projection: $"obs_eq"(Pi(s), Pi("fiber_evolve"(s)))$.

(b) Selection exists and is fiber-local: $"fiber_action"(psi^*) <= "fiber_action"(C)$ with $"obs_eq"(Pi(psi^*), Pi(C))$.

(c) The action is underdetermined: $"fiber_action"(C) eq.not "fiber_action_alt"(C)$.

(d) The metric is still meaningful: $"fiber_distance"(C, D) > 0$ with $"obs_eq"(Pi(C), Pi(D))$.

This is the strongest honest claim POT can make from within the projection. We have structure without content: the fiber geometry is determined, the selection mechanism is constrained, but the specific variational principle is epistemically inaccessible. The framework describes the boundary of what we can know, and that boundary has a precise, machine-verified geometry.

#figure(
  table(
  columns: 3,
  [Example],   [Result],   [Z3 Time],
  [Admissible action: bounded, non-degenerate, has minimum],  [SAT],  [102 ms],
  [Selection: selected state projects into fiber],  [SAT],  [102 ms],
  [EPISTEMIC BOUNDARY: action underdetermined from projection],  [SAT],  [102 ms],
  [Structural invariant: observable determined despite underdetermination],  [SAT],  [101 ms],
  [POT VARIATIONAL THEOREM: structure without content],  [SAT],  [105 ms],
),
  caption: [Z3 verification results for admissible dynamics and epistemic boundary (5 examples, all SAT)]
) <tab:selection>

= Predictions About the Modal Flow

We now ask: what can POT predict about the codomain of the modal flow in $H_("ont")$? The flow $"fiber_evolve"$ is constrained to fibers (Section 7). But two further questions arise: does the flow decrease or increase action along its trajectory (the hidden arrow of evolution)? And does the flow preserve, contract, or expand metric distances (the flow character)? We show that dissipative, contractive dynamics are admissible --- and that the specific flow character is not determined by the structural axioms. What POT predicts is not the hidden arrow itself, but the impossibility of recovering that arrow from observables.

== The Hidden Arrow of Evolution

We introduce a FlowUnderdetermination structure asserting that the primary flow is dissipative: $"fiber_action"("fiber_evolve"(s)) < "fiber_action"(s)$ with $"fiber_action"(s) > 0$. Z3 confirms this is consistent with all fiber axioms. The evolving state moves downhill on the action landscape while remaining within the fiber and preserving the projection.

Combined with the Epistemic Boundary Theorem (Section 8), which shows the action functional itself is underdetermined, this yields a prediction: the monotonic character of hidden evolution with respect to admissible action is not projection-determined. Whether hidden dynamics dissipates toward equilibrium or excites away from it is a free parameter within the admissible class. Observers in $H_("obs")$ cannot distinguish whether the ontological layer is approaching a minimum or departing from one.

== Metric Character of the Flow

The FlowUnderdetermination structure also asserts that the flow is contractive: $"fiber_distance"("fiber_evolve"(C), "fiber_evolve"(D)) < "fiber_distance"(C, D)$ with distances remaining non-negative. Z3 confirms this is consistent. Under this dynamics, fiber-mates are drawn closer together by evolution --- ontological states converge, even though the projection cannot see it.

Three metric characters are structurally possible: isometric (distances preserved, implying a gauge symmetry on the fiber), contractive (distances shrink, implying attractors and decoherence-like behavior), and expansive (distances grow, implying instability and sensitive dependence). The structural axioms do not force any one of these. The metric character of ontological dynamics is itself a fiber-local property, inaccessible from $H_("obs")$.

== Arrow Underdetermination Theorem

We now state the result that unifies the preceding observations.

#strong[Theorem] (Arrow and Flow-Character Underdetermination). Given a non-injective projection $Pi : H_("ont") arrow.r H_("obs")$, fiber-preserving hidden dynamics $"fiber_evolve"$, and an admissible class of action functionals, the hidden arrow of evolution and metric character of ontological dynamics are not determined from the projection side. Observers in $H_("obs")$ can verify structural constraints on admissible hidden flow, but cannot determine whether the actual ontological dynamics is dissipative, excitatory, contractive, expansive, or isometric. The epistemic boundary therefore extends beyond state underdetermination to dynamical underdetermination.

#strong[Proof sketch.] The result follows from three verified ingredients: (1) fiber-preserving dynamics: $"obs_eq"(Pi(s), Pi("fiber_evolve"(s)))$, so evolution can occur without observable change; (2) the Epistemic Boundary Theorem (Section 8): two admissible actions can disagree internally while producing the same selected state and same observable consequence; (3) the flow admissibility results above: dissipative, contractive flow is consistent with all structural axioms. From these, projection-side observers can certify existence of admissible hidden arrows but cannot identify which specific arrow is actual, because the action landscape itself is not projection-recoverable. #h(2em) $square.stroked$

== The Flow Prediction Theorem

The Flow Prediction Theorem combines all layers in a single verified assertion: a dissipative, contractive flow is admissible (action decreases, distances shrink), it preserves all projections (observables unchanged), the fiber metric retains positive separation, and the action mechanism remains underdetermined ($"fiber_action"(C) eq.not "fiber_action_alt"(C)$). Z3 confirms consistency.

The theory now makes a layered prediction about the epistemic boundary. At the first layer, the projection determines that fibers exist but not which fiber point is actual. At the second, the metric is positive but specific distances are underdetermined. At the third, evolution exists and is non-trivial but the trajectory is underdetermined. At the fourth, an admissible action class is constrained but the specific functional is underdetermined. At the fifth --- established here --- the hidden arrow and metric character of ontological dynamics are further degrees of freedom invisible to the projection. Each layer deepens the epistemic boundary. The theory predicts, with machine-verified precision, not what happens in $H_("ont")$, but what cannot be determined from $H_("obs")$.

#figure(
  table(
  columns: 3,
  [Example],   [Result],   [Z3 Time],
  [Dissipative flow is admissible],  [SAT],  [\< 1 ms],
  [Dissipative flow has bounded action],  [SAT],  [104 ms],
  [Contractive flow is admissible],  [SAT],  [105 ms],
  [FLOW PREDICTION THEOREM: dissipative + contractive + bounded],  [SAT],  [101 ms],
),
  caption: [Z3 verification results for flow predictions (4 examples, all SAT)]
) <tab:flow>

= Discussion

We address scope, limitations, and relationship to existing work.

== What This Is Not

This paper does not contribute new results in set theory. The independence of CH was established by Godel (1940) and Cohen (1963). The observation that non-injectivity implies non-identifiability is standard in control theory (Kalman, 1960) and statistics. The philosophical observation that observation does not exhaust ontology is ancient.

What is new is the synthesis: (1) the explicit construction of shadow theories as a design principle for theory reduction, (2) the characterization of independence as non-invariance on projection fibers, machine-verified as a formal biconditional, (3) the computational detection of independence via SMT solvers, (4) the cross-domain unification showing that the same geometric structure underlies independence in logic, indeterminacy in physics, unobservability in control, and non-uniqueness in inverse problems, (5) the demonstration that fibers carry intrinsic structure --- metrics, coordinates, and dynamics --- establishing that observable stasis does not imply ontological stasis, (6) the Epistemic Boundary Theorem: multiple admissible variational principles can produce the same observable outcome while disagreeing on internal fiber assignments, proving that the specific dynamics governing the ontological layer is underdetermined from the projection side, and (7) the Arrow Underdetermination Theorem: the hidden arrow of evolution and metric character of ontological dynamics are not determined from the projection side, extending the epistemic boundary from state underdetermination to dynamical underdetermination.

== What This Is

The contribution is a computational framework for detecting and analyzing logical independence. The framework has three components:

(1) A methodology: given a theory and a target phenomenon, construct a shadow theory by Skolemization and verify that independence survives the projection.

(2) A tool: the Kleis language with Z3 backend provides an executable substrate for writing axioms, checking consistency, and probing the model polytope.

(3) A characterization: independence is non-invariance on projection fibers. This gives a geometric criterion that can be checked computationally (show both $S$ and $not S$ are SAT against the base axioms) and interpreted structurally (the independent statement is a kernel element of the projection).

== Limitations

The framework has inherent limitations:

(1) Decidability: Z3 is a decision procedure for quantifier-free theories and some quantified fragments. If the shadow theory falls outside these fragments, Z3 may return Unknown rather than SAT or UNSAT. In our experiments, all queries terminated within the quantifier-instantiation heuristics.

(2) Faithfulness: we do not formally prove that the Cantor shadow is faithful to ZFC. Faithfulness is a meta-theoretic property that would require a model-theoretic argument relating the shadow's models to ZFC's models. We take it as an empirical observation that the shadow correctly detects CH independence.

(3) Expressiveness: the shadow theory technique works best when the independent statement can be expressed in the constraint language. Independence of statements that involve the discarded vocabulary (e.g., 'there exists a Suslin line') would require a richer shadow.

= Conclusion

We have presented a framework in which logical independence is characterized as non-invariance on the fibers of a theory projection. The framework is computationally effective: the Z3 SMT solver detects CH independence in under 30 seconds via a Skolemized shadow of ZFC cardinal theory. The geometric structure --- a non-injective projection whose kernel contains the independent statement --- is universal, appearing across set theory, physics, quantum mechanics, control theory, and inverse problems.

Beyond independence detection, we showed that fibers carry intrinsic structure at three levels. First, geometry: fibers are metric spaces with positive separation between structurally distinct states. Second, dynamics: fibers support non-trivial evolution that preserves observables, yielding hidden trajectories. Third, admissible selection: an action functional on fibers provides a variational principle at the correct epistemic level --- we characterize the class of admissible dynamics without claiming to identify the specific Lagrangian.

The Epistemic Boundary Theorem establishes that multiple admissible variational principles can produce the same selected fiber point and the same observable consequence while disagreeing on internal action values. This proves that observers in $H_("obs")$ cannot determine which variational principle governs $H_("ont")$. The analogy with gauge theory is precise: the variational principle itself is fiber-local (ontological), while its structural consequences --- fiber geometry, selection existence, observable determination --- are projection-invariant. POT characterizes the geometry of the epistemic boundary, not the content beyond it. All 46 Z3 examples pass, the invariance biconditional is machine-verified, the epistemic boundary is computationally established, and the flow predictions extend the underdetermination to the thermodynamic arrow and metric character of ontological dynamics.

The practical implication is a new tool for exploring the boundaries of formal systems. Rather than attempting to prove or refute a conjecture, one can ask: 'Is this question projection-invariant?' If Z3 satisfies both the statement and its negation against the base axioms, the question lives in the kernel. The solver's silence is the answer. And the fiber's structure --- its metric, dynamics, and selection law --- tells you what kind of silence it is.

Future work includes formalizing the faithfulness criterion for shadow theories, equipping fibers with group structure (connecting to gauge theory and fiber bundles), deriving projection-invariant predictions from the admissible action class (observable consequences that hold for all admissible dynamics, not just one), and building automated tools that construct the minimal shadow and test projection-invariance for arbitrary theories and conjectures. The goal is a general-purpose independence detector with fiber geometry: a machine that tells you when to stop searching, and shows you the structure of what you cannot see.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[godel1940\] Godel, K. (1940). The Consistency of the Axiom of Choice and of the Generalized Continuum-Hypothesis with the Axioms of Set Theory. Annals of Mathematics Studies, No. 3. Princeton University Press.]

#par(hanging-indent: 1.5em)[\[cohen1963\] Cohen, P. J. (1963). The independence of the continuum hypothesis. Proceedings of the National Academy of Sciences, 50(6), 1143--1148.]

#par(hanging-indent: 1.5em)[\[cantor1874\] Cantor, G. (1874). Ueber eine Eigenschaft des Inbegriffes aller reellen algebraischen Zahlen. Journal fur die reine und angewandte Mathematik, 77, 258--262.]

#par(hanging-indent: 1.5em)[\[cantor1891\] Cantor, G. (1891). Ueber eine elementare Frage der Mannigfaltigkeitslehre. Jahresbericht der Deutschen Mathematiker-Vereinigung, 1, 75--78.]

#par(hanging-indent: 1.5em)[\[demoura2008\] de Moura, L. and Bjorner, N. (2008). Z3: An Efficient SMT Solver. In Tools and Algorithms for the Construction and Analysis of Systems (TACAS), LNCS 4963, pp. 337--340. Springer.]

#par(hanging-indent: 1.5em)[\[kalman1960\] Kalman, R. E. (1960). On the general theory of control systems. In Proceedings of the First International Congress of IFAC, pp. 481--492.]

#par(hanging-indent: 1.5em)[\[seshia2022\] Seshia, S. A., Sadigh, D., and Sastry, S. S. (2022). Toward Verified Artificial Intelligence. Communications of the ACM, 65(7), 46--55.]

#par(hanging-indent: 1.5em)[\[atik2025pot\] Atik, E. (2025). Flat Galactic Rotation Curves as a Theorem of Projected Ontology: Machine-Verified Derivations Without Dark Matter. Kleis Research, arXiv preprint.]

#par(hanging-indent: 1.5em)[\[zermelo1904\] Zermelo, E. (1904). Beweis, dass jede Menge wohlgeordnet werden kann. Mathematische Annalen, 59(4), 514--516.]

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#align(center)[
  #text(size: 14pt, weight: "bold")[Appendix]
]
#v(1em)
= Kleis Source Files

The complete verified source files accompanying this paper are:

cantor_set_theory.kleis --- The Cantor shadow theory. 12 structures, 35 axioms, 19 verified examples covering cardinal ordering, transfinite arithmetic, Cantor's theorem, the continuum, CH independence, forcing knobs, and GCH layering. Total verification time: approximately 25 seconds.

pot_bridge.kleis --- The projection-fiber bridge, invariance theorem, fiber dynamics, admissible selection, epistemic boundary, and flow predictions. 9 structures, 38 axioms, 27 verified examples demonstrating non-injective projection, kernel distinguishers, multi-model fibers, the formal biconditional (independence iff non-invariance), fiber evolution, fiber metrics, admissible action functionals, the Epistemic Boundary Theorem, and the Flow Prediction Theorem. Total verification time: approximately 9 seconds.

To reproduce all results:

kleis test examples/cantor/cantor_set_theory.kleis

kleis test examples/cantor/pot_bridge.kleis

All 46 examples should pass with exit code 0. Individual Z3 solve times are reported with KLEIS_Z3_DEBUG=1.

= Design Note: Well-Ordering as Tagging

In full ZFC, 'every set can be well-ordered' is equivalent to the Axiom of Choice. Our shadow theory does not model sets, functions, or choice functions --- it has only cardinal tags and arithmetic. Asserting a universal quantifier $forall k . "is_well_ordered"(k)$ would claim more structural content than the system encodes.

We therefore state well-ordering as ground facts about named cardinals rather than a universal axiom. The $"is_well_ordered"$ predicate is an honest tag in the shadow, not a theorem derived from set-theoretic machinery. This design choice follows the Skolemization principle: retain only what the shadow can structurally support.
