# Semantic Bounding Box Strategy

**Date:** 2025-11-23  
**Goal:** Map layout bounding boxes to AST arguments using semantic information  
**Status:** Design Phase

---

## The Problem

Current spatial heuristic (group by Y-position) fails for:
- Horizontal layouts (inner product)
- Complex layouts (integrals with bounds)
- Nested structures (matrices)

We need to use **AST structure** to guide grouping.

---

## Approaches Considered

### ❌ Approach 1: Typst Metadata Markers

```typst
frac(#metadata((node: "0.0"))[a], #metadata((node: "0.1"))[b])
```

**Problem:** `#metadata()` doesn't work in math mode (Typst restriction)

### ❌ Approach 2: Invisible Text Markers

```typst
frac(#text(size: 0pt)[●0.0●] a, #text(size: 0pt)[●0.1●] b)
```

**Problem:** Optimized out of SVG, can't extract

### ❌ Approach 3: Typst Labels

```typst
frac(a #label("arg-0"), b #label("arg-1"))
```

**Problem:** Labels don't appear in SVG output

---

## ✅ Approach 4: Box Counting (RECOMMENDED)

**Key Insight:** We control the rendering! We know exactly what each argument will produce.

### Algorithm

**Step 1: Render Each Argument Separately**
```rust
for (i, arg) in args.iter().enumerate() {
    let arg_typst = render_expression(arg, ctx, RenderTarget::Typst);
    let arg_output = compile_to_svg(arg_typst);
    
    // Count how many layout boxes this argument produces
    arg_box_counts[i] = arg_output.layout_boxes.len();
}
```

**Step 2: Render Full Expression**
```rust
let full_typst = render_expression(full_ast, ctx, RenderTarget::Typst);
let full_output = compile_to_svg(full_typst);
// Extracts all layout boxes in rendering order
```

**Step 3: Assign Boxes to Arguments by Count**
```rust
let mut box_index = 0;
for (arg_idx, count) in arg_box_counts.iter().enumerate() {
    let arg_boxes = &full_output.layout_boxes[box_index..box_index + count];
    
    // Create bounding box for this argument
    let bbox = create_bounding_box(arg_boxes);
    bbox.arg_index = arg_idx;
    bbox.node_id = format!("0.{}", arg_idx);
    
    box_index += count;
}
```

### Why This Works

1. **Deterministic:** Same AST always produces same box count
2. **Order-preserving:** Typst renders args in order
3. **No Typst modifications:** Works with standard Typst
4. **Semantic:** Boxes are grouped by AST structure, not spatial heuristics

### Example: Fraction

```
AST: frac(a, b)

Render arg 0 (a) separately → 1 box
Render arg 1 (b) separately → 1 box

Render full frac(a, b) → 3 boxes total:
  Box 0: numerator "a"     → Assign to arg 0
  Box 1: fraction line     → Skip (not an argument)
  Box 2: denominator "b"   → Assign to arg 1
```

**Wait, this has a problem:** The fraction line is also a box! We'd miscount.

### Refinement: Filter Non-Argument Boxes

```rust
// Only count TEXT boxes (glyphs), not shapes (lines, decorations)
let text_boxes = layout_boxes.filter(|b| b.content_type == "text");
```

This filters out fraction lines, brackets, etc.

---

## ✅ Approach 5: Recursive Semantic Extraction (BEST)

**Key Insight:** We have the AST! We can traverse it and the layout tree in parallel.

### Algorithm

