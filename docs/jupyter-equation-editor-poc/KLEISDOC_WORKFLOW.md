# KleisDoc Workflow: From Creation to Publication

**Date:** January 2, 2026  
**Status:** Design  
**Goal:** Define the complete user journey for creating publishable documents

---

## Overview

KleisDoc enables a seamless workflow from initial idea to published document:

```
CREATE → AUTHOR → VERIFY → PREVIEW → ITERATE → PUBLISH
```

All work is persisted in `.kleis` files, enabling:
- Multi-session editing
- Re-editable equations (EditorNode AST preserved)
- Regenerable figures (Kleis code stored)
- Version control (Git-friendly text files)
- Verification at every step

---

## Phase 1: CREATE

### Starting a New Document

```python
# In Jupyter Notebook
from kleis.jupyter import KleisDoc, templates

# List available templates
templates.list()
# → MIT Thesis, arXiv Paper, IEEE Paper, Book Chapter, ...

# Create from template
thesis = KleisDoc.from_template("MIT Thesis")

# Or create blank
doc = KleisDoc.new()
```

### Template Selection

Templates provide:
- **Structure**: Required sections (abstract, chapters, bibliography)
- **Axioms**: Validation rules (word limits, formatting)
- **Styling**: Typst preamble (fonts, margins, headers)

```python
# Template info
thesis.template.info()
# → MIT Thesis
# → Required: Title, Abstract (≤350 words), 3+ chapters, Bibliography
# → Optional: Acknowledgments, Appendices
# → Style: US Letter, 1" margins, New Computer Modern font
```

### Metadata Entry

```python
# Set document metadata
thesis.set_metadata(
    title="Formal Verification of Knowledge Production Systems",
    author="Jane Smith",
    email="jane@mit.edu",
    department="Electrical Engineering and Computer Science",
    degree="Doctor of Philosophy",
    date="May 2025",
    supervisor="Prof. Alice Chen",
    keywords=["formal verification", "type theory", "scientific computing"]
)
```

---

## Phase 2: AUTHOR

### 2.1 Writing Prose

Use Markdown cells with `%%kleisdoc` magic:

```python
%%kleisdoc chapter:1 section:intro

# Introduction

Knowledge production in science and mathematics relies on precise 
notation and rigorous verification. Traditional approaches separate 
these concerns, leading to errors when notation outpaces verification.

This thesis presents **Kleis**, a unified framework that treats 
notation, verification, and document structure as first-class concepts.
```

The magic command tags the cell for extraction into the document.

### 2.2 Creating Equations

#### Visual Equation Editor

```python
from kleis.jupyter import equation_editor

# Open visual editor (iframe popup)
eq = equation_editor()

# User builds equation visually:
#   - Click templates (frac, integral, sum)
#   - Type symbols (Greek letters, operators)
#   - See live preview

# When user clicks "Insert", returns EditorNode
thesis.add_equation(
    eq,
    label="eq:einstein",
    chapter=1,
    numbered=True
)
```

#### Inline LaTeX (Quick Entry)

```python
# For simple equations, use LaTeX directly
thesis.add_equation_latex(
    r"E = mc^2",
    label="eq:mass-energy",
    chapter=1
)
```

#### Kleis Code (For Verification)

```python
# Equations that should be verified
thesis.add_equation_kleis(
    "forall x . P(x) => Q(x)",
    label="eq:logic",
    chapter=1,
    verify=True  # Will be checked by Z3
)
```

### 2.3 Creating Figures

#### From Kleis Plotting Code

```python
%%kleisdoc figure:fig:performance chapter:3

# This cell generates a figure
from kleis import plot, diagram

sizes = [10, 100, 1000, 10000]
times = [0.001, 0.01, 0.1, 1.0]

diagram(
    plot(sizes, times, color="blue", label="Kleis"),
    plot(sizes, [t*2 for t in times], color="red", label="Baseline"),
    title="Type Inference Performance",
    xlabel="Program Size",
    ylabel="Time (s)",
    x_scale="log",
    y_scale="log"
)
```

#### From External Image

```python
thesis.add_figure_image(
    path="figures/architecture.png",
    caption="System architecture overview",
    label="fig:arch",
    chapter=2
)
```

### 2.4 Adding Theorems and Definitions

