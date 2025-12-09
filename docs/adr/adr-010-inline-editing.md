# ADR-010: Inline Editing for Structural Mode

## Status
**Proposed** - Ready for implementation

## Context

Currently, clicking an edit marker in structural mode shows a popup dialog for entering values. This workflow is:
- ✅ Good for inserting templates (fractions, matrices, etc.)
- ❌ Disruptive for simple text entry (typing "x", "2", "α")
- ❌ Feels unnatural compared to text editors

## Proposal

### Two-Mode Interaction

1. **Regular Click** → **Inline editing** (new!)
   - Shows a text cursor at the marker position
   - Type directly like a text editor
   - ESC or click away to commit
   - Real-time or debounced rendering

2. **Shift+Click or Ctrl+Click** → **Popup dialog** (existing behavior)
   - For inserting complex templates
   - For multi-character symbols
   - Quick reference to available operations

## Decision

Implement inline editing with progressive enhancement:
- Phase 1: Basic inline text input at marker position
- Phase 2: Real-time rendering as you type
- Phase 3: Smart autocomplete (Greek letters, functions, etc.)

---

## Implementation Plan

### Phase 1: Basic Inline Editing (Essential)

#### 1.1 HTML Structure

Add an invisible text input that appears at marker position:

```html
<!-- In the SVG overlay layer -->
<foreignObject id="inline-editor" 
               x="0" y="0" 
               width="200" height="40" 
               style="display:none;">
    <input type="text" 
           id="inline-input"
           class="inline-edit-input"
           autocomplete="off"
           spellcheck="false" />
</foreignObject>
```

#### 1.2 CSS Styling

```css
.inline-edit-input {
    width: 100%;
    height: 100%;
    border: 2px solid #4CAF50;
    border-radius: 4px;
    padding: 4px 8px;
    font-family: 'Latin Modern Math', 'Cambria Math', serif;
    font-size: 18px;
    background: rgba(255, 255, 255, 0.95);
    box-shadow: 0 2px 8px rgba(76, 175, 80, 0.3);
    outline: none;
}

.inline-edit-input:focus {
    border-color: #4CAF50;
    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.4);
}
```

#### 1.3 Click Handler Logic

```javascript
function handleEditMarkerClick(event, marker) {
    const isModifierClick = event.shiftKey || event.ctrlKey || event.metaKey;
    
    if (isModifierClick) {
        // Show popup dialog (existing behavior)
        showEditDialog(marker);
    } else {
        // Show inline editor (new behavior)
        showInlineEditor(marker);
    }
}
```

#### 1.4 Inline Editor Functions

```javascript
function showInlineEditor(marker) {
    activeEditMarker = marker;
    
    // Position the foreignObject at the marker location
    const foreignObject = document.getElementById('inline-editor');
    const input = document.getElementById('inline-input');
    
    // Get marker position from bounding box
    foreignObject.setAttribute('x', marker.bbox.x - 10);
    foreignObject.setAttribute('y', marker.bbox.y - 5);
    foreignObject.setAttribute('width', Math.max(200, marker.bbox.width + 20));
    foreignObject.setAttribute('height', marker.bbox.height + 10);
    foreignObject.style.display = 'block';
    
    // Pre-fill with current value if any
    const currentValue = getNodeValueAtPath(currentAST, marker.path);
    input.value = currentValue || '';
    
    // Focus and select all
    setTimeout(() => {
        input.focus();
        input.select();
    }, 10);
    
    // Highlight the active marker
    marker.element.classList.add('editing-inline');
}

function hideInlineEditor(commit = true) {
    const foreignObject = document.getElementById('inline-editor');
    const input = document.getElementById('inline-input');
    
    if (commit && activeEditMarker) {
        const value = input.value.trim();
        if (value) {
            // Parse and insert the value
            const node = parseSimpleInput(value);
            setNodeAtPath(currentAST, activeEditMarker.path, node);
            renderStructuralEditor();
            showStatus('✅ Value updated', 'success');
        }
    }
    
    // Clean up
    foreignObject.style.display = 'none';
    input.value = '';
    if (activeEditMarker && activeEditMarker.element) {
        activeEditMarker.element.classList.remove('editing-inline');
    }
    activeEditMarker = null;
}
```

