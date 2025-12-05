# Final Documentation Consolidation Plan

**Date:** December 5, 2024

---

## Current State: docs/type-system/

9 files, some overlap:

1. ✅ HASKELL_INTEGRATION.md (9.4K) - Already consolidated, keep
2. ✅ TYPE_CHECKING_UX.md (9.1K) - Already consolidated, keep
3. 🔄 TYPE_CONTEXT_BOOTSTRAP.md (20K) - Bootstrap & context
4. 🔄 OPERATION_BASED_TYPE_INFERENCE.md (20K) - Operation queries
5. ✅ SYNTAX_COMPARISON_AND_PROPOSAL.md (14K) - Design decisions, keep
6. ✅ TYPE_INFERENCE_POC.md (8.7K) - Status report, keep
7. ✅ KLEIS_TYPE_SYSTEM.md (42K) - Core spec, keep
8. ✅ KLEIS_TYPE_UX.md (31K) - UX design, keep
9. ✅ KLEIS_EVALUATION_SYNTAX.md (12K) - Evaluation, keep

**Overlap:** Files 3 & 4 overlap ~40%

---

## Consolidation Action

### Merge: Context + Operations → CONTEXT_AND_OPERATIONS.md

**Merge:**
- TYPE_CONTEXT_BOOTSTRAP.md (bootstrap strategy)
- OPERATION_BASED_TYPE_INFERENCE.md (operation queries)

**Into:** `docs/type-system/CONTEXT_AND_OPERATIONS.md` (~35K)

**Sections:**
1. Context Bootstrap (3-tier strategy)
2. Operation Registry and Manifests
3. Type Inference with Operations
4. Loading stdlib/prelude.kleis
5. API Design

---

## Grammar Consolidation

### Move grammar changes into main doc

**Merge:** GRAMMAR_V03_CHANGES.md → section in kleis_grammar_v03.md

Keep formal specs:
- ✅ Kleis_v03.g4 (ANTLR4)
- ✅ kleis_grammar_v03.ebnf (EBNF)
- ✅ kleis_grammar_v03.md (extended with changes section)

---

## Examples/Demos Consolidation

**Move:** examples/context_bootstrap_demo.md → docs/type-system/examples/

---

## Final Structure

```
docs/type-system/
├── KLEIS_TYPE_SYSTEM.md (42K)           # Core spec
├── KLEIS_TYPE_UX.md (31K)               # UX design
├── KLEIS_EVALUATION_SYNTAX.md (12K)     # Evaluation
├── HASKELL_INTEGRATION.md (9.4K)        # Haskell adoption
├── CONTEXT_AND_OPERATIONS.md (~35K)     # NEW: Bootstrap + operations
├── TYPE_CHECKING_UX.md (9.1K)           # UX with 5 states
├── SYNTAX_COMPARISON_AND_PROPOSAL.md (14K) # Design choices
├── TYPE_INFERENCE_POC.md (8.7K)         # POC status
└── examples/
    └── context_bootstrap_demo.md        # Step-by-step demo

docs/grammar/
├── Kleis_v03.g4                         # ANTLR4 grammar
├── kleis_grammar_v03.ebnf               # EBNF grammar
└── kleis_grammar_v03.md                 # Prose (with changes)

stdlib/
├── prelude.kleis                        # Standard library
└── README.md                            # Overview
```

**Result:** 8 core docs (down from 9), plus examples/ subdir

---

**Execute?** This will consolidate context/operations docs.

