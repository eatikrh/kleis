#!/bin/bash
# Download LaTeX style guide PDFs for golden test references

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Downloading LaTeX style guide PDFs..."

# AMS Short Math Guide
echo "1. AMS Short Math Guide for LaTeX..."
curl -L -o ams_short_math_guide.pdf \
  "https://ctan.math.illinois.edu/info/short-math-guide/short-math-guide.pdf"

# Journal of Integer Sequences guide
echo "2. Journal of Integer Sequences LaTeX recommendations..."
curl -L -o jis_latex_guide.pdf \
  "https://emis.dsd.sztaki.hu/journals/JIS/texrecs.pdf"

# Create extraction instructions
cat > EXTRACTION_NOTES.md << 'EOF'
# Extracting Examples from PDFs

## Manual Extraction Process

1. Open the PDF in a viewer
2. Find mathematical expressions you want as golden tests
3. Copy the LaTeX source (usually shown in the PDF)
4. Save to appropriate category file

## Categories to Extract

### From AMS Guide:
- Section 3: Equations
- Section 4: Displayed equations  
- Section 5: Building blocks
- Section 6: Delimiters
- Section 7: Operators
- Section 8: Math symbols

### From JIS Guide:
- Variables and expressions
- Summations and products
- Fractions and binomials
- Integrals
- Matrices

## Example Format

Create files like `ams/integrals.tex`:

```latex
% Example 1: Definite integral
\int_{0}^{\infty} e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2}

% Example 2: Multiple integral
\iint_{D} f(x,y) \, dx \, dy

% Example 3: Contour integral
\oint_{C} f(z) \, dz
```

Then create corresponding Kleis test in `tests/golden_tests.rs`
EOF

echo ""
echo "✓ Downloaded style guides successfully!"
echo "✓ Created extraction instructions in EXTRACTION_NOTES.md"
echo ""
echo "Next steps:"
echo "1. Review the PDFs"
echo "2. Extract examples to sources/ams/, sources/ieee/, etc."
echo "3. Create golden tests based on examples"

