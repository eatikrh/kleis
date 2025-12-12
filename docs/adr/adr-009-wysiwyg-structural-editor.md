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

### Architecture Documents
- **[adr-006-template-grammar-duality.md](adr-006-template-grammar-duality.md)** - Templates ARE grammar structure
- **[adr-005-visual-authoring.md](adr-005-visual-authoring.md)** - Visual authoring for symbolic systems
- **[adr-004-input-visualization.md](adr-004-input-visualization.md)** - Visual input methods
- **[kleis_vision_executable_math.md](../vision/kleis_vision_executable_math.md)** - Living, extensible mathematical structure
### Implementation Guides
- **[template-implementation-strategy.md](../archive/template-implementation-strategy.md)** - Detailed implementation strategy for template system, placeholder rendering, palette generation, and AST traversal strategies

### Code References
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

## Why Traditional WYSIWYG Math Editors Are Hard

### The Core Challenge: Mathematics is 2D

Standard web technologies (HTML/CSS/contenteditable) are designed for **linear text** flowing down a page. Mathematical notation is intrinsically **two-dimensional**:

- Fractions stack vertically: $\frac{a}{b}$
- Exponents and indices move diagonally: $x^2$, $a_i$
- Square roots and integrals stretch dynamically: $\sqrt{x}$, $\int f(x)dx$
- Matrices are grids: $\begin{bmatrix}a&b\\c&d\end{bmatrix}$

**Critical insight:** You cannot use HTML `contenteditable` for math. The browser will destroy nested structures when the user presses backspace near a fraction.

### The "Cursor Nightmare"

Traditional WYSIWYG math editors face an extraordinarily difficult problem: **tracking cursor position in 2D nested mathematical structures**.

**Example scenario:** User has cursor inside a square root: $\sqrt{x|}$

Questions the editor must answer:
- User presses Right Arrow: Does cursor stay inside root or exit?
- User presses Up Arrow: Where does cursor go? (no content above)
- User types `/`: Should it create fraction? Where does numerator come from?
- User presses Backspace at start of root: Delete the root symbol? The content?

**The traditional solution requires:**
1. Continuous 2D coordinate tracking
2. Complex geometry calculations for cursor placement
3. State machines for handling every keystroke in every context
4. Heuristics for "what user probably meant"
5. Partial expression parsing on every keystroke

**Result:** This is why building math WYSIWYG from scratch typically takes a team of engineers 6+ months.

---

## Comparison with Existing Tools

### Option 1: Specialized Math Input Libraries

**MathLive** (cortex-js/mathlive)
- Modern Web Component for WYSIWYG math input
- Excellent typing experience with smart transformations
- Exports to LaTeX, MathML, spoken text
- No jQuery dependency, mobile support included

**MathQuill** (mathquill/mathquill)
- Powers Desmos graphing calculator
- Battle-tested, intuitive typing experience
- Older codebase (jQuery-based, though modern forks exist)

**Pros:**
- Complete, polished user experience
- Cursor management solved
- Years of edge case handling

**Cons (Critical for Kleis):**
- **Fixed notation set** - cannot add user-defined operations
- **No AST access** - black box input → LaTeX output
- **Not extensible** - cannot auto-generate UI from operation definitions
- **Kills the revolutionary aspect** - becomes just a fancy LaTeX input

**Verdict:** These tools solve traditional WYSIWYG but **abandon the Kleis vision** of extensible, user-defined mathematical notation.

### Option 2: Rich Text Editor Frameworks

**ProseMirror + math plugin**
- General-purpose document editor framework
- Math equations as "nodes" in document tree
- Typically embeds MathQuill/MathLive for math regions

**Pros:**
- Excellent for mixed content (paragraphs + equations)
- Professional document editing features

**Cons:**
- **Massive overkill** for equation-only editing
- Steep learning curve
- Same extensibility limitations as Option 1
- Complex integration overhead

**Verdict:** Wrong tool for this job. Designed for Google Docs-style editors, not mathematical language authoring.

### Option 3: Kleis Structural Editor (This ADR)

**The key difference:** Kleis **avoids the cursor nightmare entirely** by not having a free-moving cursor.

| Traditional WYSIWYG | Kleis Structural |
|---------------------|------------------|
| Continuous cursor in 2D space | **Discrete placeholder selection** |
| Calculate cursor movement through nested structures | **AST traversal** (simple tree walk) |
| Parse partial expressions continuously | **Parse only when filling placeholders** |
| Complex 2D coordinate geometry | **Track active placeholder ID (integer)** |
| Heuristics for "what user meant" | **Explicit structure insertion** |
| Keyboard handles all input | **Click/keyboard hybrid** |

**Why this is tractable:**
- No 2D cursor position to track
- No ambiguity about "where am I"
- Navigation = AST tree traversal (solved problem)
- Input = "fill this specific placeholder" (bounded problem)
- Structure = explicit template insertion (no guessing)

**Why this enables extensibility:**
- AST is accessible (not hidden)
- Templates define structure (not hard-coded)
- UI auto-generates from templates
- User operations = new palette buttons automatically

**Verdict:** This is the **only approach** that satisfies both usability AND extensibility requirements.

---

## Implementation Technology Options

### Frontend Rendering Technologies

**Option A: Server-Rendered HTML**

Current approach in `index.html`:
```javascript
// Server renders AST to HTML with clickable placeholders
fetch('/api/render_ast', { ast: currentAST, format: 'html' })
// Receive HTML string with onclick handlers
structural.innerHTML = htmlFromServer;
```

**Pros:**
- Rendering logic in Rust (consistent with parser)
- Can reuse render.rs templates
- Server-side validation

**Cons:**
- Network round-trip on every edit
- Server dependency for rendering
- Latency for interaction

**Option B: WebAssembly + DOM Manipulation**

Compile Rust renderer to WASM, call from JavaScript:
```rust
// In Rust, compiled to WASM
#[wasm_bindgen]
pub fn render_to_html(ast: JsValue) -> String {
    let expr: Expression = serde_wasm_bindgen::from_value(ast)?;
    let ctx = build_default_context();
    render_expression(&expr, &ctx, &RenderTarget::HTML)
}
```

```javascript
// In JavaScript
import init, { render_to_html } from './kleis_wasm.js';
await init();
const html = render_to_html(currentAST);
structural.innerHTML = html;
```

**Pros:**
- **No server round-trip** - instant rendering
- **Reuses Rust code** - same renderer as server
- Works offline
- Near-native performance

**Cons:**
- WASM cannot directly manipulate DOM (must return HTML string)
- Slightly larger initial download (~few hundred KB)

**Option C: WebAssembly + SVG Generation**

More advanced: Rust generates SVG elements via `web-sys`:
```rust
use wasm_bindgen::prelude::*;
use web_sys::{Document, SvgElement, SvgCircleElement};

#[wasm_bindgen]
pub fn render_to_svg(ast: JsValue) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let svg_ns = "http://www.w3.org/2000/svg";
    
    // Create SVG container
    let svg = document.create_element_ns(Some(svg_ns), "svg")?
        .dyn_into::<SvgElement>()?;
    
    // Render expression as SVG paths, text, shapes
    render_ast_to_svg(&expr, &svg, &document)?;
    
    // Each placeholder is an SVG <rect> with onclick
    Ok(())
}
```

**Pros:**
- **Full control over rendering** - pixel-perfect layout
- **Preserved interactivity** - clickable regions survive rendering
- **Scalable/zoomable** - SVG is resolution-independent
- **Avoids MathJax limitations** - no loss of clickability

**Cons:**
- Most complex option
- Need to implement math layout (fraction bars, superscript positioning, etc.)
- Larger scope than other options

**Option D: Client-Side JavaScript Renderer**

Rewrite render.rs templates in JavaScript:
```javascript
function renderAST(ast) {
    if (ast.type === 'operation') {
        switch (ast.name) {
            case 'scalar_divide':
                return `<div class="fraction">
                    <div class="numerator">${renderAST(ast.args[0])}</div>
                    <div class="fraction-bar"></div>
                    <div class="denominator">${renderAST(ast.args[1])}</div>
                </div>`;
            // ... 73 operations
        }
    }
}
```

**Pros:**
- Simple to implement
- No WASM complexity
- Instant rendering

**Cons:**
- **Duplicate logic** - render.rs and JS out of sync
- **Maintenance burden** - update templates in two places
- No Rust type safety for templates

### Recommended Technology Stack

**Phase 2-3 (Current):** Option A (Server-rendered HTML)
- Simplest implementation
- Validates architecture
- Gets MVP working quickly
- Can optimize later

**Phase 4+ (Production):** Option B (WASM + HTML rendering)
- Zero latency for interactions
- Works offline
- Single source of truth (Rust templates)
- Reasonable complexity

**Future/Optional:** Option C (WASM + SVG)
- Only if HTML rendering has limitations
- Most work but most control
- Consider if clickability issues arise

**Not Recommended:** Option D
- Duplicates logic across languages
- High maintenance burden
- No technical advantages over WASM

---

## Architecture Decision: Why Structural Wins

### The Fundamental Trade-off

**Traditional WYSIWYG:**
- **Gain:** Familiar cursor-based editing
- **Lose:** Ability to extend notation system

**Kleis Structural:**
- **Gain:** User-extensible operations, auto-generated UI, guaranteed validity
- **Lose:** Free cursor movement (replaced with placeholder navigation)

### Why the Trade is Worth It

**What users actually want from math input:**
1. ✅ Quick entry of common structures (both approaches)
2. ✅ Visual feedback (both approaches)
3. ✅ No syntax errors (Kleis advantage)
4. ✅ LaTeX export (both approaches)
5. ✅ **Define new notation** (Kleis only)
6. ✅ **Auto-generating UI from definitions** (Kleis only)
7. ✅ **Shareable notation packages** (Kleis only)

The cursor trade-off is **minor** compared to the extensibility gain.

**Evidence:** Scratch (visual programming) proved users prefer structured insertion over free typing when it guarantees validity and enables extension.

### The Escape Hatch: Hybrid Mode

If cursor-free editing proves limiting, Kleis can offer both:

```
┌─────────────────────────────────────┐
│ Mode: [ Structural ] [ Text LaTeX ] │  ← Toggle
├─────────────────────────────────────┤
│                                     │
│  Structural: Click + fill           │
│  Text: Type LaTeX directly          │
│                                     │
└─────────────────────────────────────┘
```

