# Debug Guide: Structural Mode Rendering Issue

**Date:** November 24, 2024  
**Issue:** Still seeing `sqrt()` instead of `sqrt(square.stroked)`  
**Status:** Enhanced debugging added

---

## Current Situation

You're still seeing:
```
Full markup: sqrt()
Expected placeholder IDs: []
```

This means the AST has an **empty argument** (not a Placeholder node).

---

## Enhanced Debugging Added

I've added detailed logging to `src/bin/server.rs` that will show:
1. The raw JSON AST received from frontend
2. The parsed Expression structure
3. All argument slots with their properties

### New Server Logs

After restarting the server, you'll see:
```
=== render_typst_handler called ===
Received AST JSON: {...}
Parsed Expression: Operation {
    name: "sqrt",
    args: [...]  ← This will show if it's Placeholder or Object
}
Argument slots: 1 total
  Slot 0: id=X, is_placeholder=true/false, hint='...'
```

---

## Restart Procedure

### 1. Stop Current Server
```bash
# In server terminal, press Ctrl+C
```

### 2. Start New Server with Debug Logging
```bash
cd /Users/eatik_1/Documents/git/cee/kleis
cargo run --bin server
```

### 3. Hard Refresh Browser
- Mac: `Cmd+Shift+R`
- Windows: `Ctrl+Shift+R`
- This clears JavaScript cache

### 4. Open Browser Console
- Press F12 or Cmd+Option+I
- Go to "Console" tab
- Look for the log: `renderStructuralEditor: sending AST to /api/render_typst`

### 5. Click "√ Square Root" in Structural Mode

### 6. Check Both Logs

**Browser Console should show:**
```javascript
renderStructuralEditor: sending AST to /api/render_typst
{
  Operation: {
    name: "sqrt",
    args: [
      {Placeholder: {id: 1, hint: "radicand"}}  ← Should be Placeholder!
    ]
  }
}
```

**Server Terminal should show:**
```
=== render_typst_handler called ===
Received AST JSON: Object { ... }
Parsed Expression: Operation {
    name: "sqrt",
    args: [
        Placeholder { id: 1, hint: "radicand" }  ← Should be Placeholder!
    ]
}
Argument slots: 1 total
  Slot 0: id=1, is_placeholder=true, hint='radicand'  ← Should be true!
Full markup: sqrt(square.stroked)  ← Should have argument!
```

---

## Diagnosis Based on Logs

### Case 1: Browser Shows `Object("")` Instead of `Placeholder`
**Problem:** Browser JavaScript is cached or AST template is wrong

**Solution:**
1. Hard refresh browser (Cmd+Shift+R)
2. Check browser console: `console.log(astTemplates.sqrt)`
3. Should show: `{Operation: {name: 'sqrt', args: [{Placeholder: ...}]}}`

### Case 2: Server Shows `Object("")` Instead of `Placeholder`
**Problem:** JSON deserialization issue or frontend sending wrong format

**Solution:**
1. Check the "Received AST JSON" line in server logs
2. Verify it has `"Placeholder": {"id": 1, "hint": "radicand"}`
3. Check `json_to_expression()` function

### Case 3: Server Shows `Placeholder` But Renders as `sqrt()`
**Problem:** Placeholder rendering is broken or template lookup fails

**Solution:**
1. Check if `sqrt` template exists in Typst templates
2. Verify placeholder renders as `square.stroked`
3. Check template string: `"sqrt({arg})"`

---

## Quick Tests

### Test 1: Verify Build Has Fix
```bash
cd /Users/eatik_1/Documents/git/cee/kleis
strings target/debug/libkleis.rlib | grep "square.stroked"
```
**Should find:** `square.stroked` (not `#sym.square`)

### Test 2: Test Rendering Directly
```bash
rustc --edition 2021 -L target/debug/deps --extern kleis=target/debug/libkleis.rlib /tmp/test_render_placeholder.rs -o /tmp/test_render_placeholder
/tmp/test_render_placeholder | grep "sqrt"
```
**Should show:** `sqrt(square.stroked)` ✅

### Test 3: Check Browser Cache
Open browser console and run:
```javascript
// Check if AST template is correct
console.log(astTemplates.sqrt);

// Should show:
// {Operation: {name: 'sqrt', args: [{Placeholder: {id: 0, hint: 'radicand'}}]}}
```

---

## Most Likely Cause

Based on the symptoms, the most likely issue is:

**The browser is using cached JavaScript** that has old AST template definitions.

Even though you restarted the server, the browser may still be using the old `index.html` JavaScript from cache.

### Solution
1. **Hard refresh:** Cmd+Shift+R (Mac) or Ctrl+Shift+R (Windows)
2. **Clear cache:** Browser settings → Clear browsing data → Cached images and files
3. **Disable cache:** Browser DevTools (F12) → Network tab → Check "Disable cache"
4. **Force reload:** Close and reopen browser completely

---

## What the Debug Logs Will Tell Us

After restart with debug logging, we'll see exactly where the problem is:

- **If browser console shows `Placeholder`** → Frontend is correct
- **If server shows `Object("")`** → JSON serialization issue
- **If server shows `Placeholder` but renders `sqrt()`** → Template lookup issue
- **If server shows `sqrt(square.stroked)`** → Everything works! ✅

---

## Action Items

1. ⚠️ **Stop server** (Ctrl+C)
2. ⚠️ **Start server** (`cargo run --bin server`)
3. ⚠️ **Hard refresh browser** (Cmd+Shift+R)
4. ⚠️ **Open browser console** (F12)
5. ✅ **Click "√ Square Root"**
6. ✅ **Check both logs** (browser console + server terminal)
7. ✅ **Report what you see** in the debug output

The enhanced debug logging will show us exactly what's happening! 🔍
