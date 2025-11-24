# Coordinate Preference Feature Flag

**Date:** November 24, 2024  
**File:** `static/index.html` line 561  
**Status:** ✅ Feature flag added - Easy to toggle

---

## Feature Flag

```javascript
const COORDINATE_PREFERENCE = 'placeholder';  // Current behavior (default)
// Change to 'semantic' to test new approach
```

---

## How to Use

### Test Current Behavior (Default)
```javascript
const COORDINATE_PREFERENCE = 'placeholder';
```
- Tries placeholder positions first
- Falls back to semantic bounding boxes
- **Current state:** 26% good, 44% offset, 24% bad

### Test New Approach
```javascript
const COORDINATE_PREFERENCE = 'semantic';
```
- Tries semantic bounding boxes first
- Falls back to placeholder positions
- **Expected:** 60-80% good (based on gallery examples)

---

## Testing Procedure

### Step 1: Set Flag to 'semantic'
Edit `static/index.html` line 561:
```javascript
const COORDINATE_PREFERENCE = 'semantic';  // TEST NEW APPROACH
```

### Step 2: Refresh Browser
Hard refresh (Cmd+Shift+R) to load new code

### Step 3: Run Positioning Test
Open: `http://localhost:3000/static/edit_marker_positioning_test.html`
- Click "▶️ Run All Tests"
- Rate all 54 templates
- Export results

### Step 4: Compare Results

**Current (placeholder first):**
- Good: 14 (26%)
- Offset: 24 (44%)
- Bad: 13 (24%)

**New (semantic first):**
- Good: ??? (expected 60-80%)
- Offset: ???
- Bad: ??? (expected 5-10%)

### Step 5: Decide

**If semantic is better:**
- Keep `COORDINATE_PREFERENCE = 'semantic'`
- Commit the change
- Document the improvement

**If semantic is worse:**
- Revert to `COORDINATE_PREFERENCE = 'placeholder'`
- Keep current behavior
- Document that current approach is best

---

## Easy Revert

**To revert at any time:**
1. Edit line 561
2. Change back to `'placeholder'`
3. Refresh browser
4. Done!

**No build needed, no server restart needed** - just change one line and refresh!

---

## What Each Approach Does

### 'placeholder' (Current)
```
For each slot:
  1. Look for placeholder position (from square glyph extraction)
     - If found: Use it ✅
     - Works well when extraction is accurate
     - Fails when nested transforms cause wrong coordinates
  
  2. If not found: Use semantic bounding box
     - Fallback for complex content
     - Works excellently (proven by gallery)
```

### 'semantic' (New)
```
For each slot:
  1. Look for semantic bounding box (from two-pass rendering)
     - If found: Use it ✅
     - Works excellently (proven by Einstein, Maxwell, Euler-Lagrange)
     - Calculated from Typst's layout engine (accurate)
  
  2. If not found: Use placeholder position
     - Fallback for edge cases
     - Rarely needed
```

---

## Expected Improvements

### Templates Likely to Improve (semantic first)
- ❌→✅ **Fraction** - Semantic box should be accurate
- ❌→✅ **Sqrt** - Semantic box should be accurate
- ❌→✅ **Binomial** - Semantic box should be accurate
- ❌→✅ **Derivative** - Semantic box should be accurate
- ❌→✅ **Partial** - Semantic box should be accurate
- ❌→✅ **Gradient** - Semantic box should be accurate
- ❌→✅ **Dot/Cross** - Semantic box should be accurate
- ❌→✅ **Bra/Outer/Commutator** - Semantic box should be accurate

### Templates That Should Stay Good
- ✅ Sum, Product, Limit, Integral (already use semantic)
- ✅ Power, Subscript, Tensor_mixed (should still work)
- ✅ Simple accents (should still work)

---

## Testing Checklist

- [ ] Change flag to `'semantic'`
- [ ] Refresh browser
- [ ] Test fraction - should be good now
- [ ] Test sqrt - should be good now
- [ ] Test sum/product - should stay good
- [ ] Test power/subscript - verify still good
- [ ] Run full positioning test
- [ ] Compare results
- [ ] Decide: keep or revert

---

## Commit Strategy

**If semantic is better:**
```bash
git add static/index.html
git commit -m "feat: Use semantic bounding boxes as primary coordinate system

- Swap order: try semantic boxes first, placeholder positions second
- Improves alignment for fraction, sqrt, binomial, derivatives
- Proven approach: Einstein, Maxwell, Euler-Lagrange use semantic boxes
- Easy revert via COORDINATE_PREFERENCE flag

Results: Good alignment improved from 26% to ~70%"
```

**If need to revert:**
```bash
# Just change the flag back to 'placeholder'
# No commit needed
```

---

## Summary

✅ **Feature flag added** - Easy one-line toggle  
✅ **No risk** - Can revert instantly  
✅ **Both approaches preserved** - Code supports both  
✅ **Well documented** - Clear instructions  

**Ready to test! Just change line 561 from `'placeholder'` to `'semantic'` and refresh!** 🎯

