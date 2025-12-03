# ✨ Inline Editing - User Guide

## 🎉 Feature Now Available!

**Inline editing** is now live in Structural Mode! You can type directly at marker positions, just like a text editor.

---

## How to Use

### Basic Workflow

1. **Switch to Structural Mode** (click "🔧 Structural Mode" button)
2. **Click a template** (e.g., "a + b")
3. **Click a placeholder** (green box) - **WITHOUT holding Shift/Ctrl**
4. ✨ **An input field appears right at the marker!**
5. **Type directly** (e.g., "x")
6. **Press Enter** to commit

Result: Your value is inserted and rendered! 🎯

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Enter** | Commit value and close inline editor |
| **ESC** | Cancel editing (discards input) |
| **Tab** | Commit and move to next placeholder (future) |
| Click outside | Auto-commit value |

---

## Symbol Buttons

While inline editor is active, you can **click symbol buttons** to insert them:

### Example: Build `α + β`

1. Click **"+"** template
2. Click first placeholder → Inline editor appears
3. Click **"α"** button → Appends to input
4. Press Enter → First arg becomes "α"
5. Click second placeholder → Inline editor appears  
6. Click **"β"** button → Appends to input
7. Press Enter → Complete! Shows: `α + β`

### Symbols You Can Click:

- ✅ **Greek letters**: α, β, γ, δ, θ, λ, μ, ν, π, σ, τ, φ, ψ, ω, Γ, Δ, Ω, etc.
- ✅ **Operators**: +, −, ×, ÷, ·, ∗, =, ≠, ±
- ✅ **Logic symbols**: ∀, ∃, ∈, ⊂, ∪, ∩, →, ⇒
- ✅ **Special**: ∞, ∂, ∇

All symbols with a **green tint** during inline editing are safe to click!

---

## Template Buttons

If you click a **template button** (like fraction, matrix, etc.) while inline editing:

### If Input is Empty:
- Template inserts immediately ✅
- No confirmation needed

### If You've Typed Something:
- **Confirmation dialog appears**: "Replace 'x' with template?"
- **"Insert Template"** → Replaces your text with the template
- **"Keep Typing"** → Returns to inline editor, preserves your text

Template buttons show **orange dashed borders** during inline editing!

---

## Advanced: Dialog Mode (Power Users)

**Hold Shift or Ctrl** while clicking a placeholder to open the old dialog:

- **Shift+Click** OR **Ctrl+Click** → Shows popup dialog
- Useful for:
  - Copying/pasting long expressions
  - Reviewing before committing
  - Old-school workflow preference

---

## Visual Indicators

### Button States During Inline Editing

| Button Type | Visual | Behavior |
|-------------|--------|----------|
| **Symbols** | Green tint | Appends to input ✅ |
| **Templates** | Orange dashed border | Shows confirmation ⚠️ |
| **Functions** | Orange dashed border | Shows confirmation ⚠️ |

### Marker States

| State | Color | Meaning |
|-------|-------|---------|
| Empty placeholder | Blue dashed box | Not filled yet |
| Filled value | Green box | Has content |
| **Editing inline** | **Green with thick border** | **Currently editing** |
| Selected (inactive) | Highlighted | Ready for action |

---

## Examples

### Example 1: Simple Variable
```
1. Click "+" template
2. Click left placeholder
3. Type "x"
4. Press Enter
5. Click right placeholder
6. Type "y"
7. Press Enter
Result: x + y ✅
```

### Example 2: Greek Letters
```
1. Click "+" template
2. Click left placeholder
3. Click "θ" button
4. Press Enter
5. Click right placeholder
6. Click "φ" button
7. Press Enter
Result: θ + φ ✅
```

### Example 3: Mixed Symbols
```
1. Click "+" template
2. Click left placeholder
3. Type "2"
4. Click "π" button
5. Input shows: "2π"
6. Press Enter
Result: First arg is "2π" ✅
```

### Example 4: Nested Templates
```
1. Click "( )" button
2. Click the placeholder inside
3. Click "fraction" button (template)
4. Since input is empty, fraction inserts immediately
5. Fill fraction numerator and denominator
Result: (a/b) ✅
```

### Example 5: Cancel During Typing
```
1. Click placeholder
2. Type "xyz"
3. Press ESC
Result: Editing cancelled, marker stays empty ✅
```

---

## Tips & Tricks

### Tip 1: Fast Symbol Entry
Click multiple symbol buttons in sequence:
- Click α → β → γ → δ
- Input shows: "αβγδ"
- Press Enter to commit all at once

### Tip 2: Combining Typed + Symbols
- Type "2"
- Click "π" button
- Type "r"
- Result: "2πr" in one expression

### Tip 3: Template Replacement
If you start typing then realize you need a template:
1. Type "x"
2. Click "fraction" button
3. Dialog asks: "Replace 'x' with template?"
4. Choose based on what you want!

### Tip 4: Quick Commit
- Click outside the editor to auto-commit
- No need to press Enter if you're done

---

## Troubleshooting

### Problem: Inline editor doesn't appear
**Solution:** Make sure you're in Structural Mode and clicked a placeholder WITHOUT holding Shift/Ctrl

### Problem: Template button does nothing
**Solution:** If inline editor is active with text, you need to confirm replacement in the dialog

### Problem: Symbols appear as LaTeX commands
**Solution:** This is correct - they render properly when committed. E.g., `\alpha` becomes `α` after rendering

### Problem: Can't see the input field
**Solution:** Make sure overlay visibility is enabled (checkbox should be checked)

---

## What Changed from Before?

### Old Behavior (v2.0):
```
Click marker → Modal dialog pops up → Type → Click OK
(4 actions, interrupts flow)
```

### New Behavior (v2.1):
```
Click marker → Type directly → Press Enter
(2 actions, natural flow)
```

### Backwards Compatible:
```
Shift+Click marker → Dialog still works!
(For power users who prefer it)
```

---

## Browser Compatibility

- ✅ **Chrome/Edge** - Full support
- ✅ **Firefox** - Full support
- ✅ **Safari** - Full support
- ⚠️ **Mobile** - May fall back to dialog (acceptable)
- ❌ **IE11** - Not supported (use dialog mode)

---

## Performance

- ✅ Instant inline editor appearance
- ✅ Button classification on page load (~5ms for 137 buttons)
- ✅ No lag during typing
- ✅ Render after commit (~20-50ms depending on complexity)

---

## Feedback Welcome!

This is a brand new feature (v2.1). If you find any issues or have suggestions:
1. Check browser console for errors (F12)
2. Try Shift+Click to use old dialog mode
3. Report issue with steps to reproduce

---

**Status:** ✅ **Live and Ready to Use!**  
**URL:** http://localhost:3000  
**Mode:** Structural Mode  
**Version:** v2.1-inline-editing

**Enjoy the natural editing experience!** ✨

