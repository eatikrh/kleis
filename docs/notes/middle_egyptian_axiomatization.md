# Middle Egyptian Axiomatization Notes

Notes from reading James P. Allen, *Middle Egyptian: An Introduction to the
Language and Culture of Hieroglyphs*, 3rd ed. (Cambridge, 2014).

Goal: formalize the writing system and grammar of Middle Egyptian as Kleis
structures and axioms, then verify real inscriptions — the same methodology
used for the Moonlight Sonata tonal harmony analysis.

---

## Lesson 1: Language and Writing

### Sign Functions (core ADT)

Every hieroglyph can function in three ways depending on context:

1. **Ideogram** — represents the thing it depicts ("house" sign = "house")
2. **Phonogram** — represents a sound, not a thing ("emerge" sign spells the sound)
3. **Determinative** — silent classifier at end of word, indicates:
   - The preceding signs are phonograms (not ideograms)
   - The general semantic category of the word
   - Word boundaries (no spaces in hieroglyphic writing)

The same sign can serve all three functions. The "house" sign is an ideogram
("house"), a phonogram (sound *pr*), and a determinative (after words denoting
buildings). Function is determined by position and context — polymorphic type.

### Direction (Section 1.6)

- Signs face toward the **beginning** of the text
- Reading direction determined by which way signs face
- Four possible arrangements: horizontal L-to-R, horizontal R-to-L,
  vertical L-to-R, vertical R-to-L
- "Retrograde" (reversed) inscriptions exist, almost exclusively in
  religious texts

### Grouping / Quadrats (Section 1.7)

Three sign shapes:
1. **Tall signs** — stand alone (reeds, standing figures)
2. **Flat signs** — horizontal (mouth, water)
3. **Small signs** — compact (quail chick, bread loaf)

Grouping rules:
- Tall signs stand by themselves
- Other signs arranged into square or rectangular groups (quadrats)
- Two flat signs stack vertically
- A tall sign can be made smaller and grouped with a flat one
- Dissimilar shapes centered when grouped
- Groups read top-to-bottom, beginning-to-end

**"Direction and grouping are the only organizing methods used in
hieroglyphic writing."** No spaces, no punctuation.

### Five Scripts (Sections 1.9-1.11)

| Script        | Period          | Medium          | Direction    |
|---------------|-----------------|-----------------|-------------|
| Hieroglyphic  | 3200 BC – end   | Stone, formal   | Both        |
| Hieratic      | Nearly as old   | Papyrus, brush  | R-to-L      |
| Demotic       | 650 BC –        | Papyrus, cursive| R-to-L      |
| Coptic        | 1st century AD  | Greek letters+7 | L-to-R      |

Coptic preserves **vowels** that hieroglyphic writing omits — key to
reconstructing pronunciation.

### Decipherment (Section 1.12)

Champollion's method was constraint satisfaction:
1. Known constraint: Ptolemy cartouche in Greek → hieroglyphic must match
2. Partial assignment: identified *p*, *t*, *o*, *l*, *s*
3. Propagation: same signs in Kleopatra → extended assignment
4. Verification: tested against Ramesses via Coptic cognate (*rê* = sun)

This is SAT solving done by hand.

---

## Lesson 2: Uniliteral Signs

### The 23-Sign Alphabet (Section 2.3)

| Gardiner | Description          | Sound | Name          |
|----------|----------------------|-------|---------------|
| G1       | vulture              | ꜣ     | aleph         |
| M17      | reed-leaf            | i     |               |
| M17M17   | dual strokes         | j/y   | yod           |
| M18      | double reed-leaf     | y     |               |
| D36      | arm                  | ꜥ     | ayin          |
| G43      | quail-chick          | w     |               |
| D58      | foot                 | b     |               |
| Q3       | stool                | p     |               |
| I9       | horned viper         | f     |               |
| G17      | owl                  | m     |               |
| N35      | water                | n     |               |
| D21      | mouth                | r     |               |
| O4       | enclosure            | h     |               |
| V28      | rope                 | ḥ     | dotted h      |
| Aa1      | unknown object       | ḫ     | third h       |
| F32      | belly and udder      | ẖ     | fourth h      |
| S29      | bolt of cloth        | s     |               |
| N37      | hill                 | q     | dotted k      |
| V31      | basket with handle   | k     |               |
| W11      | jar-stand            | g     |               |
| X1       | bread                | t     |               |
| D46      | hand                 | d     |               |
| I10      | cobra                | ḏ     |               |

### Vocalization (Section 2.7)

Hieroglyphs write only consonants. Convention: insert *e* between consonants.
Three basic vowels reconstructed from Coptic: *a* (walk), *i* (bit), *u* (put).

Three distinct representations of the same word:
- `ConsonantalForm` — what hieroglyphs write (e.g. *nfr*)
- `ConventionalPronunciation` — Egyptological convention (e.g. "nefer")
- `CopticCognate` — attested vocalized form (e.g. *noufre*)

### Writing Conventions and Sound Changes (Section 2.8)

**Abbreviated spellings:** signs omitted for compact grouping. *rmṯ* "people"
drops *m* and *n*. Confirmed by Coptic descendants.

**Doubled consonants:** written once unless separated by a vowel.
*qbb* "become cool" written as *qb*.

**Weak consonants:** *ꜣ*, *j*, *y*, *w* disappear in middle/end of words.
Multiple spellings exist for the same word.

**Sound changes (conservative vs. modern):**
- *r* disappears at end of syllables
- *ꜣ* sometimes replaced by reed-leaf
- *t* (feminine ending) often dropped
- *ṯ* → *t*, *d̲* → *d*

These define spelling variant equivalence classes.

---

## Lesson 3: Multiliteral Signs

### Biliteral Signs (Section 3.1)

~100 signs representing two-consonant combinations. Organized as a matrix
indexed by first consonant (rows) × second consonant (columns).

Structural constraints:
- No biliterals with *f* as either consonant
- None with *h*, *ḥ*, *š*, or *g* as second consonant
- Some combinations have multiple signs (one-to-many)
- Some signs have multiple biliteral values (many-to-one / polymorphism)

### Phonetic Complements (Section 3.2) — KEY AXIOMS

**Core rule:** "A uniliteral sign following a biliteral sign is almost always
a phonetic complement and not an additional letter."

Formalization:
```
follows(uniliteral, biliteral) ∧ sound(uniliteral) = second_consonant(biliteral)
    → role(uniliteral) = Complement
```

Refinements:
- Complements mostly indicate the **second consonant**
- A few biliterals also have complement for **first consonant**
- Complement can precede or go between positions
- Read **with** the biliteral, not in addition: `pr + r` = *pr*, not *prr*
- For signs with multiple values, complement **disambiguates**:
  sign + *r* = *mr*, sign + *b* = *ꜣb*

Disambiguation by complement = type inference resolving polymorphism.

### Ideogram Marker (Section 3.3)

When a biliteral/triliteral is used as an ideogram:
- **No phonetic complement**
- Marked with a **stroke** (vertical line below)

```
has_stroke(sign) ∧ ¬has_complement(sign) → role(sign) = Ideogram
```

### Triliteral Signs (Section 3.4)

~80 signs representing three-consonant combinations. Unlike biliterals,
mostly limited to **one word and its relatives** — semantically bound.

Example: sandal-strap (*ꜥnḫ*) → "live," "life," "cause to live" (*sꜥnḫ*),
"the living" (*ꜥnḫw*). All radiate from one root meaning.

### Non-Standard Spelling (Section 3.6)

- Spelling was **not fixed** — scribes added, omitted, substituted freely
- Puns were common (Hathor written with falcon inside enclosure)
- Ptolemaic/Roman texts: extreme creativity, one text all crocodile signs
- Implication: axiom set must handle spelling variants (multiple valid
  decompositions). Z3 Sat mode: enumerate all satisfiable readings.

### Transcription Symbols (Section 3.7)

- `( )` parentheses — restored weak consonants (omitted in writing)
- `[ ]` square brackets — damaged/missing text restored by scholars
- `⸢ ⸣` half brackets — partially preserved signs
- `{ }` pointed brackets — scribal errors

These are confidence levels on readings — metadata on type assignments.

### Determinative Categories (from Exercise 3, 78 words)

| Category   | Determinative | Example Words                    |
|------------|---------------|----------------------------------|
| PersonMale | (man)         | son, companion                   |
| PersonFemale| (woman)      | daughter, widow                  |
| Building   | (house)       | interior, gate, tomb             |
| MotionVerb | (motion)      | pass, enter, stop, come          |
| ForceVerb  | (force)       | take away, strength, hack        |
| EffortVerb | (effort)      | build, wipe, plow                |
| Abstract   | (abstract)    | secret, stable, protection, new  |
| Negative   | (bad)         | perish, empty, narrow            |
| BodyPart   | (flesh)       | tongue, skin                     |
| Water      | (water)       | water, swim, fluid               |
| Solar      | (sun)         | brighten                         |
| Funerary   | (mummy)       | form                             |
| Temporal   | (time)        | eternity                         |
| Agriculture| (seed)        | seed                             |
| WoodObject | (wood)        | staff                            |
| Vessel     | (boat)        | ferry                            |
| Serpent    | (snake)       | snake                            |
| Weapon     | (arrow)       | arrow                            |
| Monument   | (stela)       | stela                            |
| HairRelated| (hair)        | hair, black                      |
| Emotion    | (emotion)     | fear                             |
| Speech     | (speak/think) | blessing, witness, bring to mind |

---

## Test Corpus

### Exercise 1 — Real Inscriptions with Translations

- "The sun-disk's rays are protection over you, their hands holding health
  and life" — shrine of Tutankhamun
- "I was his servant, his true confidant" — autobiographical inscription
- "You shall reveal to him your secrets" — tomb of Seti I
- "I have followed him by night and day to all his places" — autobiography

### Exercise 2 — Senwosret III Border Stelae (Semna, Nubia)

- "I am a king whose words command action"
- "As for keeping still after an attack, it is to encourage the heart of
  an enemy"
- "Aggression is bravery, retreat is misery"
- "They are not a people to respect: they are wretches with broken spirits"
- "who makes firm the border of the one who begot him"
- "To not let any Nubian pass it going north or overland"

### Exercise 3 — 78 Words with Determinatives

See table above. Each word has known transcription, meaning, and
determinative category.

---

## Axiom Summary (from Lessons 1-3)

1. **Sign function polymorphism** — same sign can be ideogram, phonogram,
   or determinative depending on context
2. **Reading direction** — signs face toward beginning of text
3. **Grouping** — tall signs alone, others in quadrats, top-to-bottom within
4. **Phonetic complement** — uniliteral after biliteral = complement, not
   additional letter
5. **Complement disambiguates** — resolves polymorphic biliteral values
6. **Ideogram marker** — stroke + no complement = ideogram reading
7. **Determinative position** — always at end of word
8. **Determinative = word boundary** — since no spaces exist
9. **Weak consonant optionality** — ꜣ, j, y, w may be omitted
10. **Doubled consonant reduction** — written once unless vowel-separated
11. **Sound change equivalences** — r→∅, ṯ→t, d̲→d create spelling variants
12. **Abbreviated spelling** — signs omitted for compactness, confirmed by
    Coptic cognates

---

## Architecture Notes

- **Equation Editor** already has 225 Gardiner signs as SVG templates
- Hieroglyphs compose in matrices (quadrats) without special-casing
- Type inference on the AST would act as translation
- Z3 Verify: "Is this reading valid under the grammar axioms?"
- Z3 Sat: "What are all valid readings of this damaged inscription?"
- No Rust code changes needed — all in .kleis structures

## Parallel to Moonlight Sonata Analysis

| Music                  | Egyptian                          |
|------------------------|-----------------------------------|
| Notes/durations        | Hieroglyphic signs                |
| TonalHarmony axioms    | MiddleEgyptian axioms             |
| Score = model          | Inscription = model               |
| "Does Moonlight satisfy| "Does this cartouche satisfy      |
| tonic opening?"        | phonetic complement rule?"        |
| Violation = style      | Violation = period/register       |
| SAT = axiom holds      | SAT = valid reading               |

---

## Lesson 4: Nouns

### Definitions (Section 4.1)

Nouns designate things — real or imaginary: objects (*cat*, *dragon*),
concepts (*happiness*), actions (*talking*), even words themselves (*"this"*).
Can be general ("country") or specific proper nouns ("Egypt").

