# ADR-035: Multi-Domain Template Compiler Architecture

**Status:** Proposed  
**Date:** 2026-04-27  
**Relates to:** ADR-005 (Visual Authoring), ADR-006 (Template-Grammar Duality),
ADR-009 (Structural Editor), ADR-023 (Template Externalization), ADR-034
(Egyptian Hieroglyph Editor)

## Primary Requirement

**Extending the Equation Editor to a new domain must not change any Rust or
JavaScript code.** Adding Egyptian hieroglyphs, organic chemistry, electronic
circuits, quantum circuits, or any future notation system requires only:

1. `.kleist` template files (rendering rules, metadata, palette entries)
2. Asset files (SVGs, images) if the domain uses them

No Rust. No JavaScript. No recompilation. No new endpoints. The engine is a
generic compiler; templates are its production rules.

## Context

ADR-034 (Egyptian Hieroglyphs) was the first non-mathematical domain attempted
in the Equation Editor. The implementation attempt revealed architectural gaps
in the engine — specifically, zero-argument operations being invisible to the
slot system and UUID wrapping forcing math mode on all content. The
domain-specific Rust and JavaScript code written during that attempt has been
reverted; the architectural lessons are captured here.

The Equation Editor is a **compiler**. Templates are its production rules. The
engine walks the AST, looks up each operation's template, substitutes
recursively rendered children into `{left}`, `{right}`, etc., and hands the
composed string to Typst. The engine does not interpret or transform the
template output. It has zero domain knowledge.

This principle already mostly holds. The templates specify the exact Typst (and
LaTeX, HTML, Unicode, Kleis) output. A `frac` template writes
`"frac({num}, {den})"`. A `quadrat_h` template writes
`"#grid(columns: 2, gutter: 0em, [{left}], [{right}])"`. The engine just does
string substitution. It does not need to know what "mode" the output is — the
template author already decided.

Three violations of this principle currently prevent the primary requirement
from being met:

1. **UUID wrapping imposes math mode.** When a filled slot needs position
   tracking, `render_editor.rs` wraps it as `#box[$rendered$]<id{uuid}>`. The
   inner `$...$` re-enters Typst math mode, overriding what the template
   produced. This is the one actual engine bug.

2. **No generic slot validation.** There is no mechanism for templates to
   declare what they accept. Domain validation rules should be expressible in
   template metadata (`slot_type`, `accepts`), not hardcoded in Rust.

3. **Hardcoded palettes.** `static/index.html` contains ~240 lines of
   hardcoded HTML palette buttons (lines 963–1199), ~130 lines of hardcoded
   `templateMap` entries (lines 2081–2207), and ~180 lines of hardcoded
   `astTemplates` entries (lines 2211–2393). Meanwhile, `palette.kleist`
   already defines the complete palette structure but the client ignores it.

4. **Parser rejects template metadata.** `TemplateDefinition` in
   `kleist_parser.rs` has no `metadata` field, and the parser errors on
   unrecognized `identifier: "value"` pairs. Templates cannot carry
   domain-specific attributes like `mode: "content"` or `slot_type: "glyph"`.

These violations will compound as new domains are added. Organic chemistry,
electronic circuits, quantum circuits, and other notation systems each have
their own composition rules and rendering requirements. If the engine needs
per-domain code for each one, the architecture fails.

### What the Egyptian experience taught us

The rendering itself worked. The `.kleist` templates produce correct Typst
output, and the substitution engine composes them correctly. The problems were:

- **Bounding box extraction for filled slots broke** because UUID wrapping
  re-entered math mode around content-mode output (`#box[$#grid(...)$]`).
- **Zero-argument operations were invisible** — `collect_editor_slots_recursive`
  skips Operation nodes entirely, so glyphs got no UUID, no slot, no bounding
  box. This was the root cause of "Argument Bounding Boxes (semantic): (none)."
