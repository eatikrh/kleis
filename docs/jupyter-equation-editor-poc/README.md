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

### The Key Insight: Documents as Examples

Since documents are structures with axioms, we can use Kleis `example` blocks 
to **validate and compile** in one step - right in Jupyter!

```kleis
// In Jupyter Notebook

// Cell 1: Import document structures
import "stdlib/documents.kleis"

// Cell 2: Your data and figures
let convergence_data = [0.5, 0.25, 0.12, 0.06, 0.03]
let fig1 = diagram(plot(range(5), convergence_data), 
                   title = "Convergence", yscale = "log")

// Cell 3: The paper IS an example block!
example "arxiv_submission" {
    Paper(
        title = "A New Approach to Tensor Calculus",
        abstract = "We present a novel framework for symbolic 
                    computation in differential geometry that
                    enables automated verification of tensor
                    equations using SMT solvers...",
        
        sections = [
            Section("Introduction",
                text("The study of tensors began with Ricci..."),
                equation(R(μ,ν) - (1/2)*g(μ,ν)*R = 8*π*T(μ,ν))
            ),
            
            Section("Methods",
                text("Our implementation uses Z3..."),
                figure(fig1, caption = "Algorithm convergence")
            ),
            
            Section("Results",
                text("As shown in Figure 1..."),
                table(
                    headers = ["Method", "Accuracy", "Time"],
                    rows = [["Ours", "99.2%", "1.2s"],
                            ["Baseline", "87.1%", "5.4s"]]
                )
            ),
            
            Section("Conclusion",
                text("We have demonstrated that...")
            )
        ],
        
        style = "arxiv"
    )
}

// Running this cell:
// ✅ Validates: abstract > 100 words
// ✅ Validates: has Introduction section
// ✅ Validates: has Conclusion section  
// ✅ Validates: all figures have captions
// → Compiles to PDF via Typst
// → Displays PDF inline or download link
```

**Why this is elegant:**

| Traditional Workflow | Kleis Way |
|---------------------|-----------|
| Write LaTeX, hope it compiles | Structure defines validity |
| Forget required section → rejected by journal | Axiom fails → immediate feedback |
| Separate validation and compilation | `example` block does both |
| Manual template switching | Change `style = "ieee"` |

**The document definition IS the validation IS the compilation.**

### Different Document Types

```kleis
// PhD Thesis
example "dissertation" {
    Thesis(
        title = "Symbolic Methods in Differential Geometry",
        degree = "PhD",
        advisor = "Prof. Smith",
        chapters = [
            Chapter("Introduction", ...),
            Chapter("Background", ...),
            Chapter("Methodology", ...),
            Chapter("Results", ...),
            Chapter("Conclusion", ...)
        ],
        appendices = [Appendix("Proofs", ...)],
        style = "mit-thesis"
    )
}

// Nature submission (different axioms!)
example "nature_paper" {
    Paper(
        title = "...",
        // Nature has strict word limits
        abstract = "...",  // axiom: < 150 words
        sections = [...],  // axiom: no subsections allowed
        style = "nature"
    )
}

// Textbook
example "kleis_book" {
    Book(
        title = "The Kleis Programming Language",
        preface = "This book introduces...",
        chapters = [...],
        index = true,
        style = "textbook"
    )
}
```

### Status

🔮 **Not yet implemented** - This is the design direction based on POC findings.

**Implementation would require:**
1. Document structures in `stdlib/documents.kleis`
2. Example block output handler for document types
3. Typst template integration
4. PDF display in Jupyter output

---

## 9. Document Axiom Examples - VALIDATED ✅

We've created axiom-based document specifications for real-world formats.
**Both files parse and all examples pass!**

### MIT Thesis (`examples/documents/thesis_simple.kleis`)

