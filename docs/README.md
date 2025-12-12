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
- **Grammar:** `grammar/kleis_grammar_v03.md` - Formal grammar specification (v0.3)
- **Vision:** `kleis_vision_executable_math.md` - Future roadmap

### Ontology Theories
- **POT:** `POT.md` - Projected Ontology Theory
- **HONT:** `HONT.md` - Hilbert Ontology
- **Papers:** `hont/` - LaTeX papers and PDFs

---

## 🏗️ Architecture Decision Records (ADRs)

**All ADRs consolidated in:** [`adr/`](adr/README.md) directory

**22 ADRs** documenting key design choices across:
- Core language design (ADR-001 to ADR-003)
- Type system (ADR-014 to ADR-021)
- User interface & editing (ADR-004, ADR-005, ADR-009 to ADR-012, ADR-017)
- Grammar & parsing (ADR-006 to ADR-008)
- Formalism & theory (ADR-013, ADR-018)

**Key ADRs:**
- ⭐ **[ADR-015: Text as Source of Truth](adr/adr-015-text-as-source-of-truth.md)** - Notation system foundation
- ⭐ **[ADR-016: Operations in Structures](adr/adr-016-operations-in-structures.md)** - Type system architecture
- ⭐ **[ADR-021: Algebraic Data Types](adr/adr-021-algebraic-data-types.md)** - Pattern matching system

**See:** [adr/README.md](adr/README.md) for complete index with descriptions

---

## 📝 Recent Sessions

### Session 2024-12-08: Signature-Driven Type System ⭐⭐
**Focus:** SignatureInterpreter improvements + End-to-end testing  
**Status:** ✅ Complete - Phase 1: 80% done  
**See:** [Session Summary](session-2024-12-08/README.md)

**Achievements:**
- ✅ Match statement: 229 → 61 lines (73% reduction!)
- ✅ SignatureInterpreter enforces dimension constraints
- ✅ Pattern-based (zero hardcoded operation names)
- ✅ TRUE user-extensibility (Matrix = User operations)
- ✅ 364 tests passing, browser verified
- ✅ Tagged: v0.5.0-signature-driven

### Session 2024-12-07: Type System & Stdlib Integration ⭐
**Focus:** Connecting standard library to type inference  
**Status:** ✅ Complete - ADR-016 compliance achieved  
**See:** Session archived

**Achievements:**
- ✅ Stdlib loading infrastructure
- ✅ Reduced hardcoded operations by 88%
- ✅ ADR-016 compliant (operations in structures)
- ✅ Type system now self-hosting
- ✅ Working in browser with comprehensive tests

### Session 2024-12-06: Notation Design + Type Checking
**Focus:** Notation design + Type checking infrastructure  
**Status:** ✅ Complete with working implementation  
**See:** Session archived

### Session 2024-12-09: Matrix Improvements & Reality Check ⭐⭐
**Focus:** Matrix system cleanup + Testing & honest assessment  
**Status:** ✅ Complete - All pushed to GitHub  
**See:** [Session Summary](session-2024-12-09/README.md)

**Achievements:**
- ✅ Matrix constructor cleanup (generic, zero hardcoding)
- ✅ Recursive type unification (block matrices work!)
- ✅ Tensor operations for General Relativity
- ✅ 12 new stdlib function tests
- ✅ Honest documentation of self-hosting limitations
- ✅ 425 tests passing, all quality gates pass

### Subdirectories

**Core Documentation:**
- [`adr/`](adr/README.md) - **Architecture Decision Records (22 ADRs)** ⭐
- [`reference/`](reference/README.md) - **Reference documentation** (overview, technical specs, templates, analysis)
- [`guides/`](guides/) - Implementation guides and best practices
- [`testing/`](testing/README.md) - Test data and testing strategies

**Technical Specifications:**
- [`grammar/`](grammar/) - Formal grammar specifications (EBNF, ANTLR4, v0.5)
- [`type-system/`](type-system/) - Type inference and checking documentation
- [`parser-implementation/`](parser-implementation/) - Parser compatibility analyses
- [`notation/`](notation/) - Notation design documents and test cases

**Vision & Theory:**
- [`vision/`](vision/) - Future roadmap and vision documents
- [`theory/`](theory/) - Mathematical theory documents
- [`hont/`](hont/) - HONT/POT papers (LaTeX + PDF)
- [`LLMs/`](LLMs/) - LLM integration research

**Session Reports:** (Last 2-3 sessions)
- [`session-2024-12-10/`](session-2024-12-10/README.md) - Self-hosting actually fixed ⭐⭐
- [`session-2024-12-09/`](session-2024-12-09/README.md) - Matrix improvements & reality check
- [`session-2024-12-08/`](session-2024-12-08/README.md) - Signature-driven type system

**Archive:**
- [`archive/`](archive/) - Historical documents and deprecated content
- [`archive/sessions/`](archive/sessions/README.md) - Archived session reports (2024-12-06, 2024-12-07)

---

## 📖 Language Reference

### Syntax & Grammar
- **`syntax.md`** - Complete syntax reference
- **`grammar/kleis_grammar_v03.md`** - EBNF grammar specification (v0.3)
- **`grammar/kleis_grammar_v03.ebnf`** - Machine-readable EBNF (v0.3)
- **`grammar/Kleis_v03.g4`** - ANTLR4 grammar (v0.3)
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
├── README.md                           # This file - main navigation
│
├── adr/                                # ⭐ Architecture Decision Records
│   ├── README.md                       # ADR index (22 ADRs)
│   ├── adr-001-scalar-multiply.md
│   ├── adr-002-eval-vs-simplify.md
│   └── ... (20 more ADRs)
│
├── reference/                          # ⭐ Reference Documentation
│   ├── README.md                       # Reference index
│   ├── KLEIS_OVERVIEW.md               # Project overview
│   ├── TECHNICAL_REFERENCE.md          # Technical specs
│   ├── COMPLETE_TEMPLATE_REFERENCE.md  # Template system
│   ├── COMPUTATIONAL_COMPLEXITY.md     # Performance analysis
│   └── HARD_PROBLEMS_AHEAD.md          # Future challenges
│
├── session-2024-12-09/                 # ⭐ Latest session
│   ├── README.md                       # Session navigation
│   ├── SESSION_SUMMARY.md              # Complete summary
│   ├── SESSION_CORRECTION.md           # Critical reality check
│   ├── UNIVERSAL_CONSTANTS_FINDING.md  # Research discovery
│   └── PHYSICAL_CONSTANTS_PALETTE.md   # Design document
│
├── grammar/                            # Grammar specifications (v0.5)
├── type-system/                        # Type system docs
├── guides/                             # Implementation guides
│   ├── GITHUB_ACTIONS_SETUP.md         # CI/CD guide
│   ├── PALETTE_GUIDE.md
│   └── ...
├── testing/                            # Test data & strategies
│   ├── README.md
│   └── DLMF_INTEGRATION.md             # NIST test equations
├── parser-implementation/              # Parser analyses
│   ├── KLEIS_PARSER_STATUS.md          # Parser status
│   └── ...
├── notation/                           # Notation design
├── theory/                             # Mathematical theory
├── vision/                             # Future roadmap
├── hont/                               # HONT/POT papers (LaTeX + PDF)
├── LLMs/                               # LLM integration research
└── archive/                            # Historical documents
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

**Last Updated:** December 9, 2024

