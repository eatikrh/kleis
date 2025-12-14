# Equation Editor Palette → Kleis Rendering Coverage

**Date:** December 13, 2024  
**Purpose:** Audit all palette buttons for Kleis rendering support

## Summary

| Tab | Buttons | Kleis Renderer | Notes |
|-----|---------|----------------|-------|
| **Basics** | 15+ | ✅ Complete | Arithmetic, ±, ∓, ∗ all done |
| **Fences** | 8 | ✅ Complete | parens, brackets, floor, ceiling |
| **Accents** | 9 | ✅ Complete | dot, hat, bar, tilde, vec |
| **Calculus** | 14 | ✅ Complete | D(), Dt(), Fourier(), Laplace(), Convolve() |
| **Linear Algebra** | 12 | ⚠️ Partial | Matrices need literal syntax |
| **Greek** | 35 | ✅ Complete | All Greek letters have glyphs |
| **Logic & Sets** | 18+ | ✅ Complete | ∧, ∨, ¬, ∈, ⊂, quantifiers |
| **Physics** | 8 | ⚠️ Partial | Bra-ket parser issue |
| **POT** | 8 | ⚠️ Partial | Function call syntax works |

---

## Detailed Coverage

### Basics Tab ✅

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Minus | `-` | `minus` | ✅ |
| Times | `\times` | `multiply` | ✅ |
| Divide | `\div` | `scalar_divide` | ✅ |
| Plus-Minus | `\pm` | `plus_minus` → `PlusMinus(a, b)` | ✅ |
| Minus-Plus | `\mp` | `minus_plus` → `MinusPlus(a, b)` | ✅ |
| Dot | `\cdot` | `dot` | ✅ |
| Asterisk | `\ast` | `star` → `(a ∗ b)` | ✅ |
| Equal | `=` | `equals` | ✅ |
| Not Equal | `\neq` | `not_equal` | ✅ |
| Infinity | `\infty` | glyph | ✅ |
| Box | `\square` | `placeholder` | ✅ |
| Fraction | `\frac{}{}` | `frac` | ✅ |
| Power | `^{}` | `power` | ✅ |
| Subscript | `_{}` | `sub` | ✅ |
| Root | `\sqrt{}` | `sqrt` | ✅ |

### Fences Tab ✅

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Parentheses | `\left(\right)` | `parens` | ✅ |
| Brackets | `\left[\right]` | `brackets` | ✅ |
| Braces | `\left\{\right\}` | `braces` | ✅ |
| Angle | `\langle\rangle` | `inner` | ✅ |
| Absolute | `\left\|\right\|` | `abs` | ✅ |
| Norm | `\|\cdot\|` | `norm` | ✅ |
| Floor | `\lfloor\rfloor` | `floor` → `floor(x)` | ✅ |
| Ceiling | `\lceil\rceil` | `ceiling` → `ceiling(x)` | ✅ |

### Accents Tab ✅

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Dot | `\dot{}` | `dot_accent` → `ẋ` | ✅ |
| Double Dot | `\ddot{}` | `ddot_accent` → `ẍ` | ✅ |
| Hat | `\hat{}` | `hat` → `x̂` | ✅ |
| Bar | `\bar{}` | `bar` → `x̄` | ✅ |
| Tilde | `\tilde{}` | `tilde_accent` → `x̃` | ✅ |
| Overline | `\overline{}` | `overline(x)` | ✅ |
| Underline | `\underline{}` | `underline(x)` | ✅ |
| Vector | `\vec{}` | `vector_arrow` | ✅ |
| Bold | `\mathbf{}` | `vector_bold` | ✅ |

### Calculus Tab ✅

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Definite Integral | `\int_{}^{} \, dx` | `int_bounds` → `Integrate(f, x, a, b)` | ✅ |
| Summation | `\sum_{}^{}` | `sum_bounds` → `Sum(expr, i, a, b)` | ✅ |
| Product | `\prod_{}^{}` | `prod_bounds` → `Product(expr, i, a, b)` | ✅ |
| Limit | `\lim_{→}` | `lim` → `Limit(f, x, a)` | ✅ |
| Derivative | `\frac{d}{dx}` | `d_dt` → `Dt(f, x)` | ✅ |
| Partial | `\frac{\partial}{\partial}` | `d_part` → `D(f, x)` | ✅ |
| Gradient | `\nabla` | `gradient` → `∇f` | ✅ |
| Fourier | `\mathcal{F}[]` | `fourier` → `Fourier(f, ω)` | ✅ |
| Inv Fourier | `\mathcal{F}^{-1}[]` | `inv_fourier` → `InvFourier(F, t)` | ✅ |
| Laplace | `\mathcal{L}[]` | `laplace` → `Laplace(f, s)` | ✅ |
| Inv Laplace | `\mathcal{L}^{-1}[]` | `inv_laplace` → `InvLaplace(F, t)` | ✅ |
| Convolution | `\ast` | `convolution` → `Convolve(f, g)` | ✅ |
| Kernel Integral | `\int_{} K \, d\mu` | Standard integral template | ✅ |
| Green's Function | `G(x, m)` | Function call syntax | ✅ |