Current `index.html` already has this toggle. Users can:
- Build complex structures in Structural mode
- Switch to Text mode for quick tweaks
- Export/import as LaTeX

Best of both worlds.

---

## Technology Choices Summary

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **AST** | Rust (Expression enum) | ✅ Already implemented, type-safe |
| **Parser** | Rust (parser.rs) | ✅ Already implemented, 223 tests |
| **Templates** | Rust (render.rs GlyphContext) | ✅ Already implemented, 73 operations |
| **Server** | Rust (Axum) | ✅ Already implemented, serves API + HTML |
| **Phase 2-3 Renderer** | Server-rendered HTML | ✅ Simplest path to MVP |
| **Phase 4+ Renderer** | WASM + HTML generation | ⭐ Zero latency, offline-capable |
| **UI Framework** | Vanilla JS + Web Components | Minimal dependencies, native performance |
| **Display Renderer** | MathJax 3 | Industry standard for LaTeX display |
| **CSS Layout** | Flexbox + CSS Grid | Modern, responsive, no framework needed |

**Why avoid heavy frameworks (React/Vue/Angular):**
- Minimal surface area for bugs
- Fast load times
- Direct control over rendering
- WASM integration is simpler
- Easier for contributors (no framework lock-in)

**Why WASM for Phase 4+:**
- ✅ Reuses proven Rust rendering code
- ✅ No server dependency for editing
- ✅ Works offline
- ✅ 60fps rendering for complex expressions
- ✅ Single source of truth for templates

---

## Conclusion: The Viable Path

**Kleis structural editing is feasible specifically because:**

1. **Infrastructure exists** - AST, parser, renderer, templates (all done)
2. **Avoids the hard problem** - No 2D cursor tracking (use placeholder IDs instead)
3. **Enables the vision** - User-defined operations → auto-generated UI
4. **Technology ready** - WASM + web-sys makes client-side Rust rendering practical
5. **Already implemented** - Commit `be4d961` has working structural editor

**The traditional WYSIWYG path (MathLive/MathQuill) is a dead end** because it trades extensibility for cursor familiarity. The cursor problem is solvable (those libraries prove it), but the extensibility is fundamentally impossible with their architecture.

**Kleis wins by changing the problem** - from "track cursor in 2D space" to "navigate discrete placeholders in AST tree". Simpler problem, better properties (extensibility + validity), achievable with existing infrastructure.

---

## Layout Engine and Renderer Interface

### The Two-Layer Architecture

To achieve beautiful, interactive math rendering, we separate concerns into two distinct layers:

```
┌──────────────────────────────────────────┐
│  Layer 1: Layout Engine                 │
│  (Ported from KaTeX)                    │
│                                          │
│  Input:  AST with Placeholders          │
│  Output: Positioned Layout Boxes        │
│                                          │
│  Responsibilities:                       │
│  - Calculate dimensions                 │
│  - Compute positions                    │
│  - Apply spacing rules                  │
│  - Handle extensible symbols            │
│  - Knows nothing about rendering        │
└──────────────────────────────────────────┘
              ↓
       LayoutTree with coordinates
              ↓
┌──────────────────────────────────────────┐
│  Layer 2: SVG Renderer                  │
│  (Kleis-specific)                       │
│                                          │
│  Input:  Positioned Layout Boxes        │
│  Output: Interactive SVG DOM            │
│                                          │
│  Responsibilities:                       │
│  - Generate SVG elements                │
│  - Attach click handlers                │
│  - Style placeholders                   │
│  - Apply Computer Modern fonts          │
│  - Knows nothing about layout rules     │
└──────────────────────────────────────────┘
```

**Key Principle:** The layout engine calculates *where* things go, the renderer decides *how* they appear.

### Core Data Structures

#### 1. LayoutBox (Output of Layout Engine)

```rust
/// A positioned box in 2D space with dimensions
/// This is the fundamental unit of layout
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Width of the box in em units
    pub width: f64,
    
    /// Height above baseline in em units
    pub height: f64,
    
    /// Depth below baseline in em units
    pub depth: f64,
    
    /// Distance from top of box to baseline
    pub baseline: f64,
    
    /// Child elements with positions relative to this box's origin
    pub children: Vec<PositionedElement>,
}

impl LayoutBox {
    /// Total vertical size
    pub fn total_height(&self) -> f64 {
        self.height + self.depth
    }
    
    /// Bounding box for this layout
    pub fn bbox(&self) -> BoundingBox {
        BoundingBox {
            x: 0.0,
            y: -self.height,
            width: self.width,
            height: self.total_height(),
        }
    }
}
```

#### 2. PositionedElement (Children of LayoutBox)

```rust
/// An element positioned within its parent box
#[derive(Debug, Clone)]
pub struct PositionedElement {
    /// X offset from parent's origin (left edge)
    pub x: f64,
    
    /// Y offset from parent's baseline (positive = down)
    pub y: f64,
    
    /// The content to render at this position
    pub content: ElementContent,
}
```

#### 3. ElementContent (What to Render)

```rust
/// The actual content to be rendered
#[derive(Debug, Clone)]
pub enum ElementContent {
    /// Plain text character or symbol
    Text {
        content: String,
        font_size: f64,
        font_family: FontFamily,
        italic: bool,
    },
    
    /// Interactive placeholder (CRITICAL for editing)
    Placeholder {
        id: usize,
        hint: String,
        width: f64,
        height: f64,
    },
    
    /// Horizontal line (fraction bars, etc.)
    HorizontalLine {
        width: f64,
        thickness: f64,
    },
    
    /// Vertical line (matrix delimiters, etc.)
    VerticalLine {
        height: f64,
        thickness: f64,
    },
    
    /// Extensible symbol (integrals, summations, brackets)
    ExtensibleSymbol {
        base_char: char,
        target_height: f64,
        pieces: Vec<GlyphPiece>,
    },
    
    /// Nested group of elements
    Group {
        children: Vec<PositionedElement>,
        transform: Option<Transform>,
    },
    
    /// SVG path for custom shapes (radical signs, etc.)
    Path {
        data: String,
        fill: Color,
        stroke: Option<Stroke>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum FontFamily {
    Main,        // Latin Modern Math Roman
    Math,        // Latin Modern Math (special symbols)
    Script,      // Script/calligraphic
    Fraktur,     // Fraktur/gothic
    SansSerif,   // Sans-serif
    Monospace,   // Typewriter
}
```

#### 4. Supporting Types

```rust
/// Transform for rotations, scaling
#[derive(Debug, Clone)]
pub struct Transform {
    pub matrix: [f64; 6],  // SVG transform matrix
}

/// Color representation
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

/// Stroke style for paths
#[derive(Debug, Clone)]
pub struct Stroke {
    pub color: Color,
    pub width: f64,
    pub dash_array: Option<Vec<f64>>,
}

/// Piece of an extensible character
#[derive(Debug, Clone)]
pub struct GlyphPiece {
    pub glyph_id: u16,
    pub y_offset: f64,
}

/// Bounding box for collision detection
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
```

### The Interface Contract

#### Layout Engine Responsibilities

The layout engine (KaTeX port) MUST:

1. **Accept AST with Placeholders:**
   ```rust
   pub fn layout_expression(expr: &Expression, style: MathStyle) -> LayoutBox
   ```

2. **Treat Placeholders as Opaque Boxes:**
   ```rust
   Expression::Placeholder { id, hint } => {
       // Don't try to render - just allocate space
       LayoutBox {
           width: PLACEHOLDER_DEFAULT_WIDTH,   // e.g., 2.0 em
           height: PLACEHOLDER_DEFAULT_HEIGHT, // e.g., 1.0 em
           depth: 0.0,
           baseline: PLACEHOLDER_DEFAULT_HEIGHT * 0.8,
           children: vec![PositionedElement {
               x: 0.0,
               y: 0.0,
               content: ElementContent::Placeholder {
                   id,
                   hint: hint.clone(),
                   width: PLACEHOLDER_DEFAULT_WIDTH,
                   height: PLACEHOLDER_DEFAULT_HEIGHT,
               }
           }],
       }
   }
   ```

3. **Use Font Metrics:**
   - All dimensions in em units (relative to font size)
   - Baseline-relative positioning
   - Apply italic correction, kerning

4. **Follow TeX Layout Rules:**
   - Spacing around operators
   - Superscript/subscript positioning
   - Fraction bar placement (on math axis)
   - Extensible symbol assembly

5. **Output Pure Layout Data:**
   - No SVG generation
   - No DOM manipulation
   - Just numbers and positions

#### SVG Renderer Responsibilities

The SVG renderer MUST:

1. **Accept LayoutBox:**
   ```rust
   pub fn render_layout_to_svg(layout: &LayoutBox, font_size_px: f64) -> SvgElement
   ```

2. **Convert em units to pixels:**
   ```rust
   fn em_to_px(em: f64, font_size: f64) -> f64 {
       em * font_size
   }
   ```

3. **Distinguish Placeholder Content:**
   ```rust
   match &element.content {
       ElementContent::Text { content, font_size, .. } => {
           create_svg_text(content, x, y, font_size)
       }
       
       ElementContent::Placeholder { id, hint, width, height } => {
           // CRITICAL: Make it clickable!
           create_svg_placeholder_rect(x, y, width, height, *id, hint)
       }
       
       ElementContent::HorizontalLine { width, thickness } => {
           create_svg_line(x, y, x + width, y, thickness)
       }
       
       // ... other cases
   }
   ```

4. **Generate Interactive SVG:**
   ```rust
   fn create_svg_placeholder_rect(
       x: f64,
       y: f64,
       width: f64,
       height: f64,
       id: usize,
       hint: &str
   ) -> web_sys::Element {
       let rect = document.create_element_ns(SVG_NS, "rect").unwrap();
       
       // Position and size
       rect.set_attribute("x", &x.to_string()).unwrap();
       rect.set_attribute("y", &y.to_string()).unwrap();
       rect.set_attribute("width", &width.to_string()).unwrap();
       rect.set_attribute("height", &height.to_string()).unwrap();
       
       // Visual style
       rect.set_attribute("fill", "#f0f4ff").unwrap();
       rect.set_attribute("stroke", "#667eea").unwrap();
       rect.set_attribute("stroke-width", "2").unwrap();
       rect.set_attribute("stroke-dasharray", "5,5").unwrap();
       rect.set_attribute("rx", "4").unwrap();  // Rounded corners
       
       // Interactivity
       rect.set_attribute("class", "placeholder").unwrap();
       rect.set_attribute("data-id", &id.to_string()).unwrap();
       rect.set_attribute("data-hint", hint).unwrap();
       rect.set_attribute("cursor", "pointer").unwrap();
       
       // Click handler (attach in JavaScript layer)
       // onclick will trigger: selectPlaceholder(id)
       
       rect
   }
   ```

