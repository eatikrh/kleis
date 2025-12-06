# Kleis Documentation

Comprehensive documentation for the Kleis symbolic language and computational framework.

---

## 📚 Quick Links

### Getting Started
- **Main README:** `../README.md` - Project overview and quick start
- **Server API:** `../SERVER_README.md` - HTTP API reference
- **Parser Status:** `../PARSER_TODO.md` - LaTeX parser capabilities

### Core Concepts
- **Syntax Reference:** `syntax.md` - Kleis language syntax
- **Grammar:** `kleis_grammar_v02.md` - Formal grammar specification
- **Vision:** `kleis_vision_executable_math.md` - Future roadmap

### Ontology Theories
- **POT:** `POT.md` - Projected Ontology Theory
- **HONT:** `HONT.md` - Hilbert Ontology
- **Papers:** `hont/` - LaTeX papers and PDFs

---

## 🏗️ Architecture Decision Records (ADRs)

Decision records documenting key design choices:

### Core Design
- **[ADR-001](adr-001-scalar-multiply.md)** - Scalar Multiply Operation
- **[ADR-002](adr-002-eval-vs-simplify.md)** - Expression Evaluation vs Simplification
- **[ADR-003](adr-003-self-hosting.md)** - Self-Hosting Strategy

### Visualization & UI
- **[ADR-004](adr-004-input-visualization.md)** - Input Visualization
- **[ADR-005](adr-005-visual-authoring.md)** - Visual Math Authoring System (Long-term Vision)

### Grammar & Parsing
- **[ADR-006](adr-006-template-grammar-duality.md)** - Template-Grammar Duality
- **[ADR-007](adr-007-bootstrap-grammar.md)** - Bootstrap Grammar
- **[ADR-008](adr-008-bootstrap-grammar-boundary.md)** - Bootstrap Grammar Boundary

### Editing & Notation System
- **[ADR-009](adr-009-wysiwyg-structural-editor.md)** - WYSIWYG Structural Editor
- **[ADR-010](adr-010-inline-editing.md)** - Inline Editing
- **[ADR-011](adr-011-notebook-environment.md)** - Notebook Environment
- **[ADR-012](adr-012-document-authoring.md)** - Document Authoring
- **[ADR-014](adr-014-hindley-milner-type-system.md)** - Hindley-Milner Type System
- **[ADR-015](adr-015-text-as-source-of-truth.md)** - ⭐ Text as Source of Truth (Notation System)
- **[ADR-016](ADR-016-operations-in-structures.md)** - ⭐ Operations in Structures (Type System)

### Type System
- **[Type System Overview](type-system/KLEIS_TYPE_SYSTEM.md)** - Complete type system design
- **[Type Inference POC](type-system/TYPE_INFERENCE_POC.md)** - Proof of concept

---

## 📝 Notation & Type System (Dec 6, 2024)

**Major session:** Notation design + Type checking infrastructure  
**Status:** ✅ Complete with working implementation  
**See:** [Session Summary](session-2024-12-06/README.md)

### Core ADRs
1. **[ADR-015: Text as Source of Truth](adr-015-text-as-source-of-truth.md)** ⭐
   - Text representation for Kleis code
   - Explicit forms (`abs`, `frac`, etc.)
   - Git-friendly design

2. **[ADR-016: Operations in Structures](ADR-016-operations-in-structures.md)** ⭐
   - Operations belong to structures
   - Implements pattern for concrete types
   - Enables polymorphism

### Implementation
- **Parser:** `src/kleis_parser.rs` (1097 lines)
- **Type Context:** `src/type_context.rs` (313 lines)  
- **Type Checker:** `src/type_checker.rs` (251 lines)
- **Tests:** 25+ passing ✅

### Subdirectories
- `notation/` - Notation design documents and test cases
- `parser-implementation/` - Parser compatibility analyses
- `session-2024-12-06/` - Session-specific documents and milestones
- `type-system/` - Type inference and checking documentation

---

## 📖 Language Reference

