# Kleis Equation Editor — Complete Architecture

This document describes how every component of the Kleis Equation Editor works,
from the user clicking a palette button to the final interactive SVG displayed
in the browser.

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [The Editor AST](#2-the-editor-ast)
3. [Template Definitions (`.kleist` files)](#3-template-definitions-kleist-files)
4. [The `.kleist` Parser (`kleist_parser.rs`)](#4-the-kleist-parser)
5. [Server Architecture (`server.rs`)](#5-server-architecture)
6. [The Rendering Pipeline (`render_editor.rs`)](#6-the-rendering-pipeline)
7. [The Typst Compiler (`typst_compiler.rs`)](#7-the-typst-compiler)
8. [The Client UI (`static/index.html`)](#8-the-client-ui)
9. [End-to-End Data Flow](#9-end-to-end-data-flow)
10. [Known Limitations and Architectural Constraints](#10-known-limitations-and-architectural-constraints)

---

## 1. System Overview

The Equation Editor is a structural editor for mathematical notation. The user
manipulates a tree (the **Editor AST**) by clicking palette buttons or typing
into inline fields. The AST is sent to a Rust server, which renders it through
Typst into an SVG image with embedded metadata for interactive overlays.

```
┌─────────────────────────────────────────────────────────────────┐
│  Browser  (static/index.html)                                   │
│                                                                 │
│  currentAST  ──POST /api/render_typst──►  Server (server.rs)    │
│                                                                 │
│  ◄── JSON { svg, placeholders, argument_slots, bounding_boxes } │
│                                                                 │
│  SVG injected into #structuralEditor with overlay <rect>s       │
└─────────────────────────────────────────────────────────────────┘

Server-side pipeline:

  EditorNode (JSON)
      │
      ▼
  render_editor.rs  ── template substitution ──►  Typst markup string
      │
      ▼
  typst_compiler.rs ── Typst engine ──►  SVG + placeholder positions
                                         + argument bounding boxes
```

---

## 2. The Editor AST

**File:** `src/editor_ast.rs`

The Editor AST is a JSON-serializable tree with five node variants. It is
distinct from the Kleis language AST (`Expression`) — it has no source spans
and is optimized for UI manipulation, not evaluation.

### Node variants

| Variant | JSON shape | Purpose |
|---------|-----------|---------|
| Object | `{ "Object": "x" }` | Identifier or symbol (variable, Greek letter, glyph name) |
| Const | `{ "Const": "2" }` | Numeric or string literal (always a string value) |
| Placeholder | `{ "Placeholder": { "id": 1, "hint": "…" } }` | Empty slot for user input (□) |
| Operation | `{ "Operation": { "name": "frac", "args": [...] } }` | Named operation with child arguments |
| List | `{ "List": [...] }` | Ordered list of nodes (matrix rows, etc.) |

### OperationData

```rust
pub struct OperationData {
    pub name: String,                                    // e.g. "frac", "plus", "quadrat_h"
    pub args: Vec<EditorNode>,                           // child expressions
    pub kind: Option<String>,                            // e.g. "tensor" — triggers special rendering
    pub metadata: Option<HashMap<String, serde_json::Value>>,  // e.g. indexStructure for tensors
}
```

- `kind` and `metadata` are optional and omitted from JSON when absent
  (`skip_serializing_if`).
- `#[serde(untagged)]` on the enum means deserialization matches by key presence
  (not by a `"type"` discriminator).

### Node ID convention

Nodes are addressed by **dot-separated path strings** rooted at `"0"`. The
root is `"0"`, its first child is `"0.0"`, second child `"0.1"`, and so on
recursively (`"0.1.2"`). These paths are used both client-side (for overlay
click targets) and server-side (for UUID-tagged Typst labels).

---

## 3. Template Definitions (`.kleist` files)

**Directory:** `std_template_lib/`

Templates define how each operation renders across five output targets. The
system ships 13 `.kleist` files with ~377 `@template` blocks total.

### File inventory

| File | Templates | Categories |
|------|-----------|------------|
| `basic.kleist` | 38 | arithmetic, comparison, brackets, indices |
| `calculus.kleist` | 25 | trig, functions, derivatives, integrals, limits, accents |
| `control_flow.kleist` | 11 | control_flow, patterns |
| `egyptian.kleist` | 227 | egyptian_A_man through egyptian_Z_stroke, egyptian_composition |
| `greek.kleist` | 39 | greek, special, physics |
| `logic.kleist` | 13 | logic, sets |
| `pot.kleist` | 8 | pot |
| `quantum.kleist` | 7 | quantum |
| `tensors.kleist` | 2 | tensors |
| `transforms.kleist` | 7 | transforms |
| `vectors.kleist` | 7 | vectors |
| `palette.kleist` | 0 | (palette layout only — `@palette` block) |
| `tools.kleist` | 0 | (tool definitions only — `@tool` blocks) |

### Template structure

```kleist
@template frac {
    pattern: "frac(num, den)"
    unicode: "{num}/{den}"
    latex: "\\frac{{{num}}}{{{den}}}"
    html: "<span class=\"frac\"><span class=\"num\">{num}</span><span class=\"den\">{den}</span></span>"
    typst: "frac({num}, {den})"
    kleis: "frac({num}, {den})"
    category: "arithmetic"
    glyph: "½"
    svg: "<svg>...</svg>"
}
```

**Fields:**
- `name` — identifier after `@template`
- `pattern` — Kleis-style function signature (informational)
- `unicode`, `latex`, `html`, `typst`, `kleis` — per-target output strings with
  `{placeholder}` tokens
- `category` — organizational tag for palette grouping and API filtering
- `glyph` — button label text
- `svg` — inline SVG for palette button icon
- `shortcut` — optional keyboard shortcut
- `metadata` — arbitrary key-value pairs (e.g. `sign_shape`, `sign_type`,
  `sound` for Egyptian glyphs)

### Placeholder token convention

Templates use named `{tokens}` that are positionally mapped to operation arguments:

| Arg index | Token aliases |
|-----------|---------------|
| 0 (first) | `{arg}`, `{left}`, `{body}`, `{base}`, `{num}`, `{content}`, `{function}`, `{bra}`, `{A}`, `{n}`, `{operator}`, … |
| 1 (second) | `{right}`, `{den}`, `{exponent}`, `{sup}`, `{subscript}`, `{ket}`, `{B}`, `{k}`, … |
| 2 (third) | `{to}`, `{superscript}`, `{target}`, `{lower}` (for `index_mixed`), … |
| 3 (fourth) | `{idx3}`, `{variable}` (for `int_bounds`), … |
| All args | `{args}` — comma-joined rendering of all arguments |

The same template can use different alias names for the same positional argument
to make template strings read naturally.

### Egyptian glyph templates

Each of the 225 Gardiner-classified glyphs is a zero-argument operation template:

```kleist
@template A1 {
    typst: "#box(image(\"static/glyphs/egyptian/A1.svg\"))"
    unicode: "𓀀"
    category: "egyptian_A_man"
    glyph: "A1"
    svg: "<svg>...</svg>"
    sign_shape: "Tall"
    sign_type: "Det"
}
```

The two composition templates use Typst `grid()` for layout:

```kleist
@template quadrat_h {
    pattern: "quadrat_h(left, right)"
    typst: "#grid(columns: 2, gutter: 0em, [{left}], [{right}])"
    unicode: "{left}{right}"
    category: "egyptian_composition"
}

@template quadrat_v {
    pattern: "quadrat_v(left, right)"
    typst: "#grid(rows: 2, gutter: 0em, [{left}], [{right}])"
    unicode: "{left}/{right}"
    category: "egyptian_composition"
}
```

### Palette layout

`palette.kleist` defines the UI tab structure:

```kleist
@palette {
    tab "Basics" {
        group "Arithmetic" {
            equals
            plus
            minus
            ...
        }
        separator
        group "Builders" {
            @piecewise_builder
            @matrix_builder
        }
    }
    tab "Calculus" { ... }
    tab "Linear Algebra" { ... }
    ...
}
```

Templates are referenced by name; `@tool_name` references invoke interactive
builders (not templates). Separators create visual dividers between groups.

### Tools

`tools.kleist` defines interactive builders (`@tool`) that differ from
templates — they invoke JavaScript handlers (e.g. `showMatrixBuilder`) to
produce dynamic AST structures rather than simple template insertion.

---

## 4. The `.kleist` Parser

**File:** `src/kleist_parser.rs`

A hand-written recursive-descent parser: string → tokenizer → `KleistFile`.

### `TemplateDefinition` struct

```rust
pub struct TemplateDefinition {
    pub name: String,
    pub pattern: Option<String>,
    pub unicode: Option<String>,
    pub latex: Option<String>,
    pub html: Option<String>,
    pub typst: Option<String>,
    pub kleis: Option<String>,
    pub category: Option<String>,
    pub shortcut: Option<String>,
    pub svg: Option<String>,
    pub glyph: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

### Parsing logic

Inside `parse_template()`:
1. Consume `@template`, then an identifier (the name), then `{`.
2. Loop until `}`, matching each line as `keyword: "string_value"`.
3. Known keywords (`pattern`, `unicode`, `latex`, `html`, `typst`, `kleis`,
   `category`, `shortcut`, `svg`, `glyph`) are stored in their respective
   fields.
4. Any unrecognized identifier becomes a metadata key-value pair.
5. String literals handle escape sequences: `\n`, `\t`, `\"`, `\\`, `\{`,
   `\}`. Unknown escapes preserve both `\` and the character (for LaTeX
   compatibility: `\frac` stays as `\frac`).
6. Multi-line strings are supported (newlines inside `"..."` are preserved).

### Loading

| Function | Behavior |
|----------|----------|
| `parse_kleist(source)` | Parse a single string |
| `parse_kleist_file(path)` | Read file + parse |
| `load_kleist_directory(path)` | Read all `*.kleist` files in a directory, concatenate templates and tools; last file's `@palette` wins |

---

## 5. Server Architecture

**File:** `src/bin/server.rs`

An Axum HTTP server with shared state and CORS permissive for local dev.

### AppState

```rust
struct AppState {
    type_checker: Arc<Mutex<Option<TypeChecker>>>,
    registry: Arc<StructureRegistry>,
}
```

- `type_checker` — Hindley-Milner type checker loaded from embedded stdlib
  (`include_str!` of `stdlib/*.kleis`)
- `registry` — structure registry for Z3 verification, loaded from
  `stdlib/` on disk

### API endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/` | Serve `static/index.html` |
| POST | `/api/render_typst` | **Main editor endpoint**: AST → SVG + metadata |
| POST | `/api/render_ast` | AST → string output (HTML/LaTeX/Unicode/Typst/Kleis) |
| POST | `/api/render` | LaTeX string → rendered output |
| POST | `/api/parse` | LaTeX string → Kleis Expression JSON |
| POST | `/api/type_check` | AST → Hindley-Milner type inference |
| POST | `/api/export_typst` | AST → Typst source string (for clipboard) |
| POST | `/api/verify` | AST → Z3 axiom verification |
| POST | `/api/check_sat` | AST → Z3 satisfiability check |
| POST | `/api/render_kleis` | Expression JSON → Kleis text |
| GET | `/api/operations` | List of supported operations (hardcoded) |
| GET | `/api/templates` | Template catalog from `std_template_lib/` |
| GET | `/api/gallery` | Example expressions for the gallery |
| GET | `/health` | Returns `"OK"` |
| GET | `/static/*` | Static file serving via `tower_http::ServeDir` |

### The `/api/render_typst` handler (core pipeline)

This is the main endpoint that drives the structural editor. It:

1. Parses the incoming JSON into an `EditorNode` tree (`json_to_editor_node`).
2. Walks the tree with `collect_argument_slots_from_editor_node` to build the
   list of all editable positions (both placeholders and filled leaves), each
   with a path, hint, role, and `is_placeholder` flag.
3. Extracts placeholder IDs from slots where `is_placeholder == true`.
4. Builds a `node_id_to_uuid` map: for each filled (non-placeholder) leaf, assigns
   a truncated 8-character UUID. This UUID will be injected into Typst markup
   as a label so the compiler can report its bounding box.
5. Calls `compile_editor_node_with_semantic_boxes(node, unfilled_ids,
   all_slot_ids, node_id_to_uuid)`.
6. Returns JSON: `{ svg, placeholders, argument_bounding_boxes, argument_slots,
   success }`.

### The `/api/templates` handler

- Reads all `*.kleist` files from `std_template_lib/` via
  `load_kleist_directory`.
- Filters to templates that have either an `svg` or a `category`.
- Returns JSON array of `{ name, category, glyph, svg, metadata }`.
- Used by the client for the Egyptian palette (and potentially other
  dynamic palettes in the future).

### Template loading (three independent paths)

The server loads template/structure data in three separate ways:

1. **TypeChecker** — `TypeChecker::with_stdlib()` uses `include_str!` to embed
   `stdlib/types.kleis`, `prelude.kleis`, `matrices.kleis`, etc. at compile
   time.
2. **StructureRegistry** — `load_stdlib_registry()` reads `stdlib/` from disk
   at startup, resolving imports recursively.
3. **EditorRenderContext** — loaded lazily per thread via
   `from_std_template_lib()`, reading `std_template_lib/` from disk. Falls
   back to ~100 hardcoded `add_template()` calls if the directory is missing.
4. **`/api/templates`** — reads `std_template_lib/` on each request for the
   client palette.

---

## 6. The Rendering Pipeline

**File:** `src/render_editor.rs`

This module converts an `EditorNode` tree into a string for any of the five
render targets. It does **not** produce SVG — it produces Typst markup (or
LaTeX/HTML/Unicode/Kleis text) that downstream consumers compile or display.

### Render targets

```rust
pub enum RenderTarget {
    Unicode,
    LaTeX,
    HTML,
    Typst,
    Kleis,
}
```

There is no MathML target. The "HTML" target uses `<sup>`, `<sub>`, `<table>`,
`<span>` elements.

### EditorRenderContext

```rust
pub struct EditorRenderContext {
    pub unicode_templates: HashMap<String, String>,
    pub latex_templates: HashMap<String, String>,
    pub html_templates: HashMap<String, String>,
    pub typst_templates: HashMap<String, String>,
    pub kleis_templates: HashMap<String, String>,
    pub template_metadata: HashMap<String, HashMap<String, String>>,
}
```

Per-target maps: operation name → template string with `{placeholder}` tokens.

`template_metadata` stores domain-specific key-value pairs from `.kleist` files
(e.g. `sign_shape`, `sign_type` for Egyptian glyphs). Only populated when
loading from `.kleist` files, not from the hardcoded fallback.

### Dispatch (`render_internal`)

Matches on the `EditorNode` variant:

- **Object** → `render_object`: look up object string as a template key. If
  found, use the template. Otherwise, map symbols to target-specific forms
  (e.g. `α` → `\alpha` for LaTeX, `alpha` for Typst). For Typst with UUID
  labels: wrap in `#box[${rendered}$]<id{uuid}>`.
- **Const** → `render_const`: escape for LaTeX, UUID-wrap for Typst.
- **Placeholder** → target-specific empty slot: Typst produces
  `#[#box[$square.stroked$]<ph{id}>]` (the `<ph{id}>` label becomes a
  `data-typst-label` in the SVG for position extraction).
- **Operation** → `render_operation`: classifies by kind/name:
  - Tensors → `render_tensor` (custom per-target, uses `metadata.indexStructure`)
  - Matrices → dedicated matrix renderers
  - Piecewise/Cases → dedicated piecewise renderers
  - Variadic binary ops (>2 args) → `fold_variadic_binary_op`
  - `quadrat_*` → `validate_quadrat` (warnings only) then template
  - Everything else → `render_with_template`
- **List** → recursive rendering of elements

### Template substitution (`apply_template_substitutions`)

1. Look up the template string from the context map for the given target.
2. If not found, fall back to `"name({args})"`.
3. Replace positional aliases: `{left}` → `rendered_args[0]`, `{right}` →
   `rendered_args[1]`, etc. (full alias table in section 3).
4. Replace `{args}` with comma-joined rendering of all arguments.
5. Replace `{glyph}` with the operation name.

Substitution is **plain string replacement** — no recursive template expansion,
no escaping beyond what each target's renderer applies to leaf nodes.

### UUID wrapping for hit-testing

When `node_id_to_uuid` is non-empty (the render_typst path), every Object and
Const node whose path is in the map gets wrapped with a Typst label:

```
#box[${rendered}$]<id{uuid}>
```

Placeholders use a different label format: `<ph{id}>`. These labels survive
Typst compilation and appear as `data-typst-label` attributes in the output
SVG, enabling position extraction.

### Quadrat validation

`validate_quadrat(op_name, args, ctx)` checks Egyptian composition validity:
- `quadrat_h` with a "Tall" glyph on either side → warning
- `quadrat_v` with "Tall" glyphs on both top and bottom → warning

Warnings go to stderr only; rendering always proceeds.

---

## 7. The Typst Compiler

**File:** `src/math_layout/typst_compiler.rs`

This module compiles Typst markup to SVG and extracts geometric metadata for
the interactive overlay system.

### The math-mode wrapper

All markup is wrapped before compilation:

```rust
let typst_doc = format!(
    r#"#set page(width: auto, height: auto, margin: 0pt)
#set text(size: 24pt)
#box($ {} $)
"#,
    markup
);
```

This means:
- The page auto-sizes to content with no margins.
- Text is 24pt.
- **Everything is inside `$ ... $` (math mode)**, wrapped in a `#box()`.

**This is the single most important architectural constraint**: all template
output must be valid inside Typst math mode. Content-mode constructs (like
`#grid`, `#image`, `#box`) work inside math mode because Typst allows `#`
code-mode escapes within `$ ... $`. However, **children rendered as math** that
are spliced into content-mode parents can cause nesting issues (e.g.
`#grid([#box(image(...))])` works because the child is already content-mode,
but mixing would fail if children produced bare math tokens).

### MinimalWorld

A minimal Typst `World` implementation:
- **Fonts:** All fonts from `typst_assets::fonts()` (bundled at compile time).
- **Source:** Only the virtual `main.typ` document.
- **File access:** `World::file()` resolves paths relative to
  `std::env::current_dir()`. This is how `image("static/glyphs/egyptian/A1.svg")`
  resolves — the server must be started from the project root.
- **No multi-file projects** or `@import` resolution.

### Compilation pipeline (`compile_math_to_svg_with_ids`)

1. Wrap markup in the math-mode document template.
2. Create `MinimalWorld`, compile with `typst::compile(&world)`.
3. Render first page to SVG with `typst_svg::svg(page)`.
4. Extract layout bounding boxes from the Typst `Frame` tree
   (`extract_bounding_boxes_from_frame`).
5. Extract placeholder positions from SVG labels
   (`extract_positions_from_labels` — parses `data-typst-label="ph{N}"` attributes).
6. Extract UUID positions for filled slots
   (`extract_uuid_positions` — parses `data-typst-label="id{uuid}"` attributes).
7. Group content boxes into argument bounding boxes
   (`group_content_into_arguments`).
8. Expand the SVG viewBox for marker visibility.
9. Return `CompiledOutput { svg, placeholder_positions, argument_bounding_boxes }`.

### Semantic bounding boxes

`compile_editor_node_with_semantic_boxes` adds a second pass:

1. Render the EditorNode to Typst with UUID labels (via `render_editor_node_with_uuids`).
2. Compile to SVG + extract placeholder and UUID positions.
3. Walk the EditorNode tree (`extract_semantic_argument_boxes_from_editor_node`):
   - Placeholders consume labeled positions in order.
   - Operations/leaves with UUIDs use `uuid_positions` for direct box lookup.
4. Replace the generic `argument_bounding_boxes` with semantically accurate ones
   keyed by node ID paths.

### Key data structures

```rust
pub struct CompiledOutput {
    pub svg: String,
    pub placeholder_positions: Vec<PlaceholderPosition>,
    pub argument_bounding_boxes: Vec<ArgumentBoundingBox>,
}

pub struct PlaceholderPosition {
    pub id: usize,       // placeholder ID (or slot_index + 1000 for filled slots)
    pub x: f64, pub y: f64,
    pub width: f64, pub height: f64,
}

pub struct ArgumentBoundingBox {
    pub arg_index: usize,
    pub node_id: String,  // path like "0.1.2"
    pub x: f64, pub y: f64,
    pub width: f64, pub height: f64,
}
```

---

## 8. The Client UI

**File:** `static/index.html`

A single HTML file containing CSS, HTML structure, and a large `<script>` block
with all editor logic.

### Global state

| Variable | Purpose |
|----------|---------|
| `currentAST` | Root `EditorNode` of the expression being edited |
| `editorMode` | `'text'` or `'structural'` |
| `activeEditMarker` | Currently selected slot `{ id, path, nodeId, element, bbox }` |
| `undoStack` / `redoStack` | Deep-cloned AST snapshots (max 50) |
| `lastRenderResponse` | Last JSON from `/api/render_typst` (for debug panel) |
| `templateMap` | Maps palette keys → internal template names |
| `astTemplates` | Maps template names → AST subtrees (with Placeholders) |
| `egyptianTemplatesCache` | Cached `/api/templates` response |
| `currentZoom` | SVG zoom level |
| `COORDINATE_PREFERENCE` | `'placeholder'` (default) or `'semantic'` — overlay positioning mode |

### Palette system

**Static palettes** (Basics, Fences, Accents, Calculus, etc.) are defined as
HTML `<div id="palette-{name}">` blocks with hardcoded buttons. Each button
calls `insertSymbol('...')` or `insertTemplate('...')`.

**Egyptian palette** loads dynamically: on first click of the Egyptian tab,
`showEgyptianPalette()` fetches `GET /api/templates`, filters templates with
`category.startsWith('egyptian_')`, and builds buttons dynamically. Composition
templates (`egyptian_composition`) get a separate row. Glyph templates get
metadata-based filter dropdowns for `sign_shape` and `sign_type`.

### `templateMap` and `astTemplates`

These two maps connect palette buttons to AST structures:

- `templateMap`: Maps a **display key** (what the button passes to
  `insertTemplate`) to an **internal name**.
  - Math: `'\\frac{□}{□}'` → `'fraction'`
  - Egyptian: `'egyptian:quadrat_h'` → `'quadrat_h'`
  - Egyptian glyph: `'egyptian:A1'` → `'eg_A1'`

- `astTemplates`: Maps the **internal name** to an AST subtree with
  Placeholders.
  - `'fraction'` → `{ Operation: { name: "frac", args: [Placeholder, Placeholder] } }`
  - `'quadrat_h'` → `{ Operation: { name: "quadrat_h", args: [Placeholder, Placeholder] } }`
  - `'eg_A1'` → `{ Operation: { name: "A1", args: [] } }`

Math templates are hardcoded in `astTemplates`. Egyptian templates are populated
at runtime from the API response.

### Insertion pipeline

**`insertSymbol(latex)`:**
1. Map LaTeX to Unicode via `latexToUnicode` table.
2. If structural mode with an active edit marker: `setNodeAtPath(currentAST,
   path, { Object: symbol })`, clear marker, re-render.
3. If no marker: set `currentAST = { Object: symbol }`, render.

**`insertTemplate(template)`:**
1. Look up `templateMap[template]` → internal name.
2. Clone `astTemplates[name]`, call `renumberPlaceholders()` to assign fresh IDs.
3. If active edit marker: `setNodeAtPath(currentAST, path, clonedAST)` (insert
   at position).
4. If no marker: replace entire `currentAST`.
5. Re-render.

### Render loop

**`renderStructuralEditor()`** (async):
1. POST `{ ast: currentAST }` to `/api/render_typst`.
2. Receive `{ svg, placeholders, argument_slots, argument_bounding_boxes }`.
3. Build overlay rectangles:
   - For each `argument_slot`, skip "parent" slots (node IDs that are prefixes
     of other slots).
   - Look up geometry from `placeholders` (by `ph{N}` ID) or
     `argument_bounding_boxes` (by `node_id`).
   - Apply role-based adjustments (shrink/shift for `superscript`, `subscript`,
     `base`).
   - Create SVG `<rect class="arg-overlay">` elements with `data-slot-id`,
     `data-path`, `data-node-id`, and `onclick`/`onkeydown` handlers.
4. Inject `<g id="arg-overlays">` into the SVG.
5. Set `container.innerHTML = svg`.
6. Trigger debounced type checking.

### Slot interaction

**`handleSlotClick(event, id, path, nodeId)`:**
1. Save undo state.
2. Set `activeEditMarker` with path and bounding box.
3. Highlight the clicked overlay with `.active-marker`.
4. Normal click → open inline editor (`showInlineEditor`): a `<foreignObject>`
   with an `<input>` positioned over the slot.
5. Modified click (Shift/Ctrl) → prompt for text input.

**Inline editor commit:** Text is parsed by `parseSimpleInput()`:
- Empty string → new `Placeholder`
- Numeric → `Const`
- Anything else → `Object`

The parsed node replaces the slot at the active path, then re-renders.

### Undo/redo

- `saveToUndoStack()` deep-clones `currentAST`, pushes to stack (cap 50),
  clears redo.
- Ctrl+Z / Ctrl+Shift+Z for keyboard shortcuts.
- Undo state is saved on slot clicks and full template replacements. Not
  consistently saved on `insertStructuralTemplateAt` or `insertSymbol`.

### Other features

| Feature | Function | Endpoint |
|---------|----------|----------|
| Type check | `checkTypesDebounced()` | `POST /api/type_check` |
| Z3 verify | `verifyWithZ3()` | `POST /api/verify` |
| Satisfiability | `checkSatisfiable()` | `POST /api/check_sat` |
| Copy Typst | `copyTypstToClipboard()` | `POST /api/export_typst` |
| Gallery | `loadGallery()` | `GET /api/gallery` |
| Text↔Structural | `convertTextToStructural()` / `convertStructuralToText()` | `POST /api/parse` / `POST /api/render_ast` |
| Jupyter mode | `?mode=jupyter` query param | `postMessage` handshake |

---

## 9. End-to-End Data Flow

### Example: User clicks "frac" button, then fills numerator with "x"

```
1. User clicks  □/□  button in Basics palette
   └─► insertTemplate('\\frac{□}{□}')

2. templateMap['\\frac{□}{□}'] = 'fraction'
   astTemplates['fraction'] = { Operation: { name: "frac", args: [Placeholder(0), Placeholder(1)] } }
   └─► clone + renumberPlaceholders → currentAST = { Operation: { name: "frac", args: [Placeholder(3), Placeholder(4)] } }

3. renderStructuralEditor()
   └─► POST /api/render_typst  { ast: currentAST }

4. Server: json_to_editor_node → EditorNode tree
   └─► collect_argument_slots → [ { id: "ph3", path: [0], is_placeholder: true },
                                    { id: "ph4", path: [1], is_placeholder: true } ]

5. Server: render_editor_node_with_uuids(node, Typst, node_id_to_uuid={})
   └─► Typst markup: "frac(#[#box[$square.stroked$]<ph3>], #[#box[$square.stroked$]<ph4>])"

6. Server: compile_math_to_svg_with_ids(markup, [3,4], [3,4])
   └─► Typst engine compiles to SVG
   └─► extract_positions_from_labels → placeholder_positions for ph3 and ph4
   └─► Return { svg, placeholders: [{id:3, x, y, w, h}, {id:4, ...}], argument_slots, ... }

7. Client: inject overlay <rect>s at placeholder positions
   └─► SVG displayed with two clickable □ squares

8. User clicks the numerator □
   └─► handleSlotClick → activeEditMarker = { path: [0] }
   └─► showInlineEditor → <input> appears over the □

9. User types "x", presses Enter
   └─► parseSimpleInput("x") → { Object: "x" }
   └─► setNodeAtPath(currentAST, [0], { Object: "x" })
   └─► currentAST = { Operation: { name: "frac", args: [{ Object: "x" }, Placeholder(4)] } }
   └─► renderStructuralEditor() → re-render with x in numerator

10. Server re-renders:
    └─► "x" gets UUID "a1b2c3d4" → node_id_to_uuid = { "0.0": "a1b2c3d4" }
    └─► Typst: "frac(#box[$x$]<ida1b2c3d4>, #[#box[$square.stroked$]<ph4>])"
    └─► SVG with "x" in numerator, □ in denominator
    └─► Bounding boxes for both positions returned
```

### Example: Egyptian composition `quadrat_h(A1, D21)`

```
1. User clicks horizontal pair composition button
   └─► insertTemplate('egyptian:quadrat_h')
   └─► currentAST = { Operation: { name: "quadrat_h", args: [Placeholder(5), Placeholder(6)] } }

2. renderStructuralEditor() → server renders quadrat_h template:
   └─► typst_templates["quadrat_h"] = "#grid(columns: 2, gutter: 0em, [{left}], [{right}])"
   └─► After substitution: "#grid(columns: 2, gutter: 0em, [#[#box[$square.stroked$]<ph5>]], [#[#box[$square.stroked$]<ph6>]])"
   └─► SVG shows two □ side by side in a grid

3. User clicks left □, then clicks A1 glyph button
   └─► insertTemplate('egyptian:A1') at path [0]
   └─► AST: { Operation: { name: "quadrat_h", args: [
         { Operation: { name: "A1", args: [] } },
         Placeholder(6)
       ] } }

4. Server re-renders:
   └─► A1 template: typst = "#box(image(\"static/glyphs/egyptian/A1.svg\"))"
   └─► quadrat_h with A1 in left cell: "#grid(columns: 2, gutter: 0em, [#box(image(...))], [□])"
   └─► SVG shows hieroglyph image + placeholder side by side
```

---

## 10. Known Limitations and Architectural Constraints

### The math-mode wrapper

All Typst markup is compiled inside `#box($ ... $)`. This means:
- Templates must produce valid Typst math-mode content.
- Content-mode constructs (`#grid`, `#image`, `#box`) work via `#` code-escape
  inside math.
- However, children rendered as pure math tokens spliced into content-mode
  parents can cause nesting issues when `#` characters proliferate.

### Template system is string-based

Template substitution is plain string replacement — `{left}` is literally
replaced with the rendered string of the first argument. There is no type
checking, no recursive template expansion, and no escaping. This means:
- A template cannot reference another template.
- Argument rendering must produce valid output for the surrounding context.
- The same positional aliases are shared across all templates (no per-template
  parameter names in the substitution engine).

### Bounding box extraction depends on labels

Interactive editing relies on Typst labels (`<ph{N}>`, `<id{uuid}>`) surviving
compilation and appearing as `data-typst-label` in the SVG. If Typst's SVG
renderer changes how labels are emitted, or if labels are pruned by optimization,
the overlay system breaks.

For filled (non-placeholder) nodes, bounding boxes are extracted via UUID labels.
This requires each filled leaf to be individually wrapped in `#box[...]<id{uuid}>`,
which adds nesting and can affect layout.

### Egyptian glyphs use image paths

Egyptian glyph templates reference SVG files via `image("static/glyphs/egyptian/A1.svg")`.
These paths resolve relative to `std::env::current_dir()` in the `MinimalWorld`.
The server must be run from the project root directory for images to load.

### Template loading has multiple independent paths

Templates are loaded three different ways (type checker stdlib, structure
registry stdlib, and `std_template_lib/` for rendering), with no coordination
between them. Changes to template files may not be reflected until the server
is restarted (thread-local caching of `EditorRenderContext`).

### Undo is inconsistent

`saveToUndoStack()` is called in some edit paths (`handleSlotClick`,
`insertStructuralTemplate`) but not others (`insertStructuralTemplateAt`,
`insertSymbol` in structural mode). Some edits are not undoable.

### Quadrat validation is advisory only

`validate_quadrat` in `render_editor.rs` emits warnings to stderr but never
blocks rendering. The client-side validation in `validateQuadratPlacement` exists
but is not wired into the standard `insertTemplate` path — it is only called
from the unused `insertEgyptianGlyph` function.

---

*Document generated from reading: `static/index.html`, `src/editor_ast.rs`,
`src/kleist_parser.rs`, `src/render_editor.rs`, `src/math_layout/typst_compiler.rs`,
`src/math_layout/mod.rs`, `src/bin/server.rs`, and all files in `std_template_lib/`.*
