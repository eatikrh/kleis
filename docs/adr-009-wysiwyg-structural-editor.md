# ADR-009: Structural Editor with User-Extensible Templates

**Status:** Proposed  
**Date:** 2024-11-21  
**Context:** With unified AST (ast.rs), parser, and renderer integration complete  
**Extends:** ADR-006 (Template-Grammar Duality), ADR-005 (Visual Authoring), ADR-004 (Input Strategy)

---

## Context

Traditional mathematical editors follow one of two paradigms:

1. **Text-based LaTeX editors** (Overleaf, TeXstudio)
   - User types: `\frac{1}{2} \sqrt{\pi}`
   - Pro: Full LaTeX control
   - Con: Syntax errors, learning curve, cognitive overhead

2. **WYSIWYG equation editors** (Microsoft Word, MathType)
   - User clicks buttons/menus
   - Pro: Visual, intuitive
   - Con: Proprietary formats, limited export, **not extensible**

**Neither approach supports user-defined mathematical notation or operations.**

Traditional editors treat mathematics as **fixed notation**. You can only use pre-defined symbols and structures.

## The Kleis Vision: Extensible Mathematical Language

Kleis enables users to **define new mathematical operations** with:
- Type signatures (semantic structure)
- Glyphs (visual symbols)
- Templates (rendering in LaTeX, Unicode, etc.)
- Laws (semantic constraints)

**Example from kleis_vision_executable_math.md:**
```kleis
operation wedge : (Form, Form) -> Form
template wedge {
    glyph: "∧", 
    latex: "{left} \wedge {right}", 
    unicode: "{left} ∧ {right}"
}
```

This **simultaneously defines**:
- Grammar structure (2 arguments: left, right)
- Visual representation (∧ symbol)
- Rendering behavior (how it looks in different targets)

**From ADR-006:** Templates and grammar are **dual** - the template placeholders (`{left}`, `{right}`) define the grammar structure.

---

## Decision

Build a **User-Extensible Structural Editor** that:
- Edits AST directly (not text)
- Auto-generates UI from operation definitions
- Uses templates to define interaction structure
- Makes invalid expressions impossible
- Supports user-defined operations and glyphs

---

## Architecture

### Core Principle: Placeholder-Driven Structure

Instead of rendering `½` as static output, render it as:

```
┌─────┐
│  □  │  ← Clickable placeholder (AST args[0])
├─────┤
│  □  │  ← Clickable placeholder (AST args[1])
└─────┘
```

Each placeholder is **bound to an AST node**.

### The AST Structure with Placeholders

```rust
// Unified Expression from ast.rs
pub enum Expression {
    Const(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
    Placeholder { id: usize, hint: String },  // NEW
}
```

### User Interaction Flow

**Example: Creating (x+y)/2**

1. **User clicks "Fraction" template**
   ```rust
   ast = Operation {
       name: "scalar_divide",
       args: [
           Placeholder { id: 0, hint: "numerator" },
           Placeholder { id: 1, hint: "denominator" }
       ]
   }
   ```

2. **Renderer displays:**
   ```html
   <div class="fraction">
     <div class="numerator placeholder active" data-id="0">□</div>
     <div class="fraction-bar"></div>
     <div class="denominator placeholder" data-id="1">□</div>
   </div>
   ```

3. **User types "x+y" in top placeholder**
   ```rust
   // Parse fragment into sub-AST
   let parsed = parse_latex("x+y")?;
   ast.args[0] = parsed;  // Replace placeholder
   ```

4. **Renderer updates (no full re-parse):**
   ```html
   <div class="numerator">
     <span class="var">x</span>
     <span class="op">+</span>
     <span class="var">y</span>
   </div>
   ```

5. **User tabs to bottom placeholder, types "2"**
   ```rust
   ast.args[1] = Const("2");
   ```

6. **Final AST:**
   ```rust
   Operation("scalar_divide", [
       Operation("plus", [Object("x"), Object("y")]),
       Const("2")
   ])
   ```

---

## Key Features

### 1. Template Insertion

**Templates map directly to AST operations:**

| Template | Visual | AST Structure |
|----------|--------|---------------|
| Fraction | `□/□` | `scalar_divide(□, □)` |
| Square Root | `√□` | `sqrt(□)` |
| Integral | `∫ₐᵇ □ dx` | `int_bounds(□, a, b, x)` |
| Matrix 2×2 | `[□ □; □ □]` | `matrix2x2(□, □, □, □)` |
| Summation | `Σᵢ₌ₙᵐ □` | `sum_bounds(□, i=n, m)` |

**JavaScript/WASM Implementation:**
```rust
fn insert_template(template_name: &str) -> Expression {
    match template_name {
        "fraction" => Operation::operation("scalar_divide", vec![
            Expression::placeholder(0, "numerator"),
            Expression::placeholder(1, "denominator")
        ]),
        "sqrt" => Operation::operation("sqrt", vec![
            Expression::placeholder(0, "radicand")
        ]),
        "integral" => Operation::operation("int_bounds", vec![
            Expression::placeholder(0, "integrand"),
            Expression::placeholder(1, "lower bound"),
            Expression::placeholder(2, "upper bound"),
            Expression::placeholder(3, "variable")
        ]),
        // ... all 73 gallery examples as templates
        _ => Expression::placeholder(0, "unknown")
    }
}
```

