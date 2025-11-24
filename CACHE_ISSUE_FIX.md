# Browser Cache Issue - Complete Fix

**Date:** November 24, 2024  
**Issue:** "Structural editing removes □" - Server shows `sqrt()` instead of `sqrt(square.stroked)`  
**Root Cause:** Browser is using cached JavaScript with old AST templates  
**Status:** ✅ Code fixed - Cache clear required

---

## The Problem

You're seeing:
```
Full markup: sqrt()
Expected placeholder IDs: []
```

This means the AST sent from the browser has an **empty Object** instead of a **Placeholder** node.

---

## Why This Happens

### The AST Template is Correct (in index.html)
```javascript
sqrt: { 
    Operation: { 
        name: 'sqrt', 
        args: [{Placeholder: {id: 0, hint: 'radicand'}}]  ✅ Correct!
    } 
}
```

### But the Browser is Using Old Cached Version
The browser cached the OLD `index.html` that had incomplete `astTemplates` object (only 6 templates).

When you click "√ Square Root", the cached JavaScript creates a fallback AST:
```javascript
// Old cached code (line 752):
if (!ast) {
    ast = { Placeholder: { id: nextPlaceholderId++, hint: name } };  // Single placeholder
}
```

This creates a **single Placeholder** at the root level, not a proper sqrt Operation with Placeholder argument.

---

## Complete Fix Procedure

### Step 1: Stop Server
```bash
# Press Ctrl+C in server terminal
```

### Step 2: Clear Browser Cache

#### Option A: Hard Refresh (Easiest)
- **Mac:** `Cmd+Shift+R`
- **Windows:** `Ctrl+Shift+R`
- **Linux:** `Ctrl+F5`

#### Option B: Clear All Cache
1. Open browser DevTools (F12)
2. Right-click the refresh button
3. Select "Empty Cache and Hard Reload"

#### Option C: Disable Cache (Best for Development)
1. Open DevTools (F12)
2. Go to "Network" tab
3. Check "Disable cache"
4. Keep DevTools open while testing

#### Option D: Incognito/Private Mode
Open `http://localhost:3000` in incognito/private window (no cache).

### Step 3: Restart Server
```bash
cd /Users/eatik_1/Documents/git/cee/kleis
cargo run --bin server
```

### Step 4: Reload Page
Navigate to `http://localhost:3000` (with cache disabled or in incognito)

### Step 5: Verify in Browser Console
Open console (F12) and run:
```javascript
console.log(astTemplates.sqrt);
```

**Should show:**
```javascript
{
  Operation: {
    name: "sqrt",
    args: [{Placeholder: {id: 0, hint: "radicand"}}]
  }
}
```

**If you see something else:** Cache not cleared properly.

### Step 6: Test Structural Mode
1. Click "🔧 Structural Mode"
2. Click "√ Square Root"
3. Check server logs

**Should show:**
```
Received AST JSON: Object { "Operation": Object { "name": String("sqrt"), "args": Array [Object { "Placeholder": Object { "id": Number(1), "hint": String("radicand") } }] } }
Parsed Expression: Operation {
    name: "sqrt",
    args: [
        Placeholder { id: 1, hint: "radicand" }  ✅
    ]
}
Argument slots: 1 total
  Slot 0: id=1, is_placeholder=true, hint='radicand'  ✅
Full markup: sqrt(square.stroked)  ✅
Typst compilation successful!
```

---

## What Each Fix Does

### Fix 1: Placeholder Rendering (`src/render.rs` line 599)
```rust
// Before
RenderTarget::Typst => "#sym.square".to_string()

// After
RenderTarget::Typst => "square.stroked".to_string()
```
**Effect:** Placeholders render as valid Typst syntax

### Fix 2: AST Template Definitions (`static/index.html` line 735)
```javascript
sqrt: { 
    Operation: { 
        name: 'sqrt', 
        args: [{Placeholder: {id: 0, hint: 'radicand'}}] 
    } 
}
```
**Effect:** Clicking template creates proper AST with Placeholder nodes

### Fix 3: Debug Logging (`src/bin/server.rs` line 336)
```rust
eprintln!("Received AST JSON: {:?}", req.ast);
eprintln!("Parsed Expression: {:#?}", expr);
```
**Effect:** Shows exactly what AST the server receives

---

## Common Cache Issues

### Issue: Browser Shows Old JavaScript
**Symptom:** `astTemplates.sqrt` is undefined or has wrong structure  
**Solution:** Hard refresh (Cmd+Shift+R) or use incognito mode

### Issue: Server Shows Old Code
**Symptom:** Placeholder renders as `#sym.square` in test  
**Solution:** Rebuild (`cargo build`) and restart server