5. **Apply Fonts:**
   ```rust
   fn create_svg_text(
       content: &str,
       x: f64,
       y: f64,
       font_size: f64,
       font_family: FontFamily
   ) -> web_sys::Element {
       let text = document.create_element_ns(SVG_NS, "text").unwrap();
       
       text.set_attribute("x", &x.to_string()).unwrap();
       text.set_attribute("y", &y.to_string()).unwrap();
       text.set_attribute("font-size", &format!("{}px", font_size)).unwrap();
       
       let font_name = match font_family {
           FontFamily::Main | FontFamily::Math => "Latin Modern Math",
           FontFamily::Script => "Latin Modern Math Script",
           // ... etc
       };
       text.set_attribute("font-family", font_name).unwrap();
       
       text.set_text_content(Some(content));
       
       text
   }
   ```

### Example: Fraction with Placeholder

#### Input AST
```rust
Operation {
    name: "scalar_divide",
    args: vec![
        Placeholder { id: 0, hint: "numerator" },
        Object("2")
    ]
}
```

#### After Layout Engine
```rust
LayoutBox {
    width: 3.5,      // em
    height: 1.8,     // em (above baseline)
    depth: 1.5,      // em (below baseline)
    baseline: 1.8,   // em from top
    children: vec![
        // Numerator placeholder
        PositionedElement {
            x: 0.75,  // Centered
            y: -1.3,  // Above baseline
            content: ElementContent::Placeholder {
                id: 0,
                hint: "numerator".to_string(),
                width: 2.0,
                height: 1.0,
            }
        },
        // Fraction bar
        PositionedElement {
            x: 0.0,
            y: 0.0,   // On baseline (math axis)
            content: ElementContent::HorizontalLine {
                width: 3.5,
                thickness: 0.04,
            }
        },
        // Denominator "2"
        PositionedElement {
            x: 1.5,   // Centered
            y: 0.8,   // Below baseline
            content: ElementContent::Text {
                content: "2".to_string(),
                font_size: 0.7,  // Reduced size
                font_family: FontFamily::Main,
                italic: false,
            }
        },
    ]
}
```

#### After SVG Renderer (at 20px font size)
```xml
<svg viewBox="0 0 70 66" width="70" height="66">
  <!-- Numerator: Clickable placeholder -->
  <rect x="15" y="10" width="40" height="20"
        fill="#f0f4ff" stroke="#667eea" 
        stroke-width="2" stroke-dasharray="5,5" rx="4"
        class="placeholder" data-id="0" data-hint="numerator"
        onclick="selectPlaceholder(0)"/>
  
  <!-- Fraction bar -->
  <line x1="0" y1="36" x2="70" y2="36" 
        stroke="black" stroke-width="0.8"/>
  
  <!-- Denominator: Text -->
  <text x="30" y="52" 
        font-size="14px" 
        font-family="Latin Modern Math"
        text-anchor="middle">2</text>
</svg>
```

### Interface Functions

```rust
// src/math_layout/mod.rs

/// Main entry point for layout engine
pub fn layout_expression(
    expr: &Expression,
    context: &LayoutContext
) -> LayoutBox {
    match expr {
        Expression::Const(s) => layout_constant(s, context),
        Expression::Object(s) => layout_symbol(s, context),
        Expression::Placeholder { id, hint } => layout_placeholder(*id, hint, context),
        Expression::Operation { name, args } => layout_operation(name, args, context),
    }
}

/// Layout context (style, font size, etc.)
pub struct LayoutContext {
    pub style: MathStyle,
    pub font_size: f64,
    pub cramped: bool,
    pub font_metrics: &'static FontMetrics,
}

#[derive(Copy, Clone, Debug)]
pub enum MathStyle {
    Display,      // \displaystyle
    Text,         // \textstyle
    Script,       // \scriptstyle
    ScriptScript, // \scriptscriptstyle
}

// src/svg_renderer/mod.rs

/// Main entry point for SVG generation
#[wasm_bindgen]
pub fn render_to_svg(
    layout: &LayoutBox,
    container_id: &str,
    font_size_px: f64
) -> Result<(), JsValue> {
    let document = web_sys::window()
        .unwrap()
        .document()
        .unwrap();
    
    let container = document
        .get_element_by_id(container_id)
        .ok_or("Container not found")?;
    
    let svg = create_svg_container(&document, layout, font_size_px)?;
    let elements = render_layout_box(layout, &document, font_size_px)?;
    
    svg.append_child(&elements)?;
    container.append_child(&svg)?;
    
    Ok(())
}
```

### Testing Strategy

**Unit test the interface:**

```rust
#[test]
fn test_layout_to_svg_roundtrip() {
    // Create simple AST
    let ast = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Placeholder { id: 0, hint: "num".to_string() },
            Expression::Const("2".to_string()),
        ]
    };
    
    // Layout
    let layout = layout_expression(&ast, &default_context());
    
    // Check layout has placeholder
    assert!(has_placeholder_content(&layout, 0));
    
    // Render (in headless test environment)
    let svg_string = render_layout_to_svg_string(&layout, 20.0);
    
    // Check SVG has clickable rect with correct ID
    assert!(svg_string.contains(r#"data-id="0""#));
    assert!(svg_string.contains("placeholder"));
}

fn has_placeholder_content(layout: &LayoutBox, id: usize) -> bool {
    for child in &layout.children {
        if let ElementContent::Placeholder { id: pid, .. } = &child.content {
            if *pid == id {
                return true;
            }
        }
    }
    false
}
```

### Benefits of This Interface

1. **Clear Separation of Concerns**
   - Layout engine = pure math (positions, dimensions)
   - Renderer = presentation (colors, interactions, DOM)

2. **Testable in Isolation**
   - Test layout without rendering
   - Test rendering without layout calculations

3. **Swappable Implementations**
   - Could replace KaTeX with different layout engine
   - Could render to Canvas instead of SVG
   - Could render to HTML instead

4. **Type-Safe Boundary**
   - Rust types enforce correct usage
   - Can't accidentally skip positioning step
   - Can't confuse layout data with DOM elements

5. **Performance Optimization Points**
   - Cache layout results
   - Only re-render changed subtrees
   - Measure separately: "layout took 2ms, render took 1ms"

6. **🎯 Platform Independence** ⭐ CRITICAL ADVANTAGE
   - Layout engine is **pure Rust** - no web dependencies
   - Can target **web, desktop, mobile** with same codebase
   - Only the renderer changes per platform

### Cross-Platform Architecture

The interface enables **write once, render anywhere**:

```
┌────────────────────────────────────────────┐
│  Kleis Core (Pure Rust)                   │
│  - AST (ast.rs)                            │
│  - Parser (parser.rs)                      │
│  - Layout Engine (math_layout.rs) ← NEW   │
│  - Templates (templates.rs)                │
│  - Font Metrics (font_metrics.rs) ← NEW   │
│                                            │
│  NO platform dependencies                  │
│  NO GUI dependencies                       │
│  NO web dependencies                       │
└────────────────────────────────────────────┘
              ↓
    Single LayoutBox output
              ↓
   ┌──────────┴───────────┐
   ↓                      ↓
┌─────────────────┐  ┌─────────────────┐
│  Web Renderer   │  │ Desktop Renderer│
│  (web-sys)      │  │ (egui/iced)     │
│                 │  │                 │
│  SVG DOM        │  │ Native Widgets  │
│  Click handlers │  │ Mouse handlers  │
│  @font-face     │  │ System fonts    │
└─────────────────┘  └─────────────────┘
```

#### Platform Implementations

**Web (WASM + SVG):**
```rust
// src/renderers/web/mod.rs
use wasm_bindgen::prelude::*;
use web_sys;

#[wasm_bindgen]
pub fn render_to_svg(
    layout: &LayoutBox,
    container_id: &str,
    font_size_px: f64
) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let svg = create_svg_from_layout(layout, &document, font_size_px)?;
    // ... SVG-specific rendering
    Ok(())
}
```

**Desktop (egui):**
```rust
// src/renderers/desktop/mod.rs
use egui::{Ui, Painter, Rect, Color32, Pos2};

pub fn render_to_egui(
    layout: &LayoutBox,
    ui: &mut Ui,
    font_size: f32
) {
    let painter = ui.painter();
    render_layout_box_egui(layout, painter, Pos2::ZERO, font_size);
}

fn render_layout_box_egui(
    layout: &LayoutBox,
    painter: &Painter,
    origin: Pos2,
    font_size: f32
) {
    for child in &layout.children {
        let pos = Pos2::new(
            origin.x + child.x as f32 * font_size,
            origin.y + child.y as f32 * font_size
        );
        
        match &child.content {
            ElementContent::Text { content, .. } => {
                painter.text(
                    pos,
                    egui::Align2::LEFT_TOP,
                    content,
                    egui::FontId::proportional(font_size),
                    Color32::BLACK
                );
            }
            
            ElementContent::Placeholder { id, width, height, .. } => {
                let rect = Rect::from_min_size(
                    pos,
                    egui::vec2(width * font_size, height * font_size)
                );
                
                // Draw placeholder rect
                painter.rect_stroke(
                    rect,
                    4.0,  // Rounding
                    egui::Stroke::new(2.0, Color32::from_rgb(102, 126, 234))
                );
                painter.rect_filled(
                    rect,
                    4.0,
                    Color32::from_rgba_premultiplied(240, 244, 255, 100)
                );
                
                // Check for click
                if ui.interact(rect, egui::Id::new(id), egui::Sense::click()).clicked() {
                    // Handle placeholder click
                    select_placeholder(*id);
                }
            }
            
            ElementContent::HorizontalLine { width, thickness } => {
                painter.line_segment(
                    [pos, Pos2::new(pos.x + width * font_size, pos.y)],
                    egui::Stroke::new(thickness * font_size, Color32::BLACK)
                );
            }
            
            ElementContent::Group { children, .. } => {
                // Recursive rendering
                for nested_child in children {
                    render_layout_box_egui(nested_child, painter, pos, font_size);
                }
            }
            
            _ => { /* Handle other types */ }
        }
    }
}
```

