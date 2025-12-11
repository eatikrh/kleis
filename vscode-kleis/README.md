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
# NOTE: packaging requires Node.js v20+ (use nvm to switch if necessary)
npm run lint-grammar

# install the modern packager locally and package via npx (recommended)
npm install --no-save @vscode/vsce
npx @vscode/vsce package

# install the produced VSIX
code --install-extension kleis-0.1.0.vsix
```

### Packaging & local install (developer notes)

Requirements
- Node.js v20+ (use `nvm install 20 && nvm use 20`)

Build VSIX (recommended)
```bash
cd vscode-kleis
# validate grammar first
npm run lint-grammar

# install packager locally and package
npm install --no-save @vscode/vsce
npx @vscode/vsce package
# produces `kleis-<version>.vsix`
```

Make `code` CLI available (macOS)
- In VS Code: Command Palette → "Shell Command: Install 'code' command in PATH"

Install the VSIX
```bash
code --install-extension kleis-0.1.0.vsix
# verify
code --list-extensions | grep kleis
```

Quick dev test (no packaging)
- Open the extension folder in VS Code and press `F5` to launch an Extension Development Host.
- To inspect the scopes applied to a token: Command Palette → "Developer: Inspect Editor Tokens and Scopes" and click a token.

CI note
- The repository includes a GitHub Actions workflow that runs `npm run lint-grammar` on pushes and pull requests to `main` (see `.github/workflows/lint-grammar.yml`).

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

