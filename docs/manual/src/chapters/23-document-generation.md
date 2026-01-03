# Document Generation with KleisDoc

Kleis provides a complete document generation system for creating publication-quality documents like PhD theses, conference papers, and journal articles. This chapter covers everything you need to write your thesis using Kleis and Jupyter notebooks.

## Why KleisDoc?

Traditional academic writing separates:
- **Code** (Jupyter notebooks, Python scripts)
- **Equations** (LaTeX, copy-pasted into Word)
- **Plots** (matplotlib, exported as images)
- **Document** (Word, LaTeX, Google Docs)

This separation leads to:
- Copy-paste errors between code and paper
- Equations that can't be re-edited
- Plots that can't be regenerated
- No verification of mathematical claims

**KleisDoc unifies everything:**

```
┌─────────────────────────────────────────────────────────────┐
│                    KleisDoc (.kleis)                        │
├─────────────────────────────────────────────────────────────┤
│  Metadata      │ Title, authors, date, abstract             │
│  Equations     │ Stored as AST (re-editable!)               │
│  Plots         │ Kleis code (regenerable!)                  │
│  Tables        │ Structured data                            │
│  Text          │ Markdown-like sections                     │
│  Bibliography  │ BibTeX entries                             │
│  Cross-refs    │ Validated references                       │
└─────────────────────────────────────────────────────────────┘
                          ↓
              Template-driven export
                          ↓
    ┌─────────┬─────────┬─────────┬─────────┐
    │ MIT     │ arXiv   │ UofM    │ Custom  │
    │ Thesis  │ Paper   │ Thesis  │ Journal │
    └─────────┴─────────┴─────────┴─────────┘
                          ↓
                   PDF / LaTeX
```

## Quick Start

### Prerequisites

1. **Python 3.8+** with Jupyter
2. **Kleis** compiled and in PATH
3. **Typst** for PDF generation: `brew install typst`
4. **Kleis server** running (for equation rendering)

### Minimal Example

```python
from kleis_kernel.kleisdoc import KleisDoc, Author

# Create a document
doc = KleisDoc()
doc.set_metadata(
    title="My PhD Thesis",
    authors=[Author(name="Jane Smith", affiliation="MIT")],
    date="2026",
    abstract="This thesis explores..."
)

# Add content
doc.add_section("Introduction", level=1)
doc.add_text("This thesis presents novel results in...")

# Add an equation
doc.add_equation("quadratic", r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}")

# Export to PDF
doc.export_pdf(
    "thesis.pdf",
    template_path="stdlib/templates/mit_thesis.kleis"
)
```

## Installation

### 1. Install Kleis

```bash
cd /path/to/kleis
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
cargo install --path . --features numerical
```

### 2. Install Typst

```bash
# macOS
brew install typst

# Linux
curl -fsSL https://typst.app/install.sh | sh

# Windows
winget install typst
```

### 3. Install KleisDoc Python Package

```bash
cd kleis-notebook
pip install -e .
```

### 4. Start Kleis Server (for equation rendering)

```bash
kleis server --port 8080
```

## Document Styles

KleisDoc uses template files to define document styling. Each template is a `.kleis` file that specifies:
- Page margins and size
- Fonts and sizes
- Title page layout
- Heading styles
- Bibliography format

### Available Templates

| Template | Path | Use Case |
|----------|------|----------|
| **MIT Thesis** | `stdlib/templates/mit_thesis.kleis` | MIT PhD dissertations |
| **UofM Thesis** | `stdlib/templates/uofm_thesis.kleis` | University of Michigan Rackham theses |
| **arXiv Paper** | `stdlib/templates/arxiv_paper.kleis` | arXiv preprints, conference papers |

### Creating Custom Templates

To add a new style (e.g., IEEE, Nature), create a `.kleis` file:

```kleis
// stdlib/templates/ieee_paper.kleis

define template_name = "IEEE Paper"
define template_type = "paper"

// Page setup
define typst_page_setup = "#set page(paper: \"us-letter\", margin: (top: 0.75in, bottom: 1in, left: 0.625in, right: 0.625in), columns: 2)"

// Font setup
define typst_text_setup = "#set text(font: \"Times New Roman\", size: 10pt)"

// Title block
define typst_title_block = "
#align(center)[
  #text(size: 24pt, weight: \"bold\")[TITLE]
  #v(0.5em)
  #text(size: 11pt)[AUTHOR]
  #v(0.25em)
  #text(size: 10pt, style: \"italic\")[AFFILIATION]
]
"

// ... more definitions
```

