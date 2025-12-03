# Handling Arbitrary-Size Matrices in the Palette

**Question:** How do we handle arbitrary size matrices in the palette?

**Short Answer:** The backend already supports arbitrary-size matrices! We just need smart UI/UX in the palette.

---

## Current Implementation

### ✅ Parser Already Handles Arbitrary Sizes

From `src/parser.rs` lines 809-851:

```rust
// Convert rows to matrix operation
// For 2x2 matrix
if rows.len() == 2 && rows[0].len() == 2 && rows[1].len() == 2 {
    let op_name = match env_name {
        "pmatrix" => "pmatrix2x2",
        "vmatrix" => "vmatrix2x2",
        _ => "matrix2x2",
    };
    Ok(op(op_name, vec![...]))  // 4 elements
} else if rows.len() == 3 && rows[0].len() == 3 {
    let op_name = match env_name {
        "pmatrix" => "pmatrix3x3",
        "vmatrix" => "vmatrix3x3",
        _ => "matrix3x3",
    };
    Ok(op(op_name, vec![...]))  // 9 elements
} else {
    // Generic matrix - store as operation with all elements
    let all_elements: Vec<Expression> =
        rows.into_iter().flat_map(|row| row.into_iter()).collect();
    Ok(op("matrix", all_elements))  // ✅ ANY SIZE!
}
```

**Key insight:** If the matrix is not 2×2 or 3×3, it falls through to a **generic `"matrix"` operation** that stores all elements as a flat list.

### ✅ Renderer Handles Generic Matrices

The renderer can handle arbitrary-size matrices because:
1. It processes the generic `"matrix"` operation
2. LaTeX/Typst naturally support arbitrary-size matrices
3. The `\begin{bmatrix}...\end{bmatrix}` syntax works for any size

---

## The Challenge: Palette UI/UX

The palette needs to let users **create** matrices of arbitrary size. We have several options:

---

## Solution 1: Pre-defined Common Sizes (Current Approach)

**Pros:**
- Simple, no UI complexity
- Fast for common cases
- Works in both text and structural modes

**Cons:**
- Limited to pre-defined sizes
- Clutters palette with many buttons
- Doesn't scale to large matrices

**Implementation:**
```html
<!-- Current palette -->
<button onclick="insertTemplate('\\begin{bmatrix}□&□\\\\□&□\\end{bmatrix}')">
    Matrix 2×2
</button>
<button onclick="insertTemplate('\\begin{bmatrix}□&□&□\\\\□&□&□\\\\□&□&□\\end{bmatrix}')">
    Matrix 3×3
</button>
<button onclick="insertTemplate('\\begin{bmatrix}□&□&□&□\\\\□&□&□&□\\\\□&□&□&□\\\\□&□&□&□\\end{bmatrix}')">
    Matrix 4×4
</button>
```

**Recommendation:** Keep this for 2×2, 3×3, maybe 4×4. Don't go beyond that.

---

## Solution 2: Matrix Builder Dialog (Recommended)

**Pros:**
- Supports any size (1×1 to 10×10+)
- Clean palette (just one "Matrix Builder" button)
- Better UX for large matrices
- Can set delimiter type (brackets, parens, bars)

**Cons:**
- Requires modal/dialog UI
- More complex implementation
- Extra click for users

### Reference: How MathType’s Builder Works
- The MathType panel shows a **size grid** (up to 6×6) plus numeric inputs for rows/cols so users can pick a quick preset or type any number.
- Fence controls sit beside the grid, letting users choose start/end symbols (parentheses, brackets, braces, mixed, or none) with one-sided support.
- After insertion, placing the cursor inside the matrix reveals a **context toolbar** with structural actions: insert/remove rows or columns, delete the matrix, toggle fences/borders, adjust alignment (horizontal, vertical, matrix-wide), and force equal column widths or row heights.
- This flow keeps the palette uncluttered while still allowing arbitrary sizes and post-insert resizing, which is the model we should emulate in Kleis.

**Implementation:**

### HTML (Modal Dialog)
```html
<!-- In palette -->
<button class="template-btn" onclick="showMatrixBuilder()">
    🔢 Matrix Builder
</button>

<!-- Modal dialog -->
<div id="matrixBuilderModal" class="modal" style="display:none;">
    <div class="modal-content">
        <h3>Create Matrix</h3>
        
        <label>Rows: <input type="number" id="matrixRows" value="2" min="1" max="10"></label>
        <label>Columns: <input type="number" id="matrixCols" value="2" min="1" max="10"></label>
        
        <label>Delimiter:
            <select id="matrixDelimiter">
                <option value="bmatrix">Square brackets [ ]</option>
                <option value="pmatrix">Parentheses ( )</option>
                <option value="vmatrix">Vertical bars | |</option>
                <option value="Bmatrix">Curly braces { }</option>
            </select>
        </label>
        
        <button onclick="createMatrix()">Create</button>
        <button onclick="closeMatrixBuilder()">Cancel</button>
    </div>
</div>
```

