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
  #text(size: 17pt, weight: "bold")[The Scribe is the Skolem: Formal Philology as Model Construction]
  
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
      #text(size: 10pt)[We present a framework in which a hieroglyphic text is a formal model, a grammar is a set of axioms, and the act of translation is the recovery of a Skolem witness --- the specific semantic assignment the scribe selected from an ambiguous constraint space. We formalize the non-verbal fragment of Middle Egyptian grammar (Lessons 1--8 of Allen's textbook, approximately one quarter of the full language) comprising 125 axioms across three structures (Hieroglyphic Writing, Nominal Grammar, and Prepositions), covering the writing system, noun morphology, pronouns, demonstratives, adjectives, non-verbal sentences, and the preposition system. The verbal system --- verb classes, tense-aspect-mood morphology, subordinate clauses, and narrative structures --- remains unformalized. We verify these axioms against 32 encoded sentences from the Tale of Sinuhe and other Middle Kingdom literary texts (c. 2000--1700 BCE), yielding machine-checked verdicts that are philologically interpretable. We demonstrate Z3-backed disambiguation on representative ambiguities: the same surface form *nb* yields opposite Skolem witnesses in different constraint contexts (adjective after a noun, noun after a pronoun), and the solver reconstructs missing words in damaged texts from surviving grammatical constraints. The results parallel our previous work on tonal harmony: where a musical score is a model of harmonic axioms with the composer as Skolem selector, a hieroglyphic text is a model of grammatical axioms with the scribe as Skolem selector. The critical insight is that Middle Egyptian is a maximally underspecified language --- no vowels are written, no word boundaries exist, conjunction is implicit, there is no copula, no articles, and no comparative morphology --- making every text a constraint satisfaction problem. Disambiguation is not scoring or ranking; it is constraint collapse, where invalid readings are unsatisfiable rather than unlikely. The type system becomes a philological engine: inferring the type of an expression is equivalent to translating it. This is structurally opposite to what a large language model does: an LLM approximates statistical regularity from a corpus, while the grammar axioms enforce structural necessity from first principles. The framework is implemented in the Kleis verification language using the same axiom--model--witness architecture previously applied to music theory and set-theoretic independence.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* formal philology, Middle Egyptian, hieroglyphs, Skolem witness, type inference, translation, constraint satisfaction, grammar formalization, Tale of Sinuhe]

#v(1em)


= Introduction

What does a scribe do? One answer is: a scribe constructs a *model* of a grammar.

A grammar, at its core, is a collection of constraints --- rules about how signs combine, how morphology marks gender and number, how word order determines meaning. These constraints are axioms. A text that satisfies all of them is a model: a specific structure in which every grammatical rule holds. A text that appears to violate some axioms while satisfying others presents an *ambiguity* --- multiple readings are consistent with the surface, and the scribe has selected one particular interpretation.

This paper applies the axiom--model--Skolem framework developed in our previous work on music theory to the domain of ancient Egyptian philology. In our music paper, we showed that a score is a model of tonal harmony axioms, and the composer is a Skolem selector who chooses a specific witness from the satisfiability space. Here, we show that a hieroglyphic text is a model of grammatical axioms, and the scribe (and by extension, the translator) is a Skolem selector who recovers the intended reading from an underspecified surface.

Middle Egyptian is an ideal test case because it is *maximally underspecified*:

- No vowels are written. The hieroglyphic script records only consonantal skeletons.

- No word boundaries exist. Signs flow continuously with no spaces or punctuation.

- No copula. The verb 'to be' does not exist; nominal sentences are bare juxtaposition.

- No articles. There is no 'the' or 'a' in standard Middle Egyptian.

- No comparative morphology. 'Good,' 'better,' and 'best' are the same word.

- Conjunction is implicit. 'Bread and beer' is written as 'bread beer.'

Every one of these missing features is a source of ambiguity that the reader must resolve. In formal terms, each ambiguity is an existential quantifier: 'there exists a reading of this sign sequence such that the grammar is satisfied.' The correct reading is the Skolem witness. The scribe selected it when writing; the translator recovers it when reading.

We formalize 125 grammatical axioms from James P. Allen's *Middle Egyptian: An Introduction to the Language and Culture of Hieroglyphs* (3rd edition, Cambridge 2014), covering the non-verbal grammar of the language through Lesson 8. We encode 32 sentences from Allen's Exercise 8 --- drawn primarily from the Tale of Sinuhe, the masterwork of Middle Egyptian literature --- and verify the axioms against this corpus. The entire pipeline runs on the Kleis verification platform, the same substrate used for our music theory and set-theoretic independence work.

= Hieroglyphic Texts as Formal Objects

A Middle Egyptian text in our framework is encoded as a sequence of morphologically tagged words. Each word carries:

- *Form*: the consonantal skeleton in Manuel de Codage transliteration (e.g., `nfr`, `HqA`, `mAat`).

- *Gloss*: the English translation.

- *Word class*: noun, adjective, preposition, pronoun, demonstrative, verb, adverb, or particle.

- *Gender*: masculine or feminine (axiom 13: the gender binary is strict --- no neuter).

- *Number*: singular, plural, or dual.