```rust
fn extract_semantic_boxes(
    ast: &Expression,
    layout_boxes: &[LayoutBox],
    box_index: &mut usize,
    node_id: &str
) -> Vec<ArgumentBoundingBox> {
    match ast {
        Expression::Operation { name, args } => {
            let mut result = Vec::new();
            
            for (i, arg) in args.iter().enumerate() {
                let child_id = format!("{}.{}", node_id, i);
                let start_index = *box_index;
                
                // Recursively extract boxes for this argument
                let child_boxes = extract_semantic_boxes(
                    arg, 
                    layout_boxes, 
                    box_index, 
                    &child_id
                );
                
                let end_index = *box_index;
                
                // Create bounding box for this argument
                let arg_boxes = &layout_boxes[start_index..end_index];
                let bbox = create_bounding_box_from_boxes(arg_boxes);
                
                result.push(ArgumentBoundingBox {
                    arg_index: i,
                    node_id: child_id,
                    x: bbox.x,
                    y: bbox.y,
                    width: bbox.width,
                    height: bbox.height,
                });
            }
            
            result
        }
        Expression::Const(_) | Expression::Object(_) => {
            // Leaf node - consumes one box
            *box_index += 1;
            vec![]
        }
        Expression::Placeholder { .. } => {
            // Placeholder - consumes one box
            *box_index += 1;
            vec![]
        }
    }
}
```

### How It Works

1. **Traverse AST and layout boxes in parallel**
2. **Each leaf (Const/Object) consumes one layout box**
3. **Each Operation groups its children's boxes**
4. **Perfect 1:1 mapping** between AST structure and layout boxes

### Example: frac(a + b, c)

```
AST:
  frac
    ├─ plus (arg 0)
    │   ├─ a (leaf)
    │   └─ b (leaf)
    └─ c (arg 1, leaf)

Layout boxes (in render order):
  [0] "a" text
  [1] "+" text  
  [2] "b" text
  [3] "/" shape (fraction line)
  [4] "c" text

Traversal:
  Enter frac
    Enter plus (arg 0)
      Visit a → consume box[0]
      Visit b → consume box[2] (skip box[1] which is "+")
      Create bbox for arg 0 from boxes [0,1,2]
    Visit c (arg 1) → consume box[4] (skip box[3] which is "/")
    Create bbox for arg 1 from box [4]
```

**Problem:** We still need to know which boxes to skip (operators like "+", "/")

---

## ✅ Approach 6: Two-Pass Rendering (PRACTICAL)

**Pass 1:** Render each argument in isolation
```rust
for arg in args {
    let isolated_svg = compile_to_svg(render(arg));
    // Extract boxes and their positions
    arg_box_info[i] = (box_count, bounding_box);
}
```

**Pass 2:** Render full expression
```rust
let full_svg = compile_to_svg(render(full_ast));
// Match boxes from Pass 1 to boxes in Pass 2 by position/size
```

**Matching:** Find boxes in full SVG that match the isolated boxes from Pass 1.

### Advantages
- No Typst modifications needed
- Works with any template
- Semantic grouping (each arg rendered separately)
- Can handle any layout complexity

### Implementation
```rust
pub fn compile_with_semantic_boxes(ast: &Expression) -> Result<CompiledOutput> {
    // Pass 1: Render each argument
    let arg_boxes = extract_arg_boxes_separately(ast)?;
    
    // Pass 2: Render full expression
    let full_output = compile_to_svg(render(ast))?;
    
    // Match and create semantic bounding boxes
    let semantic_boxes = match_boxes_to_args(arg_boxes, full_output.layout_boxes)?;
    
    Ok(CompiledOutput {
        svg: full_output.svg,
        argument_bounding_boxes: semantic_boxes,
        ...
    })
}
```

---

## Recommendation

**Implement Approach 6 (Two-Pass Rendering)**

**Pros:**
- Practical and implementable now
- No AST changes
- No Typst restrictions
- Semantic grouping guaranteed

**Cons:**
- Requires rendering each arg separately (performance cost)
- Matching logic needed (but straightforward)

**Estimated effort:** 1-2 days

**Next steps:**
1. Implement `extract_arg_boxes_separately()`
2. Implement `match_boxes_to_args()`
3. Test with all template types
4. Verify perfect alignment

---

**Status:** Ready to implement  
**Approach:** Two-pass rendering with box matching  
**Priority:** HIGH