#### 1.5 Keyboard Handlers

```javascript
function setupInlineEditorKeyboard() {
    const input = document.getElementById('inline-input');
    
    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            hideInlineEditor(true); // Commit
        } else if (e.key === 'Escape') {
            e.preventDefault();
            hideInlineEditor(false); // Cancel
        } else if (e.key === 'Tab') {
            e.preventDefault();
            // Move to next placeholder
            hideInlineEditor(true);
            focusNextPlaceholder();
        }
    });
    
    // Click outside to commit
    document.addEventListener('click', (e) => {
        if (foreignObject.style.display !== 'none' && 
            !foreignObject.contains(e.target)) {
            hideInlineEditor(true);
        }
    });
}
```

---

### Phase 2: Live Rendering (Enhanced UX)

#### 2.1 Debounced Rendering

```javascript
let renderDebounceTimer = null;

input.addEventListener('input', (e) => {
    // Clear previous timer
    clearTimeout(renderDebounceTimer);
    
    // Debounce rendering (300ms after last keystroke)
    renderDebounceTimer = setTimeout(() => {
        const value = input.value.trim();
        if (value) {
            // Update AST and re-render
            const node = parseSimpleInput(value);
            setNodeAtPath(currentAST, activeEditMarker.path, node);
            renderStructuralEditor();
        }
    }, 300);
});
```

#### 2.2 Instant vs Debounced

```javascript
// Option A: Every keystroke (instant feedback)
renderDebounce = 0; // No delay

// Option B: Debounced (better performance for complex equations)
renderDebounce = 300; // 300ms delay

// Option C: Smart (short for simple, longer for complex)
renderDebounce = currentAST.depth > 5 ? 500 : 100;
```

---

### Phase 3: Smart Features (Future)

#### 3.1 Autocomplete

```javascript
// Detect LaTeX commands
if (input.value.startsWith('\\')) {
    showAutocomplete(input.value);
    // Shows: \alpha, \beta, \gamma, \sin, \cos, etc.
}
```

#### 3.2 Greek Letter Shortcuts

```javascript
// Type shortcuts for Greek letters
const shortcuts = {
    'alpha': '\\alpha',
    'beta': '\\beta',
    'gamma': '\\gamma',
    'theta': '\\theta',
    'sigma': '\\sigma',
    // etc.
};

// On space or tab, expand shortcuts
if (e.key === ' ' && shortcuts[input.value]) {
    input.value = shortcuts[input.value];
}
```

#### 3.3 Symbol Palette Dropdown

```javascript
// Ctrl+Space opens symbol picker at cursor
if (e.key === ' ' && e.ctrlKey) {
    showSymbolPicker(input.getBoundingClientRect());
}
```

---

## Technical Architecture

### Component Hierarchy

```
SVG Container
├── Math Rendering (read-only)
├── Edit Markers (clickable overlays)
└── Inline Editor (foreignObject)
    └── Input Field (HTML input element)
```

### State Management

```javascript
let editState = {
    mode: 'none' | 'inline' | 'dialog',
    activeMarker: null,
    inputBuffer: '',
    renderPending: false
};
```

### Event Flow

```
Click Marker (no modifier)
    ↓
Check current value
    ↓
Position foreignObject at marker
    ↓
Show input field
    ↓
User types
    ↓
[Debounced] Parse input → Update AST → Re-render
    ↓
Enter/Tab → Commit → Hide editor → Move to next
    ↓
ESC → Cancel → Restore original → Hide editor
```

---

## Challenges & Solutions

### Challenge 1: SVG Coordinate Mapping

**Problem:** Input field position must align with marker overlay.

**Solution:** Use `foreignObject` with same coordinate system as overlays:
```javascript
foreignObject.setAttribute('x', marker.bbox.x - 10);
foreignObject.setAttribute('y', marker.bbox.y - 5);
```

### Challenge 2: Rendering Performance

**Problem:** Re-rendering entire equation on every keystroke could be slow.

**Solution:** 
- Debounce rendering (300ms)
- Only update the specific node, not entire AST
- Use optimistic UI update (show typed text immediately, render later)