This representation is the linguistic analogue of the musical AST in our previous work. Where a score is built from pitches, durations, and voice assignments, a text is built from consonantal forms, glosses, and morphological tags. The invariant object is the morphologically tagged sentence; the surface hieroglyphs, the transliteration, and the English translation are projections.

Figure @fig:hieroglyph-words shows six key words from the corpus rendered in hieroglyphs using Gardiner sign SVGs from the Kleis template library --- the same signs available in the Equation Editor's Egyptian palette. These are not static images: they are rendered directly by the Typst typesetter from the same SVG assets that the Kleis template system uses, demonstrating the rendering pipeline Gardiner code $arrow$ `.kleist` template $arrow$ SVG $arrow$ Typst $arrow$ PDF.

The encoding captures exactly the information needed to verify grammatical axioms: gender agreement (axiom 53), number agreement, position constraints (axiom 54: adjectives follow nouns), and structural patterns (axiom 70: no copula in non-verbal sentences). Information not needed for non-verbal grammar verification --- phonological detail, paleographic variation, determinative classification --- is abstracted away.

#figure(
  [
#set text(size: 10pt)
#let hg(path) = box(width: 1.8em, height: 2.2em, align(center + horizon, image(path, height: 85%, fit: "contain")))
#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 2em,
  align(center)[
    #hg("/static/glyphs/egyptian/V30.svg")
    #hg("/static/glyphs/egyptian/A1.svg")
    \ #v(0.5em)
    #text(style: "italic")[nb] \ 'lord'
    \ #text(size: 8pt, fill: luma(100))[(Gardiner V30 + A1)]
  ],
  align(center)[
    #hg("/static/glyphs/egyptian/F35.svg")
    #hg("/static/glyphs/egyptian/D21.svg")
    \ #v(0.5em)
    #text(style: "italic")[nfr] \ 'good, beautiful'
    \ #text(size: 8pt, fill: luma(100))[(Gardiner F35 + D21)]
  ],
  align(center)[
    #hg("/static/glyphs/egyptian/S38.svg")
    #hg("/static/glyphs/egyptian/G1.svg")
    \ #v(0.5em)
    #text(style: "italic")[HqA] \ 'ruler'
    \ #text(size: 8pt, fill: luma(100))[(Gardiner S38 + G1)]
  ],
)
#v(1em)
#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 2em,
  align(center)[
    #hg("/static/glyphs/egyptian/R8.svg")
    #hg("/static/glyphs/egyptian/A40.svg")
    \ #v(0.5em)
    #text(style: "italic")[nTr] \ 'god'
    \ #text(size: 8pt, fill: luma(100))[(Gardiner R8 + A40)]
  ],
  align(center)[
    #hg("/static/glyphs/egyptian/G17.svg")
    #hg("/static/glyphs/egyptian/N35.svg")
    \ #v(0.5em)
    #text(style: "italic")[mw] \ 'water'
    \ #text(size: 8pt, fill: luma(100))[(Gardiner G17 + N35)]
  ],
  align(center)[
    #hg("/static/glyphs/egyptian/N35.svg")
    #hg("/static/glyphs/egyptian/V30.svg")
    #hg("/static/glyphs/egyptian/Z2.svg")
    \ #v(0.5em)
    #text(style: "italic")[nb] \ 'every, all'
    \ #text(size: 8pt, fill: luma(100))[(Gardiner N35 + V30 + Z2)]
  ],
)
],
  caption: [Key words from the corpus rendered in hieroglyphs (Gardiner sign SVGs from the Kleis template library). Each word shows the hieroglyphic writing, Manuel de Codage transliteration, and English gloss. These are the same signs available in the Kleis Equation Editor's Egyptian palette.]
) <fig:hieroglyph-words>

== The Tale of Sinuhe as Primary Specimen

The Tale of Sinuhe is to Middle Egyptian philology what the Moonlight Sonata is to tonal harmony analysis: a canonical specimen that exercises a wide range of the theory's axioms.

Composed during the 12th Dynasty (c. 1875 BCE), the Tale survives in multiple manuscript traditions:

- *Berlin Papyrus 3022* (Sin. B): the principal source, 311 lines in Middle Hieratic.

- *Ramesseum Fragments* (Sin. R): parallel text fragments.

- *Ashmolean Ostracon* (Sin. AO): a 19th Dynasty student copy, 130 lines.

Our corpus of 32 sentences includes 23 from Sinuhe (17 from B, 3 from R, 3 from AO) and 9 from other Middle Kingdom texts: the Eloquent Peasant, Pyramid Texts, Admonitions of Ipuwer, Urkunden IV, the Teaching of Amenemhat, the Teaching for King Merikare, the Satire of the Trades (Khety), and the Coffin Texts.

The transliterations are verified against Roland Koch's critical edition (*Die Erzählung des Sinuhe*, Bibliotheca Aegyptiaca 17, Brussels 1990).

Figure @fig:hieroglyph-sentence shows sentence 32 from the Coffin Texts --- *ink nTr aA* ('I am the great god') --- rendered with four annotation layers: hieroglyphic signs, Manuel de Codage transliteration, morphological tags (word class, gender, number), and English translation. This interlinear format makes the correspondence between the formal encoding and the visual hieroglyphs explicit: the three axioms verified against this sentence (53, 54, 77) are annotated directly beneath the signs.

