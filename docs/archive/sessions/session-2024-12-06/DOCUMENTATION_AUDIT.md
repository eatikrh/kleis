# Documentation Audit - December 6, 2024

**Total docs:** 115 markdown files  
**Non-archived:** 76 files  
**Goal:** Eliminate duplicates, obsolete content, improve organization

---

## Issues Found

### 1. Root-Level Clutter (Should only have ADRs + main guides)

**Move to guides/ or archive:**
- `INLINE_EDITING_BUTTON_BEHAVIOR.md` → guides/ (technical guide)
- `PALETTE_ICON_STRATEGY.md` → guides/ (design guide)
- `TYPST_TEXT_IN_MATH.md` → parser-implementation/ (technical)
- `DOCUMENTATION_ORGANIZATION.md` → DELETE (now in .cursorrules)

**Keep in root:**
- All ADR files (adr-*.md) ✅
- `README.md` (navigation) ✅
- `KLEIS_OVERVIEW.md` (main reference) ✅
- `TECHNICAL_REFERENCE.md` (main reference) ✅
- `COMPLETE_TEMPLATE_REFERENCE.md` (main reference) ✅

### 2. Type System Redundancy

**type-system/ has 13 files - many overlapping:**
- `COMPLETE_ROADMAP.md`
- `TYPE_CHECKING_NEXT_STEPS.md`
- `UPDATED_ROADMAP_ADR016.md`

**These are all roadmaps! Likely redundant.**

**Also:**
- `TYPE_CHECKING_UX.md` vs `KLEIS_TYPE_UX.md` (may overlap)
- `TYPE_INFERENCE_POC.md` vs `HINDLEY_MILNER_STATUS.md` (may overlap)

**Recommendation:**
- Consolidate roadmaps into one: `TYPE_SYSTEM_ROADMAP.md`
- Keep: `KLEIS_TYPE_SYSTEM.md` (main design)
- Keep: `TYPE_INFERENCE_POC.md` (working POC)
- Archive or delete duplicates

### 3. Parser Documentation Scattered

**parser-implementation/ has 4 files:**
- `KLEIS_AST_GRAMMAR_COMPARISON.md`
- `PARSER_GRAMMAR_COMPATIBILITY.md`
- `PARSER_IMPLEMENTATION_SUMMARY.md`
- `PARSER_RENDERER_COMPATIBILITY.md`

**Plus in root:**
- `../PARSER_TODO.md` (main status doc)

**Recommendation:**
- Keep `PARSER_TODO.md` in root (main reference)
- Keep parser-implementation/ as technical deep-dives
- Add `TYPST_TEXT_IN_MATH.md` to this folder

### 4. Grammar Files Duplication

**grammar/ folder:**
- `kleis_grammar_v02.md` (old version)
- `kleis_grammar_v03.md` (current version)

**Plus in root:**
- `../kleis_grammar_v02.ebnf` (old version)
- `../Kleis.g4` (old version?)
- `../kleis.pest` (old version?)

**Recommendation:**
- Archive v02 files
- Keep only v03 as active

### 5. Vision Documents Well-Organized ✅

**vision/ has 8 files - all unique and valuable:**
- adr-005, executable math, LLMs writing Kleis, etc.
- **No duplicates found!**

---

## Proposed Organization

### Root docs/ (ADRs + Main References Only)

**Keep:**
- All adr-*.md files (17 ADRs)
- README.md
- KLEIS_OVERVIEW.md
- TECHNICAL_REFERENCE.md
- COMPLETE_TEMPLATE_REFERENCE.md
- HARD_PROBLEMS_AHEAD.md (forward-looking)
- DLMF_INTEGRATION.md (future integration)
- GITHUB_ACTIONS_SETUP.md (infrastructure)

**Move:**
- INLINE_EDITING_BUTTON_BEHAVIOR.md → guides/
- PALETTE_ICON_STRATEGY.md → guides/
- TYPST_TEXT_IN_MATH.md → parser-implementation/

**Delete:**
- DOCUMENTATION_ORGANIZATION.md (now in .cursorrules)