### Challenge 3: Focus Management

**Problem:** Click on SVG loses focus from input field.

**Solution:**
- Use `stopPropagation()` on input clicks
- Commit on outside click (blur event)
- Clear visual feedback when editing

### Challenge 4: Multi-line Input

**Problem:** Some expressions span multiple lines (matrices, fractions).

**Solution:**
- For complex operations, keep popup dialog
- Inline editor only for leaf nodes (Const, Object, Placeholder)
- Shift+click always opens dialog for power users

### Challenge 5: Template vs Symbol Button Integration

**Problem:** When inline editor is active, what happens when user clicks palette buttons?

**Two types of buttons:**
1. **Symbol buttons** (α, β, +, ∞) - Can be typed as text → Append to input
2. **Template buttons** (fractions, matrices, integrals) - Cannot be text → Replace with AST

**Solution: Smart Button Handling**

```javascript
function handlePaletteButtonClick(buttonType, value) {
    if (!isInlineEditorActive()) {
        // Normal behavior
        if (buttonType === 'symbol') {
            insertSymbol(value);
        } else {
            insertTemplate(value);
        }
        return;
    }
    
    // Inline editor is active
    if (buttonType === 'symbol') {
        // Append symbol to inline input
        appendToInlineEditor(value);
    } else if (buttonType === 'template') {
        // Close inline editor, insert template at marker
        const marker = activeEditMarker;
        hideInlineEditor(false); // Don't commit text
        insertStructuralTemplateAt(value, marker.path);
    }
}
```

**Visual feedback:**
- Symbol buttons: Normal appearance (can be used)
- Template buttons: Dimmed or marked with "Replaces content" tooltip when inline editor active

---

## Button Interaction During Inline Editing

### Button Classification

**Safe for Text Input (append to inline editor):**
- Greek letters: `α`, `β`, `γ`, `Γ`, `Δ`, `Ω`
- Operators: `+`, `-`, `×`, `÷`, `=`, `≠`
- Logic symbols: `∀`, `∃`, `∈`, `⊂`, `∪`, `∩`
- Special symbols: `∞`, `∂`, `∇`
- Numbers and letters (obviously)

**Requires AST Operation (replace marker with template):**
- Fractions: `\frac{□}{□}`
- Powers/Subscripts: `□^{□}`, `□_{□}`
- Integrals: `\int_{□}^{□}`
- Matrices: `\begin{bmatrix}...\end{bmatrix}`
- Functions: `\sin(□)`, `\cos(□)`
- Brackets: `(□)`, `[□]`, `{□}` ⚠️

### Implementation Strategy

#### Classify Buttons by Type

```javascript
function getButtonType(latexTemplate) {
    // Single symbols that can be typed
    const symbolPatterns = [
        /^\\[a-z]+$/, // \alpha, \beta, \gamma
        /^[+\-=×÷±∓·∗]$/, // operators
        /^\\(in|subset|cup|cap|forall|exists|infty|partial|nabla)$/, // logic/special
    ];
    
    for (const pattern of symbolPatterns) {
        if (pattern.test(latexTemplate)) {
            return 'symbol';
        }
    }
    
    // Everything else is a template
    return 'template';
}
```

#### Modify Button Click Handlers

```javascript
// Update insertSymbol() function
function insertSymbol(latex) {
    if (editorMode === 'structural') {
        if (isInlineEditorActive()) {
            // Append to inline input
            appendToInlineEditor(latex);
            return;
        }
        
        // Normal symbol insertion (existing code)
        if (activeEditMarker) {
            const symbolNode = { Const: latex };
            setNodeAtPath(currentAST, activeEditMarker.path, symbolNode);
            // ... rest of code
        }
    }
    // ... text mode code
}

// Update insertTemplate() function
function insertTemplate(template) {
    if (editorMode === 'structural') {
        if (isInlineEditorActive()) {
            // Close inline editor, insert template
            const marker = activeEditMarker;
            hideInlineEditor(false); // Discard text input
            
            // Confirm if user typed something
            const inputValue = document.getElementById('inline-input').value;
            if (inputValue.trim()) {
                if (!confirm('Replace typed text with template?')) {
                    return;
                }
            }
            
            // Insert template at marker location
            insertStructuralTemplateAt(template, marker.path);
            return;
        }
        
        // Normal template insertion (existing code)
        if (activeEditMarker) {
            insertStructuralTemplateAt(template, activeEditMarker.path);
        } else {
            insertStructuralTemplate(template);
        }
    }
    // ... text mode code
}
```