#figure(
  [
#align(center)[
  #block(stroke: 0.5pt + luma(180), inset: 1.5em, radius: 4pt)[
    #text(size: 14pt, weight: "bold")[Sentence 32: Coffin Texts II, 27 1e S1C]
    #v(1em)
    #let S = 2.8em
    #let g(path) = box(width: S, height: S, align(center + horizon, image(path, width: S, height: S, fit: "contain")))
    #let gh(path) = box(width: S, height: S / 2 - 1pt, align(center + horizon, image(path, width: S, height: S / 2 - 1pt, fit: "contain")))
    #grid(
      columns: (auto, 1.5em, auto, 1.5em, auto),
      gutter: 0pt,
      align: center + bottom,
      [
        #grid(columns: 2, gutter: 1pt, align: center + horizon,
          g("/static/glyphs/egyptian/M17.svg"),
          stack(dir: ttb, spacing: 2pt,
            gh("/static/glyphs/egyptian/N35.svg"),
            gh("/static/glyphs/egyptian/V31.svg"),
          ),
        )
      ],
      [],
      [
        #grid(columns: 2, gutter: 1pt, align: center + horizon,
          g("/static/glyphs/egyptian/R8.svg"),
          g("/static/glyphs/egyptian/A40.svg"),
        )
      ],
      [],
      [
        #grid(columns: 2, gutter: 1pt, align: center + horizon,
          g("/static/glyphs/egyptian/G1.svg"),
          g("/static/glyphs/egyptian/D36.svg"),
        )
      ],
    )
    #v(0.3em)
    #grid(
      columns: (auto, 2em, auto, 2em, auto),
      gutter: 0pt,
      align: center,
      [#text(style: "italic")[ink]],
      [],
      [#text(style: "italic")[nTr]],
      [],
      [#text(style: "italic")[aA]],
    )
    #v(0.3em)
    #grid(
      columns: (auto, 2em, auto, 2em, auto),
      gutter: 0pt,
      align: center,
      [#text(size: 9pt)[PRON.1SG]],
      [],
      [#text(size: 9pt)[N.MASC.SG]],
      [],
      [#text(size: 9pt)[ADJ.MASC.SG]],
    )
    #v(0.3em)
    #text(size: 11pt)[#h(1fr) 'I am the great god' #h(1fr)]
    #v(0.5em)
    #text(size: 9pt, fill: luma(100))[
      Axiom 53 (Agreement): gender(nTr) = gender(aA) = Masculine ✓ #h(1em)
      Axiom 54 (Position): aA follows nTr ✓ #h(1em)
      Axiom 77 (Nominal A B): ink = pronoun, nTr = noun ✓
    ]
  ]
]
],
  caption: [Sentence 32 (Coffin Texts II, 27 1e S1C) rendered in hieroglyphs with interlinear annotation. This nominal sentence _ink nTr aA_ ('I am the great god') is a Skolem witness for axioms 53 (adjective agreement), 54 (adjective position), and 77 (nominal A B pattern). The hieroglyphic writing, transliteration, word class, and English gloss form four projection layers of a single formal object --- the morphologically tagged sentence in the Kleis AST.]
) <fig:hieroglyph-sentence>

= A Grammar as Axioms

Our Middle Egyptian grammar comprises 125 axioms organized in three structures:

*Structure 1: HieroglyphicWriting* (axioms 1--12). The writing system: sign polymorphism (the same sign can be ideogram, phonogram, or determinative), phonetic complements, determinative position, quadrat grouping, and reading direction. These axioms formalize the orthographic level --- the constraints on how signs combine to form words.

*Structure 2: MiddleEgyptianNominalGrammar* (axioms 13--100). The non-verbal grammar: noun morphology (gender, number, genitive constructions), pronouns (suffix, dependent, independent paradigms), demonstratives (four series, agreement rules), adjectives (agreement, position, nisbe formation), and non-verbal sentences (adjectival, nominal, and sentence-of-adherence patterns). This is the core of the grammar --- everything needed to parse and verify sentences without verbs.

*Structure 3: MiddleEgyptianPrepositions* (axioms 101--125). The preposition system: 17 primary prepositions, compound formation, prepositional nisbes, reverse nisbes, and adverbs. This structure captures the rich polysemy of Egyptian prepositions --- the preposition `m` alone has seven distinct semantic roles (locative, temporal, stative, material, ablative, essive, instrumental).

The axioms are universally quantified constraints over typed variables. For example:

#align(center)[
  _Axiom 53 (Adjective Agreement):_
  $forall a : "Adjective" . forall n : "Noun" . "modifies"(a, n) arrow.r "gender"(a) = "gender"(n)$
]

#align(center)[
  _Axiom 70 (No Copula):_
  $forall s : "Sentence" . "is_nominal"(s) arrow.r not "has_copula"(s)$
]

#align(center)[
  _Axiom 25 (Conjunction by Juxtaposition):_
  $forall n_1, n_2 : "Noun" . "adjacent"(n_1, n_2) and not "apposition"(n_1, n_2) arrow.r "conjoined"(n_1, n_2)$
]

These are not statistical regularities learned from a corpus. They are structural necessities derived from the grammar itself. A text that violates axiom 53 is not 'unlikely' --- it is *ungrammatical*.

= Verification Results

