# KleisDoc Format Design

**Date:** January 2, 2026  
**Status:** Design Phase  
**Goal:** Define a comprehensive document format that captures everything needed for publishable documents

---

## Design Philosophy

> "A document is a program. Running it validates, compiles, and produces output."

The `.kleisdoc` format must:

1. **Capture complete document state** - Everything needed to regenerate the document
2. **Enable multi-session editing** - Pick up where you left off
3. **Support validation** - Apply axioms to verify document correctness
4. **Be human-readable** - YAML/JSON for easy inspection and diffing
5. **Track provenance** - Know how each piece was created
6. **Support collaboration** - Merge changes, resolve conflicts

---

## Comprehensive Requirements Checklist

### 1. Document Identity & Metadata

| Requirement | Status | Notes |
|-------------|--------|-------|
| Unique document ID | ✅ | UUID for database/linking |
| Title | ✅ | Main title |
| Authors | ⚠️ | Need multiple authors with affiliations |
| Version | ✅ | Semantic or monotonic |
| Created/Modified timestamps | ✅ | ISO 8601 |
| Template reference | ⚠️ | Which template (mit_thesis, arxiv) |
| Keywords | ❌ | For indexing/search |
| Abstract (structured) | ⚠️ | Currently in section, should be metadata |
| License | ❌ | CC-BY, MIT, etc. |
| DOI/Citation info | ❌ | For published documents |
| Language | ❌ | en-US, de-DE, etc. |
| Git commit hash | ❌ | Link to code version |

### 2. Author Information

```yaml
authors:
  - id: "author-001"
    name: "Jane Smith"
    email: "jane@mit.edu"
    orcid: "0000-0002-1234-5678"
    affiliation: "MIT CSAIL"
    role: "primary"  # primary, advisor, contributor
    
  - id: "author-002"
    name: "Prof. Alice Chen"
    affiliation: "MIT CSAIL"
    role: "advisor"
```

### 3. Content Chunks

| Chunk Type | Current | Needed Fields |
|------------|---------|---------------|
| Text | ✅ | markdown source, rendered typst |
| Equation | ✅ | latex, kleis_ast (EditorNode), label, numbered |
| Figure | ✅ | source_type, kleis_code, svg_cache, caption, label, alt_text |
| Table | ✅ | headers, rows, kleis_code (if computed), caption |
| Code | ✅ | language, source, executable (bool), output_cache |
| Algorithm | ❌ | pseudocode, line_numbers, caption |
| Theorem | ❌ | type (theorem/lemma/proof), statement, proof (optional) |
| Definition | ❌ | term, definition |
| Example | ❌ | description, code, expected_output |
| Quote | ❌ | text, attribution, citation_key |
| List | ❌ | items, ordered (bool), level |
| Callout | ❌ | type (note/warning/tip), content |

### 4. Equation-Specific Requirements

Equations are special because they're edited visually AND need to be verified:

```yaml
equation:
  id: "eq-einstein"
  label: "eq:einstein"
  numbered: true
  
  # Multiple representations
  latex: "E = mc^2"
  kleis_text: "E = m * c^2"  # For verification
  editor_ast:                 # EditorNode for re-editing
    type: "equation"
    children:
      - type: "symbol", value: "E"
      - type: "operator", value: "="
      - type: "product"
        children: [...]
        
  # Typst output
  typst_code: "$ E = m c^2 $ <eq:einstein>"
  
  # Validation status
  verified: true
  verification_axiom: "mass_energy_equivalence"
  verification_timestamp: "2025-01-02T10:00:00Z"
```

### 5. Figure-Specific Requirements

```yaml
figure:
  id: "ch1-fig1"
  label: "fig:quadratic"
  caption: "Quadratic function showing $x^2$"
  alt_text: "A parabola opening upward"  # Accessibility
  
  source:
    type: "regenerable"  # or "static"
    kleis_code: |
      xs = range(0, 10)
      ys = list_map(xs, fn(x) x*x)
      diagram(
        plot(xs, ys),
        title = "Quadratic",
        xlabel = "x"
      )
    dependencies: ["stdlib/plotting.kleis"]
    
  # Cached outputs
  typst_fragment: |
    #lq.diagram(
      title: [Quadratic],
      lq.plot((0, 1, 2, ...), (0, 1, 4, ...))
    )
  svg_cache: "figures/ch1-fig1.svg"  # Relative path
  svg_hash: "sha256:abc123..."       # For cache invalidation
  
  # Layout hints
  width: "70%"
  placement: "here"  # here, top, bottom, page
  subfigure_of: null  # For grouped figures
```