### JavaScript
```javascript
function showMatrixBuilder() {
    document.getElementById('matrixBuilderModal').style.display = 'block';
}

function closeMatrixBuilder() {
    document.getElementById('matrixBuilderModal').style.display = 'none';
}

function createMatrix() {
    const rows = parseInt(document.getElementById('matrixRows').value);
    const cols = parseInt(document.getElementById('matrixCols').value);
    const delimiter = document.getElementById('matrixDelimiter').value;
    
    // Build LaTeX template
    let latex = `\\begin{${delimiter}}`;
    
    for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
            latex += '□';
            if (c < cols - 1) latex += '&';
        }
        if (r < rows - 1) latex += '\\\\';
    }
    
    latex += `\\end{${delimiter}}`;
    
    // Insert into editor
    insertTemplate(latex);
    closeMatrixBuilder();
}
```

---

## Solution 3: Smart Expansion (Advanced)

**Concept:** Start with a small matrix, let users add rows/columns dynamically.

**Pros:**
- Intuitive "grow as you go" workflow
- No need to know final size upfront
- Great for structural mode

**Cons:**
- Complex implementation
- Requires structural mode to work well
- Needs UI for "add row/column" buttons

**Implementation Sketch:**
```javascript
// In structural mode, show +/- buttons on matrix edges
function addMatrixRow(matrixNode) {
    // Parse current matrix AST
    // Add new row with placeholders
    // Re-render
}

function addMatrixColumn(matrixNode) {
    // Parse current matrix AST
    // Add placeholder to each row
    // Re-render
}
```

---

## Solution 4: Text Mode Fallback (Simplest)

**Concept:** For arbitrary sizes, just use text mode.

**Pros:**
- Zero implementation needed
- Maximum flexibility
- Users can copy/paste matrix templates

**Cons:**
- No structural editing for custom sizes
- Requires LaTeX knowledge

**User workflow:**
1. Click "Matrix (custom)" button
2. Inserts: `\begin{bmatrix}\n□&□\n\end{bmatrix}`
3. User manually adds `&` for columns, `\\` for rows

---

## Recommended Hybrid Approach

Combine solutions for best UX:

### In Palette:

#### Quick Access (No Dialog)
```html
<!-- Common sizes - instant insert -->
<button onclick="insertTemplate('\\begin{bmatrix}□&□\\\\□&□\\end{bmatrix}')">
    [ ] 2×2
</button>
<button onclick="insertTemplate('\\begin{bmatrix}□&□&□\\\\□&□&□\\\\□&□&□\\end{bmatrix}')">
    [ ] 3×3
</button>
<button onclick="insertTemplate('\\begin{pmatrix}□&□\\\\□&□\\end{pmatrix}')">
    ( ) 2×2
</button>
<button onclick="insertTemplate('\\begin{vmatrix}□&□\\\\□&□\\end{vmatrix}')">
    | | 2×2
</button>
```

#### Matrix Builder (For Custom Sizes)
```html
<button onclick="showMatrixBuilder()">
    🔧 Custom Size...
</button>
```

### Result:
- **90% of users** use quick 2×2 or 3×3 buttons (one click)
- **10% of users** need custom sizes (two clicks: open dialog, create)
- **Advanced users** can type directly in text mode

---

## Implementation Priority

### Phase 1: Fix Current Issues (Immediate)
1. ✅ Parser already handles arbitrary sizes
2. ❌ Fix Matrix 3×3 template in palette (currently broken)
3. ✅ Add pmatrix/vmatrix 2×2 and 3×3 buttons

### Phase 2: Add Matrix Builder (1 week)
1. Create modal dialog UI
2. Implement matrix generator function
3. Test with various sizes (1×1 to 10×10)
4. Add keyboard shortcut (Ctrl+M?)

### Phase 3: Smart Expansion (Future)
1. Add row/column buttons in structural mode
2. Implement AST manipulation for matrix resizing
3. Fix matrix edit marker issues first (prerequisite)

---

## Code Examples

### Example 1: Generate 5×3 Matrix
```javascript
function generateMatrix(rows, cols, delimiter) {
    let latex = `\\begin{${delimiter}}`;
    for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
            latex += '□';
            if (c < cols - 1) latex += '&';
        }
        if (r < rows - 1) latex += '\\\\';
    }
    latex += `\\end{${delimiter}}`;
    return latex;
}

// Usage:
generateMatrix(5, 3, 'bmatrix')
// Returns: \begin{bmatrix}□&□&□\\□&□&□\\□&□&□\\□&□&□\\□&□&□\end{bmatrix}
```

