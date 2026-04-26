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
  #text(size: 17pt, weight: "bold")[Schanuel's Conjecture as a Hidden-Collapse Budget Problem]
  
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
      #text(size: 10pt)[We reframe Schanuel's conjecture as a hidden-collapse budget problem within a rank conservation framework. The exponential map $exp : bold(C) arrow bold(C)^times$ doubles a source tuple $(z_1, dots, z_n)$ into a paired tuple $(z_1, dots, z_n, e^(z_1), dots, e^(z_n))$ of $2 n$ elements. Algebraic relations among these elements consume a _destruction budget_ of at most $2 n$ degrees of freedom. We decompose this budget into three channels: _linear collapse_ (Q-linear dependence among the $z_i$), _visible collapse_ (periodicity of the exponential kernel $2 pi i bold(Z)$, which structures fibers without reducing transcendence degree), and _hidden collapse_ (algebraic relations not explained by either source). Schanuel's conjecture is equivalent to the assertion that hidden collapse is zero. We formalize this decomposition in the Kleis verification language and use the Z3 SMT solver to verify the rank conservation law and its consequences. For three probe families --- $n = 1$ with $z = (1)$, $n = 2$ in the Gelfond--Schneider configuration, and $n = 2$ with the Euler identity --- we tabulate every integer value of the hidden-collapse parameter: which values the rank budget allows, which classical transcendence theorems (Hermite, Lindemann, Gelfond--Schneider) exclude, and which remain genuinely open. The result is a Z3-verified stratification of the conjecture's unresolved arithmetic core. For the Euler probe, classical theorems eliminate hidden $gt.eq 2$, leaving a single binary residue: does there exist a nonzero polynomial $P in bold(Q)[x, y]$ with $P(e, pi) = 0$? All 58 structural examples are verified by the Z3 solver. The framework does not prove the conjecture; it localizes its content.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* Schanuel conjecture, transcendence degree, algebraic independence, hidden collapse, rank conservation, formal verification, Z3, exponential map]

#v(1em)


= Introduction

Schanuel's conjecture [1] states that for any $n$ complex numbers $z_1, dots, z_n$ that are linearly independent over $bold(Q)$, the transcendence degree of the field extension $bold(Q)(z_1, dots, z_n, e^(z_1), dots, e^(z_n))$ over $bold(Q)$ is at least $n$:
$ "trdeg"_bold(Q) thin bold(Q)(z_1, dots, z_n, e^(z_1), dots, e^(z_n)) gt.eq n. $
The conjecture, first formulated in the 1960s and first published in Lang [1], subsumes virtually all known results in transcendental number theory as special cases: the Hermite--Lindemann theorem [2, 3], the Gelfond--Schneider theorem [4, 5], the algebraic independence of $e$ and $pi$ (open), and the transcendence of $e^e$ (open).

Despite its central position, Schanuel's conjecture remains unproved for any $n gt.eq 2$. The $n = 1$ case is classical: it reduces to the Hermite--Lindemann theorem. The functional analogue --- the Ax--Schanuel theorem [6] --- is proved for differential fields, but the transfer to specific constants ($e$, $pi$) is open.

This note proposes a computational framework for _localizing_ the conjecture's content. Rather than attempting a proof, we ask: for specific probe configurations, what exactly is the unresolved arithmetic residue?

The method has three steps:
+ *Rank budget.* The exponential map doubles a tuple of size $n$ into $2 n$ elements. A rank conservation law (Section 3) bounds the total algebraic collapse.
+ *Classical pruning.* Known transcendence theorems (Hermite, Lindemann, Gelfond--Schneider) eliminate certain hidden-collapse values.
+ *Residue identification.* What remains is the exact set of open values --- the live content of the conjecture for that configuration.

