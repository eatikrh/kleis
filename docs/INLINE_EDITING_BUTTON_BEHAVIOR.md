# Inline Editing: Button Behavior Design

## The Challenge

When inline editor is active (user is typing in a placeholder), clicking palette buttons should behave intelligently:

1. **Symbol buttons** → Safe to append to text input
2. **Template buttons** → Would cause rendering errors if treated as text

## Solution: Smart Button Classification

### Button Categories

#### Category 1: Symbols (Safe - Append to Input)

**Greek Letters:**
- Lowercase: `α`, `β`, `γ`, `δ`, `ε`, `ζ`, `η`, `θ`, `λ`, `μ`, `ν`, `π`, `ρ`, `σ`, `τ`, `φ`, `χ`, `ψ`, `ω`
- Uppercase: `Γ`, `Δ`, `Θ`, `Λ`, `Ξ`, `Π`, `Σ`, `Φ`, `Ψ`, `Ω`

**Operators:**
- `+`, `-`, `×`, `÷`, `·`, `∗`, `=`, `≠`, `±`, `∓`

**Logic & Sets:**
- `∀`, `∃`, `¬`, `∧`, `∨`, `∈`, `∉`, `⊂`, `⊆`, `∪`, `∩`, `∅`
- `<`, `>`, `≤`, `≥`, `≈`, `≡`, `→`, `⇒`, `⇔`

**Special:**
- `∞`, `∂`, `∇`, `□`

**Action:** Append LaTeX to input field

#### Category 2: Templates (Unsafe - Replace Marker)

**Structural Operations:**
- Fractions: `\frac{□}{□}`
- Powers: `□^{□}`, `□_{□}`, `□^{□}_{□}`
- Roots: `\sqrt{□}`, `\sqrt[□]{□}`
- Integrals: `\int_{□}^{□} □ \, dx`
- Sums: `\sum_{□}^{□} □`
- Matrices: `\begin{bmatrix}...\end{bmatrix}`
- Brackets: `(□)`, `[□]`, `{□}` ⚠️
- Functions: `\sin(□)`, `\ln(□)`, etc.

**Action:** Close inline editor, insert template AST

#### Category 3: Hybrid (Context-Dependent)

**Numbers with symbols:**
- `!` (factorial) - Can be typed OR template
- `^` (caret) - Can be typed OR power template

**Action:** Smart detection based on context

---

## Detailed Implementation

### 1. Classify Buttons on Page Load

```javascript
function classifyAllButtons() {
    const buttons = document.querySelectorAll('.math-btn');
    
    buttons.forEach(btn => {
        const onclick = btn.getAttribute('onclick');
        
        // Extract function name and argument
        const match = onclick.match(/(insertSymbol|insertTemplate)\('([^']+)'\)/);
        if (!match) return;
        
        const [_, funcName, latex] = match;
        
        // Classify
        let type;
        if (funcName === 'insertSymbol') {
            type = 'symbol'; // Already classified by function
        } else if (funcName === 'insertTemplate') {
            type = classifyTemplate(latex);
        }
        
        btn.setAttribute('data-button-type', type);
        btn.classList.add(`btn-${type}`);
    });
}

function classifyTemplate(latex) {
    // Check if it's a simple structure that could be typed
    if (/^[a-zA-Z0-9_]$/.test(latex)) return 'symbol';
    
    // Check for LaTeX commands that are pure symbols
    const pureSymbols = [
        'alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta',
        'iota', 'kappa', 'lambda', 'mu', 'nu', 'xi', 'pi', 'rho', 'sigma',
        'tau', 'upsilon', 'phi', 'chi', 'psi', 'omega',
        'Gamma', 'Delta', 'Theta', 'Lambda', 'Xi', 'Pi', 'Sigma',
        'Phi', 'Psi', 'Omega',
        'pm', 'mp', 'times', 'div', 'cdot', 'ast',
        'leq', 'geq', 'neq', 'approx', 'equiv',
        'in', 'notin', 'subset', 'subseteq', 'cup', 'cap', 'emptyset',
        'to', 'Rightarrow', 'Leftrightarrow',
        'forall', 'exists', 'neg', 'land', 'lor',
        'infty', 'partial', 'nabla'
    ];
    
    const simpleCommand = latex.match(/^\\([a-zA-Z]+)$/);
    if (simpleCommand && pureSymbols.includes(simpleCommand[1])) {
        return 'symbol';
    }
    
    // Everything with placeholders or complex structure is a template
    if (latex.includes('□') || latex.includes('\\frac') || latex.includes('\\begin') ||
        latex.includes('\\sqrt') || latex.includes('\\int') || latex.includes('\\sum') ||
        latex.includes('\\left') || latex.includes('(□)')) {
        return 'template';
    }
    
    // Functions
    if (/\\(sin|cos|tan|ln|log|exp)\(/.test(latex)) {
        return 'function';
    }
    
    // Default to symbol (safer)
    return 'symbol';
}
```

