# Document System Roadmap

## Vision
Create a complete pipeline from Jupyter Notebook exploration to publication-quality documents (PDF, LaTeX) with formal verification of document structure.

## What's Done ✅

### Core Architecture
- [x] **Document = Program insight** - Thesis compilation follows LISP interpreter pattern
- [x] **DocExpr AST** - Structured representation of document elements
- [x] **CompileResult** - Tagged return values (CTypst/CError)
- [x] **Template separation** - `stdlib/templates/mit_thesis.kleis` as reusable library

### Thesis Compiler
- [x] `thesis_compiler.kleis` - Compiles DocExpr to Typst (6/6 tests pass)
- [x] `jane_smith_thesis.kleis` - Example user thesis (4/4 tests pass)
- [x] `generated_thesis.pdf` - Working PDF output (29KB)

### Document Axioms
- [x] `thesis_simple.kleis` - MIT thesis requirements (5/5 tests pass)
- [x] `arxiv_simple.kleis` - arXiv paper structure (9/9 tests pass)

### Persistent Format
- [x] `kleisdoc_format.kleis` - .kleisdoc format in Kleis (4/4 tests pass)
- [x] `jupyter_integration.md` - Design document for Jupyter → .kleisdoc

### LISP Proof of Concept
- [x] `lisp_parser.kleis` - Full interpreter (9/9 tests pass, including recursion)

---

## Phase 1: Complete Kleis-Side Implementation

### 1.1 Implement `intToStr` Builtin
**Priority:** High | **Effort:** Small

```rust
// In src/evaluator.rs
"intToStr" => {
    if let Expression::Const(n) = &args[0] {
        Ok(Some(Expression::String(n.to_string())))
    } else {
        Err("intToStr requires a numeric constant".to_string())
    }
}
```

- [ ] Add to `builtin_call` match in evaluator
- [ ] Remove `num_to_str` hack from template files
- [ ] Test with chapter numbers > 12

### 1.2 Add More Document Elements
**Priority:** Medium | **Effort:** Medium

- [ ] `MITTable(caption, data)` - Data tables
- [ ] `MITCode(language, code)` - Code listings
- [ ] `MITFootnote(text)` - Footnotes
- [ ] `MITCitation(key)` - In-text citations
- [ ] `MITAppendix(letter, title, content)` - Appendices

### 1.3 Add Validation Axioms
**Priority:** High | **Effort:** Medium

- [ ] Abstract word count limits
- [ ] Figure numbering consistency (chapter-prefixed)
- [ ] Reference completeness (all citations have entries)
- [ ] Chapter ordering validation

### 1.4 Add More Templates
**Priority:** Medium | **Effort:** Medium per template

- [ ] `stdlib/templates/arxiv_paper.kleis`
- [ ] `stdlib/templates/ieee_paper.kleis`
- [ ] `stdlib/templates/acm_paper.kleis`

---

## Phase 2: Python/Jupyter Integration

### 2.1 Implement KleisDoc Python Classes
**Priority:** High | **Effort:** Medium

```python
# kleis_notebook/document.py
@dataclass
class DocChunk:
    id: str
    chunk_type: str
    source: ContentSource
    typst_code: str
    ...

@dataclass
class KleisDoc:
    def save(self, path: str): ...
    def load(cls, path: str) -> 'KleisDoc': ...
    def get_chunk(self, id: str) -> DocChunk: ...
    def regenerable_chunks(self) -> List[DocChunk]: ...
```

- [ ] Create `kleis_notebook/document.py`
- [ ] Implement YAML serialization
- [ ] Add chunk CRUD operations

### 2.2 Implement Cell Tagging
**Priority:** High | **Effort:** Medium

```python
%%kleisdoc id=ch1-fig1, type=figure, section=chapter-1, caption="My plot"
diagram(plot(xs, ys))
```

- [ ] Create `%%kleisdoc` cell magic
- [ ] Parse metadata from magic line
- [ ] Store metadata with cell output

### 2.3 Implement Export Function
**Priority:** High | **Effort:** Medium

```python
def export_kleisdoc(notebook_path: str, output_path: str):
    """Extract tagged cells and create .kleisdoc file."""
    ...
```

- [ ] Parse notebook JSON
- [ ] Extract tagged cells
- [ ] Group by section
- [ ] Generate .kleisdoc YAML

