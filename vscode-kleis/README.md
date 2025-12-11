# Kleis Language Support for VS Code

Syntax highlighting for the **Kleis** formal verification language.

## Features

- **Syntax Highlighting** for .kleis files
- **Keyword Recognition**: structure, implements, operation, axiom, data, match, verify
- **Type Highlighting**: ℝ, ℂ, ℤ, ℕ, Matrix, Vector, Scalar
- **Mathematical Operators**: ∀, ∃, λ, →, ⇒, ∈, ∇, ∂, ∫, √
- **Greek Letters**: α, β, γ, δ, and full Greek alphabet
- **Comment Support**: Line comments (//) and block comments (/* */)
- **Bracket Matching**: Automatic closing of {}, [], ()

## About Kleis

**Kleis** (κλείς - "Key" in Ancient Greek) is a universal verification platform for:
- Mathematical formulas with rigorous type checking
- Theorem proving with Z3 integration
- Business rules and domain verification
- Network protocols and security policies

**Learn more:** [kleis.io](https://kleis.io) | [GitHub](https://github.com/eatikrh/kleis)

## Example

```kleis
// Matrix structure with type parameters
structure Matrix(m: Nat, n: Nat, T) {
    operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
}

// Implementation for real matrices
implements Matrix(m, n, ℝ) {
    operation transpose = builtin_transpose
}

// Axiom example
axiom matrix_addition_commutative:
    ∀(A B : Matrix(m, n, ℝ)). A + B = B + A
```

## Installation

### From VS Code Marketplace
1. Open VS Code
2. Go to Extensions (Cmd+Shift+X / Ctrl+Shift+X)
3. Search for "Kleis"
4. Click Install

### Manual Installation
1. Download the .vsix file
2. Open VS Code
3. Extensions → ... menu → Install from VSIX

### From Source
```bash
cd vscode-kleis
npm install -g vsce
vsce package
code --install-extension kleis-0.1.0.vsix
```

## Requirements

None! Just install the extension and open any .kleis file.

## Known Issues

None currently. Please report issues at: https://github.com/eatikrh/kleis/issues

## Release Notes

### 0.1.0
- Initial release
- Complete syntax highlighting for Kleis language
- Support for mathematical symbols and Greek letters
- Bracket matching and comment toggling

## Contributing

Contributions welcome! See: https://github.com/eatikrh/kleis

## License

MIT License - Copyright (c) 2024 Engin Atik

---

**Kleis** - The Key to Universal Verification 🗝️