**Mobile (could use iced or native rendering):**
```rust
// src/renderers/mobile/mod.rs
// Similar pattern - consume LayoutBox, render to platform widgets
```

#### Concrete Example: Same Expression, Different Platforms

**Input (shared):**
```rust
let ast = Operation {
    name: "scalar_divide",
    args: vec![
        Placeholder { id: 0, hint: "numerator" },
        Const("2")
    ]
};

// Layout (shared, platform-independent)
let layout = layout_expression(&ast, &context);
```

**Web Output:**
```xml
<svg>
  <rect class="placeholder" data-id="0" onclick="..."/>
  <line x1="0" y1="36" x2="70" y2="36"/>
  <text>2</text>
</svg>
```

**Desktop Output (egui):**
```rust
// Native GUI widgets
ui.painter().rect_stroke(...)  // Placeholder
ui.painter().line_segment(...) // Fraction bar
ui.painter().text(...)         // Denominator
```

**Result:** Same beautiful layout, different rendering technology.

### Why This Matters

**Scenario: You Build the Desktop App First**

1. **Year 1:** Desktop WYSIWYG editor using egui
   - Fast iteration on local machine
   - Full filesystem access for .kleis files
   - Native performance
   - Uses **Layout Engine**

2. **Year 2:** Add web version
   - Compile to WASM
   - Swap in SVG renderer
   - **Same layout code, same beautiful math**
   - Zero duplication of layout logic

3. **Year 3:** Add mobile app
   - Native iOS/Android rendering
   - **Same layout engine again**
   - Consistent typography across platforms

**Or do it in reverse order - doesn't matter!** The layout engine is platform-agnostic.

### Implementation Targets

| Platform | Renderer | Input | Display | Status |
|----------|----------|-------|---------|--------|
| **Web** | SVG via web-sys | Mouse/Touch | Browser | Phase 2-3 |
| **Desktop** | egui/iced | Mouse/Keyboard | Native window | Future |
| **Mobile** | Native/Flutter | Touch | iOS/Android | Future |
| **Terminal** | Unicode art | Keyboard | Terminal 😎 | Maybe |

### The Layout Engine is Universal

**Key insight:** Math layout is **math**, not a platform feature.

The question "where should the fraction bar go?" has the **same answer** whether you're rendering:
- In a web browser
- In a desktop window
- On a mobile screen
- In a PDF
- As ASCII art

**Therefore:** Write the layout engine **once** (in pure Rust), render **everywhere**.

### Benefits for Kleis Vision

This enables the full ecosystem:

1. **Browser editor** - Quick access, no install
2. **Desktop IDE** - Professional tool for package authors
3. **Mobile viewer** - Read .kleis documents on phone
4. **Jupyter integration** - Render in notebooks
5. **VS Code extension** - Live preview in editor
6. **LaTeX package** - Export to traditional workflow

**All using the same layout engine, same fonts, same spacing rules.**

### Rust Crate Structure

```
kleis/
├── kleis-core/           # Platform-independent
│   ├── ast.rs
│   ├── parser.rs
│   ├── math_layout.rs    ← The valuable code
│   ├── font_metrics.rs
│   └── templates.rs
│
├── kleis-web/            # Web-specific
│   └── svg_renderer.rs
│
├── kleis-desktop/        # Desktop-specific
│   └── egui_renderer.rs
│
└── kleis-mobile/         # Mobile-specific
    └── native_renderer.rs
```

**Publish `kleis-core` separately** - others can build their own renderers!

### Prior Art

This pattern is proven:

- **rusttype/ab_glyph** - Font rendering (platform-independent metrics, platform-specific rasterization)
- **kurbo** - 2D curves (pure math, render anywhere)
- **resvg** - SVG processing (parse once, render to many backends)
- **lyon** - Tessellation (geometry calculations, render to OpenGL/Vulkan/WebGL)

**Kleis follows the same pattern:** Math layout is universal, rendering is platform-specific.

### Editing Operations: Undo, Delete, and Structure Manipulation

**Critical Insight:** Structural editing makes undo/delete **simpler** than text editing, because state is discrete (AST nodes) rather than continuous (character positions).

#### The State Model

```rust
pub struct EditorState {
    /// Current AST being edited
    pub ast: Expression,
    
    /// Currently active placeholder (None if no selection)
    pub active_placeholder_id: Option<usize>,
    
    /// Undo stack (previous AST states)
    pub history: Vec<HistoryEntry>,
    
    /// Redo stack (future states after undo)
    pub future: Vec<HistoryEntry>,
    
    /// Placeholder ID counter (monotonically increasing)
    pub next_placeholder_id: usize,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub ast: Expression,
    pub active_placeholder: Option<usize>,
    pub description: String,  // "Filled placeholder 3", "Deleted fraction", etc.
}
```

#### Core Editing Operations

**1. Fill Placeholder (Most Common)**

```rust
impl EditorState {
    /// Fill the active placeholder with parsed content
    pub fn fill_placeholder(&mut self, content: &str) -> Result<(), EditError> {
        let placeholder_id = self.active_placeholder_id
            .ok_or(EditError::NoPlaceholderSelected)?;
        
        // Save state for undo
        self.save_history("Fill placeholder");
        
        // Parse the input
        let parsed = parse_latex(content)?;
        
        // Replace placeholder in AST
        replace_node(&mut self.ast, placeholder_id, parsed)?;
        
        // Navigate to next placeholder
        self.navigate_to_next_placeholder();
        
        Ok(())
    }
}

/// Recursively find and replace a placeholder by ID
fn replace_node(
    expr: &mut Expression,
    target_id: usize,
    replacement: Expression
) -> Result<(), EditError> {
    match expr {
        Expression::Placeholder { id, .. } if *id == target_id => {
            *expr = replacement;
            Ok(())
        }
        Expression::Operation { args, .. } => {
            for arg in args {
                if replace_node(arg, target_id, replacement.clone()).is_ok() {
                    return Ok(());
                }
            }
            Err(EditError::PlaceholderNotFound(target_id))
        }
        _ => Err(EditError::PlaceholderNotFound(target_id))
    }
}
```

**2. Delete Operation (Structural)**

```rust
impl EditorState {
    /// Delete the currently selected element
    /// If placeholder: clear it (make empty again)
    /// If filled node: revert to placeholder
    pub fn delete_selection(&mut self) -> Result<(), EditError> {
        let id = self.active_placeholder_id
            .ok_or(EditError::NoSelection)?;
        
        self.save_history("Delete");
        
        // Find the node and its parent
        let context = find_node_context(&self.ast, id)?;
        
        match context {
            NodeContext::Placeholder { id, hint } => {
                // Already a placeholder - can't delete further
                Err(EditError::CannotDeletePlaceholder)
            }
            
            NodeContext::FilledNode { parent_id, child_index, original_hint } => {
                // Revert filled content back to placeholder
                let new_placeholder = Expression::Placeholder {
                    id: self.next_placeholder_id,
                    hint: original_hint,
                };
                self.next_placeholder_id += 1;
                
                replace_node(&mut self.ast, id, new_placeholder)?;
                self.active_placeholder_id = Some(self.next_placeholder_id - 1);
                
                Ok(())
            }
            
            NodeContext::Operation { parent_id, child_index } => {
                // Delete entire operation (e.g., delete fraction, revert to placeholder)
                let placeholder = Expression::Placeholder {
                    id: self.next_placeholder_id,
                    hint: "expression".to_string(),
                };
                self.next_placeholder_id += 1;
                
                // Replace operation with placeholder in parent
                replace_child_in_parent(&mut self.ast, parent_id, child_index, placeholder)?;
                self.active_placeholder_id = Some(self.next_placeholder_id - 1);
                
                Ok(())
            }
        }
    }
    
    /// Delete the entire expression (start over)
    pub fn delete_all(&mut self) {
        self.save_history("Delete all");
        
        self.ast = Expression::Placeholder {
            id: 0,
            hint: "expression".to_string(),
        };
        self.active_placeholder_id = Some(0);
        self.next_placeholder_id = 1;
    }
}

enum NodeContext {
    Placeholder { id: usize, hint: String },
    FilledNode { parent_id: Option<usize>, child_index: usize, original_hint: String },
    Operation { parent_id: Option<usize>, child_index: usize },
}
```

**3. Undo/Redo (Trivial!)**

```rust
impl EditorState {
    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), EditError> {
        let previous = self.history.pop()
            .ok_or(EditError::NothingToUndo)?;
        
        // Save current state to future (for redo)
        self.future.push(HistoryEntry {
            ast: self.ast.clone(),
            active_placeholder: self.active_placeholder_id,
            description: "redo point".to_string(),
        });
        
        // Restore previous state
        self.ast = previous.ast;
        self.active_placeholder_id = previous.active_placeholder;
        
        Ok(())
    }
    
    /// Redo the last undone operation
    pub fn redo(&mut self) -> Result<(), EditError> {
        let next = self.future.pop()
            .ok_or(EditError::NothingToRedo)?;
        
        // Save current to history
        self.save_history("redo");
        
        // Restore future state
        self.ast = next.ast;
        self.active_placeholder_id = next.active_placeholder;
        
        Ok(())
    }
    
    /// Save current state to history before making changes
    fn save_history(&mut self, description: &str) {
        // Clear redo stack when new edit is made
        self.future.clear();
        
        self.history.push(HistoryEntry {
            ast: self.ast.clone(),
            active_placeholder: self.active_placeholder_id,
            description: description.to_string(),
        });
        
        // Limit history size
        const MAX_HISTORY: usize = 100;
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
    }
}
```

**4. Insert Template (Structure Creation)**

