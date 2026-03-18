# ADR-006: Template–Grammar Duality and Synchronization in Kleis

## Status
Accepted

## Context
Kleis allows users to define new operations and bind them to visual templates for LaTeX, Unicode, and other render targets. For example:

```kleis
operation wedge : (Form, Form) -> Form
template wedge {glyph: "∧", latex: "{left} \wedge {right}", unicode: "{left} ∧ {right}"}
```

This introduces a duality between:
- The **grammar**, which defines what symbolic expressions are structurally valid
- The **template**, which defines how those expressions are visually rendered

As the Kleis DSL grows, and particularly when user-defined operations become the norm, it is essential to maintain a consistent and logical relationship between templates and the core grammar.

## Decision

Kleis will adopt a **template–grammar synchronization principle**:

### 1. Operation-Arity Awareness
Every `operation` definition defines the arity and type signature. The grammar must reflect this in the parser. Templates must match this arity when using placeholders like `{left}`, `{right}`, `{arg}`.

### 2. Optional Template Binding
Operations may be defined without visual templates, but any template provided must conform to the expected grammar form.

### 3. Validation Layer
A template-grammar validator will ensure:
- Arity match between operation definition and placeholders
- Placeholder names match allowed symbolic roles
- Invalid combinations raise warnings or errors (e.g., 2-arg op with only `{arg}` in template)

### 4. Visual Tool Integration
Visual editor components will auto-populate grammar structure when templates are added. Users will be guided in linking templates and grammar rules intuitively.

## Consequences
- Rendering is always semantically safe
- Grammar extensions through `operation` remain authoritative
- Templates remain readable and modular but structurally aligned
- Future tools (editors, exporters, linters) can perform static analysis on visual correctness

## Related
- ADR-001: Glyph Semantics
- ADR-004: Input Visualization Strategy
- ADR-005: Visual Authoring Tool Vision
- Planned: Meta-model for rendering layer + grammar cohesion