We run 13 axiom checks against the 32-sentence corpus. The results, shown in Table 1, confirm that the encoded sentences satisfy the grammatical constraints.

#figure(
  table(
    columns: 3,
    [*Axiom*], [*Result*], [*Sentences tested*],
    [13. Gender Binary], [SAT], [s04, s05, s07, s08, s14, s18],
    [14. Feminine -t Ending], [SAT], [rnpt, wart, pxrt, mAat, pDt, irt, prt, wmt, st],
    [16. Place Names Feminine], [SAT], [rtnw, qdnw, mktj],
    [23. No Articles], [SAT], [all 32 sentences],
    [43. Demonstrative Gender], [SAT], [s03, s17],
    [46. Demonstrative Position], [SAT], [s03, s17],
    [53. Adjective Agreement], [SAT], [s16, s21, s26, s32],
    [54. Adjective Position], [SAT], [s09, s13, s16, s18, s21, s26, s32],
    [55/63. nb Disambiguation], [SAT], [s09, s20, s25, s27],
    [67. Comparative with r], [SAT], [s14],
    [70/77. Nominal Sentences], [SAT], [s27, s32],
    [82. pw as Copula], [SAT], [s18, s25],
    [40. Possessive Suffix], [SAT], [s01, s11, s22, s24, s25],
  ),
  caption: [Verification of 13 grammar axioms against the Sinuhe corpus. All axioms are satisfied (SAT).]
) <tab:results>

Unlike the music analysis, where violations revealed compositional freedom, here all axioms are satisfied. This is expected: the music theory axioms were deliberately strict probes, while the grammar axioms formalize *actual* rules of Middle Egyptian. A grammatical text *should* satisfy the grammar. The interesting work is not in finding violations but in resolving *ambiguities* --- the same surface form admitting multiple readings.

= The Scribe as Skolem Selector

The parallel between music and philology is exact:

#figure(
  table(
    columns: 3,
    [*Component*], [*Music*], [*Egyptian*],
    [Theory], [TonalHarmony (7 axioms)], [MiddleEgyptianGrammar (125 axioms)],
    [Model], [Score (typed AST)], [Text (morphologically tagged words)],
    [Skolem Witness], [The specific composition], [The specific reading/translation],
    [Selector], [Composer (Beethoven)], [Scribe / Translator],
    [Surface], [Sounding notes], [Written hieroglyphs],
    [Skeleton], [Harmonic structure], [Grammatical structure],
    [Ambiguity], [Non-chord tones, harmonic rhythm], [nb ('lord' vs. 'every'), phrase/sentence],
  ),
  caption: [Structural parallel between music theory verification and philological verification.]
) <tab:parallel>

The disambiguation axioms are the most revealing. Consider the word *nb*, illustrated in Figure @fig:nb-disambiguation:

- After a noun: *pxrt nb* = 'every remedy' (adjective, axiom 55)
- Standalone: *ink nb* = 'I am the lord' (noun, axiom 63)
- With suffix: *nb.f* = 'his lord' (noun with possessive, axiom 40)

The same consonantal skeleton --- the same hieroglyphic signs --- admits two distinct readings. Position alone determines meaning. The reader performs constraint satisfaction: given the surrounding words and their morphological tags, which reading of *nb* is consistent with the grammar? The answer is the Skolem witness.

The same pattern operates at every level:

- *Sign function*: Is this sign an ideogram, phonogram, or determinative? (axiom 1)
- *Noun phrase structure*: Is this apposition, conjunction, or genitive? (axioms 25, 26, 30)
- *Sentence type*: Is this a phrase or a sentence? (axiom 98)
- *Tense*: Past, present, or gnomic? (axiom 95: no inherent tense)

Each ambiguity is a branch point in the constraint space. The scribe navigated one path; the translator must recover it. The grammar axioms prune the space; the remaining choices are where philological judgment lives.

A critical consequence: disambiguation does not always yield a unique witness. When the axioms reduce the space to a single reading --- as with *nb* after a noun --- translation is *determined*. When multiple readings survive --- as when *aA* ('great') could be adjective or appositive noun --- translation remains *underdetermined*, and the surviving witnesses form a set of equally valid Skolem candidates. The honest output of the system is not always 'the translation is X' but sometimes 'the axioms permit readings X and Y; further constraints (semantic, contextual, or pragmatic) are needed to select between them.' This is not a failure of the framework but a precise characterization of where the formal system ends and philological judgment begins.

