# Kleis Template Inventory - Current Status

**Date:** November 24, 2024

This document answers the question: **"What templates do we have for tensors, derivatives, and brackets?"**

---

## ✅ YES - We Have These

### Tensor Representations (Superscripts & Subscripts)

#### ✅ In Backend (Rust `src/templates.rs` + `src/render.rs`)
1. **Mixed Index** `T^{i}_{j}` - `template_tensor_mixed()`
   - Operation: `index_mixed`
   - LaTeX: `{base}^{{idx1}}_{{idx2}}`
   - Example: `T^{\mu}_{\nu}`

2. **Double Upper Index** `T^{ij}` - `template_tensor_upper_pair()`
   - Operation: `index_pair`
   - LaTeX: `{base}^{{idx1}{idx2}}`
   - Example: `g^{\mu\nu}`

3. **Christoffel Symbol** `\Gamma^{\mu}_{\nu\sigma}`
   - Operation: `gamma`
   - LaTeX: `\Gamma^{{idx1}}_{{idx2} {idx3}}`
   - Fully supported in renderer (lines 1547-1550, 1954-1959)

4. **Riemann Tensor** `R^{\rho}_{\sigma\mu\nu}`
   - Operation: `riemann`
   - LaTeX: `R^{{idx1}}_{{idx2} {idx3} {idx4}}`
   - Fully supported in renderer (lines 1551-1554, 1960-1964)

5. **Simple Subscript** `x_{i}`
   - Operation: `sub`
   - Template: `template_subscript()`

6. **Simple Superscript** `x^{n}`
   - Operation: `sup`
   - Template: `template_power()`

#### ❌ NOT in HTML Palette (yet)
- Only `x^{□}_{□}` (mixed index) is in the palette
- Missing: Christoffel, Riemann, double upper/lower indices

---

### Dot Notation Derivatives

#### ✅ In Backend Renderer (`src/render.rs`)
1. **Dot Accent** `\dot{x}` (velocity, first derivative)
   - Operation: `dot_accent`
   - HTML: `{arg}̇` (combining dot above U+0307)
   - Line 1993

2. **Double Dot** `\ddot{x}` (acceleration, second derivative)
   - Operation: `ddot_accent`
   - HTML: `{arg}̈` (combining diaeresis U+0308)
   - Line 1994

#### ❌ NOT in `src/templates.rs`
- No `template_dot_accent()` or `template_ddot_accent()` functions
- These work in parsing/rendering but aren't exposed as insertable templates

#### ❌ NOT in HTML Palette
- Not available as clickable templates

---

### Bracket Types (Parentheses, Braces, Brackets)

#### ✅ In Backend - Matrix Delimiters
1. **Square Brackets** `\begin{bmatrix}...\end{bmatrix}`
   - Template: `template_matrix_2x2()`, `template_matrix_3x3()`
   - In palette: ✅ Yes (2×2 only, 3×3 is broken)

2. **Parentheses** `\begin{pmatrix}...\end{pmatrix}`
   - Supported in parser/renderer
   - In palette: ❌ No

3. **Vertical Bars (Determinant)** `\begin{vmatrix}...\end{vmatrix}`
   - Supported in parser/renderer
   - In palette: ❌ No

4. **Curly Braces** `\begin{Bmatrix}...\end{Bmatrix}`
   - Supported in parser (standard LaTeX)
   - In palette: ❌ No

#### ✅ In Backend - Delimiters
1. **Absolute Value** `|x|`
   - Template: `template_abs()`
   - In palette: ✅ Yes

2. **Norm** `\|v\|`
   - Template: `template_norm()`
   - In palette: ✅ Yes

3. **Floor** `\lfloor x \rfloor`
   - Supported in parser
   - In palette: ❌ No

4. **Ceiling** `\lceil x \rceil`
   - Supported in parser
   - In palette: ❌ No

5. **Angle Brackets (Bra-Ket)** `\langle \phi | \psi \rangle`
   - Templates: `template_bra()`, `template_ket()`, `template_inner()`
   - In palette: ✅ Yes

6. **Commutator** `[A, B]`
   - Template: `template_commutator()`
   - In palette: ✅ Yes

7. **Anticommutator** `\{A, B\}`
   - Supported in parser
   - In palette: ❌ No

---

## 📊 Summary Table

| Feature | Backend Support | Template Function | In Palette | Status |
|---------|----------------|-------------------|------------|--------|
| **Tensors** | | | | |
| Mixed index `T^i_j` | ✅ | ✅ `template_tensor_mixed` | ✅ | **GOOD** |
| Double upper `T^{ij}` | ✅ | ✅ `template_tensor_upper_pair` | ❌ | **MISSING** |
| Christoffel `Γ^μ_{νσ}` | ✅ | ❌ | ❌ | **MISSING** |
| Riemann `R^ρ_{σμν}` | ✅ | ❌ | ❌ | **MISSING** |
| **Derivatives** | | | | |
| Dot `\dot{x}` | ✅ | ❌ | ❌ | **MISSING** |
| Double dot `\ddot{x}` | ✅ | ❌ | ❌ | **MISSING** |
| **Brackets** | | | | |
| Square `[...]` matrix | ✅ | ✅ `template_matrix_2x2` | ✅ (2×2) | **PARTIAL** |
| Parentheses `(...)` matrix | ✅ | ❌ | ❌ | **MISSING** |
| Vertical bars `\|...\|` det | ✅ | ❌ | ❌ | **MISSING** |
| Curly braces `{...}` | ✅ | ❌ | ❌ | **MISSING** |
| Absolute `\|x\|` | ✅ | ✅ `template_abs` | ✅ | **GOOD** |
| Norm `\|\|v\|\|` | ✅ | ✅ `template_norm` | ✅ | **GOOD** |
| Floor `⌊x⌋` | ✅ | ❌ | ❌ | **MISSING** |
| Ceiling `⌈x⌉` | ✅ | ❌ | ❌ | **MISSING** |
| Angle `⟨ψ\|φ⟩` | ✅ | ✅ `template_inner` | ✅ | **GOOD** |
| Commutator `[A,B]` | ✅ | ✅ `template_commutator` | ✅ | **GOOD** |
| Anticommutator `{A,B}` | ✅ | ❌ | ❌ | **MISSING** |