### Parts of Nouns (Section 4.2)

Egyptian nouns = **root** + additions (prefixes, endings, suffixes).

- Root: the consonantal core shared by all related words
- Example: root *nṯr* → *nṯr* "god," *nṯrw* "gods," *nṯrt* "goddess,"
  *nṯtj* "divine"
- Most roots: 2-3 consonants, some up to 5

### Gender (Section 4.4) — KEY AXIOMS

Two grammatical genders: masculine and feminine. **Every noun must be one
or the other** (no neuter). Like Italian.

**Core rule:** Feminine nouns have ending **-t** added to the root.

```
gender(noun) = if has_ending(noun, t) then Feminine else Masculine
```

Gender pairs:

| Masculine | Feminine | Root |
|-----------|----------|------|
| *sn* "brother" | *snt* "sister" | *sn* |
| *ḥqꜣ* "ruler" | *ḥqꜣt* "female ruler" | *ḥqꜣ* |
| *nṯr* "god" | *nṯrt* "goddess" | *nṯr* |
| *ḫftj* "enemy" | *ḫftt* "female enemy" | *ḫft* |
| *ḥfꜣw* "snake" (m.) | *ḥfꜣt* "snake" (f.) | *ḥfꜣ* |

**Exceptions and refinements:**

1. Some masculine nouns have *t* as part of the root, not an ending:
   *ḫt* "wood" — *t* is the last radical. Must distinguish root-*t* from
   ending-*t*.

2. Naturally gendered nouns: *jtj* "father" (masc), *mwt* "mother" (fem)
   follow the same rule; they are just inherently one gender.

3. **Semantic override:** *ḫt* "thing" (originally *ḫbt*) is feminine.
   Same spelling as masculine *ḫt* "wood." When referring to an actual
   thing → feminine; when meaning "something/anything" → masculine.
   Context determines gender for homographs.

4. **Proper names of places** are always feminine regardless of ending:
   *kmt* "Egypt," *njwt* "town."

Formalization:

```
axiom feminine_t_ending(root: Root, noun: Noun) {
    gender(noun) = Feminine ↔ (ending(noun) = t ∨ place_name(noun))
}

axiom root_t_not_ending(root: Root) {
    last_consonant(root) = t → ending(root_form(root)) ≠ t
}
```

### Number (Section 4.5) — KEY AXIOMS

Two numbers: singular and plural. (Dual exists too — covered later.)

**Plural formation rules:**

- Masculine: add **-w** to the root → *sn* → *snw* "brothers"
- Feminine: add **-wt** (replaces the -t ending) → *snt* → *snwt* "sisters"

English comparison: far more regular than English (which has -s, -es, -en,
vowel changes, and zero-plural). Egyptian has one rule per gender.

```
plural(noun) = match gender(noun) {
    Masculine => root(noun) + w
    Feminine  => root(noun) + wt
}
```

### Writing the Plural (Section 4.6) — SURFACE vs. GRAMMAR

The *-w* plural ending is a weak consonant and is **often omitted** in
writing. Instead, three short strokes ("plural determinative") are added
as a visual marker. Strokes can be horizontal, vertical, or grouped,
depending on scribe preference and surrounding sign shapes.

- Feminine nouns: almost always use just the strokes (no explicit *-wt*)
- Masculine nouns: sometimes write *-w* in addition to strokes

**Archaic plural writing:** repeat the determinative or the entire word
three times. Hardly used in Middle Egyptian except religious texts.
Exception: *nṯrw* "gods" is normally written with triple determinative.

**False plurals:** Plural strokes used on non-plural words:
- *rḫyt* "subjects" — collective noun (singular but refers to group)
- *nfrw* "perfection" — ends in *-w* but is singular

This is critical for the theory: **surface writing ≠ grammatical number**.
The axiom checker must distinguish orthographic plural markers from
actual morphological plurality. Same problem as surface vs. skeleton
harmony in the Moonlight analysis.

### The Dual (Section 4.7) — KEY AXIOMS

A third number for exactly two things.

**Dual formation rules:**
- Masculine: add **-wj** to singular → *sn* → *snwj* "two brothers"
- Feminine: add **-tj** to singular → *snt* → *sntj* "two sisters"

Note: feminine dual adds to the *singular form* (including -t), unlike
plural which replaces -t with -wt.

Examples:

| Singular (m.) | Dual (m.) | Singular (f.) | Dual (f.) |
|---------------|-----------|---------------|-----------|
| *sn* | *snwj* | *snt* | *sntj* |
| *ḥqꜣ* | *ḥqꜣwj* | *ḥqꜣt* | *ḥqꜣtj* |
| *nṯr* | *nṯrwj* | *nṯrt* | *nṯrtj* |
| *ḫftj* | *ḫftjwj* | *ḫftt* | *ḫfttj* |
| *ḥfꜣw* | *ḥfꜣwwj* | *ḥfꜣt* | *ḥfꜣtj* |

The *-j* is weak and often omitted in writing. When shown, written with
the sign for *y*. Archaic form: double the determinative or write the
word twice — more common for duals than for plurals.

**False duals:** *njwj* "local" (from *njwt* "town") written as dual
because it matches *njwtj* "two towns." Same word, different reading.

```
dual(noun) = match gender(noun) {
    Masculine => singular(noun) + wj
    Feminine  => singular(noun) + tj
}
```

### Summary of Gender and Number (Section 4.8)

The full morphological paradigm — **every noun** carries both gender and
number:

```
sort Gender = Masculine | Feminine
sort Number = Singular | Plural | Dual
```

**Masculine forms:**

| Number | Ending | Example | Meaning |
|--------|--------|---------|---------|
| Singular | ROOT | *sn* | "brother" |
| Singular | ROOT + *j* | *ḫftj* | "enemy" |
| Singular | ROOT + *w* | *ḥfꜣw* | "snake" |
| Plural | ROOT + *w* | *snw* | "brothers" |
| Dual | ROOT + *wj* | *snwj* | "two brothers" |

**Feminine forms:**

| Number | Ending | Example | Meaning |
|--------|--------|---------|---------|
| Singular | ROOT + *t* | *snt* | "sister" |
| Plural | ROOT + *wt* | *snwt* | "sisters" |
| Dual | ROOT + *tj* | *sntj* | "two sisters" |

Note the ambiguity: masculine singular *ROOT + w* (like *ḥfꜣw* "snake")
looks identical to masculine plural *ROOT + w* (like *snw* "brothers").
Only the determinative and context distinguish them.

### Axiom Summary (Lesson 4, additions to axioms 1-12)

13. **Gender binary** — every noun is masculine or feminine, no neuter
14. **Feminine -t ending** — feminine nouns end in -t added to root
15. **Root-t disambiguation** — final root consonant -t ≠ feminine ending
16. **Place names feminine** — proper names of places are always feminine
17. **Masculine plural -w** — masculine nouns pluralize by adding -w
18. **Feminine plural -wt** — feminine nouns pluralize by adding -wt
19. **Masculine dual -wj** — masculine dual adds -wj to singular
20. **Feminine dual -tj** — feminine dual adds -tj to singular
21. **Plural strokes** — three strokes indicate plural/dual in writing
    (but may be "false plurals" — orthographic, not grammatical)
22. **Surface ≠ grammar** — written form may omit weak consonants or
    use visual markers that don't reflect actual morphology

### Defined and Undefined Nouns (Section 4.9)

**No articles in standard Middle Egyptian.** No "the" or "a."
*ḥfꜣw* = "the snake," "a snake," or just "snake" depending on context.

- **Defined nouns** — refer to specific things (marked by possessives,
  demonstratives in later lessons)
- **Undefined nouns** — refer to classes
- Proper names are always defined
- Late Egyptian eventually developed articles (from demonstratives)

Formalization: definiteness is a **context-dependent property**, not
morphologically marked. The theory should model it as an inferred
attribute, not a surface feature.

```
sort Definiteness = Defined | Undefined
// Inferred from context, not from morphology
```

### Noun Phrases (Sections 4.10-4.13)

Three relationships between nouns in a phrase:

**1. Apposition (Section 4.11)** — two nouns side by side, same referent.
One general, one proper noun.
- *sꜣ.k ḥrw* "your son, Horus"
- *sš rmṯw* "scribe Ramose"
- Titles followed by names: *Egyptian apposition* = title-name pairs

**2. Connection / Conjunction (Section 4.12)** — "and" / "or"
- **No word for "and"** — conjunction expressed by juxtaposition:
  *t ḥ(n)qt* = "bread and beer" (just placed side by side)
- Sometimes *ḥnꜥ* "together with" or *ḥr* "upon" used
- **Disjunction:** juxtaposition, or *r-pw* "whichever" after second noun

This is remarkable for the theory: **conjunction is implicit**. The parser
must infer syntactic relationship from position alone. No lexical markers.

```
axiom conjunction_by_juxtaposition(n1: Noun, n2: Noun) {
    // Two nouns in sequence with no linking word = conjunction
    adjacent(n1, n2) ∧ ¬apposition(n1, n2) → conjoined(n1, n2)
}
```

**3. Possession (Section 4.13)** — "X of Y" / "X's Y"
Two methods in Egyptian (details in next page):
- Direct genitive (juxtaposition)
- Indirect genitive (with linking word)

### Possession (Section 4.13) — KEY AXIOMS

Two genitive constructions:

**1. Direct genitive** — juxtaposition, possessor SECOND. No linking word.

```
direct_genitive(A, B) → "B's A"   (A belongs to B)
// A = possessed, B = possessor
// No morphological change to either noun
```

Examples:
- *rꜣ js* "the tomb's door" (mouth/door + tomb)
- *ḥjmt wꜥb* "a priest's wife"
- *nswt tꜣwj* "Egypt's king" (king + Two Lands)
- *nswt nṯrw* "the gods' king"

Either noun can be any gender/number/definiteness. Very common.

**2. Indirect genitive** — linked by genitival adjective *n*/*nw*/*nt*.

| Form | Agreement | Example |
|------|-----------|---------|
| *n* | A is masc. singular | *sꜣ n zj* "the son of a man" |
| *nw* | A is masc. plural/dual | *smrw nw stp-sꜣ* "courtiers of the palace" |
| *nt* | A is feminine (any number) | *swḥt nt njw* "the egg of an ostrich" |

The genitival adjective **agrees with the possessed noun (A)**, not the
possessor (B). By late Middle Egyptian, all three forms were collapsing
to just *n*.

```
indirect_genitive(A, link, B) ∧ gender(A) = Masculine ∧ number(A) = Singular
    → link = n

indirect_genitive(A, link, B) ∧ gender(A) = Masculine ∧ number(A) ∈ {Plural, Dual}
    → link = nw

indirect_genitive(A, link, B) ∧ gender(A) = Feminine
    → link = nt
```

### Summary of Noun Phrases (Section 4.14)

Two juxtaposed nouns A B can express:

| Relationship | Meaning | Explicit marker |
|-------------|---------|-----------------|
| Apposition | "A, B" (same referent) | none |
| Connection | "A and B" / "A or B" | *ḥnꜥ*, *r-pw* (optional) |
| Direct genitive | "B's A" | none |
| Indirect genitive | "A of B" | *n* / *nw* / *nt* |

**Critical disambiguation rule:** When only two nouns appear with no
linking word, context determines the relationship. Constraints that
disambiguate:
- Different genders → probably not apposition
- Title + proper name → apposition
- Concrete + concrete → connection or possession
- Semantically "belongs to" → possession

**This is why Egyptian didn't need explicit markers** — the nouns and
their context almost always rule out all but one relationship.

For the theory: this is a **constraint satisfaction problem**. Given two
adjacent nouns and their properties (gender, number, definiteness,
semantic class), the axioms should narrow the possible relationships
to (ideally) one. Z3 Sat mode: "What are all valid parsings of this
noun phrase?"

### Honorific Transposition (Section 4.15) — KEY AXIOM

In the direct genitive, the possessor is spoken second. But when the
possessor is a **god** (*nṯr*) or **king** (*nswt*), it is **written
first** out of respect. The written order ≠ spoken order.

```
axiom honorific_transposition(A: Noun, B: Noun) {
    direct_genitive(A, B) ∧ (is_divine(B) ∨ is_royal(B))
        → written_order(B, A)   // B written first, but means "B's A"
        ∧ spoken_order(A, B)    // Read as A-B in transcription
}
```

