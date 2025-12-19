# Change Log

All notable changes to the Kleis VS Code extension will be documented in this file.

## [0.2.0] - 2025-12-19

### Added
- **Grammar v0.9 support**: Nested quantifiers and function types
- Syntax highlighting for nested quantifier expressions: `(x > 0) ∧ (∀(y : ℝ). y > 0)`
- Function type highlighting in quantifiers: `∀(f : ℝ → ℝ). f(0) = f(0)`
- Updated example.kleis with v0.9 features:
  - Epsilon-delta limit definitions
  - Topology continuity axioms
  - Metric space definitions
  - Sequence convergence
- New grammar documentation: `docs/grammar/kleis_grammar_v09.md`

### Impact
- Enables ~80% of Bourbaki-level mathematical expressiveness (up from ~20%)
- Full support for analysis, topology, and algebraic structures

## [0.1.0] - 2025-12-11

### Added
- Initial release of Kleis language support
- Syntax highlighting for .kleis files
- Support for all Kleis keywords: structure, implements, operation, axiom, data, match, verify, where, extends, over
- Mathematical type highlighting: ℝ, ℂ, ℤ, ℕ, ℚ, Matrix, Vector, Scalar
- Operator highlighting: ∀, ∃, λ, →, ⇒, ⊗, ∇, ∂, ∫, and more
- Greek letter support (α-ω)
- Comment support (line and block)
- Bracket matching and auto-closing pairs
- Language configuration for better editing experience

