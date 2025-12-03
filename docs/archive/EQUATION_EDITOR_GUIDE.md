# Kleis Equation Editor - User Guide

**Version:** 1.0  
**Date:** 2024-11-23  
**Status:** Production Ready

---

## Overview

The Kleis Equation Editor is a **hybrid text/structural editor** that combines the flexibility of LaTeX with the visual clarity of WYSIWYG editing. It features intelligent parsing, template-based semantic inference, and seamless mode switching.

---

## Two Editing Modes

### 📝 Text Mode

**What it is:** Traditional LaTeX text editing with live preview.

**How to use:**
1. Type LaTeX in the input box: `\frac{1}{2} + \sqrt{x}`
2. Click "Render" or press Enter
3. See MathJax preview below

**Best for:**
- Quick entry of familiar LaTeX
- Copy-pasting equations
- Experienced LaTeX users

**Features:**
- Full LaTeX syntax support
- Symbol palette for quick insertion
- Real-time MathJax rendering

---

### 🔧 Structural Mode

**What it is:** Visual editing with clickable regions and placeholders.

**How to use:**
1. Click template buttons to insert operations
2. Click on placeholders (□) to fill them
3. See live Typst-rendered preview with bounding boxes

**Best for:**
- Building complex equations step-by-step
- Understanding equation structure
- Users learning mathematical notation

**Features:**
- Visual feedback (bounding boxes show clickable regions)
- Placeholder-based editing (fill in the blanks)
- Structured templates (fractions, roots, integrals, etc.)
- Undo/redo support

---

## Seamless Mode Switching

### Text → Structural

**When:** Click "🔧 Structural Mode" button while in text mode

**What happens:**
1. System parses your LaTeX expression
2. Applies **template inference** to recognize semantic structures
3. Converts to internal AST (Abstract Syntax Tree)
4. Renders as Typst SVG with clickable regions
5. You can now edit structurally

**Example:**
```
LaTeX:    \iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y
Parsed:   Recognizes double integral pattern
AST:      double_integral(f(x,y), D, x, y)
Display:  Visual double integral with clickable parts
```

**If parsing fails:**
- Error message shown
- Previous AST preserved (if any)
- You can fix the LaTeX and try again
- **No data loss!**

---

### Structural → Text

**When:** Click "📝 Text Mode" button while in structural mode

**What happens:**
1. System renders your AST to LaTeX
2. Updates text input with generated LaTeX
3. Renders MathJax preview
4. You can now edit as text

**Example:**
```
AST:      double_integral(f(x,y), D, x, y)
Renders:  \iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y
Display:  LaTeX in text box, MathJax preview below
```

**Round-trip guarantee:** AST → LaTeX → AST preserves structure (for supported operations)

---

## Symbol Palette

**Location:** Right side panel

**Tabs:**
- **Greek**: α, β, γ, ... (lowercase and uppercase)
- **Operators**: +, −, ×, ÷, ±, ∓, ...
- **Relations**: =, ≠, <, >, ≤, ≥, ≈, ≡, ∈, ⊂, ...
- **Calculus**: ∫, ∑, ∏, ∂, ∇, lim, ...
- **Templates**: Fractions, roots, matrices, integrals, ...

**How to use:**
- **Text mode:** Click symbol → inserts LaTeX at cursor
- **Structural mode:** Click template → inserts structured operation

---

## Template Inference (Intelligent Parsing)

The editor includes **template-based semantic inference** that recognizes common mathematical patterns and converts them to structured operations.

### Supported Patterns

