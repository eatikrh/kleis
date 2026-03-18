# Typst SVG Export Analysis

**Goal:** Understand exactly how `typst-svg` transforms layout coordinates to SVG coordinates to enable pixel-perfect overlay positioning for the structural editor.

## Key Files
- `crates/typst-svg/src/lib.rs`: Main entry point
- `crates/typst/src/layout/frame.rs`: Frame structure
- `crates/typst/src/layout/transform.rs`: Transform logic

## Coordinate Systems

### 1. Typst Internal Layout (Frames)
- **Origin:** Relative to parent container.
- **Math Block:** Often centered within its container.
- **Units:** `Abs` (points, 1/72 inch).

### 2. SVG Output
- **ViewBox:** `0 0 Width Height`
- **Root Transform:** Often includes padding or centering.
- **Content Transform:** Nested `g` tags with `translate`, `scale`, `matrix`.

## Investigation Log

### 1. The `svg` function
In `crates/typst-svg/src/lib.rs`:
```rust
pub fn svg(page: &Page) -> String {
    let mut renderer = SVGRenderer::new();
    renderer.write_header(page.frame.size()); // Sets viewBox to frame size

    let state = State::new(page.frame.size(), Transform::identity());
    renderer.render_page(state, Transform::identity(), page);
    renderer.finalize()
}
```
**Observation:** `svg()` uses `page.frame.size()` for the viewBox. It starts with `Transform::identity()`.
**Implication:** If `page.frame` has `margin: 0pt`, the viewBox should match the page size exactly, starting at (0,0).

### 2. The `render_page` function
```rust
fn render_page(&mut self, state: State, ts: Transform, page: &Page) {
    // ... background fill ...
    self.render_frame(state, ts, &page.frame);
}
```
It delegates to `render_frame`.

### 3. The `render_frame` function
This is where the recursion happens.
```rust
fn render_frame(&mut self, state: State, ts: Transform, frame: &Frame) {
    for (pos, item) in frame.items() {
        let item_ts = ts.pre_concat(Transform::translate(pos.x, pos.y));
        match item {
            FrameItem::Group(group) => {
                let group_ts = item_ts.pre_concat(group.transform);
                // ... recursion ...
            }
            // ...
        }
    }
}
```
**Observation:** This logic **exactly matches** our `extract_bounding_boxes_from_frame` implementation in `typst_compiler.rs`.
- We both iterate `frame.items()`.
- We both accumulate `ts.pre_concat(translate(pos))`.
- We both handle `group.transform`.

**So why the discrepancy?**

### 4. The Missing Link: `write_header` vs Actual Content bounds
If the `page.frame` size is `43.5 x 71.1` (from our debug logs), but the content is drawn at `x = 2.4`.

Why is it at `2.4`?
The SVG string shows: `<g transform="translate(10 10)">`.
Wait, in my previous analysis I saw `translate(10 10)` when we had `margin: 10pt`.
With `margin: 0pt`, did we see `translate(0 0)`?

Let's check the debug log from the "0pt margin" test:
```
Detected SVG global transform from string: translate(0, 0)
```
Yes!

But the coordinates were still:
Layout: `x = -28.2`
SVG: `x = 2.4` (implied from visual)

**Hypothesis:** The `page.frame` itself might have a non-zero origin?
No, frames don't have origins, they have sizes. Items inside have positions.

If an item inside `page.frame` has `pos.x = 2.4`, then Layout X should be `2.4`.
Why did we get `-28.2`?

**The Negative Coordinate Mystery**
In `typst`, math equations are blocks.
`block(align(center, ...))`

If the block width is `auto` (page width), and content is centered.
Maybe the "centering" is implemented by setting `pos.x = (page_width - content_width) / 2`.

But that would result in a positive coordinate.

Negative coordinates usually imply:
1. A transform `translate(-X, ...)`
2. Or an item positioned at a negative offset relative to its parent.

If we have:
```rust
#box($ x $)
```
The `#box` creates a container. The math `$ x $` is inside.

If the math block has a transform `translate(-50%, -50%)` to center itself?

### 5. Digging Deeper into Typst Layout
I need to look at `crates/typst/src/math/mod.rs` (or similar) to see how math blocks are laid out.

**Action Plan:**
1. Search for `layout_math` in Typst source.
2. See how it constructs the Frame.
3. Look for alignment/centering logic.

## Notes for Implementation
- If we find that Typst uses a specific logic for math block alignment, we can replicate it or reverse it.
- Or we can modify the Typst input to avoid it.

**Current File Status:**
- `typst_compiler.rs`: Implements `extract_bounding_boxes` mimicking `typst-svg`.
- `render.rs`: Uses `#box` wrapper.

**Next Steps:**
Read `crates/typst/src/math` to find the layout logic.

