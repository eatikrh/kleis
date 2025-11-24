# Next Steps - Structural Editor

**Date:** 2024-11-22  
**Status:** Typst library integration complete and committed ✅  
**Test page working:** `http://localhost:3000/static/structural_test.html` ✅

---

## What's Working Now

✅ Typst library API integration (10x faster than CLI)  
✅ Placeholder extraction from SVG  
✅ Click-to-edit workflow (empty and filled values)  
✅ AST-based updates  
✅ Fraction template fully functional  
✅ Server running on `http://localhost:3000`

---

## Remaining Tasks

### 1. Fix Overlay Positioning for Filled Values (HIGH PRIORITY)

**Problem:** Green overlays (filled values) use hard-coded estimates instead of actual bounding boxes.

**Current code** (`static/structural_test.html` lines 600-618):
```javascript
if (slotIndex === 0) {
    rectX = 10; rectY = 10;  // ❌ Hard-coded estimates
}
```

**Solution:** Use the bounding boxes we're already extracting from Typst layout tree.

**Steps:**

**A. Update Server** (`src/bin/server.rs` line ~360):
```rust
// In render_typst_handler, add bounding_boxes to response:
let response = serde_json::json!({
    "svg": output.svg,
    "placeholders": output.placeholder_positions.iter().map(...),
    "argument_slots": arg_slots,
    "bounding_boxes": output.argument_bounding_boxes,  // ADD THIS
    "success": true,
});
```

**B. Update Frontend** (`static/structural_test.html` lines 579-651):

Replace hard-coded positioning logic with:
```javascript
// Use actual bounding boxes from Typst layout tree
if (data.bounding_boxes && data.bounding_boxes.length > slotIndex) {
    const bbox = data.bounding_boxes[slotIndex];
    rectX = bbox.x - 3;
    rectY = bbox.y - 3;
    rectWidth = bbox.width + 6;
    rectHeight = bbox.height + 6;
} else {
    // Fallback to placeholder position if available
    if (slot.is_placeholder && data.placeholders) {
        const ph = data.placeholders.find(p => p.id === slot.id);
        if (ph) {
            rectX = ph.x - 3;
            rectY = ph.y - ph.height - 3;
            rectWidth = ph.width + 6;
            rectHeight = ph.height + 6;
        }
    }
}
```

**Estimated time:** 30 minutes

---

### 2. Test All Templates (MEDIUM PRIORITY)

**Currently tested:** Fraction only ✅

**Need to test:**
- √ Square Root (`sqrt`)
- x^n Power (`sup`)
- x_n Subscript (`sub`)
- ∫ Integral (`int_bounds`)
- Σ Sum (`sum_bounds`)

**How to test:**

1. Start server: `cargo run --bin server`
2. Open: `http://localhost:3000/static/structural_test.html`
3. Click each template button
4. For each template:
   - ✅ Verify placeholders appear
   - ✅ Click and fill each placeholder
   - ✅ Verify rendering looks correct
   - ✅ Try editing filled values
   - ✅ Check overlay positioning

**Expected issues:**
- Some templates may need Typst template adjustments in `src/render.rs`
- Overlay positioning may be off (will be fixed by task #1)

**Estimated time:** 1 hour

---

### 3. Polish & Performance (LOW PRIORITY)

**Optional improvements:**

A. **Better overlay styling:**
   - Hover effects
   - Selection indicators
   - Loading states

B. **Keyboard navigation:**
   - Tab between placeholders
   - Enter to edit
   - Escape to cancel

C. **Undo/Redo:**
   - Keep AST history stack
   - Ctrl+Z / Ctrl+Y support

D. **Error handling:**
   - Better error messages
   - Validation feedback
   - Recovery from invalid states

**Estimated time:** 2-4 hours

---

### 4. Integrate into Main Editor (FUTURE)

**Current state:**
- Test page: `static/structural_test.html` (working)
- Main page: `static/index.html` (unchanged - LaTeX text editor)

**Integration plan:**
1. Add mode toggle to main editor (Text vs Structural)
2. Merge structural editor code into `index.html`
3. Preserve existing LaTeX editor functionality
4. Add template palette to main UI
5. Test switching between modes

**Estimated time:** 1-2 days

---

## Quick Commands Reference

### Start Server
```bash
cd /Users/eatik_1/Documents/git/cee/kleis
cargo run --bin server
```

### Test Page
```
http://localhost:3000/static/structural_test.html
```

### Rebuild After Changes
```bash
cargo build --bin server
```

### Kill Server
```bash
lsof -ti:3000 | xargs kill -9
# Or just Ctrl+C in the terminal
```

### Check Compilation
```bash
cargo check --lib
cargo check --bin server
```

### Run Example Test
```bash
cargo run --example test_typst_library
```

---

## Files to Modify for Next Steps

**Task #1 (Fix overlays):**
- `src/bin/server.rs` - Line ~360 (add bounding_boxes to response)
- `static/structural_test.html` - Lines 579-651 (use bounding boxes)

**Task #2 (Test templates):**
- `src/render.rs` - Lines 1214-1228 (Typst templates, if adjustments needed)

**Task #3 (Polish):**
- `static/structural_test.html` - CSS and JavaScript sections

---

## Current Git State

**Last commit:** "Implement Typst library API integration and structural editor backend"

**Modified but not committed:**
- `STRUCTURAL_EDITOR_STATUS.md` (needs review/update)
- `TYPST_LIBRARY_INTEGRATION_COMPLETE.md` (new documentation)
- `NEXT_STEPS.md` (this file)
- `.DS_Store`, `.idea/`, various temp files (should be in .gitignore)

**Remember:** Per repo rules, **do not push** without explicit permission. Commits are fine, pushing requires asking.

---

## Success Metrics

**Current milestone achieved:** ✅ Typst library integration working

**Next milestone:** Fix overlay positioning for production quality

**Final milestone:** All templates tested and working with accurate overlays

---

**Great work so far! The hard part (learning Typst API) is complete.** 🎉

Restart Cursor and you're ready to continue with Task #1 (fix overlay positioning) or Task #2 (test all templates).


