# JavaScript Function Analysis: static/index.html

## Overview
This document provides a comprehensive analysis of every JavaScript function in `static/index.html`, explaining their purpose, usage, and role in the Equation Editor system.

---

## Core State Management

### `log(msg)`
**Purpose:** Simple console logging wrapper  
**Usage:** `log('message')`  
**Why:** Provides consistent logging interface, can be replaced with more sophisticated logging later

### `showStatus(msg, type)`
**Purpose:** Display status messages to user  
**Parameters:**
- `msg`: Message text
- `type`: 'success', 'error', 'info', 'warning'
**Usage:** `showStatus('✅ Rendered', 'success')`  
**Why:** Centralized status display - updates the status div with styled messages

---

## Undo/Redo System

### `saveToUndoStack()`
**Purpose:** Save current AST state before modification  
**Why:** Enables undo functionality - must be called before any AST mutation  
**Implementation:** Deep clones AST, pushes to stack, clears redo stack, limits history to 50

### `undo()`
**Purpose:** Restore previous AST state  
**Keyboard:** Cmd+Z / Ctrl+Z  
**Why:** Essential UX feature - allows users to revert mistakes  
**Implementation:** Pops from undo stack, saves current to redo stack, re-renders

### `redo()`
**Purpose:** Restore next AST state (after undo)  
**Keyboard:** Cmd+Shift+Z / Ctrl+Shift+Z  
**Why:** Allows users to re-apply undone changes  
**Implementation:** Pops from redo stack, saves current to undo stack, re-renders

### `updateUndoRedoButtons()`
**Purpose:** Update button states (enabled/disabled) and tooltips  
**Why:** Visual feedback showing undo/redo availability  
**Implementation:** Checks stack lengths, updates button disabled state and title attributes

---

## Zoom Controls

### `zoomIn()`
**Purpose:** Increase SVG zoom level  
**Keyboard:** Cmd+= / Ctrl+=  
**Why:** Allows users to see fine details in complex equations  
**Implementation:** Increments `currentZoom` by 0.2, max 3.0, calls `applyZoom()`

### `zoomOut()`
**Purpose:** Decrease SVG zoom level  
**Keyboard:** Cmd+- / Ctrl+-  
**Why:** Allows users to see full equation when zoomed in  
**Implementation:** Decrements `currentZoom` by 0.2, min 0.5, calls `applyZoom()`

### `zoomReset()`
**Purpose:** Reset zoom to 100%  
**Keyboard:** Cmd+0 / Ctrl+0  
**Why:** Quick way to return to natural size  
**Implementation:** Sets `currentZoom = 1.0`, calls `applyZoom()`

### `applyZoom()`
**Purpose:** Apply zoom transform to SVG element  
**Why:** Centralized zoom application - updates SVG transform style  
**Implementation:** Finds SVG, applies `scale(${currentZoom})`, updates status message

---

## Marker Navigation (MathType-style)

### `getAllMarkers()`
**Purpose:** Get all clickable placeholder/argument overlays  
**Returns:** Array of DOM elements with classes `.arg-overlay` or `.placeholder-overlay`  
**Why:** Used by navigation functions to find all editable positions

### `focusNextMarker()`
**Purpose:** Move selection to next editable marker  
**Keyboard:** ArrowDown / ArrowRight / Tab  
**Why:** Keyboard navigation - allows users to move between placeholders without mouse  
**Implementation:** Finds current marker index, wraps around, sets `activeEditMarker`, adds visual highlight, scrolls into view

### `focusPrevMarker()`
**Purpose:** Move selection to previous editable marker  
**Keyboard:** ArrowUp / ArrowLeft / Shift+Tab  
**Why:** Keyboard navigation in reverse direction  
**Implementation:** Similar to `focusNextMarker` but decrements index