### 2. Update Button Click Handlers

```javascript
// Unified handler for all palette buttons
function handlePaletteButtonClick(event) {
    const btn = event.currentTarget;
    const buttonType = btn.getAttribute('data-button-type');
    const onclick = btn.getAttribute('onclick');
    
    // Extract the function call
    const match = onclick.match(/(insertSymbol|insertTemplate)\('([^']+)'\)/);
    if (!match) return;
    
    const [_, funcName, latex] = match;
    
    // Check if inline editor is active
    if (isInlineEditorActive()) {
        handleButtonDuringInlineEditing(buttonType, funcName, latex);
    } else {
        // Normal behavior
        if (funcName === 'insertSymbol') {
            insertSymbol(latex);
        } else {
            insertTemplate(latex);
        }
    }
}

function handleButtonDuringInlineEditing(buttonType, funcName, latex) {
    if (buttonType === 'symbol') {
        // Append to inline input
        appendToInlineEditor(latex);
    } else {
        // Template or function - needs confirmation
        const currentInput = document.getElementById('inline-input').value.trim();
        
        if (currentInput) {
            // User has typed something - ask for confirmation
            showReplaceConfirmation(currentInput, latex);
        } else {
            // Empty input - just insert template
            hideInlineEditor(false);
            insertTemplate(latex);
        }
    }
}

function showReplaceConfirmation(currentText, template) {
    const modal = document.getElementById('replace-confirm-modal');
    const message = document.getElementById('replace-message');
    
    message.textContent = `Replace "${currentText}" with template?`;
    modal.style.display = 'block';
    
    // Handle buttons
    document.getElementById('replace-confirm-yes').onclick = () => {
        modal.style.display = 'none';
        hideInlineEditor(false);
        insertTemplate(template);
    };
    
    document.getElementById('replace-confirm-no').onclick = () => {
        modal.style.display = 'none';
        document.getElementById('inline-input').focus();
    };
}
```

### 3. Visual Feedback System

```javascript
function updateButtonVisualState() {
    const isInline = isInlineEditorActive();
    
    document.body.classList.toggle('inline-editing', isInline);
    
    if (isInline) {
        // Update tooltips to show behavior
        document.querySelectorAll('.math-btn').forEach(btn => {
            const type = btn.getAttribute('data-button-type');
            const originalTooltip = btn.getAttribute('data-tooltip');
            
            if (type === 'symbol') {
                btn.setAttribute('data-tooltip-editing', `${originalTooltip} (appends)`);
            } else {
                btn.setAttribute('data-tooltip-editing', `${originalTooltip} (replaces)`);
            }
        });
    }
}
```

---

## Error Prevention

### Prevent Invalid LaTeX

```javascript
function parseSimpleInput(input) {
    // Detect if input contains template syntax
    const hasTemplateSyntax = /\\frac|\\sqrt|\\begin|\\left|□/.test(input);
    
    if (hasTemplateSyntax) {
        showError('Template syntax not supported in inline mode. Use Shift+Click to open dialog.');
        return null;
    }
    
    // Parse as normal
    if (!input) return { Placeholder: { id: nextPlaceholderId++, hint: 'val' } };
    if (/^-?\d+(\.\d+)?$/.test(input)) return { Const: input };
    if (/^\\[a-zA-Z]+$/.test(input)) return { Const: input }; // LaTeX symbol
    return { Object: input };
}
```

### Graceful Degradation