**No Python or Rust code changes needed** — just add the template file!

## Creating a Document

### Setting Metadata

```python
from kleis_kernel.kleisdoc import KleisDoc, Author

doc = KleisDoc()

# Single author
doc.set_metadata(
    title="Quantum Entanglement in Topological Systems",
    authors=[Author(
        name="Jane Smith",
        affiliation="MIT Department of Physics",
        email="jane@mit.edu",
        orcid="0000-0001-2345-6789"
    )],
    date="January 2026",
    abstract="""
    We investigate quantum entanglement properties in topological phases
    of matter. Our results show that...
    """
)

# Multiple authors
doc.set_metadata(
    title="Collaborative Research",
    authors=[
        Author(name="Alice", affiliation="MIT"),
        Author(name="Bob", affiliation="Stanford"),
        Author(name="Carol", affiliation="Berkeley")
    ],
    date="2026"
)
```

### Adding Sections

```python
# Top-level section (Chapter in thesis, Section in paper)
intro = doc.add_section("Introduction", level=1)

# Subsection
motivation = doc.add_section("Motivation", level=2, parent=intro)

# Sub-subsection
doc.add_section("Historical Context", level=3, parent=motivation)
```

### Adding Text

```python
doc.add_text("""
The study of quantum entanglement has revolutionized our understanding
of quantum mechanics. In this work, we present new results that extend
previous findings by Smith et al.
""")

# Text in a specific section
doc.add_text("This section covers the basics.", section=intro)
```

## Equations

### Simple LaTeX Equations

```python
doc.add_equation(
    label="schrodinger",
    latex=r"i\hbar\frac{\partial}{\partial t}\Psi = \hat{H}\Psi"
)
```

### Re-editable Equations with AST

For equations you want to edit later, store the AST:

```python
# Quadratic formula with full AST
quadratic_ast = {
    "Operation": {
        "name": "equals",
        "args": [
            {"Object": "x"},
            {
                "Operation": {
                    "name": "frac",
                    "args": [
                        {
                            "Operation": {
                                "name": "plus_minus",
                                "args": [
                                    {"Operation": {"name": "negate", "args": [{"Object": "b"}]}},
                                    {
                                        "Operation": {
                                            "name": "sqrt",
                                            "args": [{
                                                "Operation": {
                                                    "name": "minus",
                                                    "args": [
                                                        {"Operation": {"name": "power", "args": [{"Object": "b"}, {"Const": "2"}]}},
                                                        {"Operation": {"name": "times", "args": [{"Const": "4"}, {"Object": "a"}, {"Object": "c"}]}}
                                                    ]
                                                }
                                            }]
                                        }
                                    }
                                ]
                            }
                        },
                        {"Operation": {"name": "times", "args": [{"Const": "2"}, {"Object": "a"}]}}
                    ]
                }
            }
        ]
    }
}

doc.add_equation(
    label="quadratic",
    latex=None,  # Will be rendered from AST
    ast=quadratic_ast
)
```

### Using the Equation Editor

The Kleis Equation Editor provides a visual way to build equations:

#### In Jupyter Notebooks

```python
from kleis_kernel.equation_editor import equation_editor

# Open the visual equation editor in an iframe
equation_editor()
```

The editor opens with:
- **Structural Mode** for building equations visually
- **Symbol Palette** with math symbols, Greek letters, and templates
- **"📤 Send to Jupyter"** button to send the equation back

#### Workflow

1. **Open the editor:**
   ```python
   equation_editor()
   ```

2. **Build your equation** using the palette buttons

3. **Click "📤 Send to Jupyter"** when done

4. **Retrieve the AST and add to document:**
   ```python
   # After clicking "Send to Jupyter", the AST is stored
   # Add to your document:
   doc.add_equation_from_ast("eq:main", equation_ast)
   ```

#### Re-editing Existing Equations

```python
# Load an existing equation's AST into the editor
eq = doc.get_equation("eq:main")
equation_editor(initial=eq.ast)  # Pre-populates the editor!
```

#### Standalone Mode

For standalone use (outside Jupyter):

1. **Start Kleis server:** `kleis server --port 3000`
2. **Open in browser:** `http://localhost:3000/`
3. **Use Debug AST** to copy the JSON AST
4. **Paste into Python:** `doc.add_equation("label", ast=...)`