#### New Helper Function

```javascript
function appendToInlineEditor(text) {
    const input = document.getElementById('inline-input');
    
    // Insert at cursor position
    const start = input.selectionStart;
    const end = input.selectionEnd;
    const currentValue = input.value;
    
    input.value = currentValue.substring(0, start) + text + currentValue.substring(end);
    
    // Move cursor after inserted text
    const newPos = start + text.length;
    input.setSelectionRange(newPos, newPos);
    input.focus();
    
    // Trigger live rendering (if enabled)
    if (LIVE_RENDERING_ENABLED) {
        debouncedRenderInline();
    }
}

function isInlineEditorActive() {
    const foreignObject = document.getElementById('inline-editor');
    return foreignObject && foreignObject.style.display !== 'none';
}
```

### Visual Feedback

Add CSS to indicate button state when inline editor is active:

```css
/* When inline editor is active */
body.inline-editing .math-btn.symbol-type {
    /* Symbols are enabled - normal appearance */
}

body.inline-editing .math-btn.template-type {
    /* Templates will replace - visual indicator */
    border: 2px dashed #FF9800;
}

body.inline-editing .math-btn.template-type::before {
    content: '⚠';
    position: absolute;
    top: 2px;
    right: 2px;
    font-size: 10px;
    color: #FF9800;
}

body.inline-editing .math-btn.template-type[data-tooltip]::after {
    content: 'Replaces typed text';
}
```

### Error Handling

