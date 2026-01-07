# Solver Capabilities Schema

All solver backends must follow this schema in their `capabilities.toml`.

The authoritative schema is defined by the `SolverCapabilities` struct in `capabilities.rs`.

## Required Sections

### `[solver]` - Backend Identification

```toml
[solver]
name = "Z3"                    # Display name
version = "4.12.0"             # Version string
type = "smt"                   # Type: "smt", "theorem_prover", etc.
description = "..."            # One-line description
```

### `[capabilities]` - Theories Array

```toml
[capabilities]
theories = [
    "arithmetic",
    "boolean",
    # ... supported theories
]
```

### `[capabilities.operations]` - Operation Specifications

```toml
[capabilities.operations]
plus = { arity = 2, theory = "Int/Real", native = true }
sin = { arity = 1, theory = "Real", native = false }  # Not natively supported
```

### `[capabilities.features]` - Feature Flags

```toml
[capabilities.features]
quantifiers = true|false           # ∀, ∃ support
uninterpreted_functions = true|false
recursive_functions = true|false
evaluation = true|false            # Concrete evaluation
simplification = true|false        # Expression simplification
proof_generation = true|false      # Proof object generation
```

### `[capabilities.performance]` - Limits

```toml
[capabilities.performance]
max_axioms = 10000                 # Reasonable axiom limit
timeout_ms = 5000                  # Default operation timeout
```

## Optional Backend-Specific Sections

Backends may add extra sections for their own configuration:

- **Z3**: Uses only the required sections above
- **Isabelle**: `[server]`, `[sessions]`, `[afp]`, `[translation]`

## Why This Matters

```rust
// The abstraction layer can query ANY backend uniformly:
let caps = load_capabilities()?;
if caps.capabilities.features.quantifiers {
    // Use quantifier support
}
if caps.has_operation("plus") {
    // Translate plus operation
}
```