### `editActiveMarker()`
**Purpose:** Open edit prompt for currently selected marker  
**Keyboard:** Enter (when marker selected)  
**Why:** Quick way to edit selected placeholder  
**Implementation:** Gets current value, shows prompt, parses input, updates AST

### `clearActiveMarker()`
**Purpose:** Deselect current marker  
**Keyboard:** Escape  
**Why:** Allows users to cancel selection  
**Implementation:** Clears `activeEditMarker`, removes visual highlights

---

## AST Manipulation

### `getNodeAtPath(ast, path)`
**Purpose:** Navigate AST tree to get node at specific path  
**Parameters:**
- `ast`: Root AST node
- `path`: Array of indices `[0, 1, 2]` = first arg's second arg's third arg
**Returns:** Node at path or null  
**Why:** Core AST navigation - needed for reading values at specific locations  
**Implementation:** Recursively traverses Operation.args or List elements

### `setNodeAtPath(ast, path, newValue)`
**Purpose:** Replace node at specific path with new value  
**Parameters:**
- `ast`: Root AST (mutated)
- `path`: Array of indices
- `newValue`: New node to insert
**Why:** Core AST mutation - needed for inserting templates/values at specific locations  
**Implementation:** Traverses to parent, replaces child at final index

### `getNodeById(ast, nodeId)`
**Purpose:** Get node by string ID like "0.1.2"  
**Parameters:**
- `ast`: Root AST
- `nodeId`: Dot-separated path string
**Returns:** Node at that path  
**Why:** Alternative to `getNodeAtPath` - uses string IDs from server response  
**Implementation:** Splits nodeId, navigates AST following path

### `nodeIdFromPath(pathArray)`
**Purpose:** Convert path array to node ID string  
**Parameters:** `[0, 1, 2]`  
**Returns:** `"0.1.2"`  
**Why:** Converts between path formats - used for matching server responses  
**Implementation:** Joins with dots, prepends "0" for root

### `parseSimpleInput(input)`
**Purpose:** Parse user text input into AST node  
**Parameters:** String like "42", "x", or ""  
**Returns:** `{Const: "42"}`, `{Object: "x"}`, or `{Placeholder: {...}}`  
**Why:** Converts inline editor input to AST format  
**Implementation:** Checks if number (Const), otherwise Object, empty becomes Placeholder

### `renumberPlaceholders(node)`
**Purpose:** Assign new IDs to all placeholders in AST  
**Why:** Prevents ID collisions when cloning templates  
**Implementation:** Recursively traverses AST, updates Placeholder.id using global counter

---

## Inline Editing System

### `isInlineEditorActive()`
**Purpose:** Check if inline editor is currently visible  
**Returns:** Boolean  
**Why:** Prevents conflicts - checks if user is actively editing  
**Implementation:** Checks `inlineEditorState.active` and foreignObject visibility

### `showInlineEditor(marker)`
**Purpose:** Display inline input field at marker location  
**Parameters:** Marker object with `{id, path, nodeId, element, bbox}`  
**Why:** Core UX feature - allows direct typing into placeholders  
**Implementation:**
1. Creates/retrieves SVG foreignObject
2. Positions at marker bbox coordinates
3. Sets current value from AST
4. Focuses and selects text
5. Adds visual feedback (editing-inline class)

### `hideInlineEditor(commit)`
**Purpose:** Close inline editor and optionally save changes  
**Parameters:** `commit` - if true, save value; if false, discard  
**Why:** Handles editor cleanup and value persistence  
**Implementation:**
1. If commit: parse input, update AST, re-render
2. Hide foreignObject
3. Clear input value
4. Remove visual feedback
5. Reset state

### `appendToInlineEditor(text)`
**Purpose:** Append text to current inline editor input  
**Parameters:** Text string (usually symbol)  
**Why:** Allows palette buttons to insert symbols while editing  
**Implementation:** Gets cursor position, inserts text, updates cursor, focuses

