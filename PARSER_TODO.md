# LaTeX Parser - Current Status

This document tracks the current state and roadmap of the Kleis LaTeX parser.

---

## 📊 Current Coverage: 80% (Measured)

**Code Coverage (via cargo llvm-cov):**
- **Parser:** 78.48% line coverage (1120 lines, 241 missed)
  - 74.49% region coverage (988 regions, 252 missed)  
  - 95.16% function coverage (62 functions, 3 missed)
- **Renderer:** 82.20% line coverage (1876 lines, 334 missed)
  - 95.37% region coverage (475 regions, 22 missed)
  - 92.82% function coverage (181 functions, 13 missed)
- **Overall:** 80.22% line coverage, 80.45% region coverage

**Test Suite:** 110 tests passing (34 parser unit tests + 76 render tests)

The parser handles most common LaTeX mathematical expressions from standard math guides.

---

## ✅ **Fully Working (30+ patterns)**

### Core Parsing
- [x] **Fractions:** `\frac{a}{b}`
- [x] **Square roots:** `\sqrt{x}`, `\sqrt[n]{x}`
- [x] **Subscripts:** `x_{0}`, `x_{i}`
- [x] **Superscripts:** `x^{2}`, `x^{n}`
- [x] **Mixed indices:** `T^{\mu\nu}`, `F^\mu_{\ \nu}`

### Operators & Precedence
- [x] **Binary operators:** `+`, `-`, `*`, `/`
- [x] **Operator precedence:** `a + b * c` correctly parsed
- [x] **Implicit multiplication:** `2m` → `2 * m`, `\frac{1}{2m}` correctly parsed
- [x] **Unary operators:** `-x`, `+x`, `-\frac{1}{2}`

### Symbols
- [x] **Greek letters:** `\alpha`, `\beta`, `\gamma`, `\delta`, `\epsilon`, `\theta`, `\lambda`, `\mu`, `\nu`, `\pi`, `\rho`, `\sigma`, `\tau`, `\phi`, `\psi`, `\omega`
- [x] **Uppercase Greek:** `\Gamma`, `\Delta`, `\Theta`, `\Lambda`, `\Sigma`, `\Phi`, `\Psi`, `\Omega`
- [x] **Relations:** `\neq`, `\leq`, `\geq`, `\approx`, `\equiv`, `\propto`, `\in`
- [x] **Set operations:** `\cup`, `\cap`, `\subset`, `\subseteq`
- [x] **Logic:** `\forall`, `\exists`, `\Rightarrow`, `\Leftrightarrow`
- [x] **Operators:** `\cdot`, `\times`, `\div`, `\pm`, `\nabla`, `\partial`
- [x] **Arrows:** `\to`, `\rightarrow`, `\mapsto`
- [x] **Delimiters:** `\langle`, `\rangle`

### Functions
- [x] **Trigonometric:** `\sin{x}`, `\cos{x}`, `\tan{x}`
- [x] **Logarithmic:** `\ln{x}`, `\log{x}`
- [x] **Special functions:** `\min{...}`, `\max{...}`
- [x] **Function calls:** `f(x, y)` → multi-argument function_call
- [x] **Operator hat:** `\hat{x}`

### Environments
- [x] **Matrices:** `\begin{bmatrix}a&b\\c&d\end{bmatrix}` (2x2, 3x3, and general)
- [x] **Matrix variants:** `\begin{pmatrix}`, `\begin{vmatrix}`, `\begin{matrix}`
- [x] **Row separator:** `\\` correctly handled inside matrices

### Quantum Mechanics
- [x] **Ket vectors:** `|\psi\rangle`
- [x] **Bra vectors:** `\langle\phi|`
- [x] **Commutators:** `[A, B]`
- [x] **Anticommutators:** `\{A, B\}` (escaped braces)

