# Jupyter → .kleisdoc Integration Design

## Overview

This document describes how Jupyter Notebook cells are exported to the `.kleisdoc` 
persistent format, which is then compiled to PDF via the thesis compiler.

## Cell Tagging System

In Jupyter, cells are tagged with metadata that maps them to document structure:

```python
# Cell metadata (set via Jupyter's tag interface or magic comments)
{
  "kleisdoc": {
    "id": "ch1-fig1",           # Stable ID for cross-references
    "type": "figure",           # figure | equation | text | code | table
    "section": "chapter-1",     # Which section this belongs to
    "caption": "Plot of x²",    # For figures/tables
    "label": "fig:quadratic"    # For cross-references
  }
}
```

## Magic Comments (Alternative to Metadata)

For quick tagging without editing cell metadata:

```python
# %kleisdoc: id=ch1-fig1, type=figure, section=chapter-1, caption="Plot of x²"

xs = range(0, 10)
ys = list_map(xs, fn(x) x*x)
diagram(plot(xs, ys), title = "Quadratic Function")
```

## Export Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│ Jupyter Notebook                                                │
│                                                                 │
│  Cell 1: %kleisdoc: type=title                                  │
│          "My PhD Thesis"                                        │
│                                                                 │
│  Cell 2: %kleisdoc: id=ch1-p1, type=text, section=chapter-1     │
│          This chapter introduces...                             │
│                                                                 │
│  Cell 3: %kleisdoc: id=ch1-fig1, type=figure, section=chapter-1 │
│          xs = range(0,10); plot(xs, ys) → [SVG output]          │
│                                                                 │
│  Cell 4: %kleisdoc: id=ch1-eq1, type=equation, section=chapter-1│
│          E = mc² → [Typst equation]                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ export_kleisdoc()
┌─────────────────────────────────────────────────────────────────┐
│ Python Export Function                                          │
│                                                                 │
│  1. Parse all cells with kleisdoc tags                          │
│  2. For each tagged cell:                                       │
│     - If type=figure: Save SVG, store Kleis code as source      │
│     - If type=equation: Convert to Typst, store as-is           │
│     - If type=text: Store as plain text                         │
│     - If type=code: Store with language tag                     │
│  3. Group by section                                            │
│  4. Generate .kleisdoc YAML file                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ thesis.kleisdoc (YAML)                                          │
│                                                                 │
│  id: thesis-2025-01                                             │
│  title: "My PhD Thesis"                                         │
│  author: "Jane Smith"                                           │
│  degree: phd                                                    │
│  created: "2025-01-02T00:00:00Z"                               │
│  modified: "2025-01-02T20:00:00Z"                              │
│  version: 3                                                     │
│                                                                 │
│  sections:                                                      │
│    - type: chapter                                              │
│      number: 1                                                  │
│      title: "Introduction"                                      │
│      chunks:                                                    │
│        - id: ch1-p1                                             │
│          type: text                                             │
│          content: "This chapter introduces..."                  │
│                                                                 │
│        - id: ch1-fig1                                           │
│          type: figure                                           │
│          source:                                                │
│            type: regenerable                                    │
│            code: "xs = range(0,10); plot(xs, ys)"              │
│          typst: "image(\"ch1-fig1.svg\")"                      │
│          caption: "Plot of x²"                                  │
│          label: "fig:quadratic"                                 │
│                                                                 │
│        - id: ch1-eq1                                            │
│          type: equation                                         │
│          ast:  # EditorNode AST for re-editing                  │
│            Operation:                                           │
│              name: "equals"                                     │
│              args:                                              │
│                - Symbol: "E"                                    │
│                - Operation:                                     │
│                    name: "times"                                │
│                    args: [Symbol: "m", ...]                     │
│          typst: "$ E = m c^2 $"                                │
│          label: "eq:einstein"                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Regenerable vs Static Content

### Regenerable (has `source.code`)
- Plots created from Kleis code
- Computed tables
- Any output that can be recreated by running code

```yaml
- id: ch1-fig1
  type: figure
  source:
    type: regenerable
    code: |
      xs = range(0, 10)
      ys = list_map(xs, fn(x) x*x)
      diagram(plot(xs, ys), title = "Quadratic")
  typst: "image(\"ch1-fig1.svg\")"
  caption: "Quadratic function"
```

### Static (no `source.code`)
- Imported images
- Hand-written text
- Manually entered equations