### `setupInlineEditorHandlers(input)`
**Purpose:** Attach keyboard and click handlers to inline editor  
**Why:** Handles Enter (commit), Escape (cancel), Tab (commit+next), click-outside  
**Implementation:** Adds keydown listener, sets up click-outside detection with debounce

### `getNodeValueAtPath(ast, path)`
**Purpose:** Extract display value from node at path  
**Returns:** String value from Const or Object node  
**Why:** Shows current value in inline editor when opened  
**Implementation:** Navigates to node, extracts Const.value or Object value

### `classifyButtonType(latex)`
**Purpose:** Determine if palette button is symbol, template, or function  
**Returns:** 'symbol', 'template', or 'function'  
**Why:** Affects inline editor behavior - symbols append, templates replace  
**Implementation:** Regex patterns to classify LaTeX strings

### `showReplaceConfirmation(currentText, template)`
**Purpose:** Show modal asking if user wants to replace typed text with template  
**Why:** Prevents accidental data loss when clicking template while typing  
**Implementation:** Shows modal, sets up yes/no handlers, stores pending template

---

## Template Insertion

### `insertTemplate(template)`
**Purpose:** Insert AST template from palette button  
**Parameters:** LaTeX template string like `'\\frac{□}{□}'`  
**Why:** Main entry point for palette button clicks  
**Implementation:**
1. Checks inline editor state (may show confirmation)
2. If active marker: insert at marker
3. Else: replace whole AST
4. Maps LaTeX to AST template name
5. Creates AST, renumbers placeholders
6. Updates and re-renders

### `insertStructuralTemplate(latexTemplate)`
**Purpose:** Create AST from template and replace current AST  
**Why:** Used when no marker selected - replaces entire equation  
**Implementation:** Maps LaTeX to template name, clones AST, renumbers, saves undo, sets currentAST

### `insertStructuralTemplateAt(latexTemplate, path)`
**Purpose:** Insert AST template at specific path  
**Why:** Used when marker selected - inserts into existing equation  
**Implementation:** Creates AST, inserts at path using `setNodeAtPath`, clears marker

### `insertSymbol(latex)`
**Purpose:** Insert single symbol (not template)  
**Why:** Handles symbol buttons differently from templates  
**Implementation:**
1. Converts LaTeX to Unicode
2. If inline editor active: append to editor
3. If marker selected: replace marker with Object node
4. Else: create new AST with just symbol
5. In text mode: insert LaTeX into textarea

---

## Rendering System

### `renderStructuralEditor()`
**Purpose:** Render current AST as SVG with interactive overlays  
**Why:** Core rendering function - converts AST to visual representation  
**Implementation:**
1. Sends AST to `/api/render_typst`
2. Receives SVG + placeholder positions + argument slots
3. Injects overlay rectangles for each editable position
4. Positions overlays using placeholder positions or semantic bounding boxes
5. Adds click/keyboard handlers to overlays
6. Handles matrix-specific grid inference
7. Preserves inline editor foreignObject if active
8. Triggers type checking

### `renderEquation()`
**Purpose:** Render LaTeX input in text mode using MathJax  
**Why:** Preview for text mode - shows rendered math  
**Implementation:** Sets preview div innerHTML, calls MathJax.typesetPromise

---

## Event Handlers

### `window.handleSlotClick(event, id, path, nodeId)`
**Purpose:** Handle clicks on placeholder/argument overlays  
**Parameters:**
- `event`: Click event
- `id`: Slot ID (string or number)
- `path`: Array path
- `nodeId`: String node ID
**Why:** Main interaction point - clicking placeholders  
**Implementation:**
1. Stops propagation
2. Saves undo state
3. Sets active marker
4. Highlights marker
5. If modifier key: show prompt dialog
6. Else: show inline editor

### `window.handleSlotKeydown(event, id, path, nodeId)`
**Purpose:** Handle keyboard events on overlay elements  
**Keyboard:** Enter or Space  
**Why:** Accessibility - keyboard navigation  
**Implementation:** Calls `handleSlotClick` on Enter/Space