```javascript
function handleTemplateInInlineMode(template, marker) {
    const inputValue = document.getElementById('inline-input').value.trim();
    
    if (inputValue) {
        // User has typed something
        showConfirmDialog({
            title: 'Replace with template?',
            message: `You've typed "${inputValue}". Insert template instead?`,
            confirm: 'Insert Template',
            cancel: 'Keep Typing',
            onConfirm: () => {
                hideInlineEditor(false);
                insertStructuralTemplateAt(template, marker.path);
            },
            onCancel: () => {
                // Keep inline editor open
                document.getElementById('inline-input').focus();
            }
        });
    } else {
        // No text typed, just insert template
        hideInlineEditor(false);
        insertStructuralTemplateAt(template, marker.path);
    }
}
```

---

## User Experience

### Workflow Comparison

**Current (Dialog-based):**
```
Click marker → Popup → Type "x" → Click OK
(4 actions, modal interruption)
```

**Proposed (Inline):**
```
Click marker → Type "x" → Enter
(2 actions, no interruption)
```

**With Symbol:**
```
Click marker → Click "α" button → Done
(2 actions, instant)
```

**For Templates (Shift+Click):**
```
Shift+Click marker → Select template → Click OK
(3 actions, intentional for complex input)
```

---

## Implementation Phases

### Phase 1: Core Inline Editing (Week 1)
- ✅ Add foreignObject to SVG
- ✅ Position input at marker location
- ✅ Basic keyboard handlers (Enter, ESC, Tab)
- ✅ Commit on blur/outside click
- ✅ Update click handler to check modifiers

**Estimated effort:** 6-8 hours  
**Risk:** Low - mostly UI code

### Phase 2: Live Rendering (Week 2)
- ✅ Debounced AST update
- ✅ Re-render on input change
- ✅ Performance testing with complex equations
- ✅ Optimize for large ASTs

**Estimated effort:** 4-6 hours  
**Risk:** Medium - performance considerations

### Phase 3: Smart Features (Future)
- ⏳ Autocomplete for LaTeX commands
- ⏳ Greek letter shortcuts
- ⏳ Symbol picker dropdown
- ⏳ History/undo for inline edits

**Estimated effort:** 10-15 hours  
**Risk:** Low - optional enhancements

---

## Testing Strategy

### Unit Tests
- Click with/without modifiers
- Enter/ESC/Tab keyboard shortcuts
- Outside click commits
- Empty input handling
- Multi-character input
- Greek letters and symbols

### Integration Tests
- Build complex equation with inline editing
- Mix inline editing + template insertion
- Symbol buttons while inline editing
- Tab navigation between markers
- Nested structures

### Performance Tests
- Type rapidly in complex equation (50+ nodes)
- Measure render latency with debouncing
- Memory usage over long editing session

---

## Backwards Compatibility

### Keep Dialog Option
- Shift+Click still opens dialog
- Useful for:
  - Power users who prefer it
  - Template insertion at markers
  - Copy/paste large expressions
  - Mobile/touch interfaces (harder to inline edit)

### Gradual Migration
- Phase 1: Inline editing opt-in (feature flag)
- Phase 2: Inline editing default (dialog on Shift+Click)
- Phase 3: Remove feature flag after user testing

---

## Alternative Approaches Considered

### Option A: Contenteditable Div
**Pros:** Rich text editing, easier styling  
**Cons:** Hard to control, security issues, paste problems

### Option B: Canvas-based Text Input
**Pros:** Pixel-perfect positioning  
**Cons:** Complex implementation, accessibility issues

### Option C: Overlay Transparent Input
**Pros:** Simple, native input behavior  
**Cons:** Hard to position, coordinate sync issues

### ✅ Selected: foreignObject with Input
**Pros:** Native input, SVG coordinates, best of both worlds  
**Cons:** None significant

---

## Success Metrics

### User Experience
- ⬇️ Reduce clicks per edit: 4 → 2 (50% improvement)
- ⬇️ Time to enter simple value: 2s → 0.5s
- ⬆️ User satisfaction with editing flow

### Technical
- ✅ Render latency < 100ms after keystroke
- ✅ No visual jank during editing
- ✅ Works with all 60+ templates
- ✅ Tab navigation still smooth

---

## Workflow Examples

### Example 1: Simple Variable Entry
```
1. Click marker (no modifier) → Inline input appears
2. Type "x" → Shows "x" in input
3. Press Enter → Commits, renders {θ} 
```

### Example 2: Greek Letter via Button
```
1. Click marker → Inline input appears  
2. Click "α" button → Appends to input: "α"
3. Click "β" button → Appends to input: "αβ"
4. Press Enter → Commits, renders {αβ}
```

### Example 3: Template Insertion During Typing
```
1. Click marker → Inline input appears
2. Type "x" → Shows "x" in input
3. Click "fraction" button → Shows confirm dialog:
   "Replace 'x' with template?"
4a. Click "Insert Template" → Closes input, inserts fraction AST
4b. Click "Keep Typing" → Returns to input, can continue typing
```

### Example 4: Power User (Shift+Click)
```
1. Shift+Click marker → Dialog opens (existing behavior)
2. Select from dropdown or type
3. Click OK → Commits
```

### Example 5: Complex Expression Build
```
1. Click "+" template → Creates {□ + □}
2. Click first marker → Inline input appears
3. Type "x" → Shows "x"
4. Press Tab → Commits "x", moves to next marker
5. Click "sin" button → Replace with sin template
6. Click sin's argument marker → Inline input appears
7. Click "θ" button → Appends "θ"
8. Press Enter → Commits, renders {x + sin(θ)}
```

---

## Smart Button Behavior

### Button States Based on Context

| Context | Symbol Button | Template Button | Function Button |
|---------|---------------|-----------------|-----------------|
| **No marker selected** | Insert at root | Insert at root | Insert at root |
| **Marker selected (idle)** | Replace marker | Replace marker | Replace marker |
| **Inline editor active** | Append to input ✅ | Confirm & replace ⚠️ | Confirm & replace ⚠️ |
| **Dialog open** | Disabled | Disabled | Disabled |

### Visual Indicators

```css
/* Normal state */
.math-btn { border: 1.5px solid #e0e0e0; }

/* Symbol - safe to use during inline editing */
body.inline-editing .math-btn[data-button-type="symbol"] {
    border-color: #4CAF50;
    cursor: pointer;
}

/* Template - will replace inline content */
body.inline-editing .math-btn[data-button-type="template"] {
    border-color: #FF9800;
    border-style: dashed;
}

/* Function - will replace inline content */
body.inline-editing .math-btn[data-button-type="function"] {
    border-color: #FF9800;
    border-style: dashed;
}
```

### Button Type Detection

```javascript
// Classify all buttons on page load
function classifyPaletteButtons() {
    document.querySelectorAll('.math-btn').forEach(btn => {
        const template = btn.getAttribute('data-template') || 
                        btn.getAttribute('onclick').match(/'([^']+)'/)[1];
        
        const type = classifyButtonType(template);
        btn.setAttribute('data-button-type', type);
    });
}

