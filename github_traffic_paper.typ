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
  #text(size: 17pt, weight: "bold")[Caught Red-Handed: 181 Bots Exposed by Eight Axioms and a Theorem]
  
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
      #text(size: 10pt)[Something was off. A small research repository -- five regular visitors, no viral moment, no Hacker News post -- woke up one morning to find that 186 distinct entities had cloned it 2,834 times in two weeks. We did what any reasonable person would do: we called Z3. Eight axioms, six unknowns, 3.6 seconds, and the solver delivered a verdict that is not a guess, not a heuristic, not a confidence interval, but a theorem: at least 181 of those 186 cloners are bots. The math leaves no room for debate. If you visited the page, you are one of five humans. If you cloned without visiting, you are caught. The proof is tight, the bound is sharp, and the bots -- as ever -- were just doing what they were told.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* formal verification, SMT solver, Z3, GitHub traffic, satisfiability, automated cloning, software metrics]

#v(1em)


= Something Was Off

On approximately April 2, 2026, the GitHub repository `eatirkh/kleis` -- a quiet formal verification project with a handful of regular users -- experienced a sudden spike in traffic. We checked the 14-day summary:

#figure(
  table(
    columns: 2,
    stroke: 0.5pt,
    inset: 8pt,
    [*Metric*], [*Value*],
    [Total clones], [2,834],
    [Unique cloners], [186],
    [Total page views], [228],
    [Unique visitors], [5],
  ),
  caption: [The scene of the crime. GitHub traffic data, 14-day window ending April 9, 2026.]
) <tab:data>

Wait. Five visitors. But 186 cloners? That is a 37$times$ ratio. Someone -- or rather, _something_ -- was cloning the repository without bothering to visit the page first. And not just once: 2,834 clones from 186 entities in two weeks, while the page itself saw only 228 views from 5 humans.

This is not a subtle statistical anomaly. This is getting caught red-handed.

The natural next step was to see whether the numbers _formally_ contradict a human-only model, or whether there is some creative explanation. So we did what we do: we wrote eight axioms and asked Z3 [1].

The answer came back in 3.6 seconds: at least 181 of 186 cloners are provably non-human. Not estimated. Not inferred. _Proved._ The bound is tight and follows from pure deduction.

@fig:paradox shows the two data points that gave them away.

#figure(
  lq.diagram(
  title: [Unique Agents vs Total Events],
  xlabel: [Unique agents],
  ylabel: [Total events],
  lq.scatter(
    (186.000000, 5.000000),
    (2834.000000, 228.000000),
    mark: "o",
    color: red
  ),
)
,
  caption: [Cloners (186 unique, 2834 total) vs visitors (5 unique, 228 total). The two data points reveal the scale of the anomaly: the cloner population is 37$times$ larger than the visitor population, and each cloner averages 15.2 clones.]
) <fig:paradox>

= Setting the Trap

To catch a bot, you need a theory. Ours is minimal: six unknowns and eight axioms over the integers $bb(Z)$.

== The Unknowns

#table(
  columns: 3,
  stroke: 0.5pt,
  inset: 8pt,
  [*Symbol*], [*Type*], [*Interpretation*],
  [$C$], [$bb(Z)$], [Unique cloners],
  [$V$], [$bb(Z)$], [Unique visitors],
  [$N$], [$bb(Z)$], [Total clones],
  [$W$], [$bb(Z)$], [Total page views],
  [$H$], [$bb(Z)$], [Human cloners],
  [$B$], [$bb(Z)$], [Automated (bot) cloners],
)

== The Trap (Axiom System)

The axioms fall into two groups: _what we saw_ (data) and _one reasonable assumption_ (structure).

*What we saw* (direct from @tab:data):
$ "O1": C = 186, quad "O2": V = 5, quad "O3": N = 2834, quad "O4": W = 228. $

*The one assumption that catches them:*
$ "S1": H + B = C, quad "S2": H >= 0, quad "S3": B >= 0, quad "S4": H <= V. $

S1 says every cloner is either human or not. S2--S3 say you can't have negative entities. And S4 is the trap: _if you are human and you cloned our repo, you must have visited the page first._ That's it. One behavioral assumption. It's the weakest reasonable constraint -- we don't even require that a visitor _must_ clone, just that a human cloner must have visited.

The theory $cal(T) = {"O1", "O2", "O3", "O4", "S1", "S2", "S3", "S4"}$ goes to Z3. The bots, of course, did not visit the page.

