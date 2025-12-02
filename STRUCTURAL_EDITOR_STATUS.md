# Structural Editor Implementation Status

**Date:** 2024-11-22  
**Status:** **Production Ready** (Integrated into Main UI)

---

## ✅ Achieved Milestones

### 1. Typst Library Integration (Performance)
- **Removed CLI dependency**: Compiles in-process using Typst crate.
- **Performance**: Render time < 20ms.
- **Coordinate System**: Solved alignment issues using deterministic layout and transform tracking.

### 2. Full Template Coverage
- **Implemented:** All 34 palette templates (Calculus, Linear Algebra, Physics, etc.).
- **Verified:** 100% pass rate in `test_comparison.rs`.
- **Robustness:** Layout grouping logic handles nested structures.
- **Visual Parity:** Verified against MathJax reference rendering.

### 3. Main UI Integration
- **Merged:** `structural_test.html` logic moved to `static/index.html`.
- **Toggle:** Seamless switching between Text (LaTeX) and Structural modes.
- **Palette:** All templates insert structured AST nodes.
- **Interaction:** Click-to-edit works with pixel-perfect overlays.
- **LaTeX Import:** `/api/parse` endpoint allows loading gallery examples into structural mode.

---

## Architecture Summary

### Backend (`src/bin/server.rs`)
- Endpoint `/api/render_typst` handles AST → SVG conversion.
- Returns:
  - `svg`: The rendered math.
  - `placeholders`: Positions of empty slots (from Typst metadata).
  - `argument_bounding_boxes`: Positions of filled slots (from Layout analysis).

### Layout Engine (`src/math_layout/typst_compiler.rs`)
- Uses `typst::World` to compile in-memory.
- Extracts layout tree (`Frame`).
- Normalizes coordinates to `(0,0)` origin.
- Groups text elements by Y-position to identify argument bounds.

### Frontend (`static/index.html`)
- Manages `currentAST` state.
- Renders SVG and overlays.
- Maps overlays to AST paths (`[0, 1, ...]`).
- Handles edits via simple prompt.

---

## Known Limitations

1. **Complex Vertical Nesting:**
   - For deeply nested fractions like `x / (y / (z / w))`, the "Group by Y" logic might split the denominator into multiple boxes visually.
   - **Impact:** Minor visual glitch (multiple green boxes), but editing still works if any box is clicked.

2. **Keyboard Navigation:**
   - Tab navigation works for placeholders.
   - Arrow key navigation through the tree is not yet implemented.

---

## Session Summary (2025-12-02): Matrix Marker Positioning Fix

### Problems Identified

1. **Matrix with indices** (e.g., `a_{11}`, `a_{12}`): Markers in bottom row were bunched up with wrong x-coordinates
2. **Kaluza-Klein matrix**: Missing marker for 4th cell (bottom-right Φ)
3. **3×3 matrices with values** (bmatrix, vmatrix, pmatrix): Cells 3-8 had completely scrambled positions due to Typst's column-major layout order

### Root Causes

1. **Sequential matching failure:** Text box pattern matching failed after first row because Typst outputs text in column-major or non-sequential order
2. **Premature loop termination:** Loop broke early when text boxes were exhausted, leaving arguments unprocessed
3. **Label index mismatch:** Attempted to map label indices (`sl0`, `sl1`, etc.) to AST argument indices, but Typst uses different ordering

### Implementation (Template-Agnostic)

**Changes to `src/math_layout/typst_compiler.rs`:**

1. ✅ **Spatial Matching Fallback:** Implemented `find_matching_slice_spatial()` that reorders text boxes by (y, x) reading order before pattern matching

2. ✅ **Removed Premature Break:** Eliminated early loop termination, allowing all arguments to be processed even when boxes run out

3. ✅ **Label Position Extraction:** Integrated `extract_positions_from_labels()` to extract Typst SVG label coordinates for all slots

4. ✅ **Spatial Label Override:** When top-level operation has exactly matching labeled positions, use them directly sorted by reading order, bypassing unreliable text box matching

### Test Results

| Template | Status | Notes |
|----------|--------|-------|
| Matrix 2×2 with indices (`a_{ij}`) | ✅ | All 4 markers correctly positioned |
| Kaluza-Klein 2×2 (complex cells) | ✅ | All 4 cells including bottom-right Φ |
| bmatrix 3×3 (values 1-9) | ✅ | All 9 cells in correct row-major grid |
| vmatrix 3×3 (determinant) | ✅ | All 9 cells in correct row-major grid |

### Final Solution: Pure UUID-Based Deterministic Positioning

After analysis revealed spatial sorting was still a heuristic, we implemented **true deterministic positioning**:

**The UUID System:**

1. **UUID Generation:** Every AST node (operations AND leaves) gets a unique UUID in `collect_argument_slots()`
2. **UUID Embedding:** Render to Typst with UUID labels: `#[#box[$content$]<id{uuid}>]`
3. **UUID Extraction:** Parse SVG `data-typst-label="id{uuid}"` attributes with exact Typst coordinates
4. **Direct Lookup:** Match node_id → UUID → Position (no pattern matching, no spatial sorting)
5. **Collision Safety:** Check 8-char truncated UUIDs for uniqueness, regenerate if collision detected

