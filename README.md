# Kleis

**Kleis** is a **formal mathematical reasoning engine** with rigorous type checking and theorem proving:

- **Type system with structures** - Algebraic hierarchy (Magma → Semigroup → Monoid → Group → Ring → Field) with generic constraints, inheritance, and compositional algebra
- **Z3 theorem prover integration** - Verify mathematical axioms with SMT solving (5/5 dependency types rigorously proven)
- **Kleis language** - Self-hosted type definitions, operations in structures (ADR-016), parser with 60% grammar coverage
- **Structural equation editor** - WYSIWYG formula building with deterministic positioning and professional presentation
- **Documentation generator** - Auto-generate beautiful docs from .kleis files (Markdown + HTML with MathJax)
- **Universal verification vision** - Same type system for mathematics AND real-world domains (business rules, legal constraints, protocols)

Originally designed for **Projected Ontology Theory (POT)** and **Hilbert Ontology (HONT)**, Kleis has evolved into a general-purpose formal reasoning system that unifies mathematics, ontological structure, and real-world domain modeling under a single type-theoretic framework.

---

## ✨ Key Features

### 🎨 Structural Equation Editor
- **WYSIWYG editing** - Build formulas visually, see exactly what you get
- **Deterministic positioning** - UUID-based marker placement (92.7% of gallery examples use zero heuristics)
- **Content-aware layout** - Equations render perfectly without clipping, adaptive sizing
- **Primitive templates** - Build complex formulas from scratch (equality, +, −, ×, fractions, roots)
- **MathType-level UX** - 8 balanced palette tabs, intuitive navigation, keyboard shortcuts

### 🔬 Type System
- **Hindley-Milner type inference** - Standard syntactic unification with parametric polymorphism
- **Complete algebraic foundations** - Magma → Semigroup → Monoid → Group → Abelian Group → Ring → Field
- **Vector space axioms** - All 8 axioms formally expressed and verifiable
- **Polymorphic operations** - Same AST node, different semantics (scalar×scalar vs scalar×vector vs matrix×matrix)
- **Extensible architecture** - Add Groups, Categories, Fiber Bundles, Monads via plugin system
- **Exception handling** - Type-level errors (structure) vs value-level errors (division by zero)
- **Future work:** E-unification for algebraic equivalences (symbolic simplification)

### 🎓 Theorem Proving (Z3 Integration)
- **Axiom verification** - Z3 SMT solver integration for proving mathematical properties
- **Generic constraints** - Where clauses enable type-safe generic implementations
- **Structure inheritance** - Automatic axiom inheritance through extends keyword
- **Compositional algebra** - Nested structures (Ring = AbelianGroup + Monoid)
- **Mathematical notation** - Write axioms as mathematicians do: `∀(x y : R). x + y = y + x`

**Important distinction:**
- **Type inference** (automatic, fast ~1ms) - Checks syntactic structure (types match)
- **Axiom verification** (explicit, slower ~10ms) - Checks semantic properties (axioms hold)
- Type inference runs on every expression; axiom verification is an optional separate step
- Example: `Matrix(2,2,[a,b,c,d])` type checks instantly, but proving distributivity requires Z3

### 🧮 Evaluation Engine *(Designed)*
- **Context management** - Hierarchical scoping, bindings with types
- **Type-directed dispatch** - Routes operations based on inferred types
- **Multi-valued operations** - Handles ± and solution sets correctly
- **Symbolic computation** - Substitute, eval, typecheck, simplify

### 🤖 AI Verification
- **LLM output checking** - Paste AI-generated formulas, get instant type verification
- **Catches hallucinations** - Detects incompatible types, dimension mismatches, axiom violations
- **Trust-but-verify** - LLM generates → Kleis verifies → Human reviews
- **Universal applicability** - Math formulas, business rules, legal contracts, medical protocols

### 🌍 Universal Verification *(Vision)*
- Same type system for mathematics AND real-world domains
- Business: PurchaseOrder with inventory/credit axioms
- Legal: Contract with consent/consideration axioms
- Medical: Prescription with safety/interaction axioms
- Engineering: Design with stress/deflection axioms
- **Any structured domain with rules**

---

## 🚀 Quick Start

### Run the Web Equation Editor

**Recommended (handles Z3 setup automatically):**
```bash
./run_server.sh
```

**Or manually (requires Z3 environment variable):**
```bash
# macOS ARM (Apple Silicon)
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h

# macOS Intel
export Z3_SYS_Z3_HEADER=/usr/local/opt/z3/include/z3.h

# Linux
export Z3_SYS_Z3_HEADER=/usr/include/z3.h

cargo run --bin server
```