#figure(
  [
#align(center)[
  #grid(
    columns: (1fr, auto, 1fr),
    gutter: 1em,
    {
      let hg(path) = box(width: 1.8em, height: 2.2em, align(center + horizon, image(path, height: 85%, fit: "contain")))
      block(stroke: 0.5pt + luma(180), inset: 1em, radius: 4pt)[
        #align(center)[
          #text(weight: "bold")[Reading 1: Noun]
          #v(0.5em)
          #hg("/static/glyphs/egyptian/V30.svg")
          #hg("/static/glyphs/egyptian/A1.svg")
          #v(0.3em)
          \ #text(style: "italic")[nb] = 'lord'
          \ #text(size: 9pt)[Standalone or with suffix:]
          \ #text(size: 9pt, style: "italic")[ink nb] = 'I am the lord'
          \ #text(size: 9pt, style: "italic")[nb.f] = 'his lord'
          \ #text(size: 8pt, fill: luma(100))[(Axiom 63)]
        ]
      ]
    },
    align(center + horizon)[
      #text(size: 18pt, weight: "bold")[vs.]
    ],
    {
      let hg(path) = box(width: 1.8em, height: 2.2em, align(center + horizon, image(path, height: 85%, fit: "contain")))
      block(stroke: 0.5pt + luma(180), inset: 1em, radius: 4pt)[
        #align(center)[
          #text(weight: "bold")[Reading 2: Adjective]
          #v(0.5em)
          #hg("/static/glyphs/egyptian/N35.svg")
          #hg("/static/glyphs/egyptian/V30.svg")
          #v(0.3em)
          \ #text(style: "italic")[nb] = 'every, all'
          \ #text(size: 9pt)[After noun (agreement):]
          \ #text(size: 9pt, style: "italic")[pxrt nb] = 'every remedy'
          \ #text(size: 9pt, style: "italic")[rn nb] = 'every name'
          \ #text(size: 8pt, fill: luma(100))[(Axiom 55)]
        ]
      ]
    },
  )
]
],
  caption: [The word _nb_ demonstrates the Skolem choice in hieroglyphic interpretation. The same consonantal skeleton admits two readings depending on syntactic context: as a noun 'lord' (left) or as an adjective 'every' (right). Position determines meaning --- the reader performs constraint satisfaction over grammar axioms 55 and 63 to select the correct Skolem witness.]
) <fig:nb-disambiguation>

= Type Inference as Translation

The deepest insight of this framework is that *type inference is translation*.

In the Kleis type system (Hindley-Milner), inferring the type of an expression means determining the most general assignment of types to subexpressions such that all constraints are satisfied. The constraints come from the structure definitions --- which operations are defined, what types they accept and return, and what axioms they satisfy.

Now consider translating a Middle Egyptian sentence. The 'expression' is the sequence of hieroglyphs. The 'type constraints' are the grammar axioms. 'Inferring the type' means determining:

- The word class of each sign group (noun, verb, adjective, preposition)
- The gender and number of each noun
- The syntactic function of each word (subject, predicate, modifier, complement)
- The semantic role of each preposition

This is precisely a constraint-satisfaction problem over typed variables --- the same computational problem that Hindley-Milner solves for programming languages. The 'principal type' (most general type consistent with all constraints) is the 'best translation' (reading consistent with all grammar rules while committing to the fewest assumptions).

This is structurally opposite to what a large language model does with a language:

- An LLM learns $P("translation" | "hieroglyphs", "corpus")$ --- a statistical distribution conditioned on training data. It approximates regularity.

- The type system computes $"mgu"("constraints"("grammar", "text"))$ --- the most general unifier of structural constraints. It enforces necessity.

The LLM can produce plausible translations of texts it has never seen, but it cannot *explain* why a reading is correct or prove that no other reading is consistent. The type system can --- provably correct relative to the axiom set. The grammar axioms are not learned from data; they are derived from the structure of the language itself. Every type inference step corresponds to a grammatical rule, and the chain of inference is the *proof* that the translation is correct.

Moreover, the template system already constrains what can be expressed. The Equation Editor for hieroglyphs uses Gardiner sign templates that enforce valid sign combinations at the visual level --- ungrammatical quadrat compositions are not constructible. This is the linguistic analogue of a strongly typed programming language: the type system prevents ill-formed expressions before they can be evaluated.

= Solver Trace: End-to-End Disambiguation

We now present the actual output of the Kleis--Z3 pipeline on representative disambiguation problems. The traces below are not illustrative pseudo-code; they are the verbatim results of running the constraint solver against the grammar axioms.

*Trace 1: pxrt nb --- unique witness (adjective).*

#figure(
  table(
    columns: 4,
    align: (left, left, left, left),
    [*Step*], [*Action*], [*Constraint*], [*Result*],
    [1], [Input], [`pxrt nb` (positions 100, 101)], [],
    [2], [Domain], [`word_class(101) $in$ {noun, adj}`], [],
    [3], [Context], [`word_class(100) = noun`], [],
    [4], [Axiom 55], [`noun $arrow.r$ next = adj`], [`word_class(101) = adj`],
    [5], [Axiom 53], [`adj $arrow.r$ gender agreement`], [`word_gender(101) = fem`],
    [6], [Test: nb = noun?], [Z3 query], [*UNSAT* --- eliminated],
    [7], [Test: nb = adj?], [Z3 query], [*SAT* --- unique witness],
  ),
  caption: [Solver trace for _pxrt nb_. Axiom 55 eliminates the noun reading. Admissible set: \{adjective\}.]
) <tab:trace-pxrt>

*Trace 2: ink nb --- contextual inversion (noun).*

#figure(
  table(
    columns: 4,
    align: (left, left, left, left),
    [*Step*], [*Action*], [*Constraint*], [*Result*],
    [1], [Input], [`ink nb` (positions 200, 201)], [],
    [2], [Domain], [`word_class(201) $in$ {noun, adj}`], [],
    [3], [Context], [`word_class(200) = pronoun`], [],
    [4], [Axiom 77], [`pronoun + noun $arrow.r$ nominal A B`], [`word_class(201) = noun`],
    [5], [Test: nb = adj?], [Z3 query], [*UNSAT* --- eliminated],
    [6], [Test: nb = noun?], [Z3 query], [*SAT* --- unique witness],
  ),
  caption: [Solver trace for _ink nb_. Axiom 77 eliminates the adjective reading. Same surface form, opposite witness.]
) <tab:trace-ink>