### 2.4 Implement Compile Function
**Priority:** High | **Effort:** Medium

```python
def compile_to_pdf(doc: KleisDoc, output_path: str):
    """Compile .kleisdoc to PDF via Typst."""
    typst_code = compile_kleisdoc(doc)  # Call Kleis
    run_typst(typst_code, output_path)
```

- [ ] Generate Typst from KleisDoc
- [ ] Save figure SVGs to disk
- [ ] Run `typst compile`
- [ ] Return PDF path or bytes

### 2.5 Implement PDF Display in Jupyter
**Priority:** Medium | **Effort:** Small

```python
from IPython.display import display, PDF
display(PDF(compile_to_pdf(my_doc)))
```

- [ ] Add `display_pdf()` helper
- [ ] Handle inline vs. download options

---

## Phase 3: Equation Editor Integration

### 3.1 Embed Equation Editor in Jupyter
**Priority:** Medium | **Effort:** Medium

- [ ] Package Equation Editor for iframe embedding
- [ ] Create `%%equation` magic that opens editor
- [ ] Receive EditorNode AST via postMessage
- [ ] Store AST in .kleisdoc for re-editing

### 3.2 AST Preservation
**Priority:** Medium | **Effort:** Medium

```yaml
# In .kleisdoc
- id: ch1-eq1
  type: equation
  ast:
    Operation:
      name: "equals"
      args: [...]
  typst: "$ E = m c^2 $"
```

- [ ] Store EditorNode AST in chunk
- [ ] Reload AST when editing equation
- [ ] Regenerate Typst from AST on change

---

## Phase 4: Multi-Session Editing

### 4.1 Stable IDs
**Priority:** High | **Effort:** Small

- [ ] Generate UUIDs for new chunks
- [ ] Preserve IDs across saves
- [ ] Warn on duplicate IDs

### 4.2 Version Tracking
**Priority:** Medium | **Effort:** Small

```yaml
version: 3
modified: "2025-01-02T20:00:00Z"
```

- [ ] Increment version on save
- [ ] Update modified timestamp
- [ ] Optional: track change history

### 4.3 Regeneration
**Priority:** Medium | **Effort:** Medium

```python
for chunk in doc.regenerable_chunks():
    if chunk.needs_update():
        new_output = run_kleis(chunk.source_code)
        chunk.update(new_output)
```

- [ ] Detect stale chunks
- [ ] Re-run Kleis code
- [ ] Update Typst output
- [ ] Mark as regenerated

---

## Phase 5: Polish & Documentation

### 5.1 Manual Chapter
**Priority:** Medium | **Effort:** Medium

- [ ] Add chapter to manual: "Document Generation"
- [ ] Tutorial: Creating a thesis from Jupyter
- [ ] Reference: Template library

### 5.2 Example Notebooks
**Priority:** Medium | **Effort:** Medium

- [ ] `examples/notebooks/mini_thesis.ipynb`
- [ ] `examples/notebooks/arxiv_paper.ipynb`
- [ ] Step-by-step walkthrough

### 5.3 Error Messages
**Priority:** Low | **Effort:** Small

- [ ] Friendly validation errors
- [ ] Suggestions for fixing structure issues

---

## Success Criteria

### Minimum Viable Product (MVP)
1. ✅ Kleis compiles thesis to Typst
2. ✅ Typst compiles to PDF
3. [ ] Python can export Jupyter cells to .kleisdoc
4. [ ] Python can compile .kleisdoc to PDF
5. [ ] Basic cell tagging works

### Full Product
1. [ ] All Phase 1-4 items complete
2. [ ] Real thesis created end-to-end
3. [ ] Manual documentation complete
4. [ ] At least 3 venue templates (MIT, arXiv, IEEE)

---

## Estimated Timeline

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| Phase 1 | 2-3 days | None |
| Phase 2 | 3-5 days | Phase 1 |
| Phase 3 | 2-3 days | Phase 2 |
| Phase 4 | 1-2 days | Phase 2 |
| Phase 5 | 2-3 days | Phase 1-4 |

**Total: ~2-3 weeks of focused work**

---

## Notes

- All Kleis-side work (Phase 1) can proceed independently
- Python work (Phase 2-4) depends on stable Kleis API
- Equation Editor integration (Phase 3) is valuable but not blocking
- Each phase delivers incremental value

