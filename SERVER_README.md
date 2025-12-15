# Kleis HTTP Server - API Reference

Complete REST API documentation for the Kleis mathematical renderer server.

---

## 🚀 Quick Start

```bash
# Start server
cargo run --bin server

# Server runs at http://localhost:3000
# Web UI available at http://localhost:3000
```

---

## 📡 API Endpoints

### 1. GET `/` - Web UI

Returns the HTML equation editor interface.

**Response:** HTML page

**Features:**
- LaTeX input with live editing
- MathJax preview
- 71 gallery examples (auto-loaded)
- Symbol palettes (5 categories, 62+ symbols)
- Template library (18 structures)

---

### 2. POST `/api/render` - Render Equation

Converts LaTeX input to rendered output.

#### Request

```json
{
  "latex": "\\frac{1}{2} \\sqrt{\\pi}",
  "format": "latex"
}
```

**Parameters:**
- `latex` (string, required): LaTeX equation to render
- `format` (string, required): Output format (`"latex"` or `"unicode"`)

#### Response - Success

```json
{
  "output": "\\frac{1}{2} \\sqrt{\\pi}",
  "format": "latex",
  "success": true,
  "error": null
}
```

#### Response - Error

```json
{
  "output": "",
  "format": "latex",
  "success": false,
  "error": "Parse error at position 5: ..."
}
```

#### Example (curl)

```bash
curl -X POST http://localhost:3000/api/render \
  -H "Content-Type: application/json" \
  -d '{
    "latex": "\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}",
    "format": "latex"
  }'
```

#### Example (JavaScript)

```javascript
const response = await fetch('http://localhost:3000/api/render', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    latex: '\\frac{1}{2}',
    format: 'latex'
  })
});

const data = await response.json();
console.log(data.output);
```

#### Example (Python)

```python
import requests

response = requests.post('http://localhost:3000/api/render', json={
    'latex': r'\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}',
    'format': 'latex'
})

data = response.json()
print(data['output'])
```

---

### 3. GET `/api/gallery` - Get Gallery Examples

Returns all 71 curated mathematical examples.

#### Response

```json
{
  "examples": [
    {
      "title": "Inner product ⟨u,v⟩",
      "latex": "\\langle u, v \\rangle"
    },
    {
      "title": "Einstein Field Equations (core)",
      "latex": "G_{\\mu\\nu} + \\Lambda g_{\\mu\\nu} = \\kappa T_{\\mu\\nu}"
    },
    {
      "title": "Schrödinger equation",
      "latex": "\\hat{H} |\\psi\\rangle = E |\\psi\\rangle"
    },
    ...
  ]
}
```

**Count:** 71 examples

**Categories:**
- Physics (Einstein, Maxwell, wave equations)
- Quantum mechanics (Schrödinger, Pauli matrices)
- Calculus (Euler-Lagrange, Hamilton-Jacobi)
- Number theory (Riemann zeta)
- Linear algebra (matrices, determinants)
- Set theory & logic
- Vector calculus
- Piecewise functions

#### Example (curl)

```bash
curl http://localhost:3000/api/gallery | jq '.examples | length'
# Output: 71
```

#### Example (JavaScript)

```javascript
const response = await fetch('http://localhost:3000/api/gallery');
const data = await response.json();

// Display gallery
data.examples.forEach(ex => {
  console.log(`${ex.title}: ${ex.latex}`);
});
```

---

### 4. GET `/api/operations` - List Operations

Returns metadata about available mathematical operations.

#### Response

```json
[
  {
    "name": "Fraction",
    "description": "Creates a fraction a/b",
    "example_latex": "\\frac{a}{b}"
  },
  {
    "name": "Square Root",
    "description": "Square root of x",
    "example_latex": "\\sqrt{x}"
  },
  {
    "name": "Integral",
    "description": "Definite integral",
    "example_latex": "\\int_{a}^{b} f(x) \\, dx"
  },
  {
    "name": "Sum",
    "description": "Summation with bounds",
    "example_latex": "\\sum_{i=1}^{n} i"
  },
  {
    "name": "Matrix",
    "description": "2x2 matrix",
    "example_latex": "\\begin{bmatrix}a&b\\\\c&d\\end{bmatrix}"
  }
]
```

**Note:** This is a sample list. Full renderer supports 56 operations.

---

### 5. GET `/health` - Health Check

Simple endpoint to verify server is running.

#### Response

```
OK
```

**Status Code:** 200

#### Example

```bash
curl http://localhost:3000/health
# Output: OK
```

---

## 🎨 Web UI Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Enter` (Mac: `Cmd+Enter`) | Render equation |
| Tab | Navigate between input and preview |

### Symbol Palettes

Click tabs to access different symbol categories:

1. **Greek** - α, β, γ, δ, ε, θ, λ, μ, ν, π, ρ, σ, τ, φ, ψ, ω, Γ, Δ, Θ, Λ, Σ, Φ, Ψ, Ω
2. **Operators** - ∑, ∏, ∫, ∂, ∇, √, ⟨, ⟩, †, ‡, ±, ×, ÷, ·, ∞
3. **Relations** - =, ≠, <, >, ≤, ≥, ≈, ≡, ∝, ∈, ⊂, ⊆, ∪, ∩, ∀, ∃, ⇒, ⇔
4. **Calculus** - ∫, ∬, ∭, ∮, ∂, ∇, lim, limsup, liminf
5. **Templates** - Fractions, matrices, cases, roots, subscripts, superscripts, etc.

### Templates

Common LaTeX structures with placeholders (`■`):

- Fraction: `\frac{■}{■}`
- Square root: `\sqrt{■}`
- Power: `{■}^{■}`
- Subscript: `{■}_{■}`
- 2x2 Matrix: `\begin{bmatrix}■&■\\■&■\end{bmatrix}`
- 3x3 Matrix: `\begin{bmatrix}■&■&■\\■&■&■\\■&■&■\end{bmatrix}`
- Integral: `\int_{■}^{■} ■ \, d■`
- Sum: `\sum_{■}^{■} ■`
- Cases: `\begin{cases}■&■\\■&■\end{cases}`

---

## 🔧 Configuration

### Port

Default port is `3000`. To change:

```rust
// In src/bin/server.rs, line ~67
let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
    .await
    .expect("Failed to bind");
```

### CORS

CORS is set to permissive mode for development. For production, configure in `server.rs`:

```rust
.layer(CorsLayer::new()
    .allow_origin("https://your-domain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
)
```

---

## 🏗️ Architecture

### Tech Stack
- **Framework:** Axum (Tokio-based async)
- **CORS:** tower-http
- **JSON:** serde/serde_json
- **Static files:** tower-http ServeDir

### Server Structure

```rust
Router::new()
    .route("/", get(index_handler))           // Serves static/index.html
    .route("/api/render", post(render_handler)) // Calls parser + renderer
    .route("/api/gallery", get(gallery_handler)) // Calls collect_samples_for_gallery()
    .route("/api/operations", get(operations_handler)) // Returns sample ops
    .route("/health", get(health_handler))     // Health check
    .nest_service("/static", ServeDir::new("static")) // Static assets
    .layer(CorsLayer::permissive())
```

### Request Flow

1. **Client** sends LaTeX string
2. **Parser** (`kleis::parser::parse_latex`) converts to Expression AST
3. **Renderer** (`kleis::render::render_expression`) converts to output format
4. **Server** returns JSON response

---

## 🚨 Error Handling

### Parser Errors

```json
{
  "success": false,
  "error": "Parse error at position 15: Expected '}'"
}
```

Common causes:
- Unmatched braces: `\frac{1}{2`
- Unknown command: `\unknowncommand`
- Invalid syntax: `\begin{matrix}` without `\end{matrix}`

### Renderer Errors

Renderer is generally robust. If parsing succeeds, rendering should not fail. Unknown operations fall back to object names.

---

## 📊 Performance

### Benchmarks (on 2025 MacBook)

- **Simple equation** (`\frac{1}{2}`): ~0.1ms parse + render
- **Complex matrix:** ~1-2ms
- **Gallery (71 examples):** ~50-100ms total

### Concurrency

Server uses Tokio async runtime. Can handle 1000+ concurrent requests. No shared mutable state.

---

## 🧪 Testing

### Test API Endpoints

```bash
# Health check
curl http://localhost:3000/health

# Render simple
curl -X POST http://localhost:3000/api/render \
  -H "Content-Type: application/json" \
  -d '{"latex": "E = mc^2", "format": "latex"}'

# Get gallery count
curl http://localhost:3000/api/gallery | jq '.examples | length'

# List operations
curl http://localhost:3000/api/operations | jq '.[0]'
```

---

## 📝 Development

### Hot Reload

The server reads `static/index.html` dynamically, so HTML changes are picked up without recompiling:

```bash
# Edit static/index.html
# Refresh browser - changes appear immediately
```

Rust code changes require rebuild:

```bash
# Kill server (Ctrl+C)
cargo run --bin server
```

### Adding New Endpoints

```rust
// In src/bin/server.rs
async fn my_handler() -> impl IntoResponse {
    Json(json!({"message": "Hello"}))
}

// Add to router
.route("/api/my-endpoint", get(my_handler))
```

---

## 🔗 Related Documentation

- **Main README:** Project overview
- **Parser Status:** `PARSER_TODO.md`
- **Renderer:** See `src/render.rs` - 56 operations
- **Gallery:** See `collect_samples_for_gallery()` - 71 examples

---

**Kleis Server** - REST API for mathematical notation rendering