---

## Type Checking

### `checkTypesDebounced()`
**Purpose:** Schedule type check with 500ms delay  
**Why:** Prevents excessive API calls while user is typing  
**Implementation:** Clears timeout, sets new timeout calling `checkTypes()`

### `checkTypes()`
**Purpose:** Send AST to server for type checking  
**Why:** Shows type information to user  
**Implementation:**
1. Shows loading indicator
2. POSTs to `/api/type_check`
3. Updates type indicator with result
4. Shows success (green) or error (red) styling

### `hideTypeIndicator()`
**Purpose:** Hide type indicator panel  
**Why:** Cleanup when no AST available  
**Implementation:** Sets display to none

---

## Mode Switching

### `window.setEditorMode(mode)`
**Purpose:** Switch between text and structural modes  
**Parameters:** 'text' or 'structural'  
**Why:** Core mode switching - changes entire UI  
**Implementation:**
1. Updates mode variable
2. Shows/hides appropriate UI elements
3. If switching to text: converts AST to LaTeX
4. If switching to structural: parses LaTeX to AST

### `convertStructuralToText()`
**Purpose:** Convert current AST to LaTeX and populate text input  
**Why:** Preserves work when switching modes  
**Implementation:** POSTs AST to `/api/render_ast` with format='latex', updates textarea

### `convertTextToStructural(latex)`
**Purpose:** Parse LaTeX and create AST  
**Why:** Allows users to start in text mode, switch to structural  
**Implementation:** POSTs LaTeX to `/api/parse`, sets currentAST, renders

---

## Matrix Builder

### `initializeMatrixBuilder()`
**Purpose:** Set up matrix builder UI on page load  
**Why:** Creates 6×6 grid, sets up event handlers  
**Implementation:**
1. Creates 36 grid cells
2. Adds hover/click handlers
3. Sets up numeric inputs
4. Sets up delimiter buttons
5. Initializes state

### `showMatrixBuilder()`
**Purpose:** Display matrix builder modal  
**Why:** Entry point for matrix creation  
**Implementation:** Shows modal, resets state to defaults, highlights 2×2 grid

### `closeMatrixBuilder()`
**Purpose:** Hide matrix builder modal  
**Why:** Cleanup when done or cancelled  
**Implementation:** Hides modal, clears grid highlighting

### `highlightMatrixGrid(rows, cols)`
**Purpose:** Visually highlight grid cells for selected size  
**Why:** Visual feedback showing selected matrix dimensions  
**Implementation:** Adds 'hover' class to cells within selected bounds

### `clearMatrixGrid()`
**Purpose:** Remove all grid highlighting  
**Why:** Cleanup function  
**Implementation:** Removes 'hover' class from all cells

### `selectMatrixSize(rows, cols)`
**Purpose:** Set matrix size from grid click  
**Why:** Updates state when user clicks grid cell  
**Implementation:** Updates state, updates inputs, updates display

### `updateSizeDisplay()`
**Purpose:** Update size display text  
**Why:** Shows current selection like "2 × 3"  
**Implementation:** Sets textContent to formatted size

### `createMatrixAST(rows, cols, delimiter)`
**Purpose:** Generate AST for matrix operation  
**Returns:** AST node with Matrix/PMatrix/VMatrix operation  
**Why:** Shared function used by both builder and palette buttons  
**Implementation:**
1. Creates placeholders for each cell (a11, a12, etc.)
2. Determines operation name from delimiter
3. Returns `{Operation: {name, args: [rows, cols, List]}}`

### `createMatrixFromBuilder()`
**Purpose:** Create matrix from builder modal and insert into editor  
**Why:** Main action handler for matrix builder  
**Implementation:**
1. Gets state (rows, cols, delimiter)
2. Creates AST using `createMatrixAST`
3. Saves undo state
4. Inserts at active marker or replaces AST
5. Renders
6. Closes modal

