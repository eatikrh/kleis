# Kleis Language Constructs in Equation Editor

## Goal

Enable visual editing of Kleis language constructs (structure, define, axiom, etc.) in the Equation Editor, bridging the gap between visual editing and Kleis formal language.

---

## Render Targets = Different Outputs

**Critical insight:** Each render target produces a fundamentally different artifact:

| Target | Output | Purpose |
|--------|--------|---------|
| **Kleis** | `.kleis` program file | Round-trip storage, verification, sharing |
| **Typst** | Visual display (SVG) | Interactive editing on screen |
| **LaTeX** | Paper/document | Academic publication |
| **HTML** | Web display | Documentation, embedding |
| **Unicode** | Plain text | Terminal, simple display |

```
                    ┌─────────────────┐
                    │   Editor AST    │
                    │  (EditorNode)   │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Kleis Renderer │ │  Typst Renderer │ │  LaTeX Renderer │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  program.kleis  │ │   SVG on screen │ │   paper.tex     │
│  (executable)   │ │   (editable)    │ │   (publishable) │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

**Workflow:**
1. User creates/edits in Equation Editor (Typst display)
2. Save → Kleis renderer → `.kleis` file
3. Load → Kleis parser → Editor AST → Typst display
4. Export → LaTeX renderer → `.tex` for publication

### Kleis Notebook → Paper (Future Scope)

Rendering a **Kleis Notebook** to LaTeX/arXiv requires much more than math:

| Element | Status | Notes |
|---------|--------|-------|
| Math equations | ✅ Exists | Current focus |
| Tables | ❌ Not started | Data presentation |
| Figures/diagrams | ❌ Not started | SVG, TikZ, generated |
| Images/photos | ❌ Not started | Embedded raster images |
| Graphs/plots | ❌ Not started | Visualization of computations |
| Calculations | ❌ Not started | Show derivation steps |
| Colors/styling | ❌ Not started | Highlighting, emphasis |
| Citations | ❌ Not started | BibTeX integration |
| Sections/headings | ❌ Not started | Document structure |
| Code blocks | ❌ Not started | Kleis source display |
| **Typesetting** | ❌ Not started | Margins, page breaks, text flow, columns, headers/footers |

**This is a separate, larger project** - essentially building a Jupyter-like notebook that exports to publication-quality LaTeX.

**Architecture decision:** Kleis Notebook does NOT compete with LaTeX - it **uses** LaTeX for typesetting.

```
┌─────────────────────────────────────────────────────┐
│              Kleis Notebook                         │
│  (content authoring, math editing, verification)   │
└──────────────────────┬──────────────────────────────┘
                       │ export
                       ▼
┌─────────────────────────────────────────────────────┐
│                    LaTeX                            │
│  (typesetting engine - margins, fonts, layout)     │
└──────────────────────┬──────────────────────────────┘
                       │ compile
                       ▼
┌─────────────────────────────────────────────────────┐
│                PDF / arXiv                          │
│  (publication-ready output)                         │
└─────────────────────────────────────────────────────┘
```

Kleis provides: content, structure, math, verification
LaTeX provides: typesetting, fonts, layout, page breaks

### Embedding LaTeX in Kleis (Javadoc-style)

**Idea:** LaTeX/paper content lives in special Kleis comments, similar to Javadoc.

```kleis
/// @title On the Algebraic Properties of Monoids
/// @author Jane Doe
/// @abstract We present a formal treatment of monoid theory...
///
/// # Introduction
/// 
/// A monoid is a fundamental algebraic structure consisting of...
/// 
/// \begin{figure}
///   \includegraphics{monoid-diagram.png}
///   \caption{The monoid structure}
/// \end{figure}

structure Monoid(M) {
    /// The binary operation combines two elements.
    /// This is the core of the monoid structure.
    operation (·) : M × M → M
    
    /// The identity element satisfies $e \cdot x = x$ for all $x$.
    element e : M
    
    axiom left_identity : ∀x. e · x = x
    axiom right_identity : ∀x. x · e = x
}

