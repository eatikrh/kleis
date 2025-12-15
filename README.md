# Kleis

**Kleis** is a **formal mathematical reasoning engine** with rigorous type checking and theorem proving.

| Feature | Description |
|---------|-------------|
| **Type System** | Algebraic hierarchy (Magma → Group → Ring → Field) with Hindley-Milner inference |
| **Z3 Integration** | Verify mathematical axioms with SMT solving |
| **Kleis Language** | Self-hosted type definitions, ~65% grammar coverage |
| **REPL** | Interactive theorem proving with `:verify`, `:load`, `:export` |
| **Equation Editor** | WYSIWYG formula building with deterministic positioning |
| **Doc Generator** | Auto-generate docs from .kleis files (Markdown + HTML + MathJax) |

---

## 🚀 Quick Start

### Run the Web Editor

```bash
./run_server.sh
# Then open http://localhost:3000
```

Or manually with Z3:
```bash
# macOS ARM
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
# macOS Intel: /usr/local/opt/z3/include/z3.h
# Linux: /usr/include/z3.h

cargo run --bin server
```

### Run the REPL

```bash
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
cargo run --bin repl
```

```
🧮 Kleis REPL v0.1.0
λ> :verify ∀(x : R, y : R). x + y = y + x
✅ Valid

λ> :load examples/protocols/ip_router.kleis
✅ Loaded: 14 functions

λ> :verify is_loopback(127, 0, 0, 1) = 1
✅ Valid
```

**Key commands:** `:help`, `:syntax`, `:examples`, `:symbols`, `:verify`, `:load`, `:export`

### Run Tests

```bash
cargo test           # All tests (495 passing)
cargo test --lib     # Library tests only
```

---

## 🎨 Structural Equation Editor

- **WYSIWYG editing** - Build formulas visually from primitives
- **Deterministic positioning** - UUID-based markers (92.7% zero heuristics)
- **8 palette tabs** - Basics, Fences, Accents, Calculus, Linear Algebra, Greek, Logic, Physics
- **100+ templates** - Fractions, roots, matrices, integrals, summations
- **Keyboard navigation** - Arrows/Tab between markers, Enter to edit, Cmd+Z undo

---

## 🔬 Type System

- **Hindley-Milner inference** with parametric polymorphism
- **Complete algebraic foundations** - Magma → Semigroup → Monoid → Group → Ring → Field
- **Vector space axioms** - All 8 axioms formally expressed
- **Polymorphic dispatch** - Same operator, semantics based on types
- **Extensible** - Add Groups, Categories, Fiber Bundles via plugin system

**Important distinction:**
- **Type inference** (automatic, ~1ms) - Checks structure
- **Axiom verification** (explicit, ~10ms) - Checks properties via Z3

---

## 🎓 Theorem Proving

```kleis
structure Group(G) extends Monoid(G) {
    operation inv : G → G
    
    axiom left_inverse:
        ∀(x : G). inv(x) * x = e
}
```

Z3 verifies axioms automatically:
```
λ> :verify ∀(a : R, b : R). (a + b) * (a - b) = a*a - b*b
✅ Valid

λ> :verify ∀(p : Bool, q : Bool). ¬(p ∧ q) = (¬p ∨ ¬q)
✅ Valid   (De Morgan's Law)
```

---

## 📊 Capabilities

### Renderer (100+ Operations)

| Category | Operations |
|----------|------------|
| **Calculus** | d_dt, d_part, int_bounds, sum_bounds, limit |
| **Linear Algebra** | matrix, transpose, det, trace, dot, cross |
| **Quantum** | ket, bra, commutator, anticommutator |
| **Tensor** | sub, sup, index_mixed, nabla, gamma, riemann |
| **Functions** | sin, cos, exp, ln, sqrt, factorial |
| **Logic** | implies, forall, exists, in_set |

### Parser (~80% LaTeX coverage)

✅ Fractions, roots, subscripts, superscripts, Greek letters, matrices, bra-ket, operators  
❌ Complex piecewise blocks, advanced delimiters

See `PARSER_TODO.md` for details.

---

## 📁 Project Structure

```
kleis/
├── src/
│   ├── render.rs       # Renderer (100+ operations)
│   ├── parser.rs       # LaTeX parser
│   └── bin/
│       ├── server.rs   # HTTP server + web UI
│       ├── repl.rs     # Interactive REPL
│       └── gallery.rs  # PDF gallery generator
├── static/index.html   # Web equation editor
├── stdlib/             # Standard library (.kleis files)
├── examples/           # Example .kleis files
│   ├── authorization/  # Zanzibar, OAuth2
│   └── protocols/      # IP router
├── docs/               # Documentation
│   ├── adr/            # 22 Architecture Decision Records
│   ├── grammar/        # Formal grammar (v03-v07)
│   └── type-system/    # Type system docs
└── tests/              # Test suite
```

---

## 🌐 HTTP API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web UI |
| `/api/render` | POST | Render LaTeX → output |
| `/api/render_ast` | POST | Render AST directly |
| `/api/render_typst` | POST | Render via Typst (SVG) |
| `/api/gallery` | GET | All 91 examples |
| `/health` | GET | Health check |

See `SERVER_README.md` for full API docs.

---

## 🎯 Vision

**Same type system for mathematics AND real-world domains:**

| Domain | Example |
|--------|---------|
| Mathematics | Vector space axioms, field properties |
| Business | PurchaseOrder with inventory/credit constraints |
| Legal | Contract with consent/consideration axioms |
| Medical | Prescription with safety/interaction checks |

**AI Integration:** LLM generates → Kleis verifies → Human reviews

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [docs/README.md](docs/README.md) | Documentation index |
| [docs/adr/](docs/adr/README.md) | 22 Architecture Decision Records |
| [docs/grammar/](docs/grammar/) | Formal grammar (v03-v07) |
| [docs/guides/PALETTE_GUIDE.md](docs/guides/PALETTE_GUIDE.md) | Equation editor guide |
| [SERVER_README.md](SERVER_README.md) | Server API reference |
| [PARSER_TODO.md](PARSER_TODO.md) | Parser status |

---

## 🛠️ Development

### Add New Operation

1. Add helper + template to `render.rs`
2. Add gallery example
3. Write tests
4. Update docs

### Quality Gates

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test
```

---

## 📝 License

See `LICENSE` file.

---

**Kleis** - Where formal structure meets executable mathematics. 🦀