### 2. Smart Navigation

**Tab traversal follows AST structure:**

```rust
fn next_placeholder(ast: &Expression, current_id: usize) -> Option<usize> {
    // Depth-first traversal of AST
    // Returns next unfilled placeholder
    ast.traverse_dfs()
        .filter(|node| matches!(node, Expression::Placeholder{..}))
        .map(|node| node.id())
        .find(|id| id > current_id)
}
```

**Example: Integral template**
```
∫[□] dx  →  Tab  →  ∫ₐ[□] dx  →  Tab  →  ∫ₐᵇ[□] dx  →  Tab  →  done
 ↑               ↑                    ↑
integrand      lower                upper
```

### 3. Click-to-Edit

**Every rendered element is AST-aware:**

```html
<div class="equation" data-ast-path="[0,1,0]">
  <div class="fraction" data-ast-path="[0,1]">
    <div class="numerator" data-ast-path="[0,1,0]" onclick="editNode([0,1,0])">
      x + y
    </div>
    <div class="denominator" data-ast-path="[0,1,1]" onclick="editNode([0,1,1])">
      2
    </div>
  </div>
</div>
```

**Click handler:**
```javascript
function editNode(path) {
    const node = ast.getNodeAt(path);
    const dialog = showEditDialog(node);
    
    dialog.onSubmit = (newValue) => {
        const parsed = parseLatex(newValue);
        ast.setNodeAt(path, parsed);
        rerender();  // Instant, no full re-parse
    };
}
```

### 4. No Invalid States

**Structure enforcement:**

```rust
// User can't create: \frac{1}{2}{3}  (fraction with 3 args)
// Because AST enforces:
Operation {
    name: "scalar_divide",
    args: [_, _]  // Exactly 2 args, enforced by type
}
```

**Template validation:**
```rust
fn validate_template(op_name: &str, args: &[Expression]) -> Result<(), Error> {
    match op_name {
        "scalar_divide" if args.len() != 2 => Err("Fraction needs 2 args"),
        "sqrt" if args.len() != 1 => Err("Sqrt needs 1 arg"),
        "matrix2x2" if args.len() != 4 => Err("2x2 matrix needs 4 args"),
        _ => Ok(())
    }
}
```

---

## Implementation Path

### Phase 1: Core Infrastructure ✅ (COMPLETE)
- [x] Unified AST (ast.rs)
- [x] Parser (LaTeX → AST)
- [x] Renderer (AST → LaTeX/Unicode)
- [x] Bidirectional flow (parse + render working)
- [x] All 223 tests passing

### Phase 2: Placeholder System (Next)
- [ ] Add `Placeholder` variant to Expression enum
- [ ] Renderer: Display placeholders as clickable boxes
- [ ] Template library: All gallery examples as insertion functions
- [ ] Navigation: Tab through placeholders

### Phase 3: Interactive Editing
- [ ] Click-to-edit: Identify AST path from click coordinates
- [ ] Edit dialog: Inline editor for nodes
- [ ] AST mutation: Update nodes without full re-parse
- [ ] Live preview: Instant re-render on change

### Phase 4: Advanced Features
- [ ] Drag & drop: Restructure AST visually
- [ ] Copy/paste: Preserve structure (not just text)
- [ ] Undo/redo: AST history stack
- [ ] Transformations: Expand, factor, simplify (symbolic operations)
- [ ] Search: Find patterns in AST structure

---

## Technical Benefits

### Current System (Text-Based)
```
User Input → Parser → AST → Renderer → Display
   ↑________________________________________________|
            (full round-trip on every keystroke)
```
**Cost:** Parse + render on every change  
**Errors:** Syntax errors during typing  
**State:** Text is source of truth

### New System (Structure-Based)
```
AST (single source of truth)
 ↓                    ↑
Renderer          User edits
 ↓                    ↑
Display ←───────────→ Interaction
```
**Cost:** Only render (parse eliminated for most edits)  
**Errors:** Impossible (structure enforced)  
**State:** AST is source of truth, text is serialization

### Performance Implications

**Editing large expressions:**

Text-based:
```
User types 1 char → Parse entire document → Rebuild AST → Render all
Time: O(document_size) per keystroke
```

Structure-based:
```
User fills placeholder → Update 1 AST node → Render affected subtree
Time: O(subtree_size) per edit
```

**For a 100-term equation:**
- Text edit: Re-parse 100 terms
- Structure edit: Update 1 node, re-render ~3-5 nodes (local)

---

## Examples

### Example 1: Building Maxwell Equations

**User workflow:**

1. Click "Equals" template
   ```
   □ = □
   ```

2. Click left side, insert "F with mixed index" template
   ```
   F^□_□ = □
   ```

3. Fill superscript: type "μ" (from palette)
   ```
   F^μ_□ = □
   ```

4. Fill subscript: type "ν"
   ```
   F^μ_ν = □
   ```

5. Click right side, insert "Minus" template
   ```
   F^μ_ν = □ - □
   ```

6. Build left term: `∂_μ A_ν` (via templates)
7. Build right term: `∂_ν A_μ`