/// # Conclusion
/// 
/// We have formally verified all monoid axioms using Z3...
```

**Precedents:**
- Javadoc (`/** ... */`) → Java API docs
- Rustdoc (`///`) → Rust documentation  
- Python docstrings → Sphinx docs
- Literate Haskell (`.lhs`) → Interleaved code + prose
- Knuth's WEB → Original literate programming

**Export = Render**

"Export to LaTeX" is just another render target. All operations are renders:

| Action | Render Target | Output |
|--------|---------------|--------|
| Display on screen | Typst | SVG |
| Save file | Kleis | `.kleis` |
| Export to paper | LaTeX | `.tex` |
| Web embed | HTML | `<math>` |

```rust
render(ast, RenderTarget::Typst)  // → screen
render(ast, RenderTarget::Kleis)  // → save
render(ast, RenderTarget::LaTeX)  // → "export"
```

**Unified model:** There's no special "export" operation. It's all just rendering to different targets.

**LaTeX render workflow:**
1. Parse `.kleis` file (including `///` comments)
2. Render code → LaTeX math
3. Render `///` comments → LaTeX prose (pass-through)
4. Output combined `.tex` file

**Benefits:**
- Single source of truth (code + paper in one file)
- Code stays valid Kleis (comments are ignored by parser)
- Documentation lives next to what it documents
- Can generate both API docs AND papers from same source

For now, focus on:
- Math expressions (current)
- Kleis constructs (this plan)
- Notebook features (future)

## Current State

### What Exists

1. **Kleis AST** (`src/kleis_ast.rs`):
   - `StructureDef` - structure definitions with type params, extends, members
   - `FunctionDef` - define name(params) = body
   - `ImplementsDef` - implements Structure(Type) { ... }
   - `StructureMember::Axiom` - axiom name : proposition
   - `DataDef` - data Name = Variant1 | Variant2

2. **Editor AST** (`src/editor_ast.rs`):
   - `EditorNode` - Object, Const, Placeholder, Operation, List
   - `translate_to_editor()` - translates `Expression` → `EditorNode`
   - Already supports `kind` and `metadata` for rendering hints

3. **Kleis Parser** (`src/kleis_parser.rs`):
   - `parse_structure()` → `StructureDef`
   - `parse_function_def()` → `FunctionDef`
   - `parse_implements()` → `ImplementsDef`

### What's Missing

- Translation from Kleis declaration types to EditorNode
- Rendering templates for Kleis constructs (Typst/LaTeX/HTML)
- Palette templates in Equation Editor for creating these constructs

---

## Implementation Plan

### Phase 1: Extend Editor AST Translation

Add translation functions in `src/editor_ast.rs`:

```rust
/// Translate StructureDef to EditorNode
pub fn translate_structure(def: &StructureDef) -> EditorNode {
    EditorNode::Operation {
        operation: OperationData {
            name: "structure".to_string(),
            args: vec![
                EditorNode::object(&def.name),
                translate_type_params(&def.type_params),
                translate_members(&def.members),
            ],
            kind: Some("kleis_structure".to_string()),
            metadata: Some(build_structure_metadata(def)),
        },
    }
}

/// Translate FunctionDef to EditorNode
pub fn translate_function_def(def: &FunctionDef) -> EditorNode {
    EditorNode::Operation {
        operation: OperationData {
            name: "define".to_string(),
            args: vec![
                EditorNode::object(&def.name),
                EditorNode::list(def.params.iter().map(|p| EditorNode::object(p)).collect()),
                translate_to_editor(&def.body),
            ],
            kind: Some("kleis_define".to_string()),
            metadata: def.type_annotation.as_ref().map(|t| build_type_metadata(t)),
        },
    }
}

/// Translate axiom to EditorNode
pub fn translate_axiom(name: &str, proposition: &Expression) -> EditorNode {
    EditorNode::Operation {
        operation: OperationData {
            name: "axiom".to_string(),
            args: vec![
                EditorNode::object(name),
                translate_to_editor(proposition),
            ],
            kind: Some("kleis_axiom".to_string()),
            metadata: None,
        },
    }
}
```

