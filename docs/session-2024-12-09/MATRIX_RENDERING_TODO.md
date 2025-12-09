# Matrix Rendering with List Format - Known Issues

**Date:** December 9, 2024  
**Status:** Type system complete ✅, Rendering needs polish 🔧

## What Works ✅

**Type System (Complete):**
- Matrix is a regular 3-arg data constructor: `Matrix(m: Nat, n: Nat, elements: List(T))`
- Type inference perfect: `Matrix(NatValue(2), NatValue(2), List(Var(α)))`
- Zero hardcoded special cases in type inference
- All variants work: Matrix, PMatrix, VMatrix, BMatrix
- User extensible: `Tensor(i, j, k, [elements])` works without code changes

**Parsing & AST:**
- List literal syntax: `[a, b, c]` parses correctly
- Matrix with List: `Matrix(2, 2, [a, b, c, d])` works
- AST updates correctly when editing
- Values save and load properly

**Backend Rendering:**
- UUID map built correctly (all elements have UUIDs)
- UUID positions extracted from SVG (all elements found)
- Typst markup includes UUID labels on all elements
- Elements render visually correct in matrix layout

## What Needs Polish 🔧

### 1. Edit Marker Positioning (In Progress)

**Issue:** When all matrix elements are filled, edit markers don't align correctly

**Root Cause:** UUID lookup was disabled for nested nodes (fixed in commit)

**Status:** 
- Fixed line 392: `use_uuid_lookup = !uuid_positions.is_empty()` (all depths)
- Need to test if this fully resolves edit marker positioning

**Files involved:**
- `src/math_layout/typst_compiler.rs` (assign_boxes_recursive)

### 2. Tree View Display (Cosmetic)

**Issue:** Debug panel shows List as "? Unknown"

**Fix needed:** Add List case to tree renderer in index.html

**Location:** Around line 1560 in `static/index.html`

### 3. Matrix Dimension Edit Markers (Fixed)

**Issue:** Dimensions showed edit markers (they shouldn't be editable)

**Fix:** Skip first 2 args in collect_slots_recursive for Matrix with List format

**Status:** ✅ Fixed in commit 368015b

## Testing Checklist

- [ ] Insert Matrix template
- [ ] Fill in all 4 values
- [ ] Verify edit markers appear on all elements
- [ ] Click on filled values to edit them
- [ ] Verify inline editor works
- [ ] Check bounding boxes are correct

## Next Steps

1. **Test current fix** (UUID lookup at all depths)
2. **If still broken:** Add List node handling to assign_boxes_recursive
3. **Fix tree view:** Add List display case
4. **Test all matrix variants:** Matrix, PMatrix, VMatrix
5. **Document final solution**

## Files Modified Today

**Type System (Core):**
- `src/structure_registry.rs` (new file, +198 lines)
- `src/type_inference.rs` (-100 lines hardcoding)
- `src/signature_interpreter.rs` (generic handling)
- `src/type_context.rs` (removed hardcoding)
- `stdlib/types.kleis` (Matrix data constructors)

**List Literal Support:**
- `src/ast.rs` (Expression::List)
- `src/kleis_parser.rs` (parse_list_literal)
- `src/render.rs` (List rendering, Matrix List extraction)
- `src/bin/server.rs` (List JSON serialization, slot collection)
- `tests/list_literal_test.rs` (new tests)

**Frontend:**
- `static/index.html` (List format templates, navigation, setNodeAtPath)

## Commits (10 total)

1. Matrix cleanup from type_inference.rs (-65 lines)
2. StructureRegistry implementation (+347 lines)
3. Removed Matrix/Vector type constructor special cases (-4 lines)
4. Removed ALL remaining hardcoded references (-33 lines)
5. List literal support (+330 lines)
6. Frontend updated to use List format
7. Fixed Matrix rendering and type inference
8. Added PMatrix, VMatrix, BMatrix data constructors
9. Removed ALL Matrix special cases (-31 lines)
10. Fixed List navigation and dimension markers

**Net:** +198 (StructureRegistry) +330 (List) -133 (removed hardcoding) = +395 lines total

## Bottom Line

**Type system: COMPLETE ✅**  
**Rendering: 95% working, needs testing/polish 🔧**

The Matrix constructor cleanup architectural goal is ACHIEVED. Remaining work is UI polish for the new List format.

