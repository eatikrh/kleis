# Jupyter + Equation Editor Integration POC

**Date:** January 2, 2026  
**Branch:** `feature/jupyter-equation-editor-poc`  
**Status:** POC Complete, Ready for Implementation

---

## Overview

This POC demonstrates how to embed visual widgets (like the Kleis Equation Editor) 
in Jupyter notebooks and export plots/equations to publication-ready formats.

## 1. Iframe Embedding POC

### Problem
How can we display the Equation Editor (a React web app) inside a Jupyter notebook cell?

### Solution: Iframe Embedding
We tested three methods of embedding an HTML widget in Jupyter:

| Method | Description | Result |
|--------|-------------|--------|
| **Direct IFrame** | `IPython.display.IFrame(url)` | ✅ Works |
| **Toggle Button** | Show/hide with JavaScript button | ✅ Works |
| **Message Passing** | `postMessage` from iframe to kernel | ✅ Works |

### Files Created
```
examples/jupyter-iframe-poc/
├── README.md              # Usage instructions
├── simple_widget.html     # Test widget with symbol palette
└── test_iframe.ipynb      # Jupyter notebook with 3 methods
```

### How to Test
```bash
# Terminal 1: Start widget server
cd examples/jupyter-iframe-poc
python3 -m http.server 9999

# Terminal 2: Start Jupyter
cd kleis-notebook && ./start-jupyter.sh

# Open: examples/jupyter-iframe-poc/test_iframe.ipynb
```

### Key Findings

1. **Cross-origin works**: Jupyter (port 8891) can embed localhost widgets (port 9999)
2. **Toggle UI works**: JavaScript in notebook cells controls iframe visibility
3. **postMessage works**: Widget can send data back to notebook

### Architecture for Real Equation Editor

```
┌─────────────────────────────────────────────────────────────┐
│  Jupyter Notebook Cell                                       │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ [📐 Open Equation Editor]                                ││
│  │                                                          ││
│  │ ┌──────────────────────────────────────────────────────┐││
│  │ │  Equation Editor (iframe)                            │││
│  │ │  - Template palette                                  │││
│  │ │  - Visual equation building                          │││
│  │ │  - Live SVG preview                                  │││
│  │ │  [Insert into Notebook]                              │││
│  │ └──────────────────────────────────────────────────────┘││
│  │                                                          ││
│  │ Received equation: ∫₀^∞ e^{-x²} dx = √π/2              ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Implementation Steps (Future)

1. Add `?mode=jupyter` parameter to Equation Editor
2. Implement `postMessage` handler in editor to send:
   - SVG output
   - LaTeX representation
   - Kleis AST
3. Add Python helper in kernel: `equation_editor()`
4. Handle CORS if needed

---

## 2. Export Typst Functions

### Problem
How can users collect SVG plots/equations and compile them into a PDF document?

### Solution: `export_typst()` and `export_typst_fragment()`

Two new functions that return Typst code as strings:

| Function | Returns | Use Case |
|----------|---------|----------|
| `export_typst(...)` | Complete Typst code with preamble | Standalone `.typ` file |
| `export_typst_fragment(...)` | Just `lq.diagram(...)` call | Embed in existing document |

### Files Modified
- `src/plotting.rs` - Added `export_diagram_typst()` and `export_diagram_typst_fragment()`
- `src/evaluator.rs` - Added `builtin_export_typst()` and `builtin_export_typst_fragment()`
- `examples/export/export_typst_demo.kleis` - Demo and tests

### Usage

```kleis
// Get complete Typst file
let code = export_typst(
    plot([0, 1, 2, 3], [1, 4, 9, 16], color = "blue"),
    title = "My Plot",
    xlabel = "x"
)
out(code)
```

Output:
```typst
#import "@preview/lilaq:0.5.0" as lq
#set page(width: auto, height: auto, margin: 0.5cm)

#lq.diagram(
  title: [My Plot],
  xlabel: [x],
  lq.plot((0, 1, 2, 3), (1, 4, 9, 16), color: blue),
)
```

### PDF Workflow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Kleis Code    │     │   Typst File    │     │      PDF        │
│                 │     │                 │     │                 │
│ export_typst()  │ ──► │ paper.typ       │ ──► │ paper.pdf       │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                              │
                              ▼
                        typst compile paper.typ
```

### Example: Multi-Plot Document

```typst
// paper.typ
#import "@preview/lilaq:0.5.0" as lq

#set document(title: "Research Results")
#set page(paper: "a4", margin: 2cm)

= Introduction

Our study examines...

= Results

#figure(
  // Paste from export_typst_fragment()
  lq.diagram(
    title: [Figure 1],
    lq.plot((0, 1, 2, 3), (1, 4, 9, 16)),
  ),
  caption: [Experimental results]
)

= Conclusion

We conclude that...
```

---

## 3. LaTeX/arXiv Considerations

### Key Finding
**Typst does NOT generate LaTeX.** It compiles directly to PDF.

### arXiv Submission Options

| Approach | Works? | Notes |
|----------|--------|-------|
| Submit PDF directly | ✅ | arXiv accepts PDF! |
| Submit Typst source | ❌ | arXiv can't render Typst |
| Add LaTeX export to Kleis | 🔮 Future | Would need PGFPlots translation |

### Typst Academic Templates

Typst has academic templates that mimic arXiv/journal styles:

```typst
#import "@preview/charged-ieee:0.1.3": ieee
#import "@preview/arkheion:0.1.0": arkheion  // arXiv-like

#show: arkheion.with(
  title: "My Paper",
  authors: (
    (name: "Author One", affiliation: "University"),
  ),
)
```

### Future: LaTeX Export

If needed, we could add:

```kleis
// Hypothetical - not implemented
let latex = export_latex(
    plot([0,1,2,3], [1,4,9,16])
)
// Returns PGFPlots/TikZ code
```

This would require translating Lilaq → PGFPlots, which is significant work.

---

## 4. Future Enhancements

### High Priority
- [ ] `save_typst(filename, ...)` - Write Typst code to file
- [ ] `compile_pdf(filename, ...)` - Compile directly to PDF
- [ ] Integrate real Equation Editor with `?mode=jupyter`

### Medium Priority
- [ ] `export_latex()` - Generate PGFPlots for arXiv compatibility
- [ ] Typst document builder in Kleis
- [ ] Jupyter magic command: `%%kleis_typst`

### Low Priority
- [ ] WASM-based Typst compilation in browser (no server needed)
- [ ] Custom arXiv template for Kleis documents

---

## 5. Commits in This Branch

```
8b580e6 feat: Add Jupyter iframe POC for Equation Editor embedding
2c25c81 feat: Add export_typst() and export_typst_fragment() functions
```

---

## 6. Testing

```bash
# Test export_typst functions
cd /Users/eatik_1/git/cee/kleis
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
cargo run --bin kleis -- test examples/export/export_typst_demo.kleis

# Expected output:
# ✅ export full typst
# ✅ export fragment  
# ✅ build document
# ✅ 3 examples passed
```

---

## 8. Future: Persistent Publication Format (.kleisdoc)

### The Problem

Documents (papers, theses, books) are written over weeks, months, or years across 
hundreds of sessions. We need:
- Persistent storage that survives sessions
- Equation ASTs that can be reloaded and edited
- Regenerable content (plots from code) vs static (images)
- Structure validation against axioms
- Multi-format output (arXiv, IEEE, thesis templates)

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interfaces                           │
│  Jupyter Notebook │ Equation Editor │ CLI │ VS Code            │
└─────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────┐
│                    .kleisdoc Format                              │
│  - Metadata (title, authors, dates)                             │
│  - Content chunks with stable IDs (tags)                        │
│  - Equation ASTs (EditorNode format - reloadable!)              │
│  - Figure specs (regenerable from Kleis code)                   │
│  - Document outline (structure)                                 │
│  - Cross-references                                              │
└─────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────┐
│              Structure Definitions (Kleis)                       │
│  thesis-phd.kleis │ paper-arxiv.kleis │ book.kleis             │
│  (Axioms: "must have abstract > 150 words")                     │
└─────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────┐
│                    Typst Templates                               │
│  mit-thesis.typ │ arxiv.typ │ ieee.typ │ nature.typ            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                        PDF / HTML / EPUB
```

### Content Types

| Type | Regenerable? | Storage | Example |
|------|--------------|---------|---------|
| **Equation** | ✅ Yes | EditorNode AST | Einstein field equations |
| **Plot** | ✅ Yes | Kleis source code | Convergence diagram |
| **Table** | ✅ Yes | Data + format spec | Results comparison |
| **Text** | ❌ No | Markdown | Introduction paragraph |
| **Image** | ❌ No | File reference | Lab photo |

### Tag System

Content chunks have stable IDs that persist across sessions:

```
%tag eq-einstein     ← Created Year 1, edited Year 3, same ID
%tag fig-convergence ← Data updated, same ID
%tag ch2-intro       ← Text revised 50 times, same ID
```

### Cross-Session Workflow

```bash
# Session 1 (January 2024)
kleis new thesis.kleisdoc --template mit-thesis
jupyter lab  # Work on Chapter 1
kleis save

# Session 203 (October 2025)
kleis open thesis.kleisdoc
# Click eq-einstein → Equation Editor loads AST
# Edit visually → Save back to .kleisdoc

# Session 412 (January 2026)
kleis validate thesis.kleisdoc
kleis compile thesis.kleisdoc --style mit-thesis → thesis.pdf
```

### Key Design Principles

1. **Stable IDs**: Content chunks keep their ID forever, enabling:
   - Cross-references: `See Equation {@eq-einstein}`
   - Structure reorganization without breaking links
   - Version tracking per chunk

2. **AST Preservation**: Equations store EditorNode AST, not just rendered output
   - Equation Editor can reload and edit
   - Multiple render targets (Typst, LaTeX, MathML)

3. **Regenerable Content**: Plots/tables store source code
   - Update data → regenerate output
   - Change style → re-render all figures

4. **Separation of Concerns**:
   - Content (what you wrote) → .kleisdoc
   - Structure (what it must be) → Kleis axioms
   - Style (how it looks) → Typst templates

### Status

🔮 **Not yet implemented** - This is the design direction based on POC findings.

---

## 9. References

- [Lilaq Documentation](https://github.com/lilaq-project/lilaq)
- [Typst Documentation](https://typst.app/docs/)
- [Typst Academic Templates](https://typst.app/universe/search?kind=templates&q=academic)
- [arXiv Submission Guidelines](https://info.arxiv.org/help/submit/index.html)