Based on [MIT Libraries Thesis Specifications](https://libraries.mit.edu/distinctive-collections/thesis-specs/):

```kleis
// Data types
data DegreeLevel = Bachelor | Master | Doctor

// Validation function
define signature_valid(level: DegreeLevel, has_sig: Bool) = match level {
    Bachelor => True        // Bachelor's: signature optional
    | Master => has_sig     // Master's: signature required
    | Doctor => has_sig     // PhD: signature required
}

// Example validates the constraint
example "valid_phd" {
    let level = Doctor
    let has_sig = True
    
    assert(signature_valid(level, has_sig) = True)
    out("✓ Valid PhD configuration")
}
```

**Validated MIT Requirements:**
- ✅ PhD/Master's require signature page
- ✅ Bachelor's signature is optional
- ✅ Degree codes: sb (Bachelor), sm (Master), phd (Doctor)

**Test Results:** 5/5 examples pass

### arXiv Paper (`examples/documents/arxiv_simple.kleis`)

Based on typical arXiv structure:

```kleis
data ArxivCategory = CsPL | CsSE | CsAI | CsLG | MathLO | PhysicsGR | Other
data SectionType = Introduction | Background | Methods | Results | Conclusion | Other

define category_code(cat: ArxivCategory) = match cat {
    CsPL => "cs.PL"
    | MathLO => "math.LO"
    // ...
}

define has_valid_structure(has_intro: Bool, has_conclusion: Bool) =
    and(has_intro, has_conclusion)

example "valid_arxiv_paper" {
    let primary = CsPL
    assert(category_code(primary) = "cs.PL")
    assert(valid_author_count(2) = True)
    out("✓ Valid arXiv paper configuration verified")
}
```

**Validated arXiv Requirements:**
- ✅ Category codes (cs.PL, math.LO, gr-qc, etc.)
- ✅ Paper needs Introduction + Conclusion
- ✅ At least one author required
- ✅ Abstract length bounds (50-500 words)

**Test Results:** 9/9 examples pass

### What Works vs. What Needs Work

| Feature | Status | Notes |
|---------|--------|-------|
| Simple pattern matching | ✅ Works | `match level { Bachelor => ... }` |
| Data type definitions | ✅ Works | `data DegreeLevel = Bachelor \| Master \| Doctor` |
| Basic assertions | ✅ Works | `assert(func(x) = expected)` |
| Boolean functions | ✅ Works | `and()`, `or()`, `not()` |
| Nested function calls | ❌ Not evaluated | `get_field(get_struct(x))` stays symbolic |
| Complex `and` expressions | ⚠️ Partial | Need to split into separate assertions |
| Comparison to True/False | ✅ Works | `assert(foo(x) = True)` |

### Discovered Limitations

1. **User-defined accessor functions don't fully evaluate**
   - `get_title(thesis)` returns symbolic, not concrete value
   - Workaround: Test constraints with concrete values directly

2. **Nested `and` expressions with comparisons**
   - `and(x >= 50, x <= 500)` can cause type issues
   - Workaround: Split into separate assertions

3. **Match expressions need closing braces**
   - Syntax: `match x { Case1 => ... | Case2 => ... }`
   - First case has no `|`, subsequent cases do

### Technical Validation Summary

```bash
# Run the tests
./target/debug/kleis test examples/documents/thesis_simple.kleis
# ✅ 5 examples passed

./target/debug/kleis test examples/documents/arxiv_simple.kleis  
# ✅ 9 examples passed
```

These working examples validate the core design:
- Document constraints CAN be encoded as Kleis functions
- Example blocks CAN validate documents against constraints
- The "document as example" pattern WORKS

### Next Steps

1. ✅ **Parse test** - Files parse correctly
2. ✅ **Type check** - Functions type-check
3. ✅ **Axiom validation** - Constraints validate correctly
4. 🔮 **Simple output** - Generate Typst from valid document
5. 🔮 **PDF generation** - Compile Typst to PDF
6. 🔮 **Full structure axioms** - Use `structure` with quantified axioms

---

## 10. The Document = Program Insight

**Key Realization:** Compiling a thesis is structurally identical to parsing and executing a LISP program.

### The Isomorphism

| LISP Interpreter | Thesis Compiler |
|------------------|-----------------|
| Source: `"(+ 2 3)"` | Source: Jupyter cells |
| Parse → `SList(...)` | Parse → `ThesisDoc(...)` |
| Validate (type check) | Validate (axiom check) |
| Evaluate → `VNum(5)` | Compile → PDF |
| Environment: `Env(bindings)` | Style: Template + Axioms |

### Why LISP Works in Kleis

The LISP interpreter (`examples/meta-programming/lisp_parser.kleis`) successfully:
- Parses `"(+ 2 3)"` → `SList(SAtom("+"), SAtom("2"), SAtom("3"))`
- Evaluates → `VNum(5)` (concrete tagged value!)
- Handles recursion: `(fact 5)` → `VNum(120)`

**9/9 examples pass** including recursive factorial!

The key is **tagged return values**:

```kleis
// Every function returns a tagged concrete value
VNum(x + y)           // Not just a number - tagged as VNum
CTypst("#title[...]") // Not just a string - tagged as Typst code
CError("msg")         // Explicit error handling
```

### The Thesis Compiler Pattern

```kleis
// Document AST (like SExpr)
data DocExpr =
    DTitle(text: String)
  | DChapter(num: ℕ, title: String, sections: List(DocExpr))
  | DEquation(latex: String, label: String)
  | DFigure(num: String, caption: String, path: String)

// Compile result (like LispVal)
data CompileResult =
    CTypst(code: String)    // Typst code fragment
  | CError(message: String) // Validation failure
  | CValid                  // Structure OK

// Compile function (like eval_lisp)
define compile_doc(expr: DocExpr, style: DocStyle) : CompileResult =
    match expr {
        DTitle(text) => 
            CTypst(concat("#align(center)[", concat(text, "]")))
      | DEquation(latex, label) =>
            CTypst(concat("$ ", concat(latex, " $")))
      | DChapter(num, title, sections) =>
            merge_typst(compile_header(num, title), compile_all(sections, style))
      | _ => CTypst("")
    }
```

### The Complete Pipeline

```
Jupyter Cells → DocExpr AST → Validate(axioms) → Compile(style) → PDF
     ↑              ↑              ↑                 ↑            ↑
   Input         Parse          Check            Transform     Output
   (like         (like          (like            (like         (like
   LISP src)     parse_sexpr)   type check)      eval_lisp)    VNum)
```

### Implementation Path

1. **Define DocExpr** - Document AST (done conceptually)
2. **Define CompileResult** - Tagged output types
3. **Write compile_doc** - Pattern matching → Typst fragments
4. **Write validate_doc** - Check against style axioms
5. **Write run_thesis** - Full pipeline: validate → compile
6. **Integrate with Jupyter** - Parse cells → DocExpr → PDF display

This architecture mirrors the working LISP interpreter exactly!

---

## 10. References

- [Lilaq Documentation](https://github.com/lilaq-project/lilaq)
- [Typst Documentation](https://typst.app/docs/)
- [Typst Academic Templates](https://typst.app/universe/search?kind=templates&q=academic)
- [arXiv Submission Guidelines](https://info.arxiv.org/help/submit/index.html)