Examples:
- *ḥwt-nṯr* "temple" — written nṯr+ḥwt, read ḥwt-nṯr ("god's enclosure")
- *mdw-nṯr* "hieroglyphs" — "god's speech"
- *ḥm-nṯr* "priest" — "god's servant"
- *sꜣ-nswt* "prince" — "king's son"
- *sꜣt-nswt* "princess" — "king's daughter"
- *ḥjmt-nswt* "queen" — "king's wife"

Also applies in personal names with deity names written first:
- *ptḥ-šps* "Siptah" = "Ptah's son" (Ptah written first)
- *s-n-wsrt* "Senwosret" = "Man of goddess Wosret"
- *jmn-mry* = "beloved of Amun" (Amun written first)

In Middle Kingdom filiations: father's name written first out of respect.

**This is a critical parsing axiom.** A naive left-to-right parser would
get the possessor/possessed relationship backwards for every divine/royal
genitive. The theory must recognize honorific transposition and reorder.

This has no analog in the Moonlight analysis — it is unique to the
writing system. It means: **written order is not always syntactic order**.
The parser needs a semantic pass (identify divine/royal nouns) before
it can determine the correct syntactic structure.

### Essay 4: The Gods (Cultural Context)

Not grammar, but relevant to the theory's semantic categories:
- Egyptian gods = natural forces (wind, sun, earth, fertility)
- No separation of religion/science/daily life
- Shu (wind god): "I am Shu... my clothing is the air... my skin is
  the pressure of the wind" — god IS the phenomenon, not its controller
- Major deities: Atum (primordial matter), Geb (earth), Nut (sky),
  Re (sun), Osiris (male generation), Isis (motherhood)
- Abstract principles: Maat (order/harmony/truth)

For the theory: divine/royal classification of nouns is needed for
honorific transposition. The semantic category `is_divine` / `is_royal`
must be part of the Sign/Word type to trigger the reordering axiom.

### Axiom Summary (Lesson 4, all additions to axioms 1-12)

13. **Gender binary** — every noun is masculine or feminine, no neuter
14. **Feminine -t ending** — feminine nouns end in -t added to root
15. **Root-t disambiguation** — final root consonant -t ≠ feminine ending
16. **Place names feminine** — proper names of places are always feminine
17. **Masculine plural -w** — masculine nouns pluralize by adding -w
18. **Feminine plural -wt** — feminine nouns pluralize by adding -wt
19. **Masculine dual -wj** — masculine dual adds -wj to singular
20. **Feminine dual -tj** — feminine dual adds -tj to singular
21. **Plural strokes** — three strokes indicate plural/dual in writing
    (but may be "false plurals" — orthographic, not grammatical)
22. **Surface ≠ grammar** — written form may omit weak consonants or
    use visual markers that don't reflect actual morphology
23. **No articles** — definiteness is contextual, not morphological
24. **Apposition** — title + proper name in sequence = same referent
25. **Conjunction by juxtaposition** — no explicit "and"; adjacent nouns
    are conjoined
26. **Disjunction** — juxtaposition or *r-pw* marker
27. **Direct genitive** — A B with no linker = "B's A" (possessor second)
28. **Indirect genitive agreement** — genitival adjective *n*/*nw*/*nt*
    agrees with possessed noun (A), not possessor (B)
29. **Phrase disambiguation** — relationship between adjacent nouns is
    determined by gender, semantic class, and context constraints
30. **Honorific transposition** — divine/royal possessors written first
    but read second; written order ≠ syntactic order

### Test Corpus — Exercise 4

**Part 1: Plural and dual formation (16 nouns)**

| Noun | Meaning | Gender | Test |
|------|---------|--------|------|
| *sꜣ* | son | M | → *sꜣw*, *sꜣwj* |
| *ḥjmt* | woman | F | → *ḥjmwt*, *ḥjmtj* |
| *jtj* | father | M | → *jtjw*, *jtjwj* |
| *mwt* | mother | F | → *mwwt*, *mwtj* |
| *mjw* | cat | M | → *mjww*, *mjwwj* |
| *sš.w* | scribe | M | |
| *mnj.w* | herder | M | |
| *nbt* | mistress | F | → *nbwt*, *nbtj* |
| *šmꜥ.yt* | singer | F | |
| *jst* | place | F | → *jswt*, *jstj* |
| *pr* | house | M | → *prw*, *prwj* |
| *njwt* | town | F | → *njwwt*, *njwtj* |
| *ḥwt-nṯr* | temple | F | compound + hon. transp. |
| *sꜣ-nswt* | prince | M | compound + hon. transp. |
| *šnṯ* | farmer | M | |
| *ḏrt* | hand | F | |

**Part 2: Transcribe and translate (10 nouns in hieroglyphs)**

Body parts and animals — tests reading signs + applying morphology:
*rd* "foot," *ḥt* "belly," *msḏr* "ear," *ms* "child," *ḫt* "thing,"
*jꜥrt* "uraeus," *sprw* "petitioner," *ꜥ* "arm/hand," *sꜣt* "daughter,"
*msyt* "waterfowl"

**Part 3: Noun phrases — DISAMBIGUATION TEST (14 phrases)**

Allen warns: "some may be capable of more than one translation."
This is the Z3 Sat enumeration problem.

| Phrase | Possible readings |
|--------|-------------------|
| *nbt pt* | "mistress of the sky" (genitive) or apposition? |
| *nṯr ḥwt* | Honorific transposition → *ḥwt-nṯr* "temple" |
| *jst ꜥnḫ* | "place of living" (genitive) |
| *rm ꜣpd* | "fish and bird" (conjunction) |
| *tꜣ* | "land" (alone) |
| *jtrw* | "river" |
| *sbꜣ* | "star" |
| *r kmt* | "speech of Egypt" (genitive) |
| *t mw* | "bread and water" (conjunction) |
| *tꜣw ꜥnḫ* | "breath of life" (genitive) |

**Part 4: Damaged text reconstruction — Z3 SAT USE CASE (6 phrases)**

Square brackets = missing signs. Given partial text + known meaning,
reconstruct:

| Damaged text | Target | Missing element |
|-------------|--------|-----------------|
| *ḥr [... ] nr* | "surface of the stone" | genitive marker |
| *ḥwt-nṯr [...] mn* | "temple of Amun" | indirect genitive *n* |
| *snw [... ] snwt* | "brothers and sisters" | conjunction |
| *sn [...] sjr* | "Osiris's two sisters" | dual form + genitive |
| *wrw [... ] kmt* | "great ones of Egypt" | genitive marker |
| *ḥt [...] ꜥnḫ* | "wood of life" (= food) | genitive marker |

These are perfect Z3 test cases: partial constraints + grammar axioms
→ enumerate satisfying completions.

---

## Lesson 5: Pronouns

### Definitions and Categories (Sections 5.1-5.2)

Four kinds of personal pronouns in Middle Egyptian:
1. **Suffix** — bound to end of word (*pr.s* "her house")
2. **Dependent** — object pronouns (covered later)
3. **Independent** — standalone subject pronouns (covered later)
4. **Subject** — special subject form (covered later)

Each encodes **person** (1st/2nd/3rd), **gender** (masc/fem), and
**number** (sing/pl/dual). Theoretically 18 forms, but only ~8
commonly written — dual forms disappearing, gender distinctions in
plural/dual marked by vowels (invisible in writing).

```
sort Person = First | Second | Third
sort PronounKind = Suffix | Dependent | Independent | Subject
```

### Suffix Pronouns (Section 5.3) — KEY PARADIGM

The most common pronouns. Bound morphemes — attach to end of word,
cannot stand alone. Separated by dot or = in transcription.

| Person | Suffix | Example | Notes |
|--------|--------|---------|-------|
| 1s | *-j* | *pr.j* "my house" | Often OMITTED (weak *j*) |
| 2ms | *-k* | *pr.k* "your(m) house" | Also *-t* (sound change) |
| 2fs | *-ṯ* | *pr.ṯ* "your(f) house" | |
| 3ms | *-f* | *pr.f* "his house" | |
| 3fs | *-s* | *pr.s* "her house" | |
| 1pl | *-n* | *pr.n* "our house" | |
| 2pl | *-ṯn* | *pr.ṯn* "your(pl) house" | Also *-tn* |
| 3pl | *-sn* | *pr.sn* "their house" | → *-w* in Late Egyptian |

**Key observations for the theory:**

1. The 1s suffix *-j* is often not written — axiom 22 (surface ≠ grammar)
   applies to pronouns too
2. The 1s can be written as an **ideogram for the speaker type**: woman
   sign (female speaker), god sign (divine speaker), king sign (royal
   speaker), mummy (deceased speaker). Sign-function polymorphism from
   Lesson 1 at the pronoun level.
3. Sound changes create spelling variants: *-k* ~ *-t*, *-ṯn* ~ *-tn*
   (equivalence classes from axiom 11)
4. Suffix pronouns are **always last** in the word — positional axiom
   like determinative-final (axiom 7)

```
axiom suffix_pronoun_final(word: Word, pron: SuffixPronoun) {
    has_suffix(word, pron) → position(pron) = last(word)
}

axiom suffix_1s_optional_writing(word: Word) {
    has_suffix(word, j) → (written(j, word) ∨ ¬written(j, word))
    // Both spellings are valid — the suffix may be omitted
}
```

### Axiom Summary (Lesson 5, additions)

31. **Suffix pronoun paradigm** — 8 distinct forms encoding person,
    gender, number
32. **Suffix position** — suffix pronouns are always word-final
33. **1s optional writing** — first-person singular *-j* may be omitted
34. **1s speaker ideogram** — 1s can be written as ideogram indicating
    speaker type (human/divine/royal/deceased)
35. **Pronoun sound changes** — *-k* ~ *-t* and *-ṯn* ~ *-tn* are
    spelling variants of the same pronoun

### Dependent Pronouns (Section 5.4)

Separate words but cannot start a sentence — must follow another word.

| Person | Form | Notes |
|--------|------|-------|
| 1s | *wj* | *j* often omitted; speaker ideograms |
| 2ms | *ṯw* / *tw* | sound change variant |
| 2fs | *ṯn* / *tn* | |
| 3ms | *sw* | for people/gods |
| 3fs | *sj* | for people/gods |
| 1pl | *n* | |
| 2pl | *ṯn* / *tn* | |
| 3pl | *sn* | for people/gods |
| 3n | *st* | **NEUTRAL** — any gender/number, for things |

The 3n pronoun *st* breaks the strict gender binary — it's used for
non-personal referents regardless of grammatical gender. This is the
first exception to axiom 13.

Plural forms of dependent pronouns look identical to plural suffix
pronouns — another source of ambiguity resolved by position.

### Independent Pronouns (Section 5.5)

Freestanding words, can appear anywhere in a sentence.

| Person | Form | Composition |
|--------|------|-------------|
| 1s | *jnk* | *jn* + *k* |
| 2ms | *ntk* | *nt* + *k* |
| 2fs | *ntṯ* | *nt* + *ṯ* |
| 3ms | *ntf* | *nt* + *f* |
| 3fs | *nts* | *nt* + *s* |
| 1pl | *jnn* | *jn* + *n* |
| 2pl | *ntṯn* | *nt* + *ṯn* |
| 3pl | *ntsn* | *nt* + *sn* |

**Key structural insight:** Independent pronouns are **compositional**.
2nd/3rd person = *nt* + suffix pronoun. 1st person = *jn* + suffix.
The suffix paradigm is the generative core — all other pronoun forms
are derived from it.

Archaic forms: *twt* (2s) and *swt* (3s) — originally masculine,
used for both genders in Middle Egyptian. Found in religious texts.

### Pronoun Summary (Section 5.6)

| Person | Suffix | Dependent | Independent | English |
|--------|--------|-----------|-------------|---------|
| 1s | *-j* | *wj* | *jnk* | I, me, my |
| 2ms | *-k* | *ṯw* | *ntk* | you, your |
| 2fs | *-ṯ* | *ṯn* | *ntṯ* | you, your |
| 3ms | *-f* | *sw* | *ntf* | he, him, his |
| 3fs | *-s* | *sj/st* | *nts* | she, her, its |
| 1pl | *-n* | *n* | *jnn* | we, us, our |
| 2pl | *-ṯn* | *ṯn* | *ntṯn* | you, your |
| 3pl | *-sn* | *sn/st* | *ntsn* | they, them, their |

