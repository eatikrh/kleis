# Bracket & Grouping Templates - Added 2024-12-03

## Summary

Added missing bracket/grouping templates to the Kleis structural editor, making it complete for mathematical notation.

## New Templates Added

### 1. Bracket Operations (AST Templates)

Previously, brackets were only available as LaTeX text inserts. Now they're proper AST operations:

| Operation | LaTeX | Unicode | Description |
|-----------|-------|---------|-------------|
| `parens` | `\left( {content} \right)` | `(x)` | Parentheses - auto-scaling |
| `brackets` | `\left[ {content} \right]` | `[x]` | Square brackets - auto-scaling |
| `braces` | `\left\{ {content} \right\}` | `{x}` | Curly braces - auto-scaling |
| `angle_brackets` | `\left\langle {content} \right\rangle` | `⟨x⟩` | Angle brackets - auto-scaling |

These use Typst's `lr()` function for automatic size scaling based on content height.

### 2. Already Existed (Now Exposed in Palette)

| Operation | LaTeX | Unicode | Description |
|-----------|-------|---------|-------------|
| `floor` | `\lfloor {arg} \rfloor` | `⌊x⌋` | Floor function |
| `ceiling` | `\lceil {arg} \rceil` | `⌈x⌉` | Ceiling function |
| `abs` | `\left\lvert {arg} \right\rvert` | `\|x\|` | Absolute value |
| `norm` | `\left\lVert {arg} \right\rVert` | `‖v‖` | Vector norm |

### 3. Fixed Placeholder Labels

**Before:**
- `x^{□}` - Misleading! Looked like "x" was fixed
- `x_{□}` - Misleading! Looked like "x" was fixed  
- `x^{□}_{□}` - Misleading! Looked like "x" was fixed

**After:**
- `□^{□}` - Clear: both base and exponent are editable
- `□_{□}` - Clear: both base and subscript are editable
- `□^{□}_{□}` - Clear: all three parts are editable

**Note:** The underlying templates (`template_power()`, `template_subscript()`) always had both arguments as placeholders. We just fixed the button labels to match reality.

## Implementation Details

### Backend Changes

**File: `src/templates.rs`**
- Added `template_parens()`
- Added `template_brackets()`
- Added `template_braces()`
- Added `template_angle_brackets()`
- Added `template_floor()` (moved from elsewhere)
- Added `template_ceiling()` (moved from elsewhere)
- Registered all in `get_all_templates()`

**File: `src/render.rs`**
- Added rendering templates for all four bracket types in:
  - Unicode templates
  - LaTeX templates (with `\left` / `\right` auto-scaling)
  - HTML templates
  - Typst templates (with `lr()` auto-scaling)

### Frontend Changes

**File: `static/index.html`**
- Updated palette buttons to show `□^□` instead of `x^□`
- Added template mappings:
  - `\left(□\right)` → `parens`
  - `\left[□\right]` → `brackets`
  - `\left\{□\right\}` → `braces`
  - `\left\langle □ \right\rangle` → `angle_brackets`

## Palette Organization

The "Fences & Grouping" section now contains:

```
( ) Paren       [ ] Bracket     { } Brace       ⟨ ⟩ Angle
|x| Abs         ‖v‖ Norm        ⌊x⌋ Floor       ⌈x⌉ Ceiling
```

All create proper AST nodes with deterministic UUID-based positioning.

## Why This Matters

### 1. **Completeness**
Every standard mathematical delimiter is now a first-class structural template.

### 2. **Auto-Scaling**
Brackets automatically resize to fit their content:
```latex
\left( \frac{a}{b} \right)  % Big parens
\left[ \sum_{i=1}^n x_i \right]  % Big brackets
```

### 3. **Structural Editing**
Brackets are now AST nodes, not just text:
- Can be selected and edited as units
- UUID-based positioning (deterministic)
- Proper nesting in the AST tree

### 4. **DLMF Coverage**
The curated DLMF equations use brackets extensively:
- Legendre polynomials: `P_m(x)`, `\int_{-1}^1 [ ... ]`
- Bessel functions: `J_\nu(z)`, nested fractions
- Floor/ceiling: `\lfloor x \rfloor` in number theory

## Examples from DLMF

**Legendre Orthogonality (uses parens and brackets):**
```latex
\int_{-1}^1 P_m(x) P_n(x) \, dx = \frac{2}{2n+1} \delta_{mn}
```

**Gamma Reflection (uses parens):**
```latex
\Gamma(z)\Gamma(1-z) = \frac{\pi}{\sin(\pi z)}
```

**Bessel Series (nested parens):**
```latex
J_\nu(z) = \sum_{k=0}^\infty \frac{(-1)^k}{k!\,\Gamma(k+\nu+1)} \left(\frac{z}{2}\right)^{2k+\nu}
```

## Testing

All bracket templates:
- ✅ Render correctly in LaTeX
- ✅ Render correctly in Unicode
- ✅ Render correctly in HTML
- ✅ Render correctly in Typst with auto-scaling
- ✅ Generate UUIDs for deterministic positioning
- ✅ Support nested structures

## Next Steps

1. **Test with DLMF corpus**: Run the 36 DLMF equations through the editor
2. **Visual regression**: Compare rendering with reference images
3. **Interaction testing**: Verify click-to-edit works for nested brackets
4. **Documentation**: Update user guide with bracket template examples

---

**Status**: ✅ Complete  
**Affects**: Structural editor, palette, rendering engine  
**Backward Compatible**: Yes (legacy `x^{□}` mappings preserved)