```rust
impl EditorState {
    /// Insert a template at the active placeholder
    pub fn insert_template(&mut self, template_name: &str) -> Result<(), EditError> {
        let placeholder_id = self.active_placeholder_id
            .ok_or(EditError::NoPlaceholderSelected)?;
        
        self.save_history(&format!("Insert {}", template_name));
        
        // Create operation with fresh placeholders
        let operation = create_template_operation(template_name, &mut self.next_placeholder_id)?;
        
        // Replace current placeholder with operation
        replace_node(&mut self.ast, placeholder_id, operation)?;
        
        // Navigate to first placeholder in new structure
        self.navigate_to_next_placeholder();
        
        Ok(())
    }
}

fn create_template_operation(
    name: &str,
    next_id: &mut usize
) -> Result<Expression, EditError> {
    match name {
        "fraction" => {
            let num_id = *next_id;
            *next_id += 1;
            let den_id = *next_id;
            *next_id += 1;
            
            Ok(Expression::Operation {
                name: "scalar_divide".to_string(),
                args: vec![
                    Expression::Placeholder { id: num_id, hint: "numerator".to_string() },
                    Expression::Placeholder { id: den_id, hint: "denominator".to_string() },
                ],
            })
        }
        
        "sqrt" => {
            let arg_id = *next_id;
            *next_id += 1;
            
            Ok(Expression::Operation {
                name: "sqrt".to_string(),
                args: vec![
                    Expression::Placeholder { id: arg_id, hint: "radicand".to_string() },
                ],
            })
        }
        
        "power" => {
            let base_id = *next_id;
            *next_id += 1;
            let exp_id = *next_id;
            *next_id += 1;
            
            Ok(Expression::Operation {
                name: "sup".to_string(),
                args: vec![
                    Expression::Placeholder { id: base_id, hint: "base".to_string() },
                    Expression::Placeholder { id: exp_id, hint: "exponent".to_string() },
                ],
            })
        }
        
        // ... 70+ more templates
        
        _ => Err(EditError::UnknownTemplate(name.to_string()))
    }
}
```

**5. Navigation**

```rust
impl EditorState {
    /// Move to next placeholder (Tab key)
    pub fn navigate_to_next_placeholder(&mut self) {
        let placeholders = find_all_placeholders(&self.ast);
        
        if placeholders.is_empty() {
            self.active_placeholder_id = None;
            return;
        }
        
        // Find current position
        if let Some(current_id) = self.active_placeholder_id {
            if let Some(current_idx) = placeholders.iter().position(|p| p.id == current_id) {
                // Move to next (wrap around)
                let next_idx = (current_idx + 1) % placeholders.len();
                self.active_placeholder_id = Some(placeholders[next_idx].id);
                return;
            }
        }
        
        // Default: first placeholder
        self.active_placeholder_id = Some(placeholders[0].id);
    }
    
    /// Move to previous placeholder (Shift+Tab)
    pub fn navigate_to_prev_placeholder(&mut self) {
        let placeholders = find_all_placeholders(&self.ast);
        
        if placeholders.is_empty() {
            self.active_placeholder_id = None;
            return;
        }
        
        if let Some(current_id) = self.active_placeholder_id {
            if let Some(current_idx) = placeholders.iter().position(|p| p.id == current_id) {
                // Move to previous (wrap around)
                let prev_idx = if current_idx == 0 {
                    placeholders.len() - 1
                } else {
                    current_idx - 1
                };
                self.active_placeholder_id = Some(placeholders[prev_idx].id);
                return;
            }
        }
        
        self.active_placeholder_id = Some(placeholders[0].id);
    }
    
    /// Select a specific placeholder by clicking
    pub fn select_placeholder(&mut self, id: usize) {
        // Verify placeholder exists
        if find_placeholder(&self.ast, id).is_some() {
            self.active_placeholder_id = Some(id);
        }
    }
}

fn find_all_placeholders(expr: &Expression) -> Vec<PlaceholderInfo> {
    let mut result = Vec::new();
    collect_placeholders(expr, &mut result);
    result
}

fn collect_placeholders(expr: &Expression, result: &mut Vec<PlaceholderInfo>) {
    match expr {
        Expression::Placeholder { id, hint } => {
            result.push(PlaceholderInfo { id: *id, hint: hint.clone() });
        }
        Expression::Operation { args, .. } => {
            for arg in args {
                collect_placeholders(arg, result);
            }
        }
        _ => {}
    }
}

struct PlaceholderInfo {
    id: usize,
    hint: String,
}
```

#### Why This is Simpler Than Text Editing

| Operation | Text Editor (Hard) | Structural Editor (Easy) |
|-----------|-------------------|--------------------------|
| **Undo** | Track character insertions/deletions at positions | Clone AST (20 lines of code) |
| **Delete** | Handle cursor position, selection ranges, partial deletes | Delete AST node, replace with placeholder |
| **Navigate** | Calculate pixel positions for cursor | Tree traversal (depth-first search) |
| **Insert** | Parse partial text, maintain validity | Insert template with fresh placeholders |
| **Clipboard** | Copy text, may break structure | Copy AST subtree (always valid) |

#### Example: User Journey with Undo

```
State 1: Empty
AST: Placeholder(0, "expression")
History: []

User clicks "Fraction" button
State 2: Fraction inserted
AST: Operation("scalar_divide", [Placeholder(1), Placeholder(2)])
History: [State 1]

User types "x" in numerator
State 3: Numerator filled
AST: Operation("scalar_divide", [Object("x"), Placeholder(2)])
History: [State 1, State 2]

User types "2" in denominator
State 4: Complete fraction
AST: Operation("scalar_divide", [Object("x"), Object("2")])
History: [State 1, State 2, State 3]

User presses Ctrl+Z (undo)
→ Back to State 3
AST: Operation("scalar_divide", [Object("x"), Placeholder(2)])

User presses Ctrl+Z again
→ Back to State 2
AST: Operation("scalar_divide", [Placeholder(1), Placeholder(2)])

User presses Ctrl+Shift+Z (redo)
→ Forward to State 3
AST: Operation("scalar_divide", [Object("x"), Placeholder(2)])
```

#### Memory Efficiency

**Undo is cheap because:**
- ASTs are small (typical equation: < 1KB)
- 100 history entries ≈ 100KB
- Can use structural sharing (Rc<Expression>) if needed

**Compare to text editor:**
- Must track character positions
- Must handle partial states
- Must reparse on undo
- More complex, more memory

#### Keyboard Shortcuts and Event Handling

**All standard keyboard keys work!** The structural editor is fully keyboard-navigable:

```rust
pub enum EditorCommand {
    // Navigation
    NextPlaceholder,      // Tab
    PrevPlaceholder,      // Shift+Tab
    
    // Editing
    FillPlaceholder(String),  // Type alphanumeric keys
    InsertTemplate(String),   // Click button or hotkey (e.g., / for fraction)
    Delete,                   // Delete or Backspace key
    DeleteAll,                // Ctrl+A, then Delete
    
    // History
    Undo,                 // Ctrl+Z (Cmd+Z on Mac)
    Redo,                 // Ctrl+Shift+Z or Ctrl+Y
    
    // Clipboard
    Copy,                 // Ctrl+C (copy AST subtree)
    Cut,                  // Ctrl+X (copy + delete)
    Paste,                // Ctrl+V (insert AST subtree)
    
    // Selection
    SelectPlaceholder(usize),  // Click with mouse
    SelectAll,                 // Ctrl+A
    
    // Structure manipulation
    Wrap(String),         // Wrap selection in template
    Unwrap,              // Extract content from operation
    
    // Quick insert shortcuts
    InsertFraction,       // / key (when placeholder active)
    InsertPower,          // ^ key
    InsertSubscript,      // _ key
    InsertSqrt,           // Ctrl+R or similar
}

impl EditorState {
    pub fn execute_command(&mut self, cmd: EditorCommand) -> Result<(), EditError> {
        match cmd {
            EditorCommand::NextPlaceholder => {
                self.navigate_to_next_placeholder();
                Ok(())
            }
            EditorCommand::PrevPlaceholder => {
                self.navigate_to_prev_placeholder();
                Ok(())
            }
            EditorCommand::FillPlaceholder(text) => {
                self.fill_placeholder(&text)
            }
            EditorCommand::Delete => {
                self.delete_selection()
            }
            EditorCommand::Undo => {
                self.undo()
            }
            EditorCommand::Redo => {
                self.redo()
            }
            // ... etc
        }
    }
}
```

#### Platform-Specific Keyboard Handling

**Web Implementation:**

```javascript
// In JavaScript/TypeScript (connects to WASM)
const editor = document.getElementById('structural-editor');

editor.addEventListener('keydown', (event) => {
    // Prevent default browser behavior
    if (event.ctrlKey || event.metaKey || event.key === 'Tab') {
        event.preventDefault();
    }
    
    // Map keyboard events to editor commands
    const command = mapKeyToCommand(event);
    if (command) {
        // Call WASM function
        kleis_wasm.execute_command(editorState, command);
        
        // Re-render
        rerenderEditor();
    }
});

function mapKeyToCommand(event) {
    const { key, ctrlKey, metaKey, shiftKey } = event;
    const isMac = navigator.platform.includes('Mac');
    const cmdKey = isMac ? metaKey : ctrlKey;
    
    // Delete keys
    if (key === 'Delete' || key === 'Backspace') {
        return { type: 'Delete' };
    }
    
    // Navigation
    if (key === 'Tab' && !shiftKey) {
        return { type: 'NextPlaceholder' };
    }
    if (key === 'Tab' && shiftKey) {
        return { type: 'PrevPlaceholder' };
    }
    
    // Undo/Redo
    if (cmdKey && key === 'z' && !shiftKey) {
        return { type: 'Undo' };
    }
    if (cmdKey && key === 'z' && shiftKey) {
        return { type: 'Redo' };
    }
    if (cmdKey && key === 'y') {
        return { type: 'Redo' };
    }
    
    // Clipboard
    if (cmdKey && key === 'c') {
        return { type: 'Copy' };
    }
    if (cmdKey && key === 'x') {
        return { type: 'Cut' };
    }
    if (cmdKey && key === 'v') {
        return { type: 'Paste' };
    }
    
    // Quick insert shortcuts
    if (key === '/' && !cmdKey) {
        return { type: 'InsertTemplate', template: 'fraction' };
    }
    if (key === '^' && !cmdKey) {
        return { type: 'InsertTemplate', template: 'power' };
    }
    if (key === '_' && !cmdKey) {
        return { type: 'InsertTemplate', template: 'subscript' };
    }
    
    // Regular character input (for filling placeholders)
    if (key.length === 1 && !cmdKey && !metaKey) {
        return { type: 'TypeCharacter', char: key };
    }
    
    return null;
}

// Input buffer for multi-character input
let inputBuffer = '';
let inputTimeout = null;

editor.addEventListener('keypress', (event) => {
    if (event.key.length === 1) {
        inputBuffer += event.key;
        
        // Clear previous timeout
        if (inputTimeout) clearTimeout(inputTimeout);
        
        // Wait for user to finish typing
        inputTimeout = setTimeout(() => {
            if (inputBuffer.trim()) {
                kleis_wasm.fill_placeholder(editorState, inputBuffer);
                rerenderEditor();
                inputBuffer = '';
            }
        }, 500);  // 500ms pause = done typing
    }
});
```

