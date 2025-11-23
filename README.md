# Kleis

**Kleis** is a symbolic language and computational framework for formalizing **Projected Ontology Theory (POT)** and **Hilbert Ontology (HONT)**.

It blends metaphysical clarity with mathematical structure to express concepts like modal flow, projection, residue, bifurcation, and emergent phenomena. Kleis aims to unify ontological reasoning, physics-inspired structure, and symbolic computation under a single formal umbrella.

---

## 🚀 Quick Start

### Run the Web Equation Editor

```bash
cargo run --bin server
```

Then open **http://localhost:3000** in your browser for the web editor which now includes:
- Bidirectional text ↔ structural editing
- Live MathJax preview
- Role-aware semantic overlays with keyboard navigation (Tab through markers, Enter to edit)
- **91 gallery examples** (click to load)
- Symbol palettes (Greek, operators, calculus, etc.)
- Template library (fractions, matrices, integrals, tensors, bra-ket, etc.)

### Run Tests

```bash
cargo test
```

**Current status:** 300+ tests passing across renderer, parser, semantic layout, and golden suites.

### Generate PDF Gallery

```bash
cargo run --bin gallery
```

Creates `tmp_gallery.pdf` with all 91 rendered examples.

---

## 🧠 What is Kleis?

Kleis serves three main purposes:

1. **Symbolic Language** - For expressing modal ontological structures (POT/HONT)
2. **Mathematical Renderer** - Converts expression trees to LaTeX/Unicode with **100+ operations** and 91 gallery examples
3. **LaTeX Parser + Template Inference** - Parses LaTeX back into structured ASTs (~80% coverage) and infers higher-level templates (integrals, quantifiers, statistics, tensor ops, etc.)

### Inspiration

- Functional symbolic systems like **LISP** and **REDUCE**
- Formal language design principles
- The need to represent non-temporal structure, projection mechanics, and modal coherence

---

## 📊 Current Capabilities

### Renderer (100+ Operations)

**Coverage:** ~98% of standard mathematical notation plus POT/HONT-specific constructs

#### Core Operations
- **Arithmetic:** equals, plus, minus, times, divide, power
- **Comparisons:** less_than, greater_than, leq, geq, not_equal, approx, proportional
- **Logic:** implies, forall, exists

#### Calculus
- **Derivatives:** d_dt, d_part, d2_part, partial
- **Integrals:** int_bounds, double_int, triple_int
- **Sums/Products:** sum_bounds, sum_index, prod_bounds, prod_index
- **Limits:** limit, limsup, liminf

#### Linear Algebra
- **Matrices:** matrix2x2, matrix3x3, pmatrix2x2, pmatrix3x3, vmatrix2x2, vmatrix3x3
- **Operations:** transpose, det, trace, inverse
- **Vectors:** vector_arrow, vector_bold, norm, abs, inner, outer_product
- **Products:** dot, cross

#### Quantum Mechanics
- **Bra-ket:** ket, bra
- **Operators:** commutator, anticommutator, hat

#### Tensor/Relativity
- **Indices:** sub, sup, index_mixed, index_pair
- **Operators:** nabla, nabla_sub, box
- **Symbols:** gamma, riemann

#### Functions
- **Trig:** sin, cos, tan, sec, csc, cot, arcsin, arccos, arctan
- **Hyperbolic:** sinh, cosh
- **Logarithmic:** ln, log
- **Special:** exp, sqrt, nth_root, factorial
- **Named:** H, S, V, F, C, D, zeta, Gamma

#### Set Theory
- **Relations:** in_set, subseteq, union, intersection
- **Number sets:** mathbb (ℝ, ℂ, ℕ, etc.)

#### Vector Calculus
- **Operators:** grad, div, curl, laplacian
- **Integrals:** surface_integral

#### Miscellaneous
- **Control:** min_over
- **Complex:** conjugate, re, im
- **Functions:** floor, ceiling, binomial
- **Piecewise:** cases2, cases3
- **Modular:** congruent_mod
- **Statistics:** variance, covariance

### LaTeX Parser & Template Inference

**Coverage:** ~80% of common LaTeX math. Template inference upgrades flat parses into semantic operations (double/triple integrals, logical implications, quantifiers, modular arithmetic, statistics, tensor traces, curls/divergence, etc.).

#### ✅ Working
- Fractions, square roots, nth roots
- Subscripts, superscripts, mixed indices
- Greek letters and symbols
- Binary operators (+, -, *, /)
- Operator precedence
- Matrices (bmatrix, pmatrix, vmatrix)
- Bra-ket notation
- Commutators and anticommutators
- Function calls with multiple arguments
- Implicit multiplication (2m → 2*m)
- Unary operators (-x, +x)
- Trig, log, and special functions
- Number sets (\mathbb{R})

#### ❌ Not Yet Supported
- Complex piecewise blocks mixing text and math (partially supported)
- Advanced delimiter matching for nested environments
- General sequence literals (`1,2,3,\ldots,n`)

See `PARSER_TODO.md` for detailed status.

---

## 📁 Project Structure

```
kleis/
├── src/
│   ├── lib.rs              # Library exports
│   ├── render.rs           # Renderer (56 operations, 71 gallery examples)
│   ├── parser.rs           # LaTeX parser
│   ├── main.rs             # CLI entry point
│   └── bin/
│       ├── server.rs       # HTTP server + web UI
│       ├── gallery.rs      # PDF gallery generator
│       ├── test_parser.rs  # Parser test utility
│       └── check_parser.rs # Parser benchmark
├── static/
│   └── index.html          # Web equation editor UI
├── tests/
│   └── golden/             # Golden test suite
├── docs/
│   ├── adr-*.md           # Architecture Decision Records
│   ├── syntax.md          # Language syntax
│   ├── POT.md             # Projected Ontology Theory
│   └── HONT.md            # Hilbert Ontology
├── examples/               # Example equations
└── README.md
```