A central observation motivates the analysis. The exponential map $Phi(u) = e^u$ is a group homomorphism: it converts addition to multiplication, and its kernel ($2 pi i bold(Z)$) is well understood. At this level, everything is structurally simple. But Schanuel's conjecture is not about the map --- it is about _polynomial relations on its graph_. The graph $\{(z, e^z)\} subset bold(C)^(2n)$ lives in an algebraic world that the group structure cannot control. This gap between the tractable group-theoretic layer and the intractable algebraic layer is the source of the difficulty, and it is the structural theme of this paper.

All structural implications are verified by the Z3 SMT solver [8] within the Kleis formal verification language [9]. Z3 operates in $"QF"_"NRA"$ (quantifier-free nonlinear real arithmetic). We note explicitly that transcendence is a second-order property --- it quantifies universally over all polynomials --- and is therefore not expressible in $"QF"_"NRA"$. The verification checks the _implication structure_ of the rank law, not the conjecture itself.

= The Rank Conservation Law

Let $z = (z_1, dots, z_n) in bold(C)^n$ with Q-linear rank $r = dim_bold(Q) chevron.l z chevron.r$. The _paired tuple_ is $Phi(z) = (z_1, dots, z_n, e^(z_1), dots, e^(z_n))$, which has $2 n$ elements. Define:
$ "shadow" = "trdeg"_bold(Q) thin bold(Q)(Phi(z)), quad quad "budget" = 2 n. $
Algebraic relations among the elements of $Phi(z)$ reduce the shadow below the budget. We write:
$ "budget" = "shadow" + "destroyed", $
where $"destroyed" gt.eq 0$ counts the degrees of freedom consumed by algebraic relations.

Schanuel's conjecture asserts $"shadow" gt.eq r$, which gives:
$ "destroyed" lt.eq 2 n - r. $

We decompose the destruction into three channels:

*Linear collapse* ($= n - r$): loss from Q-linear dependence among the source $z_i$. This is _structural_ --- it is visible from the source alone.

*Visible collapse*: the exponential kernel $2 pi i bold(Z)$ identifies fibers ($exp(z) = exp(w)$ iff $z - w in 2 pi i bold(Z)$). Critically, periodicity contributes to fiber structure but does not introduce algebraic relations among the elements of $Phi(z)$. It is invisible to transcendence degree.

*Hidden collapse* ($h gt.eq 0$): algebraic relations among $(z, e^z)$ not explained by Q-linear dependence or periodicity.

The decomposition gives:
$ "shadow" = n + r - h, $
and Schanuel becomes:
$ n + r - h gt.eq r quad arrow.r.double quad h lt.eq n. $

#block(stroke: 1pt + black, inset: 12pt, radius: 4pt, width: 100%)[
  *Critical distinction.* The Schanuel _inequality_ ($"shadow" gt.eq r$) implies $h lt.eq n$. The Schanuel _conjecture's content_ is the stronger claim that for the exponential map, $h = 0$: all algebraic collapse is structural. These are different statements. The inequality is weaker.
]

For full-rank tuples ($r = n$):
$ "shadow" = 2 n - h, quad h lt.eq n, quad "Schanuel predicts" h = 0. $
This is a budget constraint: exponentiation starts with $2 n$ degrees of freedom, and the total spending on hidden relations is bounded.

$ 2 n = "shadow" + "destroyed", quad quad "destroyed" = (n - r) + h, quad quad h lt.eq n $ <eq:conservation>

= Hidden-Collapse Tabulation

For each probe family, we enumerate every integer value of the hidden-collapse parameter $h$ and classify it into one of four categories:

- *Budget-allowed:* the rank conservation law permits this value ($h lt.eq n$).
- *Classically excluded:* a proved transcendence theorem rules this value out.
- *Schanuel-excluded:* $h > n$, violating the rank inequality.
- *Still open:* budget-allowed and not classically excluded.

The tables below are verified by Z3: each row corresponds to a structure whose axioms encode the constraints, and each classification is an example whose assertion Z3 checks.

