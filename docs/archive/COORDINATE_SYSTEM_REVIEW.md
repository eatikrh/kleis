# Coordinate System Review for Edit Marker Placement

**Date:** 2025-11-23  
**Priority:** HIGH - Critical for structural editor UX  
**Status:** ROOT CAUSE IDENTIFIED ✅

## CRITICAL FINDING

The coordinate transformation logic is **CORRECT** ✅  
The grouping logic is **INCORRECT** ❌

**Evidence:**
```
Layout boxes extracted: x=2.4, y=10.6 (matches SVG translate)
After shift: x=0.0, y=0.0 (correct normalization)
After grouping: Creates NEW boxes with wrong coordinates!
```

**The bug is in `group_boxes_into_arguments()`, not coordinate transforms.**

## VERIFICATION FROM TYPST SOURCE CODE

**Checked:** `/Users/eatik_1/Documents/git/cee/typst/crates/typst-svg/src/lib.rs`

### Typst SVG Generation (lines 196-207)
```rust
for (pos, item) in frame.items() {
    let x = pos.x.to_pt();
    let y = pos.y.to_pt();
    self.xml.start_element("g");
    self.xml.write_attribute_fmt("transform", format_args!("translate({x} {y})"));
    // ... render item ...
}
```

### Our Extraction (typst_compiler.rs line 231)
```rust
let item_ts = ts.pre_concat(Transform::translate(pos.x, pos.y));
// ... then transform Point::zero() ...
let tl = Point::zero().transform(item_ts);
```

**VERDICT: Our logic MATCHES Typst's logic exactly** ✅

We accumulate transforms the same way Typst does. Our coordinates should be correct.

### The Real Problem

**Lines 597-617 in `group_content_into_arguments()`:**
```rust
// Recalculate bounding box from grouped boxes
let min_x = line.iter().map(|b| b.x).fold(f64::INFINITY, |a, b| a.min(b));
let min_y = line.iter().map(|b| b.y).fold(f64::INFINITY, |a, b| a.min(b));
// Creates NEW aggregate box
```

