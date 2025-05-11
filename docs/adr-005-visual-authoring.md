# ADR-005: Visual Math Authoring System for Kleis

## Status
Visionary / Accepted for Long-Term Roadmap

## Context
Kleis is not merely a symbolic language. It is a formal framework for defining new mathematical structures, operations, and semantic relationships. This includes user-defined:

- Operations (e.g., new morphisms, transformations, laws)
- Glyphs (visual representations)
- Templates (LaTeX, Unicode, or symbolic formatting)
- Type signatures (input/output constraints)

As the system evolves, manual input of increasingly complex and deeply nested symbolic expressions via text becomes both impractical and cognitively expensive. Furthermore, professional and academic users may wish to package, distribute, and visually interact with new symbolic definitions, including visual rendering.

## Decision
Kleis will support a long-term goal of building a **visual authoring environment** that enables:

### Key Capabilities
- **Visual construction of symbolic expressions**
- **Custom glyph/template assignment** for user-defined operations
- **Live preview of output** (Unicode, LaTeX, etc.)
- **Definition of types, inputs, outputs**, and bindings in a UI form
- **Export to `.kleis` source + metadata**
- **Visual-semantic synchronization** between layout and AST
- **Package-level templating**, allowing companies or individuals to distribute symbolic libraries with full visual definitions

## Strategic Vision
This visual editor will be to symbolic math what:
- Visual Studio Code is to software
- Figma is to design
- Blender is to modeling
- MathType never quite became for expressive symbolic reasoning

It will allow mathematicians, scientists, and system designers to:
- Create new symbolic languages
- Package and distribute their own libraries
- Visually demonstrate the semantics of abstract mathematical systems

## Consequences
- Adds new UI/UX dimension to Kleis roadmap
- Requires sync engine between visual layout and symbolic tree
- May necessitate client-side web editor or Electron app
- Enables third-party packages with glyph and semantic bundles

## Related
- ADR-004: Input Strategy for Complex Expressions
- ADR-001: Scalar Multiplication Visual Semantics
- Kleis Grammar v0.2