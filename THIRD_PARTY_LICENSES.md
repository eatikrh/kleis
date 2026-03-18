# Third-Party Licenses

This document lists the third-party dependencies used by Kleis and their licenses.

---

## Summary

Kleis uses only **permissive open-source licenses** (MIT, Apache-2.0, BSD). There are no copyleft (GPL) dependencies that would impose restrictions on Kleis's licensing.

| License Type | Count | Restriction Level |
|--------------|-------|-------------------|
| MIT | ~40 | Permissive ✅ |
| Apache-2.0 | ~18 | Permissive ✅ |
| MIT OR Apache-2.0 | ~20 | Permissive ✅ |
| BSD-3-Clause | ~3 | Permissive ✅ |
| BSD-2-Clause | ~1 | Permissive ✅ |

---

## External Tools (Not Bundled)

These tools are invoked by Kleis but not distributed with it:

| Tool | License | Usage | Notes |
|------|---------|-------|-------|
| **Typst** | Apache-2.0 | Equation/plot rendering | Compiled externally, called via `typst compile` |
| **Lilaq** | MIT | Plotting library | Typst package, imported at runtime |
| **Z3** | MIT | Theorem prover | Linked via z3-sys bindings |

---

## Rust Dependencies (Cargo.toml)

### Core Web Server

| Crate | License | Purpose |
|-------|---------|---------|
| `axum` | MIT | Web server framework |
| `tokio` | MIT | Async runtime |
| `tower` | MIT | Service abstraction |
| `tower-http` | MIT | HTTP middleware |

### Serialization

| Crate | License | Purpose |
|-------|---------|---------|
| `serde` | MIT OR Apache-2.0 | Serialization framework |
| `serde_json` | MIT OR Apache-2.0 | JSON support |
| `toml` | MIT OR Apache-2.0 | TOML config parsing |

### Typesetting (Typst Integration)

| Crate | License | Purpose |
|-------|---------|---------|
| `typst` | Apache-2.0 | Typst compiler |
| `typst-svg` | Apache-2.0 | SVG output |
| `typst-assets` | Apache-2.0 | Fonts |
| `comemo` | MIT OR Apache-2.0 | Memoization (Typst dependency) |

### CLI & REPL

| Crate | License | Purpose |
|-------|---------|---------|
| `clap` | MIT OR Apache-2.0 | Command-line parsing |
| `rustyline` | MIT | REPL line editing |

### Language Server Protocol

| Crate | License | Purpose |
|-------|---------|---------|
| `tower-lsp` | MIT | LSP server framework |
| `lsp-types` | MIT | LSP type definitions |
| `ropey` | MIT | Text rope for document handling |
| `dashmap` | MIT | Concurrent hashmap |

### Theorem Prover

| Crate | License | Purpose |
|-------|---------|---------|
| `z3` (vendored) | MIT | Z3 Rust bindings |
| Z3 (native library) | MIT | SMT solver |

### Utilities

| Crate | License | Purpose |
|-------|---------|---------|
| `regex` | MIT OR Apache-2.0 | Regular expressions |
| `time` | MIT OR Apache-2.0 | Date/time handling |
| `roxmltree` | MIT OR Apache-2.0 | XML parsing (SVG) |
| `uuid` | MIT OR Apache-2.0 | UUID generation |

### Numerical (Optional Feature)

| Crate | License | Purpose |
|-------|---------|---------|
| `ndarray` | MIT OR Apache-2.0 | N-dimensional arrays |
| `ndarray-linalg` | MIT OR Apache-2.0 | Linear algebra |
| `nalgebra` | Apache-2.0 | Linear algebra (pure Rust) |
| `lapack` | MIT OR Apache-2.0 | LAPACK bindings |
| `blas-src` | MIT OR Apache-2.0 | BLAS backend selection |
| `lapack-src` | MIT OR Apache-2.0 | LAPACK backend selection |

---

## Python Dependencies (kleis-notebook)

**Production Dependencies:**

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| `jupyter-client` | >=6.0 | BSD-3-Clause | Jupyter messaging protocol |
| `ipykernel` | >=6.0 | BSD-3-Clause | Jupyter kernel base class |

