# Matrix Marker Positioning Bug

**Issue:** Edit markers in matrix with subscripted elements are not positioned correctly.

## Example: 2×2 Matrix with Subscripts

**LaTeX:**
```latex
\begin{bmatrix} a_{11} & a_{12} \\ a_{21} & a_{22} \end{bmatrix}
```

**Expected layout:**
```
[a₁₁  a₁₂]  ← Row 0
[a₂₁  a₂₂]  ← Row 1
```

## API Response Analysis

### Argument Slots (✅ CORRECT)
```json
8 slots with unique UUIDs:
- Slot 0: path=[0,0], hint="a"    (a₁₁ base)
- Slot 1: path=[0,1], hint="11"   (a₁₁ subscript)
- Slot 2: path=[1,0], hint="a"    (a₁₂ base)
- Slot 3: path=[1,1], hint="12"   (a₁₂ subscript)
- Slot 4: path=[2,0], hint="a"    (a₂₁ base)
- Slot 5: path=[2,1], hint="21"   (a₂₁ subscript)
- Slot 6: path=[3,0], hint="a"    (a₂₂ base)
- Slot 7: path=[3,1], hint="22"   (a₂₂ subscript)
```

### Bounding Boxes (❌ WRONG)
```json
11 bounding boxes (some duplicates, some wrong positions):

Top row (✅ correct):
- 0.0.0 (a base):      x=8.7,  y=-4.0    ← Left column, top
- 0.0.1 (11 subscript): x=21.4, y=1.9
- 0.1.0 (a base):      x=51.5, y=-4.0    ← Right column, top
- 0.1.1 (12 subscript): x=64.2, y=1.9

Bottom row (❌ WRONG):
- 0.2.0 (a base):      x=51.5, y=24.7    ← Should be x≈8.7 (left)!
- 0.2.1 (21 subscript): x=64.2, y=30.6   ← Should be x≈21.4!
- 0.3.0 (a base):      x=82.4, y=10.4    ← Should be x≈51.5, y≈24.7!
- 0.3.1 (22 subscript): MISSING!
```

## Root Cause

The **semantic bounding box extraction** in `typst_compiler.rs` is not correctly grouping matrix cells. It appears to be:
1. Correctly identifying top row elements
2. Mis-grouping bottom row elements (wrong x-coordinates)
3. Missing the last subscript entirely

The issue is in the function `extract_semantic_argument_boxes` which tries to recursively parse the Typst layout tree and assign bounding boxes to AST nodes.

## Impact

**Console warning:** `❌ Slot f14716...: No position found!`

This causes:
- 7 out of 8 markers to render
- Markers in wrong positions (bottom-left element shows on right)
- User confusion when clicking markers

## Solution Options

### Option 1: Fix Semantic Box Extraction (Hard)
- Debug `assign_boxes_recursive` in `typst_compiler.rs`
- Improve matrix cell detection logic
- Handle nested operations (sub inside matrix2x2)

### Option 2: Use Grid-based Position Inference (Medium)
- For matrix operations, calculate positions geometrically
- Matrix2x2: 2 columns, 2 rows → infer grid positions
- Use first few correct boxes to establish grid spacing

### Option 3: Hybrid Approach (Recommended)
- Use semantic boxes where available
- For missing/wrong boxes, infer from:
  - Matrix structure (2×2 = 4 cells in grid)
  - Known good positions (top-left as anchor)
  - Role metadata (base, subscript)

## Temporary Workaround

For now, matrices with subscripts will have some mispositioned markers. The UUID fix ensures they're at least unique and clickable, even if position is wrong.

## Next Steps

1. Improve semantic box grouping for nested operations
2. Add grid-based fallback for matrix operations
3. Test with matrix3x3 and other complex layouts

