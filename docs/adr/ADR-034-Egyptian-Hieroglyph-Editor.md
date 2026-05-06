# ADR-034: Egyptian Hieroglyph Editor Extension

**Status:** Accepted  
**Date:** 2026-04-27  
**Relates to:** ADR-005 (Visual Authoring), ADR-023 (Template Externalization), ADR-033 (Musical Score Notation)

## Context

Kleis was designed as a knowledge production substrate where "any domain that has
notation, rules, and outputs fits the same pattern." ADR-005 envisioned custom
glyphs and templates; ADR-023 externalized templates to `.kleist` files with
`svg:`, `glyph:`, and `category:` fields; ADR-033 established the precedent of
integrating non-mathematical notation (music) through external renderers.

Egyptian hieroglyphs are a compelling test of this architecture: they are a
real-world writing system with 1,000+ signs, 2D composition (quadrat blocks),
and well-established scholarly classification (the Gardiner Sign List).

## Decision

Extend the equation editor to support Egyptian hieroglyphs as a fully integrated
domain using the existing `.kleist` template system and Typst rendering pipeline.

### Key architectural choices:

1. **SVG-via-Typst rendering:** Each hieroglyph template produces a Typst
   `#box(image("...svg", height: 1.5em))` call. The existing `/api/render_typst`
   pipeline compiles this to SVG output. This required extending `MinimalWorld`
   to support file reads (previously it rejected all file access).

2. **Data-driven palette:** The Egyptian tab loads glyph metadata from
   `GET /api/templates` rather than hardcoding buttons in HTML. This is the first
   palette tab to be API-driven, establishing the pattern for ADR-023's "Future
   Work" goal of fully dynamic palette generation.

3. **Gardiner categorization:** Templates use `category: "egyptian_X_desc"` for
   category filtering. The palette UI provides both a category dropdown and a
   search-by-code input.

4. **Quadrat composition via Typst grid:** 2D hieroglyph blocks use Typst's
   `#grid()` and `#stack()` layout functions, which render correctly inside math
   mode. Three composition templates are provided: `quadrat_h`, `quadrat_v`,
   and `quadrat_2x2`.

5. **Phase 1 scope:** 225 core Gardiner signs from PharaLex (MIT-licensed SVGs).
   The full set of 8,332 glyphs can be added incrementally.

## Consequences

- **Validates Level 3 philosophy:** Hieroglyphs use the same
  Notation → Rules → Verification → Output pattern as mathematics.
- **MinimalWorld now supports file I/O:** Any future template that references
  external files (images, data) benefits from this change.
- **API-driven palette pattern:** New domains can be added by creating a
  `.kleist` file and SVG assets — no HTML changes required for the palette.
- **Repo size:** 225 SVG files add ~2 MB. Full set (8,332 files) would add
  ~60 MB; consider git submodule or lazy download for Phase 2.

## Composability Validation

The strongest validation of this architecture came from testing: hieroglyphs
compose with the existing `Matrix` template without any special-casing. A 2×3
matrix of hieroglyphs produces this AST:

```json
{
  "Operation": {
    "name": "Matrix",
    "args": [
      {"Const": "2"}, {"Const": "3"},
      {"List": [
        {"Operation": {"name": "C4", "args": []}},
        {"Operation": {"name": "A2", "args": []}},
        {"Operation": {"name": "G17", "args": []}},
        {"Operation": {"name": "P5", "args": []}},
        {"Operation": {"name": "Y3", "args": []}},
        {"Operation": {"name": "K1", "args": []}}
      ]}
    ]
  }
}
```

Each hieroglyph is a zero-argument `Operation` node. The renderer looks up its
Typst template (producing `#box(image(...))`) and the Matrix template arranges
them in a grid. The equation editor does not know or care that these are
Egyptian glyphs — they are operations with templates, composable with any
structural element.

This is significant because **2D matrix layout is how Egyptian was actually
written.** Hieroglyphs were arranged in rectangular "quadrat" blocks. The
existing `Matrix` template serves as an ad hoc quadrat. Purpose-built
containers — cartouches (royal name enclosures), serekhs (palace facades) —
are future templates that will compose identically.