### Axiom Summary (Lesson 5, additions)

31. **Suffix pronoun paradigm** — 8 forms encoding person/gender/number
32. **Suffix position** — suffix pronouns are always word-final
33. **1s optional writing** — *-j* may be omitted in writing
34. **1s speaker ideogram** — 1s written as ideogram for speaker type
35. **Pronoun sound changes** — *-k*~*-t*, *-ṯn*~*-tn* are variants
36. **Dependent pronoun position** — cannot start a sentence
37. **Neutral 3n pronoun** — *st* is gender/number neutral for things
38. **Independent pronoun compositionality** — *nt*/*jn* + suffix
39. **Pronoun kind determines syntax** — suffix (possessive/verbal),
    dependent (object), independent (subject/predicate)

### Suffix Pronouns with Nouns (Section 5.7) — KEY RULES

Suffix + noun = possessive:
- *pr.f* "his house," *pr.s* "her house," *pr.sn* "their house"
- Suffix goes AFTER everything (endings + determinatives):
  *sn.tj.f* "his two sisters"
- Suffix gender/number is independent of noun gender/number
- Dual nouns may give suffix extra *-j*: *rdwj.fj* "his two feet"

**Critical ambiguity (1s suffix):**

The seated-man sign can be determinative AND/OR 1s suffix ideogram.
Three valid readings of *sꜣ* + man sign:
1. *sꜣ* "son" — man sign is determinative
2. *sꜣ.j* "my son" — man sign is 1s suffix, no determinative
3. *sꜣ(.j)* "my son" — man sign is determinative, suffix omitted

Z3 Sat: enumerate all valid parsings given the ambiguous signs.

### Demonstrative Pronouns (Sections 5.8-5.9) — KEY PARADIGM

Four series × three genders:

| Series | Masc (*p-*) | Fem (*t-*) | Neutral (*n-*) | Usage |
|--------|-------------|------------|----------------|-------|
| *-n* | *pn* | *tn* | *nn* | Most common "this/that" |
| *-f/-fꜣ* | *pf* | *tf* | *nf* | Contrastive "that" |
| *-w* | *pw* | *tw* | *nw* | Archaic ≈ *-n* series |
| *-ꜣ* | *pꜣ* | *tꜣ* | *nꜣ* | Colloquial variant |

**Gender agreement:** masc = *p-*, fem = *t-*, neutral = *n-*.
The suffix selects proximity/register.

```
sort DemSeries = Near | Far | Archaic | Colloquial
sort DemGender = DMasc | DFem | DNeutral

demonstrative(g: DemGender, s: DemSeries) = match (g, s) {
    (DMasc, Near)    => pn  | (DFem, Near)    => tn  | (DNeutral, Near)    => nn
    (DMasc, Far)     => pf  | (DFem, Far)     => tf  | (DNeutral, Far)     => nf
    (DMasc, Archaic) => pw  | (DFem, Archaic) => tw  | (DNeutral, Archaic) => nw
    (DMasc, Colloquial) => pꜣ | (DFem, Colloquial) => tꜣ | (DNeutral, Colloquial) => nꜣ
}
```

**Usage rules:**
- With nouns: demonstrative agrees in gender with the noun
- Standalone: neutral forms preferred (*nn* "this," *nꜣ* "that")
- *-n* vs *-f* contrast: *-n* = "this/these," *-f* = "that/those"
  (only when both appear; otherwise *-n* can mean either)

### Demonstrative Position Rules (Section 5.9, continued)

- *pn/tn/pw/tw*: always FOLLOW the noun — *nṯr pn* "this god"
- *pf/tf*: can follow or precede
- Plural demonstratives PRECEDE and link with *n*:
  *nn n nṯrw* "these gods" (literally "this of gods")
- **Direct genitive integrity:** nothing can come between the two
  nouns of a direct genitive. Demonstrative follows entire phrase:
  *ḥwt-nṯr tn* "this temple" (not *ḥwt tn nṯr*)
- Can stack with suffix pronouns: *ḏrt.j tn* "this my hand"

### Specialized Demonstrative Features (Section 5.10)

1. **Vocative use** — demonstrative in invocations:
   *j nḫt pw* "Oh, Nakht!" (no English equivalent for the dem.)
   *pw/tw* sometimes → *pwj/twj* in vocatives

2. **Archaic plural demonstratives** — separate set, used after nouns:
   - *jpn/jpw* (masculine plural): *nṯrw jpn* "these gods"
   - *jptn/jptw* (feminine plural): *nṯrwt jptn* "these goddesses"
   - *-w* forms for vocatives: *nṯrw jpw* "O gods!"
   Found mostly in religious texts.

3. **Demonstrative → definite article evolution:**
   *pꜣ, tꜣ, nꜣ* weakened from "this/that" to "the" by Late Egyptian.
   Same evolution as English/German/French/Spanish/Italian articles.
   Started in spoken Egyptian before the Middle Kingdom.

   For the theory: this means *pꜣ* is ambiguous between demonstrative
   and article readings in Middle Egyptian texts — another disambiguation
   target for Z3 Sat.

### New Possessive Pronouns (Section 5.11)

Formed from *-ꜣ* demonstrative + *-y* + suffix pronoun:

| Noun gender | Formula | Example |
|-------------|---------|---------|
| Masculine | *pꜣy* + suffix + noun | *pꜣy.sn pr* "their house" |
| Feminine | *tꜣy* + suffix + noun | *tꜣy.k ḥjmt* "your wife" |
| Plural | *nꜣy* + suffix + *n* + noun | *nꜣy.s n ḥrdw* "her children" |

Demonstrative part agrees with the NOUN, not the possessor — same
agreement rule as indirect genitive (axiom 28).

These forms appear from late Middle Kingdom onward, becoming standard
in Late Egyptian. They show language evolution through recomposition
of existing morphemes.

### Interrogative Pronouns (Section 5.12)

Five interrogative pronouns — **uninflected** (no gender/number):

| Pronoun | Meaning | Position | Composition |
|---------|---------|----------|-------------|
| *mj* | who?/what? | after other words | biliteral sign |
| *ptr* | who?/what? | sentence-initial | *pw* + *tr* |
| *jḫ* | what? (things only) | after other words | |
| *jsst* | what? (things only) | can be initial | *js* + *st* |
| *zy/zj* | which? | initial; 1st of direct gen. | |

**Key properties:**
- No inflection — first uninflected word class
- *mj* corresponds syntactically to dependent pronouns
- *ptr* corresponds to independent pronouns (stands first)
- *zy* takes direct genitive: *zy wꜣt* "which path?" = "which of path"
- *jḫ* vs *mj*: *jḫ* only for things, *mj* for people and things

### Essay 5: The Gods on Earth (Cultural Context)

Temple vocabulary for the theory's semantic categories:
- *ḥm-nṯr* "priest" = "god's servant" (honorific transposition)
- *ḥm-nṯr tpj* "high priest" = "first god's servant"
- *wꜥb* "purifier/cleaner" (lesser priest)
- *ꜥt sbꜣ* "room of teaching" (school)

### Axiom Summary (Lesson 5, all additions)

31. **Suffix pronoun paradigm** — 8 forms encoding person/gender/number
32. **Suffix position** — suffix pronouns are always word-final
33. **1s optional writing** — *-j* may be omitted in writing
34. **1s speaker ideogram** — 1s written as ideogram for speaker type
35. **Pronoun sound changes** — *-k*~*-t*, *-ṯn*~*-tn* are variants
36. **Dependent pronoun position** — cannot start a sentence
37. **Neutral 3n pronoun** — *st* is gender/number neutral for things
38. **Independent pronoun compositionality** — *nt*/*jn* + suffix
39. **Pronoun kind determines syntax** — suffix→possessive/verbal,
    dependent→object, independent→subject/predicate
40. **Possessive suffix after all endings** — suffix follows number
    endings and determinatives
41. **Suffix gender ≠ noun gender** — possessor's gender is independent
42. **1s determinative ambiguity** — man sign can be det., suffix, or both
43. **Demonstrative gender agreement** — *p-* masc, *t-* fem, *n-* neutral
44. **Demonstrative series** — *-n* (common), *-f* (far), *-w* (archaic),
    *-ꜣ* (colloquial→article)
45. **Standalone demonstrative** — neutral forms preferred when no noun
46. **Demonstrative position** — *pn/tn/pw/tw* follow noun; *pf/tf* either;
    plural demonstratives precede with *n*
47. **Direct genitive integrity** — nothing intervenes between the two
    nouns of a direct genitive
48. **Demonstrative-article ambiguity** — *pꜣ/tꜣ/nꜣ* can be demonstrative
    or definite article depending on period and context
49. **New possessive agreement** — *pꜣy/tꜣy/nꜣy* agrees with noun gender,
    not possessor gender
50. **Interrogatives uninflected** — no gender/number marking
51. **Interrogative position** — *mj/jḫ/jsst* after other words;
    *ptr/zy* sentence-initial
52. **Animate/inanimate distinction** — *jḫ/jsst* only for things,
    *mj/ptr* for people and things

---

### Test Corpus — Exercise 5

23 items from real literary texts:

| Source | Abbreviation | Period |
|--------|-------------|--------|
| Shipwrecked Sailor | ShS | Middle Kingdom |
| Tale of the Eloquent Peasant | Peas | Middle Kingdom |
| Inscriptions from Siut | Siut | First Intermediate Period |
| Helck, Historical-Biographical Texts | HBT | Various |
| Merikare | Merikare | ~2000 BC |

Key vocabulary from exercise:
*ms* "offspring," *sn* "sibling," *bꜣt* "field," *ḥjmt* "wife,"
*pr* "house," *ḥnw* "property," *šntj* "farmer," *ꜥꜣ* "donkey,"
*nb* "lord," *spꜣt* "estate," *ḥrd* "child," *ḥknw* "oil,"
*nḥw* "loss," *mšꜥ* "expeditionary force," *jst* "place,"
*sndm* "residence," *smr* "courtier," *dpt* "boat," *ḥꜣw* "vicinity,"
*qnyt* "braves" (collective), *qnbt* "council"

These are actual literary passages — ideal for end-to-end theory
validation against real Middle Egyptian texts.

### Potential Test Inscription: Merikare 11, 10-12

"Well provided are people, the flock of the god. For their sake he
has made the sky and the earth... They are his likenesses, who came
from his body... When they weep, he is listening... For the god knows
every name."

Rich in noun phrases, genitives, suffix pronouns — a good candidate
for full axiom verification.

---

## Lesson 6: Adjectives

### Definitions (Section 6.1)

Three kinds:
1. **Primary** — only *nb* "all/every"
2. **Secondary** — actually participles: *nfr* "good" from verb "to be good"
3. **Derived / nisbe** — from nouns/prepositions with *-j*:
   *njwtj* "local" from *njwt* "town"

### Agreement Rules (Section 6.2) — KEY AXIOMS

Adjectives agree in gender and number with their noun.

| Form | Primary | Secondary | Nisbe |
|------|---------|-----------|-------|
| Masc. sg. | *nb* | *nfr* | *njwtj* |
| Masc. pl. | *nbw* | *nfrw* | *njwtjw* |
| Feminine | *nbt* | *nfrt* | *njwtt* |

- Masc. pl.: add *-w* (same as nouns)
- Feminine: add *-t* (primary/secondary) or replace *-j* → *-t* (nisbe)
- Fem. pl.: same form as fem. sg. (distinct plural form lost)
- Late ME: masc. sg. used for all genders/numbers (erosion)

```
axiom adjective_agreement(noun: Noun, adj: Adjective) {
    modifies(adj, noun) →
        gender(adj) = gender(noun) ∧ number(adj) = number(noun)
}
```

**Position:** adjectives ALWAYS follow the noun. This is a HARD constraint.

```
axiom adjective_follows_noun(noun: Noun, adj: Adjective) {
    modifies(adj, noun) → position(adj) > position(noun)
}
```

This disambiguates homographs:
- *pr nb* = "every house" (adj follows noun)
- *nb pr* = "lord of the house" (noun in direct genitive)
- *ḥwt nbt* = "every enclosure" vs *nbt ḥwt* = "mistress of the enclosure"

