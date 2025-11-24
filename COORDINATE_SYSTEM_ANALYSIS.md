# Coordinate System Analysis - Investigation Report

**Date:** November 24, 2024  
**Status:** 🔍 Investigation Complete - No Code Changes  
**Finding:** Semantic boxes work excellently; placeholder extraction has issues

---

## Key Discovery

**Complex gallery examples work exceptionally well:**
- ✅ Euler–Lagrange (single var)
- ✅ Einstein Field Equations (core)
- ✅ Maxwell tensor from potential

**Simple palette templates have issues:**
- ❌ Fraction (empty placeholders)
- ❌ Square root (empty placeholders)
- ❌ Binomial (empty placeholders)

---

## The Critical Insight

### What Works (Semantic Bounding Boxes)

**Gallery examples** like Einstein Field Equations:
```rust
// Has actual content: G_μν + Λg_μν = κT_μν
equals(
    plus(G_mn, times(Λ, g_mn)),
    times(κ, T_mn)
)
```

**When rendered in structural mode:**
- Uses **semantic bounding boxes** (from two-pass rendering)
- Boxes are calculated from layout tree
- Coordinates are accurate
- **Edit markers align perfectly** ✅

### What Fails (Placeholder Position Extraction)

**Simple templates** like fraction:
```rust
// Has only placeholders: □/□
scalar_divide(
    Placeholder{id:0, hint:"numerator"},
    Placeholder{id:1, hint:"denominator"}
)
```

**When rendered in structural mode:**
- Tries to use **placeholder positions** (from square glyphs in SVG)
- Extraction reports wrong coordinates: `(0, 0)` instead of `(2.4, 13.99)`
- **Edit markers misaligned** ❌

---

## The Two Coordinate Systems

### System 1: Placeholder Positions (Lines 794-873)

**Purpose:** Find exact position of square.stroked glyphs in SVG

**Method:**
1. Parse SVG with regex to find `<g transform="translate(X Y)">` + `<use xlink:href="#square"/>`
2. Extract (X, Y) coordinates
3. Assign to placeholder IDs by order

**Problem:** Captures **outer** transforms `(0, 0)` instead of **inner** transforms `(2.4, 13.99)`

**Used by:** Simple templates with only placeholders

### System 2: Semantic Bounding Boxes (Lines 115-286)

**Purpose:** Calculate bounding boxes for each AST argument

**Method:**
1. Render each argument in isolation
2. Get text boxes from Typst layout tree
3. Match boxes in full rendering
4. Create semantic regions

**Result:** Accurate coordinates from Typst's layout engine

**Used by:** Complex expressions with content

---

## Why Gallery Examples Work

### Einstein Field Equations
```
G_μν + Λg_μν = κT_μν
```

**Structure:**
- Operation: equals
  - Arg 0: plus(G_mn, times(Λ, g_mn))
  - Arg 1: times(κ, T_mn)

**Each argument has content** (not placeholders), so:
- Backend renders each argument separately
- Gets accurate layout boxes from Typst
- Matches in full rendering
- **Semantic boxes are perfect** ✅

### Euler–Lagrange
```
∂L/∂y - d/dx(∂L/∂y') = 0
```

**Structure:**
- Complex nested operations with actual variables
- No empty placeholders
- Uses semantic bounding boxes
- **Markers align perfectly** ✅

---

## Why Simple Templates Fail

### Fraction Template
```
□/□
```

**Structure:**
- Operation: scalar_divide
  - Arg 0: Placeholder{id:0}
  - Arg 1: Placeholder{id:1}

**Both arguments are placeholders**, so:
- Backend tries to use placeholder positions
- Extraction finds square glyphs at wrong coordinates
- Frontend draws overlays at wrong positions
- **Markers misaligned** ❌

---

## The Nested Transform Problem

### Fraction SVG Structure (Hypothesis)

```svg
<svg viewBox="0 0 23.472 48.696">
  <g transform="translate(0 0)">              ← OUTER (captured by regex)
    <g class="typst-group">
      <g>
        <g transform="translate(2.4 13.99)">  ← INNER (actual position)
          <g class="typst-text" transform="scale(1, -1)">
            <use xlink:href="#square" x="0"/>
          </g>
        </g>
      </g>
    </g>
  </g>
  
  <g transform="translate(0 0)">              ← OUTER
    <g class="typst-group">
      <g>
        <g transform="translate(2.4 46.7)">   ← INNER (actual position)
          <g class="typst-text" transform="scale(1, -1)">
            <use xlink:href="#square" x="0"/>
          </g>
        </g>
      </g>
    </g>
  </g>
</svg>
```

**Current regex:** Matches outer `translate(0 0)` for both squares  
**Needed:** Match inner `translate(2.4 13.99)` and `translate(2.4 46.7)`

