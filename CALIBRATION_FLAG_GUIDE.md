# Calibration Feature Flag Guide

**Date:** November 24, 2024  
**File:** `src/math_layout/typst_compiler.rs` line 94  
**Purpose:** Easy toggle to test with/without calibration

---

## Feature Flag

```rust
const USE_CALIBRATION: bool = true;  // Current behavior (default)
// Change to false to test without calibration
```

---

## What Calibration Does

**Purpose:** Align layout coordinate system with SVG coordinate system

**Process:**
1. Extract layout boxes from Typst's frame (normalized to start at 0,0)
2. Extract placeholder positions from SVG (in SVG coordinates)
3. Match first placeholder to first layout box
4. Calculate offset: `offset = SVG_position - Layout_position`
5. Apply offset to all layout boxes

**Goal:** Make layout boxes use same coordinate system as SVG

---

## Current Issue

**With calibration enabled:**
- Works for simple expressions
- Fails for nested matrix content
- Produces negative Y coordinates
- Causes marker misalignment

**Example (matrix with inner product):**
```
Calibrated offset: (0.00, -17.45)
Result: Semantic boxes at y=-4.00 (off-screen)
```

---

## Testing Without Calibration

### Step 1: Disable Calibration
Edit `src/math_layout/typst_compiler.rs` line 94:
```rust
const USE_CALIBRATION: bool = false;  // TEST: No calibration
```

### Step 2: Rebuild
```bash
cargo build
```

### Step 3: Restart Server
```bash
cargo run --bin server
```

### Step 4: Test
1. Refresh browser
2. Test simple matrix (all placeholders)
3. Test complex matrix (with inner product)
4. Check debug panel for coordinates
5. Verify marker alignment

### Step 5: Compare Results

**With calibration (current):**
- Simple expressions: Good
- Complex nested: Bad (negative coords)

**Without calibration (test):**
- Simple expressions: ??? (might break)
- Complex nested: ??? (might fix)

---

## Expected Outcomes

### Scenario A: Without Calibration Works Better
- Semantic boxes have correct positive coordinates
- Markers align properly for nested content
- **Action:** Keep `USE_CALIBRATION = false`, commit

### Scenario B: Without Calibration Breaks Things
- Semantic boxes misaligned for simple expressions
- Coordinates don't match SVG
- **Action:** Revert to `USE_CALIBRATION = true`, investigate deeper

### Scenario C: Both Have Issues
- Need different approach entirely
- **Action:** Consider alternative coordinate strategies

---

## Revert Procedure

**If disabling breaks things:**
1. Edit line 94: Change back to `true`
2. Rebuild: `cargo build`
3. Restart server
4. Done - back to current behavior

**No git changes needed** - just change one line and rebuild.

---

## What to Check

### Test Cases
1. **Simple fraction** `□/□`
2. **Simple matrix** `3×3 all placeholders`
3. **Matrix with inner product** (your failing case)
4. **Einstein equation** (complex, should work)
5. **Sum/Product/Limit** (currently work well)

### For Each Test
- Check marker alignment visually
- Check debug panel coordinates
- Look for negative values
- Compare with/without calibration

---

## Server Logs to Watch

**With calibration:**
```
Calibration enabled
Found N candidate boxes
Calibrated offset: (X, Y) using placeholder ID Z
Applying calibration offset to N boxes
```

**Without calibration:**
```
Calibration disabled - using layout boxes as-is
Skipping calibration
```

---

## My Hypothesis

**Without calibration:**
- Layout boxes stay normalized (0,0 origin)
- Semantic boxes have positive coordinates
- But they might not align with SVG coordinate system
- Placeholder positions (from SVG) would still be correct

**The key question:** Do layout boxes and SVG use the same coordinate system naturally, or does calibration align them?

**Testing will reveal the answer!**

---

## Next Steps

1. **Test with calibration ON** (current) - Document exact issue
2. **Test with calibration OFF** - See if it helps
3. **Compare results** - Which is better?
4. **Decide:** Keep best approach or investigate further

**Ready to test! Change line 94 to `false`, rebuild, and test!** 🔧