| LaTeX Pattern | Recognized As | Structure |
|---------------|---------------|-----------|
| `\iint_{D} f \, \mathrm{d}x \, \mathrm{d}y` | Double integral | `double_integral(f, D, x, y)` |
| `\iiint_{V} f \, \mathrm{d}x \, \mathrm{d}y \, \mathrm{d}z` | Triple integral | `triple_integral(f, V, x, y, z)` |
| `P \Rightarrow Q` | Logical implication | `implies(P, Q)` |
| `P \Leftrightarrow Q` | Logical equivalence | `iff(P, Q)` |
| `\forall x \colon P(x)` | Universal quantifier | `forall(x, P(x))` |
| `\exists x \colon P(x)` | Existential quantifier | `exists(x, P(x))` |
| `a \equiv b \pmod{n}` | Modular congruence | `congruent_mod(a, b, n)` |
| `\mathrm{Var}(X)` | Variance | `variance(X)` |
| `\mathrm{Cov}(X, Y)` | Covariance | `covariance(X, Y)` |
| `\mathrm{Tr}(A)` | Trace | `trace(A)` |
| `\mathrm{Re}(z)` | Real part | `re(z)` |
| `\mathrm{Im}(z)` | Imaginary part | `im(z)` |
| `\zeta(s)` | Zeta function | `function_call(zeta, s)` |
| `\Gamma(s)` | Gamma function | `function_call(Gamma, s)` |

**Benefit:** You get semantic structure automatically, enabling:
- Structural editing of parsed LaTeX
- Proper bounding boxes for each component
- Meaningful AST for future operations

**Graceful fallback:** If pattern not recognized, keeps flat structure (still renders correctly)

---

## Parser Capabilities

The LaTeX parser now handles:

### Basic Constructs
- ✅ Fractions: `\frac{a}{b}`
- ✅ Roots: `\sqrt{x}`, `\sqrt[n]{x}`
- ✅ Superscripts/subscripts: `x^{2}`, `a_{n}`
- ✅ Prime notation: `y'`, `y''`, `y'''`
- ✅ Parentheses: `(a + b)`, `\left( \frac{a}{b} \right)`

### Functions
- ✅ Trig: `\sin(x)`, `\cos(x)`, `\tan(x)`
- ✅ Inverse trig: `\arcsin(x)`, `\arccos(x)`, `\arctan(x)`
- ✅ Hyperbolic: `\sinh(x)`, `\cosh(x)`, `\tanh(x)`
- ✅ Reciprocal trig: `\sec(x)`, `\csc(x)`, `\cot(x)`
- ✅ Logarithms: `\ln(x)`, `\log(x)`, `\exp(x)` → renders as `e^x`
- ✅ Custom functions: `f(x)`, `g(x, y)`

### Greek Letters
- ✅ Lowercase: `\alpha`, `\beta`, ..., `\omega`
- ✅ Uppercase: `\Gamma`, `\Delta`, ..., `\Omega`
- ✅ As functions: `\zeta(s)`, `\Gamma(s)` (with parentheses)

### Vectors & Accents
- ✅ Vector arrow: `\vec{v}`
- ✅ Bold vector: `\boldsymbol{v}`, `\mathbf{v}`
- ✅ Hat: `\hat{x}`, `\hat{p}`
- ✅ Bar: `\bar{x}`
- ✅ Tilde: `\tilde{x}`
- ✅ Dot: `\dot{x}` (velocity)
- ✅ Double dot: `\ddot{x}` (acceleration)

### Matrices
- ✅ Bracket matrix: `\begin{bmatrix}...\end{bmatrix}`
- ✅ Parenthesis matrix: `\begin{pmatrix}...\end{pmatrix}`
- ✅ Determinant: `\begin{vmatrix}...\end{vmatrix}`
- ✅ With subscripts: `a_{11}`, `a_{ij}`

### Calculus
- ✅ Integrals: `\int_{a}^{b} f(x) \, \mathrm{d}x`
- ✅ Double integrals: `\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y`
- ✅ Triple integrals: `\iiint_{V} f(x,y,z) \, \mathrm{d}x \, \mathrm{d}y \, \mathrm{d}z`
- ✅ Sums: `\sum_{n=1}^{\infty} a_n`
- ✅ Products: `\prod_{i=1}^{n} x_i`
- ✅ Limits: `\lim_{x \to 0} f(x)` (partial support)
- ✅ Partial derivatives: `\frac{\partial f}{\partial x}`