---

## Test Results Pattern Analysis

### Perfect Alignment (14 templates - 26%)

**What they have in common:**
- Either use semantic boxes (sum, product, limit, integral)
- Or have simple single-level structure (factorial, vector_bold, hat, bar)
- No nested transform issue

**Examples:**
- `sum`, `product`, `limit` → Complex operators, use semantic boxes
- `power`, `subscript`, `tensor_mixed` → Superscripts work, might not have nesting
- `factorial`, `vector_bold`, `hat`, `bar` → Simple wrappers

### Poor Alignment (13 templates - 24%)

**What they have in common:**
- Fraction-like layouts (fraction, sqrt, binomial, derivative)
- Horizontal binary operators (dot, cross, commutator)
- Complex quantum (bra, outer)

**Pattern:** All have **nested group structure** in SVG

---

## The Calibration Issue

**Lines 452-482:** Calibration offset calculation

```rust
// Uses first placeholder to calibrate
let first_ph = placeholder_positions.first()  // (0, 0) ← WRONG!
let match_box = candidates.first()            // (2.4, 13.99) from layout
offset_x = first_ph.x - match_box.x           // 0 - 2.4 = -2.4
offset_y = first_ph.y - match_box.y           // 0 - 13.99 = -13.99
```

**Result:** Negative offset that makes things worse!

**If first placeholder had correct position:**
```rust
first_ph = (2.4, 13.99)  ✅
match_box = (2.4, 13.99)
offset = (0, 0)  ← Correct!
```

---

## Solutions (In Order of Preference)

### Solution 1: Use Semantic Boxes for All Templates (Recommended)

**Approach:** Don't use placeholder positions at all. Always use semantic bounding boxes.

**Rationale:**
- Semantic boxes work excellently (proven by gallery examples)
- They're calculated from Typst's layout engine (accurate)
- No regex parsing issues
- No nested transform problems

**Change:** In frontend (index.html line 900), skip placeholder position lookup and go straight to semantic boxes.

**Pros:**
- Simple change
- Proven to work (Einstein, Maxwell, Euler-Lagrange)
- No backend changes needed

**Cons:**
- Might be slightly less precise for simple cases
- But gallery examples prove it works great!

### Solution 2: Fix Nested Transform Accumulation (Complex)

**Approach:** Parse SVG as XML, traverse tree, accumulate all transforms.

**Effort:** 1-2 days  
**Risk:** High (could break working templates)  
**Benefit:** Placeholder positions would be correct

### Solution 3: Hybrid Approach

**Approach:** Use semantic boxes as primary, placeholder positions as fallback (reverse of current).

**Change:** Try semantic boxes first, only use placeholder positions if semantic boxes fail.

---

## Recommendation

**Use Solution 1: Semantic Boxes for All**

**Why:**
- ✅ Proven to work (Einstein, Maxwell, Euler-Lagrange align perfectly)
- ✅ Simple change (just reorder the if/else in frontend)
- ✅ No backend changes needed
- ✅ Low risk
- ✅ Would fix most of the 13 "bad" templates

**Implementation:**
```javascript
// In index.html line 899-923
// SWAP the order: try semantic boxes FIRST, placeholder positions second

// 1. Try semantic bounding box (FIRST)
const nodePathId = nodeIdFromPath(slot.path || []);
const bbox = data.argument_bounding_boxes && 
            data.argument_bounding_boxes.find(b => b.node_id === nodePathId);

if (bbox) {
    // Use semantic box (works great!)
    rectX = bbox.x - 3;
    rectY = bbox.y - 3;
    rectWidth = bbox.width + 6;
    rectHeight = bbox.height + 6;
    foundPosition = true;
} else {
    // 2. Fallback to placeholder position
    const ph = data.placeholders && data.placeholders.find(p => p.id === slot.id);
    if (ph) {
        // Use placeholder position
        ...
    }
}
```

**Expected impact:**
- Fraction, sqrt, binomial, derivative → Would use semantic boxes → Likely good alignment
- Sum, product, limit → Already use semantic boxes → Stay good
- Overall: 13 bad → ~2-3 bad (80-90% success rate)

---

## Conclusion

**The semantic bounding box system works excellently** - proven by Einstein, Maxwell, and Euler-Lagrange equations.

**The placeholder position extraction has issues** - nested transforms cause wrong coordinates.

**Best fix:** Use semantic boxes as primary method (they're more accurate anyway).

**Effort:** ~30 minutes to swap the order and test  
**Risk:** Low (semantic boxes proven to work)  
**Benefit:** Would fix most of the 13 "bad" templates

**Shall I implement this simple fix?** 🎯

