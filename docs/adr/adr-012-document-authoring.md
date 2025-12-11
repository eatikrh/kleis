# ADR-012: Kleis as Document Authoring Environment

## Status
**Proposed** - Extension of ADR-011 for complete document authoring  
**Updated:** December 6, 2024 - References ADR-015 for text representation

## Context

Kleis notebook (ADR-011) should be more than just cells with equations - it should be a **complete authoring environment** for mathematical documents that can export to:
- PDF (via Typst)
- LaTeX (for arXiv submissions)
- HTML (for web publishing)
- Jupyter notebooks (.ipynb)

**Related Decision:** [ADR-015: Text as Source of Truth](adr-015-text-as-source-of-truth.md) defines the canonical text representation for all equations, including inline equations in document text.

## Vision

> Write mathematical papers, lecture notes, and research documents entirely in Kleis, with structural equation editing and direct export to publication-quality formats.

---

## Cell Types Expanded

### 1. Code Cell (Expression)
**Purpose:** Executable mathematical expressions

```
┌─ Cell 1 [Expression] ──────────────────┐
│ E = ½mv²                                │
│                                        │
│ Out: E = 3.75 J                        │
│ Type: Scalar ✓                         │
└────────────────────────────────────────┘
```

### 2. Text Cell (Documentation)
**Purpose:** Rich text with inline math

```
┌─ Cell 2 [Text] ────────────────────────┐
│ ## Newton's Second Law                 │
│                                        │
│ The force F acting on an object is     │
│ equal to its mass m times acceleration │
│ a. Mathematically:                     │
│                                        │
│   F = ma                               │  ← Inline equation (structural!)
│                                        │
│ This fundamental law relates force,    │
│ mass, and acceleration.                │
└────────────────────────────────────────┘
```

### 3. Mixed Cell (Text + Equations)
**Purpose:** Inline equations within paragraphs

```
┌─ Cell 3 [Mixed] ───────────────────────┐
│ The kinetic energy E_k = ½mv² depends  │
│                     ↑                   │  ← Inline structural editor!
│ on both mass and velocity squared.     │
│                                        │
│ For momentum p = mv, we can write the  │
│                ↑                        │  ← Another inline editor
│ energy as E_k = p²/(2m).               │
│               ↑                         │  ← And another!
└────────────────────────────────────────┘
```

### 4. Context Cell (Definitions)
**Purpose:** Type and variable declarations

```
┌─ Cell 0 [Context] ─────────────────────┐
│ context physics {                      │
│     m: Scalar = 1.5  // kg             │
│     v: Vector(3) = [1, 2, 0]  // m/s   │
│ }                                      │
└────────────────────────────────────────┘
```

### 5. Section Header Cell
**Purpose:** Document organization

```
┌─ Cell [Section] ───────────────────────┐
│ # 2. Kinetic Energy                    │
│                                        │
│ ── or ──                               │
│                                        │
│ ## 2.1 Derivation                      │
└────────────────────────────────────────┘
```

### 6. Figure Cell
**Purpose:** Diagrams, plots, images

```
┌─ Cell [Figure] ────────────────────────┐
│ [Plot: E vs v]                         │
│                                        │
│  ^                                     │
│  │     ╱                               │
│ E│   ╱                                 │
│  │ ╱                                   │
│  └──────> v                            │
│                                        │
│ Caption: Kinetic energy as function    │
│ of velocity for m = 1.5 kg             │
└────────────────────────────────────────┘
```

---

## Text Editing Interface

### Rich Text Editor Options

**Option A: Markdown with Live Preview**
```
┌─ Edit Mode ────────────────────────────┐
│ ## Newton's Law                        │
│                                        │
│ Force equals mass times acceleration:  │
│ $$F = ma$$                             │  ← LaTeX math
│                                        │
│ Or using our notation: `inline:F=ma`   │  ← Inline Kleis
└────────────────────────────────────────┘
          ↓
┌─ Preview Mode ─────────────────────────┐
│ Newton's Law (rendered heading)        │
│                                        │
│ Force equals mass times acceleration:  │
│ F = ma (beautifully rendered)          │
│                                        │
│ Or using our notation: F = ma          │
└────────────────────────────────────────┘
```