**Final:** `F^μ_ν = ∂_μ A_ν - ∂_ν A_μ`

**Key point:** At no point did the user type raw LaTeX. They assembled structure.

### Example 2: Nested Fractions

**Building: (1 + √2) / (3 + π)**

1. Insert fraction template: `□ / □`
2. Click numerator, insert sum: `(□ + □) / □`
3. Fill first placeholder: `1`
4. Fill second placeholder: Click sqrt template → `√□` → fill with `2`
5. Result: `(1 + √2) / □`
6. Click denominator, repeat for `3 + π`

**AST Structure:**
```rust
Operation("scalar_divide", [
    Operation("plus", [
        Const("1"),
        Operation("sqrt", [Const("2")])
    ]),
    Operation("plus", [
        Const("3"),
        Object("\\pi")
    ])
])
```

**User never wrote:** `\frac{1 + \sqrt{2}}{3 + \pi}`  
**But can export:** Full LaTeX generated automatically

### Example 3: Empty Slate → Complex Equation

**Goal:** Build Einstein Field Equations from nothing

**Starting state:** Empty canvas, just palette visible

**User workflow (pure clicks):**

1. **Click "=" button** → `□ = □`
2. **Click left placeholder**
3. **Click "+" button** → `□ + □ = □`
4. **Click first sub-placeholder**
5. **Click "G" from palette** → `G + □ = □`
6. **Click "Add subscript" suggestion** → `G_□ + □ = □`
7. **Click subscript placeholder**
8. **Click "μν" pattern button** (or build it) → `G_μν + □ = □`
9. Continue building: Λg_μν on left, κT_μν on right

**Result:** `G_μν + Λg_μν = κT_μν`

**LaTeX export:** `G_{{\mu\nu}} + \Lambda \, g_{{\mu\nu}} = \kappa \, T_{{\mu\nu}}`

**Key points:**
- Started with **empty canvas**
- Never typed LaTeX commands
- Each click: validated structure
- Built complex tensor notation through pure interaction
- LaTeX emerges automatically

### The Paradigm Shift

**Traditional editors:**
```
Mind → Text → Parse → AST → Render → Display
       ^^^^
     Bottleneck: Requires knowing syntax
```

**Kleis structural editor:**
```
Mind → Click → AST → Render → Display
       ^^^^
     Direct: Think structure, click structure
```

**The user's mental model (structure) directly becomes AST**, without the text encoding/decoding step.

### Why Empty Slate + Palette is Revolutionary

**Advantages:**

1. **Zero prerequisites:** Don't need to know LaTeX
2. **Exploratory:** Browse palette to discover operations
3. **Progressive:** Build incrementally, always valid
4. **Guided:** System suggests next steps based on context
5. **Forgiving:** Can't make syntax errors
6. **Discoverable:** See what's possible by browsing palette
7. **Extensible:** New operations = new palette buttons automatically

**Traditional approach:**
```
User must know: \int_{a}^{b} f(x) \, dx
               ^^^^  ^ ^ ^^   ^^  ^^^
           Every symbol, every brace, exact syntax
```

**Kleis approach:**
```
User sees: [∫] button
Clicks it: ∫_□^□ □ dx appears
Fills boxes: a, b, f(x)
Done.
```

**The palette teaches the user what's possible.** You don't need a manual - the UI IS the documentation.

---

## UI/UX Design

### Visual Placeholder States

```css
.placeholder {
    border: 2px dashed #667eea;
    background: #f0f4ff;
    min-width: 20px;
    min-height: 20px;
    cursor: text;
}

.placeholder.active {
    border-color: #ff6b6b;
    background: #fff5f5;
    animation: pulse 1s infinite;
}

.placeholder.filled {
    border: none;
    background: transparent;
}
```

### Smart Palette

**Context-aware suggestions:**

```rust
fn suggest_operations(context: &Expression) -> Vec<Template> {
    match context {
        Placeholder { hint } if hint.contains("exponent") => 
            vec!["Number", "Variable", "Expression"],
        
        Placeholder { hint } if hint.contains("condition") =>
            vec!["<", ">", "≤", "≥", "=", "≠"],
        
        Placeholder { hint } if hint.contains("integrand") =>
            vec!["Fraction", "Product", "Power", "Function"],
        
        _ => all_templates()  // Show everything
    }
}
```

### Keyboard Shortcuts

```
Tab        → Next placeholder
Shift+Tab  → Previous placeholder
Escape     → Clear current placeholder
Enter      → Accept and move to next
/          → Quick insert fraction
^          → Quick insert superscript
_          → Quick insert subscript
Ctrl+/     → Quick insert sqrt
```

---

## Technical Architecture

### Frontend (Browser/WASM)

```rust
// Compile parser + renderer to WASM
#[wasm_bindgen]
pub fn parse_to_ast(latex: &str) -> JsValue {
    let ast = parse_latex(latex).ok()?;
    serde_wasm_bindgen::to_value(&ast).ok()
}

#[wasm_bindgen]
pub fn render_ast(ast: JsValue, target: &str) -> String {
    let expr: Expression = serde_wasm_bindgen::from_value(ast).ok()?;
    let ctx = build_default_context();
    let target = match target {
        "unicode" => RenderTarget::Unicode,
        "latex" => RenderTarget::LaTeX,
        _ => RenderTarget::Unicode
    };
    render_expression(&expr, &ctx, &target)
}
```

