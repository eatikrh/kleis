# Documentation Audit & Consolidation Plan

**Date:** 2024-12-05  
**Total .md files:** 80 (excluding target/)

---

## Current Structure Analysis

### ROOT DIRECTORY (19 files) - TOO MANY! ⚠️

#### Core Docs (Keep)
1. ✅ `README.md` (22K) - Main project documentation
2. ✅ `CHANGELOG.md` (3K) - Version history
3. ✅ `SERVER_README.md` (8.6K) - Server setup guide
4. ✅ `PARSER_TODO.md` (13K) - Active development TODO

#### Session Summaries (4 files → ARCHIVE)
5. 📦 `SESSION_2024-12-05_INTEGRAL_TRANSFORMS.md` (5.8K)
6. 📦 `SESSION_FINAL_SUMMARY_2024-12-03.md` (10K)
7. 📦 `SESSION_SUMMARY_2024-12-03.md` (7.9K)
8. 📦 `TODAY_WE_BUILT.md` (2.6K)

#### Status/Implementation Reports (3 files → ARCHIVE/DELETE)
9. 📦 `ARBITRARY_MATRIX_SOLUTION.md` (13K) - Implementation complete
10. 📦 `STRUCTURAL_EDITOR_STATUS.md` (12K) - Implementation complete  
11. 📦 `SYMBOL_INSERTION_FIXED.md` (4.9K) - Bug fix complete

#### Inline Editing (3 files → CONSOLIDATE)
12. 🔄 `INLINE_EDITING_IMPLEMENTATION_STATUS.md` (4.6K)
13. 🔄 `INLINE_EDITING_TEST_PLAN.md` (4.5K)
14. 🔄 `INLINE_EDITING_USER_GUIDE.md` (6.4K)
**→ Merge to:** `docs/INLINE_EDITING.md` (15K)

#### Palette (3 files → CONSOLIDATE)
15. 🔄 `PALETTE_COMPLETE_GUIDE.md` (8.4K) - Main guide
16. 🔄 `PALETTE_INTEGRATION_COMPLETE.md` (3.5K) - Status
17. 🔄 `PALETTE_MISSING_SYMBOLS.md` (6.8K) - TODO list
**→ Merge to:** `docs/PALETTE_GUIDE.md` (18K)

#### Technical References (2 files → MOVE TO docs/)
18. 📁 `TECHNICAL_REFERENCE.md` (8.7K)
19. 📁 `TEST_GUIDE.md` (19K)

---

## docs/ DIRECTORY (24 files at root level)

### ADRs (13 files) - GOOD ORGANIZATION ✅
- `adr-001-scalar-multiply.md` through `adr-013-paper-scope-hierarchy.md`
- These are properly organized and should stay

### Reference Docs (3 files)
1. ✅ `KLEIS_OVERVIEW.md` (14K) - **NEW COMPREHENSIVE DOC**
2. ✅ `COMPLETE_TEMPLATE_REFERENCE.md` (6.6K) - Template catalog
3. 🔄 `INTEGRAL_TRANSFORMS_REFERENCE.md` (14K) - Detailed reference

### Quick Guides (2 files → CONSOLIDATE)
4. 🔄 `INTEGRAL_TRANSFORMS_QUICKSTART.md` (1.8K)
5. 🔄 `INTEGRAL_TRANSFORMS_POT.md` (9.3K)
**→ Merge with INTEGRAL_TRANSFORMS_REFERENCE.md to:** `docs/INTEGRAL_TRANSFORMS.md` (25K)

### Implementation Guides (2 files)
6. ✅ `TYPST_TEXT_IN_MATH.md` (6.7K) - Keep
7. ✅ `GITHUB_ACTIONS_SETUP.md` (4.3K) - Keep

### Status/Complete (1 file → ARCHIVE)
8. 📦 `BRACKET_TEMPLATES_ADDED.md` (4.8K)

### Design Docs (3 files)
9. ✅ `HARD_PROBLEMS_AHEAD.md` (11K) - Design challenges
10. ✅ `INLINE_EDITING_BUTTON_BEHAVIOR.md` (17K) - Design spec
11. ✅ `PALETTE_ICON_STRATEGY.md` (7.3K) - Design strategy

### Future Vision (1 file)
12. ✅ `DLMF_INTEGRATION.md` (5K) - Future integration

### Meta
13. ✅ `README.md` (4.8K) - docs/ index

---

## docs/ SUBDIRECTORIES - WELL ORGANIZED ✅

### docs/type-system/ (3 files)
- ✅ KLEIS_TYPE_SYSTEM.md
- ✅ KLEIS_TYPE_UX.md
- ✅ KLEIS_EVALUATION_SYNTAX.md

### docs/theory/ (3 files)
- ✅ POT.md
- ✅ HONT.md
- ✅ syntax.md

### docs/vision/ (5 files)
- ✅ kleis_vision_executable_math.md
- ✅ ARXIV_INTEGRATION_VISION.md
- ✅ FUTURE_IDE_INTEGRATION.md
- ✅ UNIVERSAL_QUALITY_GATES.md
- ✅ adr-005-visual-authoring.md

### docs/grammar/ (1 file)
- ✅ kleis_grammar_v02.md

### docs/archive/ (11 files)
- ✅ Already properly archived

---

## FUNDING/ (3 files) - KEEP ✅
- FUNDING_STRATEGY.md
- GRANT_PROPOSAL_NSF.md
- PITCH_DECK.md