```python
# Definition
thesis.add_definition(
    term="SMT Solver",
    definition="A Satisfiability Modulo Theories solver determines "
               "if a logical formula is satisfiable under a given theory.",
    chapter=2
)

# Theorem
thesis.add_theorem(
    statement="Every well-typed term in Kleis has a principal type "
              "that can be computed in polynomial time.",
    proof="By induction on the structure of terms...",  # Optional
    label="thm:principal-types",
    chapter=2
)

# Lemma (no proof given)
thesis.add_lemma(
    statement="The Christoffel symbols are symmetric in their lower indices.",
    label="lem:christoffel-symmetry",
    chapter=3
)
```

### 2.5 Cross-References

```python
# Reference an equation
thesis.add_text(
    "As shown in @eq:einstein, mass and energy are equivalent.",
    chapter=1
)

# Reference a figure
thesis.add_text(
    "The results are plotted in @fig:performance.",
    chapter=4
)

# Reference a theorem
thesis.add_text(
    "By @thm:principal-types, type inference is tractable.",
    chapter=2
)
```

### 2.6 Bibliography

```python
# Add citations
thesis.add_citation(
    key="demoura2008",
    authors="de Moura, L. and Bjørner, N.",
    title="Z3: An Efficient SMT Solver",
    venue="TACAS",
    year="2008",
    doi="10.1007/978-3-540-78800-3_24"
)

# Cite in text
thesis.add_text(
    "We use Z3 [@demoura2008] for axiom verification.",
    chapter=2
)
```

---

## Phase 3: VERIFY

### Running Verification

```python
# Verify entire document
result = thesis.verify()

if result.valid:
    print("✅ Document passes all checks")
else:
    for error in result.errors:
        print(f"❌ {error.location}: {error.message}")
    for warning in result.warnings:
        print(f"⚠️ {warning.location}: {warning.message}")
```

### What Gets Verified

#### Template Axioms
```
✅ Title present
✅ Abstract present (248 words, limit 350)
✅ At least 3 chapters (found 5)
✅ Bibliography present (5 entries)
⚠️ Acknowledgments missing (optional)
```

#### Mathematical Axioms
```
✅ eq:christoffel - Symmetry in lower indices verified
✅ eq:bianchi - Bianchi identity verified
❌ eq:custom - Could not verify: insufficient axioms
```

#### Cross-References
```
✅ All equations referenced in text
✅ All figures referenced in text
⚠️ fig:extra-diagram defined but never referenced
```

#### Citations
```
✅ All citations have bibliography entries
⚠️ demoura2008 cited but never defined (will use placeholder)
```

### Continuous Verification

```python
# Watch mode: verify as you edit
thesis.watch()
# → Runs verification after each cell execution
# → Shows inline warnings in notebook
```

---

## Phase 4: PREVIEW

### Quick Preview

```python
# Render current state to PDF and display
thesis.preview()
# → Opens PDF viewer in notebook or browser
```

### Incremental Preview

```python
# Preview single chapter (faster)
thesis.preview_chapter(3)
```

### Export Preview

```python
# Generate Typst source (for debugging)
print(thesis.to_typst())
```

---

## Phase 5: ITERATE

### Multi-Session Editing

```python
# End of session: Save
thesis.save("my_thesis.kleis")

# Next session: Load
thesis = KleisDoc.load("my_thesis.kleis")

# Everything preserved:
# - All text, equations, figures
# - EditorNode ASTs (equations re-editable!)
# - Figure source code (regenerable!)
# - Validation state
```

### Re-Editing Equations

```python
# Get existing equation
eq = thesis.get_equation("eq:einstein")

# Open in Equation Editor (pre-populated!)
updated_eq = equation_editor(initial=eq)

# Update in document
thesis.update_equation("eq:einstein", updated_eq)
```

### Regenerating Figures

```python
# Get figure with its source code
fig = thesis.get_figure("fig:performance")
print(fig.kleis_code)
# → Shows original Kleis plotting code

# Update with new data
fig.update_data(new_sizes, new_times)
fig.regenerate()  # Re-runs Kleis code
```

### Version Control

```bash
# .kleis files are text, Git-friendly
git add my_thesis.kleis
git commit -m "Add chapter 3 with tensor equations"
git diff HEAD~1 my_thesis.kleis  # See changes
```

---

## Phase 6: PUBLISH

### Export to PDF

```python
# Generate final PDF
thesis.export_pdf("my_thesis.pdf")
# → Compiles via Typst
# → Includes all figures, equations, bibliography
```

### Export to Typst