### No hardcoding principle

The type checker reports "Unknown operation: 'A1'" because no Kleis structure
defines these operations yet. This was deliberately left as-is rather than
adding hieroglyph-specific checks to `server.rs`. The correct fix is to define
a `Hieroglyph` type in a `.kleis` structure file:

```kleis
structure EgyptianGlyphs {
    sort Hieroglyph
    // Each Gardiner sign is a nullary operation producing a Hieroglyph
    operation A1 : Hieroglyph
    operation A2 : Hieroglyph
    // ...
}
```

The type checker would then infer `Matrix(2, 3, Hieroglyph)` for the example
above. No Rust code changes needed — the template system and type system both
extend through `.kleis` files.

## Type System as Philological Engine

The type system's role extends beyond simple classification. If hieroglyphic
signs are given linguistically meaningful types — phonograms, logograms, and
determinatives — then Hindley-Milner inference becomes a *translation engine*.

Consider a structure that encodes Middle Egyptian sign classification:

```kleis
structure MiddleEgyptian {
    sort Phonogram
    sort Logogram
    sort Determinative
    sort Word

    // Uniliteral phonograms
    operation M17 : Phonogram    // reed = "i"
    operation D21 : Phonogram    // mouth = "r"
    operation G43 : Phonogram    // quail chick = "w"

    // Logograms
    operation N5  : Logogram     // sun = "Ra"

    // Determinatives
    operation A1  : Determinative  // seated man (human activity)
    operation G7  : Determinative  // falcon on standard (divine)

    // Composition rules
    operation word : Phonogram -> Phonogram -> Determinative -> Word

    // Grammatical constraints
    axiom determinative_final(p1: Phonogram, p2: Phonogram, d: Determinative) {
        // Determinatives must appear at the end of a word
        word(p1, p2, d) : Word
    }
}
```

With these types, the equation editor's type indicator doesn't just classify —
it *reads*. A sequence `M17, D21, A1` placed in a quadrat would infer as
`Word` and the type trace reveals the phonetic value `i-r + human determinative`
= "to do, to make" (*iri*). The inferred type **is** the translation.

This is fundamentally different from how a large language model handles
ancient text. An LLM produces statistically plausible translations that may
violate the grammar of Middle Egyptian — it has no mechanism to reject
ill-formed sign sequences. The Kleis approach enforces grammaticality by
construction: the template system only permits structurally valid compositions,
and the type checker rejects sequences that violate the sign classification
rules. An ungrammatical hieroglyphic expression fails to type-check. The error
message is the grammar lesson.

Z3 verification adds a further dimension. Philological propositions become
satisfiability queries:

- "Does this sign sequence satisfy the phonetic complement rule?" — **Verify**
- "Can this determinative appear with a divine logogram?" — **Sat**
- "Is there a valid reading of this damaged cartouche given the surviving
  signs?" — **Sat** with partial constraints

The same `Verify` and `Sat` buttons that check mathematical propositions in the
equation editor would check philological ones. A scholar could formalize the
rules of Middle Egyptian grammar as axioms and verify inscriptions against them
mechanically. Z3 would produce counterexamples for invalid readings and
witnesses for valid ones.

This is the Kleis philosophy applied to philology: Notation (hieroglyphic
templates) + Rules (grammatical axioms in structures) + Verification (Z3) +
Output (Typst rendering). The universal formula holds.

## Middle Egyptian Theory: The Moonlight Pattern

The Equation Editor provides the **visual vocabulary** — 225 Gardiner signs
rendered as SVGs, composable in matrices. But the intellectual content — grammar
rules, sign classification, verification of real inscriptions — requires a
standalone Kleis theory that is independent of the editor's rendering pipeline.

The architecture follows the same four-layer pattern established by the
Moonlight Sonata paper (*"The Beauty is in the Skolems"*):

