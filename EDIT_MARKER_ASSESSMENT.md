# Edit Marker Positioning Assessment

**Date:** November 24, 2024  
**Source:** Manual testing in main equation editor (localhost:3000)  
**Method:** Visual inspection of each template in structural mode

---

## Assessment Instructions

Please test each template in the **main equation editor** at `http://localhost:3000`:

1. Click "🔧 Structural Mode"
2. Click each template button
3. Observe the green/blue overlay boxes
4. Rate the alignment:
   - ✅ **Good** - Overlay perfectly covers the □ placeholder
   - ⚠️ **Offset** - Overlay visible but slightly off-center
   - ❌ **Bad** - Overlay completely wrong position or invisible

---

## Templates to Test

### Basic Operations (10 templates)
- [ ] Fraction (`\frac{□}{□}`)
- [ ] Square Root (`\sqrt{□}`)
- [ ] Nth Root (`\sqrt[□]{□}`)
- [ ] Power (`x^{□}`)
- [ ] Subscript (`x_{□}`)
- [ ] Mixed Index (`x^{□}_{□}`)
- [ ] Binomial (`\binom{□}{□}`)
- [ ] Factorial (`□!`)
- [ ] Floor (`\lfloor □ \rfloor`)
- [ ] Ceiling (`\lceil □ \rceil`)

### Calculus (7 templates)
- [ ] Integral (`\int_{□}^{□} □ \, dx`)
- [ ] Sum (`\sum_{□}^{□} □`)
- [ ] Product (`\prod_{□}^{□} □`)
- [ ] Limit (`\lim_{□ \to □} □`)
- [ ] Partial (`\partial_{□} □`)
- [ ] Derivative (`\frac{d □}{d □}`)
- [ ] Gradient (`\nabla □`)

### Matrices (6 templates)
- [ ] Matrix 2×2 [brackets]
- [ ] Matrix 3×3 [brackets]
- [ ] Matrix 2×2 (parens)
- [ ] Matrix 3×3 (parens)
- [ ] Determinant 2×2
- [ ] Determinant 3×3

### Quantum (6 templates)
- [ ] Ket (`|□\rangle`)
- [ ] Bra (`\langle□|`)
- [ ] Inner Product (`\langle□|□\rangle`)
- [ ] Outer Product (`|□\rangle\langle□|`)
- [ ] Commutator (`[□, □]`)
- [ ] Expectation (`\langle □ \rangle`)

### Vectors (6 templates)
- [ ] Bold Vector (`\mathbf{v}`)
- [ ] Vector Arrow (`\vec{v}`)
- [ ] Dot Product (`a \cdot b`)
- [ ] Cross Product (`a \times b`)
- [ ] Norm (`\|v\|`)
- [ ] Absolute Value (`|x|`)

### Functions (10 templates)
- [ ] Sine (`\sin(□)`)
- [ ] Cosine (`\cos(□)`)
- [ ] Tangent (`\tan(□)`)
- [ ] Arcsine (`\arcsin(□)`)
- [ ] Arccosine (`\arccos(□)`)
- [ ] Arctangent (`\arctan(□)`)
- [ ] Natural Log (`\ln(□)`)
- [ ] Logarithm (`\log(□)`)
- [ ] Exponential (`\exp(□)`)
- [ ] e to power (`e^{□}`)

### Accents (5 templates)
- [ ] Dot (`\dot{□}`)
- [ ] Double Dot (`\ddot{□}`)
- [ ] Hat (`\hat{□}`)
- [ ] Bar (`\bar{□}`)
- [ ] Tilde (`\tilde{□}`)

### Tensors (2 templates)
- [ ] Christoffel (`\Gamma^{□}_{□ □}`)
- [ ] Riemann (`R^{□}_{□ □ □}`)

---

## Results Template

Fill in as you test:

### Good Alignment ✅
(List templates that work perfectly)
- factorial (confirmed)
- 

### Slight Offset ⚠️
(List templates with minor misalignment but usable)
- matrices (confirmed - acceptable for empty cells)
- 

### Bad Alignment ❌
(List templates that need fixing)
- 

---

## Key Questions

1. **Superscripts/Subscripts:** Do power and subscript work well in main editor?
2. **Large Operators:** Do integral, sum, product, limit work well?
3. **Matrices:** Are they usable despite offset?
4. **Simple Operations:** Do sqrt, fraction work well?

---

## Next Steps

Based on your assessment:
- If most are "Good" → Minor tweaks only
- If many are "Offset" → Adjust size reduction factors
- If many are "Bad" → Need coordinate system overhaul

Please test in the main editor and document what you find!