### JavaScript/TypeScript Layer

```typescript
class StructuralEditor {
    ast: Expression;
    activeNode: Path;
    
    constructor(initialTemplate: string) {
        this.ast = createTemplate(initialTemplate);
        this.render();
    }
    
    render() {
        // Render AST to HTML with data attributes
        const latex = renderAst(this.ast, 'latex');
        MathJax.typesetPromise([this.container]);
        
        // Attach click handlers to placeholders
        this.attachHandlers();
    }
    
    fillPlaceholder(path: Path, value: string) {
        // Parse just the fragment
        const subAst = parseLatex(value);
        
        // Update AST at path
        this.ast.setNodeAt(path, subAst);
        
        // Re-render only affected region
        this.renderSubtree(path);
    }
    
    nextPlaceholder() {
        // Traverse AST to find next placeholder
        this.activeNode = findNextPlaceholder(this.ast, this.activeNode);
        this.focusNode(this.activeNode);
    }
}
```

---

## Example Templates

### Basic Operations

```rust
// Fraction: a/b
template!("fraction", scalar_divide, [
    placeholder("numerator"),
    placeholder("denominator")
])

// Power: x^n
template!("power", sup, [
    placeholder("base"),
    placeholder("exponent")
])

// Square root: √x
template!("sqrt", sqrt, [
    placeholder("radicand")
])

// Nth root: ⁿ√x
template!("nth_root", nth_root, [
    placeholder("radicand"),
    placeholder("index")
])
```

### Calculus

```rust
// Integral: ∫ₐᵇ f(x) dx
template!("integral", int_bounds, [
    placeholder("integrand"),
    placeholder("lower"),
    placeholder("upper"),
    placeholder("variable")
])

// Sum: Σᵢ₌ₙᵐ expr
template!("sum", sum_bounds, [
    placeholder("body"),
    placeholder("from"),
    placeholder("to")
])

// Derivative: df/dx
template!("derivative", derivative, [
    placeholder("function"),
    placeholder("variable")
])

// Partial: ∂f/∂x
template!("partial", d_part, [
    placeholder("function"),
    placeholder("variable")
])
```

### Linear Algebra

```rust
// Matrix 2×2
template!("matrix_2x2", matrix2x2, [
    placeholder("a11"), placeholder("a12"),
    placeholder("a21"), placeholder("a22")
])

// Bra-ket: |ψ⟩
template!("ket", ket, [
    placeholder("state")
])

// Inner product: ⟨ψ|φ⟩
template!("inner", inner, [
    placeholder("bra"),
    placeholder("ket")
])
```

### Physics

```rust
// Commutator: [A, B]
template!("commutator", commutator, [
    placeholder("A"),
    placeholder("B")
])

// Tensor with mixed indices: T^μ_ν
template!("tensor_mixed", index_mixed, [
    placeholder("base"),
    placeholder("upper"),
    placeholder("lower")
])

// Covariant derivative: ∇_μ V^ν
template!("covariant_derivative", covariant_deriv, [
    placeholder("tensor"),
    placeholder("index")
])
```

---

## Advantages Over Text Editing

### 1. No Syntax Errors

**Text editor:**
```latex
\frac{1{2}           ← Missing }
\sqrt[\pi}           ← Wrong bracket
\begin{matrix}...    ← Missing \end
```

**Structural editor:**
```
Cannot happen - structure is always valid
```

### 2. Semantic Guidance

**Text editor:** User must know LaTeX commands  
**Structural editor:** Browse templates visually

### 3. Incremental Building

**Text editor:** Must type complete expression  
**Structural editor:** Build piece by piece, always valid

### 4. Visual Feedback

**Text editor:** See result after compilation  
**Structural editor:** Live WYSIWYG as you build

### 5. Refactoring

**Text editor:**
```latex
Change: \frac{1}{2}  →  \frac{1}{2}^2
Must: Position cursor, type ^{2}
Risk: Typos, mismatched braces
```

**Structural editor:**
```
Click fraction → "Wrap in..." → "Power"
AST: scalar_divide → sup(scalar_divide, □)
Fill □: 2
Done: No text manipulation, no errors
```

---

## Comparison to Existing Systems

| Feature | LaTeX Editor | Word/MathType | **Kleis Structural** |
|---------|--------------|---------------|---------------------|
| **Format** | Text-based | Proprietary | AST (LaTeX export) |
| **Editing** | Type code | Click buttons | Edit structure |
| **Validation** | On compile | Partial | Always valid |
| **Export** | Native LaTeX | Poor LaTeX | Perfect LaTeX |
| **Import** | Native | Limited | Full LaTeX parse |
| **Errors** | Syntax errors | Invalid clicks | Impossible |
| **Learning Curve** | High | Low | Low |
| **Power** | Full LaTeX | Limited | **Full LaTeX + extensible** |
| **Speed** | Slow (typing) | Slow (clicking) | Fast (structure) |
| **Extensibility** | None | None | **User-defined operations** |
| **Packages** | N/A | N/A | **Import/export notation systems** |
| **Starting Point** | Text field | Blank doc | **Empty slate + palette** |