**Option B: WYSIWYG with Embedded Structural Editor** ⭐ RECOMMENDED

```
┌─ Cell [Text] ──────────────────────────┐
│ ## Newton's Law                        │  ← Editable heading
│                                        │
│ The force [F = ma] acting on an       │
│              ↑                         │  ← Click to edit in structural mode
│ object equals mass times...            │
└────────────────────────────────────────┘

Click the equation → Structural editor opens:
┌─ Edit Equation ────────────────────────┐
│ F = m × a                              │
│ □   □   □  ← Inline editing (v2.2!)    │
│                                        │
│ [✓ Done] [✗ Cancel]                    │
└────────────────────────────────────────┘
```

**Benefits:**
- Uses v2.2 inline editing for embedded equations
- WYSIWYG (no mode switching)
- Natural flow (like Notion or Word)
- Visual consistency

---

## Inline Equation Syntax

### Embedding Equations in Text

**Syntax: Bracket notation** `[equation]`

```
The energy [E = ½mv²] depends on velocity.
             ↑
             Click to edit with structural editor
```

**Important:** Per [ADR-015](adr-015-text-as-source-of-truth.md), the text inside brackets follows canonical Kleis syntax:

```kleis
// In document text cell:
The absolute value [abs(x)] is always non-negative.
                    ^^^^^^
                    Canonical form (not |x|)

The cardinality [card(S)] gives the set size.
                 ^^^^^^^
                 Explicit function name

The fraction [frac(a, b)] represents division.
              ^^^^^^^^^^
              Display mode specified
```

**Benefits:**
- Clean bracket syntax for embedding
- Canonical Kleis text inside (git-friendly!)
- Visual editor generates canonical forms
- Click equation to edit with structural editor
- Renders beautifully when displayed

### Rendering and Storage

**Storage (per ADR-015):**
```kleis
// Stored as plain text in .kleis file
The energy [E = frac(1, 2) × m × v^2] is conserved.
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
           Canonical Kleis text syntax
```

**Parsing:**
```javascript
// Parse text with embedded equations
const text = "The energy [E = frac(1, 2) × m × v^2] is conserved.";

// Extract equations (parse canonical Kleis inside brackets)
const parts = parseTextWithEquations(text);
// → [
//     {type: 'text', content: 'The energy '},
//     {type: 'equation', 
//      text: 'E = frac(1, 2) × m × v^2',
//      ast: {...}, 
//      displayStyle: 'inline'},
//     {type: 'text', content: ' is conserved.'}
//   ]
```

**Rendering:**
```html
<!-- Visual display uses traditional notation -->
<p>
  The energy 
  <span class="inline-equation" onclick="editEquation(...)">
    E = ½mv²   <!-- Rendered beautifully, frac() shown as fraction -->
  </span>
  is conserved.
</p>
```

**Key Point:** Text file contains `frac(1, 2)`, visual display shows `½`. Per ADR-015, text is explicit and canonical, visual is beautiful.

---

## Export Formats

### Export to PDF (via Typst) ⭐ PRIMARY

**Advantages:**
- Kleis already uses Typst for rendering!
- Professional typography
- Fast compilation
- Modern output

**Workflow:**
```
Notebook → Convert to Typst → Compile to PDF

1. Each cell becomes Typst content
2. Context becomes Typst variables
3. Equations already in Typst format
4. Text cells become Typst markup
```

**Example conversion:**

```kleis
## Newton's Law

The force [F = ma] equals mass times acceleration.

E = ½mv²
```

↓ Converts to Typst:

```typst
= Newton's Law

The force $F = m a$ equals mass times acceleration.

$ E = 1/2 m v^2 $
```

↓ Compiles to:

**Beautiful PDF** 📄

### Export to LaTeX (for arXiv)

