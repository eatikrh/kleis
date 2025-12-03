# Documentation Cleanup Plan

**Problem:** 57 markdown files - many obsolete, redundant, or should be merged  
**Goal:** Clean, maintainable documentation structure  
**Date:** December 3, 2025

---

## Current State: 57 Files

### Root Directory: 27 files
### docs/: 29 files
### tests/golden/: 1 file

**Issue:** Too many session summaries, historical documents, and redundant guides.

---

## Cleanup Categories

### 🗑️ DELETE - Obsolete Session Summaries (13 files)

These are historical implementation notes, no longer relevant:

1. `ADR009_PHASE2_COMPLETE.md` - Old phase completion summary
2. `COMPLETE_UPDATE_SUMMARY.md` - Redundant with STRUCTURAL_EDITOR_STATUS.md
3. `MATRIX_CELL_FIX_SUMMARY.md` - Implementation detail, fixed
4. `MATRIX_MARKER_BUG.md` - Bug fixed, documented in STATUS
5. `MATRIX_NESTED_CONTENT_ISSUE.md` - Bug fixed
6. `SESSION_FINAL_SUMMARY.md` - Old session notes
7. `UUID_FIX_SUMMARY.md` - Implementation detail, fixed
8. `WYSIWYG_IMPLEMENTATION_COMPLETE.md` - Old milestone doc
9. `TYPST_LIBRARY_INTEGRATION_COMPLETE.md` - Old milestone doc
10. `MATHJAX_TO_TYPST_DECISION.md` - Decision made, move to ADR if needed
11. `CALIBRATION_FLAG_GUIDE.md` - Obsolete flag, no longer used
12. `DOCUMENTATION_AUDIT_NOV_2024.md` - Old audit (outdated)
13. `DOCUMENTATION_CLEANUP.md` - Previous cleanup attempt (ironic!)

**Rationale:** Implementation details from past work. Not useful going forward. Keep git history if needed.

### 📦 ARCHIVE - Historical Context (7 files)

Move to `docs/archive/` (keep for historical reference):

1. `docs/COORDINATE_SYSTEM_REVIEW.md` - Historical decision
2. `docs/SEMANTIC_BOUNDING_BOX_STRATEGY.md` - Strategy from old implementation
3. `docs/TYPST_SVG_ANALYSIS.md` - Analysis from early development
4. `docs/TEMPLATE_INFERENCE_IMPLEMENTATION.md` - Old implementation notes
5. `docs/LATEX_PARSING_ANALYSIS.md` - Historical parser analysis
6. `docs/template-implementation-strategy.md` - Old strategy doc
7. `docs/EQUATION_EDITOR_GUIDE.md` - Superseded by PALETTE_COMPLETE_GUIDE.md

**Rationale:** Historical context occasionally useful, but clutters main docs.

### 🔄 MERGE - Redundant Content (5 files)

Consolidate these into existing docs:

1. **COVERAGE_REPORT.md + FEATURE_COVERAGE.md**  
   → Merge into `TECHNICAL_REFERENCE.md` (Coverage section)

2. **PARSER_PARITY_UPDATE.md**  
   → Merge into `PARSER_TODO.md` (Status section)

3. **STRUCTURAL_EDITOR_TEST_RESULTS.md**  
   → Merge into `STRUCTURAL_EDITOR_STATUS.md` (Test Results section)

4. **NESTED_TEMPLATE_INSERTION.md**  
   → Merge into `PALETTE_COMPLETE_GUIDE.md` (Advanced Usage section)

5. **KLEIS_NOTEBOOK_VISION.md**  
   → Outdated, superseded by `docs/KLEIS_TYPE_UX.md`  
   → Delete or archive

**Rationale:** Content is useful but belongs in consolidated docs, not standalone files.

### ✅ KEEP - Current & Essential (32 files)

#### Core Documentation (8 files)
- ✅ `README.md` - Main entry point (just updated!)
- ✅ `STRUCTURAL_EDITOR_STATUS.md` - Current implementation status
- ✅ `PALETTE_COMPLETE_GUIDE.md` - User guide for templates
- ✅ `PALETTE_MISSING_SYMBOLS.md` - TODO for missing symbols
- ✅ `ARBITRARY_MATRIX_SOLUTION.md` - Matrix handling design
- ✅ `SERVER_README.md` - API documentation
- ✅ `PARSER_TODO.md` - Parser status
- ✅ `TECHNICAL_REFERENCE.md` - Technical details

#### Reference Guides (2 files)
- ✅ `TEST_GUIDE.md` - Testing procedures
- ✅ `tests/golden/README.md` - Golden test docs