**Agreement as type checking:** The adjective's gender/number ending
reveals WHICH noun it modifies. In *ḥjmt wꜥb nbt*, the *-t* on *nbt*
shows it modifies *ḥjmt* (feminine), not *wꜥb* (masculine). This is
exactly what a type checker does — uses type annotations to resolve
binding ambiguity.

### Adjective Order (Section 6.3)

Multiple adjectives follow the noun in sequence:
- *ḫt nbt nfrt wꜥbt* "every good and clean thing"
- *nb* "all" always first among adjectives
- Demonstratives precede other adjectives: *nṯr pf mnḫ* "that beneficent god"
- Suffix pronouns are part of the noun: *sḫrw.j jprw* "my excellent plans"

**Direct genitive + adjective interaction:**
- Adj. modifying 2nd noun: OK — *ḥjmt wꜥb nb* "every priest's wife"
- Adj. modifying 1st noun: follows entire phrase (*ḥjmt wꜥb nbt*)
  OR convert to indirect genitive (*ḥjmt nbt nt wꜥb*)
- Agreement endings disambiguate: *nbt* (fem) → modifies *ḥjmt*,
  not *wꜥb* (masc)

### Adjectives as Nouns (Section 6.4)

All Egyptian adjectives (except *nb*) ARE nouns. Adjective-noun
modification = apposition: *sḫrw.j jprw* "my plans, the good ones."

- *nfrt* alone = "the beautiful woman" or "a good thing"
- Adjective-nouns take suffixes, demonstratives, other adjectives:
  *nfrt.sn* "their good one," *nfr pn* "this good one"
- Exception: *nb* is ONLY a modifier, never a noun by itself.
  *nb* without preceding noun = the noun "lord"

```
axiom adjectives_are_nouns(adj: Adjective) {
    adj ≠ nb → can_be_noun(adj)
}
axiom nb_only_modifier() {
    standalone(nb) → word_class(nb) = Noun("lord")
    // nb without preceding noun is always the noun "lord"
}
```

### The *nfr ḥr* Construction (Section 6.5)

Adjective as first noun of a direct genitive:
- *nfr ḥr* "good of face" = "kind-faced"
- *ꜥšꜣ zrw* "many of sheep" = "one with many sheep"
- *nfrwt nt ḥꜥw.sn* "beautiful ones of their bodies"

Egyptian assigns quality to the OWNER, not the thing owned.
English reverses: "skilled of fingers" → "with skilled fingers."

### Apparent Adjectives (Section 6.7) — POSITION DISTINGUISHES

*ky* "other" is NOT an adjective — it's a noun that PRECEDES:
- *ky sbꜣ* "another gate" = apposition "another one, a gate"
- Real adjectives follow; *ky* precedes → different word class

| Form | Gender | Meaning |
|------|--------|---------|
| *ky* | m.sg. | another |
| *kt* | f.sg. | another |
| *kywj* | pl. | others |

Also *tnw* "each" and *nḥj* "some" — always first noun of genitive.

Prepositional phrases as adjectives: *r ḏr* "entire," *r ꜣw* "complete,"
*mj qd* "whole."

### Axiom Summary (Lesson 6, additions)

53. **Adjective agreement** — adjective agrees in gender and number
    with the noun it modifies
54. **Adjective position** — adjectives always follow their noun
55. **Position disambiguates word class** — *nb* after noun = "every"
    (adjective); *nb* before noun = "lord" (noun in genitive)
56. **Nisbe feminine** — replaces *-j* with *-t* (not adds *-t*)
57. **Feminine plural = feminine singular** — for adjectives, no
    distinct plural form
58. **nb first** — *nb* "all" precedes other adjectives
59. **Demonstrative before adjectives** — *nṯr pf mnḫ* "that beneficent god"
60. **Suffix before adjectives** — suffix pronouns precede adjectives
61. **Genitive + adjective** — adjective after direct genitive;
    agreement endings disambiguate which noun is modified
62. **Adjectives are nouns** — all adjectives (except *nb*) can function
    as nouns; adjective modification = apposition
63. **nb standalone = noun** — *nb* without preceding noun is always
    the noun "lord/master," never the adjective "all/every"
64. **nfr ḥr construction** — adjective-noun as first element of
    direct genitive describes characteristics
65. **Apparent adjectives precede** — *ky* "other," *tnw* "each,"
    *nḥj* "some" precede the noun (unlike real adjectives)
66. **Prepositional adjective phrases** — *r ḏr*, *r ꜣw*, *mj qd*
    function as adjectives meaning "entire/complete/whole"

### Comparative and Superlative (Section 6.8)

NO comparative or superlative forms. *nfr* = "good"/"better"/"best"
depending on context.

- **Comparative:** *r* "with respect to" after adjective:
  *nṯr mnḫ r nṯr nb* "a god more beneficent than any god"
- **Superlative:** genitive construction:
  *wr wrw* "greatest of the great"
  *wr n wrw* "the greatest of all"
  *wr jm(j) sꜥḥw* "greatest among the dignitaries"

For the theory: comparative/superlative are syntactic constructions,
not morphological forms. The axioms handle them as genitive phrases
with *r* or *n*, not as special adjective types.

### "Have" Expressions (Section 6.9)

NO verb for "have." Possession expressed through constructions:

1. *nb* + direct genitive: *nb ꜥꜣw* "owner of donkeys"
2. *nfr ḥr* construction: *ꜥšꜣ zrw* "many of sheep" = "has many sheep"
3. Genitival adjective *n(j)*: *n(j)-swt* → *nswt* "king"
   (= "he who has the sedge" = "the one belonging to the sedge")

```
axiom no_have_verb() {
    // Possession is expressed through genitive constructions
    // No dedicated verb "have" exists
    possession(owner, thing) →
        direct_genitive(nb, thing)     // "lord/owner of X"
        ∨ nfr_hr(quality, thing)       // "Q of X" = "has X with Q"
        ∨ nisbe_n(owner, thing)        // "the one belonging to X"
}
```

### Essay 6: The King's Names (Cultural Context)

From 5th Dynasty onward, every king had five official names
(the fivefold titulary):
1. **Horus name** — oldest; falcon on palace facade with name inside

The titulary is a rich test case for the theory: compound nouns,
honorific transposition, divine references, nisbe constructions.

### Axiom Summary (Lesson 6, all additions)

53. **Adjective agreement** — agrees in gender/number with noun
54. **Adjective position** — always follows noun
55. **Position disambiguates** — *nb* after = "every"; before = "lord"
56. **Nisbe feminine** — *-j* → *-t* (replacement, not addition)
57. **Fem. plural = fem. singular** — for adjectives
58. **nb first** — among multiple adjectives
59. **Demonstrative before adjectives**
60. **Suffix before adjectives**
61. **Genitive + adjective** — agreement disambiguates binding
62. **Adjectives are nouns** — except *nb*
63. **nb standalone = noun** — "lord/master"
64. **nfr ḥr construction** — adj + genitive = characteristic
65. **Apparent adjectives precede** — *ky*, *tnw*, *nḥj*
66. **Prepositional adjective phrases** — *r ḏr*, *r ꜣw*, *mj qd*
67. **No comparative/superlative morphology** — expressed by *r* or
    genitive constructions
68. **No verb "have"** — possession through genitive constructions
69. **nswt etymology** — *n(j)-swt* = nisbe "belonging to the sedge"

### The Fivefold Royal Titulary (Essay 6) — TEST CASE

Every king from Dynasty 5 onward had five names:

| # | Title | Construction | Axioms used |
|---|-------|-------------|-------------|
| 1 | Horus | *nfr ḥr* on *serekh* | 64 |
| 2 | Two Ladies | Compound name | 24 (apposition) |
| 3 | Gold Falcon | *nfr ḥr* on gold sign | 64 |
| 4 | Throne (*nswt-bjt*) | Nisbe + hon. transposition | 30, 69 |
| 5 | Sun's Son (*sꜣ rꜥ*) | Direct genitive + hon. transp. | 27, 30 |

Names 4 and 5 are written inside **cartouches** — a ring of rope
representing dominion over the world. Cartouche = phrase-level
determinative marking "this is a royal name."

Key vocabulary:
- *nswt-bjt* = *n(j)-swt + bjtj* = "he of the sedge and bee" = "Dual King"
- *sꜣ rꜥ* = "Son of Re" (honorific transposition: Re written first)
- *pr ꜥꜣ* = "Big House" → "pharaoh" (originally estate, not person)
- *nṯr nfr* = "young god" (epithet before cartouche)
- *nb tꜣwj* = "lord of the Two Lands"

The titulary is an ideal first test case: it combines *nfr ḥr*
constructions, honorific transposition, nisbe derivations, cartouche
framing, and compound nouns — exercising axioms 24, 27, 30, 64, 69
simultaneously.

### Test Corpus — Exercise 6

**Part 1: 16 items from real literary/medical/historical texts**
Sources: Peasant, Westcar Papyrus, Coffin Texts, Urk. IV, Sinuhe,
Kahun Papyrus, Ebers Medical Papyrus

Vocabulary: *sḫr* "plan," *jnw* "products," *ꜥt* "room,"
*ḥnw* "interior," *ꜥḥ* "palace," *ꜣḫt* "Akhet/horizon,"
*pḥtj* "strength," *mnḏ* "breast," *wr* "great," *mnw* "monument,"
*zp* "time/occasion," *jnpwt* "crew," *pḥrt* "prescription,"
*rnpt* "year," *mrwt* "dependants"

**Part 2: Royal throne names as *nfr ḥr* constructions**
- Pepi II (Dyn. VI, ca. 2246-2152 BC): with *kꜣ* "life force"
- Thutmose III (Dyn. XVIII, ca. 1479-1425 BC): *mn* "permanent,"
  *ḫpr* "evolution"
- Hatshepsut (Dyn. XVIII, ca. 1473-1458 BC): *mꜣꜥt* "true" (fem.)
- Ramesses I (Dyn. XIX, ca. 1302-1301 BC): *pḥtj* "strength"

Four throne names spanning 1,000 years (Dyn. VI through XIX), all
following the same *nfr ḥr* construction. Each is: adj + direct
genitive, with honorific transposition of *rꜥ*. The theory should
parse all four identically — demonstrating the grammar's stability
across a millennium.

Note: Hatshepsut's throne name uses the feminine *mꜣꜥt* — significant
because she was a female pharaoh using the normally masculine titulary.

---

## Lesson 7: Adjectival and Nominal Sentences

### Definitions (Section 7.1) — FUNDAMENTAL

**Egyptian has NO verb "be."** No copula. Predication is achieved by
juxtaposition alone. This makes non-verbal sentences the primary
sentence type.

```
sort SentenceType = Adjectival | Nominal | Verbal
// Adjectival and Nominal are both non-verbal
// No copula verb exists
```

### Adjectival Sentences (Section 7.2) — KEY AXIOMS

**Word order: PREDICATE first, SUBJECT second** (reverse of English).

```
axiom adjectival_sentence_order(pred: Adjective, subj: NounPhrase) {
    sentence(pred, subj) → position(pred) < position(subj)
}
```

Examples:
- *jqr sḥr pn* "This plan is excellent" (excellent + this plan)
- *nfr ḥjmt tn* "This woman is beautiful" (beautiful + this woman)
- *jqr nn n sḥrw* "These plans are excellent" (excellent + these plans)

**Predicate adjective is ALWAYS masculine singular** — no agreement
with subject. This is the critical distinction from modifying
adjectives (which agree in gender/number, axiom 53):

```
axiom predicate_adjective_uninflected(pred: Adjective, subj: NounPhrase) {
    is_predicate(pred) → form(pred) = MasculineSingular
    // Regardless of subject's gender or number
}
```

This means position determines function:
- Adjective AFTER noun → modifier (agrees) — axiom 53
- Adjective BEFORE noun → predicate (uninflected) — this axiom

Only secondary adjectives can be predicates. *nb* = modifier only.

**Exclamatory form:** Dual ending *-wj* on predicate = exclamation:
- *nfrwj ḥjmt tn* "How beautiful is this woman!"
- This is the ONLY case where predicate has an ending
- *-wj* on predicate always = exclamatory, never dual

```
axiom exclamatory_wj(pred: Adjective) {
    is_predicate(pred) ∧ has_ending(pred, wj)
        → exclamatory(sentence)
}
```

### Subject Types (Section 7.3)

Subjects can be: nouns, noun phrases, adjectives-as-nouns (§6.4),
or pronouns (demonstrative/personal). All follow the predicate.

