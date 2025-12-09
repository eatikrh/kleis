# ADR-007: Minimal Bootstrap Grammar and Kleis Self-Definition

## Status
Accepted

## Context
As Kleis grows in symbolic complexity (supporting categories, functors, spinors, adjunctions, etc.), maintaining an ever-expanding hand-coded parser in Rust becomes unsustainable. At the same time, much of Kleis's structure is inherently self-descriptive: operations, types, and templates can be represented in Kleis itself.

This gives rise to the bootstrapping principle: a small **core grammar**, implemented in Rust, will be responsible for parsing a minimal subset of the language. All higher-level constructs — including new operations, templates, symbolic rules — will be defined in `.kleis` files and interpreted by Kleis itself.

## Decision

### 1. Bootstrap Grammar in Rust
A **minimal core grammar** will be implemented in Rust using a PEG parser (e.g., `pest` or `nom`). This grammar supports:
- Object declarations
- Operation declarations
- Constants
- Expressions with core binary operations
- Template definitions
- Law assertions

### 2. Self-Extensible Symbolic Layer
The rest of the language — including:
- Higher-order constructs
- Multi-parameter type families
- Law frameworks
- Categories, functors, adjunctions
- Complex visual templates

...will be declared using Kleis itself in `.kleis` files, parsed and loaded at runtime into the interpreter.

### 3. Bootstrap Runtime Loader
Kleis will include a bootstrap loader that:
- Parses core `.kleis` libraries at startup
- Extends the runtime with user-defined operations and templates
- Performs structural and visual validation

## Consequences
- Simplifies core Rust implementation
- Enables powerful extensibility without recompiling Kleis
- Supports user/community-defined symbolic extensions
- Allows truly self-hosted symbolic mathematics
- Makes the visual editor and renderers dynamically extensible at runtime

## Related
- ADR-003: Self-hosting Parser Strategy
- ADR-005: Visual Authoring Tool
- ADR-006: Template–Grammar Synchronization