**Desktop Implementation (egui):**

```rust
// In Rust using egui
impl EditorApp {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        // egui handles keyboard events automatically
        ctx.input(|input| {
            // Delete/Backspace
            if input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace) {
                let _ = self.editor_state.delete_selection();
            }
            
            // Tab navigation
            if input.key_pressed(egui::Key::Tab) {
                if input.modifiers.shift {
                    self.editor_state.navigate_to_prev_placeholder();
                } else {
                    self.editor_state.navigate_to_next_placeholder();
                }
            }
            
            // Undo/Redo
            if input.modifiers.command && input.key_pressed(egui::Key::Z) {
                if input.modifiers.shift {
                    let _ = self.editor_state.redo();
                } else {
                    let _ = self.editor_state.undo();
                }
            }
            
            // Copy/Paste
            if input.modifiers.command && input.key_pressed(egui::Key::C) {
                if let Ok(text) = self.editor_state.copy() {
                    ctx.output_mut(|o| o.copied_text = text);
                }
            }
            if input.modifiers.command && input.key_pressed(egui::Key::V) {
                if let Some(text) = ctx.input(|i| i.raw.clipboard_text.clone()) {
                    let _ = self.editor_state.paste(&text);
                }
            }
            
            // Quick insert shortcuts
            if input.key_pressed(egui::Key::Slash) && !input.modifiers.command {
                let _ = self.editor_state.insert_template("fraction");
            }
            
            // Character input (for filling placeholders)
            for event in &input.raw.events {
                if let egui::Event::Text(text) = event {
                    // User typed a character
                    self.input_buffer.push_str(text);
                }
            }
        });
        
        // Process buffered input after a pause
        if self.input_buffer_updated() {
            if !self.input_buffer.is_empty() {
                let _ = self.editor_state.fill_placeholder(&self.input_buffer);
                self.input_buffer.clear();
            }
        }
    }
}
```

**Desktop Implementation (iced):**

```rust
// Using iced framework
#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(keyboard::KeyCode, keyboard::Modifiers),
    CharacterTyped(char),
    PlaceholderClicked(usize),
}

impl Application for EditorApp {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::KeyPressed(key, modifiers) => {
                use keyboard::KeyCode;
                
                match key {
                    KeyCode::Delete | KeyCode::Backspace => {
                        let _ = self.editor_state.delete_selection();
                    }
                    
                    KeyCode::Tab => {
                        if modifiers.shift() {
                            self.editor_state.navigate_to_prev_placeholder();
                        } else {
                            self.editor_state.navigate_to_next_placeholder();
                        }
                    }
                    
                    KeyCode::Z if modifiers.command() => {
                        if modifiers.shift() {
                            let _ = self.editor_state.redo();
                        } else {
                            let _ = self.editor_state.undo();
                        }
                    }
                    
                    _ => {}
                }
            }
            
            Message::CharacterTyped(c) => {
                self.input_buffer.push(c);
                // Process after pause...
            }
            
            Message::PlaceholderClicked(id) => {
                self.editor_state.select_placeholder(id);
            }
        }
        
        Command::none()
    }
    
    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| {
            Some(Message::KeyPressed(key, modifiers))
        })
    }
}
```

#### Key Behavior Details

**Tab Key: Focus Management**

Tab moves between placeholders, **NOT** between AST nodes. It only focuses on empty slots to be filled:

```rust
impl EditorState {
    pub fn navigate_to_next_placeholder(&mut self) {
        // Find all unfilled placeholders in the AST
        let placeholders = find_all_placeholders(&self.ast);
        
        // Move to next one (skip over filled nodes)
        let next_idx = (current_idx + 1) % placeholders.len();
        self.active_placeholder_id = Some(placeholders[next_idx].id);
    }
}
```

**Example Tab behavior:**

```
Expression: □/□ + □²
            ↑ ↑   ↑
            1 2   3

Tab sequence: 1 → 2 → 3 → 1 (wraps around)
Filled content is skipped!
```

If user fills placeholder 1 with "x":
```
Expression: x/□ + □²
              ↑   ↑
              2   3

Tab sequence: 2 → 3 → 2 (placeholder 1 removed from navigation)
```

**Delete Key: Structure Removal**

Delete has **two levels** depending on what's selected:

```rust
impl EditorState {
    pub fn delete_selection(&mut self) -> Result<(), EditError> {
        let selected_id = self.active_placeholder_id
            .ok_or(EditError::NoSelection)?;
        
        self.save_history("Delete");
        
        // Find what's at this ID
        let node_info = find_node_info(&self.ast, selected_id)?;
        
        match node_info.node_type {
            NodeType::EmptyPlaceholder => {
                // Can't delete empty placeholder - delete parent operation instead
                if let Some(parent_id) = node_info.parent_id {
                    delete_operation_at(parent_id)?;
                } else {
                    // Top-level placeholder - can't delete
                    return Err(EditError::CannotDelete);
                }
            }
            
            NodeType::FilledPlaceholder => {
                // First delete: clear content → empty placeholder
                clear_placeholder_content(selected_id)?;
                // Placeholder ID stays in navigation
            }
            
            NodeType::Operation => {
                // Delete entire operation structure → placeholder
                let new_placeholder = Expression::Placeholder {
                    id: self.next_placeholder_id,
                    hint: "expression".to_string(),
                };
                self.next_placeholder_id += 1;
                
                replace_node(&mut self.ast, selected_id, new_placeholder)?;
                self.active_placeholder_id = Some(self.next_placeholder_id - 1);
            }
        }
        
        Ok(())
    }
}

enum NodeType {
    EmptyPlaceholder,    // □
    FilledPlaceholder,   // x, 2, etc.
    Operation,           // fraction, sqrt, etc.
}
```

**Two-Stage Delete Behavior:**