```python
# Get Typst source (for manual editing if needed)
thesis.export_typst("my_thesis.typ")
```

### Export to LaTeX (for arXiv)

```python
# Generate LaTeX for arXiv submission
thesis.export_latex("my_thesis/")
# → Creates:
#   my_thesis/
#   ├── main.tex
#   ├── chapters/
#   ├── figures/
#   └── bibliography.bib
```

### Export to HTML (for web)

```python
# Generate HTML version
thesis.export_html("my_thesis_web/")
```

---

## File Layout

### Single-File Mode

```
my_thesis.kleis          # Everything in one file
```

### Multi-File Mode (Large Documents)

```
my_thesis/
├── thesis.kleis              # Main document, metadata
├── chapters/
│   ├── 01_introduction.kleis
│   ├── 02_background.kleis
│   ├── 03_system.kleis
│   ├── 04_evaluation.kleis
│   └── 05_conclusion.kleis
├── equations/
│   ├── einstein.kleis        # Complex equations with AST
│   └── tensors.kleis
├── figures/
│   ├── performance.kleis     # Kleis source code
│   ├── performance.svg       # Cached render
│   └── architecture.png      # Static image
├── bibliography.kleis
└── output/
    ├── thesis.typ            # Generated Typst
    └── thesis.pdf            # Final PDF
```

### Import in Main Document

```kleis
// thesis.kleis
import "chapters/01_introduction.kleis"
import "chapters/02_background.kleis"
import "chapters/03_system.kleis"
import "chapters/04_evaluation.kleis"
import "chapters/05_conclusion.kleis"
import "bibliography.kleis"
```

---

## Jupyter Integration Details

### Magic Commands

| Magic | Purpose | Example |
|-------|---------|---------|
| `%%kleisdoc chapter:N` | Tag cell as chapter content | `%%kleisdoc chapter:1` |
| `%%kleisdoc figure:LABEL` | Tag cell as figure source | `%%kleisdoc figure:fig:perf` |
| `%%kleisdoc equation:LABEL` | Tag cell as equation | `%%kleisdoc equation:eq:main` |
| `%%kleisdoc abstract` | Tag cell as abstract | `%%kleisdoc abstract` |

### Cell Tags (Alternative)

Cells can be tagged via Jupyter's tag feature:
- `kleisdoc:chapter:1`
- `kleisdoc:figure:fig:perf`
- `kleisdoc:equation:eq:main`

### Equation Editor Integration

```python
from kleis.jupyter import equation_editor

# Option 1: Popup iframe
eq = equation_editor()

# Option 2: Inline widget (future)
eq = equation_editor(inline=True)

# Option 3: From URL (for testing)
eq = equation_editor(url="http://localhost:8080")
```

### Communication Protocol

```
Jupyter Notebook
      │
      │ 1. Display iframe with Equation Editor
      │ 2. postMessage: "ready"
      ↓
Equation Editor (iframe)
      │
      │ 3. User builds equation
      │ 4. postMessage: { type: "insert", ast: EditorNode }
      ↓
Jupyter Notebook
      │
      │ 5. Receive EditorNode JSON
      │ 6. Store in document
      ↓
thesis.add_equation(ast, label="eq:main")
```

---

## Questions to Resolve

1. **Kernel vs. Server**
   - Should Kleis run in the Jupyter kernel (Python subprocess)?
   - Or as a separate server (LSP-style)?

2. **Live Preview Latency**
   - How fast can we regenerate PDF?
   - Incremental Typst compilation?

3. **Offline Support**
   - Can everything work without network?
   - WASM Kleis for browser-only mode?

4. **Collaboration**
   - Real-time editing (like Google Docs)?
   - Or just Git-based collaboration?

---

## Implementation Priorities

### Phase 1: Core Workflow
- [ ] `KleisDoc` Python class
- [ ] Save/load `.kleis` files
- [ ] Basic PDF export via Typst

### Phase 2: Jupyter Integration
- [ ] Magic commands
- [ ] Equation Editor iframe
- [ ] Figure cell extraction

### Phase 3: Verification
- [ ] Template axiom checking
- [ ] Cross-reference validation
- [ ] Z3 integration for math axioms

### Phase 4: Polish
- [ ] Live preview
- [ ] Watch mode
- [ ] Multi-file documents

### Phase 5: Export Formats
- [ ] LaTeX export
- [ ] HTML export
- [ ] EPUB export (books)