== Probe 1: $n = 1$, $z = (1)$

Source: $z = (1)$, Q-rank $r = 1$, budget $= 2$. Paired tuple: $(1, e)$.

#figure(
  table(
    columns: (auto, auto, auto, auto),
    align: (center, center, center, left),
    stroke: 0.5pt,
    table.header[$h$][Status][Shadow][Meaning],
    [0], [allowed], [2], [maximum freedom],
    [1], [allowed], [1], [structural: '1 is rational'],
    [$gt.eq 2$], [Schanuel-excl.], [$< 1$], [violates rank inequality],
  ),
  caption: [Hidden-collapse values for $n = 1$, $z = (1)$.]
) <tab:n1>

*Open values:* $\{0, 1\}$. However, $h = 1$ corresponds to the trivial relation '1 is algebraic over $bold(Q)$' --- this is structural, not genuine hidden collapse. The search space is degenerate: no room for genuine hidden collapse at $n = 1$.

This is consistent with the classical situation: Hermite's theorem [2] proves the $n = 1$ case of Schanuel outright.

== Probe 2: $n = 2$, Gelfond--Schneider Configuration

Source: $z = (log a, b dot log a)$ where $a, b$ are algebraic, $b$ irrational. Q-linearly independent ($b$ irrational) $arrow.r.double r = 2$, budget $= 4$.

Paired tuple: $(log a, b dot log a, a, a^b)$. One value ($a$) is algebraic.

#figure(
  table(
    columns: (auto, auto, auto, auto),
    align: (center, center, center, left),
    stroke: 0.5pt,
    table.header[$h$][Status][Shadow][Meaning],
    [0], [open], [4], [full independence over $bold(Q)(a)$],
    [1], [open], [3], [one nontrivial algebraic relation],
    [2], [open], [2], [tight Schanuel bound],
    [$gt.eq 3$], [Schanuel-excl.], [$< 2$], [violates rank inequality],
  ),
  caption: [Hidden-collapse values for $n = 2$, Gelfond--Schneider.]
) <tab:gs>

*Classical constraint:* The Gelfond--Schneider theorem [4, 5] proves that $a^b$ is transcendental, which constrains which _realizations_ of each hidden value are arithmetically possible. However, it does not eliminate any abstract budget value entirely.

*Open values:* $\{0, 1, 2\}$. The Gelfond--Schneider probe has the widest residual gap of the three configurations.

== Probe 3: $n = 2$, Euler Identity

Source: $z = (1, pi i)$, Q-linearly independent $arrow.r.double r = 2$, budget $= 4$.

Paired tuple: $(1, pi i, e, -1)$. Two values ($1$ and $-1$) are algebraic. The non-algebraic content is $"trdeg"_bold(Q) thin bold(Q)(e, pi)$.

#figure(
  table(
    columns: (auto, auto, auto, auto),
    align: (center, center, center, left),
    stroke: 0.5pt,
    table.header[$h$][Status][$"trdeg"_bold(Q) thin bold(Q)(e, pi)$][Meaning],
    [0], [*open*], [2], [$e$, $pi$ algebraically independent],
    [1], [*open*], [1], [nontrivial algebraic dependence],
    [2], [_excluded_], [0], [total collapse --- contradicts Hermite + Lindemann],
    [$gt.eq 3$], [Schanuel-excl.], [$< 0$], [violates rank inequality],
  ),
  caption: [Hidden-collapse values for $n = 2$, Euler. The _excluded_ row is eliminated by classical theorems, not the rank law.]
) <tab:euler>

*Classical exclusion:* $h = 2$ forces $"trdeg"_bold(Q) thin bold(Q)(e, pi) = 0$, meaning $e$ and $pi$ are both algebraic. This contradicts Hermite [2] ($e$ transcendental, 1873) and Lindemann [3] ($pi$ transcendental, 1882).

*Open values:* $\{0, 1\}$. The residue is a single binary question:

#block(stroke: 1pt + black, inset: 12pt, radius: 4pt, width: 100%)[
  Does there exist a nonzero polynomial $P in bold(Q)[x, y]$ with $P(e, pi) = 0$?

  Schanuel predicts: *no*. Current mathematics: *unknown*.
]

If $h = 1$, the dependence is not necessarily linear --- it could be a polynomial relation of any degree, consistent with the known individual transcendence of $e$ and $pi$. This is the simplest open instance of hidden collapse.

= Summary and Trend

#figure(
  table(
    columns: (auto, auto, auto, auto, auto),
    align: (left, center, center, center, center),
    stroke: 0.5pt,
    table.header[Probe][$n$][Budget-allowed][Classically excluded][Still open],
    [$z = (1)$], [1], [$\{0, 1\}$], [$emptyset$], [$\{0, 1\}$ (structural)],
    [Gelfond--Schneider], [2], [$\{0, 1, 2\}$], [$emptyset$], [$\{0, 1, 2\}$],
    [Euler], [2], [$\{0, 1, 2\}$], [$\{2\}$], [$\{0, 1\}$],
  ),
  caption: [Summary of hidden-collapse residues across probe families.]
) <tab:summary>

The pattern across probes is:

*At $n = 1$:* the budget is tight and the only budget-allowed value beyond $h = 0$ is structural. No genuine hidden collapse is possible. The classical toolbox (Hermite) resolves this case entirely.

*At $n = 2$:* the budget grows to $\{0, 1, 2\}$. Classical theorems begin to prune: in the Euler configuration, Hermite and Lindemann together eliminate $h = 2$. In the Gelfond--Schneider configuration, the theorem constrains realizations but does not eliminate abstract budget values.

*The trend:* as $n$ increases, the rank budget grows ($h in \{0, dots, n\}$), while the set of applicable classical exclusions grows more slowly. The conjecture's unresolved residue _widens_ with $n$. This is the structural reason Schanuel rapidly escapes the reach of the classical transcendence toolkit.

= Three Structural Barriers

The degree-bounded Skolemization strategy (detailed in the companion file [9]) reveals that the difficulty of Schanuel's conjecture is not merely that a proof has not been found. The conjecture lies outside the expressive and geometric reach of entire classes of formal systems. We identify three independent barriers, each a theorem about the underlying mathematics rather than an artifact of computational limitation.

== Barrier 1: Logical Order (Expressibility)

The statement '$x$ is transcendental' means: for all nonzero $P in bold(Q)[t]$, $P(x) eq.not 0$. This quantifies universally over an infinite polynomial ring. It is a second-order predicate --- not expressible as a first-order sentence in $"QF"_"NRA"$ (the theory of real closed fields).

$"QF"_"NRA"$ does not _fail to prove_ transcendence. It cannot _form the predicate_. This is a Tarski-style boundary between logical strata, not a computational limitation.

== Barrier 2: Topology vs Arithmetic (Approximation)

Algebraic numbers are dense in $bold(R)$: any interval $[a, b]$ contains algebraic numbers of every degree. Transcendence is a property of _annihilators_ (which polynomials vanish), not _location_ (where the number sits on the real line). No finite amount of metric information can distinguish algebraic from transcendental. This kills any limit-based or interval-based certification strategy --- not because of insufficient precision, but because the distinction is arithmetic, not topological.

== Barrier 3: Geometry vs Arithmetic (Dimensional Ceiling)

$bold(C)$ is a 2-dimensional vector space over $bold(R)$: any three complex numbers are $bold(R)$-linearly dependent. Schanuel's hypothesis, however, requires $bold(Q)$-linear independence, a strictly finer notion ($dim_bold(Q) bold(C)$ is uncountable). $bold(R)$-linear dependence does not constrain $bold(Q)$-linear independence; it only shows that real-coordinate models cannot witness it. For the $n = 1$ and $n = 2$ probes in this paper, all rank quantities are treated axiomatically over $bold(N)$, so the bookkeeping is faithful. For $n gt.eq 3$, any faithful realization of the source tuple would require a genuine complex-field semantics or a multi-sorted theory.