```yaml
- id: ch2-img1
  type: figure
  source:
    type: static
    path: "images/imported-diagram.png"
  typst: "image(\"images/imported-diagram.png\")"
  caption: "Imported diagram"
```

## Multi-Session Editing

The `.kleisdoc` format supports editing across sessions:

1. **Open existing document:**
   ```python
   doc = load_kleisdoc("thesis.kleisdoc")
   ```

2. **Find and edit a chunk:**
   ```python
   chunk = doc.get_chunk("ch1-fig1")
   # Chunk knows its Kleis source code
   # User can edit in Jupyter, re-run, update chunk
   ```

3. **Regenerate outdated chunks:**
   ```python
   # Find all regenerable chunks
   for chunk in doc.regenerable_chunks():
       if chunk.needs_update():
           new_output = run_kleis(chunk.source_code)
           chunk.update(new_output)
   ```

4. **Save updated document:**
   ```python
   doc.save("thesis.kleisdoc")  # Increments version
   ```

## Compilation Pipeline

```python
# In Jupyter:
doc = load_kleisdoc("thesis.kleisdoc")

# Option 1: Compile to Typst string
typst_code = compile_kleisdoc(doc)

# Option 2: Compile to PDF and display
pdf_bytes = compile_to_pdf(doc)
display(PDF(pdf_bytes))

# Option 3: Export Typst files for manual editing
export_typst_project(doc, "output/")
# Creates:
#   output/thesis.typ
#   output/figures/ch1-fig1.svg
#   output/figures/ch2-fig1.svg
#   ...
```

## Python Implementation Sketch

```python
# kleis_notebook/document.py

from dataclasses import dataclass
from typing import List, Optional, Dict, Any
import yaml
from datetime import datetime

@dataclass
class ContentSource:
    type: str  # "regenerable" | "static" | "computed"
    code: Optional[str] = None
    path: Optional[str] = None
    result: Optional[str] = None

@dataclass
class DocChunk:
    id: str
    chunk_type: str  # "text" | "equation" | "figure" | "table" | "code"
    source: ContentSource
    typst_code: str
    caption: str = ""
    label: str = ""
    ast: Optional[Dict[str, Any]] = None  # For equations

@dataclass
class DocSection:
    section_type: str  # "title" | "abstract" | "chapter" | "appendix" | "references"
    title: str
    number: Optional[int] = None
    letter: Optional[str] = None
    chunks: List[DocChunk] = None

@dataclass
class KleisDoc:
    id: str
    title: str
    author: str
    degree: str
    department: str
    date: str
    sections: List[DocSection]
    created: str
    modified: str
    version: int

    def save(self, path: str):
        self.modified = datetime.now().isoformat()
        self.version += 1
        with open(path, 'w') as f:
            yaml.dump(self.to_dict(), f)

    @classmethod
    def load(cls, path: str) -> 'KleisDoc':
        with open(path, 'r') as f:
            data = yaml.safe_load(f)
        return cls.from_dict(data)

    def get_chunk(self, chunk_id: str) -> Optional[DocChunk]:
        for section in self.sections:
            if section.chunks:
                for chunk in section.chunks:
                    if chunk.id == chunk_id:
                        return chunk
        return None

    def regenerable_chunks(self) -> List[DocChunk]:
        result = []
        for section in self.sections:
            if section.chunks:
                for chunk in section.chunks:
                    if chunk.source.type == "regenerable":
                        result.append(chunk)
        return result
```

## Magic Commands for Jupyter

```python
# Register Kleis magics
from IPython.core.magic import register_cell_magic

@register_cell_magic
def kleisdoc(line, cell):
    """Tag a cell for document export.
    
    Usage:
        %%kleisdoc id=ch1-fig1, type=figure, section=chapter-1, caption="My plot"
        
        xs = range(0, 10)
        ys = list_map(xs, fn(x) x*x)
        diagram(plot(xs, ys))
    """
    # Parse metadata from line
    # Execute cell
    # Store output with metadata for later export
    ...
```

## Next Steps

1. **Implement Python classes** for KleisDoc, DocChunk, DocSection
2. **Create export_kleisdoc()** function to extract tagged cells
3. **Create load_kleisdoc()** function to read YAML format
4. **Create compile_to_pdf()** using existing Typst integration
5. **Add Jupyter magics** for cell tagging
6. **Test with real notebook** creating a mini-thesis