**Workflow:**
```
Notebook → Convert to LaTeX → Submit to arXiv

1. Each equation exports to LaTeX (already supported!)
2. Text cells become LaTeX markup
3. Context becomes \newcommand definitions
4. Package as .tex + .bbl
```

**Example conversion:**

```kleis
## Newton's Law

E = ½mv²
```

↓ Converts to LaTeX:

```latex
\section{Newton's Law}

\begin{equation}
E = \frac{1}{2} m v^2
\end{equation}
```

### Export to HTML (for Web)

**Workflow:**
```
Notebook → Convert to HTML → Publish

1. Static HTML page
2. MathJax for equations
3. Interactive (can toggle to show AST)
4. Self-contained
```

### Export to Jupyter (.ipynb)

**Workflow:**
```
Kleis Notebook → .ipynb

1. Code cells → Python cells with SymPy
2. Equations → SymPy expressions
3. Context → Variable definitions
4. Compatible with Google Colab
```

---

## Document Templates

### Template 1: Research Paper

```
┌────────────────────────────────────────┐
│ # Title: [Your Paper Title]            │
│ Authors: [Names]                       │
│ Date: [2024-12-03]                     │
├────────────────────────────────────────┤
│ ## Abstract                            │
│ [Text cell]                            │
├────────────────────────────────────────┤
│ ## 1. Introduction                     │
│ [Text with inline equations]           │
├────────────────────────────────────────┤
│ ## 2. Theory                           │
│ [Context: physics]                     │
│ [Equation cell: E = mc²]               │
│ [Text: Derivation...]                  │
├────────────────────────────────────────┤
│ ## 3. Results                          │
│ [Equation cell with numerical output]  │
├────────────────────────────────────────┤
│ ## References                          │
│ [Bibliography]                         │
└────────────────────────────────────────┘

[📄 Export to PDF] [📤 Export to LaTeX]
```

### Template 2: Lecture Notes

```
┌────────────────────────────────────────┐
│ # Lecture 5: Energy Conservation       │
├────────────────────────────────────────┤
│ Context: physics                       │
├────────────────────────────────────────┤
│ ## Key Concepts                        │
│ - Kinetic energy: [E_k = ½mv²]        │
│ - Potential energy: [E_p = mgh]        │
│ - Total energy: [E = E_k + E_p]        │
├────────────────────────────────────────┤
│ ## Example Problem                     │
│ Given m = 2kg, v = 3m/s:               │
│ [E_k = ½ × 2 × 3²]                     │
│ Result: 9 J ✓                          │
├────────────────────────────────────────┤
│ ## Homework                            │
│ 1. Calculate energy for...            │
└────────────────────────────────────────┘
```

### Template 3: Problem Set

```
┌────────────────────────────────────────┐
│ # Problem Set 3: Classical Mechanics   │
├────────────────────────────────────────┤
│ Problem 1: [5 points]                  │
│                                        │
│ A mass m = 1.5kg moves with velocity   │
│ v = [3, 4, 0] m/s. Calculate:          │
│                                        │
│ a) Momentum: [p = mv]                  │
│    Solution: [Equation with result]    │
│                                        │
│ b) Kinetic energy: [E = ½mv²]          │
│    Solution: [Equation with result]    │
├────────────────────────────────────────┤
│ Problem 2: [10 points]                 │
│ ...                                    │
└────────────────────────────────────────┘

[📄 Export PDF (with/without solutions)]
```

---

## Rich Text Features

### Formatting Options

```
┌─ Text Toolbar ─────────────────────────┐
│ [B] [I] [U] [H1] [H2] [•] [1.] [→]    │
│ [Link] [Image] [Table] [Equation]      │
└────────────────────────────────────────┘

Text cell:
┌────────────────────────────────────────┐
│ **Bold**, *italic*, and __underline__ │
│                                        │
│ - Bullet lists                         │
│ 1. Numbered lists                      │
│                                        │
│ > Block quotes                         │
│                                        │
│ `Code snippets`                        │
│                                        │
│ Hyperlinks: `[text](url)`              │
│                                        │
│ Images: `![alt](url)`                  │
│                                        │
│ Tables: | Col1 | Col2 |                │
└────────────────────────────────────────┘
```

