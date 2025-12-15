# Matrix Migration Issues - December 11, 2025

**Context:** After migrating from `Matrix(m,n,a,b,c,d)` to `Matrix(m,n,[a,b,c,d])` format

---

## Issue: One Displaced Edit Marker on Palette Matrix Buttons

**Status:** 🐛 Known Issue  
**Severity:** Minor - Matrix Builder works perfectly  
**Affected:** Palette button insertion only

### Symptoms

When inserting a matrix via **palette button** (e.g., "matrix2x2", "pmatrix2x2"):
- ❌ One edit marker is displaced
- ❌ Bounding box for one argument points to wrong location
- ❌ Caused by duplicate placeholder ID (ID 0 appears twice)

**Example from debug output:**
```
Placeholders:
  ID 0: (x=0.00, y=4.97)    // "left" from equals
  ID 0: (x=62.01, y=-7.36)  // "a11" from matrix - DUPLICATE!
  ID 1: (x=87.08, y=-7.36)  // a12 ✅
  ID 2: (x=62.01, y=14.17)  // a21 ✅
  ID 3: (x=87.08, y=14.17)  // a22 ✅

Argument Bounding Boxes:
  Arg 2 [0.1.2]: (x=0.00, y=4.97)  // ❌ Points to first ID 0, not matrix element
```

### What Works Perfectly

**Matrix Builder:** ✅ No issues
```
IDs: 19, 20, 21, 22 (all unique)
Positions: All correct
Format: Matrix(2, 2, [elements])
```

Matrix Builder uses the same `Matrix(m,n,[elements])` format and works flawlessly.

---

## Root Cause Analysis

### NOT Caused By Migration

The migration to `Matrix(m,n,[elements])` format is correct:
- ✅ AST structure is correct
- ✅ Type inference works
- ✅ Rendering works
- ✅ Matrix Builder proves the format is sound

### Actual Cause: Palette Placeholder ID Coordination

**The real issue:**
When palette button inserts a template into an existing expression, it doesn't coordinate placeholder IDs.

**What happens:**
1. User creates `equals` template → placeholder ID 0 ("left")
2. User clicks in right side
3. User clicks "matrix2x2" palette button
4. Template calls `template_matrix_2x2()` which uses static counter
5. Counter returns ID 0 (duplicate!) instead of continuing from 1

**Why Matrix Builder works:**
- Creates matrix in isolation (fresh ID sequence)
- Or properly coordinates with existing IDs
- Uses same `Matrix(m,n,[elements])` format

---

## Technical Details

### Template Code (Correct)
```rust
pub fn template_matrix_2x2() -> Expression {
    Expression::operation("Matrix", vec![
        Expression::Const("2"),
        Expression::Const("2"),
        Expression::List(vec![
            Expression::placeholder(next_id(), "a11"),  // Gets next ID
            Expression::placeholder(next_id(), "a12"),
            Expression::placeholder(next_id(), "a21"),
            Expression::placeholder(next_id(), "a22"),
        ]),
    ])
}
```

### The Issue
`next_id()` uses a static counter that doesn't coordinate with the equation editor's existing placeholder IDs.

---

## Impact Assessment

**Severity:** Minor
- Matrix Builder (primary workflow) works perfectly ✅
- Only affects palette button insertion into existing expressions
- User can still edit matrix, just one marker slightly off
- Does not affect functionality, only visual feedback

**Affected Operations:**
- matrix2x2 palette button
- pmatrix2x2 palette button
- vmatrix2x2 palette button
- All matrix palette buttons (when inserting into existing expression)

**Not Affected:**
- Matrix Builder UI ✅
- Standalone matrix creation ✅
- Type inference ✅
- Rendering ✅

---

## Solution

### Fix Needed
Equation editor palette insertion needs to:
1. Detect highest existing placeholder ID in expression
2. Pass that to template functions as starting ID
3. Or reset counter before calling template
4. Or use a different ID allocation strategy

### Where to Fix
- **Frontend:** Equation editor palette button handler
- **OR Backend:** API endpoint that handles template insertion
- **NOT:** Template functions (they're correct)

### Workaround
Use Matrix Builder instead of palette buttons for inserting matrices into existing expressions.

---

## Testing Notes

**Manual testing (Dec 11, 2025):**
- ✅ Matrix Builder: Perfect with new format
- ⚠️ Palette buttons: One displaced marker
- ✅ Type inference: Working correctly
- ✅ Rendering: Correct output

**Automated testing:**
- ✅ 421 library tests passing
- ✅ All template tests passing
- ✅ Type inference tests passing

---

## Recommendation

**For migration PR:** ✅ Merge
- Migration is sound
- Matrix Builder (primary UI) works perfectly
- Issue is in palette insertion (separate concern)

**For future work:** 🔧 Fix palette ID coordination
- Separate issue/PR
- Not blocking for migration
- Enhancement, not critical bug

---

**Discovered:** December 11, 2025 (manual testing)  
**Documented by:** User observation during equation editor testing  
**Status:** Known issue, workaround available (use Matrix Builder)