---

## CONSOLIDATION PLAN

### Phase 1: Archive Completed Status Docs (7 files)
**Move to:** `docs/archive/session-2024-12-03/`
```
SESSION_FINAL_SUMMARY_2024-12-03.md
SESSION_SUMMARY_2024-12-03.md
TODAY_WE_BUILT.md
ARBITRARY_MATRIX_SOLUTION.md
STRUCTURAL_EDITOR_STATUS.md
SYMBOL_INSERTION_FIXED.md
BRACKET_TEMPLATES_ADDED.md
```

**Move to:** `docs/archive/session-2024-12-05/`
```
SESSION_2024-12-05_INTEGRAL_TRANSFORMS.md (already there)
```

### Phase 2: Consolidate Inline Editing (3 → 1)
**Create:** `docs/guides/INLINE_EDITING.md`
```
Combine:
- INLINE_EDITING_IMPLEMENTATION_STATUS.md (status section)
- INLINE_EDITING_TEST_PLAN.md (testing section)
- INLINE_EDITING_USER_GUIDE.md (usage section)
```

### Phase 3: Consolidate Palette (3 → 1)
**Create:** `docs/guides/PALETTE_GUIDE.md`
```
Combine:
- PALETTE_COMPLETE_GUIDE.md (main content)
- PALETTE_INTEGRATION_COMPLETE.md (implementation notes)
- PALETTE_MISSING_SYMBOLS.md (future work section)
```

### Phase 4: Consolidate Integral Transforms (3 → 1)
**Create:** `docs/guides/INTEGRAL_TRANSFORMS.md`
```
Combine:
- INTEGRAL_TRANSFORMS_REFERENCE.md (main reference)
- INTEGRAL_TRANSFORMS_QUICKSTART.md (quick start section)
- INTEGRAL_TRANSFORMS_POT.md (motivation/context section)
```

### Phase 5: Move Technical Refs to docs/ (2 files)
```
TECHNICAL_REFERENCE.md → docs/TECHNICAL_REFERENCE.md
TEST_GUIDE.md → docs/guides/TEST_GUIDE.md
```

---

## FINAL STRUCTURE (After Consolidation)

### ROOT (4 files only)
```
README.md
CHANGELOG.md
SERVER_README.md
PARSER_TODO.md
```

### docs/ (well-organized)
```
docs/
├── README.md (index)
├── KLEIS_OVERVIEW.md (comprehensive)
├── TECHNICAL_REFERENCE.md
├── COMPLETE_TEMPLATE_REFERENCE.md
├── TYPST_TEXT_IN_MATH.md
├── GITHUB_ACTIONS_SETUP.md
├── HARD_PROBLEMS_AHEAD.md
├── INLINE_EDITING_BUTTON_BEHAVIOR.md
├── PALETTE_ICON_STRATEGY.md
├── DLMF_INTEGRATION.md
├── adr-001-*.md through adr-013-*.md
├── guides/
│   ├── INLINE_EDITING.md (NEW - consolidated)
│   ├── PALETTE_GUIDE.md (NEW - consolidated)
│   ├── INTEGRAL_TRANSFORMS.md (NEW - consolidated)
│   └── TEST_GUIDE.md (moved)
├── type-system/
│   ├── KLEIS_TYPE_SYSTEM.md
│   ├── KLEIS_TYPE_UX.md
│   └── KLEIS_EVALUATION_SYNTAX.md
├── theory/
│   ├── POT.md
│   ├── HONT.md
│   └── syntax.md
├── vision/
│   ├── kleis_vision_executable_math.md
│   ├── ARXIV_INTEGRATION_VISION.md
│   ├── FUTURE_IDE_INTEGRATION.md
│   ├── UNIVERSAL_QUALITY_GATES.md
│   └── adr-005-visual-authoring.md
├── grammar/
│   └── kleis_grammar_v02.md
└── archive/
    ├── session-2024-12-03/ (7 files)
    └── session-2024-12-05/ (11 files)
```

### funding/ (unchanged)
```
FUNDING_STRATEGY.md
GRANT_PROPOSAL_NSF.md
PITCH_DECK.md
```

---

## BENEFITS

### Before
- ❌ 19 files in root (cluttered)
- ❌ Duplicate/overlapping content
- ❌ Hard to find current docs
- ❌ Mix of active docs and historical status

### After
- ✅ 4 files in root (clean)
- ✅ Consolidated guides
- ✅ Clear organization by category
- ✅ Historical docs properly archived

---

## CONSOLIDATION REDUCTION

| Category | Before | After | Reduction |
|----------|--------|-------|-----------|
| Root .md files | 19 | 4 | **-79%** |
| Inline Editing docs | 3 | 1 | -67% |
| Palette docs | 3 | 1 | -67% |
| Integral Transforms | 3 | 1 | -67% |
| **Total reduction** | **28 files** | **10 files** | **-64%** |

---

## RECOMMENDATION

**Execute all 5 phases** to achieve:
1. Clean root directory (4 files only)
2. Well-organized docs/ with guides/ subdirectory
3. Proper archival of historical docs
4. Consolidated user guides
5. 64% reduction in documentation sprawl

**Time estimate:** 20-30 minutes
**Risk:** Low (all consolidation, no deletion of content)
**Benefit:** Much easier navigation and maintenance