### Inline Equations

**Click "Equation" button** → Inserts equation placeholder:

```
Text: "The energy [] depends on velocity"
              ↑
              Click to edit with structural editor (v2.2!)
```

**Result after editing:**

```
Text: "The energy [E = ½mv²] depends on velocity"
                   ↑
                   Rendered equation (clickable to re-edit)
```

---

## Export System

### Export to PDF via Typst ⭐ PRIMARY

**Why Typst?**
- ✅ Kleis already uses Typst for rendering
- ✅ Modern, fast compiler
- ✅ Beautiful typography
- ✅ Native math support
- ✅ One rendering engine for everything

**Conversion Pipeline:**

```
Kleis Notebook
    ↓
Typst Document
    ↓
PDF Output
```

**Example conversion:**

```kleis
# My Paper

## Introduction

The energy [E = mc²] is fundamental.

context physics {
    c: Scalar = 299792458
}

E = mc²
```

↓ Converts to Typst:

```typst
#set document(title: "My Paper")
#set page(paper: "a4")
#set text(font: "Latin Modern Math")

= My Paper

== Introduction

The energy $E = m c^2$ is fundamental.

$ E = m c^2 $
```

↓ Compiles to PDF:

**Professional academic paper** 📄

### Export to LaTeX (for arXiv)

**Template structure:**

```latex
\documentclass{article}
\usepackage{amsmath, amssymb}

\title{My Paper}
\author{Author Name}
\date{\today}

\begin{document}
\maketitle

\section{Introduction}

The energy $E = mc^2$ is fundamental.

\begin{equation}
E = mc^2
\end{equation}

\end{document}
```

**Includes:**
- Proper LaTeX preamble
- Bibliography support (.bib file)
- arXiv-compliant formatting
- All equations as LaTeX (already have this!)

### Export to HTML

**Self-contained HTML document:**

```html
<!DOCTYPE html>
<html>
<head>
    <title>My Paper</title>
    <script src="https://cdn.jsdelivr.net/npm/mathjax@3/..."></script>
    <style>/* Professional article styling */</style>
</head>
<body>
    <article>
        <h1>My Paper</h1>
        
        <section>
            <h2>Introduction</h2>
            <p>The energy \(E = mc^2\) is fundamental.</p>
            
            <div class="equation">
                \[E = mc^2\]
            </div>
        </section>
    </article>
</body>
</html>
```

**Features:**
- MathJax for equations
- Responsive design
- Print-friendly CSS
- Self-contained (embeds images as data URLs)

---

## Export UI

### Export Menu

```
File → Export →
    📄 Export to PDF (Typst)
    📄 Export to PDF (LaTeX)
    📋 Export to LaTeX (.tex)
    🌐 Export to HTML
    📓 Export to Jupyter (.ipynb)
    📦 Export Package (with contexts)
```

### Export Dialog

```
┌─ Export to PDF ────────────────────────┐
│                                        │
│ Template: [Research Paper  ▼]         │
│           - Research Paper             │
│           - Lecture Notes              │
│           - Problem Set                │
│           - Technical Report           │
│                                        │
│ Options:                               │
│  ☑ Include context definitions         │
│  ☑ Show equation numbers               │
│  ☑ Include AST debug info              │
│  ☐ Solutions only (hide problems)      │
│                                        │
│ Paper size: [A4 ▼]                     │
│ Font size: [11pt ▼]                    │
│                                        │
│ Output: [classical_mechanics.pdf____]  │
│                                        │
│ [Preview] [Export]                     │
└────────────────────────────────────────┘
```

---

## Text Cell Editor

### Design: Hybrid WYSIWYG + Structural

