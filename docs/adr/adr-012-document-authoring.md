# ADR-012: Kleis as Document Authoring Environment

## Status
**Implemented** - January 3, 2026  
**Updated:** January 3, 2026 - Revised to reflect pure Kleis implementation

## Context

Kleis should be a **complete authoring environment** for mathematical documents that can export to:
- PDF (via Typst)
- HTML (future)
- LaTeX (future)

**Key Principle:** Documents are Kleis programs, not Python objects.

## Decision: Documents Are Programs

Rather than a Python API for building documents, we chose:

1. **Templates in Kleis** - Document styles defined in `stdlib/templates/*.kleis`
2. **Documents in Kleis** - Each document is a `.kleis` file
3. **Python as thin shell** - Only calls `kleis test` and `typst compile`

### Why This Approach?

| Old Approach (Rejected) | New Approach (Implemented) |
|------------------------|---------------------------|
| Python `KleisDoc` class | Pure Kleis data types |
| `doc.add_section("...")` | `MITChapter(1, "...", "...")` |
| Complex Python methods | Simple constructors |
| Python knows about styling | Templates are Kleis code |
| 3000+ lines of Python | 227 lines thin shell |

**Benefits:**
- Single source of truth (the `.kleis` file)
- Version control friendly
- Templates are user-extensible without Python
- Kleis handles all the heavy lifting
- Consistent with "Kleis defines Kleis" philosophy

---

## Implementation

### Available Templates

```
stdlib/templates/
├── mit_thesis.kleis      # MIT PhD dissertations
├── uofm_thesis.kleis     # University of Michigan Rackham
└── arxiv_paper.kleis     # arXiv preprints
```

### Document Structure

A document is a `.kleis` file that:
1. Imports a template
2. Defines content elements
3. Assembles them into a document
4. Compiles via an example block

```kleis
import "stdlib/templates/mit_thesis.kleis"

// Define content
define ch1 = MITChapter(1, "Introduction", "This thesis...")
define eq1 = MITEquation("eq:einstein", "$ E = m c^2 $")

// Assemble
define my_thesis = mit_thesis(
    "My Title",
    "Jane Smith",
    "Department of EECS",
    "May 2026",
    PhD,
    "Abstract text...",
    "Prof. Advisor",
    "Professor of CS",
    "Acknowledgments...",
    "Dedication...",
    [ch1, eq1]
)

// Compile
example "compile" {
    let typst = compile_mit_thesis(my_thesis)
    out(typst)
}
```

### Content Elements

Each template defines data types for content:

| Template | Content Types |
|----------|---------------|
| **MIT** | `MITChapter`, `MITSection`, `MITEquation`, `MITFigure`, `MITTable`, `MITDiagram`, `MITReference`, `MITAppendix`, `MITAcknowledgments`, `MITDedication` |
| **UofM** | `UMichChapter`, `UMichSection`, `UMichEquation`, `UMichFigure`, `UMichTable`, `UMichDiagram`, `UMichReference`, `UMichAppendix` |
| **arXiv** | `ArxivSection`, `ArxivEquation`, `ArxivFigure`, `ArxivTable`, `ArxivDiagram`, `ArxivAlgorithm`, `ArxivReference`, `ArxivAppendix`, `ArxivAcknowledgments` |

### Diagrams with Lilaq

Documents can include plots via [Lilaq](https://typst.app/universe/package/lilaq):

```kleis
define fig_perf = MITDiagram("fig:perf", "Performance comparison", "
import \"@preview/lilaq:0.5.0\" as lq
lq.diagram(
    lq.plot((1, 2, 3, 4, 5), (10, 25, 40, 55, 70)),
    xlabel: \"Iteration\",
    ylabel: \"Accuracy (%)\"
)")
```

### PDF Generation

```bash
# Compile Kleis → Typst
kleis test my_thesis.kleis > my_thesis.typ

# Compile Typst → PDF
typst compile my_thesis.typ my_thesis.pdf
```

Or via Python shell:

```python
from kleis_kernel import compile_to_pdf
compile_to_pdf("my_thesis.kleis", "my_thesis.pdf")
```

---

## Template Architecture

Each template file defines:

1. **Data types** - Document structure (chapters, sections, etc.)
2. **Typst styling** - Page setup, fonts, headings
3. **Section templates** - Title page, abstract, TOC, etc.
4. **Compile functions** - Assemble document into Typst

### Example: MIT Thesis Template Structure

```kleis
// 1. Data types
data MITDegree = SB | SM | PhD
data MITDocExpr = 
    MITChapter(num: ℕ, title: String, content: String)
  | MITEquation(label: String, typst_code: String)
  | MITFigure(label: String, caption: String, typst_code: String)
  | ...

// 2. Typst styling
define typst_mit_page_setup = "#set page(paper: \"us-letter\", ...)"
define typst_mit_text_setup = "#set text(font: \"New Computer Modern\", ...)"

// 3. Section templates
define typst_mit_title_page = "#align(center)[...]"
define typst_mit_abstract = "#align(center)[Abstract]..."

// 4. Compile function
define compile_mit_thesis(thesis) = match thesis {
    MITThesisDoc(...) =>
        concat(typst_mit_page_setup,
        concat(typst_mit_text_setup,
        concat(typst_mit_title_page,
        ...)))
}
```

---

## Known Limitations

### Inline Math in Prose

Currently NOT supported:
> "A feedforward neural network is a function f: ℝⁿ → ℝᵐ composed of..."

**Workaround:** Use separate equation blocks or plain text descriptions.

**Tracked in:** `docs/NEXT_SESSION.md` → "FUTURE: Inline Math in Document Text"

---

## Example Documents

| Document | Template | Purpose |
|----------|----------|---------|
| `examples/documents/jane_smith_thesis.kleis` | MIT | Complete PhD thesis (`kleis test <file> > thesis.typ`) |
| `examples/documents/alex_chen_dissertation.kleis` | UofM | Rackham dissertation (`kleis test <file> > dissertation.typ`) |
| `examples/documents/sample_arxiv_paper.kleis` | arXiv | Research paper (`kleis test <file> > paper.typ`) |

---

## Creating Custom Templates

To add a new template (e.g., IEEE, Nature):

1. Create `stdlib/templates/ieee_paper.kleis`
2. Define data types for content elements
3. Define Typst styling strings
4. Define compile function
5. Done - no Rust or Python changes needed

---

## Related ADRs

- [ADR-011](adr-011-notebook-environment.md) - Jupyter integration
- [ADR-015](adr-015-text-as-source-of-truth.md) - Text representation
- [ADR-023](ADR-023-kleist-template-externalization.md) - Equation editor templates (`.kleist`)

---

## References

- Manual: [Chapter 23: Document Generation](https://kleis.io/docs/manual/book/chapters/23-document-generation.html)
- Templates: `stdlib/templates/`
- Examples: `examples/documents/`
