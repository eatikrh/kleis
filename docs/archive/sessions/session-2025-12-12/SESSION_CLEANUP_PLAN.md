# Session 2025-12-12 Documentation Cleanup Plan

**Created:** December 12, 2025  
**Documents:** 26 files (214KB total)  
**Status:** Needs consolidation and organization

---

## 📊 Document Categories

### 🏗️ Solver Abstraction (Core - 61KB)

**Keep and Consolidate:**
1. **SOLVER_ABSTRACTION_LAYER_DESIGN.md** (20K) ← Primary design doc
2. **SOLVER_MCP_STYLE_CAPABILITIES.md** (22K) ← MCP architecture  
3. **SOLVER_AST_RETURN_REQUIREMENT.md** (19K) ← Critical boundary requirement

**Action:**
- [ ] Combine these 3 into one comprehensive architecture document
- [ ] Move to permanent location: `docs/solver-abstraction/ARCHITECTURE.md`
- [ ] Create ADR-023 based on consolidated doc

### 🔧 Z3 Implementation Details (11 docs - 77KB)

**Z3-Specific Investigations:**
1. BUILTIN_FUNCTIONS_Z3_COVERAGE.md (11K) - Coverage analysis
2. RUST_OPERATOR_OVERLOADING_IN_Z3.md (4.7K) - Implementation notes
3. Z3_CALCULUS_CAPABILITIES.md (7.1K) - What Z3 can't do
4. Z3_FUNCTION_COMPOSITION_FIX.md (3.5K) - RecFuncDecl solution
5. Z3_FUNCTION_INTEGRATION_DESIGN.md (15K) - Grammar v0.6 integration
6. Z3_MODEL_EVALUATION.md (6.5K) - How model.eval works
7. Z3_QUANTIFIER_INSTANTIATION_ISSUE.md (3.6K) - Quantifier bug
8. Z3_RECFUNCDECL_SOLUTION.md (6.2K) - RecFuncDecl fix
9. Z3_RESULT_TO_KLEIS_AST.md (8.0K) - Result conversion
10. Z3_VS_EVALUATOR_FUNCTIONS.md (8.8K) - Backend comparison
11. TRANSCENDENTAL_FUNCTIONS.md (5.5K) - Matrix transcendentals

**Action:**
- [ ] Consolidate into 2-3 docs:
  - `Z3_IMPLEMENTATION_NOTES.md` (coverage, capabilities, limitations)
  - `Z3_FUNCTION_HANDLING.md` (RecFuncDecl, composition, evaluation)
  - Keep `TRANSCENDENTAL_FUNCTIONS.md` (important standalone topic)
- [ ] Move to: `docs/solver-abstraction/z3/`

### 📝 TODO Tracking (4 docs - 28KB)

1. TODO_INVENTORY.md (8.7K) - Original audit
2. **TODO_INVENTORY_UPDATED.md** (6.4K) - Post-refactoring ✅ KEEP
3. TODO_57_IMPLEMENTATION_COMPLETE.md (7.0K) - Status update
4. REMAINING_TODOS_SUMMARY.md (6.0K) - Summary

**Action:**
- [ ] Keep only TODO_INVENTORY_UPDATED.md (most current)
- [ ] Archive others to session archive
- [ ] Consider moving updated inventory to project root or docs/

### 📚 Grammar v0.6 (7 docs - 48KB)

1. GRAMMAR_V06_IMPLEMENTATION_COMPLETE.md (8.2K) - Status
2. GRAMMAR_V06_RATIONALE.md (6.9K) - Design decisions
3. GRAMMAR_TODO_ANALYSIS.md (9.6K) - Parser gaps
4. FUNCTION_INTEGRATION_IMPLEMENTATION_PLAN.md (9.3K) - Integration plan
5. FUNCTIONS_AS_AXIOMS_SOUNDNESS.md (8.8K) - Soundness proof
6. TEX_FILES_GRAMMAR_V06_REVIEW.md (7.8K) - TeX compatibility
7. DOCUMENTATION_UPDATED_V06.md (4.7K) - Update log

**Action:**
- [ ] These are mostly session-specific - keep in session folder
- [ ] Create session README.md summarizing grammar v0.6 work
- [ ] Update main docs/grammar/ with final state

### 🗑️ Obsolete (1 doc)

1. COMMIT_PLAN.md (4.7K) - Planning doc, work is done

**Action:**
- [ ] Delete (no longer needed)

---

## 📋 Consolidation Tasks

### Task 1: Create Solver Abstraction Architecture Doc

**Target:** `docs/solver-abstraction/ARCHITECTURE.md`

**Combine:**
- SOLVER_ABSTRACTION_LAYER_DESIGN.md
- SOLVER_MCP_STYLE_CAPABILITIES.md  
- SOLVER_AST_RETURN_REQUIREMENT.md
- BUILTIN_FUNCTIONS_Z3_COVERAGE.md (coverage section)

**Result:** One comprehensive architecture document (~60K → 40K consolidated)

### Task 2: Create Z3 Implementation Notes

**Target:** `docs/solver-abstraction/z3/IMPLEMENTATION_NOTES.md`

**Combine:**
- Z3_CALCULUS_CAPABILITIES.md
- BUILTIN_FUNCTIONS_Z3_COVERAGE.md
- RUST_OPERATOR_OVERLOADING_IN_Z3.md
- Z3_MODEL_EVALUATION.md

**Result:** Z3-specific implementation details (~35K)

### Task 3: Create Z3 Function Handling Doc

**Target:** `docs/solver-abstraction/z3/FUNCTION_HANDLING.md`

**Combine:**
- Z3_RECFUNCDECL_SOLUTION.md
- Z3_FUNCTION_COMPOSITION_FIX.md
- Z3_FUNCTION_INTEGRATION_DESIGN.md
- Z3_QUANTIFIER_INSTANTIATION_ISSUE.md
- Z3_VS_EVALUATOR_FUNCTIONS.md

**Result:** How functions work in Z3 backend (~35K)

### Task 4: Keep Standalone

**Important standalone topics:**
- ✅ TRANSCENDENTAL_FUNCTIONS.md → `docs/type-system/TRANSCENDENTAL_FUNCTIONS.md`
- ✅ TODO_INVENTORY_UPDATED.md → Keep in session or move to root
- ✅ GRAMMAR_TODO_ANALYSIS.md → `docs/parser-implementation/TODO_ANALYSIS.md`

### Task 5: Session README

**Create:** `docs/session-2025-12-12/README.md`

**Summarize:**
- Grammar v0.6 complete (functions in structures)
- Z3 integration with functions
- Solver abstraction layer implemented (major refactoring!)
- 776 tests passing

---

## 🔗 Link Checking

After consolidation, run:

```bash
python3 scripts/check_markdown_links.py
```

Fix any broken links from:
- Moved documents
- Renamed files
- Deleted obsolete docs

---

## 📈 Landing Page Update

**Current:** 336+ Cloners  
**Check:** https://github.com/eatikrh/kleis/graphs/traffic

If count has increased, update `index.html` line 144.

---

## 🎯 Execution Order

1. ✅ Check GitHub for latest cloner count
2. ✅ Create consolidated architecture docs
3. ✅ Move standalone docs to permanent locations
4. ✅ Create session README
5. ✅ Delete obsolete docs
6. ✅ Run link checker and fix broken links
7. ✅ Update landing page if needed
8. ✅ Commit documentation organization

**Estimated time:** 1-2 hours