---

## The "Empty Slate" Workflow

### Starting From Nothing

**The Vision:** User opens editor to blank canvas. No text field, no cursor, just:
- Empty equation space
- Symbol palette (auto-generated from loaded operations)
- Template buttons (one for each defined operation)

**This is the pure structural approach:**
- Start with nothing
- Build by clicking palette buttons
- Each click adds validated structure
- LaTeX emerges as byproduct, not input

**Step-by-step example:**

```
Initial state:
┌────────────────────┐
│                    │  ← Empty workspace
│    (click here     │
│     to start)      │
│                    │
└────────────────────┘
```

1. **User clicks "Fraction" button**
   ```
   ┌────────────────────┐
   │       ┌───┐        │
   │       │ □ │        │  ← Top placeholder active (blue border)
   │       ├───┤        │
   │       │ □ │        │
   │       └───┘        │
   └────────────────────┘
   ```
   
   AST: `Operation("scalar_divide", [Placeholder(0), Placeholder(1)])`

2. **User clicks "1" from number palette** (or types it)
   ```
   ┌────────────────────┐
   │       ┌───┐        │
   │       │ 1 │        │  ← Filled
   │       ├───┤        │
   │       │ □ │        │  ← Auto-focus next placeholder
   │       └───┘        │
   └────────────────────┘
   ```
   
   AST: `Operation("scalar_divide", [Const("1"), Placeholder(1)])`

3. **User clicks "Square Root" button**
   ```
   ┌────────────────────┐
   │       ┌───┐        │
   │       │ 1 │        │
   │       ├───┤        │
   │       │√□ │        │  ← Sqrt with placeholder
   │       └───┘        │
   └────────────────────┘
   ```
   
   AST: `Operation("scalar_divide", [Const("1"), Operation("sqrt", [Placeholder(2)])])`

4. **User clicks π from Greek palette**
   ```
   ┌────────────────────┐
   │       ┌───┐        │
   │       │ 1 │        │
   │       ├───┤        │
   │       │√π │        │  ← Complete!
   │       └───┘        │
   └────────────────────┘
   ```
   
   AST: `Operation("scalar_divide", [Const("1"), Operation("sqrt", [Object("\\pi")])])`

**Result:** Built `1/√π` entirely through clicks. Zero typing, zero LaTeX knowledge needed.

### Adding More Terms

5. **Workspace shows: 1/√π**  
   User clicks space after it, then clicks "×" button
   
   ```
   1/√π  ×  □  ← New placeholder for next term
   ```

6. **User clicks "ℵ" (aleph) from Hebrew section**
   
   ```
   1/√π  ×  ℵ  ← Still incomplete, subscript expected?
   ```

7. **System suggests: "Add subscript?" → User clicks "Yes"**
   
   ```
   1/√π  ×  ℵ_□  ← Subscript placeholder added
   ```

8. **User types "0"**
   
   ```
   1/√π  ×  ℵ₀  ← Complete!
   ```

**Final AST:**
```rust
Operation("scalar_multiply", [
    Operation("scalar_divide", [
        Const("1"),
        Operation("sqrt", [Object("\\pi")])
    ]),
    Operation("sub", [
        Object("\\aleph"),
        Const("0")
    ])
])
```

**Export to LaTeX:** `\frac{1}{\sqrt{\pi}} \aleph_0`

### Pure Click-Based Building

**User journey building Schrödinger equation: Ĥ|ψ⟩ = E|ψ⟩**

1. Empty slate
2. Click "Equals" → `□ = □`
3. Click left box → Click "Hat" → `□̂ = □`
4. Click box under hat → Click "H" from palette → `Ĥ = □`
5. Click space after Ĥ → Click "Ket" → `Ĥ|□⟩ = □`
6. Click box inside ket → Click "ψ" → `Ĥ|ψ⟩ = □`
7. Click right side → Click "E" → `Ĥ|ψ⟩ = E`
8. Click after E → Click "Ket" → `Ĥ|ψ⟩ = E|□⟩`
9. Click box → Click "ψ" → `Ĥ|ψ⟩ = E|ψ⟩`

**Done! Never typed LaTeX, never saw syntax errors.**

## Real-World Workflow

### Scenario: Student Solving Integral

**Traditional (LaTeX):**
1. Type: `\int_{0}^{\infty} e^{-x^2} dx`
2. Error: Missing space before dx
3. Fix: Add `\,`
4. Error: Wrong exponent grouping
5. Fix: Add braces around `-x^2`
6. Finally works after 3 errors

**Structural (Kleis):**
1. Click "Integral" template → `∫ₐᵇ □ dx` appears
2. Tab to lower bound → type "0"
3. Tab to upper bound → type "\infty" (or click ∞ button)
4. Tab to integrand → type "e^{-x^2}" (or build via templates)
5. Done: No errors possible, always valid

### Scenario: Researcher Building Equation

**Traditional:**
```latex
% Building: Euler-Lagrange equation
% 20 minutes later...
\frac{\partial L}{\partial y} - \frac{d}{dx}\left(\frac{\partial L}{\partial y'}\right) = 0
% Finally works after fixing bracket matching
```

