# Type System Documentation Consolidation Plan

**Date:** December 5, 2025  
**Issue:** Created 9 overlapping type system docs in docs/type-system/

---

## Current State

### New Files Created Today (9 files)

1. `GHC_TYPE_SYSTEM_STUDY_GUIDE.md` (11K) - How to study GHC source
2. `HASKELL_TYPE_SYSTEM_LESSONS.md` (19K) - What to adopt from Haskell
3. `TYPE_SYSTEM_SIMPLIFIED.md` (13K) - Simplified Hindley-Milner explanation
4. `HASKELL_TYPES_FOR_SYMBOLIC_MATH.md` (11K) - Why Haskell works for symbolic
5. `TYPES_FOR_SYMBOLIC_MATH.md` (12K) - Symbolic vs evaluated
6. `SYNTAX_COMPARISON_AND_PROPOSAL.md` (14K) - Kleis vs Haskell syntax
7. `USER_DEFINED_TYPES.md` (14K) - PurchaseOrder and non-math types
8. `INCREMENTAL_TYPE_CHECKING.md` (18K) - UX design with 5 states
9. `TYPE_INFERENCE_POC.md` (8.7K) - POC status report

**Total:** 130K of documentation with significant overlap!

### Overlap Analysis

**Theme: "Why Haskell works for symbolic math"**
- Files 2, 3, 4, 5 all explain this (overlap: 70%)

**Theme: "How to learn from Haskell/GHC"**
- Files 1, 2, 3 cover this (overlap: 50%)

**Theme: "UX and user-defined types"**
- Files 7, 8 cover this (overlap: 40%)

---

## Consolidation Plan

### Consolidate → `HASKELL_INTEGRATION.md` (Comprehensive)

**Merge:**
1. GHC_TYPE_SYSTEM_STUDY_GUIDE.md
2. HASKELL_TYPE_SYSTEM_LESSONS.md
3. TYPE_SYSTEM_SIMPLIFIED.md
4. HASKELL_TYPES_FOR_SYMBOLIC_MATH.md
5. TYPES_FOR_SYMBOLIC_MATH.md

**Into:** `docs/type-system/HASKELL_INTEGRATION.md` (~50K)

**Sections:**
1. Why Haskell's Type System Works for Symbolic Math
2. What to Adopt (type classes, inference, GADTs, etc.)
3. Hindley-Milner Simplified
4. How to Study GHC Source Code
5. Mapping Haskell Concepts to Kleis

### Keep → `SYNTAX_COMPARISON_AND_PROPOSAL.md`

**Keep as is** - This is a design decision document about syntax choices.

### Consolidate → `TYPE_CHECKING_UX.md`

**Merge:**
7. USER_DEFINED_TYPES.md
8. INCREMENTAL_TYPE_CHECKING.md

**Into:** `docs/type-system/TYPE_CHECKING_UX.md` (~30K)

**Sections:**
1. Non-Intrusive Type Checking (5 states)
2. Visual Feedback Design
3. Context Management
4. User-Defined Types (PurchaseOrder example)
5. API Endpoints
6. Frontend Integration

### Keep → `TYPE_INFERENCE_POC.md`

**Keep as is** - Status report for current POC implementation.

---

## Final Structure

### docs/type-system/ (After Consolidation)

**Core Design Docs (3 existing):**
- ✅ KLEIS_TYPE_SYSTEM.md (42K) - Full specification
- ✅ KLEIS_TYPE_UX.md (31K) - User experience design
- ✅ KLEIS_EVALUATION_SYNTAX.md (12K) - Evaluation semantics

**Haskell Integration (1 new consolidated):**
- 🆕 HASKELL_INTEGRATION.md (~50K) - Complete guide on adopting Haskell's approach

**Design Decisions (1 existing):**
- ✅ SYNTAX_COMPARISON_AND_PROPOSAL.md (14K) - Syntax design choices

**Implementation (2 files):**
- 🆕 TYPE_CHECKING_UX.md (~30K) - UX design + user-defined types
- ✅ TYPE_INFERENCE_POC.md (8.7K) - Current POC status

**Total:** 7 files (down from 12, eliminating 130K → 180K organized)

### Other Locations

**ADR:**
- ✅ docs/adr-014-hindley-milner-type-system.md - The architectural decision

**Overview:**
- ✅ docs/KLEIS_OVERVIEW.md - Comprehensive overview for NotebookLM
- ✅ docs/KLEIS_OVERVIEW.pdf - PDF version

---

## Execution

**Step 1:** Create HASKELL_INTEGRATION.md (merge 5 files)  
**Step 2:** Create TYPE_CHECKING_UX.md (merge 2 files)  
**Step 3:** Delete 7 source files  
**Step 4:** Update cross-references in remaining docs

**Result:** 12 → 7 files in docs/type-system/

---

## Reduction Summary

| Before | After | Reduction |
|--------|-------|-----------|
| 12 files | 7 files | -42% |
| 130K overlap | 180K organized | Better structure |

---

**Execute consolidation?** This will clean up the type system docs significantly.