### Logic & Set Theory
- ✅ Quantifiers: `\forall x \colon P(x)`, `\exists x \colon P(x)`
- ✅ Implications: `P \Rightarrow Q`, `P \Leftrightarrow Q`
- ✅ Set membership: `x \in S`
- ✅ Subset: `A \subseteq B`
- ✅ Union/intersection: `A \cup B`, `A \cap B`

### Quantum Mechanics
- ✅ Ket: `|\psi\rangle`
- ✅ Bra: `\langle\phi|`
- ✅ Inner product: `\langle u | v \rangle`
- ✅ Commutator: `[\hat{x}, \hat{p}]`
- ✅ Anticommutator: `\{\hat{A}, \hat{B}\}`

### Special Notation
- ✅ Modular arithmetic: `a \equiv b \pmod{n}`
- ✅ Binomial: `\binom{n}{k}`
- ✅ Floor/ceiling: `\lfloor x \rfloor`, `\lceil x \rceil`
- ✅ Absolute value: `|x|`
- ✅ Norm: `\|v\|`
- ✅ Factorial: `n!` (partial support)

---

## Known Limitations

### Not Supported (Yet)

1. **Comma-separated sequences**: `1, 2, 3, \ldots, n`
   - Reason: No Sequence node in AST
   - Workaround: Use programmatic construction or raw strings

2. **Limit subscripts with arrows**: `\lim_{x \to 0}`
   - Reason: `\to` in subscripts not fully parsed
   - Workaround: Programmatic construction works

3. **Complex multi-line environments**: `align`, `gather`, etc.
   - Reason: Not yet implemented
   - Workaround: Use single-line equations

4. **Text mode in math**: `\text{if}` may split characters
   - Reason: Implicit multiplication of letters
   - Status: Mostly fixed with quotes in templates

### Graceful Degradation

For unsupported constructs:
- Parser does best-effort (flat symbol approach)
- Still renders visually correct
- May lose semantic structure
- Can still edit in text mode

---

## Technical Architecture

### Components

1. **Frontend (static/index.html)**
   - Text mode: LaTeX input + MathJax preview
   - Structural mode: Typst SVG + clickable overlays
   - Mode switching with bidirectional conversion

2. **Backend (Rust server)**
   - Parser: LaTeX → AST (with template inference)
   - Renderer: AST → LaTeX/Typst/HTML
   - Typst compiler: Typst → SVG with layout info

3. **Template Inference (src/template_inference.rs)**
   - Post-processes flat parsed ASTs
   - Pattern matches against known templates
   - Infers semantic structure
   - Graceful fallback to flat structure

### Data Flow

**Text → Structural:**
```
LaTeX input
  ↓ (parse)
Flat AST (multiplication chains)
  ↓ (template inference)
Structured AST (semantic operations)
  ↓ (render to Typst)
Typst markup
  ↓ (compile)
SVG + layout info
  ↓ (display)
Visual editor with clickable regions
```

**Structural → Text:**
```
AST (current state)
  ↓ (render to LaTeX)
LaTeX string
  ↓ (update input)
Text box populated
  ↓ (render)
MathJax preview
```

---

## API Endpoints

### POST `/api/parse`
**Request:**
```json
{
  "latex": "\\frac{1}{2} + \\sqrt{x}"
}
```

**Response:**
```json
{
  "ast": { ... },
  "success": true
}
```

### POST `/api/render`
**Request:**
```json
{
  "ast": { ... },
  "target": "latex" | "typst" | "html"
}
```

**Response:**
```json
{
  "output": "\\frac{1}{2} + \\sqrt{x}",
  "success": true
}
```

### POST `/api/render_structural`
**Request:**
```json
{
  "ast": { ... }
}
```

**Response:**
```json
{
  "svg": "<svg>...</svg>",
  "layout": {
    "placeholders": [...],
    "arguments": [...]
  },
  "success": true
}
```