function classifyButtonType(template) {
    // Symbols: Single character or simple LaTeX command
    if (/^[a-zA-Z0-9+\-=×÷±∓·∗∞]$/.test(template)) return 'symbol';
    if (/^\\(alpha|beta|gamma|delta|epsilon|zeta|eta|theta|iota|kappa|lambda|mu|nu|xi|pi|rho|sigma|tau|phi|chi|psi|omega)$/.test(template)) return 'symbol';
    if (/^\\(Gamma|Delta|Theta|Lambda|Xi|Pi|Sigma|Phi|Psi|Omega)$/.test(template)) return 'symbol';
    if (/^\\(in|notin|subset|subseteq|cup|cap|emptyset|to|Rightarrow|Leftrightarrow|forall|exists|neg|land|lor)$/.test(template)) return 'symbol';
    if (/^\\(leq|geq|approx|equiv|neq|infty|partial|nabla|times|div|cdot|pm|mp|ast)$/.test(template)) return 'symbol';
    
    // Functions: Need AST structure
    if (/^\\(sin|cos|tan|arcsin|arccos|arctan|ln|log|exp)\(/.test(template)) return 'function';
    
    // Templates: Complex structures
    if (/\\frac|\\sqrt|\\int|\\sum|\\prod|\\lim|\\begin|\\left/.test(template)) return 'template';
    if (/□\^|□_|□\^.*_/.test(template)) return 'template';
    
    // Default to template (safe)
    return 'template';
}
```

---

## Implementation Checklist

### Phase 1 Core (Essential)

- [ ] Add foreignObject HTML structure to SVG
- [ ] Add CSS for inline-edit-input
- [ ] Classify all palette buttons by type (symbol/template/function)
- [ ] Update handleEditMarkerClick() to check modifiers
- [ ] Implement showInlineEditor()
- [ ] Implement hideInlineEditor()
- [ ] Implement appendToInlineEditor() for symbol buttons
- [ ] Implement handleTemplateInInlineMode() with confirmation
- [ ] Add keyboard handlers (Enter, ESC, Tab)
- [ ] Add blur/outside-click handler
- [ ] Add visual feedback for button types during inline editing
- [ ] Test with simple values (numbers, variables)
- [ ] Test with Greek letters from symbol buttons
- [ ] Test template insertion during inline editing (with confirmation)
- [ ] Test modifier keys (Shift+Click for dialog)

### Phase 2 Live Rendering (Enhanced)

- [ ] Add input event listener
- [ ] Implement debounced rendering (300ms)
- [ ] Update AST on each keystroke
- [ ] Test performance with complex equations
- [ ] Add loading indicator during render
- [ ] Handle render errors gracefully
- [ ] Optimize debounce delay based on complexity

### Phase 3 Smart Features (Future)

- [ ] Autocomplete for \commands
- [ ] Greek letter shortcuts (alpha → α)
- [ ] Symbol picker on Ctrl+Space
- [ ] Recent values history
- [ ] Undo/redo for inline edits

---

## Code Structure

### New Files
- `static/js/inline-editor.js` - Inline editing logic (optional refactor)

### Modified Files
- `static/index.html` - Add foreignObject, CSS, event handlers
  - Lines ~200: CSS for .inline-edit-input
  - Lines ~800: HTML foreignObject structure
  - Lines ~1500: Click handler modification
  - Lines ~1900: Inline editor functions (250 lines)

### Estimated LOC
- HTML/CSS: ~100 lines
- JavaScript: ~300 lines
- Total: ~400 lines (manageable)

---

## Risks & Mitigation

### Risk 1: Performance Degradation
**Risk Level:** Medium  
**Impact:** Slow rendering during typing  
**Mitigation:**
- Debounce rendering (300ms)
- Only re-render if value changed
- Use requestAnimationFrame for smooth updates
- Add performance profiling

### Risk 2: Coordinate Sync Issues
**Risk Level:** Low  
**Impact:** Input appears at wrong position  
**Mitigation:**
- Use same coordinate system as overlays
- Test with zoomed/transformed SVG
- Add visual debugging mode

### Risk 3: Browser Compatibility
**Risk Level:** Low  
**Impact:** foreignObject not supported in old browsers  
**Mitigation:**
- Feature detection
- Fallback to dialog for unsupported browsers
- Only affects IE11 and older (acceptable)

### Risk 4: Mobile Touch Issues
**Risk Level:** Medium  
**Impact:** Small touch targets, keyboard overlap  
**Mitigation:**
- Larger touch targets on mobile
- Auto-scroll to keep input visible
- Virtual keyboard awareness
- Keep dialog option for mobile

---

## Alternative: Simpler Approach

If foreignObject proves problematic, use **absolute-positioned HTML input**:

```html
<input id="inline-editor" 
       class="inline-edit-absolute"
       style="position: absolute; display: none;" />
```

```javascript
// Position using SVG → screen coordinate conversion
const screenCoords = marker.element.getBoundingClientRect();
input.style.left = screenCoords.left + 'px';
input.style.top = screenCoords.top + 'px';
```

**Pros:** Simpler, better browser support  
**Cons:** Coordinate conversion needed, z-index issues

---

## Mockups

### Visual States

```
State 1: Idle
┌─────────────────┐
│   { θ + η }     │  ← Rendered equation
│   ┌─┐   ┌─┐     │  ← Edit markers (green boxes)
└─────────────────┘

State 2: Regular Click (Inline Edit)
┌─────────────────┐
│   { [____] + η }│  ← Input field replaces marker
│      ^ cursor    │  ← Type directly
└─────────────────┘

State 3: Shift+Click (Dialog)
┌─────────────────┐
│   { θ + η }     │
│                 │
│   ╔═══════════╗ │
│   ║ Enter:    ║ │  ← Modal dialog
│   ║ [_______] ║ │
│   ║ [OK][Cancel]║ │
│   ╚═══════════╝ │
└─────────────────┘
```

---

## Rollout Plan

### Step 1: Feature Flag (Week 1)
```javascript
const INLINE_EDITING_ENABLED = true; // Can toggle for testing
```

### Step 2: Beta Testing (Week 2)
- Enable for internal testing
- Gather feedback on UX
- Measure performance metrics
- Fix any edge cases

### Step 3: Production (Week 3)
- Enable by default
- Document in user guide
- Add to onboarding tutorial
- Monitor for issues

---

## Open Questions

1. **Autocomplete trigger:** Tab, Ctrl+Space, or automatic dropdown?
2. **Debounce delay:** 100ms (instant), 300ms (balanced), 500ms (conservative)?
3. **Symbol buttons:** Append to input or replace? (Probably append)
4. **Multi-line editing:** Dialog only, or allow in inline mode?
5. **Undo/Redo:** Native browser undo or custom implementation?

---

## Recommendation

**Implement Phase 1 immediately** - It's straightforward, low-risk, and provides immediate UX improvement.

**Key advantages:**
- ✅ Natural text editor feel
- ✅ Fewer clicks per edit (50% reduction)
- ✅ Backwards compatible (Shift+Click for dialog)
- ✅ Works with existing symbol buttons
- ✅ Manageable scope (~400 LOC)

**Start with:** Basic inline editing + debounced rendering (Phases 1 + 2A)

**Estimated time:** 1-2 days for full implementation and testing

---

## Decision

**Status:** ✅ **APPROVED** - Ready to implement

**Next Steps:**
1. Create feature branch: `feature/inline-editing`
2. Implement Phase 1 (basic inline editing)
3. Test with all template types
4. Add Phase 2 (debounced rendering)
5. User testing
6. Merge to main

**Target completion:** 1 week