#### Type System & Vision (5 files) **← NEW, CRITICAL**
- ✅ `docs/KLEIS_TYPE_SYSTEM.md` - Complete type system design
- ✅ `docs/KLEIS_TYPE_UX.md` - UX for type system
- ✅ `docs/KLEIS_EVALUATION_SYNTAX.md` - Evaluation design
- ✅ `docs/ARXIV_INTEGRATION_VISION.md` - Publishing integration
- ✅ `docs/UNIVERSAL_QUALITY_GATES.md` - Universal verification vision

#### Architecture Decision Records (9 files)
- ✅ `docs/adr-001-scalar-multiply.md`
- ✅ `docs/adr-002-eval-vs-simplify.md`
- ✅ `docs/adr-003-self-hosting.md`
- ✅ `docs/adr-004-input-visualization.md`
- ✅ `docs/adr-005-visual-authoring.md`
- ✅ `docs/adr-006-template-grammar-duality.md`
- ✅ `docs/adr-007-bootstrap-grammar.md`
- ✅ `docs/adr-008-bootstrap-grammar-boundary.md`
- ✅ `docs/adr-009-wysiwyg-structural-editor.md`

#### Theory & Vision (4 files)
- ✅ `docs/POT.md` - Projected Ontology Theory
- ✅ `docs/HONT.md` - Hilbert Ontology
- ✅ `docs/syntax.md` - Language syntax
- ✅ `docs/kleis_vision_executable_math.md` - Long-term vision

#### Other (4 files)
- ✅ `docs/README.md` - Docs directory overview
- ✅ `docs/kleis_grammar_v02.md` - Grammar specification

---

## Proposed New Structure

### Root Level (Keep Minimal - User Facing)

```
README.md                          ← Main entry (keep)
PALETTE_COMPLETE_GUIDE.md          ← User guide (keep)
PALETTE_MISSING_SYMBOLS.md         ← TODO (keep)
STRUCTURAL_EDITOR_STATUS.md        ← Implementation status (keep)
SERVER_README.md                   ← API docs (keep)
PARSER_TODO.md                     ← Parser status (keep, merge PARSER_PARITY_UPDATE)
TECHNICAL_REFERENCE.md             ← Tech details (keep, merge COVERAGE/FEATURE)
TEST_GUIDE.md                      ← Testing (keep)
ARBITRARY_MATRIX_SOLUTION.md       ← Matrix design (keep)
CHANGELOG.md                       ← NEW: Version history
```

### docs/ (Organized by Topic)

```
docs/
├── README.md                      ← Docs navigation
│
├── theory/                        ← NEW: Theory documents
│   ├── POT.md
│   ├── HONT.md
│   └── syntax.md
│
├── type-system/                   ← NEW: Type system docs
│   ├── KLEIS_TYPE_SYSTEM.md
│   ├── KLEIS_TYPE_UX.md
│   └── KLEIS_EVALUATION_SYNTAX.md
│
├── vision/                        ← NEW: Future vision
│   ├── ARXIV_INTEGRATION_VISION.md
│   ├── UNIVERSAL_QUALITY_GATES.md
│   ├── kleis_vision_executable_math.md
│   └── adr-005-visual-authoring.md  [move from adr/]
│
├── adr/                           ← Architecture decisions
│   ├── adr-001-scalar-multiply.md
│   ├── adr-002-eval-vs-simplify.md
│   ├── adr-003-self-hosting.md
│   ├── adr-004-input-visualization.md
│   ├── adr-006-template-grammar-duality.md
│   ├── adr-007-bootstrap-grammar.md
│   ├── adr-008-bootstrap-grammar-boundary.md
│   └── adr-009-wysiwyg-structural-editor.md
│
├── archive/                       ← NEW: Historical docs
│   ├── COORDINATE_SYSTEM_REVIEW.md
│   ├── SEMANTIC_BOUNDING_BOX_STRATEGY.md
│   ├── TYPST_SVG_ANALYSIS.md
│   ├── LATEX_PARSING_ANALYSIS.md
│   └── ...session summaries...
│
└── grammar/                       ← NEW: Grammar specs
    └── kleis_grammar_v02.md
```

---

## Cleanup Actions

### Phase 1: Delete Obsolete (Immediate)

