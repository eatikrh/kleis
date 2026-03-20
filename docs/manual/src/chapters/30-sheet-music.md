# Sheet Music

Kleis can generate publication-quality sheet music using the same pipeline as
academic papers. A musical score is defined as structured data, compiled to
[LilyPond](https://lilypond.org/) source, and rendered to PDF.

```
Score -> compile_score -> .ly -> lilypond -> .pdf
```

This mirrors the paper pipeline:

```
ArxivPaper -> compile_arxiv_paper -> .typ -> typst compile -> .pdf
```

## Quick Start

### Prerequisites

1. **Kleis** compiled and in PATH
2. **LilyPond** for engraving: `brew install lilypond`

### Your First Score (5 minutes)

Create a file `my_score.kleis`:

```kleis
import "stdlib/templates/sheet_music.kleis"

define my_score = solo_score(
    "My First Score",
    "Composer Name",
    Treble,
    KeySig(C, "major"),
    TimeSig(4, 4),
    Cons(Measure(Cons(n(C, Quarter), Cons(n(D, Quarter),
         Cons(n(E, Quarter), Cons(n(F, Quarter), Nil))))),
    Cons(Measure(Cons(n(G, Half), Cons(n(C, Half), Nil))),
    Nil))
)

example "compile" {
    let ly = compile_score(my_score) in
    out(typst_raw(ly))
}
```

Generate PDF:

```bash
kleis test --raw-output --example compile my_score.kleis > my_score.ly
lilypond my_score.ly
open my_score.pdf
```

## Data Types

The template defines types for two layers of musical notation:

### Layer 1: Syntax — Symbolic Tokens

| Type | Constructors | Description |
|------|-------------|-------------|
| `NoteName` | `C`, `D`, `E`, `F`, `G`, `A`, `B` | Note letter names |
| `Accidental` | `Natural`, `Sharp`, `Flat` | Accidentals |
| `Pitch` | `P(NoteName, Accidental, ℤ)` | Full pitch (octave 4 = middle C) |
| `Duration` | `Whole`, `Half`, `Quarter`, `Eighth`, `Sixteenth`, `Dotted(Duration)`, `Triplet(Duration)` | Note durations |
| `Clef` | `Treble`, `Bass`, `Alto`, `Tenor` | Staff clefs |
| `KeySig` | `KeySig(NoteName, String)` | Key signature (root + "major"/"minor") |
| `TimeSig` | `TimeSig(ℤ, ℤ)` | Time signature (beats, beat unit) |

### Layer 1: Events and Annotations

| Type | Constructors | Description |
|------|-------------|-------------|
| `Event` | `Note(Pitch, Duration)`, `Rest(Duration)`, `Chord(List, Duration)`, `Marked(Event, List)` | Things that happen in time |
| `Articulation` | `Staccato`, `Accent`, `Tenuto`, `Fermata`, `Marcato` | Articulation marks |
| `Annotation` | `Tie`, `SlurStart`, `SlurEnd`, `Artic(Articulation)`, `Dyn(Dynamic)` | Attachments to events |
| `Dynamic` | `PPP`, `PP`, `Piano`, `MP`, `MF`, `Forte`, `FF`, `FFF` | Volume markings |

### Layer 2: Structure

| Type | Constructors | Description |
|------|-------------|-------------|
| `Measure` | `Measure(List)` | List of events |
| `StaffContent` | `Staff(Clef, KeySig, TimeSig, List)` | One staff with its measures |
| `ScoreMeta` | `ScoreMeta(String, String, String)` | Title, composer, subtitle |
| `Score` | `Score(ScoreMeta, List)` | Metadata + list of staves |

## Convenience Constructors

The template provides shortcuts for common operations:

```kleis
// Notes
n(C, Quarter)                  // C4 quarter note
no(G, 5, Eighth)               // G5 eighth note
ns(F, 4, Half)                 // F#4 half note
nb(B, 3, Quarter)              // Bb3 quarter note

// Rests
r(Quarter)                     // quarter rest

// Measures
m(Cons(n(C, Quarter), ...))    // measure from event list

// Annotations
slur_start(n(C, Quarter))      // c'4(
slur_end(n(E, Quarter))        // e'4)
tied(n(D, Half))               // d'2~
staccato(n(G, Eighth))         // g'8-.
accent(n(C, Quarter))          // c'4->
fermata(n(G, Dotted(Half)))    // g'2.\fermata
dyn(n(C, Quarter), Forte)      // c'4\f

// Score constructors
solo_score(title, composer, clef, key, time, measures)
piano_score(title, composer, key, time, treble_measures, bass_measures)
```

## Examples

### Solo Instrument: Ode to Joy

A 16-bar excerpt of Beethoven's theme for single voice:

```kleis
import "stdlib/templates/sheet_music.kleis"

define m1 = m(Cons(n(E, Quarter), Cons(n(E, Quarter),
         Cons(n(F, Quarter), Cons(n(G, Quarter), Nil)))))
define m2 = m(Cons(n(G, Quarter), Cons(n(F, Quarter),
         Cons(n(E, Quarter), Cons(n(D, Quarter), Nil)))))
// ... more measures ...

define ode = solo_score("Ode to Joy", "Ludwig van Beethoven",
    Treble, KeySig(C, "major"), TimeSig(4, 4), ode_measures)

example "compile" {
    out(typst_raw(compile_score(ode)))
}
```

See the full example: `examples/music/ode_to_joy.kleis`

### Piano: Minuet in G Major

A two-staff arrangement demonstrating slurs, dynamics, and key signatures:

```kleis
import "stdlib/templates/sheet_music.kleis"

define fis(oct, dur) = Note(P(F, Sharp, oct), dur)

// Right hand with slurs and dynamics
define rh1 = m(Cons(no(D, 5, Quarter),
           Cons(slur_start(no(G, 4, Eighth)),
           Cons(no(A, 4, Eighth),
           Cons(no(B, 4, Eighth),
           Cons(slur_end(no(C, 5, Eighth)), Nil))))))

// Left hand with walking bass
define lh1 = m(Cons(no(G, 3, Half), Cons(no(A, 3, Quarter), Nil)))

// Assemble as piano score (auto-detects two staves -> PianoStaff)
define minuet = piano_score(
    "Minuet in G Major",
    "Christian Petzold (attr. J.S. Bach)",
    KeySig(G, "major"),
    TimeSig(3, 4),
    treble_measures,
    bass_measures
)

example "compile" {
    out(typst_raw(compile_score(minuet)))
}
```

The `piano_score` constructor creates two staves. When `compile_score` detects
exactly two staves, it wraps them in a LilyPond `\new PianoStaff` block,
which draws a brace connecting them.

See the full example: `examples/music/minuet_in_g.kleis`

## Measure Completeness Verification

The template includes duration arithmetic so you can verify that measures
add up correctly:

```kleis
// Duration values (in sixteenths of a whole note)
duration_value(Whole)           // 16
duration_value(Half)            // 8
duration_value(Quarter)         // 4
duration_value(Dotted(Quarter)) // 6

// Check a measure sums to its time signature
let ts = TimeSig(4, 4) in
assert(measure_duration(my_measure) = measure_expected_duration(ts))

// Works for any time signature
let waltz = TimeSig(3, 4) in
assert(measure_expected_duration(waltz) = 12)  // 12 sixteenths
```

This catches a common notation error at compile time rather than when
LilyPond warns about it.

## LilyPond Output

The compiler translates Kleis data types to LilyPond syntax:

| Kleis | LilyPond | Description |
|-------|----------|-------------|
| `P(C, Natural, 4)` | `c'` | Middle C |
| `P(F, Sharp, 5)` | `fis''` | F# one octave above middle C |
| `P(B, Flat, 3)` | `bes` | Bb below middle C |
| `Note(P(C,Natural,4), Quarter)` | `c'4` | Quarter note |
| `Rest(Eighth)` | `r8` | Eighth rest |
| `Chord([P(C,4), P(E,4), P(G,4)], Quarter)` | `<c' e' g'>4` | C major chord |
| `slur_start(n(C, Quarter))` | `c'4(` | Begin slur |
| `dyn(n(C, Quarter), Forte)` | `c'4\f` | Dynamic marking |

## Generating PDF

### One-liner

```bash
kleis test --raw-output --example compile my_score.kleis > my_score.ly && lilypond my_score.ly && open my_score.pdf
```

### What `--raw-output` Does

- Suppresses test banners (the checkmark lines)
- `typst_raw()` in the code produces unquoted output
- Together they produce clean LilyPond source on stdout

### Bonus: MIDI

LilyPond also generates a MIDI file alongside the PDF (because the
template includes `\midi { }` in the layout block). You can play it
with any MIDI player.

## Architecture

The sheet music template follows the same pattern as the arXiv paper
template (`stdlib/templates/arxiv_paper.kleis`):

1. **Data types** define the domain vocabulary
2. **Compiler functions** translate to the target format
3. **`out(typst_raw(...))`** emits the output
4. **External tool** renders to PDF

No Rust code changes are needed. The template is pure Kleis.

## Future Directions

The current template covers Layers 1-2 (syntax and structure). Future
phases will add:

- **Layer 3: Semantics** — Counterpoint rules, harmonic analysis as
  axioms verified by Z3. "Does this passage satisfy voice-leading
  constraints?" becomes a machine-checked question.
- **Layer 4: Performance** — MIDI realization constraints. "Does this
  performance satisfy the score's articulation intent?"
- **Layer 5: Rendering** — Engraving rules formalized as axioms.
  Spacing, beam grouping, collision avoidance — verified, not heuristic.
- **Native Typst rendering** — Once engraving rules are formalized,
  a Typst backend could match LilyPond quality without the external
  dependency.

See ADR-033 for the full architectural roadmap.

---

-> [Previous: Document Generation](./23-document-generation.md)