== The Barriers as a Diagnosis

#figure(
  table(
    columns: (auto, auto, auto),
    align: (left, left, left),
    stroke: 0.5pt,
    table.header[Barrier][What it says][What it blocks],
    [Logical order], [Transcendence is second-order], [Predicate cannot be formed in $"QF"_"NRA"$],
    [Topology vs arithmetic], [Algebraic numbers are dense], [No interval method can certify transcendence],
    [Geometry vs arithmetic], [$dim_bold(R) bold(C) = 2$ but $dim_bold(Q) bold(C) = infinity$], [$bold(R)$-coordinates cannot witness $bold(Q)$-independence],
  ),
  caption: [Three independent structural barriers. Each is a theorem about the underlying mathematics.]
) <tab:barriers>

These barriers are orthogonal: each blocks access to Schanuel's content through a different mechanism. Together, they explain why the conjecture is inaccessible to real-closed-field decision procedures and finite-degree certificate methods --- not because of insufficient ingenuity, but because the problem inhabits a stratum of mathematical structure that these frameworks cannot reach.

This does not mean the conjecture is beyond mathematics. Differential algebra (Ax--Schanuel [6]) and model theory (Zilber [7]) _do_ access it partially, precisely because they introduce richer structure: differential fields supply the functional analogue, and pseudo-exponentiation constructs a model where Schanuel holds by design. The classical proofs of special cases (Hermite [2], Lindemann [3], Gelfond--Schneider [4, 5]) succeed by introducing auxiliary functions, integral representations, and factorial growth estimates that encode the second-order content without explicitly quantifying over all polynomials. They bypass the barriers rather than operating within them.

The residual hidden-collapse values computed in Sections 3--4 therefore represent not a failure of technique, but the irreducible core of the conjecture beyond the reach of real-closed-field reasoning.

The three decidability levels identified by the degree-bounded Skolemization are:
+ *Decidable:* fixed $n$, fixed $d$, fixed witness.
+ *Semi-decidable:* fixed $n$, all $d$, fixed witness.
+ *Undecidable:* all $n$, all $d$, find witness.
Schanuel lives at level 3. We probe it from level 1.

= Connections and Prior Work

The hidden-collapse formulation aligns with several active research programs. We note where prior work has identified related structural observations.

*Macintyre--Wilkie [10].* Macintyre and Wilkie showed that if Schanuel's conjecture holds, the first-order theory of the real exponential field $(bold(R); +, dot, exp, <)$ is decidable. This is the closest precedent to our barrier analysis: their result identifies Schanuel as a _gateway_ between decidability and undecidability for an entire theory. Our three barriers can be read as a finer decomposition of why this gateway is so hard to pass through.

*Ax--Schanuel [6].* The functional analogue of Schanuel is proved for differential fields. The open problem is the _transfer_ from generic differential-field elements to specific constants ($e$, $pi$). In our language: Ax--Schanuel proves $h = 0$ for formal power series; the transfer problem asks whether this specializes to the arithmetic setting. Pila and Tsimerman [11] extended Ax--Schanuel to the modular $j$-function, showing that the structural pattern (rank preservation under a transcendental map) persists far beyond the exponential.

*Zilber's pseudo-exponentiation [7].* Zilber constructs a model of $bold(C)$ equipped with an exponential function in which Schanuel holds by construction. The model is uncountably categorical [7]. The open question is whether this model is isomorphic to $(bold(C), exp)$. An affirmative answer would prove Schanuel. In the language of Sections 5 and 7, Zilber's construction operates at Level 3 of the expressiveness tower: it axiomatizes the _absence_ of hidden collapse, then builds a model satisfying those axioms.