- **Layout debugging consumed excessive time** because the relationship between
  templates, the math-mode wrapper, and UUID wrapping was not understood as a
  system.

The solution is to let templates declare a generic `mode` property in their
metadata. The engine reads this property to determine wrapping behavior. It
never references specific templates or domains — `mode: "content"` is a
generic rendering property any template can set.

### Type inference as translation

The Middle Egyptian paper ("The Scribe is the Skolem") establishes that type
inference IS translation. The Equation Editor already has a type-checking
pipeline:

1. Client sends `currentAST` via `POST /api/type_check` (debounced 500ms)
2. Server converts `EditorNode` → `Expression` via `editor_node_to_expression`
3. `TypeChecker::with_stdlib()` loads `.kleis` structures (matrices, tensors)
4. HM unification via `infer_and_solve()` checks against structure axioms
5. `TypeContextBuilder.infer_operation_type()` delegates to
   `SignatureInterpreter` for unification against operation type signatures

For matrices, this catches `Matrix(3,3) + Matrix(2,2)` via `NatValue`
unification failure. For Middle Egyptian, the **same machinery** catches
grammatical violations:

- `MiddleEgyptianNominalGrammar` defines `gender : Noun → Gender`,
  `adjective_gender : Adjective → Gender`
- Axiom 53: `modifies(a, n) → adjective_gender(a) = gender(n)` — a type
  constraint enforced through the same HM unification path
- The `EditorTypeTranslator` generically reads `kind` and `metadata` from
  EditorNodes — it already supports any domain without Rust changes

**Consequence for the engine fixes:** The four changes below handle
**rendering**. Type checking already works generically. For a new domain, the
template `.kleist` files specify both rendering AND type information — the
same template that tells the renderer how to draw a glyph also tells the type
system (via `kind` and `metadata`) what grammatical/structural role it fills.

## Decision

### Principle: Templates are the single source of truth

The Equation Editor is a compiler from Editor AST to output strings. Templates
are the code generation rules. The engine provides exactly four generic
services:

1. **Substitution** — walk the AST, look up template, replace placeholders with
   recursively rendered children
2. **Transparent labeling** — attach Typst labels for position tracking without
   altering the rendered content
3. **Generic slot validation** — check `slot_type`/`accepts` metadata strings,
   with no knowledge of what the strings mean
4. **Palette serving** — deliver the `@palette` structure and template ASTs to
   the client

Everything else — what domains exist, how they render, what compositions are
valid, what the palette looks like — lives in `.kleist` files.

The changes below are **one-time engine fixes** to make the engine fully
generic. After these changes, adding a new domain requires only `.kleist` files
and assets.

### Change 1: Zero-argument Operations must be trackable

**Problem:** Zero-argument Operations (like Egyptian glyph `A1`, or any future
zero-arg template) are invisible to the interactive editing system.

`collect_editor_slots_recursive` in `server.rs` (line ~1360) explicitly skips
Operation nodes: "Don't create slots for operations themselves — they're not
editable. Only their arguments are editable." For a zero-arg Operation, the
args loop runs zero times. Result: **no slot, no UUID, no wrapping, no bounding
box.** The glyph renders in the SVG but cannot be clicked or replaced.

This is the root cause of the `Argument Bounding Boxes (semantic): (none)` bug
observed during the Egyptian implementation.

**Fix:** In `collect_editor_slots_recursive`, treat zero-argument Operations as
filled leaves — same as Objects and Consts:

```rust
EditorNode::Operation { operation } => {
    if operation.args.is_empty() {
        // Zero-arg operation = leaf (e.g., glyph, symbol template)
        let uuid = uuid::Uuid::new_v4().to_string().replace("-", "");
        slots.push(ArgumentSlot {
            id: uuid,
            path: path.clone(),
            hint: format!("value: {}", operation.name),
            is_placeholder: false,
            role: role.clone(),
        });
    } else {
        // Multi-arg operation: recurse into children (existing logic)
        for (i, arg) in operation.args.iter().enumerate() {
            // ... existing structural skip logic for Matrix/Piecewise ...
        }
    }
}
```