```bash
# Delete 13 obsolete session summary files
rm ADR009_PHASE2_COMPLETE.md
rm COMPLETE_UPDATE_SUMMARY.md
rm MATRIX_CELL_FIX_SUMMARY.md
rm MATRIX_MARKER_BUG.md
rm MATRIX_NESTED_CONTENT_ISSUE.md
rm SESSION_FINAL_SUMMARY.md
rm UUID_FIX_SUMMARY.md
rm WYSIWYG_IMPLEMENTATION_COMPLETE.md
rm TYPST_LIBRARY_INTEGRATION_COMPLETE.md
rm MATHJAX_TO_TYPST_DECISION.md
rm CALIBRATION_FLAG_GUIDE.md
rm DOCUMENTATION_AUDIT_NOV_2024.md
rm DOCUMENTATION_CLEANUP.md
```

### Phase 2: Archive Historical (Immediate)

```bash
mkdir -p docs/archive

mv docs/COORDINATE_SYSTEM_REVIEW.md docs/archive/
mv docs/SEMANTIC_BOUNDING_BOX_STRATEGY.md docs/archive/
mv docs/TYPST_SVG_ANALYSIS.md docs/archive/
mv docs/TEMPLATE_INFERENCE_IMPLEMENTATION.md docs/archive/
mv docs/LATEX_PARSING_ANALYSIS.md docs/archive/
mv docs/template-implementation-strategy.md docs/archive/
mv docs/EQUATION_EDITOR_GUIDE.md docs/archive/
```

### Phase 3: Reorganize (Next)

```bash
# Create new structure
mkdir -p docs/theory
mkdir -p docs/type-system
mkdir -p docs/vision
mkdir -p docs/grammar

# Move theory
mv docs/POT.md docs/theory/
mv docs/HONT.md docs/theory/
mv docs/syntax.md docs/theory/

# Type system already in docs/, just note in README

# Move vision
mv docs/kleis_vision_executable_math.md docs/vision/
mv docs/adr-005-visual-authoring.md docs/vision/

# Move grammar
mv docs/kleis_grammar_v02.md docs/grammar/

# ADRs stay in docs/ (common convention)
```

### Phase 4: Merge Redundant (Next)

**Merge coverage reports into TECHNICAL_REFERENCE.md:**
- Add section "### Feature Coverage" from COVERAGE_REPORT.md
- Delete COVERAGE_REPORT.md and FEATURE_COVERAGE.md

**Merge parser updates into PARSER_TODO.md:**
- Add latest status from PARSER_PARITY_UPDATE.md
- Delete PARSER_PARITY_UPDATE.md

**Merge test results into STRUCTURAL_EDITOR_STATUS.md:**
- Add relevant test data from STRUCTURAL_EDITOR_TEST_RESULTS.md
- Delete STRUCTURAL_EDITOR_TEST_RESULTS.md

**Delete KLEIS_NOTEBOOK_VISION.md:**
- Superseded by KLEIS_TYPE_UX.md
- Outdated vision

**Merge NESTED_TEMPLATE_INSERTION.md into PALETTE_COMPLETE_GUIDE.md:**
- Add as "Advanced: Template Composition" section

---

## New Documentation Map (After Cleanup)

### For Users
```
README.md                    → Start here
PALETTE_COMPLETE_GUIDE.md    → How to use templates
PALETTE_MISSING_SYMBOLS.md   → Known gaps
SERVER_README.md             → API reference
```

### For Developers
```
STRUCTURAL_EDITOR_STATUS.md  → Implementation details
PARSER_TODO.md               → Parser status & TODO
TECHNICAL_REFERENCE.md       → Architecture & internals
TEST_GUIDE.md                → How to test
ARBITRARY_MATRIX_SOLUTION.md → Matrix handling design
```

### For Researchers/Academics
```
docs/type-system/
  - KLEIS_TYPE_SYSTEM.md     → Type theory design
  - KLEIS_TYPE_UX.md         → UX for types & context
  - KLEIS_EVALUATION_SYNTAX.md → Evaluation semantics

docs/vision/
  - ARXIV_INTEGRATION_VISION.md → Academic publishing
  - UNIVERSAL_QUALITY_GATES.md  → Broader applications
  - kleis_vision_executable_math.md → Long-term vision
  
docs/theory/
  - POT.md                   → Ontology theory
  - HONT.md                  → Hilbert ontology
  - syntax.md                → Language syntax

docs/adr/
  - adr-001 through adr-009  → Design decisions
```

### For Historical Reference
```
docs/archive/
  - Old analyses, strategies, implementation notes
  - Only if you want to dig into history
```

---

## Proposed README Structure

Update main README.md to clearly point to organized docs:

