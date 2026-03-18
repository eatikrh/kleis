# ADR-008: Bootstrap Grammar Boundary

## Status
Accepted

## Context
Kleis is evolving toward a self-defining symbolic language. To support extensibility, modularity, and future self-hosting, we must clearly define what parts of the grammar are hardcoded in Rust and what parts will be defined in `.kleis` source files and interpreted at runtime.

This decision establishes the **minimum viable grammar** to be implemented in Rust. All constructs beyond this bootstrap set will be authored using Kleis itself, making Kleis flexible and extensible without recompilation.

## Decision

### Rust-Based Grammar Scope

The following constructs **will be implemented** in the core Rust parser:

- `object` declarations  
- `operation` declarations  
- `const` declarations  
- `template` definitions (glyphs for LaTeX, Unicode, etc.)  
- `define` statements  
- `assert` statements  
- `annotation` blocks  
- `expression` trees with core binary operators: `+`, `-`, `*`, `/`  
- Type signatures and type propagation rules  

### Excluded from Rust Grammar (Handled in Kleis DSL)

- `import` statements (handled by loader)  
- Higher-order structures: `category`, `functor`, `adjoint`, etc.  
- Proof steps and symbolic simplification rules  
- Meta-level language transformations or syntax extensions  
- Domain-specific symbolic constructs (e.g., relativity algebra, exterior calculus)

### Future Synchronization

The interpreter and visual engine will consume `.kleis` definitions for:
- New symbolic operations and types
- Associated templates
- Symbolic transformation rules
- Structural laws and patterns

## Consequences

- Simplifies the Rust parser and grammar infrastructure  
- Makes the core language stable and minimal  
- Empowers end users to extend the language without modifying core source  
- Lays the foundation for Kleis to become a living, self-hosting system

## Related
- ADR-003: Self-hosting Strategy
- ADR-005: Visual Authoring Vision
- ADR-007: Bootstrap Grammar and Kleis Self-Definition
- ADR-006: Template–Grammar Synchronization