### Pronoun Subjects in Adjectival Sentences (Section 7.3 cont.)

Dependent pronouns used as subjects — only 2nd/3rd person:
- *nfr sw* "He is good," *nfr sj* "She is good," *nfr st* "It is good"
- Predicate still uninflected (masculine singular) regardless of
  pronoun's gender/number

With exclamatory: *nfrw(j) st* "How good it is!"

Pronoun + noun apposition: *rwḏ rwḏ sw jb.j* "How firm is my heart!"
— pronoun *sw* agrees with appositional noun *jb.j* in gender/number.

### Additions (Section 7.4)

- *wrt* "very" — between predicate and subject
- Comparative *r* — after subject: *nfr st r ḫt nbt* "It is better
  than anything"
- Pronoun as subject with noun in apposition — the noun elaborates
  the pronoun

### Nominal Sentences (Sections 7.5-7.7) — THREE PATTERNS

**A B**, **A pw**, **A pw B** — all without a verb.

**A B with pronouns (Section 7.6):**

Two configurations:
1. Independent pronoun + noun: *jnk wḥmw jqr* "I am an able herald"
   (independent pronouns always FIRST)
2. Noun + demonstrative: *dpt m(w)t nn* "This is the taste of death"
   (demonstratives always SECOND)

```
axiom nominal_AB_pronoun(A: IndependentPronoun, B: NounPhrase) {
    nominal_sentence(A, B) → position(A) < position(B)
}
axiom nominal_AB_demonstrative(A: NounPhrase, B: Demonstrative) {
    nominal_sentence(A, B) → position(A) < position(B)
}
```

**A B with nouns (Section 7.7):** Both A and B can be nouns, but
only in special circumstances (continued next page).

### A B with Nouns (Section 7.7)

Two special cases where both A and B are nouns:
1. **Inalienable nouns** — kinship (*mwt* "mother") or identity (*rn* "name"):
   *mwt.j nwt* "My mother is Nut"
2. **Balanced sentences** — same noun in both parts:
   *mkt.t mkt rꜥ* "Your protection is the Sun's protection"

### Sentences of Adherence (Section 7.8) — KEY PATTERN

Using the nisbe *n(j)* "belonging" + pronoun:

| Pronoun type | Structure | Meaning | Example |
|-------------|-----------|---------|---------|
| Dependent | *n(j)* dep B | "A belongs to B" | *n(j) wj rꜥ* "I belong to the Sun" |
| Independent | *n(j)* indep B | "B belongs to A" | *n(j) ntk ḥrw* "Daytime belongs to you" |

Fused forms: *n(j)-wj* → *nīwa*, *n(j)-sw* → *nīsu*, *nnk* = *n(j)-jnk*.

Very common in personal names: *n(j)-sw-mntw* "He belongs to Montu"
(with honorific transposition).

Contracted: *nnk pt nnk tꜣ* "The sky is mine, the earth is mine."

### Subject/Predicate Identification (Section 7.12) — DISAMBIGUATION

Rules for determining which element is subject vs predicate:

1. **Kinship/name nouns as A** — *mwt.j nwt* "My mother is Nut"
   (kinship terms go in A, always the subject)
2. **rn "name"** — *rn* part is always the subject; *bꜣbꜣ sꜣ r-jnt rn.f*
   "His name is Baba, son of Reinet"
3. **Balanced sentences** — *mkt.t mkt rꜥ* answers "What is your
   protection?" (A = subject)

**3rd person subject/predicate split** — the most important rule:
- 3rd person as **subject** → A pw: *ḥqꜣ pw* "He is the ruler"
- 3rd person as **predicate** → A B: *ntf ḥqꜣ* "He is the ruler"
- 1st/2nd person → ambiguous: *jnk ḥqꜣ* can be either

Full paradigms (pronoun as subject vs predicate):

| Person | Subject ("Who are you?") | Predicate ("Who is the ruler?") |
|--------|--------------------------|----------------------------------|
| 1s | *jnk ḥqꜣ* | *jnk ḥqꜣ* (also *jnk pw ḥqꜣ*) |
| 2s | *ntk ḥqꜣ / ntt ḥqꜣt* | *ntk ḥqꜣ / ntt ḥqꜣt* |
| 3s | *ḥqꜣ pw / ḥqꜣt pw* | *ntf ḥqꜣ / nts ḥqꜣt* |
| 1pl | *jnn ḥqꜣw* | *jnn ḥqꜣw* |
| 2pl | *ntṯn ḥqꜣw* | *ntṯn ḥqꜣw* |
| 3pl | *ḥqꜣw pw* | *ntsn ḥqꜣw* |

→ The 3rd person is where Egyptian grammar forces disambiguation via
pattern choice. All other persons are structurally ambiguous.

### Interrogatives in Nominal Sentences (Section 7.13) — KEY PATTERN

| Pronoun | Meaning | Position | Pattern | Example |
|---------|---------|----------|---------|---------|
| *mj* | who/what | after *jn* | *jn mj* dep | *jn mj ṯr pw* "Who are you?" |
| *mj* | who/what | after indep | indep *mj* | *ṯwt mj* "Who are you?" (archaic) |
| *ptr* | who/what | **always first** | *ptr* NP / dep | *ptr rn.k* "What is your name?" |
| *jsst* | what | in A pw | *jsst (pw)j* | "What is it?" |
| *zy* | which | after indep | indep *zy* | *ntk zy* "Which one are you?" |

Key insight: *jn mj* fused to one word → Coptic *nim* "who" —
diachronic phonological evidence confirming syntactic analysis.

*ptr* is the "normal" interrogative; *mj* after *jn* is extremely
common in funerary/religious texts; *jsst* and *zy* are less frequent.

General rule: interrogative-first, except when independent pronoun is
subject. Interrogative adjective *wr* also serves as predicate:
*wr pw* "How much is it?"

### First Person in Adjectival Sentences (Section 7.14)

1st person avoids adjectival sentences — uses nominal construction:

| Person | Form | Sentence Type |
|--------|------|---------------|
| 1s | *jnk nfr/nfrt* "I am (a) good (man/woman)" | **Nominal** |
| 2m/fs | *nfr ṯw/ṯn* "You are good" | **Adjectival** |
| 3ms | *nfr sw* "He is good" | **Adjectival** |
| 3fs | *nfr sj* "She is good" | **Adjectival** |
| 1pl | *jnn nfrw/nfrt* "We are good (men/women)" | **Nominal** |
| 2pl | *nfr ṯn* "You are good" | **Adjectival** |
| 3pl | *nfr sn* "They are good" | **Adjectival** |
| 3n | *nfr st* "It is good" / "They are good" | **Adjectival** |

The adjective in *jnk nfr* is (usually) the predicate, but a **nominal**
predicate — not an adjectival one.

### Nominal vs. Adjectival Sentences (Section 7.15)

*ḥns pw* (Peas. B1, 25) "It is narrow" — but *pw* makes this a nominal
sentence meaning "It is a narrow one." The adjectival equivalent would
be *ḥns st* "It is narrow." The distinction: adjective-as-noun (nominal)
vs. adjective-as-predicate (adjectival).

### Tense in Non-verbal Sentences (Section 7.16) — KEY INSIGHT

Non-verbal sentences have **no verbs** → **no inherent tense**.

Two temporal modes:
1. **Gnomic** — always true: *plrt pw ꜥnḫ* "Life is a cycle"
2. **Contextual** — tense comes from discourse:
   - In narrative → past: *šꜣ wrt wꜣt* "the path was very remote"
   - In report → present: "the path is very remote"
   - In prophecy → future: "the path will be very remote"

→ Tenselessness is a fundamental property of copulaless predication.
Context is the only tense-assigning mechanism.

### Phrases vs. Sentences (Section 7.17)

Identical word sequences can be phrases or sentences:

| Words | As Phrase | As Sentence |
|-------|-----------|-------------|
| *nfr ḥr* | "good of face" (§6.5) | "The face is good" (§7.2) |
| *mwt.j nwt* | "my mother, Nut" (§4.11) | "My mother is Nut" (§7.8) |
| *sꜣ.j pw* | "this my son" (§5.8) | "He is my son" (§7.9) |

Out of context: genuinely ambiguous. In actual texts: context almost
always resolves. Most adjectival and nominal sentences are clear enough
to be read only as sentences.

→ For Kleis: this is a constraint satisfaction problem. Given a word
sequence, enumerate all valid parsings (phrase vs sentence), then use
discourse context to select.

### Essay 7: Human Nature (Cultural Context)

Five elements of Egyptian personhood — ontological framework:

| Element | Egyptian | Meaning | Note |
|---------|----------|---------|------|
| Body | *ḏt/ḥꜥ* (*ḥꜥw* coll.) | physical shell | *ḥꜥw* = "body parts" (collective) |
| Heart/Mind | *jb/ḥꜣty* | thought + emotion center | *ḥꜣty* = nisbe from *ḥꜣt* "front" |
| Shadow | *šwt* | essential adjunct | gods' images called "shadows" |
| Ba | *bꜣ* (*bꜣw* false pl.) | personality/soul | human-headed bird; lives after death |
| Ka | *kꜣ* (*kꜣw* false pl.) | life force | transmitted via embrace; *n kꜣ k* "for your ka" |
| Name | *rn* | essential identity | destruction of name = destruction of person |

*bꜣw* and *kꜣw* reinforce false-plural pattern (axiom 23): abstract
nouns written with plural determinative but singular meaning.

Name (*rn*) = "identity" — destroying a name destroys the person.
Writing a name on a statue identifies image with individual.
Gods' names are "inaccessible"/"secret" — unknowable even by other gods.
→ Reinforces *rn*'s special syntactic behavior in nominal sentences (§7.12).

### Exercise 7 — Test Corpus (42 sentences)

Instruction: Transcribe and translate; underline the predicate in each.

**Source texts represented:**
- ShS (Shipwrecked Sailor) — #2, 15-24
- Peas. (Eloquent Peasant) — #30-40
- CT (Coffin Texts) — #4, 25, 29
- Sin. (Sinuhe) — #6
- Leb. (Man and His Ba) — #13-14
- Urk. (Urkunden) — #7, 12
- Helck HBT — #8-11
- Kahun, Beni Hasan, Siut, etc. — #1, 3, 5

**Key new vocabulary:**
- *wꜣḏ-wr* "sea" (lit. "great blue-green") — compound adjective+noun
- *bw-nfr* "goodness" — compound with *bw* "thing/place"
- *pḥtj* "strength" — false dual
- *ḥmwṯ* "craftsman" — nisbe from *ḥmwt* "craft"
- *wnḏwt* "tenants" — collective noun
- *mḫꜣt* "scale, measure of worth" — metaphorical use
- *ḏt* "self" — reflexive noun

**Tense-context examples:**
- #19-22 are explicitly noted as "from a story, past"
- #30 is "from a story, past"
- Others are gnomic or context-dependent (reinforces axiom 97)

**Predicate identification** is the primary test — every sentence
requires determining which element is the predicate, testing axioms
70-98. This exercise is the ideal test corpus for the Kleis
disambiguation engine.

### A pw Nominal Sentences (Section 7.9) — KEY PATTERN

*pw* as copula substitute. A = any noun/pronoun; *pw* = neutral
demonstrative functioning as predicate marker:

- *sꜣ.j pw* "He is my son"
- *rꜥ pw* "It is the Sun"
- *ḥjmt wꜥb pw* "She is a priest's wife"

**pw is neutral** — any gender/number referent. *pw* stands as close
to the beginning as possible, but cannot break a direct genitive.

```
axiom pw_position(A: NounPhrase) {
    A_pw_sentence(A, pw) →
        position(pw) = position(A) + 1
        // pw immediately follows A (or the minimal A that
        // preserves direct genitive integrity)
}
```

### Axiom Summary (Lesson 7, partial)

70. **No copula** — no verb "be"; predication by juxtaposition
71. **Predicate-subject order** — predicate first, subject second
    (adjectival sentences)
72. **Predicate adjective uninflected** — always masculine singular
    regardless of subject's gender/number
73. **Position determines function** — adj before noun = predicate;
    adj after noun = modifier
74. **Exclamatory -wj** — dual ending on predicate = "How...!"
75. **Only secondary adjectives as predicates** — *nb* excluded
76. **Dependent pronoun as subject** — only 2nd/3rd person in
    adjectival sentences