### Example 2: Parse Existing Matrix Size
```javascript
function getMatrixSize(ast) {
    if (ast.Operation) {
        const name = ast.Operation.name;
        if (name === 'matrix2x2' || name === 'pmatrix2x2' || name === 'vmatrix2x2') {
            return { rows: 2, cols: 2 };
        }
        if (name === 'matrix3x3' || name === 'pmatrix3x3' || name === 'vmatrix3x3') {
            return { rows: 3, cols: 3 };
        }
        if (name === 'matrix') {
            // Generic matrix - need to infer from args
            // This is tricky without row metadata
            return { rows: '?', cols: '?' };
        }
    }
    return null;
}
```

---

## Testing Strategy

### Test Cases for Arbitrary Matrices

1. **Small matrices:** 1×1, 2×1, 1×2
2. **Common sizes:** 2×2, 3×3, 4×4
3. **Rectangular:** 2×3, 3×2, 4×2
4. **Large:** 5×5, 10×10
5. **Edge cases:** 1×10, 10×1

### Test Each Delimiter Type
- `bmatrix` (square brackets)
- `pmatrix` (parentheses)
- `vmatrix` (vertical bars)
- `Bmatrix` (curly braces)

### Test Complex Content
```latex
\begin{bmatrix}
\frac{1}{2} & \sqrt{3} & e^{i\pi} \\
\sin(\theta) & \cos(\theta) & \tan(\theta) \\
\alpha & \beta & \gamma
\end{bmatrix}
```

---

## Performance Considerations

### Large Matrices (10×10+)

**Concern:** Will 100+ placeholders slow down the editor?

**Analysis:**
- 10×10 matrix = 100 placeholders
- Each placeholder needs bounding box calculation
- Current implementation may struggle with >50 placeholders

**Solutions:**
1. **Lazy rendering:** Only render visible cells
2. **Pagination:** Show matrix in chunks
3. **Text mode fallback:** For very large matrices, disable structural mode
4. **Warning:** Show alert if user tries to create 15×15+ matrix

**Recommended limit:** 10×10 (100 cells) for structural mode

---

## UI Mockup

```
┌─────────────────────────────────────┐
│  Matrix Builder                     │
├─────────────────────────────────────┤
│                                     │
│  Size:                              │
│  Rows:    [2] ▲▼                    │
│  Columns: [2] ▲▼                    │
│                                     │
│  Delimiter:                         │
│  ○ Square brackets [ ]  (bmatrix)   │
│  ○ Parentheses ( )      (pmatrix)   │
│  ○ Vertical bars | |    (vmatrix)   │
│  ○ Curly braces { }     (Bmatrix)   │
│                                     │
│  Preview:                           │
│  ┌─────────────┐                    │
│  │ □  □        │                    │
│  │ □  □        │                    │
│  └─────────────┘                    │
│                                     │
│  [Create Matrix]  [Cancel]          │
└─────────────────────────────────────┘
```

---

## Conclusion

### ✅ What We Have
- Parser supports arbitrary-size matrices
- Renderer handles any size
- Generic `"matrix"` operation for non-standard sizes

### ⚠️ What We Need
- Matrix builder UI for custom sizes
- Better palette organization (2×2, 3×3, custom)
- Fix Matrix 3×3 template (currently broken)

### 🎯 Recommended Approach
1. **Keep quick buttons** for 2×2, 3×3 (most common)
2. **Add "Custom Size" button** that opens matrix builder dialog
3. **Support all delimiter types** (brackets, parens, bars, braces)
4. **Limit to 10×10** for structural mode performance
5. **Text mode fallback** for larger matrices

---

## Implementation Checklist

- [ ] Fix Matrix 3×3 template in palette
- [ ] Add pmatrix and vmatrix buttons (2×2, 3×3)
- [ ] Create matrix builder modal UI
- [ ] Implement matrix generator function
- [ ] Add delimiter type selector
- [ ] Add size validation (1-10 rows/cols)
- [ ] Add preview in dialog
- [ ] Test with various sizes
- [ ] Add keyboard shortcut (Ctrl+M)
- [ ] Update documentation

**Estimated effort:** 1-2 days for full implementation  
**Priority:** Medium (nice-to-have, not critical)  
**Dependencies:** None (can be done independently)

---

**Bottom Line:** The backend already handles arbitrary-size matrices perfectly. We just need a good UI/UX layer in the palette to let users create them easily. A matrix builder dialog is the best solution for flexibility without cluttering the palette.