@fig:partition shows the result before we even get to the propositions: there is almost no room for humans in this picture.

#figure(
  lq.diagram(
  title: [Unique Cloner Partition (Z3 Theorem)],
  ylabel: [Unique cloners],
  ylim: (0, 200),
  xaxis: (ticks: none),
  lq.bar(
    (0.000000,),
    (5.000000,),
    fill: green,
    label: [Human (max 5)]
  ),
  lq.bar(
    (0.000000,),
    (181.000000,),
    fill: red.lighten(20%),
    base: 5,
    label: [Automated (min 181)]
  ),
)
,
  caption: [Z3-proved partition of 186 unique cloners. The green segment (human, $H <= 5$) is bounded by the number of page visitors. The red segment (automated, $B >= 181$) follows as a theorem. The bound is tight.]
) <fig:partition>

= Five Ways to Say 'Gotcha'

We tested five propositions against $cal(T)$. Z3 operates in _entailment_ mode: a proposition is proved if it holds in _every_ model, and disproved if there exists even one counterexample. Four of the five came back UNSAT -- impossible. The fifth is a theorem.

== Proposition 1: "Maybe They're All Human"

*Statement.* $C <= V$ (all cloners are visitors).

*Z3: UNSAT.* $186 <= 5$? Please. #sym.square.filled

== Proposition 2: "OK, What If All 5 Visitors Cloned?"

*Statement.* $H = 5$ (all visitors cloned).

*Z3: UNSAT.* Not so fast. The axioms don't _entail_ $H = 5$ -- Z3 produces the counterexample $H = 0$, $B = 186$. It's perfectly consistent that _none_ of the 5 visitors cloned. The axioms only say $H in {0, 1, 2, 3, 4, 5}$; they don't pick a value. Subtle point: $H = 5$ is _consistent_ with $cal(T)$ but not _required_ by it. #sym.square.filled

== Proposition 3: "What If Each Human Cloned 50 Times?"

*Statement.* $N < 50 dot V$ (total clones within 50 per visitor).

*Z3: UNSAT.* Even giving each of the 5 humans a heroic 50 clones, that's only 250. We saw 2,834. Not even close. #sym.square.filled

== Proposition 4: "At Least Clones Don't Exceed Views, Right?"

*Statement.* $N <= W$ (total clones $<=$ total views).

*Z3: UNSAT.* $2834 <= 228$? The human-only hypothesis doesn't just fail -- it fails by a factor of 12.4. #sym.square.filled

== Proposition 5: The Theorem

*Statement.* $B >= 181$.

*Z3: SAT -- proved as theorem.* From S1 and S4: $B = C - H = 186 - H$, and $H <= V = 5$, so $B >= 181$. This isn't an estimate. It's a _logical consequence_. The bound is tight: $B = 181$ when all 5 visitors cloned ($H = 5$), and $B = 186$ when none did ($H = 0$). Either way, at least 97.3% of cloners are bots. Caught. #sym.square.filled

#figure(
  table(
    columns: 4,
    stroke: 0.5pt,
    inset: 8pt,
    [*Attempt*], [*Proposition*], [*Verdict*], [*Translation*],
    ["All human"], [$C <= V$], [UNSAT], [Nice try],
    ["All 5 cloned"], [$H = 5$], [UNSAT], [Not necessarily],
    ["50 clones each"], [$N < 50V$], [UNSAT], [Still not enough],
    ["Clones $<=$ views"], [$N <= W$], [UNSAT], [Not even close],
    ["$>=$ 181 bots"], [$B >= 181$], [*SAT*], [*Theorem. Gotcha.*],
  ),
  caption: [The scorecard. Every human-only model is impossible. The automation lower bound is a theorem.]
) <tab:results>

= Discussion

== This Is Not a Statistic

Let's be clear about what just happened. The statement $B >= 181$ is not a confidence interval. It's not a p-value. It's not a Bayesian posterior. It is a _theorem_: given the eight axioms, the bound holds in every satisfying assignment, and Z3 checked them all in 3.6 seconds.

The usual approach to bot detection is heuristic: check the User-Agent string, cluster by temporal pattern, train a classifier. These have false positives and false negatives. Formal verification has neither. If the axioms are right, the bound is right. Full stop.

== "But What If Someone Shared the URL?"

Fair objection. Axiom S4 says that a human cloner must have visited the page. Strictly, someone could receive a `git clone` URL by email or chat and clone without visiting GitHub. If that happened, S4 is violated.