### Phase 2: Add Rendering Templates

In `src/render.rs`, add templates for each target:

```rust
// Typst templates
typst_templates.insert("structure".to_string(), 
    "**structure** {name}({params}) {{ {members} }}".to_string());
typst_templates.insert("define".to_string(), 
    "**define** {name}({params}) = {body}".to_string());
typst_templates.insert("axiom".to_string(), 
    "**axiom** {name} : {proposition}".to_string());

// LaTeX templates
latex_templates.insert("structure".to_string(), 
    "\\textbf{structure}~{name}({params})~\\{~{members}~\\}".to_string());
latex_templates.insert("define".to_string(), 
    "\\textbf{define}~{name}({params}) = {body}".to_string());
latex_templates.insert("axiom".to_string(), 
    "\\textbf{axiom}~{name} : {proposition}".to_string());
```

### Phase 3: Palette Templates (JavaScript)

In `static/index.html`, add AST templates:

```javascript
const astTemplates = {
    // ... existing templates ...
    
    // Kleis Language Constructs
    kleis_structure: {
        Operation: {
            name: 'structure',
            kind: 'kleis_structure',
            args: [
                { Placeholder: { id: 0, hint: 'name' } },
                { List: [{ Placeholder: { id: 1, hint: 'type_param' } }] },
                { List: [] }  // members start empty
            ],
            metadata: null
        }
    },
    
    kleis_define: {
        Operation: {
            name: 'define',
            kind: 'kleis_define',
            args: [
                { Placeholder: { id: 0, hint: 'name' } },
                { List: [{ Placeholder: { id: 1, hint: 'param' } }] },
                { Placeholder: { id: 2, hint: 'body' } }
            ],
            metadata: null
        }
    },
    
    kleis_axiom: {
        Operation: {
            name: 'axiom',
            kind: 'kleis_axiom',
            args: [
                { Placeholder: { id: 0, hint: 'name' } },
                { Placeholder: { id: 1, hint: 'proposition' } }
            ],
            metadata: null
        }
    },
    
    kleis_implements: {
        Operation: {
            name: 'implements',
            kind: 'kleis_implements',
            args: [
                { Placeholder: { id: 0, hint: 'structure_name' } },
                { List: [{ Placeholder: { id: 1, hint: 'type_arg' } }] },
                { List: [] }  // members start empty
            ],
            metadata: null
        }
    }
};
```

### Phase 4: Add Palette Tab

Add a new "Kleis" tab in the palette:

```html
<button class="palette-tab" onclick="showPalette('kleis', this)">Kleis</button>

<div id="palette-kleis" class="symbol-grid" style="display:none;">
    <button class="math-btn" onclick="insertTemplate('kleis_structure')" 
            data-tooltip="Structure Definition">structure</button>
    <button class="math-btn" onclick="insertTemplate('kleis_define')" 
            data-tooltip="Function Definition">define</button>
    <button class="math-btn" onclick="insertTemplate('kleis_axiom')" 
            data-tooltip="Axiom Declaration">axiom</button>
    <button class="math-btn" onclick="insertTemplate('kleis_implements')" 
            data-tooltip="Implements Block">implements</button>
    <button class="math-btn" onclick="insertTemplate('kleis_data')" 
            data-tooltip="Data Type">data</button>
</div>
```

---

## Visual Representation Options

### Option A: Block-Style (Code-like)

```
┌─ structure Monoid(M) ──────────────┐
│ operation (·) : M × M → M          │
│ element e : M                      │
│ axiom left_identity : ∀x. e·x = x  │
│ axiom right_identity : ∀x. x·e = x │
│ axiom associativity : ...          │
└────────────────────────────────────┘
```

### Option B: Mathematical Notation

```
Monoid(M) = {
  (·) : M × M → M
  e : M
  ∀x. e·x = x ∧ x·e = x
  ∀x y z. (x·y)·z = x·(y·z)
}
```

### Option C: Hybrid (Default)