### 6. Table-Specific Requirements

```yaml
table:
  id: "ch2-table1"
  label: "tab:results"
  caption: "Experimental results"
  
  source:
    type: "regenerable"  # or "static"
    kleis_code: |
      // Code that generates table data
      compute_results(experiment_data)
      
  # Static table definition (if not regenerable)
  headers: ["Method", "Precision", "Recall", "F1"]
  rows:
    - ["Baseline", "0.85", "0.82", "0.83"]
    - ["Ours", "0.92", "0.89", "0.90"]
    
  # Styling
  alignment: ["left", "center", "center", "center"]
  highlight_rows: [1]  # 0-indexed
```

### 7. Cross-References

Documents need rich cross-referencing:

```yaml
references:
  internal:
    - type: "figure"
      label: "fig:quadratic"
      display: "Figure 1.1"
      target_id: "ch1-fig1"
      
    - type: "equation"
      label: "eq:einstein"
      display: "Equation (2)"
      target_id: "eq-einstein"
      
    - type: "section"
      label: "sec:intro"
      display: "Section 1"
      target_id: "ch1"
      
  bibliography:
    - key: "demoura2008"
      citation: |
        de Moura, L. and Bjørner, N. 
        Z3: An Efficient SMT Solver. 
        TACAS 2008.
      bibtex: |
        @inproceedings{demoura2008,
          title={Z3: An efficient SMT solver},
          author={De Moura, Leonardo and Bj{\o}rner, Nikolaj},
          booktitle={TACAS},
          year={2008}
        }
      doi: "10.1007/978-3-540-78800-3_24"
```

### 8. Document Structure

Beyond flat sections, we need hierarchy:

```yaml
structure:
  - type: "frontmatter"
    children:
      - type: "titlepage"
      - type: "abstract"
      - type: "acknowledgments"
      - type: "toc"
      
  - type: "mainmatter"
    children:
      - type: "chapter"
        number: 1
        title: "Introduction"
        id: "ch1"
        children:
          - type: "section"
            number: "1.1"
            title: "Background"
            chunks: ["ch1-p1", "ch1-eq1", "ch1-fig1"]
          - type: "section"
            number: "1.2"
            title: "Contributions"
            chunks: ["ch1-p2"]
            
  - type: "backmatter"
    children:
      - type: "bibliography"
      - type: "appendix", letter: "A", title: "Proofs"
      - type: "index"
```

### 9. Validation & Verification

Track which axioms apply and their status:

```yaml
validation:
  template: "stdlib/templates/mit_thesis.kleis"
  
  axioms_checked:
    - axiom: "title_on_every_page"
      status: "passed"
      checked_at: "2025-01-02T10:00:00Z"
      
    - axiom: "abstract_word_limit"
      status: "passed"
      details:
        word_count: 248
        limit: 350
        
    - axiom: "chapters_sequential"
      status: "passed"
      
    - axiom: "all_figures_referenced"
      status: "warning"
      details:
        unreferenced: ["fig:extra-plot"]
        
  overall_status: "valid"  # valid, warning, error
  last_full_validation: "2025-01-02T10:00:00Z"
```

### 10. Compilation Cache

To avoid recompiling unchanged content:

```yaml
cache:
  # Which chunks have been compiled
  compiled_chunks:
    "ch1-fig1":
      typst_hash: "sha256:abc..."
      svg_hash: "sha256:def..."
      compiled_at: "2025-01-02T10:00:00Z"
      
  # Full document outputs
  outputs:
    - format: "pdf"
      path: "output/thesis.pdf"
      hash: "sha256:..."
      generated_at: "2025-01-02T12:00:00Z"
      
    - format: "typst"
      path: "output/thesis.typ"
      hash: "sha256:..."
      generated_at: "2025-01-02T12:00:00Z"
```

### 11. Jupyter Integration

Track the connection to Jupyter cells:

```yaml
jupyter:
  notebook_path: "research/thesis_work.ipynb"
  last_sync: "2025-01-02T11:00:00Z"
  
  cell_mappings:
    - cell_id: "abc123"
      cell_type: "code"
      tags: ["kleisdoc:ch1-fig1"]
      kleisdoc_chunk: "ch1-fig1"
      execution_count: 42
      
    - cell_id: "def456"
      cell_type: "markdown"
      tags: ["kleisdoc:ch1-p1"]
      kleisdoc_chunk: "ch1-p1"
```

