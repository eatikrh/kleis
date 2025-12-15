# Commit Plan - Grammar v0.6 Implementation

**Date:** December 12, 2025  
**Feature:** Grammar v0.6 with Functions in Structures

---

## ✅ Files to Commit

### 1. Grammar Files (New - v0.6)
```
docs/grammar/kleis_grammar_v06.ebnf          # EBNF specification
docs/grammar/Kleis_v06.g4                    # ANTLR4 grammar
docs/grammar/kleis_grammar_v06.md            # Documentation
vscode-kleis/docs/grammar/kleis_grammar_v06.ebnf  # VSCode sync
```

### 2. Parser & AST Changes
```
src/kleis_ast.rs                             # Added FunctionDef to StructureMember
src/kleis_parser.rs                          # Parse functionDef instead of skip (removed TODO #11)
```

### 3. Tests
```
tests/grammar_v06_function_in_structure_test.rs  # 4 comprehensive tests
```

### 4. Documentation Updates
```
docs/README.md                               # Updated grammar refs: v0.5 → v0.6
docs/grammar/GRAMMAR_SYNC_STATUS.md          # Updated to v0.6
docs/parser-implementation/PARSER_GRAMMAR_COMPATIBILITY.md  # Updated to v0.6
```

### 5. LaTeX Learning Materials
```
docs/mathematics/kleis_language_specification.tex  # Added v0.6 example, updated version
```

### 6. Session Documentation (All New)
```
docs/session-2025-12-12/TODO_INVENTORY.md
docs/session-2025-12-12/GRAMMAR_TODO_ANALYSIS.md
docs/session-2025-12-12/GRAMMAR_V06_RATIONALE.md
docs/session-2025-12-12/GRAMMAR_V06_IMPLEMENTATION_COMPLETE.md
docs/session-2025-12-12/DOCUMENTATION_UPDATED_V06.md
docs/session-2025-12-12/TEX_FILES_GRAMMAR_V06_REVIEW.md
docs/session-2025-12-12/COMMIT_PLAN.md
```

---

## ❌ Files NOT to Commit

### Generated Files (Should be ignored)
```
docs/mathematics/kleis_language_specification.pdf  # Generated from .tex
docs/mathematics/magma_semigroup_monoid.pdf        # Generated from .tex
docs/mathematics/mathematicians_guide_to_kleis.pdf # Generated from .tex
```
**Reason:** PDFs are build artifacts, can be regenerated from `.tex` sources

### Root-Level Temporary Files
```
NEXT_SESSION_TODO_REVIEW.md           # Should be in docs/session-* or deleted
NEXT_SESSION_UPDATE_ADR022.md         # Should be in docs/session-* or deleted
```
**Reason:** Temporary planning files, not part of the Grammar v0.6 feature

### Unrelated Changes
```
docs/grammar/Kleis_v05.g4             # We renamed this to v06, why is it modified?
docs/grammar/kleis_grammar_v05.ebnf   # We renamed this to v06, why is it modified?
docs/vision/FIRST_LLM_WRITING_KLEIS.md    # Unrelated to Grammar v0.6?
docs/vision/LLMS_WRITING_KLEIS.md         # Unrelated to Grammar v0.6?
```
**Action Required:** Check what changed in these files

---

## 🔍 Files Needing Review

Let me check what changed in the v0.5 grammar files and vision docs:

```bash
git diff docs/grammar/Kleis_v05.g4
git diff docs/grammar/kleis_grammar_v05.ebnf
git diff docs/vision/FIRST_LLM_WRITING_KLEIS.md
git diff docs/vision/LLMS_WRITING_KLEIS.md
```

---

## 📝 Suggested Commit Message

```
feat: Grammar v0.6 - Functions in Structures

Add support for function definitions inside structures, enabling derived
operations with default implementations.

Grammar Changes:
- Add functionDef to structureMember production in EBNF and ANTLR4 grammars
- Update grammar documentation to v0.6
- Sync VSCode extension grammar

Parser/AST Changes:
- Add StructureMember::FunctionDef variant to AST
- Parse function definitions in structures (was skipping with TODO #11)
- Remove TODO #11 - feature now fully implemented

Tests:
- Add 4 comprehensive tests for functions in structures
- All 600+ tests passing (421 unit + 200+ integration)

Documentation:
- Update all living documentation to reference Grammar v0.6
- Add v0.6 example to kleis_language_specification.tex
- Create comprehensive session documentation

Examples:
  structure Ring(R) {
    operation (-) : R × R → R
    define (-)(x, y) = x + negate(y)  // NEW in v0.6!
  }

Resolves: #11 (TODO in parser)
Quality Gates: ✅ fmt, ✅ clippy, ✅ all tests pass
```

---

## 📊 Statistics

- **Grammar files:** 4 new files
- **Code changes:** 2 files (AST, Parser)
- **Tests:** 1 new test file (4 tests)
- **Documentation:** 4 files updated, 7 session docs created
- **Total files to commit:** ~18 files
- **Tests passing:** 600+ (all)
- **Lines of code changed:** ~100 lines (mostly additions)

---

## ✅ Pre-Commit Checklist

- [x] All grammar files created (EBNF, ANTLR4, MD)
- [x] VSCode grammar synced
- [x] Parser updated to parse (not skip)
- [x] AST updated with new variant
- [x] Tests created and passing
- [x] Quality gates passed (fmt, clippy, test)
- [x] Documentation updated
- [x] Session docs created
- [ ] Review unrelated file changes
- [ ] Decide on PDF files (commit or .gitignore)
- [ ] Move/delete root-level NEXT_SESSION_*.md files