| State | First Delete | Second Delete |
|-------|-------------|---------------|
| **Simple value** |
| `x` | → `□` (clear) | (stops - can't delete placeholder) |
| **In structure** |
| `x/2` (numerator focused) | → `□/2` (clear numerator) | → `□` (delete whole fraction) |
| `□/2` (empty numerator focused) | → `□` (delete whole fraction) | - |
| **Nested** |
| `(x+y)/2` (x focused) | → `(□+y)/2` (clear x) | → `□/2` (delete sum) → `□` (delete fraction) |

**Example: Deleting a fraction step by step**

```
State 1: x/2
User tabs to numerator (x), presses Delete

State 2: □/2  (content cleared, structure preserved)
User presses Delete again

State 3: □    (fraction structure deleted)
User presses Delete again

State 3: □    (can't delete top-level placeholder - no effect)
```

**Focus Management After Delete:**

```rust
impl EditorState {
    fn delete_and_refocus(&mut self) -> Result<(), EditError> {
        let deleted_id = self.active_placeholder_id.unwrap();
        
        // Perform deletion
        self.delete_selection()?;
        
        // If we deleted an operation, new placeholder was created
        // Focus automatically moves to it
        
        // If we cleared a placeholder, focus stays on it
        
        // If we can't delete (top-level), focus unchanged
        
        Ok(())
    }
}
```

**Smart Delete for Nested Structures:**

```
User has: sqrt(x/2)
          
Tab to numerator 'x', press Delete:
→ sqrt(□/2)  (just cleared x)

Press Delete again:
→ sqrt(□)    (deleted fraction)

Press Delete again:
→ □          (deleted sqrt)
```

**What We Track: Position in AST Tree (Simple!)**

Unlike traditional text editors, we **don't track 2D cursor positions**. We only track which AST node is active:

```rust
pub struct EditorState {
    /// The complete expression tree
    pub ast: Expression,
    
    /// ONLY tracking: Which placeholder ID is focused
    /// This is just an integer! Not coordinates!
    pub active_placeholder_id: Option<usize>,
    
    /// Optional: Which node is selected (for structural operations)
    pub selected_node_id: Option<usize>,
    
    /// Counter for generating new placeholder IDs
    pub next_placeholder_id: usize,
    
    // That's it! No cursor X/Y, no character positions!
}
```

**Comparison: Traditional vs. Structural Cursor**

| Traditional Text Editor | Structural Editor (Kleis) |
|------------------------|---------------------------|
| Track: `{ x: 45.2, y: 120.8 }` | Track: `active_placeholder_id: Some(3)` |
| Calculate: "Is cursor inside fraction?" | Know: "Placeholder 3 is numerator" |
| Update: Recalculate on every resize | Update: Just change ID (integer) |
| Navigate: Complex 2D geometry | Navigate: Tree traversal (DFS) |
| State: Pixel coordinates (floats) | State: Node ID (integer) |

**Example: "Cursor" in AST Tree**

```rust
// The expression
let ast = Operation {
    name: "scalar_divide",
    args: vec![
        Placeholder { id: 1, hint: "numerator" },    // ← "cursor" here
        Object("2")
    ]
};

// The "cursor position" is just:
active_placeholder_id = Some(1);

// That's it! No X/Y coordinates needed!
```

**Navigation is Tree Walking:**

```rust
// Tab to next placeholder = Find next ID in tree
fn navigate_to_next_placeholder(&mut self) {
    let placeholders = find_all_placeholders(&self.ast);
    //  placeholders = [1, 3, 5, 7]  (just IDs!)
    
    let current_idx = placeholders.iter()
        .position(|p| p.id == self.active_placeholder_id.unwrap());
    
    let next_idx = (current_idx + 1) % placeholders.len();
    self.active_placeholder_id = Some(placeholders[next_idx].id);
    
    // No geometry calculations!
    // No "where should cursor appear on screen?"
    // Just: "next ID in list"
}
```

**Why This is Dramatically Simpler:**

**Traditional editor must answer:**
- "Where is the cursor?" → Calculate pixel position
- "User pressed Right Arrow" → Calculate new pixel position based on font metrics
- "User resized window" → Recalculate all positions
- "Is cursor inside a fraction?" → Hit-test geometry

**Structural editor only needs:**
- "Which placeholder is active?" → `active_placeholder_id = 3`
- "User pressed Tab" → `active_placeholder_id = next_id(3)` 
- "User resized window" → Nothing! IDs don't change!
- "Is placeholder in a fraction?" → Walk up parent links (tree operation)

**The Layout Engine Handles Position:**

```rust
// We track: ID (just an integer)
active_placeholder_id = Some(3);

// The layout engine calculates: position (for rendering only)
let layout = layout_expression(&ast);
// layout.find_placeholder(3) → { x: 45.2, y: 120.8 }

// But we NEVER store or track those coordinates!
// They're recalculated on every render.
// The "cursor position" is always just ID 3.
```

**Complete State Model:**

```rust
pub struct EditorState {
    // The data (AST tree)
    pub ast: Expression,
    
    // The "cursor" (just an ID, not a position)
    pub active_placeholder_id: Option<usize>,
    
    // History (for undo) - just clone the AST
    pub history: Vec<Expression>,
    
    // ID counter (monotonically increasing)
    pub next_placeholder_id: usize,
}

// Total state: ~100 bytes for typical expression
// Compare to: Text editor state can be megabytes
```

**Finding Nodes is Tree Search:**

```rust
// "Where is placeholder 3?" is just a tree search
fn find_placeholder(expr: &Expression, target_id: usize) -> Option<&Expression> {
    match expr {
        Expression::Placeholder { id, .. } if *id == target_id => Some(expr),
        Expression::Operation { args, .. } => {
            for arg in args {
                if let Some(found) = find_placeholder(arg, target_id) {
                    return Some(found);
                }
            }
            None
        }
        _ => None
    }
}

// O(n) where n = number of nodes (typically < 100)
// Compare to: Text editor cursor positioning = complex font metrics + layout
```

**Two Modes of Interaction:**

**Mode 1: Placeholder Focus (Tab navigation)**
```
Expression: x/□ + y²
              ↑
         (focused placeholder)

State: active_placeholder_id = Some(2)

Delete → Can't delete (empty)
         OR delete parent structure (fraction)
```

**Mode 2: Node Selection (Click to select)**
```
Expression: x/2 + y²
            ↑↑↑
        (whole fraction selected via click)

State: selected_node_id = Some(5)  (the fraction operation's ID)

Delete → □ + y²  (entire fraction removed)
```

**Implementation:**

```rust
pub struct EditorState {
    /// ID of focused placeholder (for Tab navigation and typing)
    pub active_placeholder_id: Option<usize>,
    
    /// ID of selected AST node (for structural operations like Delete)
    /// If None, operations affect active_placeholder's parent
    pub selected_node_id: Option<usize>,
}

impl EditorState {
    /// Tab focuses placeholders (only navigates between empty slots)
    pub fn navigate_to_next_placeholder(&mut self) {
        let placeholders = find_all_placeholders(&self.ast);
        let next = find_next_in_list(placeholders, self.active_placeholder_id);
        
        self.active_placeholder_id = Some(next.id);
        self.selected_node_id = None;  // Clear structural selection
        
        // Still just tracking IDs! No positions!
    }
    
    /// Click can select any node by ID
    pub fn select_node_by_id(&mut self, node_id: usize) {
        self.selected_node_id = Some(node_id);
        
        // If it's a placeholder, also focus it
        if is_placeholder(&self.ast, node_id) {
            self.active_placeholder_id = Some(node_id);
        }
    }
    
    /// Delete operates on selected node OR active placeholder's parent
    pub fn delete_selection(&mut self) -> Result<(), EditError> {
        let target_id = self.selected_node_id
            .or(self.active_placeholder_id)
            .ok_or(EditError::NoSelection)?;
        
        // Find node by ID (tree search - O(n))
        let node = find_node(&self.ast, target_id)?;
        
        // Perform deletion based on node type
        // ... (delete logic as above)
        
        // Still just manipulating IDs and tree structure!
        // No coordinate math!
    }
}
```

**Rendering Uses IDs to Apply Visual State:**

```rust
// When rendering, we use the IDs to apply CSS classes
fn render_placeholder_to_svg(placeholder: &Expression, is_active: bool) -> SvgElement {
    match placeholder {
        Expression::Placeholder { id, .. } => {
            let rect = create_svg_rect(...);
            
            // Apply visual state based on ID matching
            if is_active && *id == active_placeholder_id {
                rect.set_attribute("class", "placeholder active");
                rect.set_attribute("stroke", "#667eea");
            } else {
                rect.set_attribute("class", "placeholder");
                rect.set_attribute("stroke", "#d0d0d0");
            }
            
            rect
        }
    }
}
```

**The Key Insight:**

```
Traditional Editor:
State = "cursor at pixel (X, Y)"
Challenge: Calculate what's under the cursor, handle all edge cases

Structural Editor:
State = "placeholder ID 3 is active"
Challenge: None - it's just an integer!

The layout engine converts IDs → positions for rendering,
but the editor state never needs to know or care about positions!
```

**This is why structural editing is tractable** - we track discrete node identities (IDs), not continuous geometric positions (X/Y coordinates). Tree operations on IDs are simple and well-understood algorithms.

impl EditorState {
    /// Tab focuses placeholders
    pub fn navigate_to_next_placeholder(&mut self) {
        self.active_placeholder_id = Some(next_placeholder_id);
        self.selected_node_id = None;  // Clear selection
    }
    
    /// Click can select any node (for deletion)
    pub fn select_node(&mut self, node_id: usize) {
        self.selected_node_id = Some(node_id);
        // Also update placeholder focus if it's a placeholder
        if is_placeholder(node_id) {
            self.active_placeholder_id = Some(node_id);
        }
    }
    
    /// Delete operates on selection OR active placeholder's parent
    pub fn delete_selection(&mut self) -> Result<(), EditError> {
        let target_id = self.selected_node_id
            .or(self.active_placeholder_id)
            .ok_or(EditError::NoSelection)?;
        
        // ... perform deletion as above
    }
}
```

**Visual Indication:**

```css
/* Focused placeholder (Tab) - dashed border */
.placeholder.focused {
    border: 2px dashed #667eea;
    background: #f0f4ff;
}

/* Selected node (Click) - solid border */
.node.selected {
    outline: 2px solid #ff6b6b;
    outline-offset: 2px;
}
```

**User Experience Flow:**

```
1. User tabs to placeholder → dashed blue border (ready to type)
2. User types content → placeholder fills
3. User tabs to next placeholder
4. User clicks on filled content → red outline (ready to delete)
5. User presses Delete → structure removed
6. New placeholder created → auto-focused
```

**Smart shortcuts (inspired by LaTeX editors):**

```rust
// When typing in an active placeholder:
'/' → Detect if previous content, wrap in fraction
    "x" + "/" → converts to x/□
    
'^' → Detect if previous content, add superscript
    "x" + "^" → converts to x^□
    
'_' → Detect if previous content, add subscript
    "a" + "_" → converts to a_□

// Or make these explicit commands:
'/' in empty placeholder → Insert fraction template
'^' in empty placeholder → Insert superscript template
```

#### Full Keyboard Navigation

The editor is **100% keyboard accessible:**

```
1. Start with empty placeholder (active by default)
2. Type "x" → fills placeholder with "x"
3. Press "/" → wraps in fraction: x/□
4. Type "2" → completes: x/2
5. Press Tab → (no more placeholders, could exit or wrap)
6. Press Ctrl+Z → undo to: x/□
7. Press Delete → clear denominator: x/□
8. Press Tab → navigate to numerator
9. Press Delete → clear numerator: □/□
10. Press Ctrl+Z twice → back to: x/2
11. Press Ctrl+A, Delete → delete all: □
```

**No mouse required!** Power users can work entirely from keyboard.

#### Focus Management

**Critical:** The editor container must be focusable to receive keyboard events.

**Web:**
```html
<div id="structural-editor" 
     tabindex="0"             <!-- Makes it focusable -->
     role="application"       <!-- Accessibility -->
     aria-label="Math equation editor">
  <!-- SVG content here -->
</div>

<style>
#structural-editor:focus {
    outline: 2px solid #667eea;
    outline-offset: 2px;
}
</style>
```

**Desktop:**
```rust
// egui: focus is automatic when interacting
// iced: set focus in update() if needed
```

### Summary: Keyboard is First-Class

✅ **Delete/Backspace** - Clear or delete selected node  
✅ **Tab/Shift+Tab** - Navigate placeholders  
✅ **Ctrl+Z/Ctrl+Y** - Undo/Redo  
✅ **Ctrl+C/X/V** - Copy/Cut/Paste  
✅ **Alphanumeric keys** - Fill placeholders  
✅ **/, ^, _** - Quick insert templates  
✅ **Escape** - Deselect/Cancel  
✅ **Enter** - Accept and move to next  

**The structural editor is MORE keyboard-friendly than traditional text editors** because:
- Tab navigation is deterministic (not ambiguous)
- No need for arrow keys to position cursor
- Shortcuts map directly to structure operations
- Can't make syntax errors by typing wrong keys
```

#### Advanced: Copy/Paste (AST Subtrees)

```rust
impl EditorState {
    /// Copy the selected subtree to clipboard
    pub fn copy(&self) -> Result<String, EditError> {
        let id = self.active_placeholder_id
            .ok_or(EditError::NoSelection)?;
        
        let subtree = extract_subtree(&self.ast, id)?;
        
        // Serialize to JSON for clipboard
        Ok(serde_json::to_string(&subtree)?)
    }
    
    /// Paste AST subtree at active placeholder
    pub fn paste(&mut self, clipboard: &str) -> Result<(), EditError> {
        let id = self.active_placeholder_id
            .ok_or(EditError::NoPlaceholderSelected)?;
        
        self.save_history("Paste");
        
        // Deserialize from clipboard
        let subtree: Expression = serde_json::from_str(clipboard)?;
        
        // Renumber placeholders to avoid conflicts
        let renumbered = renumber_placeholders(subtree, &mut self.next_placeholder_id);
        
        // Replace at current location
        replace_node(&mut self.ast, id, renumbered)?;
        
        Ok(())
    }
}

/// Renumber all placeholder IDs in a subtree to avoid conflicts
fn renumber_placeholders(mut expr: Expression, next_id: &mut usize) -> Expression {
    match &mut expr {
        Expression::Placeholder { id, .. } => {
            *id = *next_id;
            *next_id += 1;
        }
        Expression::Operation { args, .. } => {
            for arg in args {
                *arg = renumber_placeholders(arg.clone(), next_id);
            }
        }
        _ => {}
    }
    expr
}
```

