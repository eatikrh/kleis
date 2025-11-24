# Quick Answer: Template Availability

## Your Questions

> Do we have superscripts and subscripts for Tensor representations in the templates?

**YES** ✅ - But only partially exposed in the palette.

> Do we have dot notation derivatives?

**YES** ✅ - In the backend, but NOT in the palette.

> Do we have regular, curly, bracket parenthesis?

**YES** ✅ - All supported in backend, only square brackets in palette.

---

## Detailed Breakdown

### Tensor Representations

| Template | Backend | Palette | LaTeX Example |
|----------|---------|---------|---------------|
| Simple subscript | ✅ | ✅ | `x_{i}` |
| Simple superscript | ✅ | ✅ | `x^{n}` |
| Mixed index | ✅ | ✅ | `T^{i}_{j}` |
| Double upper | ✅ | ❌ | `g^{\mu\nu}` |
| Christoffel | ✅ | ❌ | `\Gamma^{\mu}_{\nu\sigma}` |
| Riemann | ✅ | ❌ | `R^{\rho}_{\sigma\mu\nu}` |

**Status:** Basic tensors work, advanced tensors (Christoffel, Riemann) are supported but not in palette.

---

### Dot Notation Derivatives

| Template | Backend | Palette | LaTeX Example | Meaning |
|----------|---------|---------|---------------|---------|
| Dot accent | ✅ | ❌ | `\dot{x}` | Velocity, 1st derivative |
| Double dot | ✅ | ❌ | `\ddot{x}` | Acceleration, 2nd derivative |

**Status:** Fully supported in backend, completely missing from palette.

---

### Bracket/Parenthesis Types

#### For Matrices

| Type | Backend | Palette | LaTeX Example |
|------|---------|---------|---------------|
| Square brackets `[ ]` | ✅ | ✅ | `\begin{bmatrix}a&b\\c&d\end{bmatrix}` |
| Parentheses `( )` | ✅ | ❌ | `\begin{pmatrix}a&b\\c&d\end{pmatrix}` |
| Vertical bars `\| \|` | ✅ | ❌ | `\begin{vmatrix}a&b\\c&d\end{vmatrix}` |
| Curly braces `{ }` | ✅ | ❌ | `\begin{Bmatrix}a&b\\c&d\end{Bmatrix}` |

#### For Delimiters

| Type | Backend | Palette | LaTeX Example | Use Case |
|------|---------|---------|---------------|----------|
| Absolute value `\| \|` | ✅ | ✅ | `\|x\|` | Magnitude |
| Norm `\|\| \|\|` | ✅ | ✅ | `\|\|v\|\|` | Vector norm |
| Floor `⌊ ⌋` | ✅ | ❌ | `\lfloor x \rfloor` | Round down |
| Ceiling `⌈ ⌉` | ✅ | ❌ | `\lceil x \rceil` | Round up |
| Angle brackets `⟨ ⟩` | ✅ | ✅ | `\langle\psi\|\phi\rangle` | Inner product |
| Square brackets `[ ]` | ✅ | ✅ | `[A, B]` | Commutator |
| Curly braces `{ }` | ✅ | ❌ | `\{A, B\}` | Anticommutator |

**Status:** Most bracket types supported in backend, only a few in palette.

---

## Summary

### ✅ What Works Right Now (In Palette)
- Basic subscript/superscript: `x_{i}`, `x^{n}`
- Mixed tensor index: `T^{i}_{j}`
- Square bracket matrices: `[a b; c d]`
- Absolute value: `|x|`
- Norm: `||v||`
- Angle brackets (bra-ket): `⟨ψ|φ⟩`
- Commutator: `[A, B]`

### ⚠️ What Works But Not In Palette
- Christoffel symbol: `Γ^μ_{νσ}`
- Riemann tensor: `R^ρ_{σμν}`
- Dot accent: `ẋ`
- Double dot: `ẍ`
- Parenthesis matrices: `(a b; c d)`
- Determinant matrices: `|a b; c d|`
- Curly brace matrices: `{a b; c d}`
- Floor/ceiling: `⌊x⌋`, `⌈x⌉`
- Anticommutator: `{A, B}`

### ❌ Known Issues
- Matrix 3×3 template is broken (shows "3x3" text instead of proper template)
- Matrix edit markers don't align correctly in structural mode

---

## What To Do

### Option 1: Use Text Mode
You can type any of these directly in text mode:
```latex
\Gamma^{\mu}_{\nu\sigma}
R^{\rho}_{\sigma\mu\nu}
\dot{x}
\ddot{x}
\begin{pmatrix}a&b\\c&d\end{pmatrix}
\begin{vmatrix}a&b\\c&d\end{vmatrix}
```

### Option 2: Add To Palette (Recommended)
See `PALETTE_TEMPLATES_ANALYSIS.md` for detailed implementation plan.

Quick fix - add these buttons to `static/index.html`:

```html
<!-- Tensor tab -->
<button class="template-btn" onclick="insertTemplate('\\Gamma^{□}_{□ □}')">
    Γ Christoffel
</button>
<button class="template-btn" onclick="insertTemplate('R^{□}_{□ □ □}')">
    R Riemann
</button>

<!-- Accents tab -->
<button class="template-btn" onclick="insertTemplate('\\dot{□}')">
    ẋ Dot
</button>
<button class="template-btn" onclick="insertTemplate('\\ddot{□}')">
    ẍ Double Dot
</button>

<!-- Matrices tab -->
<button class="template-btn" onclick="insertTemplate('\\begin{pmatrix}□&□\\\\□&□\\end{pmatrix}')">
    ( ) Matrix
</button>
<button class="template-btn" onclick="insertTemplate('\\begin{vmatrix}□&□\\\\□&□\\end{vmatrix}')">
    | | Determinant
</button>
```

---

## Files Created

For full details, see:
1. `TEMPLATE_INVENTORY.md` - Complete inventory with examples
2. `PALETTE_TEMPLATES_ANALYSIS.md` - Comprehensive analysis and implementation plan
3. `PALETTE_ANALYSIS_SUMMARY.md` - Executive summary
4. `static/improved_palette.html` - Visual demo of proposed 79-template palette
5. `static/palette_test.html` - Test page for current templates

---

**Bottom Line:** The renderer is phenomenal and supports everything you asked about. The palette just needs to expose these features through clickable buttons. This is primarily a UI task, not a backend engineering challenge.

