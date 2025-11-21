# Golden Test Examples for Kleis Renderer

This directory contains reference examples from authoritative LaTeX style guides, used to validate the rendering output of Kleis.

## Directory Structure

```
tests/golden/
├── sources/           # Original LaTeX examples from style guides
│   ├── ams/          # AMS Short Math Guide examples
│   ├── ieee/         # IEEE Math Typesetting Guide examples
│   └── custom/       # Our own curated examples
├── outputs/          # Generated output from Kleis renderer (auto-generated)
└── references/       # Expected/golden outputs (manually verified)
```

## Sources

### Free Resources (Public Domain / Permissive License)

1. **AMS Short Math Guide for LaTeX**
   - URL: https://ctan.math.illinois.edu/info/short-math-guide/short-math-guide.pdf
   - License: Free to use for educational purposes
   - Download: `curl -O https://ctan.math.illinois.edu/info/short-math-guide/short-math-guide.pdf`

2. **IEEE Math Typesetting Guide**
   - URL: https://conferences.ieeeauthorcenter.ieee.org/wp-content/uploads/sites/8/IEEE-Math-Typesetting-Guide-for-LaTeX-Users.pdf
   - License: Educational use
   - Download: See URL

3. **Journal of Integer Sequences LaTeX Guide**
   - URL: https://emis.dsd.sztaki.hu/journals/JIS/texrecs.pdf
   - License: Public
   - Download: `curl -O https://emis.dsd.sztaki.hu/journals/JIS/texrecs.pdf`

## Creating Golden Tests

### Step 1: Extract Examples

From the PDF guides, manually extract LaTeX snippets:

```latex
% Example from AMS guide
\int_{0}^{\infty} e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2}
```

Save to `sources/ams/integrals.tex`

### Step 2: Create Kleis Equivalent

Create corresponding Kleis expression in test:

```rust
let integral = int_e(
    func("exp", vec![minus(c("0"), pow_e(o("x"), c("2")))]),
    c("0"),
    o("\\infty"),
    o("x")
);
let rhs = over(func("sqrt", vec![o("\\pi")]), c("2"));
let eq = equals(integral, rhs);
```

### Step 3: Store Golden Output

Run test once, manually verify, then save to `references/`:

```bash
cargo test test_name -- --nocapture > tests/golden/references/integrals.txt
```

### Step 4: Create Regression Test

```rust
#[test]
fn golden_ams_integrals() {
    let output = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
    let expected = include_str!("../tests/golden/references/integrals.txt");
    assert_eq!(output.trim(), expected.trim());
}
```

## Test Categories

Organize examples by mathematical domain:

- `algebra/` - Basic operations, polynomials
- `calculus/` - Derivatives, integrals, limits
- `linear_algebra/` - Matrices, vectors, determinants
- `set_theory/` - Sets, logic, relations
- `number_theory/` - Modular arithmetic, primes
- `differential_geometry/` - Tensors, curvature
- `quantum/` - Bra-ket, operators, commutators
- `statistics/` - Probability, distributions

## Maintenance

1. **When adding new operations**: Create golden test first
2. **When changing renderer**: All golden tests must pass
3. **Update golden outputs**: Only after manual verification
4. **Document changes**: Note in git commit why output changed

## Running Golden Tests

```bash
# Run all golden tests
cargo test golden_

# Update all outputs (manual verification needed!)
cargo test golden_ -- --nocapture > tests/golden/outputs/all.txt

# Compare outputs
diff tests/golden/references/test.txt tests/golden/outputs/test.txt
```