### Conclusion: Editing is Tractable

**All core editing operations are straightforward:**
- ✅ **Undo/Redo:** Just clone AST (trivial)
- ✅ **Delete:** Replace node with placeholder
- ✅ **Insert:** Create template with fresh placeholder IDs
- ✅ **Navigate:** DFS traversal of AST
- ✅ **Copy/Paste:** Serialize/deserialize AST subtrees

**Why it works:**
- State is discrete (AST nodes), not continuous (text positions)
- Operations are atomic and reversible
- No partial/invalid states to handle
- Tree structure makes navigation deterministic

**This is actually SIMPLER than text editing** because you don't need to:
- Track cursor pixel positions
- Handle selection ranges
- Parse partial expressions
- Maintain text-buffer invariants
- Deal with invalid intermediate states

The structural approach turns a complex problem (arbitrary text manipulation) into a simple problem (tree manipulation).

### Layout Engine Implementation Decision

**Decision (2024-11-22):** Use **Typst** as the math layout engine instead of porting KaTeX.

**Rationale:**

| Option | Status | Pros | Cons |
|--------|--------|------|------|
| **Port KaTeX (JavaScript → Rust)** | ❌ Rejected | Proven algorithms, well-documented | Months of work, 50K+ lines to port, maintenance burden |
| **Write from scratch** | ❌ Rejected | Full control | Years to reach quality, reinventing wheel |
| **Use Typst** | ✅ **SELECTED** | Already exists in Rust, professional quality, 100K+ lines, actively maintained, pure Rust (WASM-ready), handles all complex math | Larger dependency (~5MB), need adapter layer |

**Why Typst:**

1. **Exists and Works** - Typst is a production-ready typesetting system used by thousands
2. **Pure Rust** - No JavaScript/WASM bridge needed, compiles to native and WASM
3. **Professional Quality** - Handles all edge cases we'd take years to discover
4. **Font Metrics Included** - Ships with Computer Modern compatible metrics
5. **Extensible Symbols** - Brackets, integrals, summations all work
6. **Active Development** - Regular updates, bug fixes, improvements
7. **MIT Licensed** - Compatible with Kleis

**What We Build:**

```
┌──────────────────────────────────────────┐
│  Kleis AST (with Placeholders)          │
│  Expression::Placeholder { id, hint }    │
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  Typst Adapter Layer (Kleis code)       │
│  - Convert Expression → Typst Content    │
│  - Preserve placeholder metadata         │
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  Typst Layout Engine (library)          │
│  - Apply TeX layout rules                │
│  - Font metrics                          │
│  - Spacing, sizing, positioning          │
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  LayoutBox Conversion (Kleis code)       │
│  - Extract positioned elements           │
│  - Mark placeholders for interactivity   │
└──────────────────────────────────────────┘
              ↓
┌──────────────────────────────────────────┐
│  SVG Renderer (Kleis code)               │
│  - Render text with fonts                │
│  - Render placeholders as clickable      │
└──────────────────────────────────────────┘
```

**Key Insight:** We're not building a layout engine, we're building an **interactive structural editor** that uses a layout engine. Typst handles the hard part (layout), we handle the novel part (interactivity + extensibility).

**Kleis-Specific Code:**
- ✅ AST with Placeholders (unique to Kleis)
- ✅ Adapter: Expression → Typst Content (glue code)
- ✅ LayoutBox interface (our abstraction)
- ✅ SVG renderer with clickable placeholders (our code)
- ✅ Editor state management (undo, delete, navigate)
- ✅ Template system integration (our templates)
- ✅ User-extensible operations (our vision)

**What Typst Provides:**
- ✅ Math layout algorithms (complex, battle-tested)
- ✅ Font metrics (Computer Modern compatible)
- ✅ Extensible symbols (integrals, brackets, etc.)
- ✅ Spacing rules (TeX-compatible)
- ✅ Professional typography

**Dependencies:**

```toml
[dependencies]
typst = "0.11"          # Core typesetting engine
typst-svg = "0.11"      # SVG output (optional, might not use directly)
```

**Estimated Scope:**
- Without Typst: 6-12 months to replicate KaTeX quality
- With Typst: 2-4 weeks to build adapter + interactive layer

**Risk Mitigation:**

If Typst proves unsuitable (unlikely), our `LayoutBox` abstraction allows swapping engines:
```rust
// Interface stays the same
pub fn layout_expression(expr: &Expression) -> LayoutBox

// Implementation can change:
// - Current: use Typst
// - Alternative: use ported KaTeX
// - Alternative: use custom engine
```

**Trade-offs Accepted:**

1. **Dependency size** (~5MB) - Acceptable for quality gained
2. **Some Typst-specific quirks** - Can work around if needed
3. **Less low-level control** - Gain: Don't need it for math layout

**Conclusion:** Use battle-tested professional library (Typst) for the hard part (layout), focus our effort on the novel part (interactive structural editing with extensibility).

---

### Next Implementation Steps

1. **Add Typst dependency** → Update `Cargo.toml`
2. **Create Typst adapter** → Convert `Expression` to Typst `Content`
3. **Extract layout info** → Typst layout → `LayoutBox` with placeholder positions
4. **Build SVG renderer** → `LayoutBox` → interactive SVG DOM
5. **Add click handlers** → wire up `selectPlaceholder(id)`
6. **Implement EditorState** → undo, delete, navigation
7. **Test round-trip** → AST → Typst → LayoutBox → SVG → Click → Update AST
8. **Integrate templates** → 73 operations work with Typst
9. **Optimize** → caching, partial updates

---

## Architectural Decision: LaTeX Parsing vs. Template-Based Semantic Inference

**Date:** 2024-11-23  
**Decision:** Treat quantifiers (`\forall`, `\exists`) and similar constructs as **flat symbols** in the parser, not structural operators.

### The Problem

LaTeX math notation is inherently **flat/linear**: `\forall x \colon x \in S` is a sequence of symbols.  
Structural editors need **tree-based AST**: `forall(variable: x, body: in_set(x, S))`.

### Parsing Approaches Considered

**Option A: Structural Parsing**  
Parse `\forall x \colon P(x)` as `Operation("forall", [x, P(x)])`.

*Pros:*
- Semantically rich AST
- Editor knows `x` is bound by `\forall`
- Enables scope-aware operations

*Cons:*
- **Ambiguous scope**: Where does `\forall` end? `\forall x P(x) \land Q(x)` could mean `(\forall x P(x)) \land Q(x)` or `\forall x (P(x) \land Q(x))`
- **Rigid syntax**: Requires specific separators (`:`, `.`)
- **Parsing complexity**: Must handle operator precedence for logical connectives
- **Fragile**: Breaks on non-standard LaTeX like `\forall x` (incomplete statement)

**Option B: Flat Symbol Parsing (CHOSEN)**  
Parse `\forall` as `Object("\\forall")`, let implicit multiplication chain symbols.

*Pros:*
- **Simple and robust**: No ambiguity about scope
- **Flexible**: Users can type symbols in any order during editing
- **Visually correct**: Renders properly regardless of structure
- **Predictable**: Matches LaTeX's flat nature

*Cons:*
- **Semantically weak**: Editor doesn't know `x` is bound by `\forall`
- **Limited structural operations**: Can't auto-rename bound variables

### Decision Rationale

1. **LaTeX is fundamentally flat.** Forcing tree structure on ambiguous syntax creates more problems than it solves.

2. **Structural editing is for TEMPLATES, not arbitrary LaTeX.** The editor palette provides structured templates like `forall(var, body)` that users insert. These have clear boundaries.

3. **Parsing arbitrary LaTeX is a convenience feature**, not the primary workflow. The structural editor works with programmatically constructed ASTs.

4. **We can have both:** 
   - **Editor palette**: Provides `forall(var, body)` template with clear structure
   - **LaTeX import**: Parses `\forall x` as flat symbols (best effort)
   - **Future enhancement**: Pattern-match LaTeX against template outputs to infer structure

### Template-Based Semantic Inference (Future Work)

**Concept:** If LaTeX text matches what a template would generate, infer the semantic structure.

Example:
- Template: `forall` renders as `\forall {var} \colon {body}`
- Input: `\forall x \colon x \in S`
- **Pattern match:** This looks like `forall(x, x \in S)`
- **Infer AST:** Parse as `Operation("forall", [x, in_set(x, S)])`

This allows:
- **Simple parsing** for most cases (flat symbols)
- **Smart inference** when LaTeX matches template patterns
- **Best of both worlds**: Flexibility + semantic richness

Implementation approach:
1. Parse LaTeX as flat symbols (current behavior)
2. Post-process AST to detect template patterns
3. Upgrade matched patterns to structured operations
4. Preserve flat structure for non-matching cases

This is deferred to future work as it requires:
- Pattern matching engine
- Template output analysis
- Heuristics for ambiguous cases

### Current Implementation

- **Parser:** Treats `\forall`, `\exists` as objects (symbols)
- **Renderer:** Has templates for `forall(var, body)` and `exists(var, body)` operations
- **Editor Palette:** Will provide structured templates for insertion
- **LaTeX Import:** Parses as flat symbols, renders correctly

This architecture allows the structural editor to work with clean ASTs while gracefully handling arbitrary LaTeX input.

---

**Status:** Architecture validated, foundation complete, layout engine selected (Typst), ready for implementation  
**Date Updated:** 2024-11-22  
**Layout Engine:** Typst (Rust library)  
**Estimated Timeline:** 2-4 weeks for MVP with Typst vs. 6-12 months porting KaTeX  
**Impact:** Transforms fixed-notation editor into **extensible mathematical language authoring system**  
**Scope:** Not just editing LaTeX - **defining and using new mathematical notations**  
**Unique Value:** Empty slate + palette workflow with user-extensible operations (nothing else like this exists)  
**Risk:** Low (all infrastructure in place, only UI work remains)  
**Effort:** Phase 2-3: ~2-3 weeks, Phase 4-5: ~2-3 months  
**Foundation:** **100% ready** - unified AST, template system, parser/renderer, 223 tests passing  
**Technology Path:** Server HTML (MVP) → WASM + HTML (production) → WASM + SVG (if needed)