### 12. Collaboration & History

For multi-author documents:

```yaml
history:
  - version: 3
    timestamp: "2025-01-02T14:00:00Z"
    author: "author-001"
    message: "Updated results table with final numbers"
    chunks_modified: ["ch4-table1", "ch4-p3"]
    
  - version: 2
    timestamp: "2025-01-01T10:00:00Z"
    author: "author-001"
    message: "Added background chapter"
    chunks_added: ["ch2-p1", "ch2-p2", "ch2-fig1"]
    
comments:
  - id: "comment-001"
    author: "author-002"
    timestamp: "2025-01-02T15:00:00Z"
    chunk: "ch3-p4"
    text: "Can you clarify this derivation?"
    resolved: false
```

---

## File Format: YAML vs JSON vs Custom

### Recommendation: YAML with Kleis Extensions

```yaml
# thesis.kleisdoc
# KleisDoc v1.0

$schema: "https://kleis.io/schemas/kleisdoc/v1.0.json"

meta:
  id: "thesis-jane-smith-2025"
  version: 3
  created: "2024-09-01T00:00:00Z"
  modified: "2025-01-02T20:00:00Z"
  title: "Formal Verification of Knowledge Production Systems"
  template: "mit_thesis"
  
authors:
  - name: "Jane Smith"
    email: "jane@mit.edu"
    
structure:
  - !chapter
    number: 1
    title: "Introduction"
    chunks:
      - !text
        id: "ch1-intro"
        content: |
          Knowledge production in science...
          
      - !equation
        id: "ch1-eq1"
        latex: "\\forall x. P(x) \\Rightarrow Q(x)"
        label: "eq:logic"
        
      - !figure
        id: "ch1-fig1"
        kleis_code: |
          xs = range(0, 10)
          plot(xs, map(x -> x*x, xs))
        caption: "Quadratic growth"
        label: "fig:quadratic"
```

### Why YAML?

1. **Human readable** - Easy to inspect and debug
2. **Comments** - Can annotate with `#`
3. **Multi-line strings** - `|` for code blocks
4. **Type tags** - `!chapter`, `!equation` for explicit chunk types
5. **Git-friendly** - Diffs are meaningful
6. **JSON compatible** - Can convert to JSON for APIs

---

## Migration Path

### Phase 1: Core Types (Current)
- `DocChunk`, `DocSection`, `KleisDoc`
- Basic compilation to Typst
- ✅ Done in `kleisdoc_format.kleis`

### Phase 2: Rich Content (Next)
- Equation AST storage
- Figure regeneration
- Cross-reference tracking

### Phase 3: Validation Integration
- Template axiom checking
- Validation status tracking
- Error reporting

### Phase 4: Jupyter Integration
- Cell-to-chunk mapping
- Sync mechanism
- Export workflow

### Phase 5: Collaboration
- Version history
- Comments
- Multi-author support

---

## Open Questions

1. **How to handle large binary assets?**
   - Option A: Store paths, keep binaries external
   - Option B: Base64 encode small assets (< 1MB)
   - Option C: Git LFS for large files

2. **How to represent Kleis AST in YAML?**
   - Option A: JSON-like nested structure
   - Option B: S-expression string that we parse
   - Option C: Reference to `.kleis` file

3. **How to handle conflicting edits?**
   - Option A: Lock chunks during editing
   - Option B: Three-way merge like git
   - Option C: Operational transforms

4. **Should validation block compilation?**
   - Option A: Yes, errors must be fixed
   - Option B: Warnings allowed, errors block
   - Option C: User choice via flags

5. **How deep should the structure hierarchy go?**
   - Chapter > Section > Subsection > ?
   - What about nested lists, nested theorems?

---

## Next Steps

1. **Design Kleis data types** for complete chunk representation
2. **Define serialization format** (YAML with type tags)
3. **Implement round-trip parsing** (YAML ↔ Kleis AST)
4. **Add validation hooks** to compilation pipeline
5. **Build Jupyter magic** that reads/writes `.kleisdoc`

---

## Appendix: Example Complete `.kleisdoc`

See `examples/documents/complete_kleisdoc_example.yaml` (to be created)