### Formatting
- [x] **Number sets:** `\mathbb{R}`, `\mathbb{C}`, etc.
- [x] **Text formatting:** `\mathbf{...}`, `\boldsymbol{...}`, `\mathrm{...}`
- [x] **Box operator:** `\Box` (d'Alembertian)
- [x] **Spacing commands:** `\,`, `\!`, `\;` (ignored appropriately)
- [x] **Delimiters:** `\left(`, `\right)`, `\left\{`, `\right\}` (basic support)

### Integrals & Sums
- [x] **Integrals:** `\int`
- [x] **Sums:** `\sum`
- [x] **Products:** `\prod`

### Piecewise Functions
- [x] **Cases environment:** `\begin{cases}...\end{cases}` (2, 3, and N-row support)

### Advanced Greek
- [x] **Greek variants:** `\varepsilon`, `\varphi`, `\vartheta`, `\varkappa`, `\varpi`, `\varrho`, `\varsigma`

### Function Flexibility
- [x] **Trig with parentheses:** `\sin(x)`, `\cos(x)`, `\tan(x)` now work alongside `\sin{x}`
- [x] **Nested functions:** `\sin(\cos(x))` fully supported

---

## ❌ **Not Yet Supported (Known Gaps)**

### 1. **Text Mode** 🟡 MEDIUM PRIORITY
```latex
\text{if } x > 0
\text{for all } x \in \mathbb{R}
```
**Issue:** `\text{}` command not implemented  
**Effort:** 1 hour  
**Impact:** Labels in piecewise functions and annotations

### 2. **More Matrix Variants** 🟢 LOW PRIORITY
```latex
\begin{Bmatrix}...\end{Bmatrix}  % Curly braces
\begin{Vmatrix}...\end{Vmatrix}  % Double bars
```
**Issue:** Not all environment variants implemented  
**Effort:** 30 minutes  
**Impact:** bmatrix, pmatrix, vmatrix cover most use cases

---

## 🔧 **Known Issues**

### Complex Matrix Cells
Matrix cells with complex expressions may parse as string objects rather than structured expressions:
```latex
\begin{bmatrix}\frac{a}{b}&c\\d&e\end{bmatrix}
```
The parser currently collects cell content as raw strings in some cases. This doesn't break rendering but loses structure.

### Delimiter Matching
Complex delimiter nesting may not be fully robust:
```latex
\left\{ \left( ... \right) \middle| ... \right\}
```
Basic `\left` and `\right` work, but `\middle` is not supported.

---

## 🎯 **Priority Roadmap**

### ✅ Phase 1: Essential Completeness (COMPLETE!)
1. ✅ **Cases environment** - Enable piecewise functions (**DONE**)
2. ✅ **Parentheses for trig functions** - Allow `\sin(x)` alongside `\sin{x}` (**DONE**)
3. ✅ **Greek variants** - `\varepsilon`, `\varphi`, etc. (**DONE**)

**Achieved:** **80.2% measured line coverage** (78.5% parser, 82.2% renderer)

### Phase 2: Remaining Features (1-2 days)
1. **Text mode** - Support `\text{...}` for annotations
2. **Matrix cell parsing** - Full expression parsing inside matrix cells (partially done)
3. **More environments** - Variants like Bmatrix, Vmatrix (low priority)

**Target:** 80% → 85% coverage

### Phase 3: Polish (1-2 days)
4. **Advanced delimiters** - Better `\left`, `\middle`, `\right` handling
5. **Error messages** - Better error reporting with position info
6. **Edge cases** - Fix remaining parsing quirks
7. **Test the untested** - Cover the 3 missing parser functions (95.16% → 100%)

**Target:** 85% → 90% coverage

---

## 🧪 **Testing**

### Current Test Suite
- **110 total tests** passing ✅
  - **34 parser unit tests** in `parser.rs`
  - **76 renderer tests** in `render.rs`
  - **37 golden tests** (end-to-end integration)
- **Test binaries:** `test_parser`, `check_parser`, `test_guide_examples`, `test_top5`, etc.
- **Coverage:** Run `cargo llvm-cov --lib --summary-only` for current metrics

### Test Coverage
```rust
// Working examples
parse_latex(r"\frac{1}{2}")                    // ✅
parse_latex(r"\sqrt{x}")                       // ✅
parse_latex(r"x_{0}^{2}")                      // ✅
parse_latex(r"\begin{bmatrix}a&b\\c&d\end{bmatrix}")  // ✅
parse_latex(r"\{A, B\}")                       // ✅ (anticommutator)
parse_latex(r"2m")                             // ✅ (implicit mult)
parse_latex(r"-\frac{1}{2}")                   // ✅ (unary minus)
parse_latex(r"f(x, y)")                        // ✅ (multi-arg)
parse_latex(r"\begin{cases}x^2&x\geq 0\\0&x<0\end{cases}")  // ✅ (cases)
parse_latex(r"\sin(\cos(x))")                  // ✅ (nested with parens)
parse_latex(r"\varepsilon")                    // ✅ (Greek variants)
parse_latex(r"E = mc^{2}")                     // ✅ (equations)

// Not working
parse_latex(r"\text{if } x > 0")               // ❌ (text mode)
```

### Run Tests
```bash
# All tests
cargo test --all

# Parser tests only
cargo test parser::

# Specific test
cargo test parses_simple_matrix

# Interactive testing
cargo run --bin test_parser '\frac{1}{2}'

# Code coverage
cargo llvm-cov --lib --summary-only

# Coverage with details
cargo llvm-cov --lib --html
open target/llvm-cov/html/index.html
```

---

## 📝 **Design Notes**

### Parser Architecture
- **Recursive descent** parser with precedence climbing
- **Character stream** input (Vec<char>)
- **Expression AST** output
- **Error recovery** with position tracking

### Key Design Decisions

#### 1. Implicit Multiplication
Lowercase single letters are treated as separate variables:
- `ab` → `a * b`
- `2m` → `2 * m`
- `\frac{1}{2m}` → `1 / (2 * m)`

Uppercase letters consume full identifiers:
- `ABC` → single object "ABC"

#### 2. Environment Name Parsing
Environment names use `parse_text_group()` to avoid implicit multiplication:
- `\begin{bmatrix}` correctly parsed (not `b*m*a*t*r*i*x`)

#### 3. Function vs Braces
Commands like `\sin` expect braces `{...}`:
- `\sin{x}` ✅
- `\sin(x)` ❌ (could be fixed)

---

## 🔗 **Related Documentation**

- **Main README:** Project overview and quick start
- **Renderer:** `src/render.rs` - Expression → LaTeX/Unicode
- **Tests:** `src/parser.rs` - Unit tests at bottom of file
- **Golden Tests:** `tests/golden/` - End-to-end validation

---

## 🤝 **Contributing**

To add parser support for a new LaTeX construct:

1. **Add parsing logic** in `src/parser.rs`
   - Extend `parse_latex_command()` match statement
   - Add helper functions if needed

2. **Write tests**
   - Unit test in `parser.rs` tests module
   - Test with `test_parser` binary

3. **Update this document**
   - Move item from "Not Supported" to "Working"
   - Update coverage estimate

4. **Test with gallery**
   - Try parsing gallery examples
   - Verify round-trip (parse → render → LaTeX)

---

**Last Updated:** November 22, 2024  
**Parser Version:** 0.1.0  
**Status:** Phase 1 complete, **80.2% measured coverage**, 110/110 tests passing ✅  
**Coverage Detail:** Parser 78.5% | Renderer 82.2% | 95.2% function coverage  
**Maintainer:** Kleis Development Team