**File:** `src/bin/server.rs` — `collect_editor_slots_recursive`

### Change 1b: UUID wrapping for zero-arg Operations

Once zero-arg Operations have UUIDs, they need to be wrapped in the Typst
output for position tracking. Currently, UUID wrapping happens only for Objects
and Consts (via `render_object` and `render_const`), and for matrix/piecewise
cells. It does **not** happen in the general `render_operation` path.

The rendered output for a zero-arg Operation comes from its template — which
could be math-mode (`"alpha"`) or content-mode
(`"#box(image(\"static/glyphs/egyptian/A1.svg\"))"`) depending on the domain.

**The existing UUID wrapping format** is:
```rust
format!("#[#box[${}$]<id{}>]", rendered, uuid)
```

The `$...$` works for math content but breaks content-mode output. However,
removing `$...$` entirely (`#[#box[{}]<id{}>]`) would break math content —
bare `alpha` in a `#box[...]` renders as the text "alpha", not the Greek
letter.

**Fix:** Read the template's `mode` metadata to determine wrapping. If
`mode` is `"content"`, wrap without `$...$`. If absent or `"math"`, use
the existing math-mode wrapping. This is a generic rendering property —
the engine never references specific templates or domains:

```rust
fn uuid_wrap(rendered: &str, uuid: &str, is_math_mode: bool) -> String {
    if is_math_mode {
        format!("#[#box[${}$]<id{}>]", rendered, uuid)
    } else {
        format!("#[#box[{}]<id{}>]", rendered, uuid)
    }
}
```

A new domain template sets `mode: "content"` in its `.kleist` file:

```kleist
@template
  name: "egyptian_glyph_A1"
  typst: "#image(\"static/glyphs/egyptian/A1.svg\", height: 1.5em)"
  mode: "content"
```

The engine reads `mode` from the template metadata and passes it to
`uuid_wrap`. Existing templates have no `mode` field, so they default to
math mode (backward compatible). No template names appear in Rust code.

**File:** `src/render_editor.rs` — extract a `uuid_wrap` helper, use it at all
four existing wrapping sites plus the new zero-arg Operation wrapping site.

### Change 2: Generic slot validation from template metadata

**Current:** No slot validation exists. Any child can be inserted into any
parent slot without constraint.

**Fix:** Introduce a generic validator that reads two metadata fields:

- `slot_type` — what a template **is** (its type tag)
- `accepts` — what a composition template's slots **accept** (comma-separated
  type tags)

```kleist
@template quadrat_h {
    slot_type: "egyptian_composition"
    accepts: "egyptian_glyph,egyptian_composition"
    typst: "#grid(columns: 2, gutter: 0em, [{left}], [{right}])"
    ...
}

@template A1 {
    slot_type: "egyptian_glyph"
    typst: "#box(image(\"static/glyphs/egyptian/A1.svg\"))"
    ...
}
```

The engine has one generic rule: when inserting child B into a slot of parent A,
check that B's `slot_type` appears in A's `accepts` list. The engine does not
know what `"egyptian_glyph"` means — it matches strings.

Templates without `accepts` accept anything (backward compatible — all existing
math templates work unchanged). Templates without `slot_type` have no type tag
and pass any `accepts` check.

**Parser change required** (`kleist_parser.rs`): `TemplateDefinition` currently
has no `metadata` field, and the parser errors on unrecognized pairs. Add
`pub metadata: HashMap<String, String>` to `TemplateDefinition` and change
the parser's error-returning catch-all arm to collect unknown
`identifier: "string"` pairs into this map. After this one-time fix,
`slot_type`, `accepts`, `mode`, and any future metadata fields are
automatically supported without further parser changes.