| Layer | Music | Egyptian |
|-------|-------|----------|
| **Theory** | `stdlib/theories/tonal_harmony.kleis` | `stdlib/theories/middle_egyptian.kleis` |
| **Data** | `moonlight_sonata.kleis` (score AST) | `inscription.kleis` (sign sequences) |
| **Analysis** | `moonlight_analysis.kleis` (axiom checks) | `egyptian_analysis.kleis` (grammar checks) |
| **Paper** | `moonlight_paper.kleis` (prose + proofs) | `egyptian_paper.kleis` (prose + proofs) |

### Theory layer (`middle_egyptian.kleis`)

Domain-independent grammar axioms, analogous to `pitch_class` and
`check_tonic_opening`:

- **Sign function ADT:** `Ideogram | Phonogram(sound) | Determinative(category)`
- **Phonetic complement rule:** uniliteral following a biliteral with matching
  sound is a complement, not an additional consonant
- **Determinative position:** determinatives are always word-final
- **Complement disambiguation:** when a sign has multiple sound values,
  the complement selects the intended reading
- **Ideogram marker:** stroke + no complement → ideogram reading
- **Weak consonant optionality:** ꜣ, j, y, w may be omitted in spelling
- **Grouping axioms:** tall signs alone, others in quadrats, top-to-bottom

Each axiom checker returns `-1` (SAT) or the index of the first violation,
exactly as in `tonal_harmony.kleis`.

### Data layer (`inscription.kleis`)

Specific inscriptions encoded as typed ASTs. The base types are:

- **Sign:** Gardiner code + shape + sound value(s) — analogous to `Pitch`
- **Word:** ordered list of signs with assigned roles — analogous to `Measure`
- **Inscription:** list of words — analogous to `Score`

Candidate first specimens: the Ptolemy cartouche (Champollion's original
decipherment, a constraint-satisfaction problem solved by hand) or the
Senwosret III border stela from Allen's Exercise 2.

### Analysis layer (`egyptian_analysis.kleis`)

Runs grammar axioms against inscription data, producing machine-checked
verdicts:

- "Does this cartouche satisfy the phonetic complement rule?" — **SAT**
- "Is this sign functioning as ideogram or phonogram?" — **type inference**
- "What are all valid readings of this damaged text?" — **Z3 Sat**

### Key distinction: theory vs. rendering

The theory operates on the AST of sign sequences. It never references SVGs,
Typst templates, or the Equation Editor. The Equation Editor is the rendering
layer — the analog of LilyPond for music. Just as `tonal_harmony.kleis` never
mentions LilyPond, `middle_egyptian.kleis` will never mention the editor or
its templates.

The Equation Editor makes hieroglyphs *visible*. The theory makes them
*verifiable*.

## Amendment: Template Metadata and Quadrat Validation

**Date:** 2026-04-27  
**Branch:** `feature/egyptian-metadata-validation`

### Problem

The existing `.kleist` template format has only fixed fields (`pattern`, `typst`,
`category`, etc.). Egyptian hieroglyphs require domain-specific metadata — sign
shape, sign type, phonetic value — that does not belong in the fixed schema
because it is meaningless for mathematics, music, or physics templates.

Without this metadata, quadrat composition cannot be validated: the editor has
no way to know that placing two tall signs side-by-side is philologically
illegal.

### Decision: Generic `metadata` HashMap on `TemplateDefinition`

Add a single new field to `TemplateDefinition` in `src/kleist_parser.rs`:

```rust
pub metadata: HashMap<String, String>
```

The parser's `parse_template()` method gains a new match arm before the
catch-all error arm:

```rust
Token::Identifier(key) => {
    let key = key.clone();
    self.advance()?;
    self.expect(Token::Colon)?;
    let value = self.expect_string()?;
    template.metadata.insert(key, value);
}
```

Existing keyword-based fields (`pattern`, `typst`, `category`, etc.) continue
to match first via their `Token::Keyword(...)` arms. The new arm only fires
for identifiers that are not Kleis keywords. For existing math/music/physics
templates that use only keyword fields, the `metadata` HashMap remains empty.