**Note:** Z3 integration is required for the type checker and axiom verification features.

Then open **http://localhost:3000** in your browser for the web editor:

**Structural Mode:**
- Build formulas from primitives (□ = □, □ + □, □ − □, □·□, fractions, roots)
- 8 balanced palette tabs (Basics, Fences, Accents, Calculus, Linear Algebra, Greek, Logic, Physics)
- UUID-based deterministic marker positioning (no spatial heuristics)
- Content-aware viewBox (proportional padding, handles negative coordinates)
- Keyboard navigation (arrows/Tab between markers, Enter to edit, Cmd+Z undo)
- Click-to-edit with visual feedback
- Auto-generated bounding boxes from Typst rendering

**Text Mode:**
- LaTeX input with live MathJax preview
- Bidirectional conversion (LaTeX ↔ structural AST)
- Parser with template inference (~80% coverage)

**Gallery:**
- **100+ templates** across all domains
- **91 curated examples** (physics, quantum, calculus, tensor ops)
- Click to load, instant structural editing

**Features:**
- Real-time type inference (hover to see types)
- Debug panel (AST visualization, bounding box info)
- Undo/redo with full history
- Zoom controls (Cmd +/−)
- Scrollable canvas for large equations

### Run Tests

```bash
# Set Z3 header path first
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h  # macOS ARM

# Run all library tests
cargo test --lib

# Run Z3 proof tests (requires axiom-verification feature)
cargo test --test z3_dependency_proof_tests --features axiom-verification
```

**Current status:** 421 tests passing (library tests) + 5 rigorous Z3 proof tests.

### Generate PDF Gallery

```bash
cargo run --bin gallery
```

Creates `tmp_gallery.pdf` with all 91 rendered examples.

---

## 🧠 What is Kleis?

Kleis is a **formal reasoning platform** that bridges visual editing, symbolic computation, and type-theoretic verification.

### Core Components

1. **Structural Equation Editor**
   - WYSIWYG formula builder (build quadratic formula from primitives)
   - Deterministic UUID-based marker positioning (100% accurate)
   - Content-aware rendering (no clipping, adaptive sizing)
   - 8 balanced palette tabs (Basics, Fences, Accents, Calculus, Linear Algebra, Greek, Logic, Physics)
   - Template composition (equality, +, −, ×, fractions, roots, matrices)

2. **Type System**
   - Complete algebraic hierarchy (Magma → Semigroup → Monoid → Group → Ring → Field)
   - Polymorphic dispatch (same operation, different semantics based on types)
   - Extensible architecture (add Groups, Categories, Fiber Bundles via plugins)
   - Exception handling (type-level errors vs runtime preconditions)
   - Vector space axioms, field axioms, all formally expressed

3. **Evaluation Engine** *(designed, not yet implemented)*
   - Context-aware symbolic computation
   - Type-directed evaluation with polymorphic dispatch
   - Substitute, eval, typecheck, simplify operations
   - Multi-valued results (±, solution sets)

4. **Renderer + Parser**
   - **100+ operations** (calculus, linear algebra, quantum mechanics, tensor ops)
   - **91 gallery examples** (physics, number theory, vector calculus)
   - LaTeX parser with template inference (~80% coverage)
   - Multiple output formats (LaTeX, Unicode, HTML, SVG with semantic bounding boxes)

5. **Universal Verification** *(vision)*
   - Same type system for mathematics AND real-world domains
   - Business rules (PurchaseOrder axioms), legal constraints (Contract axioms)
   - Medical protocols (Prescription safety), engineering specs (stress limits)
   - AI verification layer (catch LLM hallucinations via type checking)

### Key Innovation

**Axiomatic Types = Universal Verification**

The same type system that verifies:
```
"A vector space over field F satisfies 8 axioms"
```

Can also verify:
```
"A purchase order must have: total = sum(line_items), customer.credit_limit ≥ total"
```

**Mathematical correctness and business logic use the same verification engine.**

### Inspiration

- Symbolic systems: **LISP**, **REDUCE**, **Mathematica**
- Proof assistants: **Coq**, **Lean**, **Agda**
- Type theory: **Haskell**, **ML**, dependent types
- Structural editors: **MathType**, **Maple**
- Domain modeling: **Alloy**, **Z notation**, formal methods
- Ontological formalism: POT/HONT theories

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

## 🎯 Vision & Roadmap