### `insertMatrixFromPalette(rows, cols, delimiter)`
**Purpose:** Insert matrix from palette button (quick insert)  
**Why:** Allows quick 2×2, 3×3 inserts without opening builder  
**Implementation:** Similar to `createMatrixFromBuilder` but called directly from button

---

## Piecewise Function Builder

### `showPiecewiseBuilder()`
**Purpose:** Display piecewise builder modal  
**Why:** Entry point for piecewise function creation  
**Implementation:** Shows modal, resets to 2 cases, updates preview

### `closePiecewiseBuilder()`
**Purpose:** Hide piecewise builder modal  
**Why:** Cleanup  
**Implementation:** Hides modal

### `updatePiecewisePreview()`
**Purpose:** Update preview text showing function structure  
**Why:** Visual feedback showing what will be created  
**Implementation:** Formats text like "f(x) = { expr₁ if cond₁\n{ expr₂ if cond₂"

### `createPiecewiseFromBuilder()`
**Purpose:** Create piecewise AST and insert into editor  
**Why:** Main action handler for piecewise builder  
**Implementation:**
1. Gets case count
2. Creates AST with Piecewise operation
3. Generates placeholders for expressions and conditions
4. Inserts at marker or replaces AST
5. Renders
6. Closes modal

---

## Verification (Z3)

### `verifyWithZ3()`
**Purpose:** Verify expression validity using Z3  
**Why:** Mathematical verification - checks if expression is always true  
**Implementation:**
1. Gets AST (from structural or parses LaTeX)
2. POSTs to `/api/verify`
3. Displays result (valid/invalid/unknown)
4. Shows Kleis syntax and counterexample if invalid

### `checkSatisfiable()`
**Purpose:** Check if expression can be true (satisfiability)  
**Why:** Existence check - different from verification  
**Implementation:**
1. Gets AST
2. POSTs to `/api/check_sat`
3. Displays result (satisfiable/unsatisfiable/unknown)
4. Shows example if satisfiable

---

## Palette Management

### `window.showPalette(name, btn)`
**Purpose:** Switch visible palette tab  
**Parameters:** Tab name ('basics', 'fences', etc.) and button element  
**Why:** Tab navigation - shows different symbol categories  
**Implementation:** Hides all palettes, shows selected, updates active tab styling

### `classifyAllButtons()`
**Purpose:** Classify all palette buttons by type (symbol/template/function)  
**Why:** Sets up inline editor behavior - symbols append, templates replace  
**Implementation:** Iterates buttons, extracts onclick, classifies, sets data-button-type attribute

---

## Gallery System

### `loadGallery()`
**Purpose:** Load example equations from server  
**Why:** Provides example equations users can load  
**Implementation:** Fetches `/api/gallery`, creates clickable items, renders with MathJax

### `window.loadExample(latex)`
**Purpose:** Load example equation into editor  
**Why:** Allows users to try pre-made examples  
**Implementation:**
1. Updates text input
2. If structural mode: parses LaTeX to AST
3. Else: renders in preview
4. Handles errors gracefully

---

## Utility Functions

### `window.clearInput()`
**Purpose:** Clear LaTeX input and preview  
**Why:** Reset button functionality  
**Implementation:** Clears textarea and preview div

### `window.resetStructuralEditor()`
**Purpose:** Reset structural editor to empty state  
**Why:** Clear button for structural mode  
**Implementation:** Sets currentAST to null, resets placeholder counter, re-renders

### `window.toggleBoundingBoxes()`
**Purpose:** Toggle visibility of overlay rectangles  
**Why:** Debug/UX feature - allows hiding overlays  
**Implementation:** Toggles SVG overlay group visibility based on checkbox