### Linear Algebra Tab ⚠️

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Matrix 2×2 | `\begin{bmatrix}...\end{bmatrix}` | - | ❌ Need matrix template |
| Matrix 3×3 | `\begin{bmatrix}...\end{bmatrix}` | - | ❌ Need matrix template |
| pmatrix | `\begin{pmatrix}...\end{pmatrix}` | - | ❌ |
| Determinant | `\begin{vmatrix}...\end{vmatrix}` | `det` | ✅ (for det(A)) |
| Matrix Builder | custom | - | ❌ |
| Bold Vector | `\mathbf{v}` | `vector_bold` | ✅ |
| Arrow Vector | `\vec{v}` | `vector_arrow` | ✅ |
| Matrix Mult | `A \bullet B` | - | ❌ Need template |
| Dot Product | `a \cdot b` | `dot` | ✅ |
| Cross Product | `a \times b` | `cross` | ✅ |
| Transpose | `^T` | `transpose` | ✅ |
| Trace | `\text{tr}` | `trace` | ✅ |

### Greek Tab ✅

All Greek letters have Kleis glyphs:
- Lowercase: α, β, γ, δ, ε, ζ, η, θ, ι, κ, λ, μ, ν, ξ, π, ρ, σ, τ, υ, φ, χ, ψ, ω
- Uppercase: Γ, Δ, Θ, Λ, Ξ, Π, Σ, Υ, Φ, Ψ, Ω

### Logic & Sets Tab ✅

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| True | `\text{True}` | - | ✅ (parsed as identifier) |
| False | `\text{False}` | - | ✅ |
| Equals | `=` | `equals` | ✅ |
| Not Equal | `\neq` | `not_equal` | ✅ |
| Less Than | `<` | `less_than` | ✅ |
| Greater Than | `>` | `greater_than` | ✅ |
| Leq | `\leq` | `leq` | ✅ |
| Geq | `\geq` | `geq` | ✅ |
| Approx | `\approx` | `approx` | ✅ |
| AND | `\land` | `logical_and` | ✅ |
| OR | `\lor` | `logical_or` | ✅ |
| NOT | `\lnot` | `logical_not` | ✅ |
| Equiv | `\equiv` | `equiv` | ✅ |
| Element Of | `\in` | `in_set` | ✅ |
| Not In | `\notin` | `not_in_set` | ✅ |
| Subset | `\subset` | `subset` | ✅ |
| Subseteq | `\subseteq` | `subseteq` | ✅ |
| Union | `\cup` | `union` | ✅ |
| Intersection | `\cap` | `intersection` | ✅ |
| Implies | `\Rightarrow` | `implies` | ✅ |
| Iff | `\Leftrightarrow` | `iff` | ✅ |
| Forall | `\forall` | `forall` | ✅ |
| Exists | `\exists` | `exists` | ✅ |

### Physics Tab ⚠️

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Ket | `\|\psi\rangle` | `ket` | ⚠️ Parser issue |
| Bra | `\langle\phi\|` | `bra` | ⚠️ Parser issue |
| Inner Product | `\langle\|\rangle` | `braket` | ⚠️ Parser issue |
| Outer Product | `\|\rangle\langle\|` | - | ❌ Need template |
| Expectation | `\langle A \rangle` | - | ❌ Need template |
| Commutator | `[A, B]` | - | ❌ Need template |
| Christoffel | `\Gamma^{\lambda}_{\mu\nu}` | - | ❌ Need template |
| Riemann | `R^{\rho}_{\sigma\mu\nu}` | - | ❌ Need template |

### POT Tab ❌

| Button | LaTeX | Kleis Template | Status |
|--------|-------|----------------|--------|
| Projection | `\Pi[\psi](x)` | - | ❌ Need template |
| Modal Integral | `\int_M d\mu(m)` | - | ❌ Need template |
| Kernel | `K(x, m)` | - | ✅ Function call |
| Causal Bound | `c(x)` | - | ✅ Function call |
| Residue | `\mathrm{Residue}[\Pi, X]` | - | ❌ Need template |
| Modal Space | `\mathcal{M}_H` | - | ❌ Need template |
| Spacetime | `\mathbb{R}^4` | - | ✅ (identifier + superscript) |
| Hont | `\mathcal{H}_\infty` | - | ❌ Need template |

---

## Priority Actions

### High Priority (Calculus Complete ✅)
- [x] D(f, x) - Partial derivative
- [x] Dt(f, x) - Total derivative
- [x] Integrate(f, x, a, b) - Definite integral
- [x] Sum(expr, i, a, b) - Summation
- [x] Product(expr, i, a, b) - Product
- [x] Limit(f, x, a) - Limit
- [x] ∇f - Gradient

### Medium Priority (Add Templates)
- [ ] floor, ceiling fences
- [ ] dot, hat, bar, tilde accents
- [ ] Fourier, Laplace transforms
- [ ] Convolution
- [ ] Matrix literals

### Lower Priority (Physics/POT)
- [ ] Bra-ket notation (parser fix needed)
- [ ] Commutator
- [ ] POT operators

---

## Coverage Summary

| Category | Coverage |
|----------|----------|
| Core Arithmetic | 100% ✅ |
| Fences | 100% ✅ |
| Accents | 100% ✅ |
| Calculus | 100% ✅ |
| Logic/Sets | 100% ✅ |
| Greek | 100% ✅ |
| Linear Algebra | 70% |
| Physics | 40% |
| POT | 40% |
| **Overall** | **~90%** |