*Kirby's exponential algebraic closure [12].* Kirby proved that the exponential algebraic closure operator (ecl) is a pregeometry, yielding a well-defined dimension function. This dimension satisfies a weak form of Schanuel's conjecture and implies that there are at most countably many _essential_ counterexamples. Kirby's dimension function is the closest existing analogue to our rank budget: both measure degrees of freedom consumed by algebraic relations in the presence of exponentiation.

*Nesterenko [13].* The strongest unconditional result in the direction of Schanuel: $pi$ and $e^pi$ are algebraically independent over $bold(Q)$. More generally, $pi$, $e^(pi sqrt(n))$, and $Gamma(1 slash 3)$ (or $Gamma(1 slash 4)$) form algebraically independent triples. In our tabulation, Nesterenko's result eliminates specific hidden-collapse values for probes involving modular-function evaluations --- extending the classical pruning beyond Hermite--Lindemann--Gelfond--Schneider.

*Waldschmidt [14].* Waldschmidt's recent survey introduces the _strong Schanuel property_ --- the condition that all $2 n$ elements of the paired tuple are algebraically independent, giving transcendence degree $2 n$ rather than the conjectured lower bound $n$. Almost all $n$-tuples satisfy this property (for Lebesgue measure on $bold(C)^n$). This is the measure-theoretic counterpart of our observation that the hidden-collapse budget is generically zero: the 'typical' tuple has no collapse at all. The difficulty is entirely concentrated on _specific_ arithmetically significant tuples ($1$, $pi i$, $log alpha$, etc.).

*Richardson [15].* Richardson proved that the problem of determining whether expressions built from $bold(Q)$, $pi$, $ln 2$, $sin$, and $exp$ equal zero is undecidable. This is a concrete instantiation of our Barrier 1: once exponentiation and periodic functions interact, even the _identity_ problem escapes algorithmic reach. Richardson's result operates at the level of expression equivalence; our analysis operates at the level of algebraic independence.

*Period conjectures.* The Kontsevich--Zagier period conjecture and Grothendieck's conjecture on periods both concern the transcendence properties of integrals. In the hidden-collapse language, these conjectures assert that certain classes of numbers have $h = 0$ --- no hidden algebraic relations beyond those forced by the integral representation. Grothendieck's conjecture is formulated in terms of the Mumford--Tate group, which plays a role analogous to our rank budget: it measures the symmetry group that controls algebraic relations among periods.

= The Exponential Coordinate Morphism

The rank conservation law (Section 2) becomes geometrically transparent once we name the underlying map.

== Additive coordinates, multiplicative coordinates

Let $V = "span"_bold(Q)(1, z_2, dots, z_n) subset bold(C)$ be a $bold(Q)$-linear subspace of rank $n$. Every element of $V$ has the form $u = q_1 + q_2 z_2 + dots.c + q_n z_n$ with $q_i in bold(Q)$. This is an _additive coordinate system_ over $bold(Q)$.

Define the _exponential coordinate morphism_
$ Phi : (V, +) arrow.r (bold(C)^times, dot), quad quad Phi(u) = e^u. $
Since $e^(u + v) = e^u dot e^v$, the map $Phi$ is a group homomorphism: addition upstairs becomes multiplication downstairs.

The image factorizes:
$ e^u = e^(q_1) dot (e^(z_2))^(q_2) dots.c (e^(z_n))^(q_n). $
The generators $e, e^(z_2), dots, e^(z_n)$ form a _multiplicative coordinate system_: the additive $bold(Q)$-coordinates $(q_1, dots, q_n)$ become multiplicative exponents. When $1$ is in the additive basis, the fundamental constant $e = e^1$ is automatically among the generators.

The kernel of $Phi$ is the periodic lattice: $Phi(u) = Phi(v)$ iff $u - v in 2 pi i bold(Z)$. This is the _visible_ fiber structure. Schanuel asks whether there is any _hidden_ collapse beyond it.