Render as formatted text with inline math:

```
structure Monoid(M) {
  operation (·) : M × M → M
  element e : M
  axiom left_identity : ∀x. e · x = x
}
```

---

## Bi-directional Translation

### Kleis Text → Editor AST

```
Input: "define double(x) = x + x"
       ↓ parse_function_def()
       FunctionDef { name: "double", params: ["x"], body: plus(x, x) }
       ↓ translate_function_def()
       EditorNode::Operation { name: "define", kind: "kleis_define", ... }
```

### Editor AST → Kleis Text

```
EditorNode::Operation { name: "define", args: [...] }
       ↓ render(target: Kleis)
       "define double(x) = x + x"
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/editor_ast.rs` | Add `translate_structure()`, `translate_function_def()`, etc. |
| `src/render.rs` | Add Typst/LaTeX/HTML/Kleis templates for constructs |
| `static/index.html` | Add "Kleis" palette tab with templates |
| `src/bin/server.rs` | Ensure API handles new AST types |

---

## Dependencies

- Requires working PatternFly editor OR keep static/index.html alive
- Should work with existing placeholder system
- Need to handle nested structures (members contain members)

---

## Testing Strategy

1. **Unit tests**: Translate known Kleis AST → EditorNode → back to Kleis text
2. **Visual tests**: Render structure definitions in Typst, verify output
3. **Integration**: Load Kleis file → edit in editor → save back → verify valid Kleis

---

---

## Chunked Rendering (Important Architecture Note)

**Problem:** Can't load an entire notebook into Typst at once - too large.

**Solution:** Render in chunks, just like we edit equations in chunks.

```
┌─────────────────────────────────────────────────┐
│  Kleis Notebook (in memory)                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐           │
│  │ Cell 1  │ │ Cell 2  │ │ Cell 3  │ ...       │
│  └────┬────┘ └────┬────┘ └────┬────┘           │
└───────┼──────────┼──────────┼──────────────────┘
        │          │          │
        ▼          ▼          ▼
   ┌────────┐ ┌────────┐ ┌────────┐
   │ Typst  │ │ Typst  │ │ Typst  │  (independent renders)
   └────────┘ └────────┘ └────────┘
        │          │          │
        ▼          ▼          ▼
   ┌────────┐ ┌────────┐ ┌────────┐
   │  SVG   │ │  SVG   │ │  SVG   │  (displayed in UI)
   └────────┘ └────────┘ └────────┘
```

**Like Jupyter:**
- Each cell is an independent render unit
- Edit one cell → re-render only that cell
- Scroll through cells, render on-demand (virtualization)
- Export to LaTeX combines all cells into one document

**Benefits:**
- Fast editing (only touched cells re-render)
- Handles large notebooks
- Memory efficient

**Technical Hurdles:**
- **File system interaction** - Loading/saving large notebooks, lazy loading, partial saves
- **Scroll virtualization** - Only render visible cells, load more as user scrolls
- **Cell index management** - Inserting/deleting cells without re-indexing everything
- **Cross-cell references** - Cell 5 references result from Cell 2, need dependency tracking
- **Undo across cells** - Global undo stack or per-cell?

---

## Future Considerations (Not in Scope Yet)

| Topic | Question |
|-------|----------|
| **Bi-directional sync** | Edit in visual OR text mode - how to keep them in sync? |
| **Live verification** | Z3 checking as you type in the editor? |
| **Error highlighting** | Show type errors, unproved axioms visually? |
| **Auto-completion** | Suggest axioms/operations based on structure being defined? |
| **Templates library** | Pre-built structures (Ring, Field, VectorSpace) from stdlib? |
| **Git-friendly format** | `.kleis` files should diff cleanly |
| **Chunked rendering** | Can't load entire notebook in Typst - render cell-by-cell like Jupyter |

---

## Priority

Medium-High. This enables:
- Visual editing of Kleis axiom files
- Easier authoring of new structures
- Bridge between UI and formal language

Start with `define` (simplest), then `axiom`, then `structure` (most complex).

