# Change Log

All notable changes to the Kleis VS Code extension will be documented in this file.

## [0.4.13] - 2025-12-29

### Added
- **Grammar v0.95 support**: Big operator syntax
  - `Σ(from, to, body)` — Summation
  - `Π(from, to, body)` — Product
  - `∫(lower, upper, body, var)` — Integral
  - `lim(var, target, body)` — Limit
  - Parser reorders arguments for equation-editor compatibility
  - Round-trip support: Kleis renderer outputs parseable syntax

### Documentation
- Added `kleis_grammar_v095.ebnf`, `kleis_grammar_v095.md`

## [0.4.12] - 2025-12-26

### Added
- **Grammar v0.94 support**: N-ary product types
  - `A × B × C × D` (any number of factors)
  - Right-associative: `A × B × C = A × (B × C)`
  - Enables clean multi-argument type signatures:
    ```kleis
    operation metric_probe : FieldR4 × Point × Point → ℝ
    operation mass_at : GreenKernel × Flow × Event → ℝ
    ```

### Documentation
- Added `kleis_grammar_v093.ebnf`, `kleis_grammar_v093.md`
- Added `kleis_grammar_v094.ebnf`, `kleis_grammar_v094.md`

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