```javascript
function commitInlineEdit() {
    const input = document.getElementById('inline-input');
    const value = input.value.trim();
    
    if (!value) {
        // Empty - just close
        hideInlineEditor(false);
        return;
    }
    
    try {
        const node = parseSimpleInput(value);
        if (node === null) {
            // Parse error - keep editor open
            input.classList.add('error');
            return;
        }
        
        setNodeAtPath(currentAST, activeEditMarker.path, node);
        renderStructuralEditor();
        hideInlineEditor(false);
        showStatus('✅ Value updated', 'success');
    } catch (error) {
        console.error('Inline edit error:', error);
        showError('Invalid input: ' + error.message);
        input.classList.add('error');
    }
}
```

---

## HTML Modal for Confirmation

```html
<!-- Add to index.html -->
<div id="replace-confirm-modal" class="modal" style="display:none;">
    <div class="modal-content">
        <h3>Replace text with template?</h3>
        <p id="replace-message"></p>
        <div class="modal-buttons">
            <button id="replace-confirm-yes" class="btn-primary">Insert Template</button>
            <button id="replace-confirm-no" class="btn-secondary">Keep Typing</button>
        </div>
    </div>
</div>

<style>
.modal {
    position: fixed;
    top: 0; left: 0;
    width: 100%; height: 100%;
    background: rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
}

.modal-content {
    background: white;
    padding: 24px;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    max-width: 400px;
}

.modal-buttons {
    display: flex;
    gap: 12px;
    margin-top: 20px;
    justify-content: flex-end;
}
</style>
```

---

## Testing Scenarios

### Test 1: Symbol Append
```
1. Click marker → Inline editor appears
2. Click "α" → Input shows "α"
3. Click "β" → Input shows "αβ"
4. Click "+" → Input shows "αβ+"
5. Press Enter → Renders successfully ✅
```

### Test 2: Template Replacement (Empty Input)
```
1. Click marker → Inline editor appears (empty)
2. Click "fraction" button → Immediately inserts fraction template ✅
3. No confirmation needed (nothing to lose)
```

### Test 3: Template Replacement (Has Text)
```
1. Click marker → Inline editor appears
2. Type "x" → Input shows "x"
3. Click "fraction" button → Shows confirmation:
   "Replace 'x' with template?"
4a. Click "Insert Template" → Fraction replaces marker ✅
4b. Click "Keep Typing" → Returns to input ✅
```

### Test 4: Function Button
```
1. Click marker → Inline editor appears
2. Type "0" → Input shows "0"
3. Click "sin(x)" button → Shows confirmation
4. Click "Insert Template" → sin operation replaces marker ✅
```

### Test 5: Nested Editing
```
1. Have equation: x + □
2. Click marker → Inline editor appears
3. Type "y" → Shows "y"
4. Press Tab → Commits "y", no next marker
   Result: x + y ✅
```

### Test 6: Cancel During Typing
```
1. Click marker → Inline editor appears
2. Type "xyz" → Input shows "xyz"
3. Press ESC → Input closes, marker stays empty
   No change to AST ✅
```

---

## Performance Considerations

### Rendering Strategy Options

#### Option A: Render on Commit (Conservative)
```javascript
// Only render when user presses Enter or clicks away
hideInlineEditor(true); // Commits and renders once
```
**Pros:** Best performance, no interruptions  
**Cons:** No live preview while typing

#### Option B: Debounced Rendering (Recommended)
```javascript
// Render 300ms after last keystroke
let renderTimer;
input.addEventListener('input', () => {
    clearTimeout(renderTimer);
    renderTimer = setTimeout(() => {
        updatePreviewFromInput();
    }, 300);
});
```
**Pros:** Live preview, not too heavy  
**Cons:** Slight delay, some renders may be wasted

#### Option C: Every Keystroke (Aggressive)
```javascript
// Render immediately on every keystroke
input.addEventListener('input', () => {
    updatePreviewFromInput();
});
```
**Pros:** Instant feedback  
**Cons:** May lag with complex equations

**Recommendation:** Start with **Option A** (commit-only), add **Option B** (debounced) in Phase 2.

### Optimization Tricks

```javascript
function updatePreviewFromInput() {
    const value = input.value.trim();
    
    // Skip if value hasn't changed
    if (value === lastRenderedValue) return;
    lastRenderedValue = value;
    
    // Skip rendering if input is partial/invalid
    if (value.endsWith('\\')) return; // Typing LaTeX command
    
    // Optimize: Only update the specific node, not full re-render
    const node = parseSimpleInput(value);
    setNodeAtPath(currentAST, activeEditMarker.path, node);
    
    // Use requestAnimationFrame for smooth updates
    requestAnimationFrame(() => {
        renderStructuralEditor();
    });
}
```

