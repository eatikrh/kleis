# Equation Editor Documentation

Documentation for the Kleis Equation Editor - a visual WYSIWYG editor for mathematical notation.

---

## Technical Reference

| Document | Description |
|----------|-------------|
| [TECHNICAL_REFERENCE.md](TECHNICAL_REFERENCE.md) | Architecture, rendering pipeline, API reference |
| [COMPLETE_TEMPLATE_REFERENCE.md](COMPLETE_TEMPLATE_REFERENCE.md) | All 157 templates with examples |

---

## User Guides

| Document | Description |
|----------|-------------|
| [PALETTE_GUIDE.md](PALETTE_GUIDE.md) | Template palette organization and usage |
| [PALETTE_ICON_STRATEGY.md](PALETTE_ICON_STRATEGY.md) | Icon design principles |
| [INLINE_EDITING.md](INLINE_EDITING.md) | Inline editing workflow |
| [INLINE_EDITING_BUTTON_BEHAVIOR.md](INLINE_EDITING_BUTTON_BEHAVIOR.md) | Button behavior specifications |

---

## Feature Documentation

| Document | Description |
|----------|-------------|
| [INTEGRAL_TRANSFORMS.md](INTEGRAL_TRANSFORMS.md) | Fourier, Laplace, and other transforms |
| [LET_BINDINGS.md](LET_BINDINGS.md) | Let binding support in equations |

---

## Quick Start

The Equation Editor runs as part of `kleis server`:

```bash
# Start the server
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
cargo run --release --bin server

# Open in browser
open http://localhost:3000
```

### Key Features

- **Visual WYSIWYG editing** - See rendered math as you type
- **157 templates** - Covering calculus, linear algebra, quantum mechanics, tensors
- **Multi-target rendering** - Typst, LaTeX, HTML, Unicode
- **Type checking** - Validates mathematical structure
- **Copy Typst** - Export equations for thesis documents

---

## Related

- [Manual Chapter](../manual/src/chapters/23-document-generation.md) - Document generation with equations
- [ADR-023: Template Externalization](../adr/ADR-023-kleist-template-externalization.md) - `.kleist` file format

---

**Last Updated:** January 2026

