# Enhanced Debug Guide - Structural Mode Issue

**Date:** November 24, 2024  
**Issue:** AST has `Object("")` instead of `Placeholder` node  
**Status:** Enhanced logging added to both frontend and backend

---

## What We Know

From your server logs:
```
Received AST JSON: {"Operation": {"name": "sqrt", "args": [{"Object": ""}]}}
                                                             ^^^^^^^^^^^^^^
                                                             Should be: {"Placeholder": {"id": 1, "hint": "radicand"}}

Parsed Expression: Operation { name: "sqrt", args: [Object("")] }
                                                     ^^^^^^^^^^^
                                                     Should be: Placeholder { id: 1, hint: "radicand" }
```

The browser is sending an **empty Object** instead of a **Placeholder** node!

---

## Enhanced Debugging Added

### Frontend Logging (static/index.html)
Added console.log statements to track:
- Template name lookup
- AST template retrieval
- AST cloning
- Placeholder renumbering
- Final AST structure

### Backend Logging (src/bin/server.rs)
Added eprintln statements to show:
- Raw JSON received
- Parsed Expression structure
- Argument slot details (id, is_placeholder, hint)

---

## Testing Procedure

### Step 1: Restart Server
```bash
# Stop current server (Ctrl+C)
cargo run --bin server
```

### Step 2: Open Browser Console
1. Open browser (use incognito: Cmd+Shift+N)
2. Navigate to `http://localhost:3000`
3. Open DevTools (F12)
4. Go to "Console" tab
5. Clear console

### Step 3: Test Square Root Template
1. Click "🔧 Structural Mode"
2. Click "√ Square Root" button
3. Watch the console output

### Step 4: Analyze Logs

**Browser Console will show:**
```
insertStructuralTemplate called with: \sqrt{□}
Mapped to template name: sqrt
AST template before clone: {Operation: {...}}
AST after clone: {Operation: {...}}
renumberPlaceholders called on: {Operation: {...}}
  Processing operation: sqrt with 1 args
renumberPlaceholders called on: ??? ← THIS IS THE KEY!
  Found Placeholder/Object/Const: ???
AST after renumber: {Operation: {...}}
Final currentAST: {Operation: {...}}
renderStructuralEditor: sending AST to /api/render_typst {Operation: {...}}
```

**Server Terminal will show:**
```
=== render_typst_handler called ===
Received AST JSON: {...}
Parsed Expression: Operation { name: "sqrt", args: [...] }
Argument slots: 1 total
  Slot 0: id=???, is_placeholder=true/false, hint='???'
```

---

## Diagnosis Scenarios

### Scenario A: Console Shows "No AST template found"
**Meaning:** `astTemplates[name]` lookup failed  
**Cause:** Browser cache - old JavaScript without new templates  
**Solution:** Hard refresh or incognito mode

### Scenario B: Console Shows "Found Object" Instead of Placeholder
**Meaning:** AST template has `{Object: ""}` instead of `{Placeholder: {...}}`  
**Cause:** Typo in astTemplates definition or wrong structure  
**Solution:** Check line 735 in index.html

### Scenario C: Console Shows Placeholder, Server Shows Object
**Meaning:** JSON serialization issue  
**Cause:** Placeholder not being serialized correctly  
**Solution:** Check JSON.stringify output

### Scenario D: Everything Looks Correct But Still Fails
**Meaning:** Something else is modifying the AST  
**Solution:** Check for other code that might transform the AST

---

## Quick Check: Is Cache the Issue?

Run this in browser console:
```javascript
// Check if astTemplates has sqrt
console.log('sqrt' in astTemplates);  // Should be true

// Check sqrt structure
console.log(astTemplates.sqrt);
// Should show: {Operation: {name: 'sqrt', args: [{Placeholder: {id: 0, hint: 'radicand'}}]}}

// Check if it's the old version (only 6 templates)
console.log(Object.keys(astTemplates).length);
// Should be: 54 (not 6!)
```

**If it shows 6 or sqrt is undefined:** Browser cache issue - use incognito mode!

---

## Expected Correct Flow

```
1. User clicks "√ Square Root"
   ↓
2. insertStructuralTemplate('\sqrt{□}')
   ↓
3. name = templateMap['\sqrt{□}'] = 'sqrt'  ✅
   ↓
4. ast = astTemplates['sqrt'] = {Operation: {name: 'sqrt', args: [{Placeholder: {id: 0, hint: 'radicand'}}]}}  ✅
   ↓
5. ast = JSON.parse(JSON.stringify(ast))  // Clone
   ↓
6. renumberPlaceholders(ast)
   - Walks to Operation.args[0]
   - Finds Placeholder
   - Updates id: 0 → 1  ✅
   ↓
7. currentAST = {Operation: {name: 'sqrt', args: [{Placeholder: {id: 1, hint: 'radicand'}}]}}  ✅
   ↓
8. Send to server
   ↓
9. Server receives: {"Operation": {"name": "sqrt", "args": [{"Placeholder": {"id": 1, "hint": "radicand"}}]}}  ✅
   ↓
10. render_expression() → "sqrt(square.stroked)"  ✅
    ↓
11. Typst compiles successfully!  ✅
```

---

## Actual Broken Flow (What's Happening)

```
1. User clicks "√ Square Root"
   ↓
2. insertStructuralTemplate('\sqrt{□}')
   ↓
3. name = templateMap['\sqrt{□}'] = 'sqrt'  ✅
   ↓
4. ast = astTemplates['sqrt'] = ??? (undefined or wrong structure)  ❌
   ↓
5. Falls back OR gets wrong structure
   ↓
6. currentAST = {Operation: {name: 'sqrt', args: [{Object: ""}]}}  ❌
   ↓
7. Send to server
   ↓
8. Server receives: {"Operation": {"name": "sqrt", "args": [{"Object": ""}]}}  ❌
   ↓
9. render_expression(Object("")) → ""  (empty string)
   ↓
10. Full markup: "sqrt()"  ❌
    ↓
11. Typst fails: "missing argument"  ❌
```

---

## The Fix

The browser console logs will show exactly where it breaks. Most likely:
- `astTemplates.sqrt` is undefined (cache issue)
- OR `astTemplates.sqrt` has wrong structure (code issue)

---

## Action Required

**Restart server and test with console open:**

```bash
# 1. Stop server
Ctrl+C

# 2. Start server
cargo run --bin server

# 3. Open INCOGNITO browser window
Cmd+Shift+N (Mac) or Ctrl+Shift+N (Windows)

# 4. Go to http://localhost:3000

# 5. Open console (F12) and keep it visible

# 6. Click "🔧 Structural Mode"

# 7. Click "√ Square Root"

# 8. READ THE CONSOLE LOGS
```

The console will show:
- What template name is looked up
- What AST template is found
- What the structure is at each step
- Where it becomes `{Object: ""}`

**Then we'll know exactly what to fix! 🔍**