---

## Mobile Considerations

### Touch Interaction

```javascript
// On mobile, always show dialog (no inline editor)
if (isMobileDevice()) {
    showEditDialog(marker);
    return;
}

function isMobileDevice() {
    return /Android|iPhone|iPad|iPod/i.test(navigator.userAgent) ||
           ('ontouchstart' in window);
}
```

**Reasoning:**
- Small screens make inline editing harder
- Virtual keyboard covers content
- Dialog is more reliable on mobile

### Responsive Behavior

```css
@media (max-width: 768px) {
    /* Disable inline editing on mobile */
    #inline-editor {
        display: none !important;
    }
    
    /* Always use dialog */
}
```

---

## User Guidance

### Visual Hints

Show a subtle hint when inline editor first appears:

```javascript
function showInlineEditorFirstTime() {
    if (!localStorage.getItem('inline-editor-hint-shown')) {
        showTemporaryTooltip(
            'Tip: Type directly, or click symbols to insert. Press Enter to confirm.',
            3000
        );
        localStorage.setItem('inline-editor-hint-shown', 'true');
    }
}
```

### Button Hover States

```css
/* During inline editing */
body.inline-editing .math-btn[data-button-type="symbol"]:hover::after {
    content: '✓ Appends to input';
}

body.inline-editing .math-btn[data-button-type="template"]:hover::after {
    content: '⚠ Replaces input';
}
```

---

## State Machine Diagram

```
┌─────────────────────────────────────────────────┐
│                  IDLE STATE                      │
│            (No marker selected)                  │
└────────────┬────────────────────┬────────────────┘
             │                    │
    Click Marker        Shift+Click Marker
             │                    │
             ▼                    ▼
    ┌────────────────┐   ┌────────────────┐
    │ INLINE EDITING │   │  DIALOG OPEN   │
    │  (typing text) │   │  (existing UI) │
    └────────┬───────┘   └────────┬───────┘
             │                    │
      Click Symbol          Click Template
      (appends)              (inserts AST)
             │                    │
      Press Enter/ESC        Click OK/Cancel
             │                    │
             ▼                    ▼
    ┌────────────────────────────────────┐
    │         COMMITTED STATE            │
    │     AST updated, re-rendered       │
    └────────────────────────────────────┘
             │
             ▼
    ┌────────────────┐
    │   IDLE STATE   │
    └────────────────┘
```

---

## Implementation Estimate

### Phase 1: Core Inline Editing
**Components:**
- foreignObject structure (30 lines HTML/CSS)
- Click handler modification (20 lines)
- showInlineEditor() (40 lines)
- hideInlineEditor() (30 lines)
- appendToInlineEditor() (20 lines)
- Keyboard handlers (40 lines)
- Button classification (60 lines)

**Total:** ~240 lines  
**Time:** 6-8 hours  
**Risk:** Low

### Phase 2: Smart Button Behavior
**Components:**
- classifyAllButtons() (50 lines)
- handleButtonDuringInlineEditing() (40 lines)
- showReplaceConfirmation() (30 lines)
- Modal HTML/CSS (50 lines)
- Visual feedback CSS (40 lines)

**Total:** ~210 lines  
**Time:** 4-6 hours  
**Risk:** Low

### Phase 3: Live Rendering
**Components:**
- Debounced render (30 lines)
- Performance optimization (40 lines)
- Error handling (30 lines)

**Total:** ~100 lines  
**Time:** 2-4 hours  
**Risk:** Medium (performance)

---

## Total Estimate

**Lines of Code:** ~550 lines  
**Time:** 12-18 hours (1.5-2 days)  
**Complexity:** Medium  
**Value:** Very High ⭐⭐⭐⭐⭐

---

## Decision

**Recommendation: Implement in 2 phases**

1. **Phase 1 + 2** (Core + Smart Buttons) - Essential, high value
2. **Phase 3** (Live Rendering) - Nice-to-have, add later if needed

**Start with:** Phase 1 + 2 = ~450 lines, ~10-14 hours

This gives users a natural editing experience while preventing errors from template buttons.

---

**Status:** ✅ **Design Complete - Ready for Implementation**

