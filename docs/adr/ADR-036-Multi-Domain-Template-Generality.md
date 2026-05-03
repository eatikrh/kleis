# ADR-036: Multi-Domain Template Generality

**Status:** Accepted  
**Date:** 2026-05-03  
**Relates to:** ADR-005 (Visual Authoring), ADR-006 (Template-Grammar Duality),
ADR-023 (Template Externalization), ADR-034 (Egyptian Hieroglyph Editor),
ADR-035 (Multi-Domain Template Compiler)

## Context

The Kleis Equation Editor was originally built for mathematical notation. ADR-034
extended it to Egyptian hieroglyphs. ADR-035 identified three engine fixes needed
to make domain extension purely data-driven (no Rust, no JavaScript — only
`.kleist` templates and assets).

Before implementing those fixes, we investigated whether the template model
generalizes across fundamentally different visual-notation domains. We analyzed
four domains in depth:

1. **Mathematics** (implemented) — operators, Greek letters, fractions, matrices,
   integrals, summations
2. **Egyptian hieroglyphs** (partially implemented) — Gardiner sign list, quadrat
   compositions, horizontal/vertical stacking
3. **Electronic circuits** (investigated) — passive/active components, standard
   topologies, IEC/IEEE symbol standards
4. **Chemistry / molecular structures** (investigated) — elements, functional
   groups, skeletal formulas, reaction schemes

The investigation included studying:
- [KiCad](https://www.kicad.org/) schematic capture architecture (s-expression
  file format, symbol library with pin model, netlist derivation from wires and
  junctions)