**Structural:**
```
1. Click "Equals" → □ = □
2. Left: Click "Minus" → □ - □ = □
3. First term: Click "Partial" → ∂□/∂□
4. Fill: L, y
5. Second term: Click "Total Deriv" → d□/dx
6. Inner: Click "Partial" again → ∂L/∂y'
7. Right side: type "0"
Done in 2 minutes, zero errors
```

---

## Why This Works for Kleis

### 1. Complete Parser Coverage
- 100% of render.rs patterns parse ✅
- All gallery examples tested ✅
- Implicit multiplication working ✅

### 2. Unified AST
- Single Expression type ✅
- No conversion overhead ✅
- Direct parser → renderer flow ✅

### 3. Bidirectional Rendering
- AST → LaTeX ✅
- AST → Unicode ✅
- LaTeX → AST ✅

### 4. Template System Ready
- 73 gallery examples = 73 templates ✅
- Helper functions for every operation ✅
- Rendering tested for all patterns ✅

---

## Future: Executable Mathematics

Once the structural editor works, you can add:

### Semantic Annotations
```rust
Expression {
    Const(String),
    Object(String),
    Operation { name, args, type_info: Option<Type> },  // NEW
}

enum Type {
    Scalar,
    Vector(usize),
    Matrix(usize, usize),
    Tensor { contravariant: Vec<usize>, covariant: Vec<usize> },
}
```

### Type Checking
```rust
fn typecheck(expr: &Expression) -> Result<Type, TypeError> {
    match expr {
        Operation { name: "scalar_multiply", args } => {
            let t1 = typecheck(&args[0])?;
            let t2 = typecheck(&args[1])?;
            compatible_multiplication(t1, t2)?
        }
        // Catch index errors, dimension mismatches, etc.
    }
}
```

### Symbolic Evaluation
```rust
fn simplify(expr: Expression) -> Expression {
    match expr {
        Operation { name: "plus", args } 
            if matches!(args[0], Const("0")) => args[1],
        Operation { name: "scalar_multiply", args }
            if matches!(args[0], Const("1")) => args[1],
        // ... hundreds of rules
    }
}
```

### Connection to Kleis Vision

This structural editor becomes the **input layer** for:
- `kleis/ontology.kleis` - Define mathematical structures
- `kleis/physics.kleis` - Express physics equations
- `kleis/cosmology.kleis` - Build cosmological models

Users **author** in the visual editor, which generates **executable** Kleis expressions.

---

## User-Extensible Operations: The Meta-Level

### Beyond Built-In Operations

The revolutionary aspect is that **the editor itself is extensible**. When a user defines:

```kleis
operation my_transform : (Tensor, Field) -> Scalar
template my_transform {
    glyph: "⊛",
    latex: "{left} \\circledast {right}",
    unicode: "{left} ⊛ {right}",
    placeholders: ["tensor", "field"]
}
```

**The visual editor automatically:**

1. **Adds palette button**
   ```javascript
   // Auto-generated from template definition
   palette.addButton({
       symbol: "⊛",
       label: "my_transform",
       onClick: () => insertTemplate("my_transform")
   });
   ```

2. **Generates insertion function**
   ```rust
   fn insert_my_transform() -> Expression {
       Operation::operation("my_transform", vec![
           Placeholder { id: gen_id(), hint: "tensor" },
           Placeholder { id: gen_id(), hint: "field" }
       ])
   }
   ```

3. **Validates structure**
   ```rust
   // Template has {left} and {right}
   // Therefore: requires exactly 2 arguments
   // Editor enforces this automatically
   ```

4. **Renders correctly**
   ```rust
   // Uses the template the user provided
   render_expression(&my_op_ast, &ctx, &RenderTarget::Unicode)
   // → "tensorExpr ⊛ fieldExpr"
   ```

### Package-Level Extensibility

**Import a mathematical package:**
```kleis
import differential-geometry from kleis-pkg-diffgeo

// Package contains:
// - operation covariant_deriv : (TensorField, Index) -> TensorField
// - template covariant_deriv { glyph: "∇", ... }
// - operation lie_bracket : (VectorField, VectorField) -> VectorField
// - template lie_bracket { latex: "[{left}, {right}]", ... }
```

**Editor automatically:**
- Loads all operations from package
- Adds buttons to palette (∇, [...], etc.)
- Knows placeholder structure from templates
- Validates according to type signatures

**User experience:**
```
1. Import package
2. Palette updates with new symbols
3. Click ∇ button → Inserts covariant_deriv with correct placeholders
4. Fill placeholders
5. Export: Perfect LaTeX using package's templates
```

### Current System as Foundation

**What we have now (render.rs):**
```rust
// Hard-coded operations and templates
latex_templates.insert("scalar_divide", r"\frac{{num}}{{den}}");
latex_templates.insert("sqrt", r"\sqrt{{arg}}");
// ... 73 operations
```

**What's next (user-extensible):**
```rust
// Runtime-loaded from .kleis definitions
fn load_operation_templates(package: &KleisPackage) {
    for op in package.operations {
        latex_templates.insert(op.name, op.template.latex);
        unicode_templates.insert(op.name, op.template.unicode);
        
        // Auto-generate palette button
        ui.add_template_button(op.name, op.template.glyph);
    }
}
```