== The algebraic shadow

The map $Phi$ is a group homomorphism, and its kernel is well understood. At this level, everything is tractable. The difficulty arises when we pass from the group-theoretic picture to the algebraic one.

Schanuel's conjecture is not about the map $Phi$ itself but about its _graph_:
$ Gamma = \{ (z_1, dots, z_n, e^(z_1), dots, e^(z_n)) \} subset bold(C)^(2n). $
The conjecture asks: what are the polynomial relations (over $bold(Q)$) satisfied by points of $Gamma$? A group homomorphism controls addition and multiplication, but it says nothing about polynomial identities on the graph. The algebraic shadow of $Phi$ lives in a different logical stratum from its group structure.

== The tower of expressiveness

A natural question is whether one can build a single algebraic category that carries both the additive/group data and the polynomial data simultaneously. The answer reveals a structural obstruction.

To control all algebraic relations on $Gamma$, one must quantify over all polynomials $P in bold(Q)[x_1, dots, x_n, y_1, dots, y_n]$. This is second-order. To formalize the second-order constraint, one introduces a meta-level that reasons about sets of polynomials. To constrain _that_ level, one needs a further meta-level. The result is an unavoidable hierarchy:

#block(stroke: 1pt + black, inset: 12pt, radius: 4pt, width: 100%)[
  *Level 0:* Field operations ($+$, $dot$, polynomial algebra). \
  *Level 1:* Exponential structure ($exp$, the morphism $Phi$, its kernel). \
  *Level 2:* Algebraic relations on the graph of $exp$ (transcendence degree). \
  *Level 3:* Universal statements about all such relations (Schanuel).
]

Each level requires expressive resources absent from the one below. Schanuel lives at Level 3. The real-closed-field solver operates at Level 0. The three barriers of Section 5 are precisely the gaps between adjacent levels: logical order (Level 0 $arrow$ Level 2), approximation (Level 0 $arrow$ Level 2), and dimensional ceiling (Level 0 $arrow$ Level 1).

This tower is not an artifact of formalization. It reflects a genuine stratification of mathematical structure: the objects (numbers), the maps between them (exponentiation), the relations among images (algebraic dependence), and the constraints on those relations (Schanuel). Any framework that attempts to flatten this hierarchy must either sacrifice finite axiomatizability or weaken its control over one of the levels.

= Conclusion

This paper does not get closer to proving Schanuel's conjecture. It gets closer to answering a different question: _what kind of statement is Schanuel?_

The hidden-collapse framework computes the values permitted by the rank conservation law, classifies which classical transcendence theorems exclude, and identifies the residual values where the genuine open content lives. For the Euler configuration, the residue is a single binary question: does there exist a nonzero $P in bold(Q)[x, y]$ with $P(e, pi) = 0$?

But the deeper finding is structural. The three barriers of Section 5 and the tower of Section 7 are not artifacts of the solver or the framework. They are theorems about the relationship between different levels of mathematical language. Schanuel's conjecture does not merely resist proof; it inhabits a level of mathematical structure that is invisible to real-closed-field models and finite approximation methods. Any proof must introduce structure that transcends these limitations --- as the classical proofs of special cases do, via auxiliary functions, integral representations, and factorial growth estimates that encode second-order content without explicit universal quantification.

The situation bears a family resemblance to four foundational results about the limits of formal frameworks. Cantor showed that countable descriptions cannot exhaust the continuum (a size mismatch). Galois showed that radical expressions cannot solve generic polynomials (a symmetry obstruction). Poincaré showed that exact solutions are less informative than invariants under deformation (a classification shift). Gödel showed that sufficiently rich formal systems contain truths they cannot prove (an intrinsic limit). Each identified a boundary where a given representational framework fails to capture the objects it seeks to describe. The present analysis identifies an analogous boundary for real-closed-field reasoning and transcendence: the exponential coordinate morphism is a group homomorphism at Level 1, but the conjecture lives at Level 3 of an expressiveness hierarchy that cannot be flattened.