The juxtaposition is the central result: `word_class(101) $ne$ word_class(201)`. The same hieroglyphic signs, the same consonantal skeleton, but the constraint context flips the solution space. Meaning is not lexical --- it is constraint-induced.

*Trace 3: ink \_ aA --- lacuna reconstruction.*

#figure(
  table(
    columns: 4,
    align: (left, left, left, left),
    [*Step*], [*Action*], [*Constraint*], [*Result*],
    [1], [Input], [`ink _ aA` (positions 300, 301, 302)], [],
    [2], [Domain], [`word_class(301) $in$ {noun, adj}`], [],
    [3], [Axiom 77], [`pronoun $arrow.r$ next = noun`], [`word_class(301) = noun`],
    [4], [Axiom 54], [`adj follows noun`], [consistent],
    [5], [Axiom 53], [`adj $arrow.r$ gender agreement`], [`word_gender(301) = masc` (if aA = adj)],
    [6], [Test: lacuna = adj?], [Z3 query], [*UNSAT* --- eliminated],
    [7], [Test: lacuna = noun?], [Z3 query], [*SAT*],
    [8], [Result], [Missing word: masculine noun], [Candidates: nTr, nsw, HqA],
  ),
  caption: [Lacuna reconstruction. Z3 determines the missing word's grammatical class and gender from surviving context.]
) <tab:trace-lacuna>

These three traces establish a complete behavior spectrum:

- *Unique solution*: the axioms collapse the candidate set to a single witness (*pxrt nb* $arrow.r$ adjective).
- *Contextual inversion*: the same form yields opposite witnesses in different contexts (*pxrt nb* vs. *ink nb*).
- *Reconstruction*: the solver fills a gap from surrounding constraints (*ink \_ aA* $arrow.r$ masculine noun).
- *Honest limit*: when axioms do not fully resolve --- as with `word_class(302)` for *aA*, which could be noun or adjective --- the system reports the surviving witness set, not a forced choice.

This is executable semantics of grammar. Disambiguation is not scoring or ranking; it is constraint collapse. Invalid readings are not 'less likely' --- they are *unsatisfiable*. The epistemology is categorically different from statistical models: a candidate is either admitted or eliminated, with no middle ground.

= The Universal Pattern Revisited

With the addition of philology, the axiom--model--witness pattern now spans four domains:

#figure(
  table(
    columns: 4,
    [*Domain*], [*Theory (Axioms)*], [*Model*], [*Skolem Witness*],
    [Music], [TonalHarmony], [Score], [The composition],
    [Philology], [MiddleEgyptianGrammar], [Text], [The translation],
    [Chess], [Movement rules], [Position], [The winning move],
    [Set theory], [ZFC], [Universe of sets], [Gödel / Cohen models],
  ),
  caption: [Four domains, one architecture. All implemented on the Kleis verification platform.]
) <tab:universal>

The philological case is especially instructive because it bridges the humanities and formal methods. Egyptology has traditionally been practiced through close reading, philological intuition, and scholarly debate. This framework does not replace that practice --- it *formalizes the structure* that underlies it. When an Egyptologist debates whether *nb* in a given context means 'lord' or 'every,' they are performing constraint satisfaction over grammar axioms. When they reconstruct a damaged text, they are searching for a Skolem witness consistent with the surviving constraints.

The Afroasiatic typological connection reinforces the universality. Middle Egyptian shares deep structural features with Classical Arabic: copulaless sentences, suffix pronouns, nisbe adjectives, genitive constructions. The same grammar axioms, with parametric variation, could formalize both languages. The axiom--model--witness pattern is not specific to one language family; it captures the *structure of grammar* itself.

= Discussion

The 125 axioms formalized here cover only the non-verbal grammar of Middle Egyptian --- Lessons 1 through 8 of Allen's textbook, approximately 140 pages of a 600-page work. The remaining 460 pages cover the verbal system: verb classes, tense-aspect-mood morphology, subordinate clauses, and narrative structures. Extending the formalization to the verbal system is the natural next phase.

Several refinements are immediate:

*Z3-backed disambiguation.* We implement Z3-backed disambiguation and demonstrate it on representative ambiguities from the corpus. Grammar axioms are encoded as constraints over uninterpreted functions (`word_class`, `word_gender`), and Z3 determines the admissible parse set. The solver trace for *pxrt nb* ('every remedy') shows axiom 55 eliminating the noun reading (UNSAT), leaving a unique witness: nb = adjective. The reverse context *ink nb* ('I am the lord') shows axiom 77 eliminating the adjective reading, yielding the opposite witness: nb = noun. The same surface form, two constraint contexts, two different Skolem witnesses --- computed, not guessed. The solver trace is not illustrative; it is the actual output of the Kleis--Z3 pipeline.

