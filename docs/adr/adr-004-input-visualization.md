# ADR-004: Input Strategy for Complex and User-Defined Mathematical Expressions

## Status
Accepted

## Context
Kleis supports symbolic expressions that include standard operators (e.g., grad, integral, scalar_divide) as well as user-defined operations, many of which are associated with glyphs and rendering templates.

Entering these expressions manually via text — especially as the language grows to support:
- Unicode-based symbolic representations,
- Custom operators with visual glyphs,
- Non-standard syntax (e.g., ∮, ∇, ⨂, ↦, etc.),

...will become cumbersome and error-prone using standard keyboard input.

MathML is a standardized markup language for mathematical expressions, but it lacks support for:
- User-defined operations and glyphs
- Declarative binding of meaning to symbol
- Integration with a symbolic evaluation engine

## Decision
Kleis will pursue a layered input strategy:

### Phase 1: Kleis DSL Input
- Continue to allow structured symbolic input via the Kleis DSL (`.kleis` files).
- Complex operators will be defined symbolically, using ASCII-safe fallbacks.

### Phase 2: Visual Editor (MathType-like)
- Develop a visual expression editor inspired by MathType, enabling drag-and-drop or inline construction of expressions.
- Support for entering:
  - User-defined operations
  - Custom glyph bindings
  - Template previews in Unicode or LaTeX

### Phase 3: Symbolic-Visual Sync
- When defining a new operation in Kleis:
  - Allow user to bind rendering templates and glyphs.
  - Enable immediate visual rendering in the editor.
  - Persist symbolic definitions alongside visual metadata.

## Consequences
- Complex input becomes accessible to a wider range of users.
- Authoring mathematical documents and theories becomes intuitive.
- The system remains extensible: new operators can be both declared and visually bound.
- MathML will be treated as a potential export format, but not the internal input format due to its limitations.

## Related
- Kleis Rendering Pipeline
- ADR-001: Scalar Multiplication Semantics
- Planned: Math authoring assistant using Kleis