- [KiCad symbol library format](https://dev-docs.kicad.org/en/file-formats/sexpr-symbol-lib/) —
  how components declare graphic primitives + typed pins
- [KiCad schematic format](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/) —
  how symbols are instantiated and connected
- [Zap](https://zap.grangelouis.ch/) (v0.6.0) — Typst circuit drawing package,
  IEC/IEEE standard symbols, named anchors for wiring
- [alchemist](https://typst.app/universe/package/alchemist/) (v0.1.9) — Typst
  skeletal formula package based on CeTZ, Lewis structures, stereochemistry
- [molchemist](https://typst.app/universe/package/molchemist) (v0.1.1) — Typst
  .mol/.sdf parser that renders via alchemist
- SVG component libraries: Acheron Project (IEEE/ANSI 315-1975), upb-lea
  Inkscape electrical symbols, Bioicons molecular icons, NIAID BioArt

## Decision

### Finding: The Atom-Molecule Decomposition

Every visual-notation domain decomposes into two kinds of templates:

- **Atoms** — zero-argument templates that render a single visual symbol
- **Molecules** — composition templates with placeholder slots for atoms or values

| Domain    | Atoms                                     | Molecules                                                       |
|-----------|-------------------------------------------|-----------------------------------------------------------------|
| Math      | `alpha`, `pi`, `infinity`                 | `frac({num},{den})`, `integral({lo},{hi},{body})`, `matrix(...)` |
| Egyptian  | Gardiner glyphs (A1, D21, N35...)         | `quadrat_pair_h({left},{right})`, `quadrat_pair_v({top},{bot})` |
| Circuits  | resistor, capacitor, op-amp symbols       | `voltage_divider({r1},{r2},{vin})`, `rc_filter({r},{c})`        |
| Chemistry | element symbols, functional groups (-OH)  | `benzene({subs})`, `ester({r1},{r2})`, `reaction({lhs},{rhs})`  |

The Kleis `.kleist` template system already provides both:

- Zero-arg templates: `@template` with no `pattern:` or a parameterless pattern
- Composition templates: `@template` with `pattern: name({left}, {right})`
  where `{left}`, `{right}` are placeholder slots filled by substitution

The engine does not need to know about domains. It provides four generic services
(as formalized in ADR-035):

1. **Substitution** — walk the AST, look up template, replace placeholders
2. **Transparent labeling** — attach Typst labels for position tracking
3. **Generic slot validation** — match `slot_type`/`accepts` metadata strings
4. **Palette serving** — deliver `@palette` structure and template ASTs

### Finding: Every Domain Has a Typst Rendering Backend

Each domain maps to a mature Typst package for compositional rendering:

| Domain    | Typst rendering backend                                     | For molecules |
|-----------|-------------------------------------------------------------|---------------|
| Math      | Typst built-in math mode (`$ frac(a, b) $`)                | Composition via math syntax |
| Egyptian  | Typst layout primitives (`#grid`, `#stack`, `#image`)       | Spatial arrangement of glyph SVGs |
| Circuits  | [Zap](https://zap.grangelouis.ch/) `#zap.circuit({...})`   | Complete schematics with wiring |
| Chemistry | [alchemist](https://typst.app/universe/package/alchemist/) `#skeletize({...})` | Skeletal formulas with bonds |

Templates generate the appropriate Typst code for their domain. The engine
compiles whatever Typst the template produces — it does not need to know
which package is being used.

### Finding: Dual Rendering Strategy (SVGs + Typst Packages)

Atom and molecule templates use different rendering approaches:

**Atoms (standalone symbols):** SVG images stored in `static/glyphs/<domain>/`,
referenced via `#image(...)` in Typst templates. This is the pattern already
established by Egyptian hieroglyphs (`static/glyphs/egyptian/*.svg`).

SVG sources by domain:

| Domain    | SVG sources |
|-----------|-------------|
| Egyptian  | Unicode hieroglyph renderings, custom sign drawings |
| Circuits  | [Acheron Project](https://github.com/Acheron-Project) (IEEE/ANSI 315-1975), [upb-lea](https://github.com/upb-lea/Inkscape_electric_symbols), [Wikimedia electrical symbols](https://commons.wikimedia.org/wiki/File:Electrical_symbols_library.svg) |
| Chemistry | [Bioicons](https://bioicons.com/) molecules section, [NIAID BioArt](https://www.niaid.nih.gov/news-events/bioart), PubChem exports |

**Molecules (compositions):** Typst package calls that produce proper domain
output — wired circuit schematics (Zap), bonded skeletal formulas (alchemist),
laid-out quadrats (Typst grid/stack). SVGs arranged in a grid cannot produce
wires, junctions, or bonds — the Typst packages handle the domain-specific
visual composition.

This dual approach already exists in math: standalone symbols (`alpha` renders
as a font glyph) vs. compositional structures (`frac` generates Typst math
layout code).

### Circuit Domain: Structured Composition (Level 1)

Circuits present a unique challenge: real circuits are **graphs** (components
connected by wires to shared nodes), while our Editor AST is a **tree**
(`EditorNode` with parent-child nesting).

KiCad's architecture confirms this: its schematic format has symbols placed at
positions, connected by wires (point-to-point coordinate lists) and junctions
(where wires meet). The electrical connectivity (netlist) is derived from
wire/pin overlap — fundamentally graph-based.

Three levels of circuit support are possible:

| Level | Model | AST requirement | Scope |
|-------|-------|-----------------|-------|
| **1: Structured composition** | Pre-defined topologies with value slots | Tree (current) | This ADR |
| 2: Netlist-based | Circuits as Kleis programs, auto-layout rendering | Tree (Kleis AST) | Future |
| 3: Graph editor | KiCad-like 2D canvas with drag-and-drop wiring | Graph (new AST) | Out of scope |

Level 1 works within our current tree AST. Topology templates are complete Zap
circuit blocks where placeholder slots accept component labels and values:

```typst
// What a "voltage_divider" template generates
#import "@preview/zap:0.6.0"
#zap.circuit({
  import zap: *
  vsource("v1", (0, 0), (0, 4), label: ${vin}$)
  resistor("r1", "v1.out", (rel: (4, 0)), label: ${r1}$)
  resistor("r2", "r1.out", (rel: (0, -4)), label: ${r2}$)
  wire("r2.out", "v1.in")
  ground("g1", (2, 0))
})
```

The user clicks "Voltage Divider" in the palette, fills in R1, R2, Vin values,
and gets a properly rendered schematic — without knowing Zap syntax. Same
workflow as clicking "Fraction" and filling numerator/denominator.

Zap provides: resistors (IEC + IEEE variants), capacitors, inductors, diodes
(standard, Zener, tunnel, Schottky, LED, photodiode), BJT/MOSFET/JFET
transistors, op-amps, voltage/current sources (DC, AC, triangle, sawtooth,
square), logic gates (AND, OR, XOR, NOT with IEC and IEEE variants), switches,
fuses, transformers, instruments (voltmeter, ammeter, ohmmeter, wattmeter),
MCU blocks, flip-flops, and custom symbol support.

### Chemistry Domain: Skeletal Formula Templates

Chemistry molecule templates generate alchemist `#skeletize({...})` blocks:

```typst
// What a "benzene" template generates
#import "@preview/alchemist:0.1.9": *
#skeletize({
  cycle(6, {
    single(); double(); single(); double(); single(); double()
  })
})
```

alchemist provides: single/double/triple bonds, branches, cycles (arbitrary
ring size), cram bonds (stereochemistry), Lewis structures, parenthesis groups,
resonance operators, and CeTZ interop for annotations.

The [molchemist](https://typst.app/universe/package/molchemist) package can
render directly from .mol/.sdf files, offering three modes: full (all atoms
and bonds), abbreviated (hide carbon backbone), and skeletal (zigzag lines +
heteroatoms only).

### What This ADR Does NOT Cover

- **ADR-035 engine fixes** — zero-arg tracking, mode-aware UUID wrapping,
  generic validation, data-driven palettes. These are prerequisite
  implementation work documented separately.
- **Level 2 circuit support** — circuits as Kleis programs rendered as
  schematics. This is orthogonal to the visual editor; circuits expressed in
  `.kleis` files are regular Kleis programs that happen to produce circuit
  output.
- **Level 3 graph editing** — KiCad-like 2D canvas with arbitrary wiring.
  Requires a graph AST. Deferred indefinitely.
- **Quantum circuit templates** — a natural fifth domain (gates on qubit lines,
  inherently sequential/tree-structured). Not yet investigated.

## Consequences

### Positive

1. **The template model generalizes.** Four fundamentally different domains
   (symbolic math, pictographic writing, circuit schematics, molecular
   structures) all decompose into atoms + molecules expressible as `.kleist`
   templates. No domain required changes to the decomposition model.
2. **Every domain has a Typst rendering backend.** The Typst ecosystem provides
   mature packages for each domain. Templates generate domain-appropriate Typst;
   the engine compiles it.
3. **The ADR-035 engine fixes are confirmed as sufficient.** No additional
   engine changes were identified by the circuit or chemistry analysis.
4. **The visual editor value proposition extends to all domains.** "Click
   buttons, get Typst/LaTeX" works for circuit schematics and molecular
   structures, not just equations.
5. **Implementation is incremental.** Each domain is an independent set of
   `.kleist` files and SVG assets. Domains do not interact or conflict.

### Negative

1. **Level 1 circuit support is limited to pre-defined topologies.** Users
   cannot compose arbitrary circuits — only fill values into template slots.
   This is useful for documentation and teaching but not for design work.
2. **Typst package dependencies.** Circuit templates require Zap; chemistry
   templates require alchemist. These are external packages with their own
   release cycles.
3. **SVG asset curation.** Each domain needs a curated set of SVG files for
   atom templates. These must be sourced, standardized, and maintained.

### Neutral

1. **No engine changes in this ADR.** This is a design validation, not an
   implementation decision. The implementation work is in ADR-035.
2. **The atom-molecule vocabulary is a useful design language** for discussing
   template architecture across the team and in documentation.

## References

- ADR-035: Multi-Domain Template Compiler — engine fixes for multi-domain support
- `docs/equation-editor-architecture.md` — complete Equation Editor codebase analysis
- [Zap documentation](https://zap.grangelouis.ch/) — Typst circuit drawing (v0.6.0)
- [alchemist on Typst Universe](https://typst.app/universe/package/alchemist/) — skeletal formulas (v0.1.9)
- [molchemist on Typst Universe](https://typst.app/universe/package/molchemist) — .mol/.sdf rendering (v0.1.1)
- [KiCad schematic format](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/) — studied for circuit architecture
- [KiCad symbol library format](https://dev-docs.kicad.org/en/file-formats/sexpr-symbol-lib/) — studied for component model