### Automatic UI Generation from Templates

**Key insight from ADR-006:** Template structure → Grammar structure → UI structure

When user defines:
```kleis
operation wedge : (Form, Form) -> Form
template wedge {
    glyph: "∧",
    latex: "{left} \\wedge {right}",
    unicode: "{left} ∧ {right}"
}
```

**System extracts:**
1. **Arity:** 2 arguments (sees `{left}` and `{right}`)
2. **Placeholder hints:** "left" and "right" 
3. **Glyph:** "∧" for palette button
4. **Rendering:** Template strings for output

**Auto-generates:**
```rust
fn insert_wedge_template() -> Expression {
    Operation {
        name: "wedge",
        args: vec![
            Placeholder { id: 0, hint: "left" },   // From {left}
            Placeholder { id: 1, hint: "right" }   // From {right}
        ]
    }
}
```

**Palette button:**
```html
<button class="symbol-btn" 
        data-operation="wedge"
        onclick="insertOperation('wedge')">
    ∧
</button>
```

**This means:**
- Define operation once in `.kleis`
- Editor automatically provides UI
- No manual UI code needed
- Works for ANY user-defined operation

**The template IS the UI specification.** (ADR-006 principle)

### The Living Mathematics Cycle

**From kleis_vision_executable_math.md:**

```
1. Mathematician defines algebra
   ↓
2. Binds glyphs + templates
   ↓
3. Packages for distribution
   ↓
4. Others import package
   ↓
5. Visual editor updates automatically
   ↓
6. Users compose using new notation
   ↓
7. Export, share, iterate
```

**Example: Differential Geometry Package**

**Package author creates:**
```kleis
// Define operations
operation christoffel : (Metric, Index, Index, Index) -> Scalar
template christoffel {
    glyph: "Γ",
    latex: "\\Gamma^{{{idx1}}}_{{{idx2} {idx3}}}",
    unicode: "Γ^{idx1}_{idx2 idx3}"
}

operation riemann : (Metric, Index, Index, Index, Index) -> Scalar  
template riemann {
    glyph: "R",
    latex: "R^{{{idx1}}}_{{{idx2} {idx3} {idx4}}}",
    unicode: "R^{idx1}_{idx2 idx3 idx4}"
}
```

**Package user experience:**
```javascript
// Import package
import("kleis-pkg-diffgeo");

// Palette automatically shows new buttons:
// [Γ] [R] [∇] ...

// Click Γ button → Inserts:
Γ^□_□□  // 4 placeholders, pre-structured

// Fill placeholders by clicking palette or typing
// System knows structure from template
```

**Impact:**
- **Notation becomes shareable code**
- **Mathematical systems become installable packages**
- **Visual editor adapts to ANY mathematical domain**
- **Users define languages, not just expressions**

### Example: User Creates Custom Notation

**User writes in `my-algebra.kleis`:**
```kleis
// Define a new operation
operation tensor_contract : (Tensor, Index, Index) -> Tensor
template tensor_contract {
    glyph: "⊗",
    latex: "{tensor}^{{upper}}_{{{lower}}}",
    unicode: "{tensor}^{upper}_{lower}",
    placeholders: ["tensor", "upper", "lower"]
}
```

**Editor automatically generates:**

1. **Palette button with ⊗ symbol**
2. **Insertion creates 3 placeholders:**
   ```
   ┌─────┐
   │  □  │ ← tensor (placeholder 0)
   └─────┘
      ^
      │ ← upper (placeholder 1)
      │
      └─ lower (placeholder 2)
   ```

3. **Tab order:** tensor → upper → lower
4. **Type validation:** Checks signature `(Tensor, Index, Index)`
5. **Rendering:** Uses provided templates

**User never writes UI code.** The template definition IS the UI specification.

### The Glyph-Template-Grammar Triangle

```
         GLYPH
          ∧
         /  \
        /    \
       /      \
   TEMPLATE  GRAMMAR
      │        │
      └────┬───┘
           │
      placeholder
      structure
```

**From ADR-006:** These three are synchronized:
- **Glyph**: What users see/click ("∧")
- **Template**: Placeholder structure (`{left} ∧ {right}`)
- **Grammar**: Expected arguments (2 args: left, right)

**When user defines operation:**
- Template placeholders → Define grammar expectations
- Glyph → Becomes palette button
- Type signature → Enables validation

**Visual editor uses this to:**
1. Generate insertion UI (palette buttons)
2. Create placeholder structure (from template)
3. Validate fills (from type signature)
4. Render result (from template)

**All automatic. All user-defined.**

---

## Conclusion

The unified AST makes structural editing not just possible, but **natural and extensible**:

- **Users define operations:** New mathematical structures
- **Templates define interaction:** Placeholder structure = UI structure
- **Editor generates UI:** Palette buttons from glyphs automatically
- **Grammar enforces validity:** Type signatures from operation definitions
- **AST represents everything:** No text, no fixed notation, pure structure
- **LaTeX is one target:** Also Unicode, possibly SVG, MathML, etc.