### grammar/ (Current Version Only)

**Keep:**
- kleis_grammar_v03.ebnf ✅
- kleis_grammar_v03.md ✅
- Kleis_v03.g4 ✅

**Archive:**
- Move v02 files to archive/grammar/

### guides/ (User-Facing Guides)

**Current:**
- INLINE_EDITING.md
- INTEGRAL_TRANSFORMS.md
- PALETTE_GUIDE.md
- TEST_GUIDE.md

**Add:**
- INLINE_EDITING_BUTTON_BEHAVIOR.md (from root)
- PALETTE_ICON_STRATEGY.md (from root)
- MATRIX_BUILDER_GUIDE.md (new - user guide for matrix builder)

### parser-implementation/ (Technical Deep Dives)

**Keep all 4 current files** ✅

**Add:**
- TYPST_TEXT_IN_MATH.md (from root)

### type-system/ (Consolidate!)

**Master documents (keep):**
- KLEIS_TYPE_SYSTEM.md (main design)
- TYPE_INFERENCE_POC.md (working code)
- HINDLEY_MILNER_STATUS.md (current status)
- DEPENDENT_TYPES_EXAMPLE.md (examples)
- examples/context_bootstrap_demo.md

**Consolidate these 3 roadmaps into TYPE_SYSTEM_ROADMAP.md:**
- COMPLETE_ROADMAP.md
- TYPE_CHECKING_NEXT_STEPS.md
- UPDATED_ROADMAP_ADR016.md

**Merge UX docs into KLEIS_TYPE_UX.md:**
- TYPE_CHECKING_UX.md (merge)
- Keep KLEIS_TYPE_UX.md

**Archive or delete:**
- CONTEXT_AND_OPERATIONS.md (if covered in main docs)
- HASKELL_INTEGRATION.md (future/speculative)
- KLEIS_EVALUATION_SYNTAX.md (if obsolete)
- SYNTAX_COMPARISON_AND_PROPOSAL.md (if obsolete)

### notation/ (Well-Organized) ✅

**3 files - all unique and valuable**
- No changes needed

### theory/ (Well-Organized) ✅

**3 files - all unique**
- No changes needed

### vision/ (Well-Organized) ✅

**8 files - all unique and valuable**
- No changes needed

### session-2024-12-06/ (Already Cleaned) ✅

**15 files - cleaned up today**
- No further changes needed

---

## Action Items

### Phase 1: Root Cleanup (Quick Wins)

- [ ] Delete DOCUMENTATION_ORGANIZATION.md
- [ ] Move INLINE_EDITING_BUTTON_BEHAVIOR.md → guides/
- [ ] Move PALETTE_ICON_STRATEGY.md → guides/
- [ ] Move TYPST_TEXT_IN_MATH.md → parser-implementation/

### Phase 2: Grammar Archive

- [ ] Create archive/grammar/
- [ ] Move kleis_grammar_v02.ebnf → archive/grammar/
- [ ] Move ../kleis_grammar_v02.md → archive/grammar/
- [ ] Move ../Kleis.g4 → archive/grammar/ (if v02)
- [ ] Move ../kleis.pest → archive/grammar/ (if unused)

### Phase 3: Type System Consolidation

- [ ] Review and consolidate 3 roadmap files
- [ ] Merge TYPE_CHECKING_UX.md into KLEIS_TYPE_UX.md
- [ ] Archive speculative/old files
- [ ] Create single TYPE_SYSTEM_ROADMAP.md

### Phase 4: Create Missing Guides

- [ ] MATRIX_BUILDER_GUIDE.md (user guide)
- [ ] STRUCTURE_DEFINITION_GUIDE.md (how to write structures)

---

## Estimated Impact

**Current:** 115 files (76 active)  
**After cleanup:** ~55-60 active files  
**Reduction:** ~20-25% fewer files  
**Benefit:** Much easier to navigate, no duplicates

---

**Recommendation:** Start with Phase 1 (quick wins), then Phase 2 (grammar). 
Phase 3 requires careful review of type system docs.