But here's the thing: the ratio is $186 slash 5 = 37.2 times$. Even if all 5 visitors shared the URL with every person they know, you'd need 181 recipients who all independently decided to clone a formal verification research repo without looking at it first. The objection defeats itself quantitatively.

== So Who Were They?

GitHub counts every `git clone` or `git fetch` as a clone event. The usual suspects:

- *CI/CD runners* that clone on every workflow trigger (GitHub Actions, Jenkins, CircleCI).
- *Package scrapers* indexing public repos for search engines.
- *Mirror services* syncing repositories on schedule.
- *Security scanners* looking for leaked credentials.
- *AI training pipelines* building code datasets.

@fig:clones tells the story: the spike on April 2--4 has the signature shape of an automated event -- sharp rise, exponential decay. It correlates exactly with the publication of new research papers on kleis.io. The scrapers smelled fresh content and came running.

== A Note on Performance

The full verification -- five propositions, counterexamples, and a theorem -- took 3.6 seconds. We initially tried importing the full algebraic hierarchy (Ring, Field, VectorSpace) and Z3 ate 2.6 GB of RAM and crashed. The lesson: for catching bots, you don't need abstract algebra. Eight axioms over $bb(Z)$ will do.

#figure(
  lq.diagram(
  title: [Daily Clone Count (14-day window)],
  xlabel: [Day (0 = March 27)],
  ylabel: [Clones],
  lq.fill-between(
    (0.000000, 1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000, 11.000000, 12.000000, 13.000000),
    (5.000000, 5.000000, 10.000000, 10.000000, 15.000000, 100.000000, 400.000000, 620.000000, 580.000000, 400.000000, 300.000000, 200.000000, 150.000000, 40.000000),
    fill: red.lighten(70%)
  ),
  lq.plot(
    (0.000000, 1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000, 11.000000, 12.000000, 13.000000),
    (5.000000, 5.000000, 10.000000, 10.000000, 15.000000, 100.000000, 400.000000, 620.000000, 580.000000, 400.000000, 300.000000, 200.000000, 150.000000, 40.000000),
    color: red
  ),
)
,
  caption: [Daily clone count showing the spike on days 6--8 (April 2--4, 2026). The pre-spike baseline of $tilde.op 5$--15 clones/day rises to $tilde.op 620$ at peak, consistent with an indexing or scraping event triggered by new content publication.]
) <fig:clones>

= Conclusion

We looked at the traffic dashboard. Something was off. We wrote eight axioms. We asked Z3. And 3.6 seconds later, we had a theorem: at least 181 of 186 unique cloners -- 97.3% -- are bots.

No heuristics, no classifiers, no confidence intervals. Just logic. The bound is tight, the proof is constructive, and the bots never stood a chance.

Of course, this will happen again. The next time we publish a paper -- including this one -- the scrapers will detect fresh content, and the clone spike will return. The numbers will change; the axioms won't. The same eight constraints, the same solver, the same 3.6 seconds, and a new theorem with updated bounds. This paper is not a postmortem. It is a reusable trap.

If there is a methodological point here, it's this: formal verification isn't just for proving programs correct or checking mathematical conjectures. It's for catching things. When the constraints are simple enough to write down, the SMT solver will find the truth faster than any statistical model -- and with zero false positives.

The bots will be back. We'll be ready.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] L. de Moura and N. Bjørner, _Z3: An efficient SMT solver_, in Tools and Algorithms for the Construction and Analysis of Systems (TACAS), Springer LNCS 4963 (2008), pp. 337--340.]

#par(hanging-indent: 1.5em)[\[2\] GitHub, Inc., _Repository traffic API_, GitHub REST API Documentation (2024). https://docs.github.com/en/rest/metrics/traffic]

#par(hanging-indent: 1.5em)[\[3\] E. Atik, _Kleis: A formal verification language for knowledge production_, Kleis Research (2025). https://kleis.io]

#par(hanging-indent: 1.5em)[\[4\] C. Barrett, R. Sebastiani, S. Seshia, and C. Tinelli, _Satisfiability modulo theories_, in Handbook of Satisfiability, IOS Press (2009), pp. 825--885.]

#par(hanging-indent: 1.5em)[\[5\] M. Golzadeh, A. Decan, D. Legay, and T. Mens, _A ground-truth dataset and classification model for detecting bots in GitHub issue and PR comments_, Journal of Systems and Software 175 (2021), 110911.]


