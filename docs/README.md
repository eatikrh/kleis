# Kleis Documentation

Comprehensive documentation for the Kleis symbolic language and computational framework.

---

## 📚 Quick Links

| Document | Description |
|----------|-------------|
| [NEXT_SESSION.md](NEXT_SESSION.md) | **Start here** - Current priorities and context |
| [adr/README.md](adr/README.md) | Architecture Decision Records (22 ADRs) |
| [grammar/kleis_grammar_v08.md](grammar/kleis_grammar_v08.md) | Current grammar specification |
| [../README.md](../README.md) | Project overview |

---

## 🏗️ Architecture Decision Records (ADRs)

**Directory:** [`adr/`](adr/README.md)

**22 ADRs** documenting key design choices:

| Key ADRs | Description |
|----------|-------------|
| [ADR-011](adr/adr-011-notebook-environment.md) | Kleis Notebook Environment |
| [ADR-015](adr/adr-015-text-as-source-of-truth.md) | Text as Source of Truth |
| [ADR-016](adr/adr-016-operations-in-structures.md) | Operations in Structures |
| [ADR-021](adr/adr-021-algebraic-data-types.md) | Algebraic Data Types |
| [ADR-022](adr/adr-022-z3-integration-for-axiom-verification.md) | Z3 Integration |

---

## 📂 Directory Structure

```
docs/
├── NEXT_SESSION.md          # ⭐ Current priorities and context
├── QUALITY_GATES.md         # Quality standards
│
├── adr/                     # Architecture Decision Records (22 ADRs)
├── grammar/                 # Grammar specs (v03-v08, EBNF, ANTLR4)
├── guides/                  # Implementation guides
├── type-system/             # Type inference documentation
├── parser-implementation/   # Parser status and compatibility
├── notation/                # Notation design documents
├── solver-abstraction/      # Z3 solver architecture
├── testing/                 # Test strategies
│
├── vision/                  # Future roadmap documents
├── theory/                  # Mathematical theory
├── mathematics/             # Research papers (TeX + PDF)
├── hont/                    # HONT/POT papers (TeX + PDF)
├── LLMs/                    # LLM integration research
│
├── proposals/               # Proposed features
├── reference/               # Reference documentation
│
├── session-2025-12-13/      # Current session (only active session)
│
└── archive/                 # Historical documents
    └── sessions/            # All past sessions (12-03 through 12-12)
```

---

## 📝 Current Session

### Session 2025-12-13 (Active)
**Focus:** Palette Kleis coverage, AST translation

**Files:**
- `PALETTE_KLEIS_COVERAGE.md` - Analysis of palette button coverage
- `AST_TRANSLATION_PROBLEM.md` - Editor AST → Kleis translation
- `SESSION_SUMMARY.md` - Session summary

**Next:** Kleis Notebook development (ADR-011)

---

## 📖 Key References

### Grammar (Current: v0.8)
- [`grammar/kleis_grammar_v08.md`](grammar/kleis_grammar_v08.md) - Human-readable spec
- [`grammar/kleis_grammar_v08.ebnf`](grammar/kleis_grammar_v08.ebnf) - Machine-readable EBNF
- `grammar/Kleis_v07.g4` - ANTLR4 grammar (pending v0.8 sync)

### Type System
- [`type-system/FORMAL_SPECIFICATION.md`](type-system/FORMAL_SPECIFICATION.md) - Formal spec
- [`type-system/HINDLEY_MILNER_STATUS.md`](type-system/HINDLEY_MILNER_STATUS.md) - HM implementation

### Guides
- [`guides/PALETTE_GUIDE.md`](guides/PALETTE_GUIDE.md) - Equation Editor palette
- [`guides/TEST_GUIDE.md`](guides/TEST_GUIDE.md) - Testing guide

---

## 🔬 Research Papers

### Projected Ontology Theory (POT)
- `hont/projected_ontology_theory.pdf`
- `hont/projected_ontology_theory_naive_questions.pdf`

### Mathematics
- `mathematics/` - 15+ research papers (TeX + PDF)

### LLM Integration
- `LLMs/Tracer_Framework_Report_fixed.pdf`

---

## 📦 Archive

Past sessions and deprecated documents are in [`archive/`](archive/).

| Archived | Content |
|----------|---------|
| `archive/sessions/` | Sessions 2025-12-03 through 2025-12-12 |
| `archive/type-system-roadmaps/` | Historical type system plans |
| `archive/grammar/` | Old grammar versions |

---

**Last Updated:** December 15, 2025