**Key Files Modified:**
- `src/bin/server.rs`: Build node_id→UUID map with collision detection
- `src/render.rs`: Add `render_expression_with_ids()` with UUID labeling for all nodes
- `src/math_layout/typst_compiler.rs`: Extract UUID positions, direct lookup, post-processing safety net

### Test Results: 100% Deterministic

**Matrix Templates (Verified):**
- ✅ matrix2x2, matrix3x3: **0 heuristics**
- ✅ vmatrix2x2, vmatrix3x3: **0 heuristics** 
- ✅ pmatrix2x2, pmatrix3x3: **0 heuristics**
- ✅ All complex cases (Kaluza-Klein, sqrt, fractions, indices): **Pure UUID matching**

**Heuristic Usage:**
- Matrix operations: **0** pattern fallbacks, **0** spatial fallbacks, **0** post-processing fixes
- Non-matrix nested ops: Some geometric estimation fallbacks (doesn't affect matrix cells)

### Status: **Production Ready - Zero Heuristics for Matrices**

All matrix cell positions determined by **direct UUID→Position lookup** from Typst's SVG output. No guessing, no proximity measurements, no spatial sorting—just deterministic UUID matching.

**Design Constraint Maintained:** Template-agnostic UUID system. Post-processing safety net exists but unused for correctly-labeled templates.

---

## Future Work (Documented 2025-12-02)

### 1. Make Everything Deterministic
**Current State:**
- ✅ Matrix operations: 100% UUID-based deterministic (all variants)
- ✅ Integrals: UUID-based for integral variables (dx, dy, dz) after removing skip_wrap_indices
- ✅ Literal chains: UUID-based after adding wrapping to children
- ⚠️  Function calls: Function name and deeply nested function calls use geometric fallback (~2-10 cases depending on complexity)
- ⚠️  Some nested operations: Geometric estimation used when UUID label extraction fails

**Known Limitation:**
Function calls in special Typst contexts (e.g., inside fractions, after operators) cannot be wrapped with `#[#box[$f$]<id>](args)` as it breaks Typst syntax. These use pattern matching fallback.

**Why It's Acceptable:**
With Option B filtering, function_call operations and their function names are **hidden from UI** (they're parent/child nodes). Only the function arguments are shown if they're terminal nodes. The fallback positions don't affect visible markers.

**Goal:** Investigate if function names can be wrapped without breaking Typst, or accept pattern matching for hidden elements.

### 2. Investigate Parent Operation Empty Markers
**Issue:** Operations create slots/markers that may appear empty or redundant in the UI.

**Examples:**
- A `sub` operation has a UUID and potentially a marker, but users interact with its children (base, subscript)
- Parent operations might create visual clutter with overlapping markers

**Real-World Example (Riemann tensor in matrix cell):**
```
matrix3x3[7] contains:
  riemann operation (gets UUID for cell 0.7)
    ├─ Object (empty symbol)
    ├─ Placeholder "upper"
    ├─ Placeholder "lower1"
    ├─ Placeholder "lower2"
    └─ Placeholder "lower3"
```

**The UX Question:**

Do we show interactive markers for:
- ~~Option A: 6 markers (1 for parent Riemann + 5 for its children)?~~
- **✅ Option B: Only the 5 child markers (edit components, not parent)** ← **IMPLEMENTED**
- ~~Option C: Context-aware (show parent when children filled, children when empty)?~~

**Implementation (2025-12-02):**
- ✅ **Backend**: Generates UUIDs for ALL nodes (operations + leaves) for deterministic positioning
- ✅ **Frontend**: Filters markers to show only leaf/terminal nodes:
  - Builds set of parent node IDs (nodes that have children in the tree)
  - Skips creating overlays for parent operations
  - Shows markers only for editable children (Const, Object, Placeholder)
  - Users edit components directly, not the containing operation

**Example (Matrix with indexed elements):**
```
matrix2x2 with a_{11}, a_{12}, a_{21}, a_{22}:
  Total slots: 13
  Hidden: 5 (matrix2x2 op + 4 sub ops)
  Shown: 8 (the "a" and "11", "12", "21", "22" values)
```

**Rationale:** Users want to edit "the subscript of x" not "the sub operation". Showing only terminal nodes provides cleaner, more intuitive interaction while backend keeps full structural information for positioning.

### 3. Other Improvements
- UI: Better scrollbar styling for structural editor
- UI: Visual feedback for deeply nested operations
- Performance: Cache UUID-based position lookups
- Testing: Comprehensive gallery test suite with heuristic detection

---

## Conclusion

The project has successfully transitioned from a proof-of-concept to a fully integrated feature in the main application. The structural editor allows users to build complex mathematical expressions by clicking templates and filling values, backed by the professional quality of the Typst rendering engine.