### Backward Compatibility

This change is strictly additive:

1. **Existing `.kleist` files** — no modification needed; `metadata` is empty
2. **`TemplateDefinition` consumers** — existing code reads `template.typst`,
   `template.pattern`, etc., which are unchanged
3. **Error handling** — truly invalid tokens still hit the `_ =>` error arm
4. **`/api/templates`** — `TemplateInfo` is extended with an optional
   `metadata` field; clients that ignore it are unaffected

### Sign Classification Metadata

Each Egyptian glyph template gains three metadata keys:

| Key | Values | Purpose |
|-----|--------|---------|
| `sign_shape` | `Tall`, `Flat`, `Small` | Governs quadrat slot eligibility (Axiom 6) |
| `sign_type` | `Uni`, `Bi`, `Tri`, `Det` | Linguistic classification |
| `sound` | e.g. `"m"`, `"mn"`, `"nfr"` | Phonetic value (omitted for determinatives) |

Example annotated template:

```kleist
@template G17 {
    pattern: "G17()"
    typst: "#box(image(\"static/glyphs/egyptian/G17.svg\", height: 1.5em))"
    unicode: "G17"
    category: "egyptian_G_bird"
    glyph: "G17"
    svg: "static/glyphs/egyptian/G17.svg"
    sign_shape: "Flat"
    sign_type: "Bi"
    sound: "m"
}
```

Shape defaults by Gardiner category (refined per-sign where needed):

| Category | Typical Shape | Notes |
|----------|---------------|-------|
| A (Man) | Tall | Standing/seated figures |
| B (Woman) | Tall | Standing/seated figures |
| C (Gods) | Tall | Anthropomorphic deities |
| D (Body parts) | Mixed | D21 (mouth) = Small; D58 (foot) = Tall |
| G (Birds) | Mixed | G17 (owl) = Flat; G5 (falcon) = Tall |
| M (Plants) | Mixed | M17 (reed) = Tall; M23 (sedge) = Small |
| N (Sky/Earth) | Flat | N1 (sky), N35 (water) |
| O (Buildings) | Flat | O1 (house), O4 (shelter) |
| Z (Strokes) | Small | Z1 (stroke), Z2 (plural strokes) |
| Aa (Unclassified) | Small | Aa1, Aa2 |

### Quadrat Validation Rules

Server-side validation in `render_editor.rs` enforces Axiom 6 when rendering
quadrat composition templates:

| Composition | Rule | Rationale |
|-------------|------|-----------|
| `quadrat_h(A, B)` | Neither A nor B is `Tall` | Two tall signs cannot sit side-by-side |
| `quadrat_v(A, B)` | At least one of A, B is `Flat` or `Small` | Vertical stack needs a compact sign |
| `quadrat_2x2(tl, tr, bl, br)` | All four are `Small` | 2x2 grid only works with compact signs |

Validation returns a structured warning in the render response — not a hard
error. The editor can display the warning (e.g., red outline on the quadrat)
while still rendering the composition. This matches philological practice:
non-standard groupings occasionally appear in real inscriptions.

### API Extension

`TemplateInfo` in `server.rs` gains:

```rust
pub metadata: HashMap<String, String>,
```

The client uses `metadata.sign_shape` for pre-insertion validation (blocking
illegal drops) and `metadata.sign_type` / `metadata.sound` for tooltip
display and future linguistic analysis.

## Alternatives Considered

- **Option C (Dedicated page):** A separate HTML page with client-side SVG
  composition. Rejected because Typst grid/stack proved sufficient for quadrat
  layout, and full integration validates the extensibility thesis.
- **Unicode-only rendering:** Egyptian hieroglyphs have Unicode block U+13000,
  but font support is poor and inconsistent. SVG rendering is more reliable.
- **Hardcoded hieroglyph type detection:** Adding `is_egyptian_hieroglyph()`
  to `server.rs` to bypass the type checker. Rejected because it violates the
  extensibility principle — every new domain would require Rust code changes.
  The `.kleis` structure approach is the right solution.
