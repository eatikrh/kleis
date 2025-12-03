# DLMF Integration for Kleis Testing

## Overview

The Kleis project now includes **36 curated equations** from the NIST Digital Library of Mathematical Functions (DLMF) for comprehensive testing of the structural editor.

## What is DLMF?

The **Digital Library of Mathematical Functions** (https://dlmf.nist.gov/) is:
- Successor to the legendary Abramowitz & Stegun handbook (1964)
- Maintained by NIST (National Institute of Standards and Technology)
- Contains thousands of formulas for special functions
- Gold standard reference for mathematical software

## Generated Test Suite

### Location
```
tests/golden/sources/dlmf/
  ├── gamma_function.tex          (5 equations)
  ├── bessel_functions.tex        (5 equations)
  ├── hypergeometric.tex          (4 equations)
  ├── legendre_polynomials.tex    (4 equations)
  ├── zeta_function.tex           (5 equations)
  ├── elliptic_integrals.tex      (4 equations)
  ├── orthogonal_polynomials.tex  (4 equations)
  └── special_cases.tex           (5 equations)
```

### Coverage Analysis

The curated equations test these Kleis templates:

| Template Category | Examples | Count |
|------------------|----------|-------|
| **Integrals** | `∫₀^∞ t^(z-1) e^(-t) dt` | ~15 |
| **Fractions** | `π / sin(πz)` | ~20 |
| **Greek Letters** | `Γ, ζ, ψ, φ, θ` | ~25 |
| **Subscripts** | `t_{s-1}`, `n^s` | ~15 |
| **Summations** | `Σ_{n=1}^∞` | ~8 |
| **Derivatives** | `d/dz ln Γ(z)` | ~3 |
| **Square Roots** | `√π`, `√(1-k²sin²θ)` | ~6 |
| **Function Calls** | `sin`, `cos`, `arccos` | ~12 |
| **Limits** | `_0^∞`, `_{-1}^1` | ~18 |

## Usage

### Generate the Test Files

```bash
# Generate all topics (default)
python3 scripts/fetch_dlmf_v2.py

# Generate specific topics only
python3 scripts/fetch_dlmf_v2.py --topics gamma_function,zeta_function

# Custom output directory
python3 scripts/fetch_dlmf_v2.py --output my_custom_dir/
```

### Run Tests

```bash
# Run golden tests (if integrated)
cargo test golden_tests

# Or test manually by loading into the editor
# Visit http://localhost:3000 and import the .tex files
```

## Example Equations

### Gamma Function

```latex
\[ \Gamma(z) = \int_0^\infty t^{z-1} e^{-t} \, dt \]
```
- Tests: integral, subscript, superscript, Greek letter
- DLMF Reference: 5.2.1

### Riemann Zeta

```latex
\[ \zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s} \]
```
- Tests: summation, limits, fraction, superscript
- DLMF Reference: 25.2.1

### Legendre Polynomials (Orthogonality)

```latex
\[ \int_{-1}^1 P_m(x) P_n(x) \, dx = \frac{2}{2n+1} \delta_{mn} \]
```
- Tests: integral, subscripts, function calls, fraction
- DLMF Reference: 18.3.1

### Hypergeometric Series

```latex
\[ {}_2F_1(a,b;c;z) = \sum_{n=0}^\infty \frac{(a)_n(b)_n}{(c)_n} \frac{z^n}{n!} \]
```
- Tests: subscripts, summation, Pochhammer symbol, nested fractions
- DLMF Reference: 15.2.1

## Why These Equations?

### 1. **Real-World Complexity**
- Not toy examples - actual mathematical formulas from literature
- Used in physics, engineering, statistics, and applied math

### 2. **Template Coverage**
- Exercises nearly all Kleis palette templates
- Finds edge cases in nested structures

### 3. **Canonical Forms**
- Standard notation from authoritative source
- Can validate against established references

### 4. **Historical Significance**
- Many famous formulas (Basel problem, Euler's reflection formula)
- Good for demos and documentation

## Extending the Collection

To add more equations, edit `scripts/fetch_dlmf_v2.py`:

```python
DLMF_EQUATIONS = {
    # ... existing topics ...
    
    "my_new_topic": [
        (r"\LaTeX{code}", "DLMF_ID", "Description"),
        # Add more equations...
    ],
}
```

Then regenerate:
```bash
python3 scripts/fetch_dlmf_v2.py
```

## Alternative: Web Scraping (Experimental)

For automated fetching from the DLMF website:

```bash
# Experimental - DLMF uses MathML which is hard to parse
python3 scripts/fetch_dlmf.py --chapters 5,10,25
```

**Note:** This approach is still in development. The curated version (`fetch_dlmf_v2.py`) is more reliable.

## Next Steps

1. **Integrate with Golden Tests**
   - Add DLMF files to golden test runner
   - Generate reference outputs
   - Track rendering accuracy

2. **Gallery Showcase**
   - Feature select DLMF equations in the web UI
   - "Famous Formulas" section for demos

3. **Expand Collection**
   - Add more topics (Mathieu functions, Painlevé transcendents)
   - Include matrix formulas (3j/6j/9j symbols)
   - Add asymptotic expansions

4. **Validate Rendering**
   - Compare Kleis output against DLMF reference images
   - Measure visual accuracy
   - Benchmark performance

## References

- **DLMF Website**: https://dlmf.nist.gov/
- **Abramowitz & Stegun** (1964): Original handbook
- **NIST**: National Institute of Standards and Technology
- **Handbook Tradition**: HÜTTE, Bronshtein, Gradshteyn & Ryzhik

---

**Status**: ✅ Complete - 36 equations ready for testing  
**Generated**: 2024-12-03  
**Maintainer**: Automated via `scripts/fetch_dlmf_v2.py`