### Syntax & Grammar
- **`syntax.md`** - Complete syntax reference
- **`kleis_grammar_v02.md`** - EBNF grammar specification
- **`kleis_grammar_v02.ebnf`** - Machine-readable EBNF
- **`Kleis.g4`** - ANTLR4 grammar
- **`kleis.pest`** - Pest parser grammar

### Expression System
- **Types:** Const, Object, Operation
- **Operations:** 56 mathematical operations (see main README)
- **Rendering:** Unicode and LaTeX output formats

---

## 🔬 Ontology Theories

### Projected Ontology Theory (POT)
Modal ontological framework for expressing projection, residue, and emergent phenomena.

**Files:**
- `POT.md` - Overview
- `hont/projected_ontology_theory.pdf` - Full paper
- `hont/projected_ontology_theory_naive_questions.pdf` - Q&A

### Hilbert Ontology (HONT)
Metaphysical clarity meets mathematical structure.

**Files:**
- `HONT.md` - Overview
- `hont/hont_modal_enrichment.pdf` - Modal enrichment paper
- `hont/POT_Hont_Session_Summary_1_Simple.pdf` - Session summary

### Related Papers
- `hont/dirac_leap_credit.pdf` - Dirac's contribution
- `hont/philosophy_101_lost.pdf` - Philosophical foundations
- `hont/justice_factcheck.pdf` - Justice perspective

---

## 🎯 Vision Documents

### Future Roadmap
- **`kleis_vision_executable_math.md`** - Vision for executable mathematics
  - Living framework for mathematical structure
  - Package distribution system
  - Visual authoring environment

### Key Concepts
1. **Executable Notation** - Notation becomes computable
2. **Shareable Algebra** - Math systems as software packages
3. **Visual Reasoning** - Symbolic reasoning as a shared visual medium

---

## 🤖 LLM Integration

Research on integrating Kleis with Large Language Models:

- **`LLMs/Tracer_Framework_Report_fixed.pdf`** - Tracer framework for formal reasoning

Future goal: LLMs output formal Kleis DSL instead of natural language, creating provable reasoning chains.

---

## 📂 Directory Structure

```
docs/
├── README.md                    # This file
├── syntax.md                    # Syntax reference
├── POT.md                       # Projected Ontology Theory overview
├── HONT.md                      # Hilbert Ontology overview
├── kleis_vision_executable_math.md  # Future vision
├── kleis_grammar_v02.md         # Grammar specification
├── adr-00X-*.md                 # Architecture Decision Records
├── notation-design-summary.md   # ⭐ Notation system design overview
├── notation-mapping-tests.md    # Text ↔ visual mapping tests
├── notation-poc-tests.md        # Proof of concept tests
├── content-editing-paradigm.md  # Editing paradigm discussion
├── grammar/                     # Grammar specifications
│   ├── kleis_grammar_v03.md
│   ├── kleis_grammar_v03.ebnf
│   └── Kleis_v03.g4
├── type-system/                 # Type system documentation
│   ├── KLEIS_TYPE_SYSTEM.md
│   ├── TYPE_INFERENCE_POC.md
│   └── examples/
├── guides/                      # Implementation guides
│   ├── PALETTE_GUIDE.md
│   ├── INLINE_EDITING.md
│   └── TEST_GUIDE.md
├── hont/                        # HONT/POT papers (LaTeX + PDF)
│   ├── projected_ontology_theory.pdf
│   ├── hont_modal_enrichment.pdf
│   └── ...
└── LLMs/                        # LLM integration research
    └── Tracer_Framework_Report_fixed.pdf
```

---

## 🔗 External Links

- **Project Repository:** (Add GitHub URL when available)
- **Issue Tracker:** (Add issue tracker URL)
- **Discussions:** (Add discussions URL)

---

## 📝 Contributing to Documentation

### Adding New ADRs

```markdown
# ADR-XXX: Title

## Status
Proposed / Accepted / Deprecated

## Context
Why is this decision needed?

## Decision
What did we decide?

## Consequences
What are the implications?
```

### Updating Documentation

1. Keep ADRs immutable (add new ADRs instead of editing)
2. Update syntax.md when language changes
3. Update this README when adding new documentation
4. Keep code examples up to date

---

**Last Updated:** December 6, 2024