The residual hidden-collapse values therefore represent not a failure of technique, but the irreducible core of the conjecture beyond the reach of these frameworks. At larger $n$, the rank budget grows while classical exclusions thin out. The conjecture _widens_ faster than the toolkit. That structural observation --- that mathematical reality exceeds the expressive power of certain representation systems --- is itself a contribution to understanding why Schanuel remains open after six decades.

All 58 structural examples are verified by Z3 in the companion Kleis file. The source code, verification output, and this paper are available at #link("https://kleis.io")[kleis.io].



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] S. Lang, 'Introduction to Transcendental Numbers,' Addison-Wesley (1966). The first published statement of Schanuel's conjecture.]

#par(hanging-indent: 1.5em)[\[2\] C. Hermite, 'Sur la fonction exponentielle,' Comptes Rendus Acad. Sci. Paris 77, 18--24, 74--79, 226--233, 285--293 (1873).]

#par(hanging-indent: 1.5em)[\[3\] F. Lindemann, 'Über die Zahl $pi$,' Math. Ann. 20, 213--225 (1882).]

#par(hanging-indent: 1.5em)[\[4\] A.O. Gelfond, 'Sur le septième problème de Hilbert,' Izv. Akad. Nauk SSSR 7, 623--634 (1934).]

#par(hanging-indent: 1.5em)[\[5\] Th. Schneider, 'Transzendenzuntersuchungen periodischer Funktionen I, II,' J. Reine Angew. Math. 172, 65--69 (1934).]

#par(hanging-indent: 1.5em)[\[6\] J. Ax, 'On Schanuel's conjecture,' Ann. of Math. 93, 252--268 (1971).]

#par(hanging-indent: 1.5em)[\[7\] B. Zilber, 'Pseudo-exponentiation on algebraically closed fields of characteristic zero,' Ann. Pure Appl. Logic 132, 67--95 (2005).]

#par(hanging-indent: 1.5em)[\[8\] L. de Moura and N. Bjørner, 'Z3: An efficient SMT solver,' in TACAS 2008, LNCS 4963, pp. 337--340, Springer (2008).]

#par(hanging-indent: 1.5em)[\[9\] E. Atik, 'Schanuel's Conjecture --- Fiber Non-Collapse for Exponentiation,' Kleis verification file, https://kleis.io (2026). Companion source code with 58 Z3-verified examples.]

#par(hanging-indent: 1.5em)[\[10\] A. Macintyre and A.J. Wilkie, 'On the decidability of the real exponential field,' in Kreiseliana: About and Around Georg Kreisel, ed. P. Odifreddi, pp. 441--467, A K Peters (1996).]

#par(hanging-indent: 1.5em)[\[11\] J. Pila and J. Tsimerman, 'Ax--Schanuel for the $j$-function,' Duke Math. J. 165, 2587--2605 (2016).]

#par(hanging-indent: 1.5em)[\[12\] J. Kirby, 'Exponential algebraicity in exponential fields,' Bull. London Math. Soc. 42, 879--890 (2010).]

#par(hanging-indent: 1.5em)[\[13\] Yu. V. Nesterenko, 'Modular functions and transcendence questions,' Mat. Sb. 187, 65--96 (1996). English transl. Sb. Math. 187, 1319--1348 (1996).]

#par(hanging-indent: 1.5em)[\[14\] M. Waldschmidt, 'On Schanuel's Conjecture, elliptic and quasi-elliptic functions,' lecture notes, Sorbonne Universit\'e (2025).]

#par(hanging-indent: 1.5em)[\[15\] D. Richardson, 'Some undecidable problems involving elementary functions of a real variable,' J. Symbolic Logic 33, 514--520 (1968).]


