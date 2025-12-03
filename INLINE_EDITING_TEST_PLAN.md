# Inline Editing - Test Plan

## Test the implementation at: http://localhost:3000

## Quick Test Checklist

### ✅ Test 1: Basic Inline Editing
1. Switch to **Structural Mode**
2. Click **"+"** template
3. Click first placeholder (WITHOUT Shift/Ctrl)
4. Should see inline input field appear at marker location ✅
5. Type "x"
6. Press Enter
7. Should show: x + □

### ✅ Test 2: Symbol Button During Inline Edit
1. Click second placeholder
2. Should see inline input field
3. Click **"α"** button from Greek palette
4. Should append "α" to input field ✅
5. Click **"β"** button
6. Should now show "αβ" in input ✅
7. Press Enter
8. Should show: x + αβ

### ✅ Test 3: Template Button with Empty Input
1. Click **"+"** template
2. Click first placeholder
3. Inline editor appears (empty)
4. Click **"fraction"** button
5. Should immediately insert fraction (no confirmation needed) ✅
6. Should show: (a/b) + □

### ✅ Test 4: Template Button with Typed Text
1. Click **"+"** template
2. Click first placeholder
3. Type "x"
4. Click **"fraction"** button
5. Should show confirmation dialog: "Replace 'x' with template?" ✅
6. Click "Keep Typing"
7. Should return to input with "x" still there ✅
8. Press ESC
9. Click **"fraction"** button again
10. Click "Insert Template"
11. Should insert fraction ✅

### ✅ Test 5: Keyboard Shortcuts
1. Click marker → Inline editor appears
2. Type "test"
3. Press **ESC** → Should cancel, marker stays empty ✅
4. Click marker again
5. Type "x"
6. Press **Enter** → Should commit "x" ✅
7. Click next marker
8. Type "y"
9. Press **Tab** → Should commit "y" and try to move to next ✅

### ✅ Test 6: Modifier Key for Dialog
1. **Shift+Click** a placeholder
2. Should show the old prompt dialog ✅
3. Type value and click OK
4. Should work as before (backwards compatible) ✅

### ✅ Test 7: Click Outside to Commit
1. Click marker → Inline editor appears
2. Type "z"
3. Click somewhere outside the editor (not on a button)
4. Should commit "z" automatically ✅

### ✅ Test 8: Nested Structure
1. Click **"( )"** (parentheses) template
2. Click the placeholder inside
3. Inline editor appears
4. Click **"[ ]"** (brackets) button
5. Should show confirmation (or insert if empty)
6. Result should be: ([□])

### ✅ Test 9: Operators Mix
1. Build: x + α - β
2. Steps:
   - Click "+" template
   - Click first placeholder
   - Type "x", press Enter
   - Click second placeholder
   - Click "α" button, click "-" button, click "β" button
   - Press Enter
3. Should show: x + α - β (all in second argument)

### ✅ Test 10: Button Visual Feedback
1. Click a placeholder → Inline editor active
2. Hover over **"α"** button → Should show green tint (symbol, safe)
3. Hover over **"fraction"** button → Should show orange dashed border (template, replaces) ✅

---

## Edge Cases to Test

### Edge 1: Empty Input + Enter
1. Click marker
2. Don't type anything
3. Press Enter
4. Should close editor without changing AST ✅

### Edge 2: Rapid Clicking
1. Click marker A
2. Immediately click marker B (before typing)
3. Should close editor on A, open on B ✅

### Edge 3: Template Button While Empty
1. Click marker
2. Don't type
3. Click fraction button
4. Should insert immediately (no confirmation) ✅

### Edge 4: Multiple Symbols
1. Click marker
2. Click α, β, γ, δ buttons in sequence
3. Input should show: αβγδ ✅
4. Press Enter
5. Should render correctly ✅

---

## Known Limitations

- Tab navigation to next placeholder not yet fully implemented (TODO)
- Live rendering (as-you-type) not yet implemented (Phase 2 optional)
- Autocomplete not yet implemented (Phase 3)
- Mobile/touch: May fall back to dialog (acceptable)

---

## Success Criteria

✅ Regular click shows inline editor  
✅ Shift/Ctrl+Click shows dialog (backwards compatible)  
✅ Symbol buttons append to input  
✅ Template buttons show confirmation if text typed  
✅ Template buttons insert immediately if input empty  
✅ Enter commits value  
✅ ESC cancels editing  
✅ Click outside commits value  
✅ Button visual feedback during inline editing  
✅ No console errors  
✅ Rendering works correctly after inline edits  

---

## If Tests Fail

Check browser console for errors:
1. Open DevTools (F12)
2. Look for JavaScript errors
3. Check if foreignObject was created
4. Check if event listeners are attached
5. Verify button classification logged correctly

---

**Test Status:** Ready for testing!  
**URL:** http://localhost:3000  
**Mode:** Structural Mode  
**Browser:** Chrome/Firefox/Safari recommended