**Server-side** (`render_editor.rs`): Add `validate_slot(parent_name,
child_name, ctx)` that reads `accepts` and `slot_type` from template metadata.

**Client-side** (`static/index.html`): Wire `validateInsertion()` into
`insertStructuralTemplateAt` using metadata already available from
`/api/templates`. When a slot is selected and its parent has `accepts`, gray
out incompatible palette items.

**Future domains use the same mechanism:**

| Domain | `slot_type` examples | `accepts` examples |
|--------|---------------------|--------------------|
| Egyptian | `egyptian_glyph`, `egyptian_composition` | `egyptian_glyph,egyptian_composition` |
| Chemistry | `atom`, `functional_group`, `bond` | `atom,functional_group` |
| Circuits | `resistor`, `capacitor`, `source`, `component` | `component` |
| Quantum | `gate`, `measurement`, `qubit_line` | `gate,measurement` |
| Math | (none — accepts anything) | (none — accepts anything) |

### Change 3: Data-driven palettes

**Current state:**
- Math palettes: ~240 lines of hardcoded HTML buttons in `index.html`
  (lines 963–1199), plus ~130 lines of `templateMap` and ~180 lines of
  `astTemplates`
- `@palette` in `palette.kleist` defines tab/group structure but the client
  ignores it
- Egyptian loads dynamically from `/api/templates` but with Egyptian-specific JS
- `templateMap` and `astTemplates` are half hardcoded, half runtime-populated

**Fix:** The `@palette` block is already the complete palette specification.
Serve it.

**Server** (`server.rs`): New `GET /api/palette` endpoint returns the palette
structure with each template's metadata and generated AST:

```json
{
  "tabs": [
    {
      "name": "Basics",
      "groups": [
        {
          "name": "Arithmetic",
          "items": [
            {
              "type": "template",
              "name": "frac",
              "glyph": "\u00bd",
              "svg": "<svg>...</svg>",
              "ast": {
                "Operation": {
                  "name": "frac",
                  "args": [
                    {"Placeholder": {"id": 0, "hint": "num"}},
                    {"Placeholder": {"id": 1, "hint": "den"}}
                  ]
                }
              },
              "metadata": {},
              "accepts": null
            }
          ]
        },
        {"type": "separator"},
        {
          "name": "Builders",
          "items": [
            {
              "type": "tool",
              "name": "matrix_builder",
              "handler": "showMatrixBuilder",
              "glyph": "\u229e",
              "svg": "<svg>...</svg>"
            }
          ]
        }
      ]
    }
  ]
}
```

The AST is generated from the `pattern:` field. `"frac(num, den)"` becomes an
Operation with two Placeholders whose hints are the parameter names.
Zero-argument templates produce `{ "Operation": { "name": "A1", "args": [] } }`.
Templates without a `pattern:` field use the same zero-argument form.

**Client** (`index.html`): Replace all hardcoded palette HTML, `templateMap`,
and `astTemplates` with a single `buildPaletteFromAPI()` function:

1. Fetch `/api/palette` on startup
2. Generate tabs, groups, buttons, separators from the JSON structure
3. Populate `astTemplates` from the `ast` field — no hardcoded AST definitions
4. When templates have filterable metadata keys, generate filter dropdowns
   automatically (generalizes the Egyptian sign_shape/sign_type dropdowns —
   the code sees "templates in this tab have metadata key X with N distinct
   values, so show a dropdown")
5. Tools invoke their `handler` function by name (the handful of built-in
   handlers like `showMatrixBuilder` remain in JS)

Adding a new domain means adding `.kleist` files and optional asset files
(SVGs, images). No HTML, no JavaScript, no Rust.

### Future direction: Unifying .kleist into .kleis

Templates are currently a separate DSL (`.kleist`). The long-term path is to
express them as Kleis structures:

```kleis
structure QuadratH {
    define name = "quadrat_h"
    define typst = "#grid(columns: 2, gutter: 0em, [{left}], [{right}])"
    define slot_type = "egyptian_composition"
    define accepts = "egyptian_glyph,egyptian_composition"

    axiom horizontal_rule : sign_shape(left) != "Tall"
}
```

Validation constraints become axioms verified by Z3. The type system enforces
slot compatibility. The `@palette` structure becomes a Kleis data definition.

This aligns with the Kleis philosophy (ADR-005, Level 3): notation + rules +
verification + output. Templates are notation. Slot constraints are rules. Z3
does verification. Typst does output.

This is deferred. The `.kleist` approach works and should be completed first.
The architecture must not introduce anything in `.kleist` that cannot eventually
be expressed in `.kleis`.

## Consequences

### Positive

1. **Adding a new domain requires zero Rust and zero JavaScript changes.**
   Only `.kleist` files and assets. This is the primary requirement.
2. **The UUID/slot fix is localized.** Slot collection change is in one
   function in `server.rs`. UUID wrapping is a helper function used at five
   sites in `render_editor.rs`. No changes to `typst_compiler.rs`.
3. **Validation becomes declarative.** Template authors define what composes
   with what. The engine matches strings.
4. **The palette is fully data-driven.** `palette.kleist` is already written —
   the client just needs to consume it.
5. **AST generation from `pattern:` eliminates ~310 lines of hardcoded JS**
   (`templateMap` + `astTemplates`) and ensures the server and client agree
   on template structure.

### Negative

1. **`buildPaletteFromAPI()` is a significant JS rewrite.** The current
   hardcoded palettes work. Replacing them requires careful testing to avoid
   regressions in button behavior, keyboard shortcuts, and tool handlers.
2. **The hardcoded `astTemplates` fallback** is currently load-bearing. Removing
   it requires that `pattern:` fields exist on all templates and that the
   server's AST generation is correct.
3. **The math-mode top-level wrapper stays.** Content-mode templates work inside
   it via Typst's `#` code escaping, which is standard Typst behavior. But this
   is a coupling to Typst's current semantics — if Typst changes how `#` works
   inside `$ ... $`, it would break.

### Neutral

1. **`kleist_parser.rs` needs a one-time fix** — add `metadata` field and
   open the catch-all arm. After that, it never needs to change again for
   new domains or new metadata fields.
2. **`typst_compiler.rs` needs no changes.** The top-level wrapper stays.
3. **Existing math templates are unaffected.** Templates without `slot_type`,
   `accepts`, or `mode` work exactly as before.

## Files affected

| File | Change |
|------|--------|
| `src/kleist_parser.rs` | Add `metadata: HashMap<String, String>` to `TemplateDefinition`. Change parser catch-all to collect unknown `identifier: "string"` pairs. |
| `src/bin/server.rs` | Treat zero-arg Operations as filled leaves in `collect_editor_slots_recursive`. New `/api/palette` endpoint. AST generation from `pattern:` fields. |
| `src/render_editor.rs` | Extract `uuid_wrap` helper (mode from template metadata). Add UUID wrapping for zero-arg Operations. Add generic `validate_slot`. |
| `static/index.html` | Replace hardcoded palettes, `templateMap`, `astTemplates` with `buildPaletteFromAPI()`. Add generic `validateInsertion()`. |
| `std_template_lib/*.kleist` | Add `slot_type:`, `accepts:`, `mode:` metadata where needed. Add missing `pattern:` fields for AST generation. |
| `src/math_layout/typst_compiler.rs` | No changes. |

## References

- `docs/equation-editor-architecture.md` — complete codebase analysis that
  identified the three violations
- ADR-023 Future Work item 4: "Generate palette UI from palette.kleist"
- ADR-023 Future Work item 7: "Domain-specific metadata"
- ADR-006 amendment: "Domain-specific composition constraints via metadata"
- ADR-034: First non-math domain; exposed the violations documented here