### Phase 1: Mathematical Foundation *(Current)*
- ✅ Structural editor with deterministic positioning
- ✅ Content-aware rendering and adaptive layout
- ✅ Primitive templates for formula composition
- ✅ Type system design (algebraic hierarchy, extensibility)
- ✅ Evaluation syntax design (substitute, eval, multi-valued)
- ✅ Type inference implementation (Hindley-Milner)
- ✅ Z3 theorem prover integration (axiom verification)
- 🔄 Context management and eval() pipeline (in progress)

### Phase 2: Universal Verification Engine
- Implement axiomatic type definitions
- Add plugin architecture for domain-specific types
- Create domain libraries:
  - `std.business` - PurchaseOrder, Invoice, Contract
  - `std.legal` - Contract axioms, compliance rules
  - `std.medical` - Prescription safety, drug interactions
  - `std.engineering` - Design specs, tolerance checking
- Build verification API for AI integration

### Phase 3: AI Integration
- API endpoint: `/api/verify` for formula/document validation
- LLM integration: AI generates → Kleis verifies → AI corrects
- Trust-but-verify workflow for AI-generated mathematics
- Type error feedback with suggested fixes
- **Goal:** Make LLM-generated formulas reliable and trustworthy

### Phase 4: Visual Authoring
- Visual editor for defining new mathematical structures
- Custom operations with glyphs
- Type signatures and semantic bindings
- Package distribution (.kleis modules)
- Live validation and proof obligations

### Phase 5: Executable Mathematics
- Notebook interface with context management
- Template contexts (Physics, Quantum, Linear Algebra)
- Interactive type prompts and inference
- Proof assistant integration
- Collaborative mathematical reasoning

### Ultimate Vision

**Kleis becomes the universal verification layer for structured knowledge:**
- ✅ Mathematical formulas (type checking, axiom verification)
- ✅ Business processes (rule validation, constraint checking)
- ✅ Legal documents (compliance verification, axiom satisfaction)
- ✅ AI-generated content (structural correctness, type safety)
- ✅ Executable specifications (formal methods for any domain)

**In the AI era:** LLMs generate content quickly; Kleis verifies it rigorously. The combination makes AI-assisted formal reasoning actually reliable.

See `docs/adr-005-visual-authoring.md`, `docs/kleis_vision_executable_math.md`, and `docs/KLEIS_TYPE_UX.md` for details.

---

## 🎯 Use Cases

### 1. Mathematical Research
- Build complex formulas structurally without LaTeX expertise
- Type-check equations before using in papers
- Verify dimensional consistency across derivations
- Example: Build quadratic formula from primitives, verify types match

### 2. AI-Assisted Mathematics
- **LLM generates formula** → Kleis type-checks → Human reviews
- Catches AI hallucinations (incompatible Hilbert spaces, dimension mismatches)
- Example: LLM suggests projection kernel with wrong types → Kleis catches immediately
- Makes AI-generated mathematics trustworthy

### 3. Physics & Engineering
- Model field theories with proper type structure
- Verify gauge field tensors, stress calculations, circuit equations
- Type system prevents nonsense (adding scalar to vector field)
- Example: Gauss's law ∇·E = ρ/ε₀ type-checks field dimensions

### 4. Education
- Students build formulas visually, learn mathematical structure
- Verify homework solutions against axioms
- Understand why certain operations are invalid
- Example: See why 3⁻¹ ∉ ℤ (integers are Ring, not Field)

### 5. Business Rules Verification *(Future)*
- Model purchase orders, contracts, financial transactions as types
- Verify axioms (credit limits, inventory constraints, legal capacity)
- Same type system as mathematics, different domain
- Example: Check PO total matches line items, customer credit sufficient

### 6. Formal Methods
- Express system invariants as axioms
- Verify protocol correctness
- Model state machines with type-safe transitions
- Integration with formal verification tools

---

## 📖 Documentation

### Getting Started
- **[README](README.md)** - You are here
- **[Palette Guide](docs/guides/PALETTE_GUIDE.md)** - Using the structural editor
- **[Server API](SERVER_README.md)** - API reference
- **[Parser Status](PARSER_TODO.md)** - Parser implementation TODO

### Type System & Verification
- **[Type System Design](docs/type-system/KLEIS_TYPE_SYSTEM.md)** - Algebraic foundations, polymorphic dispatch
- **[Type System UX](docs/type-system/KLEIS_TYPE_UX.md)** - Context management, inference prompts
- **[Evaluation Syntax](docs/archive/type-system-roadmaps/KLEIS_EVALUATION_SYNTAX.md)** - Substitute, eval, multi-valued operations