**Text editing:**
- Rich text editor (like Notion, Medium)
- Markdown shortcuts (## for heading, ** for bold)
- Toolbar for formatting

**Equation insertion:**
```
1. Type text: "The energy"
2. Click [+Equation] button or type [ ]
3. Structural editor appears inline
4. Build equation with v2.2 inline editing
5. Click outside or press ESC
6. Equation renders inline in text
```

**Example:**

```
Type: "The force"
Click: [+Equation]
Edit: F = ma (using structural editor)
Result: "The force [F = ma] equals..."
         ↑
         Click to re-edit
```

---

## Document Structure

### Hierarchical Organization

```
Document
├── Frontmatter
│   ├── Title
│   ├── Authors
│   ├── Abstract
│   └── Keywords
│
├── Section 1: Introduction
│   ├── Text cell
│   ├── Equation cell
│   └── Text cell
│
├── Section 2: Theory
│   ├── Context cell (definitions)
│   ├── Subsection 2.1: Derivation
│   │   ├── Text cell
│   │   ├── Equation cell
│   │   └── Equation cell
│   └── Subsection 2.2: Results
│       ├── Text cell
│       └── Figure cell
│
└── References
    └── Bibliography cell
```

### Navigation Panel

```
┌─ OUTLINE ──────────────────┐
│ ▼ 1. Introduction          │  ← Click to jump
│ ▼ 2. Theory                │
│   ▸ 2.1 Derivation         │
│   ▸ 2.2 Results            │
│ ▼ 3. Conclusion            │
│ ▼ References               │
│                            │
│ Cells: 15                  │
│ Equations: 8               │
│ Words: 2,450               │
└────────────────────────────┘
```

---

## PDF Export Implementation

### Using Typst (Recommended)

**Backend endpoint:**

```rust
POST /api/export/pdf
Body: {
    "notebook": {...},
    "template": "research_paper",
    "options": {
        "include_context": true,
        "equation_numbers": true,
        "paper_size": "a4",
        "font_size": "11pt"
    }
}

Response: PDF binary data
```

**Conversion steps:**

1. **Parse notebook** → Extract cells, contexts
2. **Convert to Typst:**
   - Context → Typst variables
   - Text cells → Typst markup
   - Equation cells → Typst math ($...$)
   - Inline equations → Typst inline math
3. **Apply template** → Research paper layout
4. **Compile** → PDF with Typst library
5. **Return** → Binary PDF data

**Server code:**

```rust
async fn export_to_pdf(notebook: Notebook, options: ExportOptions) -> Result<Vec<u8>, Error> {
    // 1. Convert notebook to Typst
    let typst_source = notebook_to_typst(&notebook, &options)?;
    
    // 2. Compile with Typst
    let pdf_bytes = compile_typst_to_pdf(&typst_source)?;
    
    Ok(pdf_bytes)
}

fn notebook_to_typst(notebook: &Notebook, options: &ExportOptions) -> String {
    let mut typst = String::from("#set document(title: \"");
    typst.push_str(&notebook.title);
    typst.push_str("\")\n");
    typst.push_str("#set page(paper: \"a4\", margin: 2.5cm)\n");
    typst.push_str("#set text(font: \"Latin Modern Math\", size: 11pt)\n\n");
    
    // Convert each cell
    for cell in &notebook.cells {
        typst.push_str(&convert_cell_to_typst(cell, options));
        typst.push('\n');
    }
    
    typst
}
```

### LaTeX Export

**For arXiv submissions:**

```rust
POST /api/export/latex
Body: { "notebook": {...} }

Response: {
    "main.tex": "\\documentclass{article}...",
    "figures/": [...],
    "references.bib": "...",
    "arxiv_ready": true
}
```

**Downloads as .zip:**
```
paper.zip
├── main.tex
├── figures/
│   ├── fig1.pdf
│   └── fig2.pdf
├── references.bib
└── README.txt
```

---

## Cell Toolbar

### Per-Cell Actions

```
┌─ Cell 3 ───────────────────────────────────────────┐
│ [▶ Run] [⬆] [⬇] [🗑] [+Below] [⋮ More]            │  ← Toolbar
├─────────────────────────────────────────────────────┤
│ E = ½mv²                                           │
│                                                    │
│ Out: 3.75 J                                        │
└─────────────────────────────────────────────────────┘
```

**Actions:**
- **▶ Run** - Execute cell
- **⬆⬇** - Move cell up/down
- **🗑** - Delete cell
- **+Below** - Insert cell below
- **⋮ More** - Convert type, copy, hide output, etc.

---

## Typst Template System

### Research Paper Template

```typst
// kleis-templates/research_paper.typ

#let paper(title, authors, abstract, body) = {
  set document(title: title)
  set page(
    paper: "a4",
    margin: (x: 2.5cm, y: 2.5cm),
    numbering: "1",
  )
  set text(font: "Latin Modern Math", size: 11pt)
  set par(justify: true)
  
  // Title
  align(center)[
    #text(17pt, weight: "bold")[#title]
    #v(1em)
    #text(12pt)[#authors.join(", ")]
    #v(2em)
  ]
  
  // Abstract
  if abstract != none [
    #heading(level: 2)[Abstract]
    #abstract
    #v(1em)
  ]
  
  // Body
  body
}
```

**Usage in Kleis:**

```rust
let typst_doc = format!(r#"
#import "kleis-templates/research_paper.typ": paper
#show: paper.with(
    title: "{}",
    authors: ({}),
    abstract: [{}],
)

{}
"#, 
    notebook.title,
    notebook.authors.join(", "),
    notebook.abstract,
    converted_body
);
```

---

## Implementation Plan

### Phase 1: Text Cells (Week 1-2)

**Add to notebook:**
- [ ] Text cell type
- [ ] Rich text editor (TinyMCE or Quill)
- [ ] Markdown support
- [ ] Inline equation syntax `[equation]`
- [ ] Click equation to edit with structural editor

**Deliverable:** Can create documents with text + equations

### Phase 2: Export to PDF (Week 3-4)

**Add backend:**
- [ ] `/api/export/pdf` endpoint
- [ ] Notebook → Typst converter
- [ ] Typst compilation
- [ ] PDF download

**Deliverable:** Can export notebook to professional PDF

### Phase 3: Templates & LaTeX (Week 5-6)

**Add features:**
- [ ] Document templates (research, lecture, problem set)
- [ ] LaTeX export
- [ ] arXiv-ready packaging
- [ ] Bibliography support

**Deliverable:** Can submit to arXiv directly from Kleis

### Phase 4: Polish (Week 7-8)

**Add features:**
- [ ] HTML export
- [ ] Jupyter export
- [ ] Outline/navigation panel
- [ ] Document metadata editor
- [ ] Print preview

**Deliverable:** Complete document authoring system

---

## File Format: Extended .kleis

**Per ADR-015, stored as canonical Kleis text:**

```kleis
---
metadata:
  title: "Classical Mechanics"
  authors: ["John Doe", "Jane Smith"]
  date: 2024-12-03
  keywords: ["physics", "mechanics", "energy"]
  export_template: "research_paper"
---

# Abstract

This paper presents...

---

## 1. Introduction

Classical mechanics describes motion. The fundamental equation [F = m × a]
relates force, mass, and acceleration.

The absolute value [abs(x - x₀)] represents distance.
                   ^^^^^^^^^^^
                   Canonical form (ADR-015)

---

context physics {
    m: Scalar = 1.5  // kg
    v: Vector(3) = [1, 2, 0]  // m/s
}

---

## 2. Energy

The kinetic energy is defined as:

E_k = frac(1, 2) × m × v^2

Using our values:

>>> E_k = frac(1, 2) × 1.5 × (1^2 + 2^2 + 0^2)
Result: 3.75 J

---

## References

[1] Newton, I. (1687). Principia Mathematica.
```

**Syntax:**
- `---` separates cells
- `##` creates section headers
- `[equation]` for inline math (uses canonical Kleis syntax inside)
- `>>> code` for executable equations
- `context { }` for definitions

**Text Representation (ADR-015):**
- Inline equations use explicit forms: `abs(x)`, `card(S)`, `norm(v)`
- Display mode specified: `frac(a, b)` for fractions
- Unicode symbols allowed: `×`, `Σ`, `∫`, etc.
- Git diffs show actual equation changes clearly

---

## UI Mockup: Complete Notebook

```
┌────────────────────────────────────────────────────────────────┐
│ 📘 Classical Mechanics                    [💾] [▶ Run All]     │
├────────────────────────────────────────────────────────────────┤
│ ┌─ CONTEXT ──┬─ NOTEBOOK ─────────────────┬─ OUTLINE ──────┐  │
│ │ 📦 physics │                            │ ▼ Abstract      │  │
│ │            │ ┌─ Cell 1 [Text] ──────┐  │ ▼ 1. Intro      │  │
│ │ m: 1.5kg   │ │ # Abstract            │  │ ▼ 2. Theory    │  │
│ │ v: [1,2,0] │ │                       │  │   ▸ 2.1 Energy  │  │
│ │ F: Vector  │ │ This paper...         │  │   ▸ 2.2 Force   │  │
│ │            │ └───────────────────────┘  │ ▼ 3. Results   │  │
│ │ [+ Add]    │                            │ ▼ References   │  │
│ │            │ ┌─ Cell 2 [Expression] ─┐  │                │  │
│ │ 📤 Export: │ │ E = ½mv²              │  │ 15 cells       │  │
│ │  PDF       │ │                       │  │ 8 equations    │  │
│ │  LaTeX     │ │ Out: 3.75 J           │  │ 2,450 words    │  │
│ │  HTML      │ │ Type: Scalar ✓        │  │                │  │
│ │            │ └───────────────────────┘  │                │  │
│ └────────────┴────────────────────────────┴────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

---

## Example: Complete Paper Workflow

### 1. Create Document

```
File → New → Research Paper Template
```

### 2. Edit Title and Abstract

```
Cell 1 [Text]:
# Energy Conservation in Classical Mechanics

Cell 2 [Text]:
## Abstract

We derive the principle of energy conservation from Newton's laws.
The total energy [E = E_k + E_p] remains constant in a closed system.
```

### 3. Add Definitions

```
Cell 3 [Context]:
context physics {
    m: Scalar = 2.0  // kg
    h: Scalar = 10.0  // m
    g: Scalar = 9.81  // m/s²
}
```

### 4. Write Theory Section

```
Cell 4 [Text]:
## 1. Theory

Kinetic energy depends on velocity:

Cell 5 [Expression]:
E_k = ½mv²

Cell 6 [Text]:
Potential energy depends on height:

Cell 7 [Expression]:
E_p = mgh

Cell 8 [Text]:
Total mechanical energy:

Cell 9 [Expression]:
E = E_k + E_p
Out: E = 196.2 J
```

### 5. Export

```
File → Export → PDF (Typst)
Choose: Research Paper template
Click: Export

Result: classical_mechanics.pdf
- Professional formatting
- All equations beautifully rendered
- Ready for submission!
```

---

## Advantages Over Alternatives

### vs Jupyter + LaTeX

**Jupyter:**
- ❌ Text editing is basic markdown
- ❌ Math is code (SymPy strings)
- ❌ No structural editing
- ❌ Export to LaTeX is clunky

**Kleis:**
- ✅ Rich text with inline structural editing
- ✅ Math is visual AST
- ✅ v2.2 inline editing for all equations
- ✅ Direct Typst → PDF

### vs Mathematica Notebooks

**Mathematica:**
- ❌ Proprietary format
- ❌ Expensive license
- ❌ Not Git-friendly
- ✅ Excellent typography

**Kleis:**
- ✅ Open source
- ✅ Free
- ✅ Plain text .kleis files (Git-friendly)
- ✅ Excellent typography (Typst)

### vs Overleaf (LaTeX)

**Overleaf:**
- ❌ Text-only editing
- ❌ Must know LaTeX syntax
- ❌ No structural editor
- ✅ arXiv submission ready

**Kleis:**
- ✅ Structural + inline editing (v2.2!)
- ✅ No LaTeX knowledge needed
- ✅ Visual equation building
- ✅ Exports LaTeX for arXiv

---

## Technical Stack

### Frontend:
- **Notebook UI:** React or Vue (cell management)
- **Text editor:** TinyMCE or Quill (rich text)
- **Math editor:** Existing v2.2 structural editor ✅
- **Rendering:** Typst (backend) + MathJax (preview)

### Backend:
- **Server:** Existing Rust server (extend endpoints)
- **Parser:** Existing Kleis parser
- **Type system:** To be implemented (ADR-011)
- **Typst compiler:** Already integrated! ✅

### Storage:
- **.kleis:** Plain text (Git)
- **.kleis-nb:** JSON (local)
- **PDF:** Generated on demand
- **LaTeX:** Generated on demand

---

## Example Output: PDF from Notebook

### Source (.kleis file):

**Stored as canonical Kleis text (ADR-015):**

```kleis
# Classical Mechanics

## Introduction

Newton's second law states [F = m × a].

The magnitude [abs(F)] gives the force strength.
               ^^^^^^
               Explicit form for git diffs

context physics {
    m: Scalar = 1.5
}

E = frac(1, 2) × m × v^2

Result: 3.75 J
```

### Generated PDF:

**Visual rendering uses traditional notation:**

```
┌──────────────────────────────────────┐
│   Classical Mechanics                 │
│                                       │
│ 1. Introduction                       │
│                                       │
│ Newton's second law states F = ma.   │
│                                       │
│ The magnitude |F| gives the force    │
│ strength.                             │
│      ↑                                │
│      abs(F) rendered as |F|           │
│                                       │
│ Given m = 1.5 kg:                    │
│                                       │
│          1                            │
│      E = ─ mv²            (1)         │
│          2                            │
│      ↑                                │
│      frac(1,2) rendered as fraction   │
│                                       │
│ Result: E = 3.75 J                    │
│                                       │
│                                     1 │
└──────────────────────────────────────┘
```

**Key:** Text file has `frac(1,2)` and `abs(F)` (canonical), PDF renders as ½mv² and |F| (beautiful)!

---

## Implementation Estimate

### Core Features:
- Text cells: 2 weeks
- Inline equations in text: 1 week
- PDF export (Typst): 2 weeks
- LaTeX export: 1 week
- Templates: 1 week

### Total: 7-8 weeks

**Builds on:**
- ✅ v2.2 inline editing (already done!)
- ✅ Typst integration (already done!)
- ✅ Rendering pipeline (already done!)

**Only need to add:**
- Text cell editor
- Notebook shell
- Export converters

---

## Decision

**Recommendation:** Implement document authoring as natural extension of ADR-011 notebook environment

**Benefits:**
1. **Complete authoring tool** - Write papers entirely in Kleis
2. **Reuses v2.2 editor** - Inline editing for all equations
3. **Professional output** - Typst generates beautiful PDFs
4. **arXiv-ready** - LaTeX export included
5. **Git-friendly** - Plain text .kleis files (per ADR-015)
6. **Canonical text** - Explicit forms for clear version control

**Key Design Principles (from ADR-015):**
- Text is source of truth (files store canonical Kleis syntax)
- Visual display uses traditional notation (beautiful rendering)
- Inline equations in `[brackets]` use canonical forms
- Visual editor generates explicit text: `abs(x)`, `frac(a,b)`, etc.
- Git diffs show actual equation changes clearly

**Timeline:** Q1 2025 (alongside notebook implementation)

---

**Status:** ✅ **Fully Specified - Ready for Implementation**

**Related ADRs:**
- [ADR-011](adr-011-notebook-environment.md) - Notebook Environment
- [ADR-015](adr-015-text-as-source-of-truth.md) - Text Representation (critical!)

Next: Create UI mockups and begin prototyping text cells with embedded structural editor. Ensure visual editor generates canonical text per ADR-015.