For Jupyter mode, the URL `http://localhost:3000/?mode=jupyter` enables
the "Send to Jupyter" button and auto-switches to structural mode

## Figures and Plots

### Kleis Plots (Regenerable)

```python
# Store the Kleis code - plot will be regenerated on export
doc.add_figure(
    label="fig:sine",
    caption="A sine wave demonstrating periodic behavior",
    kleis_code="""
let xs = linspace(0, 6.28, 100)
let ys = list_map(lambda x . sin(x), xs)
diagram(
    plot(xs, ys, color = "blue", stroke = "2pt"),
    title = "Sine Wave",
    xlabel = "x",
    ylabel = "sin(x)"
)
"""
)
```

### Static Images

```python
doc.add_figure(
    label="fig:experiment",
    caption="Experimental apparatus",
    image_path="images/apparatus.png",
    width="80%"
)
```

## Tables

```python
doc.add_table(
    label="tab:results",
    caption="Experimental results comparing methods",
    headers=["Method", "Accuracy", "Runtime"],
    rows=[
        ["Baseline", "72.3%", "1.2s"],
        ["Our Method", "89.7%", "0.8s"],
        ["State-of-art", "87.1%", "2.4s"]
    ]
)
```

### Computed Tables

```python
# Table generated from Kleis computation
doc.add_table(
    label="tab:eigenvalues",
    caption="Eigenvalues of test matrices",
    headers=["Matrix", "λ₁", "λ₂"],
    kleis_code="""
let A = [[1, 2], [3, 4]]
let B = [[2, 1], [1, 2]]
let eig_A = eigenvalues(A)
let eig_B = eigenvalues(B)
// Returns table data
"""
)
```

## Cross-References

### Referencing Equations

```python
doc.add_equation("maxwell", r"\nabla \cdot \mathbf{E} = \frac{\rho}{\epsilon_0}")

doc.add_text("""
As shown in Equation @eq:maxwell, the divergence of the electric field
is proportional to the charge density.
""")

# Or programmatically
ref = doc.ref_equation("maxwell")  # Returns CrossRef object
```

### Referencing Figures and Tables

```python
doc.add_text("See Figure @fig:sine for the sine wave plot.")
doc.add_text("Results are summarized in Table @tab:results.")

# Programmatic references
doc.ref_figure("sine")
doc.ref_table("results")
```

### Referencing Sections

```python
intro = doc.add_section("Introduction", level=1, label="sec:intro")

doc.add_text("As discussed in Section @sec:intro, we begin with...")

# Programmatic
doc.ref_section("intro")
```

### Validating Cross-References

```python
errors = doc.validate_cross_refs()
if errors:
    print("Cross-reference errors:")
    for error in errors:
        print(f"  - {error}")
else:
    print("All cross-references valid!")
```

## Bibliography

### Adding Citations

```python
# Add bibliography entries
doc.add_bib_entry(
    key="einstein1905",
    entry_type="article",
    title="On the Electrodynamics of Moving Bodies",
    author="Albert Einstein",
    journal="Annalen der Physik",
    year="1905",
    volume="17",
    pages="891-921"
)

doc.add_bib_entry(
    key="dirac1928",
    entry_type="article",
    title="The Quantum Theory of the Electron",
    author="P. A. M. Dirac",
    journal="Proceedings of the Royal Society A",
    year="1928"
)

# Cite in text
doc.add_text("""
The theory of special relativity @cite:einstein1905 revolutionized physics.
Later, Dirac's equation @cite:dirac1928 unified quantum mechanics with
special relativity.
""")
```

### Export Bibliography

```python
# Export BibTeX file
doc.export_bibtex("references.bib")
```

## Saving and Loading

### Saving Your Document

KleisDoc saves to `.kleis` files (pure Kleis code):

```python
doc.save("my_thesis.kleis")
```

The saved file looks like:

```kleis
// KleisDoc: My PhD Thesis
// Generated: 2026-01-03

define meta_title = "My PhD Thesis"
define meta_date = "2026"
define meta_abstract = "This thesis explores..."

define author_0 = Author(
    name = "Jane Smith",
    affiliation = "MIT",
    email = "jane@mit.edu"
)
define doc_authors = List(author_0)

define equation_quadratic = Equation(
    label = "quadratic",
    latex = "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}",
    ast = { ... }  // Full AST preserved!
)

define section_0 = Section(
    level = 1,
    title = "Introduction",
    label = "sec:intro",
    content = List(...)
)

// ... rest of document
```

### Loading a Document