### Vision & Future
- **[arXiv Integration](docs/vision/ARXIV_INTEGRATION_VISION.md)** - Academic publishing transformation
- **[Universal Verification](docs/vision/UNIVERSAL_QUALITY_GATES.md)** - Beyond mathematics
- **[Executable Mathematics](docs/vision/kleis_vision_executable_math.md)** - Long-term vision
- **[Visual Authoring](docs/adr/adr-005-visual-authoring.md)** - Custom operation design

### For Developers
- **[Parser Status](PARSER_TODO.md)** - Parser implementation TODO
- **[Test Guide](docs/guides/TEST_GUIDE.md)** - Testing procedures
- **[Grammar Specification](docs/grammar/)** - Formal grammar and specifications

### Architecture Decisions (22 ADRs)
- **[ADR-001](docs/adr/adr-001-scalar-multiply.md)** - Scalar multiply semantics
- **[ADR-002](docs/adr/adr-002-eval-vs-simplify.md)** - Evaluation vs simplification
- **[ADR-003](docs/adr/adr-003-self-hosting.md)** - Self-hosting strategy
- **[ADR-004](docs/adr/adr-004-input-visualization.md)** - Input visualization
- **[ADR-005](docs/adr/adr-005-visual-authoring.md)** - Visual authoring
- **[ADR-006](docs/adr/adr-006-template-grammar-duality.md)** - Template grammar duality
- **[ADR-007](docs/adr/adr-007-bootstrap-grammar.md)** - Bootstrap grammar
- **[ADR-008](docs/adr/adr-008-bootstrap-grammar-boundary.md)** - Grammar boundary
- **[ADR-009](docs/adr/adr-009-wysiwyg-structural-editor.md)** - WYSIWYG editor design
- **[ADR-010](docs/adr/adr-010-inline-editing.md)** - Inline editing
- **[ADR-011](docs/adr/adr-011-notebook-environment.md)** - Notebook environment
- **[ADR-012](docs/adr/adr-012-document-authoring.md)** - Document authoring
- **[ADR-013](docs/adr/adr-013-paper-scope-hierarchy.md)** - Paper scope hierarchy
- **[ADR-014](docs/adr/adr-014-hindley-milner-type-system.md)** - Hindley-Milner type system
- **[ADR-015](docs/adr/adr-015-text-as-source-of-truth.md)** - Text as source of truth
- **[ADR-016](docs/adr/adr-016-operations-in-structures.md)** - Operations in structures
- **[ADR-017](docs/adr/adr-017-vite-patternfly-frontend.md)** - Vite + PatternFly frontend
- **[ADR-018](docs/adr/adr-018-universal-formalism.md)** - Universal formalism
- **[ADR-019](docs/adr/adr-019-dimensional-type-checking.md)** - Dimensional type checking
- **[ADR-020](docs/adr/adr-020-metalanguage-for-type-theory.md)** - Metalanguage for type theory
- **[ADR-021](docs/adr/adr-021-algebraic-data-types.md)** - Algebraic data types
- **[ADR-022](docs/adr/adr-022-z3-integration-for-axiom-verification.md)** - Z3 integration for axiom verification

### Theory & Foundations
- **[Projected Ontology Theory](docs/theory/POT.md)** - POT formalism
- **[Hilbert Ontology](docs/theory/HONT.md)** - HONT formalism
- **[Language Syntax](docs/theory/syntax.md)** - Kleis syntax reference
- **[Grammar Specification](docs/grammar/kleis_grammar_v03.md)** - Formal grammar (v0.3)

### Historical Reference
- **[Archive](docs/archive/)** - Old analyses and implementation notes

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

## 📚 Documentation

### Main Guides
- **[Palette Guide](docs/guides/PALETTE_GUIDE.md)** - Using the structural editor
- **[Test Guide](docs/guides/TEST_GUIDE.md)** - Testing procedures
- **[Server API](SERVER_README.md)** - Server API documentation
- **[Parser Status](PARSER_TODO.md)** - Parser implementation TODO

### Architecture Decision Records
- See `docs/adr-*.md` for design decisions

---

## 🙏 Acknowledgments

Built with Rust 🦀 using:
- **axum** - HTTP server
- **serde** - JSON serialization
- **tower-http** - CORS and static file serving

Mathematical notation powered by **MathJax** in the web UI.

---

**Kleis** - Where formal structure meets executable mathematics.
