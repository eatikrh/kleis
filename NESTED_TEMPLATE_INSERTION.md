# Nested Template Insertion with Undo/Redo

**Date:** November 24, 2024  
**Status:** ✅ Implemented  
**Version:** 2.2

---

## New Features

### 1. Insert Templates at Edit Markers
Click an edit marker, then click a palette button to insert a template at that position.

### 2. Undo/Redo Support
- **Undo:** Cmd+Z (Mac) or Ctrl+Z (Windows)
- **Redo:** Cmd+Shift+Z (Mac) or Ctrl+Shift+Z (Windows)
- **Buttons:** ↶ Undo and ↷ Redo buttons in structural controls
- **History:** Up to 50 actions

---

## How to Use

### Building Nested Expressions

**Example: Build `√(a²/b)`**

1. **Start:** Click "√ Square Root" → Shows `√□`
2. **Click the □ marker** → Marker highlights in red, pulses
3. **Click "📐 Fraction"** → Becomes `√(□/□)`
4. **Click numerator □** → Highlights
5. **Click "x^n Power"** → Becomes `√(□²/□)`
6. **Click the □ in power** → Highlights
7. **Type "a"** in prompt → Becomes `√(a²/□)`
8. **Click denominator □** → Highlights
9. **Type "b"** → Final: `√(a²/b)` ✅

### Using Undo/Redo

**Made a mistake?**
- Press **Cmd+Z** to undo last action
- Press **Cmd+Shift+Z** to redo
- Or click **↶ Undo** / **↷ Redo** buttons

**Each action is saved:**
- Template insertion
- Value entry
- Template replacement

**You can undo up to 50 actions!**

---

## User Flow

### Method 1: Direct Replacement
1. Click template button (no marker selected)
2. Replaces entire expression
3. **Use for:** Starting new expression

### Method 2: Nested Insertion
1. Click edit marker → Marker highlights (red, pulsing)
2. Status shows: "📍 Marker selected. Click a template to insert."
3. Click palette button → Template inserted at marker
4. **Use for:** Building nested expressions

### Method 3: Simple Value Entry
1. Click edit marker
2. Type value in prompt
3. Press OK → Value inserted
4. **Use for:** Simple values (numbers, variables)

---

## Visual Feedback

### Edit Marker States

**Normal (not selected):**
- Blue/green dashed border
- Subtle hover effect

**Active (selected):**
- **Red solid border**
- **Pulsing animation**
- Clearly indicates where next action will apply

**After insertion:**
- Marker disappears (replaced with content)
- New markers appear for new placeholders

---

## Keyboard Shortcuts

**In Structural Mode:**
- **Cmd+Z** / **Ctrl+Z** - Undo
- **Cmd+Shift+Z** / **Ctrl+Shift+Z** - Redo
- **Tab** - Navigate between markers (existing feature)
- **Enter** - Edit marker (existing feature)

---

## Examples

### Example 1: Fraction with Powers

**Goal:** `(x²+y²)/(x²-y²)`

1. Click "Fraction" → `□/□`
2. Click numerator → Highlights
3. Click "+" (from operators) → `(□+□)/□`
4. Click first □ in sum → Highlights
5. Click "Power" → `(□²+□)/□`
6. Click □ in power, type "x" → `(x²+□)/□`
7. Click second □ in sum → Highlights
8. Click "Power" → `(x²+□²)/□`
9. Click □, type "y" → `(x²+y²)/□`
10. Continue for denominator...

**With undo:** Any mistake? Just Cmd+Z!

### Example 2: Nested Integrals

**Goal:** `∫₀¹ √(1-x²) dx`

1. Click "Integral" → `∫□□ □ dx` (bounds and integrand)
2. Click lower bound □, type "0"
3. Click upper bound □, type "1"
4. Click integrand □ → Highlights
5. Click "Square Root" → `∫₀¹ √□ dx`
6. Click □ under root → Highlights
7. Click "-" operator → `∫₀¹ √(□-□) dx`
8. Continue building `1-x²`...

---

## Implementation Details

### Undo Stack
```javascript
undoStack = [
    {Operation: {name: 'sqrt', args: [...]}},  // State 1
    {Operation: {name: 'scalar_divide', ...}}, // State 2
    // ... up to 50 states
]
```

### Active Marker
```javascript
activeEditMarker = {
    id: 0,           // Placeholder ID
    path: [0, 1],    // Path in AST tree
    nodeId: '0.0.1'  // Node identifier
}
```

### Template Insertion
```javascript
// Get template AST
let templateAST = astTemplates['fraction'];

// Clone and renumber placeholders
templateAST = JSON.parse(JSON.stringify(templateAST));
renumberPlaceholders(templateAST);

// Insert at active marker's path
setNodeAtPath(currentAST, activeEditMarker.path, templateAST);

// Re-render
renderStructuralEditor();
```

---

## Benefits

✅ **Intuitive** - Click marker, click template  
✅ **Powerful** - Build arbitrarily complex nested expressions  
✅ **Safe** - Undo any mistake  
✅ **Visual** - Clear feedback on active marker  
✅ **Reuses existing UI** - No new palette needed  
✅ **Keyboard accessible** - Cmd+Z/Cmd+Shift+Z  

---

## Testing

**Test nested insertion:**
1. Start with fraction
2. Insert sqrt in numerator
3. Insert power in sqrt
4. Verify structure is correct
5. Test undo - should step back through each action
6. Test redo - should step forward

**Test undo/redo:**
1. Build complex expression with 5-6 steps
2. Undo all the way back
3. Redo all the way forward
4. Verify expression matches

---

## Known Limitations

**Current:**
- Prompt dialog still used for simple values (will be replaced with modal)
- No visual undo/redo history viewer
- No branch/merge of undo states

**Future:**
- Replace prompt with inline editor
- Add undo history panel
- Add "undo to this point" feature

---

## User Guide Summary

**To build nested expressions:**
1. Click edit marker (highlights in red)
2. Click palette button (template inserted)
3. Repeat for nested structure
4. Use Cmd+Z to undo mistakes

**To enter simple values:**
1. Click edit marker
2. Type value in prompt
3. Press OK

**The palette is now your construction toolkit for building complex mathematical expressions interactively! 🎨**