```markdown
## 📖 Documentation

### Getting Started
- [README](README.md) - You are here
- [Palette Guide](PALETTE_COMPLETE_GUIDE.md) - Using templates
- [Server API](SERVER_README.md) - API reference

### Type System & Verification
- [Type System Design](docs/type-system/KLEIS_TYPE_SYSTEM.md) - Algebraic foundations
- [Type System UX](docs/type-system/KLEIS_TYPE_UX.md) - Context, inference, prompts
- [Evaluation Syntax](docs/type-system/KLEIS_EVALUATION_SYNTAX.md) - Substitute, eval

### Vision & Future
- [arXiv Integration](docs/vision/ARXIV_INTEGRATION_VISION.md) - Academic publishing
- [Universal Verification](docs/vision/UNIVERSAL_QUALITY_GATES.md) - Beyond math
- [Executable Mathematics](docs/vision/kleis_vision_executable_math.md) - Long-term

### For Developers
- [Structural Editor Status](STRUCTURAL_EDITOR_STATUS.md) - Implementation
- [Technical Reference](TECHNICAL_REFERENCE.md) - Architecture
- [Parser Status](PARSER_TODO.md) - Parser TODO
- [Test Guide](TEST_GUIDE.md) - Testing
- [Architecture Decisions](docs/adr/) - ADR-001 through ADR-009

### Theory & Foundations
- [Projected Ontology Theory](docs/theory/POT.md)
- [Hilbert Ontology](docs/theory/HONT.md)
- [Language Syntax](docs/theory/syntax.md)
```

---

## Execution Plan

### Step 1: Delete Obsolete (Safe, Reversible via Git)
```bash
# Can always recover from git history if needed
git rm ADR009_PHASE2_COMPLETE.md
git rm COMPLETE_UPDATE_SUMMARY.md
# ... (all 13 files)
git commit -m "chore: remove obsolete session summary documents"
```

### Step 2: Create New Structure
```bash
mkdir -p docs/archive
mkdir -p docs/theory
mkdir -p docs/type-system
mkdir -p docs/vision
mkdir -p docs/grammar
```

### Step 3: Move Files
```bash
git mv docs/POT.md docs/theory/
git mv docs/HONT.md docs/theory/
# ... (organize as proposed)
git commit -m "chore: reorganize documentation structure"
```

### Step 4: Merge Redundant
```bash
# Manually merge content, then:
git rm COVERAGE_REPORT.md
git rm FEATURE_COVERAGE.md
# ... (after content merged)
git commit -m "chore: consolidate redundant documentation"
```

### Step 5: Update README
```bash
# Update doc links to new structure
vim README.md
git commit -m "docs: update README with clean doc structure"
```

---

## Before & After

### Before: 57 Files (Chaos)
```
./ADR009_PHASE2_COMPLETE.md
./CALIBRATION_FLAG_GUIDE.md
./COMPLETE_UPDATE_SUMMARY.md
./COVERAGE_REPORT.md
./DOCUMENTATION_AUDIT_NOV_2024.md
... (50 more files)
```
**Problem:** Hard to find anything, unclear what's current

### After: ~25 Files (Organized)
```
README.md
PALETTE_COMPLETE_GUIDE.md
STRUCTURAL_EDITOR_STATUS.md
...

docs/
├── type-system/ (3 files)
├── vision/ (4 files)
├── theory/ (3 files)
├── adr/ (9 files)
└── archive/ (historical)
```
**Result:** Clear navigation, easy to maintain

---

## Metrics

### Reduction
- Total files: 57 → ~25 (**-56%**)
- Root level: 27 → 10 (**-63%**)
- Active docs: Well-organized, findable

### Quality
- No redundant content
- No obsolete information
- Clear topic grouping
- Easy navigation

### Maintainability
- Fewer files to update
- Clear ownership (what doc covers what)
- Historical context preserved (archive)

---

## Risk Mitigation

**Risk:** Delete something important

**Mitigation:**
1. Everything in git history (recoverable)
2. Archive before delete (for uncertain cases)
3. Review plan with team before executing
4. Can revert any change

---

## Recommendation

**Execute cleanup in phases:**

**Immediate (Today):**
- Delete 13 obsolete session summaries
- Create new directory structure
- Move 7 files to archive

**This Week:**
- Merge 5 redundant docs
- Update README with new structure
- Update docs/README.md as navigation guide

**Next Week:**
- Review with fresh eyes
- Solicit feedback if team exists
- Finalize structure

**Result:** Professional, maintainable documentation that doesn't look like "AI slop"

---

**Status:** Plan ready for execution. Approve and we'll clean up!