*Damaged text reconstruction.* We demonstrate formal lacuna reconstruction on the sentence *ink \_ aA* ('I am the great \_'), where the middle word is treated as unknown. Z3 derives that the missing word must be a noun (axioms 54 and 77 independently force this), and if *aA* is an adjective, the missing word is masculine (axiom 53). The admissible candidate set --- *nTr* (god), *nsw* (king), *HqA* (ruler) --- is determined by the grammar, not by statistical guess. This is formal reconstruction: finding a Skolem witness for the existential claim 'there exists a word in the lacuna such that the sentence is grammatical.'

*Comparative Afroasiatic grammar.* The structural parallels between Middle Egyptian and Classical Arabic suggest that the same axiom architecture, with parametric differences, could formalize both languages. The shared features --- copulaless sentences, suffix pronouns, construct-state genitives --- are precisely the axioms that would be *invariant* across the two formalizations. The differences --- Arabic's three cases, Egyptian's lack of case marking --- would appear as axioms present in one theory but absent from the other.

*Hieroglyph rendering.* As demonstrated in Figures @fig:hieroglyph-words, @fig:hieroglyph-sentence, and @fig:nb-disambiguation, the Kleis template library already includes 226 Gardiner sign templates that render as inline SVG images via Typst. The rendering pipeline --- Gardiner code $arrow$ `.kleist` template $arrow$ SVG $arrow$ Typst $arrow$ PDF --- operates identically to the LilyPond pipeline used for sheet music in the music paper. The same signs are available in the Kleis Equation Editor's Egyptian palette for visual composition of hieroglyphic expressions.

Two limitations bound the current work. First, the 125 axioms cover only the non-verbal grammar --- Lessons 1 through 8 of a 26-lesson textbook. Any sentence involving verb forms, tense-aspect-mood, relative clauses, or subordination falls outside the current axiom set. The disambiguation and reconstruction results demonstrated here apply only within the formalized fragment; extending the claims to the full language requires axiomatizing the verbal system. Second, the test corpus of 32 sentences from Exercise 8 exercises many axioms but does not constitute a comprehensive test suite. A robust validation would require encoding hundreds of sentences from diverse text genres: literary (Sinuhe, Eloquent Peasant), historical (royal stelae, tomb biographies), religious (Pyramid Texts, Coffin Texts), and administrative (letters, legal documents).

= Conclusion

We have presented a framework in which a hieroglyphic text is a model, a grammar is a set of axioms, and the act of translation is the recovery of a Skolem witness. The framework is implemented in the Kleis verification language and demonstrated on 32 sentences from the Tale of Sinuhe and other Middle Kingdom texts, yielding machine-checked verdicts that are philologically interpretable.

The central claim extends our previous work on music: *the scribe is the Skolem*. The grammar axioms constrain the space of possible readings. The type system can verify any candidate translation. But the choice of which reading to instantiate --- which word class for *nb*, which genitive construction, which sentence pattern --- is the irreducibly human act. The scribe made this choice when composing the text four thousand years ago; the translator recovers it today.

This is the opposite of what a large language model does. An LLM approximates statistical patterns from a corpus. The grammar axioms enforce structural necessity from first principles. The LLM produces plausible translations; the type system produces translations that are *provably consistent with the axiom set* --- a weaker but formally precise guarantee. The two approaches are complementary but structurally distinct, and the distinction is precisely the one between sampling from a distribution and selecting a Skolem witness from a satisfiability space.

The axiom--model--witness pattern now spans music, philology, chess, and set theory, running on the same Kleis verification substrate. This universality is not coincidental. It reflects the structure of knowledge production itself: notation defines a vocabulary, axioms constrain it, verification checks consistency, and the specific instantiation --- the Skolem witness --- is where domain-specific insight lives. In mathematics, that is the proof strategy. In chess, the combinatorial insight. In music, the beauty. In philology, the translation.

The formal system accounts for everything except what makes the result meaningful. That is its strength, and its honest limitation.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[atik2026skolem\] Atik, E. (2026). The Beauty is in the Skolems: Formal Music Theory as Model Construction. arXiv preprint.]

#par(hanging-indent: 1.5em)[\[atik2026fibers\] Atik, E. (2026). Independence as Non-Invariance: Detecting Undecidability via Projection Fibers. arXiv preprint.]

#par(hanging-indent: 1.5em)[\[allen2014egyptian\] Allen, J. P. (2014). Middle Egyptian: An Introduction to the Language and Culture of Hieroglyphs, 3rd ed. Cambridge University Press.]

#par(hanging-indent: 1.5em)[\[koch1990sinuhe\] Koch, R. (1990). Die Erzählung des Sinuhe. Bibliotheca Aegyptiaca 17, Brussels.]

#par(hanging-indent: 1.5em)[\[gardiner1957grammar\] Gardiner, A. H. (1957). Egyptian Grammar: Being an Introduction to the Study of Hieroglyphs, 3rd ed. Griffith Institute, Oxford.]

#par(hanging-indent: 1.5em)[\[demoura2008z3\] de Moura, L. and Bjorner, N. (2008). Z3: An Efficient SMT Solver. TACAS 2008, LNCS 4963, pp. 337--340.]

#par(hanging-indent: 1.5em)[\[hindley1969\] Hindley, R. (1969). The Principal Type-Scheme of an Object in Combinatory Logic. Transactions of the AMS, 146, 29--60.]

