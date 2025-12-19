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

## References

- Typst Universe: https://typst.app/universe
- Rivet: Bitfields and registers
- CeTZ: TikZ-like diagrams
- Fletcher: Commutative diagrams
- typst-syntax crate: Build AST programmatically

