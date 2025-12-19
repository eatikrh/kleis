# Typst ADT Rendering Vision

*Captured: December 19, 2024*
*Source: Gemini conversation about Typst integration*

## The Opportunity

Kleis already has:
- Typst rendering (`src/typst_renderer.rs`)
- Algebraic data types (`data BitVec = ...`)
- Complex mathematical structures

Typst can render these **beautifully** using:
- Built-in drawing primitives
- Community packages (Rivet, CeTZ, Fletcher)
- Recursive "dispatch on type" rendering

## Key Packages

| Package | Best For |
|---------|----------|
| **Rivet** | Bitfields and Registers - perfect for `BitVec(n)` |
| **CeTZ** | Tree structures and Pointers - AST visualization |
| **Fletcher** | Graphs and Flow - state machines, proofs |
| **Plotst** | Data visualizations - charts, plots |

## Architecture Options

### Option 1: Data-Driven (JSON)

```rust
// Kleis serializes ADT to JSON
let json = serde_json::to_string(&my_adt)?;
fs::write("data.json", json)?;
```

```typst
// Typst consumes and renders
#let data = json("data.json")
#render_adt(data)
```

### Option 2: Code Generation (Native Typst)

```rust
trait TypstRepresentable {
    fn to_typst(&self) -> String;
}

impl TypstRepresentable for BitVector {
    fn to_typst(&self) -> String {
        format!(
            "#bitfield(bits: {}, label: [{}])\n", 
            self.bits.len(), 
            self.label
        )
    }
}
```

### Option 3: In-Memory (typst crate)

Use the `typst` Rust crate directly - no filesystem, compile in-memory.

## Example: Recursive ADT Renderer in Typst

```typst
#import "@preview/rivet:0.1.0": *

#let render_adt(item) = {
  if item.type == "BitVector" {
    bitfield(
      bits: item.data.value.len(),
      field(0, item.data.value.len(), fill: blue.lighten(90%))[#item.data.name]
    )
    
  } else if item.type == "Sum" {
    block(stroke: 0.5pt + gray, inset: 10pt, radius: 4pt)[
      #set align(center)
      *Sum Type* \
      #grid(
        columns: item.data.len(),
        gutter: 1em,
        ..item.data.map(render_adt)
      )
    ]
  } else if item.type == "Product" {
    // Record/struct rendering
  }
}
```

## Kleis-Specific Use Cases

### 1. BitVec Visualization

```kleis
define x : BitVec(8) = bvones(8)
```

→ Rivet renders as:

```
┌───┬───┬───┬───┬───┬───┬───┬───┐
│ 1 │ 1 │ 1 │ 1 │ 1 │ 1 │ 1 │ 1 │
└───┴───┴───┴───┴───┴───┴───┴───┘
  7   6   5   4   3   2   1   0
```

### 2. AST Visualization

```kleis
define expr = EOperation("plus", [EVariable("x"), ENumber(1)])
```

→ CeTZ renders as tree:

```
       plus
      /    \
     x      1
```

### 3. Proof Trees

```kleis
axiom commutativity : ∀(a b : ℕ). a + b = b + a
```

→ Fletcher renders as derivation:

```
      ────────────────
      a + b = b + a   (comm)
```

### 4. Type Hierarchies

```
      ℂ
      ↑
      ℝ
      ↑
      ℚ
      ↑
      ℤ
      ↑
      ℕ
```

## Integration with Existing Typst Renderer

The current `src/typst_renderer.rs` handles:
- Mathematical expressions
- Tensor notation
- Greek symbols

Could be extended to:
- Dispatch on `data` type constructors
- Use Rivet for BitVec
- Use CeTZ for AST trees
- Use Fletcher for proof diagrams

## Implementation Path

1. Add `#[derive(Serialize)]` to Kleis AST types
2. Create `to_typst()` trait for Expression/Declaration
3. Add Rivet/CeTZ imports to Typst templates
4. Extend REPL with `:render` command for visual output

## Why This Matters

- **Documentation**: Auto-generate beautiful math papers
- **Debugging**: Visualize complex ADTs
- **Education**: Show proof trees and type derivations
- **Marketing**: Professional-looking output from Kleis

## Kleis Notebooks Vision

Typst as the rendering backend opens up **Kleis Notebooks** - a Jupyter-like experience for verified mathematics:

### What a Kleis Notebook Could Contain

```
┌─────────────────────────────────────────────────────────────────┐
│ [1] # Introduction                                              │
│     This notebook proves the fundamental theorem of...         │
├─────────────────────────────────────────────────────────────────┤
│ [2] define square(x : ℝ) : ℝ = x * x                           │
│     ✅ Type checked                                              │
├─────────────────────────────────────────────────────────────────┤
│ [3] axiom square_positive : ∀(x : ℝ). square(x) ≥ 0            │
│     ✅ Verified by Z3                                            │
├─────────────────────────────────────────────────────────────────┤
│ [4] [GRAPH: Plot of y = x²]                                     │
│     📈 Generated by CeTZ/Plotst                                  │
├─────────────────────────────────────────────────────────────────┤
│ [5] [IMAGE: experimental_data.png]                              │
│     Photo of laboratory results                                 │
├─────────────────────────────────────────────────────────────────┤
│ [6] [BITVEC DIAGRAM]                                            │
│     ┌───┬───┬───┬───┬───┬───┬───┬───┐                          │
│     │ 1 │ 0 │ 1 │ 1 │ 0 │ 0 │ 0 │ 1 │  ← Rivet rendered       │
│     └───┴───┴───┴───┴───┴───┴───┴───┘                          │
├─────────────────────────────────────────────────────────────────┤
│ [7] Therefore, by axiom square_positive, we conclude...        │
│                                                                 │
│     $∀x ∈ ℝ : x² ≥ 0$ ∎                                         │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                    [Export to PDF]
                              ↓
                    publication_ready.pdf
```

### Mixed Content Types

| Content | Source | Renderer |
|---------|--------|----------|
| Text/Markdown | User | Typst native |
| Math symbols | Kleis expressions | Typst math mode |
| Code blocks | Kleis source | Typst raw blocks |
| Graphs/Plots | CeTZ/Plotst | Typst packages |
| Images/Photos | External files | `image()` function |
| BitVec diagrams | Kleis BitVec | Rivet package |
| Proof trees | Kleis axioms | Fletcher package |
| Tables | Kleis data | Typst tables |

### Direct PDF Generation

Typst compiles directly to PDF - no LaTeX intermediary:

```bash
# From Kleis Notebook to PDF
kleis notebook paper.kleis.nb --to pdf

# Or in REPL
:export pdf my_proof.pdf
```

### Why This Matters

1. **Scientific Publishing**: Write proofs, include data, export paper
2. **Education**: Interactive textbooks with verified examples
3. **Documentation**: Mix code explanations with actual verified code
4. **Reports**: Business logic + proofs + charts in one document

### Technical Path

1. Define `.kleis.nb` notebook format (JSON or custom)
2. Notebook cells: `{type: "code"|"markdown"|"image", content: ...}`
3. Typst template that renders each cell type
4. `kleis notebook` command for interactive editing
5. `--to pdf` flag for export

## References

- Typst Universe: https://typst.app/universe
- Rivet: Bitfields and registers
- CeTZ: TikZ-like diagrams
- Fletcher: Commutative diagrams
- typst-syntax crate: Build AST programmatically
- Typst PDF export: Native, no LaTeX needed