---

## 🌐 HTTP Server & API

### Start Server

```bash
cargo run --bin server
```

Server runs at **http://localhost:3000**

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web UI |
| `/api/render` | POST | Render LaTeX equation (returns LaTeX/Unicode/HTML) |
| `/api/render_ast` | POST | Render an AST directly (LaTeX/Unicode/HTML) |
| `/api/render_typst` | POST | Render an AST via Typst (SVG + semantic boxes) |
| `/api/gallery` | GET | Get all 91 examples |
| `/api/operations` | GET | List operations |
| `/health` | GET | Health check |

#### Example: Render Equation

```bash
curl -X POST http://localhost:3000/api/render \
  -H "Content-Type: application/json" \
  -d '{"latex": "\\frac{1}{2}", "format": "latex"}'
```

Response:
```json
{
  "output": "\\frac{1}{2}",
  "format": "latex",
  "success": true,
  "error": null
}
```

See `SERVER_README.md` for detailed API documentation.

---

## 🧪 Testing

### Run All Tests

```bash
cargo test
```

### Test Categories

- **Renderer/unit tests:** 200+ covering operations, formatting, semantic helpers
- **Parser/tests:** LaTeX parsing, template inference patterns
- **Golden tests:** End-to-end validation (SVG/Typst vs MathJax)
- **Semantic layout tests:** Verifies bounding box extraction and overlay roles

### Parser Testing

```bash
# Test specific LaTeX
cargo run --bin test_parser '\frac{1}{2}'

# Benchmark parser
cargo run --bin check_parser
```

---

## 📚 Gallery Examples

The renderer includes 71 professionally curated examples:

- **Physics:** Einstein Field Equations, Maxwell equations, wave equations
- **Quantum Mechanics:** Schrödinger equation, Pauli matrices, bra-ket notation
- **Calculus:** Euler-Lagrange, Hamilton-Jacobi-Bellman, multiple integrals
- **Number Theory:** Riemann zeta function (3 forms)
- **Linear Algebra:** Matrices, determinants, traces
- **Set Theory:** Quantifiers, subset relations
- **Vector Calculus:** Divergence, curl, Laplacian
- **Piecewise Functions:** Sign function, absolute value

Access them via:
1. Web UI gallery (auto-loaded)
2. API: `GET /api/gallery`
3. PDF: `cargo run --bin gallery`

---

## 🔬 Formal Foundations

### Philosophy

**Simplification is Cognitive Optimization**

Simplification, like typesetting, is fundamentally about cognition. It produces forms that are easier for humans to understand without changing mathematical correctness. In Kleis, simplification is a separate, optional layer distinct from evaluation.

### Expression Evaluation

Expression evaluation in Kleis is **minimal, meaning-preserving, and necessary**:

- Controlled, type-aware unfolding of symbolic structures
- Context-aware transformations with symbol tables
- Deferred evaluation for unevaluated forms
- Explicit typing (scalar, vector, operator, tensor)
- Proof-backed transformations where needed

### Challenges

| Challenge | Description |
|-----------|-------------|
| **Symbolic algebra** | Manipulate structures, preserve relationships |
| **Type resolution** | Scalars, vectors, matrices, tensors, operators |
| **Context awareness** | Variables have meaning based on definition layers |
| **Non-commutative ops** | AB ≠ BA for operators and matrices |
| **Side conditions** | Orientation, domain constraints |
| **Lazy vs eager** | When to evaluate vs preserve |

---

## 🎯 Future Vision

### Vision 1: LLM Integration
Instead of natural language responses, LLMs will output formal Kleis DSL. Reasoning becomes structured, provable formal chains—no hallucination possible.

### Vision 2: Visual Authoring
A visual editor for defining new mathematical structures:
- Custom operations with visual glyphs
- Template assignment (LaTeX, Unicode)
- Type signatures and semantic bindings
- Package distribution (.kleis modules)
- Live preview and validation

See `docs/adr-005-visual-authoring.md` and `docs/kleis_vision_executable_math.md` for details.

### Vision 3: Executable Mathematics
Mathematicians define new algebras with notation, glyphs, and laws. Kleis understands, applies, renders, and shares them. Notation becomes executable; algebra becomes live.

---

## 📖 Documentation

- **Architecture Decisions:** `docs/adr-*.md`
- **Syntax Reference:** `docs/syntax.md`
- **Server API:** `SERVER_README.md`
- **Parser Status:** `PARSER_TODO.md`
- **Ontology Theories:**
  - Projected Ontology Theory: `docs/POT.md`
  - Hilbert Ontology: `docs/HONT.md`

---

## 🛠️ Development

### Add New Operation

1. Add to `render.rs`:
   - Create helper function
   - Add templates to `build_default_context()`
   - Add gallery example to `collect_samples_for_gallery()`
2. Write tests in `render.rs`
3. Add golden test in `tests/golden/`
4. Update this README

### Add Parser Support

1. Add to `parser.rs`:
   - Extend `parse_latex_command()` match statement
   - Add parsing logic
2. Write tests
3. Update `PARSER_TODO.md`

---

## 📝 License

See `LICENSE` file for details.

---

## 🙏 Acknowledgments

Built with Rust 🦀 using:
- **axum** - HTTP server
- **serde** - JSON serialization
- **tower-http** - CORS and static file serving

Mathematical notation powered by **MathJax** in the web UI.

---

**Kleis** - Where formal structure meets executable mathematics.
