# ADR-001: Scalar Multiplication Visual Semantics

## Status
Accepted

## Context
In the initial rendering pipeline of Kleis, `scalar_multiply` operations were visually rendered using the `×` glyph in both Unicode and LaTeX outputs.

However, this caused semantic confusion:
- In mathematical typesetting, `×` is commonly reserved for vector cross products.
- Scalar multiplication is typically represented without any glyph or with minimal spacing, especially in LaTeX.

Thus, scalar multiplication must be disambiguated visually from other forms of multiplication.

## Decision
We will remove the explicit `×` glyph from `scalar_multiply` in both glyph maps.

Instead:
- **Unicode rendering**: A regular space will be used to separate left and right operands.
- **LaTeX rendering**: A thin space `\,` will be inserted between operands.

Template will be:
```kleis
latex_templates.insert("scalar_multiply".to_string(), "{left} \, {right}".to_string())
```

No glyph will be defined for `scalar_multiply` in the `latex_glyphs` or `unicode_glyphs`.

## Consequences
- This avoids semantic ambiguity with cross products.
- Scalar multiplications will now visually resemble their common typographic representations in math papers and books.
- Rendering remains accurate and cognitively optimized without altering symbolic structure.

## Related
- Kleis Grammar v0.2 rendering section
- Planned: cognitive load modeling via Kolmogorov-inspired heuristics