```python
# Load for continued editing
doc = KleisDoc.load("my_thesis.kleis")

# Continue editing
doc.add_section("New Chapter", level=1)
doc.save("my_thesis.kleis")
```

### Revising Content

Thesis writing is iterative. Use these methods to revise existing content:

**Update equations:**
```python
# Change the LaTeX
doc.update_equation("loss", latex=r"\mathcal{L}_{revised} = \frac{1}{n}\sum(y - \hat{y})^2")

# Or update the AST for Equation Editor compatibility
doc.update_equation("einstein", ast=new_ast_dict)
```

**Update sections:**
```python
# Change section content
doc.update_section("Introduction", content="This thesis explores quantum computing...")

# Rename a section
doc.update_section("Intro", new_title="Chapter 1: Introduction")

# Add a label for cross-referencing
doc.update_section("Methods", label="sec:methods")
```

**Update figures and tables:**
```python
doc.update_figure("fig1", caption="Updated architecture diagram")
doc.update_table("tab1", rows=[[1, 2, 3], [4, 5, 6]])
```

**Remove content:**
```python
# Clean up before submission
doc.remove_section("Draft Notes")
doc.remove_equation("old_eq")
doc.remove_figure("placeholder_fig")
doc.remove_table("temp_data")
```

**Find content:**
```python
# Get specific items
eq = doc.get_equation("loss")
section = doc.get_section("Methods")
fig = doc.get_figure("architecture")
table = doc.get_table("results")
```

### Multi-Session Workflow

```
Session 1 (Monday):
  → Create document
  → Write Chapter 1
  → Save to my_thesis.kleis

Session 2 (Tuesday):
  → Load my_thesis.kleis
  → Edit equations (AST preserved!)
  → Write Chapter 2
  → Save

Session 3 (Friday):
  → Load my_thesis.kleis
  → Regenerate plots (code preserved!)
  → Final edits
  → Export to PDF
```

## Exporting to PDF

### Basic Export

```python
doc.export_pdf("thesis.pdf")
```

### With Template

```python
# MIT Thesis format
doc.export_pdf(
    "thesis.pdf",
    template_path="stdlib/templates/mit_thesis.kleis"
)

# arXiv paper format
doc.export_pdf(
    "paper.pdf", 
    template_path="stdlib/templates/arxiv_paper.kleis"
)
```

### What Happens During Export

1. **Load template** — Read styling from `.kleis` template
2. **Render equations** — Convert ASTs to Typst math
3. **Execute plot code** — Regenerate all figures
4. **Generate Typst** — Create `.typ` file with content
5. **Compile PDF** — Run `typst compile`

## Complete Example: PhD Thesis

Here's a complete example of creating a thesis:

```python
from kleis_kernel.kleisdoc import KleisDoc, Author

# Initialize
doc = KleisDoc()

# Metadata
doc.set_metadata(
    title="Quantum Entanglement in Topological Phases of Matter",
    authors=[Author(
        name="Jane Smith",
        affiliation="MIT Department of Physics",
        email="jsmith@mit.edu",
        orcid="0000-0001-2345-6789"
    )],
    date="January 2026",
    abstract="""
    This thesis investigates quantum entanglement properties in topological
    phases of matter. We develop new theoretical tools for characterizing
    entanglement in systems with topological order and apply these methods
    to study several model systems. Our main results include...
    """
)

# Set content blocks for thesis-specific fields
doc.set_content_block("degree", "Doctor of Philosophy")
doc.set_content_block("department", "Department of Physics")
doc.set_content_block("supervisor", "Prof. John Doe")

# Chapter 1: Introduction
ch1 = doc.add_section("Introduction", level=1, label="sec:intro")
doc.add_text("""
Quantum entanglement is one of the most profound features of quantum
mechanics. First recognized by Einstein, Podolsky, and Rosen in 1935,
entanglement has since become central to our understanding of quantum
information and condensed matter physics.
""")

# Subsection
doc.add_section("Motivation", level=2, parent=ch1)
doc.add_text("""
The study of topological phases has revealed new paradigms for
understanding quantum matter. In this thesis, we explore the
intersection of topology and entanglement.
""")

# Chapter 2: Background
ch2 = doc.add_section("Theoretical Background", level=1, label="sec:background")

# Add equation
doc.add_equation(
    label="entanglement_entropy",
    latex=r"S_A = -\text{Tr}(\rho_A \log \rho_A)"
)

doc.add_text("""
The entanglement entropy, defined in Equation @eq:entanglement_entropy,
quantifies the quantum correlations between subsystem A and its complement.
""")

# Chapter 3: Methods
ch3 = doc.add_section("Methods", level=1, label="sec:methods")

# Add a figure
doc.add_figure(
    label="fig:phase_diagram",
    caption="Phase diagram showing topological and trivial regions",
    kleis_code="""
let xs = linspace(0, 2, 50)
let ys = linspace(0, 2, 50)
// Phase boundary computation
diagram(
    contour(phase_data),
    title = "Phase Diagram",
    xlabel = "Parameter g",
    ylabel = "Parameter h"
)
"""
)

# Chapter 4: Results
ch4 = doc.add_section("Results", level=1, label="sec:results")

# Add results table
doc.add_table(
    label="tab:entanglement_scaling",
    caption="Entanglement entropy scaling coefficients",
    headers=["System", "Area Law", "Logarithmic", "Topological"],
    rows=[
        ["Trivial Insulator", "Yes", "No", "γ = 0"],
        ["Topological Insulator", "Yes", "No", "γ = ln(2)"],
        ["Critical Point", "Yes", "Yes", "—"]
    ]
)

# Chapter 5: Conclusion
ch5 = doc.add_section("Conclusion", level=1, label="sec:conclusion")
doc.add_text("""
In this thesis, we have developed new methods for characterizing
entanglement in topological phases. Our main contributions include...
""")

# Bibliography
doc.add_bib_entry(
    key="kitaev2006",
    entry_type="article",
    title="Topological Entanglement Entropy",
    author="Kitaev, Alexei and Preskill, John",
    journal="Physical Review Letters",
    year="2006",
    volume="96"
)

# Save document (for multi-session editing)
doc.save("quantum_thesis.kleis")

# Export to PDF
doc.export_pdf(
    "quantum_thesis.pdf",
    template_path="stdlib/templates/mit_thesis.kleis"
)

print("Thesis exported successfully!")
```

## Jupyter Notebook Workflow

### Recommended Workflow

**Cell 1: Setup and load**
```python
from kleis_kernel.kleisdoc import KleisDoc, Author

# Create new or load existing
doc = KleisDoc()
# OR: doc = KleisDoc.load("my_thesis.kleis")
```

**Cell 2: Set metadata**
```python
doc.set_metadata(
    title="My Thesis",
    authors=[Author(name="Jane Smith", affiliation="MIT")],
    date="2026",
    abstract="..."
)
```

**Cell 3-N: Add content**
```python
# Add sections, equations, figures as you work
doc.add_section("Introduction", level=1)
doc.add_text("...")
```

**Cell N+1: Preview (optional)**
```python
# Generate preview PDF
doc.export_pdf("preview.pdf", template_path="stdlib/templates/mit_thesis.kleis")

# Display in notebook (if using ipywidgets)
from IPython.display import IFrame
IFrame("preview.pdf", width=800, height=600)
```

**Cell N+2: Save progress**
```python
doc.save("my_thesis.kleis")
print("Saved!")
```

### Tips for Thesis Writing

1. **Save frequently** — Use `doc.save()` after significant changes
2. **Use labels** — Label all equations, figures, tables for cross-referencing
3. **Store ASTs** — For equations you'll edit, store the AST not just LaTeX
4. **Use Kleis code for plots** — They'll regenerate automatically
5. **Validate before export** — Run `doc.validate_cross_refs()`
6. **Version control** — `.kleis` files are text, perfect for git

## Troubleshooting

### "Kleis server not running"

Start the server:
```bash
kleis server --port 8080
```

### "Typst not found"

Install Typst:
```bash
brew install typst  # macOS
```

### "Template not found"

Ensure template path is relative to Kleis root or absolute:
```python
# Relative to KLEIS_ROOT
doc.export_pdf("out.pdf", template_path="stdlib/templates/mit_thesis.kleis")

# Absolute path
doc.export_pdf("out.pdf", template_path="/path/to/kleis/stdlib/templates/mit_thesis.kleis")
```

### Cross-reference errors

```python
errors = doc.validate_cross_refs()
for e in errors:
    print(e)
# Fix missing labels, then re-export
```

## Next Steps

- [Chapter 21: Jupyter Notebook](./21-jupyter-notebook.md) — Interactive Kleis
- [Chapter 11: Z3 Verification](./11-z3-verification.md) — Verify your math
- [Standard Library](./22-standard-library.md) — Available functions