#par(hanging-indent: 1.5em)[\[mauch2019typst\] Madje, L. and Haug, M. (2022). Typst: A new markup-based typesetting system for the sciences. https://typst.app]

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#align(center)[
  #text(size: 14pt, weight: "bold")[Appendix]
]
#v(1em)
= Axiom Summary (125 Axioms)

The full grammar is defined in `stdlib/theories/middle_egyptian_grammar.kleis`.

*Structure 1: HieroglyphicWriting (axioms 1--12)*

Axioms 1--4: Sign polymorphism, determinative position, phonetic complement rule, ideogram marker. Axioms 5--8: Reading direction, quadrat grouping, no word boundaries, complement disambiguation. Axioms 9--12: Triliteral semantic binding, spelling variants, consonants only, sign tucking.

*Structure 2: MiddleEgyptianNominalGrammar (axioms 13--100)*

Nouns (13--30): Gender binary, feminine -t ending, place names feminine, number morphology (plural -w/-wt, dual -wj/-tj), false plurals, no articles, proper names defined, conjunction by juxtaposition, direct and indirect genitives, honorific transposition, no 'have' verb, apposition.

Pronouns (31--42): Suffix paradigm (8 forms), suffix position (word-final), 1s optional writing, 1s speaker ideogram, sound changes, dependent not initial, neutral 3n pronoun, independent compositionality, pronoun syntax, possessive after endings, suffix gender independence, 1s determinative ambiguity.

Demonstratives (43--52): Gender agreement (p-/t-/n-), four series (-n/-f/-w/-A), standalone neutral, position rules, genitive integrity, demonstrative-article ambiguity, new possessive agreement, interrogatives uninflected, interrogative position, animate/inanimate distinction.

Adjectives (53--69): Agreement, position (after noun), nb disambiguation, nisbe feminine, feminine plural, nb first, demonstrative before adjectives, suffix before adjectives, genitive + adjective, adjectives are nouns, nb standalone, nfr Hr construction, apparent adjectives precede, no comparative form, comparative with r, superlative with genitive, prepositional adjective phrases.

Non-verbal sentences (70--100): No copula, predicate-subject order, predicate uninflected, position determines function, exclamatory -wj, secondary predicates only, dependent subject restriction, nominal A B pattern, pronoun-noun apposition, balanced sentences, adherence sentences, pw neutral, pw position, pw respects genitive, A pw B semantics, two nominal patterns, A is predicate, 3rd person uses pw, 1st/2nd person ambiguity, ptr initial, jn mj fusion, interrogative-first, 1st person avoids adjectival, nominal vs adjectival, no inherent tense, gnomic mode, contextual tense, phrase/sentence ambiguity, prosodic disambiguation, sign tucking feminine.

*Structure 3: MiddleEgyptianPrepositions (axioms 101--125)*

Axioms 101--108: Three prep forms, m polysemy (7 roles), n vs r goals, with split (hna vs m), social register, suffix on prepositions, jn/mj no suffix, mj workaround. Axioms 109--117: Compound prep composition, subjectless adjectival + n, nisbe derivation, nisbe agreement, nisbe suffix placement, Hrj possession, reverse nisbes, jmj-r lexicalization, prep modifier constraint. Axioms 118--125: Three primary adverbs, compositional interrogative adverbs, adverb from adjective, temporal nouns as adverbs, prep adverb formation, adverb position, no comparative adverb, phrasal cohesion.

= Corpus Excerpt: Key Sentences

The following shows the Kleis encoding for representative sentences from the corpus.

*Sentence 14 (Sin. B 82) --- Comparative with r:*
```
define s14 = Sent(14, "Sin. B 82",
    Cons(adj("wr", "abundant", Masculine, Singular),
    Cons(prep("n", "for"),
    Cons(pron(".f", "it", Masculine, Singular),
    Cons(noun("irp", "wine", Masculine, Singular),
    Cons(prep("r", "than"),
    Cons(noun("mw", "water", Masculine, Singular),
    Nil)))))))
```
'More abundant for it is wine than water' (description of the land of Iaa). Tests axiom 67: comparative via preposition *r*, not morphological form.

*Sentence 18 (Sin. R 55) --- pw as copula-substitute:*
```
define s18 = Sent(18, "Sin. R 55",
    Cons(noun("HqA", "ruler", Masculine, Singular),
    Cons(particle("pw", "copula"),
    Cons(prep("n", "of"),
    Cons(noun("rtnw", "Retjenu", Feminine, Singular),
    Cons(adj("Hrt", "upper", Feminine, Singular),
    Nil))))))
```
'He is a ruler of Upper Retjenu' (introduction of Amunenshi). Tests axiom 82 (pw neutral) and axiom 27 (indirect genitive with n).

*Sentence 32 (CT II, 27 1e S1C) --- Nominal A B:*
```
define s32 = Sent(32, "CT II, 27 1e S1C",
    Cons(pron("ink", "I", Masculine, Singular),
    Cons(noun("nTr", "god", Masculine, Singular),
    Cons(adj("aA", "great", Masculine, Singular),
    Nil))))
```
'I am the great god' (Coffin Texts declaration). Tests axioms 53 (agreement), 54 (position), and 77 (nominal A B pattern).