77. **Nominal A B** — independent pronoun first, or demonstrative second
78. **Pronoun-noun apposition** — pronoun subject + appositional noun
    agree in gender/number
79. **Balanced sentences** — same noun in A and B = equation
80. **Adherence with dependent** — *n(j)* + dependent = "A belongs to B"
81. **Adherence with independent** — *n(j)* + independent = "B belongs to A"
82. **pw neutral** — *pw* in A *pw* is gender/number neutral
83. **pw position** — *pw* as close to beginning as possible,
    respecting direct genitive integrity
84. **pw insertion** — *pw* can split A at indirect genitive or
    adjective boundaries, but NOT direct genitives
85. **A pw B** — "B is A" when both are nouns; A = predicate
86. **Two nominal patterns** — A B (one element is pronoun) and
    A pw B (both elements are nouns) are complementary
87. **A = predicate** — in nominal sentences, A is usually the
    predicate (exception: adherence with dependent pronoun)
88. **3rd person subject uses pw** — *ḥqꜣ pw* "He is the ruler"
    (3rd person subject); *ntf ḥqꜣ* (3rd person predicate)
89. **1st/2nd person ambiguity** — *jnk ḥqꜣ* can be subject or
    predicate; A pw B disambiguates
90. **ptr sentence-initial** — most common interrogative in nominal
    sentences, always first
91. **jn mj** — *mj* in nominal sentences usually follows particle *jn*;
    fused to *jn-mj* → Coptic *nim*
92. **Interrogative-first preference** — interrogative pronouns prefer
    sentence-initial position; exception: independent pronoun subject
    precedes interrogative
93. **1st person adjectival avoidance** — 1st person uses nominal
    *jnk nfr* instead of adjectival *\*nfr wj*; the adjective
    functions as a nominal predicate
94. **Nominal vs adjectival distinction** — *ḥns pw* "It is a narrow
    one" (nominal, adj used as noun) vs *ḥns st* "It is narrow"
    (adjectival, adj as predicate)
95. **No inherent tense** — non-verbal sentences have no tense;
    tenselessness is a fundamental property of copulaless predication
96. **Gnomic mode** — non-verbal sentences can express timeless truths:
    *plrt pw ꜥnḫ* "Life is a cycle" (always true)
97. **Contextual tense** — non-verbal sentences acquire tense from
    surrounding discourse context (narrative past, reporting present,
    prophetic future)
98. **Phrase/sentence ambiguity** — identical word sequences can be
    either phrases or sentences; *nfr ḥr* = "good of face" (phrase)
    or "The face is good" (sentence); context resolves
99. **Prosodic disambiguation** — Coptic evidence: stress distinguishes
    *jnk **ḥqꜣ*** (predicate stressed) from ***jnk** ḥqꜣ* (subject
    stressed); phonological layer invisible in writing
100. **Sign tucking** — *t* sign can be tucked into bird sign bellies
     (*šwt*, *mwt*); visual arrangement ≠ reading order; aesthetic
     graphemic constraint

### Axiom Summary (Lesson 8, partial)

101. **Three preposition forms** — each preposition can have up to three
     forms: before noun, with pronoun (suffix), adverbial
102. **m polysemy** — *m* has 7+ semantic roles (locative, temporal,
     stative, material, ablative, essive, instrumental); context selects
103. **n vs r for goals** — *n* = goal-of-person; *r* = goal-of-place
104. **ḥnꜥ vs m for "with"** — *ḥnꜥ* = accompaniment; *m* = instrument
105. **ḫr social register** — *n* for equals/inferiors; *ḫr* for
     superiors (king, gods)
106. **Suffix pronouns on prepositions** — suffix attaches directly;
     *jn* and *mj* are exceptions (cannot take suffixes)
107. **mj pronoun workaround** — *mjṯw/mjṯj* "likeness" replaces
     *mj* + pronoun: *šṯj mjṯw.j* "a farmer like me"
108. **Compound preposition formation** — three patterns: prep+noun,
     prep+infinitive, adverb+prep; meaning is compositional
109. **Subjectless adjectival + n** — *nfr n.tn* "It is good for you";
     Egyptian omits dummy subject
110. **Prepositional nisbe derivation** — systematic: preposition + -j/-t
     → adjective; productive for all primary prepositions
111. **Nisbe agreement** — adjective agrees with the nisbe, NOT the
     governed noun: *jrj-ꜥ nb* "every room-keeper"
112. **Suffix placement on nisbes** — position determines meaning:
     pronoun on governed noun vs pronoun on nisbe itself
113. **ḫrj possession** — *ḫrj* "under" → "having": *ḫrj-ḥ ḥꜣt*
     "one who has the scroll" (lector priest)
114. **Reverse nisbes** — *nfr ḥr* construction with prepositional
     nisbes: *jmt pr* = "that which the house is in" (= "will/testament")
115. **jmj-r lexicalization** — reverse nisbe *jmj-r* "overseer"
     (one in whom the mouth is) — most common Egyptian title
116. **Prepositional modifier constraint** — prepositions cannot
     directly modify nouns; must convert to nisbe form
117. **Three primary adverbs** — only *ꜥꜣ*, *rj*, *grw*; Egyptian
     uses prepositional phrases for adverbial meaning
118. **Compositional interrogative adverbs** — "how" = *mj mj*
     (prep + interrog.), "why" = *ḥr mj*; not dedicated lexemes
119. **Adverb from adjective** — three suffixes: zero (*nfr*), *-w*
     (*ꜥꜣw*), *-t* (*wrt*); or preposition + feminine adjective
120. **Temporal nouns as adverbs** — *mjn* "now," *sf* "yesterday,"
     *ḏt* "forever" can function as adverbs without change
121. **Prepositional adverb formation** — drop object; add *-j* or
     *-w* for special adverbial form
122. **Adverb position rule** — before prepositions, after
     adjectives/adverbs; never modifies nouns
123. **No comparative/superlative adverb form** — context or *r* +
     NP: *wrt r ḥt nbt* "more greatly than anything"
124. **mꜣꜥ ḫrw epithet** — "true of voice" = *nfr ḥr* construction;
     adjective + prepositional complement; standard for the justified dead
125. **Phrasal cohesion constraint** — *wꜥ jm* kept together even when
     *nb* logically modifies *wꜥ*; adjective displaced to preserve
     prepositional adverb phrase unity

---

## Lesson 8: Prepositions and Adverbs

### Definitions (Section 8.1)

Prepositions relate one thing to another; they **govern** nouns/pronouns.
When used alone (no governed noun), they function as **adverbs**.
Egyptian prepositions can have up to **three forms**:
1. Before a noun
2. With a personal pronoun
3. Adverbial (standalone)

### Primary Prepositions (Section 8.2) — CORE VOCABULARY

**1. jmjtw** — "between" (dual), "among" (plural)
- Adverbial: *jmjtw-nj* (compound with nisbe)
- Second noun introduced by preposition *r*:
  *jmjtw bꜣt nt r nhrn* "between this country and Naharina"
- Dyn. 18 variant: *r-jmjtw*

**2. jn** — agent marker (NOT a true preposition)
- Used like English "by" with passive verbs
- Always followed by noun/noun phrase, NEVER personal pronoun
- More in later lessons (verbal system)

**3. m** — "in" — THE MOST COMMON PREPOSITION

Seven (or more) distinct semantic roles:

| Role | Translation | Example |
|------|------------|---------|
| Locative | in, into | *m pr* "in the house" |
| Temporal | in, by, for, during | *m grḥ* "in/by night"; *m rnpwt 3* "for 3 years" |
| Stative | in (a state) | *m ḥtp* "in peace" |
| Material/Partitive | in, of | *m jnr* "of stone"; *wꜥ.w m jm.sn* "one of them" |
| Ablative | from, of | *prj m njwt* "emerge from the town" |
| Essive | as | *ḥꜥ m nswt* "appear as king" |
| Instrumental | with, through, by | *wrḥ m mrkrt* "anoint with oil"; *njs m rn* "call by name" |

Pronominal form: *jm* (also *ꜣm*)

→ The polysemy of *m* is the single largest disambiguation challenge.
Context (verb semantics, noun class) selects the reading.

**4. mj** — "like"; adverbially *mjj*
- Comparison: *mj šnr nṯr* "like the plan of a god"
- Conformity: *mj nt-ꜥ.f* "according to his custom"
- Equivalence: *krw mj grḥ* "day as well as night"

**5. mm(j)** — "among, amidst"
- General sense (vs *jmjtw* = specific physical position)
- *mm ꜥnḫw* "among the living"; *mm mw* "amidst the waters"

**6. n** — "to, for" (goal/benefactive)
- Adverbial: *nj*; before nouns also *nj*
- Indicates the goal of something
- "to" or "for": *nḏ ḥr.w n jsjr* "giving praise to Osiris"
- "at" (toward): *dꜣj n q ꜥrw.k* "look at your elbows"
- "in, for" (temporal): *n jbd 2* "in two months"; *n ḏt* "forever"
- "for, because of": *rmj n mr* "weep for/because of pain"
- Goal of motion when goal is a **person** (vs *r* for places)

**7. r** — "with respect to" — HIGHLY POLYSEMOUS
- Adverbial/pronominal: *jrj*
- "to, toward, at" (place): *prj r pt* "go toward the sky"
  → Goal of motion when goal is a **place** (vs *n* for persons)
- "at" (time): *r ṯr pn* "at this season"
- "to, in order to" (purpose): *r jnt ꜥrpw* "in order to get rations"
- "against": *jrj r* "act against"
- "from" (separation): *rḫ wḥꜣ r rḫ* "know the foolish from the wise"
- "than" (comparative): *nfr r ḥt nbt* "better than everything"
- "concerning, about": *ḏd r* "speak about"
- **Topicalizer** (*jr*): *jr st jsjr pw* "As for yesterday, it is Osiris"

**8. ḥꜣ** — "behind, around"
- From noun *ḥꜣ* "back of the head"
- *pḥr ḥꜣ jnb* "going around the wall"

**9. ḥnꜥ** — "with" (ACCOMPANIMENT ONLY)
- *ḥnꜥ snw.j* "together with my siblings"
- Also used as conjunction "and": *ḥꜣtj ḥnꜥ zmꜣ* "the heart and the lungs"
- **Critical distinction**: *ḥnꜥ* = accompaniment; *m* = instrumental
  (English "with" conflates both; Egyptian splits them)

**10. ḥr** — "on" (from noun *ḥr* "face, surface")
- Locative: *ḥr wꜣt* "on the path"; *ḥr kmt* "in Egypt"
- Additive: *jrj ḥꜣw ḥr nfr* "do more than well"
- Distributive: *t-ḥḏ ḥr wꜥb nb* "a loaf per priest"
- Ablative: *jꜣj ḥr jbḥyt* "come from Ibhyt"
- Causal: *ḥtp ḥr* "content at"; *mḫj ḥr* "forget about"
- Also coordination: *d ꜥ ḥr ḫyt* "stormwind and rain"

**11. ḫft** — "opposite, in accordance with"
- Spatial: *ꜥḥꜥ ḫft* "stand opposite, before" someone
- Conformity: *ḫft sḫꜣ pn* "in accordance with this writing"
- Nisbe *ḫftj* = "opponent, enemy"

**12. ḫnt** — "at the head of"
- Spatial priority: *ḫmsj ḫnt nṯrw* "sit in front of the gods"
- Superiority: *ḫnt ꜥnḫw* "at the head of the living"
- Adverbial = temporal: *ḫpr ḫntw* "happen before, previously"
- **ḫft vs ḫnt**: facing (*ḫft*) vs first-in-line (*ḫnt*)

**13. ḫr** — "near" (deference/proximity)
- Used when governed NP is higher status:
  *ḏd ḫr ḥm.f* "speak to His Incarnation"
- *ḫr nṯrw* "in the presence of the gods"
- Common phrases: *ḫr ḥm n* "during the reign of [king]";
  *jmꜣḫj ḫr* "honored with [god]"
- **Social register rule**: *n* "to" for equals/inferiors;
  *ḫr* "near" for superiors (king, gods, plural groups)

**14. ẖr** — "throughout"
- *ḫpr ẖr tꜣ* "happen throughout the land"