This creates boxes that encompass multiple glyphs, causing:
- Overlapping (boxes too large)
- Missing args (horizontal layouts grouped as one line)
- Imprecise clicking (can't select individual elements)

**Solution:** Don't aggregate. Use individual glyph boxes or use smarter grouping.

---

## The Problem

Edit markers (clickable overlays) are positioned using bounding boxes from Typst's layout engine. However, there are multiple coordinate systems involved:

1. **Typst Layout Coordinates** - From layout tree
2. **SVG Coordinates** - In the generated SVG
3. **Browser/DOM Coordinates** - Where we place overlays

**Current assumption:** Typst coordinates = SVG coordinates (with small padding)  
**Reality:** This may not always be true due to transforms, viewBox, margins, etc.

---

## Coordinate Systems Involved

### 1. Typst Internal Layout

**Source:** `typst::layout::Frame` and `FrameItem` positions

**Characteristics:**
- Origin: Relative to parent frame
- Units: Absolute points (1pt = 1/72 inch)
- Y-axis: Typically top-down (0 at top)
- Positions: Can be negative (for centering, etc.)

**Example from test:**
```
Layout bounds: min_x=2.40, min_y=10.61
Arg 0: x=-4.0, y=-4.0, w=28.0, h=32.0
```

**Note:** Negative coordinates suggest positioning relative to some reference point.

### 2. SVG ViewBox and Coordinates

**Source:** Generated SVG string

**Characteristics:**
- ViewBox: `0 0 width height` (defines coordinate space)
- Units: User units (typically same as layout points)
- Y-axis: Top-down (0 at top)
- Transforms: May include `translate()`, `scale()`, `matrix()`

**Example:**
```svg
<svg viewBox="0 0 17.5 43.6" ...>
  <g transform="translate(0, 0)">
    <!-- content -->
  </g>
</svg>
```

**Critical:** If there's a `translate()` or other transform, we must apply it to our overlay coordinates!

### 3. Browser DOM Coordinates

**Source:** Where we place `<rect>` overlays

**Characteristics:**
- Same coordinate space as SVG (if overlays are inside SVG)
- Affected by CSS transforms on SVG element
- Need to account for any parent transforms

**Current approach (lines 755-758 in index.html):**
```javascript
rectX = bbox.x - 3;
rectY = bbox.y - 3;
rectWidth = bbox.width + 6;
rectHeight = bbox.height + 6;
```

**Assumption:** `bbox.x/y` are directly usable as SVG coordinates.

---

## Test Results Analysis

### Working Cases

**Fraction:** 2 args, no overlap ✓
```
Arg 0 (numerator):   x=-4.0, y=-4.0,  w=28.0, h=32.0
Arg 1 (denominator): x=-3.0, y=28.7,  w=28.0, h=32.0
```
- Vertical separation: ~32pt (numerator height)
- No overlap
- **This works because:** Simple vertical stacking

### Problematic Cases

**Matrix 2x2:** Args overlap ⚠️
```
Arg 0: x=-4.0, y=-4.0, w=71.1, h=46.4  (entire matrix?)
Arg 1: x=9.5,  y=24.7, w=44.9, h=32.0  (second row?)
```
- Arg 0 seems to be the whole matrix
- Arg 1 is inside Arg 0
- **Problem:** Can't click individual elements

**Integral:** All args overlap ⚠️
```
Arg 0 (upper bound): x=20.0, y=-4.0,  w=28.0, h=28.0
Arg 1 (integrand):   x=-4.0, y=22.7,  w=124.3, h=32.0
Arg 2 (lower bound): x=9.2,  y=48.1,  w=28.0, h=28.0
```
- Bounds and integrand all overlap
- **Problem:** Can't distinguish which part to click

**Inner Product:** Only 1 arg detected ⚠️
- Should have 2 args (bra and ket)
- Only showing 1 bounding box
- **Problem:** Can't edit both parameters separately

---

## Root Causes

### Issue 1: Bounding Box Extraction Logic

**Location:** `src/math_layout/typst_compiler.rs`

The `extract_argument_bounding_boxes` function groups layout boxes by line. This works for:
- Fractions (numerator on line 0, denominator on line 1)
- Simple vertical layouts

But fails for:
- Horizontal layouts (inner product: `⟨u|v⟩`)
- Complex nested structures (integrals with bounds)
- Matrices (2D grid)

**Current logic:**
```rust
// Group by line (Y coordinate)
let mut lines: Vec<Vec<&LayoutBox>> = Vec::new();
// ... group boxes with similar Y ...
// Each line becomes one argument
```

**Problem:** This assumes arguments are on different lines (vertical stacking).

### Issue 2: Coordinate Transform Not Applied

**Location:** `static/index.html` lines 755-758

We use `bbox.x` and `bbox.y` directly without checking for:
- SVG viewBox offset
- Global `<g transform="...">` 
- Page margins
- Scaling factors

**Current code:**
```javascript
rectX = bbox.x - 3;  // Just add padding
rectY = bbox.y - 3;
```

**Missing:**
- Parse SVG viewBox
- Extract and apply transforms
- Account for margins

### Issue 3: Negative Coordinates

Test shows negative coordinates (x=-4.0, y=-4.0). This suggests:
- Coordinates are relative to some reference point
- May need offset adjustment
- Could indicate centering or margin

---

## What Needs Review

### 1. Backend: Bounding Box Extraction

**File:** `src/math_layout/typst_compiler.rs`

**Functions to review:**
- `extract_argument_bounding_boxes()` - Line grouping logic
- `extract_bounding_boxes_from_frame()` - Coordinate accumulation
- `group_boxes_into_arguments()` - How boxes are grouped

**Questions:**
- Should we group by spatial proximity instead of lines?
- How to handle horizontal layouts (inner product)?
- How to handle nested structures (matrix elements)?
- Should we extract individual elements or grouped regions?

### 2. Frontend: Coordinate Transformation

**File:** `static/index.html`

**Functions to review:**
- Overlay positioning logic (lines 750-775)
- SVG parsing and transform extraction
- Coordinate conversion

**Questions:**
- Do we parse viewBox?
- Do we extract and apply `<g transform="...">`?
- Do we handle scaling?
- Do we account for negative coordinates?

### 3. Typst SVG Generation

**File:** `src/math_layout/typst_compiler.rs`

**Settings to review:**
- Page margins (currently 0pt?)
- Alignment (center vs left)
- Padding around content
- Transform generation

**Questions:**
- Can we control SVG coordinate system?
- Can we ensure consistent origin?
- Can we avoid transforms for simpler overlay math?

---

## Proposed Investigation Plan

### Phase 1: Document Current Behavior (1-2 hours)

1. **For each template type:**
   - Generate SVG
   - Extract viewBox
   - Extract transforms
   - Note bounding box coordinates
   - Compare to visual positions

2. **Create test HTML** showing:
   - SVG rendering
   - Bounding boxes as colored rectangles
   - Coordinate values displayed
   - Visual verification

3. **Document findings:**
   - Which templates work correctly?
   - Which have misaligned overlays?
   - What transforms are present?
   - What coordinate offsets exist?

### Phase 2: Fix Coordinate Transformation (2-4 hours)

1. **Parse SVG structure:**
   ```javascript
   const svg = document.querySelector('svg');
   const viewBox = svg.getAttribute('viewBox').split(' ');
   const transform = svg.querySelector('g').getAttribute('transform');
   ```

2. **Extract transform matrix:**
   ```javascript
   function parseTransform(transformStr) {
       // Parse "translate(x, y)" or "matrix(...)"
       // Return {tx, ty, sx, sy, ...}
   }
   ```

3. **Apply to overlay coordinates:**
   ```javascript
   const transformed = applyTransform(bbox, transform);
   rectX = transformed.x - padding;
   rectY = transformed.y - padding;
   ```

### Phase 3: Fix Bounding Box Extraction (4-6 hours)

1. **Improve grouping logic:**
   - Don't just group by lines
   - Consider spatial relationships
   - Handle horizontal layouts
   - Detect nested structures

2. **Add metadata to bounding boxes:**
   ```rust
   pub struct ArgumentBoundingBox {
       pub arg_index: usize,
       pub x: f64,
       pub y: f64,
       pub width: f64,
       pub height: f64,
       pub element_type: String,  // NEW: "numerator", "matrix_element", etc.
       pub parent_index: Option<usize>,  // NEW: For nested structures
   }
   ```

3. **Test with all template types**

### Phase 4: Visual Testing (2-3 hours)

1. Create interactive test HTML
2. Test each template type
3. Verify overlays align perfectly
4. Document any remaining issues

---

## Immediate Action Items

### 1. Create Visual Test Page

Create `static/test_overlay_alignment.html` that:
- Shows SVG rendering
- Overlays bounding boxes as colored rectangles
- Displays coordinate values
- Allows testing different templates
- Shows viewBox and transform info

### 2. Add Coordinate Transform Extraction

Update `static/index.html` to:
- Parse SVG viewBox
- Extract `<g transform="...">`
- Apply transforms to overlay coordinates
- Handle negative coordinates

### 3. Document SVG Structure

For each template, document:
- ViewBox values
- Transform values
- Bounding box coordinates
- Visual positions
- Any discrepancies

---

## Test Cases Needed

Priority templates to test:

1. ✅ **Fraction** - Working (baseline)
2. ⚠️ **Matrix** - Overlapping boxes
3. ⚠️ **Integral** - Overlapping boxes
4. ⚠️ **Sum/Product** - Overlapping boxes
5. ⚠️ **Inner Product** - Missing second arg
6. **Subscript/Superscript** - Need to test
7. **Nested fractions** - Need to test
8. **Square root** - Need to test
9. **Commutator** - Need to test
10. **Quantum kets/bras** - Need to test

---

## Questions to Answer

1. **Why are coordinates negative?**
   - Centering logic?
   - Margin offset?
   - Transform artifact?

2. **Why do integrals/sums have overlapping boxes?**
   - Are we grouping too broadly?
   - Should bounds be separate from integrand?
   - Is line grouping wrong for these?

3. **Why does inner product only show 1 arg?**
   - Are both parameters on same line?
   - Is grouping merging them?
   - Is one parameter not being detected?

4. **Do we need different strategies for different templates?**
   - Vertical layouts (fractions) - group by line ✓
   - Horizontal layouts (inner product) - group by column?
   - 2D layouts (matrices) - grid detection?
   - Complex (integrals) - semantic understanding?

---

## Resources

**Related Files:**
- `docs/TYPST_SVG_ANALYSIS.md` - Previous analysis
- `src/math_layout/typst_compiler.rs` - Bounding box extraction
- `static/index.html` - Overlay positioning
- `src/bin/test_edit_markers.rs` - Automated testing

**Typst Source:**
- `/Users/eatik_1/Documents/git/cee/typst` - Typst library source
- `crates/typst-svg/src/lib.rs` - SVG generation
- `crates/typst/src/layout/` - Layout engine

---

**Status:** Investigation needed  
**Priority:** HIGH - Blocks structural editor UX  
**Estimated Effort:** 8-12 hours for complete fix  
**Next Step:** Create visual test page and document findings