### `latexToUnicode(latex)`
**Purpose:** Convert LaTeX commands to Unicode symbols  
**Returns:** Unicode character or original string  
**Why:** Displays symbols in structural mode (not LaTeX)  
**Implementation:** Map of LaTeX commands to Unicode

---

## Debug Panel

### `toggleDebugPanel()`
**Purpose:** Show/hide AST debug panel  
**Why:** Developer tool - shows AST structure and metadata  
**Implementation:** Toggles display, calls `updateDebugPanel()` if showing

### `updateDebugPanel()`
**Purpose:** Populate debug panel with AST information  
**Why:** Shows detailed AST structure, placeholder info, render metadata  
**Implementation:**
1. Formats AST as JSON
2. Creates tree visualization
3. Shows statistics (placeholders, operations, depth)
4. Shows marker placement info from last render

### `formatASTAsTree(node, depth)`
**Purpose:** Format AST as indented tree string  
**Why:** Human-readable AST visualization  
**Implementation:** Recursively formats nodes with indentation, symbols (□, #, ○, ⊕)

### `countPlaceholdersInAST(node)`
**Purpose:** Count total placeholders in AST  
**Why:** Statistics for debug panel  
**Implementation:** Recursive count

### `countOperationsInAST(node)`
**Purpose:** Count total operations in AST  
**Why:** Statistics  
**Implementation:** Recursive count

### `getASTDepth(node)`
**Purpose:** Calculate maximum depth of AST tree  
**Why:** Statistics  
**Implementation:** Recursive max depth calculation

### `countNodesInAST(node)`
**Purpose:** Count total nodes in AST  
**Why:** Statistics  
**Implementation:** Recursive count

### `copyASTToClipboard()`
**Purpose:** Copy AST JSON to clipboard  
**Why:** Developer tool - allows exporting AST  
**Implementation:** Stringifies AST, uses Clipboard API

### `downloadAST()`
**Purpose:** Download AST as JSON file  
**Why:** Developer tool - allows saving AST  
**Implementation:** Creates blob, triggers download

---

## Initialization

### `window.onload`
**Purpose:** Initialize editor on page load  
**Why:** Sets up gallery, verifies template count, initializes systems  
**Implementation:**
1. Loads gallery
2. Verifies AST template count (should be 54)
3. Warns if old version cached

### `initializeInlineEditing()`
**Purpose:** Set up inline editing system  
**Why:** Initializes button classification, sets up handlers  
**Implementation:** Calls `classifyAllButtons()`, logs completion

---

## Keyboard Event Handler

### `document.addEventListener('keydown', ...)`
**Purpose:** Global keyboard shortcuts  
**Why:** Provides keyboard navigation and shortcuts  
**Handles:**
- Cmd+Z / Ctrl+Z: Undo
- Cmd+Shift+Z / Ctrl+Shift+Z: Redo
- Cmd+= / Ctrl+=: Zoom in
- Cmd+- / Ctrl+-: Zoom out
- Cmd+0 / Ctrl+0: Zoom reset
- Arrow keys: Navigate markers
- Tab: Navigate markers
- Enter: Edit active marker
- Escape: Clear selection

**Implementation:** Checks mode, prevents default, calls appropriate functions

---

## Click-Outside Handler

### `document.addEventListener('click', ...)` (inline editor)
**Purpose:** Close inline editor when clicking outside  
**Why:** Standard UX pattern - commit changes on blur  
**Implementation:** Checks if click is outside editor/modal/palette, closes editor

### `document.addEventListener('click', ...)` (matrix builder)
**Purpose:** Close matrix builder when clicking backdrop  
**Why:** Standard modal behavior  
**Implementation:** Checks if click target is modal backdrop, closes modal

### `document.addEventListener('click', ...)` (piecewise builder)
**Purpose:** Close piecewise builder when clicking backdrop  
**Why:** Standard modal behavior  
**Implementation:** Similar to matrix builder

---

## MathJax Initialization

### `window.MathJax.typesetPromise(...)`
**Purpose:** Render LaTeX in palette buttons  
**Why:** Palette buttons show rendered math symbols  
**Implementation:** Called on page load, renders all `.math-btn` elements

---

## Summary by Category

### **State Management** (4 functions)
- `log`, `showStatus`, `saveToUndoStack`, `updateUndoRedoButtons`

### **Undo/Redo** (2 functions)
- `undo`, `redo`

### **Zoom** (4 functions)
- `zoomIn`, `zoomOut`, `zoomReset`, `applyZoom`

### **Navigation** (5 functions)
- `getAllMarkers`, `focusNextMarker`, `focusPrevMarker`, `editActiveMarker`, `clearActiveMarker`

### **AST Manipulation** (6 functions)
- `getNodeAtPath`, `setNodeAtPath`, `getNodeById`, `nodeIdFromPath`, `parseSimpleInput`, `renumberPlaceholders`

### **Inline Editing** (8 functions)
- `isInlineEditorActive`, `showInlineEditor`, `hideInlineEditor`, `appendToInlineEditor`, `setupInlineEditorHandlers`, `getNodeValueAtPath`, `classifyButtonType`, `showReplaceConfirmation`

### **Template Insertion** (4 functions)
- `insertTemplate`, `insertStructuralTemplate`, `insertStructuralTemplateAt`, `insertSymbol`

### **Rendering** (2 functions)
- `renderStructuralEditor`, `renderEquation`

### **Event Handlers** (2 functions)
- `handleSlotClick`, `handleSlotKeydown`

### **Type Checking** (3 functions)
- `checkTypesDebounced`, `checkTypes`, `hideTypeIndicator`

### **Mode Switching** (3 functions)
- `setEditorMode`, `convertStructuralToText`, `convertTextToStructural`

### **Matrix Builder** (9 functions)
- `initializeMatrixBuilder`, `showMatrixBuilder`, `closeMatrixBuilder`, `highlightMatrixGrid`, `clearMatrixGrid`, `selectMatrixSize`, `updateSizeDisplay`, `createMatrixAST`, `createMatrixFromBuilder`, `insertMatrixFromPalette`

### **Piecewise Builder** (4 functions)
- `showPiecewiseBuilder`, `closePiecewiseBuilder`, `updatePiecewisePreview`, `createPiecewiseFromBuilder`

### **Verification** (2 functions)
- `verifyWithZ3`, `checkSatisfiable`

### **Palette** (2 functions)
- `showPalette`, `classifyAllButtons`

### **Gallery** (2 functions)
- `loadGallery`, `loadExample`

### **Utilities** (5 functions)
- `clearInput`, `resetStructuralEditor`, `toggleBoundingBoxes`, `latexToUnicode`

### **Debug** (7 functions)
- `toggleDebugPanel`, `updateDebugPanel`, `formatASTAsTree`, `countPlaceholdersInAST`, `countOperationsInAST`, `getASTDepth`, `countNodesInAST`, `copyASTToClipboard`, `downloadAST`

### **Initialization** (2 functions)
- `window.onload`, `initializeInlineEditing`

**Total: ~75 JavaScript functions**

---

## Critical Dependencies

### Functions that must be called in order:
1. **Before any AST mutation:** `saveToUndoStack()`
2. **After AST mutation:** `renderStructuralEditor()`
3. **Before showing inline editor:** `getNodeValueAtPath()` to get current value
4. **After inline edit:** `parseSimpleInput()` then `setNodeAtPath()` then `renderStructuralEditor()`

### Functions that depend on server:
- `renderStructuralEditor()` → `/api/render_typst`
- `checkTypes()` → `/api/type_check`
- `verifyWithZ3()` → `/api/verify`
- `checkSatisfiable()` → `/api/check_sat`
- `convertTextToStructural()` → `/api/parse`
- `convertStructuralToText()` → `/api/render_ast`
- `loadGallery()` → `/api/gallery`

---

## Key Patterns

### 1. **State Mutation Pattern**
```javascript
saveToUndoStack();           // Save state
setNodeAtPath(...);          // Mutate AST
renderStructuralEditor();    // Re-render
```

### 2. **Marker Selection Pattern**
```javascript
activeEditMarker = {...};                    // Set state
element.classList.add('active-marker');      // Visual feedback
showInlineEditor(activeEditMarker);         // Show editor
```

### 3. **Template Insertion Pattern**
```javascript
if (activeEditMarker) {
    insertStructuralTemplateAt(template, path);  // Insert at marker
} else {
    insertStructuralTemplate(template);          // Replace whole AST
}
```

### 4. **Error Handling Pattern**
```javascript
try {
    // Operation
    showStatus('✅ Success', 'success');
} catch (error) {
    showStatus('❌ Error: ' + error.message, 'error');
    console.error('Details:', error);
}
```

---

## Missing in PatternFly Version

Functions that exist in static/index.html but may need implementation in PatternFly:

1. ✅ **Implemented:**
   - Undo/redo (`useUndoRedo` hook)
   - Zoom controls
   - Keyboard navigation
   - Inline editing (SVGEditor component)
   - Template insertion
   - Type checking
   - Verify/SAT
   - Matrix builder

2. ⚠️ **Partially Implemented:**
   - Marker navigation (needs testing)
   - Inline editor positioning (may need SVG foreignObject approach)
   - **Palette button rendering** - Currently shows plain text, should render MathJax SVGs

3. ❌ **Not Yet Implemented:**
   - Piecewise builder modal
   - Gallery examples
   - Debug panel
   - Bounding box toggle
   - Mode switching (text ↔ structural)
   - LaTeX parsing/conversion
   - **MathJax integration** - Buttons should render LaTeX as SVG (like `\(a = b\)` → rendered math)

---

## Critical Visual Difference: MathJax Rendering

### static/index.html Approach
- **Buttons contain LaTeX:** `\(a = b\)`, `\(\frac{a}{b}\)`, `\(\int_a^b f\,dx\)`
- **MathJax renders to SVG:** On page load, MathJax converts LaTeX to SVG elements
- **CSS styles SVGs:** `.math-btn .MathJax_SVG svg { fill: #333; }`
- **Result:** Beautiful rendered math symbols in buttons

### PatternFly Current Approach
- **Buttons show plain text:** `config.symbol || config.label` (e.g., "a/b", "∫", "×")
- **No MathJax:** No LaTeX rendering
- **Result:** Plain Unicode/text labels, not rendered math

### Impact
- **Visual quality:** MathJax SVGs look professional and match rendered equations
- **Consistency:** Buttons match the visual style of the rendered output
- **Complexity:** Some math notation is hard to represent with Unicode alone

### Solution Needed
Add MathJax to PatternFly version:
1. Load MathJax library (CDN or npm package)
2. Store LaTeX strings in button configs (like `\(a = b\)`)
3. Render buttons with MathJax after mount
4. Style MathJax SVG output appropriately

---

## Notes for PatternFly Implementation

1. **Inline Editor:** static/index.html uses SVG `foreignObject` - PatternFly version uses absolute positioning overlay. May need to switch to foreignObject for better integration.

2. **Marker Click Handling:** static/index.html injects overlays into SVG and uses inline onclick handlers. PatternFly version needs to ensure click handlers are properly attached after SVG render.

3. **State Management:** static/index.html uses global variables. PatternFly version uses React state - ensure all state updates trigger re-renders.

4. **Keyboard Events:** static/index.html uses global document listener. PatternFly version should use React keyboard handlers or ensure global listener doesn't conflict.

5. **Debouncing:** Type checking uses debouncing - ensure PatternFly version implements this.

6. **Error Handling:** static/index.html has comprehensive error handling with user feedback. Ensure PatternFly version shows errors to users.