**15. ḫr** — "under"; adverbially *ḫrj*
- Literal: *ḥmsj ḫr ḥḏꜣw* "sit under sails"
- Carrying/having: *jw ḫr jnw* "come with tribute"
- More literal than English: *ḫꜣp ḫr jtj* "loaded with grain"

**16. ḏp** — "atop" (from *ḏpj* "head")
- *ḏp jnb* "on top of the wall"
- **ḏp vs ḥr**: both = "on/above" but *ḥr* implies closer contact
- Cultural difference: speech is *ḏp r* "atop the mouth"
  (Egyptian) vs "in the mouth" (English)

**17. ḏr** — "since, before" (from *ḏr* "limit, end")
- *ḏr rk ḫrw* "since the age of Horus"
- Adverbially: "over, finished"

### Compound Prepositions (Section 8.3)

Three formation patterns:
1. **Prep + noun**: *m ḥꜣt* "in front of," *r ḥꜣt* "to the front of,"
   *ḥr ḥꜣt* "at the front of" — all from *ḥꜣt* "front"
2. **Prep + infinitive**: *r ḏbꜣ* "in exchange for" (lit. "to replace")
3. **Adverb + prep**: *ḥrw r* "apart from" (adverb *ḥrw* + prep *r*)

Meaning is compositional — look up the major component in dictionaries.

### Object of Prepositions (Section 8.4) — KEY RULE

- Nouns/noun phrases: no change after prepositions
- **Suffix pronouns** attach directly: *ḥnꜥ.j* "with me",
  *ḥnꜥ.f* "with him", *m ḥꜣt.k* "in front of you"

Two exceptions — prepositions that CANNOT take suffix pronouns:
1. *jn* "by" — never with personal pronouns
2. *mj* "like" — uses noun *mjṯw* or nisbe *mjṯj* "likeness" instead:
   *šṯj mjṯw.j* "a farmer like me" (lit. "a farmer, my likeness")

### n with Adjectival Predicates (Section 8.5)

Subjectless adjectival sentences with *n* = English dummy "it":
- *nfr n.tn* "It is good for you"
- *bjn(j) n.j* "How bad it is for me"

→ The predicate has no explicit subject; *n* + pronoun = benefactive.

### Prepositional Nisbes (Section 8.6) — SYSTEMATIC DERIVATION

Preposition + -j/-t → adjective. Most primary prepositions have a nisbe:

| Preposition | Nisbe | Meaning | Note |
|------------|-------|---------|------|
| *m* "in" | *jmj* | inherent in | Most common |
| *mj* "like" | *mjṯj* | similar | From abstract *mjṯt* "similarity" |
| *n* "for" | *nj* | belonging to | = indirect genitive marker! |
| *r* "w.r.t." | *jrj* | pertaining to | |
| *ḥꜣ* "behind" | *ḥꜣ(j)* | surrounding | |
| *ḥnꜥ* "with" | *ḥnꜥ(j)* | accompanying | |
| *ḥr* "on" | *ḥrj* | upper, on | |
| *ḫft* "opposite" | *ḫftj* | opposing | → "enemy" |
| *ḫnt* "at head" | *ḫntj* | foremost | |
| *ḫr* "near" | *ḫrj* | adjacent | |
| *ḫr* "under" | *ḫrj* | lower, under | Same form as "adjacent"! |
| *ḏp* "atop" | *ḏpj* | standing atop | |

### Uses of Prepositional Nisbes (Section 8.7)

Two uses: modify nouns AND function as nouns.

**Compositional titles/epithets:**
- *jrj-ꜥ* "room-keeper" (lit. "one pertaining to a room")
- *ḥrj-ḥb(t)* "lector priest" (lit. "he under the festival-scroll")
- *ḫntj-jmnṯjw* "foremost of the westerners" (epithet of Osiris)
- *ḏpj-ḏw.f* "atop his mountain" (epithet of Anubis)

**Agreement rule**: adjective agrees with the NISBE, not the governed noun:
- *jrj-ꜥ nb* "every room-keeper" ✓
- *jrj ꜥ nbt* "one pertaining to every room" (different meaning!)

**Suffix pronoun placement = meaning change:**
- *jmj ḥꜣt.sn* "he who is in front of them" (pronoun on governed noun)
- *jmj.sn ḥꜣt* "their predecessor" (pronoun on nisbe)

### Special Uses of ḫrj (Section 8.8)

*ḫrj* "under" → possession ("having"):
- *ḫrj-ḥ ḥꜣt* "lector-priest" = one who HAS the scroll
- *ḫrj-nṯr* "cemetery" = place that HAS the god (lit. "under the god")
- *bw ḥr(j).f* "the place where he is" (lit. "the place under him")

### "Reverse" Nisbes (Section 8.9) — REMARKABLE AMBIGUITY

The *nfr ḥr* construction applied to prepositional nisbes:

*mdꜣt jmt pr* can mean:
1. **Normal**: "the scroll that is in the house" (*jmt* = where scroll is)
2. **Reverse**: "the scroll that the house is in" (*jmt* = what contains *pr*)

The reverse reading: *jmt pr* alone = "will, testament" (the scroll
in which one's estate/*pr* is listed) — a lexicalized reverse nisbe!

→ Same structural ambiguity as *nfr ḥr*: "good of face" vs
"the one whose face is good." The nisbe refers to the head noun
even though the governed noun bears the semantic property.

### jmj-r "Overseer" — Lexicalized Reverse Nisbe (Section 8.9 cont.)

*jmj-r* "overseer" = "the one in whom the mouth (*r*) is" — the
command-giver. Precedes the domain of oversight:
- *jmj-r pr* "steward" (overseer of the house)
- *jmj-r mšꜥ* "general" (overseer of the army)
Often spelled with tongue sign, confirming the reverse reading.

### Prepositional Phrases as Modifiers (Section 8.10)

Prepositional phrases CANNOT directly modify nouns in Egyptian.
Must convert to nisbe form:
- *nṯrw jmjw pt* "the gods in the sky" ✓ (nisbe *jmjw*)
- *\*nṯrw m pt* ✗ (bare preposition cannot modify noun)

The *n(j)* construction with suffix pronoun and nisbe *jmj*:
*ms n.f jmj* "a child of his" — uses preposition *n* + nisbe.

### Adverbs — Definitions (Section 8.11)

Adverbs indicate where, when, why, how. Can modify verbs,
adjectives, prepositions, other adverbs. Prepositional phrases
can function as adverbs.

### Primary Adverbs (Section 8.12)

Only **three** primary adverbs in Middle Egyptian:
1. *ꜥꜣ* "here"
2. *rj* "entirely, at all"
3. *grw* "also, further, any more"

→ Remarkably sparse! Egyptian relies heavily on prepositional
phrases for adverbial meaning instead of dedicated adverb lexemes.

### Interrogative Adverb (Section 8.13)

| Question | Form | Composition |
|----------|------|-------------|
| Where? | *tn(j)* / *ṯn(j)* | Primary interrogative adverb |
| Where? | *tnw* | From word for "each" |
| How? | *mj mj* | lit. "like what?" (prep + interrog.) |
| Why? | *ḥr mj* | lit. "because of what?" (prep + interrog.) |
| When? | *zy nw* | lit. "which time?" (interrog. + noun) |

→ "How" and "why" are compositional: preposition + interrogative
pronoun. Not dedicated adverb lexemes. Same parsimony as the
3-adverb inventory.

### Other Adverbs (Section 8.14)

**From adjectives** — three derivation patterns:
1. **Identical form**: *nfr* "well" (= *nfr* "good"); *ꜥšꜣ* "often" (= "many")
2. **+ w ending**: *ꜥꜣw* "greatly" (from *ꜥꜣ* "big") — *w* often omitted
3. **+ t ending**: *wrt* "very" (from *wr* "great")
4. **Preposition + feminine adj**: *r ꜥꜣt* "greatly" (lit. "w.r.t. greatness")

**Temporal nouns as adverbs**: *mjn* "now," *sf* "yesterday," *ḏt* "forever";
noun phrases: *ḥrw pn* "today," *rꜥ nb* "every day";
with prepositions: *m mjn* "today," *n ḏt* "forever"

**Reflexive adverbs**: *ḏs* "self" + suffix pronoun:
- *nswt ḏs.f* "the king himself"
- *m ḥrw.k ḏs.k* "in your own day" (lit. "in your time yourself")

### Prepositional Adverbs (Section 8.15)

Preposition without object = adverb. Special adverbial forms:
- Adding *-j*: *mj* → *mjj*, *n* → *nj*, *r* → *jrj*, *ḫr* → *ḫrj*
- Adding *-w*: *ḥnꜥ* → *ḥnꜥw*, *ḫft* → *ḫftw*, *ḫnt* → *ḫntw*
- Adding *jrj*: *ḫft jrj* "accordingly" (= *ḫftw*)

Egyptian is much freer than English in using prepositions adverbially.
English must add "there-" (*therein, therewith, therefrom*) or a
pronominal object; Egyptian just drops the object.

### Uses of Adverbs (Section 8.16)

- Adverbs modifying prepositions: **precede** the preposition
- Adverbs modifying adjectives/adverbs: **follow** the modified word:
  *ḫjr wrt* "very excellent"; *r wrt* "very greatly"
- Adverbs do NOT normally modify nouns

### Comparative and Superlative Adverbs (Section 8.17)

Like adjectives (§6.8), adverbs have **no special comparative/
superlative form**. Context determines degree. When explicit,
comparative meaning uses preposition *r*:
*wrt r ḥt nbt* "more greatly than anything"
(lit. "greatly with respect to everything")

### Essay 8: Death and the Afterlife (Cultural Context)

Mummification process: body preserved in natron, internal organs
removed (except heart), placed in four **Canopic jars** — sons of Horus:

| Son | Head | Organ |
|-----|------|-------|
| Imsety (*jmstj*) | Human | Liver |
| Hapy (*ḥpj*) | Baboon | Lungs |
| Duamutef (*dwꜣ-mwt.f*) | Jackal | Stomach |
| Qebehsenuef (*qbḥ-snw.f*) | Falcon | Intestines |

70-day mummification, burial in tomb with burial chamber below ground
and offering chapel above. "Mouth-Opening Ritual" performed by priests
to restore the dead person's senses.

Key terms:
- *ꜥnḫ* "living ba" — the deceased's animated spirit
- *ꜣḫ* "akh" — "effective one" (nonphysical afterlife form)
- *mꜣꜥ ḫrw* "justified" — lit. "true of voice"; verdict after
  weighing of the heart against *mꜣꜥt* (Maat, "proper behavior")
- "Osiris So-and-So" — standard form for addressing the deceased

→ *mꜣꜥ ḫrw* is a *nfr ḥr* construction: "true of voice" =
adjective + prepositional complement. Very common epithet.

### Exercise 8 — Test Corpus (32 sentences)

Primary source: **Tale of Sinuhe** (~20 of 32 sentences)
Also: Amenemhat, Merikare, Eloquent Peasant, Coffin Texts, Admonitions

Key vocabulary includes:
- *rḏnw* "Retjenu" (Lebanon region) — geographic
- *qdnw* "Qatna" (Syrian town) — geographic
- *mjktj* "Megiddo" (Canaanite town) — geographic
- *sḫmt* "Sekhmet" — goddess of pestilence
- *mꜣꜥt* "truth" — personified concept
- *pḥ.n.k* "you have reached" — first verbal form in exercises!

**Note 1 — Word order constraint**: In *wꜥ nb jm*, *nb* "every"
modifies *wꜥ* but follows *jm* because Egyptian prefers to keep
*wꜥ jm* ("one of them") as a cohesive phrase unit.

---

## Next Steps

1. **Phase 1 — Write the Kleis theory** with 125 axioms (Lessons 1-8):
   - `HieroglyphicSpelling` structure (axioms 1-12)
   - `MiddleEgyptianNominalGrammar` structure (axioms 13-98)
   - `MiddleEgyptianPrepositions` structure (axioms 99-125)
   - Encode non-verbal sentences as test cases
   - Implement disambiguation as Z3 Sat queries
   - Analyze real texts (Sinuhe, Eloquent Peasant, Coffin Texts)
2. **Phase 2 — Continue reading** (Lessons 9+: verbal system)
   - Adverbial sentences (Lesson 9?)
   - Negation
   - Verbal forms (*sḏm.f*, *sḏm.n.f*, etc.)
   - Participles, relative forms, infinitives
3. Publish as a paper following the Moonlight pattern