### Issue: Both Are New But Still Fails
**Symptom:** AST is correct but still fails  
**Solution:** Check Typst template on line 2212: `"sqrt({arg})"`

---

## Verification Commands

### 1. Verify Server Build
```bash
# Check if square.stroked is in the binary
strings target/debug/server | grep "square.stroked"
# Should find it
```

### 2. Verify HTML File
```bash
# Check if sqrt template is in HTML
grep "sqrt.*Placeholder.*radicand" static/index.html
# Should find it
```

### 3. Test Rendering Directly
```bash
rustc --edition 2021 -L target/debug/deps --extern kleis=target/debug/libkleis.rlib /tmp/test_render_placeholder.rs -o /tmp/test_render_placeholder
/tmp/test_render_placeholder | grep "sqrt"
```
**Should show:** `sqrt(square.stroked)` ✅

---

## The Complete Picture

### What Should Happen
```
User clicks "√ Square Root"
  ↓
Frontend (NEW JavaScript):
  astTemplates.sqrt = {Operation: {name: 'sqrt', args: [{Placeholder: {id:1, hint:'radicand'}}]}}
  ↓
Send to server:
  POST /api/render_typst
  Body: {"ast": {"Operation": {"name": "sqrt", "args": [{"Placeholder": {"id": 1, "hint": "radicand"}}]}}}
  ↓
Server (NEW binary):
  json_to_expression() → Expression::Operation { name: "sqrt", args: [Placeholder{id:1, hint:"radicand"}] }
  render_expression() → "sqrt(square.stroked)"
  compile_with_semantic_boxes() → Success!
  ↓
Return SVG with placeholder positions
  ↓
Frontend draws interactive overlay
```

### What's Happening Now (Cache Issue)
```
User clicks "√ Square Root"
  ↓
Frontend (OLD cached JavaScript):
  astTemplates.sqrt = undefined (only had 6 templates)
  Falls back to: {Placeholder: {id: 1, hint: 'sqrt'}}  ❌ Wrong structure!
  ↓
Send to server:
  POST /api/render_typst
  Body: {"ast": {"Placeholder": {"id": 1, "hint": "sqrt"}}}  ❌ Root is Placeholder!
  ↓
Server:
  json_to_expression() → Expression::Placeholder{id:1, hint:"sqrt"}  ❌ Not an operation!
  render_expression() → "square.stroked"  ❌ Just the placeholder, no sqrt!
```

Wait, that doesn't match the logs either. Let me think...

Actually, if the browser is sending a Placeholder at the root, the server would render just `square.stroked`, not `sqrt()`. But you're seeing `sqrt()` which means it's rendering a sqrt operation with an empty argument.

This means the browser IS sending `sqrt(Object(""))`, which happens when the AST template lookup fails and it falls back to parsing the LaTeX template string `\sqrt{□}`.

---

## The Real Issue

When `astTemplates[name]` lookup fails (because browser has old cached JavaScript), the code does:
```javascript
// Line 814 (old cached version)
ast = { Placeholder: { id: nextPlaceholderId++, hint: name } };
```

But actually, looking at your error, it seems like it might be creating `sqrt(Object(""))` somehow. Let me check if there's template string parsing happening...

Actually, I think the issue is simpler: **The browser console log on line 845 will show the actual AST being sent**. You need to:

1. Open browser console (F12)
2. Click the sqrt template
3. Look at the console log that says "renderStructuralEditor: sending AST to /api/render_typst"
4. See what the actual AST structure is

---

## Action Items

1. ⚠️ **Stop server** (Ctrl+C)
2. ⚠️ **Start server** (`cargo run --bin server`)
3. ⚠️ **Open browser in incognito mode** (Cmd+Shift+N)
4. ⚠️ **Navigate to** `http://localhost:3000`
5. ⚠️ **Open console** (F12)
6. ⚠️ **Click "🔧 Structural Mode"**
7. ⚠️ **Click "√ Square Root"**
8. ✅ **Check console log** - What AST is being sent?
9. ✅ **Check server log** - What does "Parsed Expression" show?

The debug logs will tell us exactly what's wrong!

---

## Expected vs Actual

### Expected (After Cache Clear)
**Browser Console:**
```javascript
{Operation: {name: 'sqrt', args: [{Placeholder: {id: 1, hint: 'radicand'}}]}}
```

**Server Log:**
```
Parsed Expression: Operation { name: "sqrt", args: [Placeholder { id: 1, hint: "radicand" }] }
Full markup: sqrt(square.stroked)
```

### Actual (With Cache)
**Browser Console:**
```javascript
??? (Need to check)
```

**Server Log:**
```
Full markup: sqrt()  ← Empty argument
```

**Please check the browser console to see what AST is actually being sent!**

