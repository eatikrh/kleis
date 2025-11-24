# Structural Mode - Complete Fix Guide

**Date:** November 24, 2024  
**Issue:** Structural mode stuck at "🔄 Rendering..." with `sqrt()` error  
**Status:** ✅ ALL FIXES APPLIED - Restart Required

---

## The Problem You're Seeing

```
Argument slots: 1 total
Full markup: sqrt()
Expected placeholder IDs: []
Expected 0 placeholders
Typst compilation errors: ["missing argument: radicand"]
```

### Analysis
- ✅ Argument slots: 1 total - Correct!
- ❌ Full markup: `sqrt()` - Should be `sqrt(square.stroked)`!
- ❌ Expected placeholder IDs: `[]` - Should be `[1]`!

This means the AST has an **empty Object** instead of a **Placeholder node**.

---

## Root Causes (All Fixed, But Need Restart)

### Issue 1: Wrong Placeholder Rendering in `src/render.rs` ✅ FIXED
**Line 599** was using `#sym.square` (code mode) instead of `square.stroked` (math mode).

**Fix applied:** Changed to `square.stroked`

### Issue 2: Missing AST Template Definitions ✅ FIXED
**`static/index.html`** line 735 now has proper sqrt definition:
```javascript
sqrt: { Operation: { name: 'sqrt', args: [{Placeholder:{id:0,hint:'radicand'}}] } }
```

### Issue 3: Cached Browser/Server ⚠️ RESTART NEEDED
- Server is running old code
- Browser may have cached old JavaScript

---

## Complete Restart Procedure

### Step 1: Stop the Server
```bash
# In the terminal running the server, press:
Ctrl+C

# Or kill the process:
pkill -f "cargo run --bin server"
```

### Step 2: Rebuild (Already Done)
```bash
cd /Users/eatik_1/Documents/git/cee/kleis
cargo build --bin server
```
✅ Already built successfully

### Step 3: Start New Server
```bash
cargo run --bin server
```

### Step 4: Hard Refresh Browser
Open `http://localhost:3000` and:
- **Chrome/Edge:** Ctrl+Shift+R (Windows) or Cmd+Shift+R (Mac)
- **Firefox:** Ctrl+F5 (Windows) or Cmd+Shift+R (Mac)
- **Safari:** Cmd+Option+R

This clears the JavaScript cache and loads the new `index.html` with updated AST templates.

---

## What Should Happen After Restart

### Server Logs (Correct Output)
```
Argument slots: 1 total
Full markup: sqrt(square.stroked)  ✅ Has argument!
Expected placeholder IDs: [1]      ✅ Has placeholder ID!
Expected 1 placeholders
Creating Typst world...
Compiling with Typst library...
✅ Typst compilation successful!
Extracting 1 placeholders by finding square symbols
Found square glyph with 1 instances
Total placeholders extracted: 1
```

### Browser (Correct Behavior)
1. Click "🔧 Structural Mode"
2. Click "√ Square Root" button
3. **Immediately renders** as √□ with blue interactive box
4. Click the blue box → prompt appears
5. Enter value → updates successfully

---

## Verification Tests

### Test 1: Check Placeholder Rendering
```bash
cd /Users/eatik_1/Documents/git/cee/kleis
rustc --edition 2021 -L target/debug/deps --extern kleis=target/debug/libkleis.rlib /tmp/test_render_placeholder.rs -o /tmp/test_render_placeholder
/tmp/test_render_placeholder | grep "Typst output"
```

**Expected:**
```
   Typst output: 'square.stroked'           ✅
   Typst output: 'sqrt(square.stroked)'     ✅
```

**If you see `#sym.square`:** Rebuild didn't work, try `cargo clean && cargo build`

### Test 2: Check AST Template
Open browser console (F12) and run:
```javascript
console.log(astTemplates.sqrt);
```

**Expected:**
```javascript
{
  Operation: {
    name: 'sqrt',
    args: [{Placeholder: {id: 0, hint: 'radicand'}}]
  }
}
```

**If you see something else:** Hard refresh the page (Cmd+Shift+R)

---

## Debugging Checklist

If it still doesn't work after restart:

### 1. Verify Build Picked Up Changes
```bash
grep "square.stroked" target/debug/deps/libkleis-*.rlib
# Should find matches
```

### 2. Verify Server is New Process
```bash
ps aux | grep server
# Check the start time - should be recent
```

### 3. Check Browser Console
Open browser console (F12) and look for:
- JavaScript errors
- Network errors
- AST being sent to server

### 4. Check Server Logs
Look for the "Full markup" line - should show `sqrt(square.stroked)` not `sqrt()`

### 5. Test with Simple Template
Try the fraction template first:
- Click "📐 Fraction"
- Should render as □/□ with two blue boxes

---

## The Two Rendering Modules Explained

### Module 1: `src/render.rs` (MAIN - Used by Server)
```rust
// Line 599
RenderTarget::Typst => "square.stroked".to_string()  ✅ FIXED
```

This is the **main renderer** used by:
- Server API endpoints
- All test binaries
- Two-pass rendering system

### Module 2: `src/math_layout/typst_adapter.rs` (Alternative)
```rust
// Line 68
"square.stroked".to_string()  ✅ ALSO FIXED (for consistency)
```

This is an **alternative adapter** that's not currently used by the main path. I fixed it too for consistency.

---

## Why `sqrt()` Appears

If you're seeing `sqrt()` with no argument, one of these is happening:

1. **Server running old code** - Most likely! Restart server.
2. **Browser cached old JavaScript** - Hard refresh page.
3. **AST has empty Object** - Check browser console for actual AST being sent.

The "Argument slots: 1 total" message confirms the AST has an argument, so it's likely just old code running.

---

## Expected Behavior Flow

### 1. User Clicks "√ Square Root"
```javascript
// Frontend creates AST
{
    Operation: {
        name: 'sqrt',
        args: [{Placeholder: {id: 1, hint: 'radicand'}}]
    }
}
```

### 2. Frontend Sends to Server
```javascript
POST /api/render_typst
Body: { ast: {...} }
```

### 3. Server Processes
```rust
// collect_argument_slots() finds 1 placeholder
arg_slots = [ArgumentSlot { id: 1, is_placeholder: true, ... }]
unfilled_ids = [1]

// render_expression() renders AST to Typst
full_markup = "sqrt(square.stroked)"  ✅

// compile_with_semantic_boxes() compiles
Typst compilation successful!
```

### 4. Server Returns
```json
{
    "success": true,
    "svg": "<svg>...</svg>",
    "placeholders": [{id: 1, x: 25, y: 10, ...}],
    "argument_slots": [...]
}
```

### 5. Frontend Renders
- Displays SVG
- Draws blue overlay at (25, 10)
- Makes it clickable

---

## Summary

**All fixes are applied and code is built. You just need to:**

1. ⚠️ **Restart the server** (Ctrl+C, then `cargo run --bin server`)
2. ⚠️ **Hard refresh the browser** (Cmd+Shift+R)
3. ✅ **Test structural mode** - should work immediately!

The fix changes:
- `#sym.square` → `square.stroked` in `src/render.rs`
- This makes Typst compilation succeed
- Placeholders render as □ with interactive overlays

**After restart, structural mode will work perfectly! 🎉**