**Dev Dependencies:**

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| `pytest` | >=7.0 | MIT | Testing framework |
| `black` | >=23.0 | MIT | Code formatter |
| `mypy` | >=1.0 | MIT | Static type checker |

**Build System:**

| Package | License | Purpose |
|---------|---------|---------|
| `setuptools` | MIT | Build backend |
| `wheel` | MIT | Wheel package format |

---

## Node.js Dependencies

### PatternFly Editor (patternfly-editor/)

**Production Dependencies:**

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| `@patternfly/patternfly` | ^6.4.0 | MIT | PatternFly CSS framework |
| `@patternfly/react-core` | ^6.4.0 | MIT | PatternFly React components |
| `@patternfly/react-icons` | ^6.4.0 | MIT | PatternFly icons |
| `react` | ^19.2.0 | MIT | UI framework |
| `react-dom` | ^19.2.0 | MIT | React DOM rendering |

**Dev Dependencies:**

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| `@eslint/js` | ^9.39.1 | MIT | ESLint JavaScript config |
| `@types/node` | ^24.10.1 | MIT | Node.js type definitions |
| `@types/react` | ^19.2.5 | MIT | React type definitions |
| `@types/react-dom` | ^19.2.3 | MIT | React DOM type definitions |
| `@vitejs/plugin-react` | ^4.7.0 | MIT | Vite React plugin |
| `eslint` | ^9.39.1 | MIT | Linter |
| `eslint-plugin-react-hooks` | ^7.0.1 | MIT | React hooks linting |
| `eslint-plugin-react-refresh` | ^0.4.24 | MIT | React refresh linting |
| `globals` | ^16.5.0 | MIT | Global variable definitions |
| `typescript` | ~5.9.3 | Apache-2.0 | TypeScript compiler |
| `typescript-eslint` | ^8.46.4 | BSD-2-Clause | TypeScript ESLint plugin |
| `vite` | ^5.4.21 | MIT | Build tool |

### VS Code Extension (vscode-kleis/)

**Production Dependencies:**

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| `vscode-languageclient` | ^9.0.1 | MIT | LSP client library |

**Dev Dependencies:**

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| `@types/node` | ^20.0.0 | MIT | Node.js type definitions |
| `@types/vscode` | ^1.75.0 | MIT | VS Code API types |
| `@vscode/vsce` | ^3.7.1 | MIT | VS Code extension packaging |
| `typescript` | ^5.0.0 | Apache-2.0 | TypeScript compiler |
| `vscode-oniguruma` | ^1.6.2 | MIT | TextMate grammar engine |
| `vscode-textmate` | ^5.3.0 | MIT | TextMate grammar support |

### Equation Editor (static/index.html)

| Library | License | Purpose |
|---------|---------|---------|
| MathJax | Apache-2.0 | Math rendering fallback (CDN) |

---

## License Texts

### MIT License

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Apache License 2.0

See: https://www.apache.org/licenses/LICENSE-2.0

### BSD 3-Clause License

See: https://opensource.org/licenses/BSD-3-Clause

---

## Kleis License

Kleis is licensed under the **MIT License** with the following copyright:

```
Copyright (c) 2024-2026 Engin Atik
```

**Attribution Requirement**: Any derivative work or product based on Kleis must include attribution to **Engin Atik** as the original author. This is required by the MIT License's copyright notice preservation clause.

---

## Compliance

### For Kleis Users (Creating Derivative Works)

To create derivative works from Kleis, you must:

1. **Include the copyright notice**: "Copyright (c) 2024-2026 Engin Atik"
2. **Include the MIT license text** in your distribution
3. **Attribute Engin Atik** as the original author in your documentation or about page

### For Kleis (Using Third-Party Dependencies)

Kleis complies with its dependencies' licenses by:

1. **MIT/BSD**: Including copyright notices and license texts in this document
2. **Apache-2.0**: Documenting the use of Typst and TypeScript
3. **All**: Not using project names to endorse Kleis without permission

---

*Last updated: January 1, 2026*