---

## 🔧 What Needs to Be Added

### High Priority (Common Use Cases)

1. **Add to `src/templates.rs`:**
   ```rust
   /// Christoffel symbol: Γ^μ_{νσ}
   pub fn template_christoffel() -> Expression {
       Expression::operation(
           "gamma",
           vec![
               Expression::placeholder(next_id(), "upper"),
               Expression::placeholder(next_id(), "lower1"),
               Expression::placeholder(next_id(), "lower2"),
           ],
       )
   }
   
   /// Riemann tensor: R^ρ_{σμν}
   pub fn template_riemann() -> Expression {
       Expression::operation(
           "riemann",
           vec![
               Expression::placeholder(next_id(), "upper"),
               Expression::placeholder(next_id(), "lower1"),
               Expression::placeholder(next_id(), "lower2"),
               Expression::placeholder(next_id(), "lower3"),
           ],
       )
   }
   
   /// Dot accent: ẋ (velocity, time derivative)
   pub fn template_dot_accent() -> Expression {
       Expression::operation(
           "dot_accent",
           vec![Expression::placeholder(next_id(), "variable")],
       )
   }
   
   /// Double dot accent: ẍ (acceleration)
   pub fn template_ddot_accent() -> Expression {
       Expression::operation(
           "ddot_accent",
           vec![Expression::placeholder(next_id(), "variable")],
       )
   }
   
   /// Matrix with parentheses: (a b; c d)
   pub fn template_pmatrix_2x2() -> Expression {
       Expression::operation(
           "pmatrix2x2",
           vec![
               Expression::placeholder(next_id(), "a11"),
               Expression::placeholder(next_id(), "a12"),
               Expression::placeholder(next_id(), "a21"),
               Expression::placeholder(next_id(), "a22"),
           ],
       )
   }
   
   /// Determinant matrix: |a b; c d|
   pub fn template_vmatrix_2x2() -> Expression {
       Expression::operation(
           "vmatrix2x2",
           vec![
               Expression::placeholder(next_id(), "a11"),
               Expression::placeholder(next_id(), "a12"),
               Expression::placeholder(next_id(), "a21"),
               Expression::placeholder(next_id(), "a22"),
           ],
       )
   }
   ```

2. **Add to HTML Palette (`static/index.html`):**
   ```html
   <!-- Tensor tab -->
   <button class="template-btn" onclick="insertTemplate('\\Gamma^{□}_{□ □}')">
       Γ^μ_νσ Christoffel
   </button>
   <button class="template-btn" onclick="insertTemplate('R^{□}_{□ □ □}')">
       R^ρ_σμν Riemann
   </button>
   <button class="template-btn" onclick="insertTemplate('T^{□□}')">
       T^ij Upper Pair
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
       (  ) Matrix 2×2
   </button>
   <button class="template-btn" onclick="insertTemplate('\\begin{vmatrix}□&□\\\\□&□\\end{vmatrix}')">
       |  | Determinant 2×2
   </button>
   ```

---

## 🎯 Recommendations

### Immediate Actions
1. **Add missing template functions** to `src/templates.rs` (see above)
2. **Add to palette** in `static/index.html`
3. **Create "Tensors" tab** in palette with:
   - Simple subscript/superscript (already have)
   - Mixed index (already have)
   - Double upper/lower indices
   - Christoffel symbol
   - Riemann tensor

### Medium Priority
1. **Expand "Accents" tab** with:
   - Dot (velocity)
   - Double dot (acceleration)
   - Hat, bar, tilde (already in improved_palette.html)

2. **Add matrix variants** to "Matrices" tab:
   - pmatrix (parentheses) - 2×2, 3×3
   - vmatrix (determinant) - 2×2, 3×3
   - Bmatrix (curly braces) - 2×2, 3×3

### Long-term
1. **Smart tensor builder** - GUI for building complex tensor expressions
2. **Bracket auto-sizing** - Automatically scale brackets to content
3. **Custom index notation** - User-defined tensor conventions

---

## 📝 Notes

- **Backend is solid**: All rendering infrastructure exists
- **Templates are incomplete**: Missing wrapper functions for many operations
- **Palette is sparse**: Only ~29 templates vs 79 proposed
- **Matrix 3×3 is broken**: Uses placeholder text instead of proper template

---

## ✅ Action Items

- [ ] Add 6 new template functions to `src/templates.rs`
- [ ] Add Christoffel and Riemann to palette
- [ ] Add dot/ddot accents to palette
- [ ] Add pmatrix and vmatrix variants
- [ ] Fix Matrix 3×3 template
- [ ] Create comprehensive test for all tensor operations
- [ ] Update documentation with tensor examples

---

**Bottom Line:** 

✅ **YES** - We have superscripts/subscripts for tensors (including Christoffel & Riemann in backend)  
✅ **YES** - We have dot notation derivatives (in backend)  
✅ **YES** - We have regular, curly, bracket parentheses (in backend)  

❌ **BUT** - Most are NOT exposed in the palette as clickable templates  
❌ **BUT** - Some lack wrapper functions in `src/templates.rs`

**The renderer works perfectly - we just need to expose these features in the UI.**