This is the missing piece between:
- **Mathematical creativity** (define new systems)
- **Visual authoring** (build with defined systems)
- **Computational mathematics** (execute, validate)
- **Publication** (export to LaTeX, papers)
- **Sharing** (package and distribute new notation)

### The Revolutionary Aspect

**Traditional:** Mathematics notation is **fixed** (LaTeX commands are hard-coded)

**Kleis:** Mathematics notation is **programmable** (users define operations + templates → system adapts)

This transforms mathematics from **static notation** to **live, extensible structure**.

Users don't just edit equations - they **define new mathematical languages** and edit within them.

---

## Next Steps

### Phase 1: Foundation ✅ (COMPLETE)
- [x] Unified AST (ast.rs)
- [x] Parser (LaTeX → AST)
- [x] Renderer (AST → LaTeX/Unicode)
- [x] Template system (GlyphContext with 73 operations)
- [x] 223 tests passing

### Phase 2: Placeholder System (Next)
- [ ] Add `Placeholder` variant to Expression enum
- [ ] Renderer: Display placeholders as clickable boxes
- [ ] Template parser: Extract placeholders from template strings
- [ ] Navigation: Tab through placeholders following template order

### Phase 3: Basic Structural Editor
- [ ] Empty slate workflow
- [ ] Click palette → Insert template with placeholders
- [ ] Click placeholder → Fill with value or sub-expression
- [ ] Export to LaTeX

### Phase 4: Template Loader (User-Extensible)
- [ ] Parse `.kleis` operation definitions
- [ ] Extract glyph + template bindings
- [ ] Auto-generate palette buttons
- [ ] Dynamic template insertion based on user definitions

### Phase 5: Package System
- [ ] Import/export `.kleis` packages
- [ ] Load operations + templates at runtime
- [ ] Palette updates with package operations
- [ ] Type validation from operation signatures

---

## References

- **adr-006-template-grammar-duality.md** - Templates ARE grammar structure
- **adr-005-visual-authoring.md** - Visual authoring for symbolic systems
- **adr-004-input-visualization.md** - Visual input methods
- **kleis_vision_executable_math.md** - Living, extensible mathematical structure
- **src/ast.rs** - Unified Expression type (NEW)
- **src/render.rs** - Template system (GlyphContext)
- **static/index.html** - Current web UI (foundation for structural version)

---

---

## Why The Foundation is Ready

### Infrastructure Complete ✅

**1. Unified AST (`src/ast.rs`):**
```rust
pub enum Expression {
    Const(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
    // Ready to add: Placeholder { id: usize, hint: String }
}
```

**2. Template System (`src/render.rs`):**
```rust
pub struct GlyphContext {
    unicode_glyphs: HashMap<String, String>,     // ∧, ∇, etc.
    unicode_templates: HashMap<String, String>,  // {left} ∧ {right}
    latex_glyphs: HashMap<String, String>,       // \wedge, \nabla
    latex_templates: HashMap<String, String>,    // {left} \wedge {right}
}
```
- 73 operations defined
- Template → Placeholder structure already working
- Just need to expose placeholders to UI

**3. Parser Integration:**
```rust
pub fn latex_to_unicode(latex: &str) -> Result<String, ParseError>
pub fn latex_to_latex(latex: &str) -> Result<String, ParseError>
```
- Parse any LaTeX → AST
- AST → Render to any target
- Bidirectional flow working

**4. Web UI (`static/index.html`):**
- Symbol palette with 46 Greek/Hebrew letters
- Template buttons
- MathJax rendering
- Just needs: placeholder rendering + click handlers

### What's Missing (Minimal)

**To enable structural editing:**

1. **Add Placeholder variant** (5 lines of code)
2. **Render placeholders as boxes** (CSS + HTML generation)
3. **Click handler:** `onClick(placeholder_id) → editPlaceholder(id)`
4. **Tab navigation:** Find next placeholder in AST

**To enable extensibility:**

5. **Parse operation definitions** from `.kleis` files
6. **Extract templates** and add to GlyphContext
7. **Generate palette buttons** from glyphs
8. **Load on startup** or on-demand

**Everything else is already working.**

### From Foundation to Revolutionary Interface

**Current state:**
- Text input → Parser → AST → Renderer → Display ✅
- All 223 tests passing ✅
- Template system proven ✅

**Add placeholders:**
- Empty canvas + Palette ✅
- Click button → Insert template with placeholders ✅
- Fill placeholders → Build equation ✅

**Add operation loader:**
- User defines operations ✅
- System generates UI ✅
- Notation becomes programmable ✅

**The hard part (mathematical correctness) is done.**  
**The remaining work is UI/UX.**

---

**Status:** Architecture validated, foundation complete, ready for implementation  
**Impact:** Transforms fixed-notation editor into **extensible mathematical language authoring system**  
**Scope:** Not just editing LaTeX - **defining and using new mathematical notations**  
**Unique Value:** Empty slate + palette workflow with user-extensible operations (nothing else like this exists)  
**Risk:** Low (all infrastructure in place, only UI work remains)  
**Effort:** Phase 2-3: ~2-3 weeks, Phase 4-5: ~2-3 months  
**Foundation:** **100% ready** - unified AST, template system, parser/renderer, 223 tests passing

