# Matrix Nested Content - Coordinate Issue Analysis

**Date:** November 24, 2024  
**Issue:** Edit markers misaligned when matrix cells contain nested operations  
**Status:** 🔍 Root cause identified - Fix in progress  
**Priority:** High - Blocks real-world usage

---

## The Problem

**Simple matrix (all placeholders):** ✅ Works perfectly
```
□ □ □
□ □ □
□ □ □
```

**Matrix with nested content:** ❌ Markers misaligned
```
⟨□|□⟩ ⟨□|□⟩ □
  □      □    □
  □      □    □
```

---

## Test Case Data

### Matrix with Inner Product in Cell a11

**AST Structure:**
```
matrix3x3(
  inner(Placeholder{id:9}, Placeholder{id:10}),  ← Nested in cell a11
  Placeholder{id:1},  ← Cell a12
  Placeholder{id:2},  ← Cell a13
  ... 6 more cells
)
```

**Marker Placement Data:**

**Placeholders (square glyph positions):**
```
ID 9:  (x=9.34,   y=18.00)  ← Bra in inner product
ID 10: (x=50.62,  y=18.00)  ← Ket in inner product
ID 1:  (x=29.98,  y=46.75)  ← Cell a12
ID 2:  (x=29.98,  y=75.46)  ← Cell a13
ID 3:  (x=90.62,  y=18.00)  ← Cell a21
...
```
✅ These look reasonable (positive coordinates)

**Semantic Bounding Boxes:**
```
Arg 0 [0.0]:     (x=12.01,  y=-4.00)   ← NEGATIVE Y! (inner product operation)
Arg 0 [0.0.0]:   (x=21.34,  y=-4.00)   ← NEGATIVE Y! (bra placeholder)
Arg 1 [0.0.1]:   (x=62.62,  y=-4.00)   ← NEGATIVE Y! (ket placeholder)
Arg 1 [0.1]:     (x=41.98,  y=24.75)   ← OK (cell a12)
Arg 2 [0.2]:     (x=41.98,  y=53.46)   ← OK (cell a13)
...
```
❌ Nested placeholders have **negative Y coordinates**

**Argument Slots:**
```
Slot 9:  path=[0,0],  hint="bra"  ← Path to nested placeholder
Slot 10: path=[0,1],  hint="ket"
Slot 1:  path=[1],    hint="a12"  ← Path to matrix cell
...
```

---

## Root Cause Analysis

### The Coordinate Mismatch

**With semantic-first enabled:**
1. Frontend looks for semantic box with node ID `"0.0.0"` (for slot 9, bra)
2. Backend returns semantic box `[0.0.0]` with `y=-4.00`
3. Frontend uses this negative Y coordinate
4. Marker appears off-screen or misaligned

**Why negative Y?**

The backend's **calibration offset** is calculated incorrectly for nested matrix structures:

```rust
// In typst_compiler.rs line 452-482
// Uses first placeholder to calibrate
let first_ph = placeholder_positions.first()  // ID 9 at (9.34, 18.00)
let match_box = candidates.first()            // From layout tree
offset_y = first_ph.y - match_box.y           // Calculates wrong offset
```

When the first placeholder is **inside a nested operation inside a matrix cell**, the layout tree structure is complex, and the matching logic fails.

**Result:** Wrong calibration offset → All semantic boxes get corrupted Y coordinates.

---

## Why Simple Matrix Works

**Simple matrix (all placeholders):**
- All 9 placeholders are direct children of matrix operation
- No nesting beyond matrix structure
- Layout tree is straightforward
- Calibration works correctly
- Semantic boxes have correct coordinates

**Complex matrix (nested content):**
- Some placeholders are nested 2-3 levels deep
- Layout tree has complex nested groups
- Calibration matching fails
- Semantic boxes get wrong coordinates

---

## Why Placeholder Positions Are Better Here

**For the nested case:**
- Placeholder positions: `(9.34, 18.00)`, `(50.62, 18.00)` ✅ Correct
- Semantic boxes: `y=-4.00` ❌ Wrong

**But with semantic-first, we use semantic boxes** (which are wrong), instead of placeholder positions (which are correct).

---

## The Fix Strategy

### Option 1: Detect and Skip Bad Semantic Boxes

**Check if semantic box has negative coordinates:**
```javascript
if (bbox && bbox.x >= 0 && bbox.y >= 0) {
    // Use semantic box (good coordinates)
} else {
    // Fall back to placeholder position
}
```

**Pros:** Simple, safe  
**Cons:** Band-aid, doesn't fix root cause

### Option 2: Fix Backend Calibration

**Improve calibration matching in `typst_compiler.rs`:**
- Better heuristic for finding matching layout box
- Account for nested structures
- Handle matrix cells specially

**Pros:** Fixes root cause  
**Cons:** Complex, risky, affects all templates

### Option 3: Hybrid Approach for Matrices

**Use placeholder-first for matrices specifically:**
```javascript
if (isMatrixOperation(currentAST)) {
    // Use placeholder positions for matrices (more reliable)
} else {
    // Use semantic boxes for other operations
}
```

**Pros:** Targeted fix, low risk  
**Cons:** Special case logic

### Option 4: Fix Path-to-NodeID Matching

**The slots have paths like `[0,0]` but should they be `[0,0,0]` for nested?**

Looking at the data:
- Slot 9 has path `[0,0]` but semantic box is `[0.0.0]`
- The path is **missing a level**!

**If we fix the path generation:**
- Nested placeholders get correct paths
- Matching works
- Semantic boxes are used correctly

---

## My Recommendation

**Start with Option 1 (Detect Bad Coordinates)** - Quick, safe, effective:

```javascript
// In overlay positioning logic
if (bbox) {
    // Check if coordinates are valid
    if (bbox.x < 0 || bbox.y < 0) {
        console.warn(`⚠️ Semantic box has negative coords (${bbox.x}, ${bbox.y}), using placeholder instead`);
        // Fall through to placeholder position
    } else {
        // Use semantic box (good coordinates)
        rectX = bbox.x - 3;
        rectY = bbox.y - 3;
        // ...
        foundPosition = true;
    }
}

if (!foundPosition) {
    // Try placeholder position
    const ph = ...
}
```

**This will:**
- Fix the immediate issue (negative Y coordinates)
- Use placeholder positions for problematic cases
- Keep semantic boxes for cases where they work
- Low risk, easy to test

**Then investigate Option 4** (path generation) as the deeper fix.

**Want me to implement Option 1 first?** 🔧

