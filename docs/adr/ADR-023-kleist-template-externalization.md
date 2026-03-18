# ADR-023: Template Externalization with .kleist Files

**Status:** Accepted  
**Date:** 2024-12-17  
**Tags:** templates, rendering, architecture, extensibility

## Context

The equation editor renders mathematical notation using templates that map operation names to output formats (LaTeX, Typst, HTML, Unicode, Kleis). Previously, these templates were hardcoded in `render_editor.rs` (~900 lines of Rust code), making them:

1. **Hard to modify** - Required Rust knowledge and recompilation
2. **Not user-extensible** - Users couldn't add custom notation
3. **Duplicated** - Similar logic in `render.rs` and `render_editor.rs`
4. **Mixed concerns** - Rendering patterns embedded in code

## Decision

Externalize templates to `.kleist` files with a custom grammar and parser.

### New File Format: `.kleist`

```kleist
@template integral {
    pattern: "integral(lower, upper, body, var)"
    unicode: "∫_{lower}^{upper} {body} d{var}"
    latex: "\\int_{{lower}}^{{upper}} {body} \\, d{var}"
    html: "∫<sub>{lower}</sub><sup>{upper}</sup> {body} d{var}"
    typst: "integral_({lower})^({upper}) {body} dif {var}"
    kleis: "integral({lower}, {upper}, {body}, {var})"
    category: "calculus"
    glyph: "∫"
    svg: "<svg>...</svg>"
}

@tool matrix_builder {
    glyph: "⊞"
    handler: "showMatrixBuilder"
    category: "linear_algebra"
    builtin: "true"
    svg: "<svg>...</svg>"
}

@palette {
    tab "Calculus" {
        group "Integrals" {
            integral shortcut: "Ctrl+I"
        }
        @matrix_builder
    }
}
```

### Three Block Types

| Block | Purpose | Example |
|-------|---------|---------|
| `@template` | Static rendering patterns | `plus`, `integral`, `α` |
| `@tool` | Interactive builders | `matrix_builder`, `piecewise_builder` |
| `@palette` | UI layout with tabs/groups | Organizes buttons |

### Key Insight: Symbols Are Templates

Greek letters and math symbols (α, β, ∞, ∂, ∇) are just **zero-argument templates**:

```kleist
@template α {
    unicode: "α"
    latex: "\\alpha"
    typst: "alpha"
    html: "α"
    glyph: "α"
}
```

This unifies the model - no special `@symbol` block needed.

## Implementation

### Files Created

```
std_template_lib/
├── basic.kleist      # Arithmetic, comparison, special
├── calculus.kleist   # Integrals, derivatives, limits
├── greek.kleist      # Greek letters + math symbols
├── logic.kleist      # Boolean, sets, quantifiers
├── quantum.kleist    # Ket, bra, commutator
├── vectors.kleist    # Grad, div, curl
├── tensors.kleist    # Christoffel, Riemann, metric
├── transforms.kleist # Fourier, Laplace, convolution
├── pot.kleist        # POT-specific templates
├── tools.kleist      # Interactive builders
└── palette.kleist    # UI layout (11 tabs)
```

### Parser: `src/kleist_parser.rs`

- Full tokenizer with Unicode identifier support (α, ∞, ∂)
- Parses `@template`, `@tool`, `@palette` blocks
- Handles comments (`//` and `/* */`)
- Returns `KleistFile` with templates, tools, palette

### Loader: `src/render_editor.rs`

```rust
// Thread-local context loads from std_template_lib
thread_local! {
    static DEFAULT_CONTEXT: EditorRenderContext = 
        EditorRenderContext::from_std_template_lib();
}

// Loads all .kleist files from directory
pub fn from_std_template_lib() -> Self {
    match load_kleist_directory("std_template_lib") {
        Ok(file) => Self::from_kleist_file(&file),
        Err(_) => Self::new(),  // Fallback to hardcoded
    }
}
```

### Object Rendering with Templates

```rust
fn render_object_with_context(s: &str, ctx: &EditorRenderContext, target: &RenderTarget) -> String {
    // Try template lookup first (e.g., "α" -> "\\alpha" for LaTeX)
    if let Some(template) = ctx.get_template(s, target) {
        return template;
    }
    // Fallback to hardcoded
    render_object_for_target(s, target)
}
```

## Consequences

### Positive

1. **User-extensible** - Add templates without Rust knowledge
2. **Hot-swappable** (future) - Reload without recompilation
3. **Self-documenting** - Templates readable as specification
4. **Unified model** - Operations and symbols both use templates
5. **UI-definable** - Palette layout in data, not code

### Negative

1. **Runtime parsing** - Slight startup overhead (negligible)
2. **Two systems** - Hardcoded fallback still exists
3. **Grammar to maintain** - New `.kleist` format needs docs

### Neutral

1. **139 templates** loaded from 11 `.kleist` files
2. **Hardcoded fallback** preserved for safety
3. **No breaking changes** - Server API unchanged

## Architecture Dependency

This was only possible because of **ADR-022** (render_editor.rs decoupling). The prior refactor separated `EditorNode` rendering from `Expression`, creating a clean context where templates could be externalized.

```
Before ADR-022:
  Expression → render.rs → hardcoded patterns

After ADR-022:
  EditorNode → render_editor.rs → EditorRenderContext → templates

After ADR-023:
  EditorNode → render_editor.rs → EditorRenderContext → .kleist files
```

## Future Work

1. **Remove hardcoded fallback** - Once `.kleist` files are battle-tested
2. **Hot reload** - Watch for file changes, reload templates
3. **User template directories** - `~/.kleis/templates/`
4. **Generate palette UI** - Build React components from `palette.kleist`
5. **Validation** - Check template placeholders match patterns
6. **Delete render.rs** - The original 6900-line renderer is likely obsolete

## Note on render.rs

The original `render.rs` (6900+ lines) is **probably no longer used**. It has been superseded by `render_editor.rs` which:

- Renders `EditorNode` directly (not `Expression`)
- Preserves metadata (tensor indices, operation kinds)
- Uses external `.kleist` templates

We are **not deleting it yet** because:
1. It serves as reference during transition
2. Some edge cases may still need it
3. Historical comparison is valuable

A deprecation notice has been added to the top of `render.rs`.

## References

- Tag: `templates-externalized`
- Grammar: `docs/grammar/kleist_grammar.ebnf`
- Parser: `src/kleist_parser.rs`
- Templates: `std_template_lib/*.kleist`

