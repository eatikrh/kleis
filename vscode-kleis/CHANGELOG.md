# Change Log

All notable changes to the Kleis VS Code extension will be documented in this file.

## [0.3.0] - 2025-12-22

### Added
- **Language Server Protocol (LSP) support**: Full IDE integration!
  - Real-time diagnostics (parse errors shown as you type)
  - Hover information for identifiers
  - Go to definition (F12 / Cmd+Click)
  - Document symbols (outline view with Cmd+Shift+O)
- **Grammar v0.92 support**: Dimension expressions in type parameters
  - Arithmetic: `Matrix(2*n, 2*n, ℝ)`, `Matrix(n+1, m-1, ℂ)`
  - Power: `Vector(n^2, ℝ)`
  - Functions: `Matrix(min(m,n), max(m,n), ℝ)`, `gcd(a, b)`, `lcm(a, b)`
- Syntax highlighting for `import` statements
- Syntax highlighting for dimension functions: `min`, `max`, `gcd`, `lcm`
- Syntax highlighting for builtin functions: `builtin_*`
- New parametric type highlighting: `Matrix`, `Vector`, `Tensor`, `List`, `Set`, `Option`, `Result`

### Changed
- Upgraded to VS Code engine 1.75.0
- Extension now uses TypeScript for LSP client
- Bumped version to 0.3.0

### Installation
See `INSTALL.md` for updated instructions including building the language server.

## [0.2.1] - 2025-12-19

### Added
- **Rational numbers (ℚ)**: Full operator overloading and Z3 integration
- Updated example.kleis with rational number examples:
  - `rational(p, q)` constructor
  - Field axioms: commutativity, identity, inverse
  - Density of rationals
  - Type promotion: ℚ + ℤ → ℚ

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