---

## Tips & Tricks

### Getting Started

1. **Start in text mode** if you know LaTeX
   - Type your equation
   - Switch to structural to see structure
   - Edit visually

2. **Start in structural mode** if learning
   - Click templates to build equation
   - Fill placeholders step by step
   - Switch to text to see LaTeX

### Best Practices

- **Use templates for complex structures** (integrals, matrices)
- **Use text mode for simple expressions** (x + y)
- **Switch modes freely** - no data loss!
- **Check preview** before finalizing

### Troubleshooting

**Tabs not working?**
- Hard refresh browser (Cmd+Shift+R or Ctrl+Shift+R)
- Clear cache

**Parse error when switching to structural?**
- Check LaTeX syntax
- Previous AST is preserved - you can fix and retry
- Or stay in text mode

**Missing features?**
- Check "Known Limitations" section
- Use programmatic construction for unsupported patterns
- Report issues for future enhancements

---

## Examples

### Example 1: Quadratic Formula

**Text mode input:**
```latex
x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
```

**Structural mode:**
- Click on numerator to edit `-b ± √(b² - 4ac)`
- Click on denominator to edit `2a`
- Visual feedback shows structure

---

### Example 2: Integral

**Text mode input:**
```latex
\int_{0}^{\infty} e^{-x^2} \, \mathrm{d}x = \frac{\sqrt{\pi}}{2}
```

**Structural mode:**
- Integral recognized with bounds and variable
- Click on integrand to edit `e^(-x²)`
- Click on bounds to change limits

---

### Example 3: Matrix

**Text mode input:**
```latex
\begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}
```

**Structural mode:**
- Matrix structure recognized
- Click on each element to edit
- Visual grid shows matrix structure

---

## Advanced Features

### Template Inference

The system automatically recognizes patterns and infers semantic structure:

**Before inference:**
```
Flat: \iint * D * f(x,y) * mathrm(d) * x * mathrm(d) * y
```

**After inference:**
```
Structured: double_integral(integrand: f(x,y), region: D, var1: x, var2: y)
```

**Benefit:** Better editing experience, semantic operations available

### Graceful Fallback

If inference fails:
- Keeps flat structure
- Still renders correctly
- Structural editing still works (just less semantic)
- No errors or data loss

---

## For Developers

### Adding New Templates

1. Define operation in `src/ast.rs`
2. Add templates in `src/render.rs`:
   - `unicode_templates` (Unicode display)
   - `latex_templates` (LaTeX output)
   - `html_templates` (HTML rendering)
   - `typst_templates` (Typst rendering)
3. Add parser support in `src/parser.rs` (if needed)
4. Add inference pattern in `src/template_inference.rs` (if applicable)
5. Test with `cargo test`

### Testing Parser

```bash
# Test specific LaTeX expression
cargo run --bin test_comparison

# Check for errors
cargo run --bin test_comparison 2>&1 | grep "unknown variable"

# Run parser tests
cargo test --lib parser
```

---

## Version History

**v1.0 (2024-11-23)**
- ✅ Bidirectional mode switching
- ✅ Template inference (13 patterns)
- ✅ Zero compilation errors
- ✅ 100+ Typst templates
- ✅ Comprehensive parser support
- ✅ Safe error handling with AST preservation

---

## Future Enhancements

**Planned:**
- Limit subscript parsing (`x \to 0`)
- Sequence node type for comma-separated lists
- More template inference patterns
- Keyboard shortcuts
- Export to multiple formats

**Under Consideration:**
- Collaborative editing
- Equation library/favorites
- Custom user templates
- Mobile support

---

**For more technical details, see:**
- `docs/adr-009-wysiwyg-structural-editor.md` - Architecture
- `docs/TEMPLATE_INFERENCE_IMPLEMENTATION.md` - Inference system
- `docs/LATEX_PARSING_ANALYSIS.md` - Parser analysis

