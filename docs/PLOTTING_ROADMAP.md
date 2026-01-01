# Kleis Plotting Roadmap

> **Last Updated:** December 31, 2024
> **Status:** Phase 1 COMPLETE - All basic plot types implemented

## Vision

MATLAB-inspired plotting capabilities integrated with Kleis's numerical (LAPACK) and verification (Z3) features. Seamless workflow: compute → visualize → verify.

## Architecture

```
Kleis Expression  →  Lilaq/Typst  →  SVG  →  Jupyter/Browser/PDF
```

- **Backend**: Lilaq (Typst's plotting library)
- **Output**: SVG for web/Jupyter, PDF for documents
- **Integration**: Jupyter kernel emits `image/svg+xml` MIME bundles

---

## Implementation Status

### Phase 1: Core 2D Plots ✅ COMPLETE

| Function | Status | Description |
|----------|--------|-------------|
| `plot(x, y)` | ✅ | Line plot from data points |
| `scatter(x, y)` | ✅ | Scatter plot with markers |
| `fill_between(x, y)` | ✅ | Shaded area under curve (to y=0) |
| `bar(x, heights)` | ✅ | Vertical bar chart |
| `hbar(y, widths)` | ✅ | Horizontal bar chart |
| `stem(x, y)` | ✅ | Stem plot (discrete signals) |
| `hstem(x, y)` | ✅ | Horizontal stem plot |
| `boxplot(data1, data2, ...)` | ✅ | Box and whisker plots |
| `hboxplot(...)` | ✅ | Horizontal boxplots |
| `heatmap(matrix)` | ✅ | 2D color mesh / colormesh |
| `colormesh(matrix)` | ✅ | Alias for heatmap |
| `contour(matrix)` | ✅ | Contour lines |
| `quiver(x, y, u, v)` | ✅ | Vector/arrow field |

**All functions support optional title:** `plot(x, y, "Title")`

### Phase 2: Plot Styling & Options 🔜 NEXT

Currently we only support basic `plot(x, y)` calls. Lilaq provides many more styling options that should be exposed through an options record syntax:

```kleis
plot(x, y, { mark: "o", color: "blue", yerr: errors, smooth: true })
```

#### `plot()` Missing Parameters

| Parameter | Lilaq | Description |
|-----------|-------|-------------|
| `xerr` | ✅ | X error bars (symmetric or asymmetric) |
| `yerr` | ✅ | Y error bars (symmetric or asymmetric) |
| `color` | ✅ | Combined line/mark color |
| `stroke` | ✅ | Line style (width, dash pattern) |
| `mark` | ✅ | Marker type ("o", "x", "star", etc.) |
| `mark-size` | ✅ | Marker size |
| `mark-color` | ✅ | Marker color |
| `step` | ✅ | Step mode (none, start, end, center) |
| `smooth` | ✅ | Bézier spline interpolation |
| `every` | ✅ | Mark interval (skip marks) |
| `label` | ✅ | Legend label |
| `clip` | ✅ | Clip to data area |
| `z-index` | ✅ | Rendering order |

#### `scatter()` Missing Parameters

Scatter differs from plot by supporting **per-point** size and color:

| Parameter | Lilaq | Description |
|-----------|-------|-------------|
| `size` | ✅ | Per-point marker size (array) - area scales proportionally |
| `color` | ✅ | Per-point color (array of colors or floats for colormap) |
| `map` | ✅ | Color map (viridis, magma, etc.) when color is float array |
| `min/max` | ✅ | Color range for colormap |
| `norm` | ✅ | Normalization (linear, log, custom function) |
| `mark` | ✅ | Marker type |
| `stroke` | ✅ | Marker stroke |
| `alpha` | ✅ | Fill opacity (single or per-point array) |
| `label` | ✅ | Legend label |
| `clip` | ✅ | Clip to data area |
| `z-index` | ✅ | Rendering order |

**Key scatter() use cases:**
```kleis
// Variable size bubbles
scatter(x, y, { size: populations })

// Color-coded by value
scatter(x, y, { color: temperatures, map: "viridis" })

// Both size and color
scatter(x, y, { size: sizes, color: values, map: "plasma" })
```

#### `stem()` / `hstem()` Missing Parameters

| Parameter | Lilaq | Description |
|-----------|-------|-------------|
| `color` | ✅ | Combined line/mark color |
| `stroke` | ✅ | Line style |
| `mark` | ✅ | Marker type at stem tip |
| `mark-size` | ✅ | Marker size |
| `base` | ✅ | Y coordinate of baseline (default: 0) |
| `base-stroke` | ✅ | Baseline style |
| `label` | ✅ | Legend label |
| `clip` | ✅ | Clip to data area |
| `z-index` | ✅ | Rendering order |

**Key stem() use cases:**
```kleis
// Digital signal with custom baseline
stem(t, signal, { base: -1, color: "blue" })

// Impulse response
stem(n, h, { mark: "d", base: 0 })
```

#### `quiver()` Missing Parameters

| Parameter | Lilaq | Description |
|-----------|-------|-------------|
| `stroke` | ✅ | Arrow stroke style |
| `scale` | ✅ | Arrow length scaling |
| `pivot` | ✅ | Arrow pivot point (start, center, end) |
| `tip` | ✅ | Arrow tip style |
| `toe` | ✅ | Arrow tail style |
| `color` | ✅ | Arrow color (single, array, or function) |
| `map` | ✅ | Color map (viridis, etc.) |
| `min/max` | ✅ | Color range |
| `norm` | ✅ | Color normalization |
| `label` | ✅ | Legend label |

#### `heatmap()` / `colormesh()` Missing Parameters

| Parameter | Lilaq | Description |
|-----------|-------|-------------|
| `map` | ✅ | Color map |
| `min/max` | ✅ | Data range for colors |
| `norm` | ✅ | Normalization (linear, log) |

#### `contour()` Missing Parameters

| Parameter | Lilaq | Description |
|-----------|-------|-------------|
| `levels` | ✅ | Contour level values |
| `stroke` | ✅ | Line style |
| `fill` | ✅ | Fill between contours |
| `labels` | ✅ | Show level labels |

#### Implementation Approach

```kleis
// Proposed syntax using options record
plot(x, y, {
    mark: "o",
    color: "blue",
    yerr: [0.1, 0.2, 0.1],
    smooth: true,
    label: "Measured data"
})
```

**Requirements:**
1. Parse record literals `{ key: value, ... }` in evaluator
2. Map Kleis option names to Lilaq parameter names
3. Generate appropriate Typst code with options

### Phase 3: Function Plotting

| Function | Status | Description |
|----------|--------|-------------|
| `fplot(f, x_min, x_max)` | ⏳ | Plot function with auto-sampling |
| `fplot([f, g], x_min, x_max)` | ⏳ | Multiple functions |
| `fplot(f, domain)` | ⏳ | Domain syntax: `0..2*pi` |

**Implementation notes:**
- Need to evaluate Kleis function at sample points
- Sample count configurable (default: 100)
- Handle singularities/discontinuities

### Phase 4: Additional Plot Types

| Function | Status | Description |
|----------|--------|-------------|
| `histogram(data, bins)` | ⏳ | Histogram |
| `pie(values, labels)` | ⏳ | Pie chart |
| `polar(theta, r)` | ⏳ | Polar coordinates |
| `semilogy(x, y)` | ⏳ | Log Y axis |
| `loglog(x, y)` | ⏳ | Log both axes |

### Phase 5: 3D & Surface Plots

| Function | Status | Description |
|----------|--------|-------------|
| `surf(X, Y, Z)` | ⏳ | 3D surface |
| `mesh(X, Y, Z)` | ⏳ | 3D wireframe |
| `contourf(...)` | ⏳ | Filled contours |

**Note:** Lilaq has limited 3D support. May need:
- Typst + CeTZ for 3D
- Or generate Plotly JSON for interactive 3D

### Phase 6: Control Systems

| Function | Status | Description |
|----------|--------|-------------|
| `bode(tf)` | ⏳ | Bode plot for control systems |
| `nyquist(tf)` | ⏳ | Nyquist diagram |
| `step_response(tf)` | ⏳ | Step response plot |

### Phase 7: Matrix Visualization

| Function | Status | Description |
|----------|--------|-------------|
| `imagesc(matrix)` | ⏳ | Scaled image of matrix |
| `spy(matrix)` | ⏳ | Sparsity pattern |

### Phase 8: Plot Annotations & Overlays

Lilaq provides shape primitives for annotating plots:

| Function | Status | Description |
|----------|--------|-------------|
| `rect(x, y, width, height)` | ⏳ | Rectangle overlay |
| `circle(x, y, radius)` | ⏳ | Circle overlay |
| `line(x1, y1, x2, y2)` | ⏳ | Line segment |
| `text(x, y, content)` | ⏳ | Text annotation |
| `arrow(x1, y1, x2, y2)` | ⏳ | Arrow annotation |
| `vlines(...x)` | ⏳ | Vertical lines |
| `hlines(...y)` | ⏳ | Horizontal lines |
| `vspan(x1, x2)` | ⏳ | Shaded vertical region |
| `hspan(y1, y2)` | ⏳ | Shaded horizontal region |

#### `rect()` Parameters (from Lilaq)

| Parameter | Description |
|-----------|-------------|
| `x`, `y` | Origin coordinates (data, length, or %) |
| `width`, `height` | Size (data, length, or %) |
| `align` | Alignment at origin |
| `fill` | Fill color/gradient |
| `stroke` | Border style |
| `radius` | Corner rounding |
| `inset`, `outset` | Padding/expansion |
| `label` | Legend label |
| `clip` | Clip to data area |
| `z-index` | Rendering order |

#### `vlines()` / `hlines()` Parameters

| Parameter | Description |
|-----------|-------------|
| `..x` / `..y` | One or more line positions (variadic) |
| `min` | Start coordinate (auto = edge of diagram) |
| `max` | End coordinate (auto = edge of diagram) |
| `stroke` | Line style |
| `label` | Legend label |
| `z-index` | Rendering order |

**Use cases:**
```kleis
// Mark threshold
vlines(critical_value, { stroke: "red", label: "Threshold" })

// Mark multiple events
vlines(t1, t2, t3, { stroke: "blue" })

// Partial line (fixed range)
vlines(x, { min: 0, max: 5, stroke: "green" })
```

#### General Use Cases
- Highlight regions of interest
- Add bounding boxes
- Create custom legends
- Annotate specific data points
- Mark thresholds and critical values

### Phase 9: Axis Scaling & Transforms

Lilaq supports various axis scales for visualizing data with large ranges:

| Function | Status | Description |
|----------|--------|-------------|
| `xscale("log")` | ⏳ | Logarithmic X axis |
| `yscale("log")` | ⏳ | Logarithmic Y axis |
| `xscale("symlog")` | ⏳ | Symmetric log (handles negatives) |
| `yscale("symlog")` | ⏳ | Symmetric log Y axis |

#### Built-in Scales (from Lilaq)

| Scale | Transform | Use Case |
|-------|-----------|----------|
| `linear` | x → x | Default, uniform scaling |
| `log` | x → log(x) | Large positive ranges |
| `symlog` | symmetric log | Data with positive and negative |
| `sqrt` | x → √x | Moderate compression |
| `power(n)` | x → xⁿ | Custom power scaling |
| `datetime` | linear + date ticks | Time-series data |

#### DateTime Scale

For time-series plots with proper date/time axis labels:

```kleis
// Plot with datetime X axis
plot(timestamps, values, { xscale: "datetime" })

// Lilaq auto-formats: "Jan 2024", "Feb 2024", etc.
```

**Use cases:**
- Stock prices over time
- Sensor data with timestamps
- Event timelines
- Historical data visualization

### Phase 10: Data Processing (Lilaq Math Library)

Lilaq provides a [math library](https://lilaq.org/docs/category/math) for common data processing tasks:

| Function | Lilaq | Description |
|----------|-------|-------------|
| `linspace(start, end, num)` | ✅ | Evenly-spaced numbers in interval |
| `arange(start, end, step)` | ✅ | Numbers spaced by step |
| `mesh(x, y)` | ✅ | Rectangular mesh from two arrays |
| `minmax(data)` | ✅ | Min and max (ignoring NaN) |
| `cmin(data)` | ✅ | Min ignoring NaN |
| `cmax(data)` | ✅ | Max ignoring NaN |
| `percentile(data, q)` | ✅ | Compute q-th percentile |
| `sign(x)` | ✅ | Sign of number |
| `pow10(x)` | ✅ | Compute 10^x |
| `divmod(a, b)` | ✅ | Integer division with remainder |

**Use cases:**
```kleis
// Generate x values for function plotting
let x = linspace(0, 2*pi, 100) in
plot(x, map(sin, x))

// Create mesh for 3D/contour plots
let (X, Y) = mesh(linspace(-2, 2, 50), linspace(-2, 2, 50)) in
contour(X, Y, f(X, Y))

// Data statistics
let (lo, hi) = minmax(data) in
plot(x, data, { ylim: (lo - 0.1, hi + 0.1) })
```

**Note:** Some of these may overlap with existing Kleis builtins or could be implemented in stdlib.

#### Convenience Functions

| Function | Description |
|----------|-------------|
| `semilogy(x, y)` | Plot with log Y axis |
| `semilogx(x, y)` | Plot with log X axis |
| `loglog(x, y)` | Plot with log both axes |

#### Proposed Syntax

```kleis
// Using diagram options
diagram({ xscale: "log", yscale: "linear" }, [
    plot(frequencies, magnitudes)
])

// Or convenience functions
semilogy(frequencies, magnitudes)
loglog(frequencies, response)
```

**Quantum computing use case:**
```kleis
// Visualize density matrix
heatmap(density_matrix)

// Probability distribution
bar([0, 1, 2, 3], probabilities(quantum_state))
```

### Phase 11: Vector Operations (Lilaq Vec Library) ⏳ NEW

Lilaq provides a [vec module](https://lilaq.org/docs/category/vec) for vector operations:

| Function | Description | Kleis Status |
|----------|-------------|--------------|
| `add(a, b)` | Pair-wise addition: `[a₁+b₁, a₂+b₂, ...]` | ❌ |
| `subtract(a, b)` | Pair-wise subtraction: `[a₁-b₁, a₂-b₂, ...]` | ❌ |
| `multiply(v, s)` | Scalar multiplication: `[s·v₁, s·v₂, ...]` | ❌ |
| `inner(a, b)` | Inner/dot product: `Σ(aᵢ·bᵢ)` | ❌ |
| `transform(a, b, f)` | Apply `f(aᵢ, bᵢ)` to pairs | ❌ |

**Use cases:**
```kleis
// Combine two data series
let combined = add(signal1, signal2) in
plot(time, combined)

// Scale data
let scaled = multiply(data, 0.5) in
scatter(x, scaled)

// Compute correlation (via inner product)
let corr = inner(normalize(a), normalize(b))

// Custom transformation
let polar_to_xy = transform(r, theta, (r, t) => (r * cos(t), r * sin(t)))
```

**Note:** Some of these may overlap with existing Kleis list operations.

### Phase 12: Color Maps (Lilaq Color Library) ⏳ NEW

Lilaq provides [perceptually uniform, CVD-friendly color maps](https://lilaq.org/docs/category/color):

**Sequential color maps:**
| Map | Origin | Use Case |
|-----|--------|----------|
| `viridis` | Matplotlib | Default, excellent for most data |
| `magma` | Matplotlib | Dark to bright, good for intensity |
| `plasma` | Matplotlib | Purple to yellow |
| `inferno` | Matplotlib | Dark to bright fire colors |
| `cividis` | Optimized viridis | Best for CVD accessibility |

**Diverging color maps:**
| Map | Use Case |
|-----|----------|
| `vik` | Centered data (e.g., temperature anomalies) |
| `roma` | Diverging with distinct colors |
| `berlin` | Blue-white-red style |

**Bi-sequential color maps:**
| Map | Use Case |
|-----|----------|
| `tovu` | CVD-friendly topo + bukavu combination |

**Qualitative color maps:**
| Map | Use Case |
|-----|----------|
| Cycle maps | Categorical data, line/scatter plot styling |

**All maps are:**
- ✅ CVD (color-vision deficiency) friendly
- ✅ Perceptually uniform
- ✅ Perceptually ordered

**Proposed Kleis syntax:**
```kleis
// Heatmap with specific color map
heatmap(matrix, { map: "viridis" })

// Scatter with color-coded values
scatter(x, y, { color: values, map: "plasma" })

// Contour with diverging colors
contour(matrix, { map: "vik", center: 0 })
```

**Not yet implemented in Kleis** - currently using Lilaq defaults.

### Phase 13: Tick Formatters ⏳ NEW

Lilaq provides [tick formatters](https://lilaq.org/docs/category/tick-formatters) for controlling axis label display:

| Formatter | Description | Use Case |
|-----------|-------------|----------|
| `linear` | Standard numeric formatting | Default for linear scales |
| `log` | Logarithmic formatting (10², 10³) | Log-scale axes |
| `symlog` | Symmetric log formatting | Symlog-scale axes |
| `datetime` | Date/time formatting | Time-series data |
| `datetime-smart-first` | Smart period start (month/day/hour) | Time-series ticks |
| `datetime-smart-format` | Auto datetime formatting | Time-series ticks |
| `datetime-smart-offset` | Offset for datetime sets | Relative time display |
| `manual` | Explicit custom labels | Categorical axes |

**Proposed Kleis syntax:**
```kleis
// Custom tick labels for bar chart
bar([0, 1, 2], heights, { 
    xticks: manual(["Q1", "Q2", "Q3"]) 
})

// Log-formatted ticks
plot(x, y, { 
    yscale: "log",
    yticks: log()
})

// Time series with smart datetime formatting
plot(dates, values, {
    xscale: "datetime",
    xticks: datetime-smart-format()
})
```

**Not yet implemented in Kleis** - currently using Lilaq auto-detection.

### Phase 14: Tick Locators ⏳ NEW

Lilaq provides [tick locators](https://lilaq.org/docs/category/tick-locators) for controlling WHERE ticks are placed:

**Scale-based locators:**
| Locator | Description |
|---------|-------------|
| `linear` | Evenly-spaced ticks on linear scale |
| `log` | Power-of-base ticks on log scale |
| `symlog` | Ticks for symmetric log scale |
| `manual` | Explicit tick positions |

**Datetime locators:**
| Locator | Description |
|---------|-------------|
| `years` | Year boundaries |
| `months` | Month boundaries |
| `days` | Day boundaries |
| `hours` | Hour boundaries |
| `minutes` | Minute boundaries |
| `seconds` | Second boundaries |

**Subtick locators:**
| Locator | Description |
|---------|-------------|
| `subticks-linear` | Minor ticks between major linear ticks |
| `subticks-log` | Minor ticks between major log ticks |
| `subticks-symlog` | Minor ticks between major symlog ticks |

**Locators vs Formatters:**
- **Locators** → WHERE: `[0, 0.5, 1.0, 1.5, 2.0]`
- **Formatters** → HOW: `["0", "½", "1", "1½", "2"]`

**Proposed Kleis syntax:**
```kleis
// Custom tick positions
plot(x, y, {
    xticks: manual([0, 0.25, 0.5, 0.75, 1.0]),
    xtick-format: manual(["0", "¼", "½", "¾", "1"])
})

// Logarithmic with minor ticks
semilogy(x, y, {
    yticks: log(),
    ysubticks: subticks-log()
})

// Time series by month
plot(dates, values, {
    xticks: months(),
    xsubticks: days()
})
```

**Not yet implemented in Kleis** - currently using Lilaq auto-detection.

### Phase 15: Data Loading ⏳ NEW

Lilaq provides [`load-txt`](https://lilaq.org/docs/reference/load-txt) for CSV/text file parsing:

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `delimiter` | `","` | Column separator |
| `comments` | `"#"` | Comment character |
| `skip-rows` | `0` | Header rows to skip |
| `usecols` | `auto` | Which columns to extract |
| `header` | `false` | Parse first row as column names |
| `converters` | `float` | Type conversion function |

**Key feature:** Returns columns (not rows) - ready for plotting!

**Proposed Kleis syntax:**
```kleis
// Load CSV with headers
let data = load_csv("experiment.csv", { header: true }) in
plot(data["time"], data["temperature"])

// Load specific columns, skip header
let (x, y) = load_csv("data.txt", { 
    usecols: [0, 2],
    skip_rows: 1 
}) in
scatter(x, y)

// Tab-separated with comments
let cols = load_csv("measurements.tsv", {
    delimiter: "\t",
    comments: "//"
}) in
plot(cols[0], cols[1])
```

**Use cases:**
- Scientific data from instruments
- Financial data from exports
- Benchmark results
- Any columnar text data

**Not yet implemented in Kleis.**

### Phase 16: Equation Annotations ⭐ KLEIS UNIQUE

**Kleis advantage:** We can render beautiful mathematical equations directly on plots using our existing Typst rendering pipeline!

**Proposed Kleis syntax:**
```kleis
// Annotate with rendered equation
plot(x, sin_values, {
    title: $sin(ωt + φ)$,           // Equation as title
    xlabel: $t$ " (seconds)",
    ylabel: $A sin(ωt)$
})

// Equation annotation at specific point
plot(x, y) with [
    text(pi, 0, $f(π) = 0$),        // Equation label at (π, 0)
    text(2*pi, 1, $∫_0^{2π} sin(x) dx = 0$)
]

// Function plot with equation in legend
fplot(x => sin(x), 0, 2*pi, { label: $y = sin(x)$ })
fplot(x => cos(x), 0, 2*pi, { label: $y = cos(x)$ })

// Tensor equation annotation
heatmap(metric_tensor, {
    title: $g_{μν} = η_{μν} + h_{μν}$
})
```

**What we can render (already have):**
- Greek letters: $α, β, γ, μ, ν, ω$
- Subscripts/superscripts: $x_i$, $x^2$
- Fractions: $\frac{∂f}{∂x}$
- Integrals: $∫_a^b f(x) dx$
- Tensors: $Γ^λ_{μν}$, $R^ρ_{σμν}$
- Matrices: Full matrix notation

**This differentiates Kleis from:**
- Matplotlib: LaTeX requires external renderer
- Plotly: Limited math support
- MATLAB: Basic LaTeX subset

**Implementation approach:**
1. Detect `$...$` in plot text parameters
2. Render via existing Typst pipeline
3. Embed as SVG elements in plot

**Not yet implemented** - but we have all the pieces!

---

## Integration with Kleis Features

### Numerical (LAPACK)
```kleis
let A = [[1, 2], [3, 4]] in
let eigs = eigenvalues(A) in
scatter(real_parts(eigs), imag_parts(eigs))  // Eigenvalue plot
```

### Verification (Z3)
```kleis
// Plot and verify stability
let response = step_response(system) in
plot(time, response)
assert all(t => abs(response(t)) < 1.1)  // Bounded response
```

---

## Jupyter Kernel Updates Needed

File: `kleis-notebook/kleis_kernel/kernel.py`

```python
def _format_output(self, output):
    # Detect SVG plot output
    if "PLOT_SVG:" in output:
        svg_start = output.index("PLOT_SVG:") + 9
        svg_data = output[svg_start:]
        return {
            'data': {
                'image/svg+xml': svg_data,
                'text/plain': '[Plot]'
            },
            'metadata': {}
        }
    # ... existing formatting
```

---

## References

- [Lilaq Documentation](https://lilaq.org)
- [Typst CeTZ](https://github.com/cetz-package/cetz) - For 3D graphics
- [MATLAB Plot Gallery](https://www.mathworks.com/products/matlab/plot-gallery.html) - Inspiration

---

## Session Log

- **Dec 31, 2024**: Full Lilaq integration complete
  - Phase 1 COMPLETE: All 13 plot types implemented
  - Added: plot, scatter, fill_between, bar, hbar, stem, hstem, boxplot, hboxplot, heatmap, colormesh, contour, quiver
  - Created `src/plotting.rs` module with Lilaq code generation
  - Jupyter kernel: Fixed SVG rendering, added multi-plot support, stdlib import fix
  - 18 plotting examples passing in `examples/plotting/basic_plots.kleis`
  - Documented Phases 2-11 enhancements from Lilaq reference:
    - Phase 2: Plot styling & options (color, stroke, marks, error bars)
    - Phase 8: Annotations (rect, circle, line, text, arrow, vlines, hlines, vspan, hspan)
    - Phase 9: Axis scaling (linear, log, symlog, sqrt, power, datetime)
    - Phase 10: Math library (linspace, arange, mesh, minmax, percentile, sign, pow10, divmod)
    - Phase 11: Vec library (add, subtract, multiply, inner, transform)
    - Phase 12: Color maps (viridis, magma, plasma, inferno, cividis, diverging, qualitative)
    - Phase 13: Tick formatters (linear, log, symlog, datetime, manual)
    - Phase 14: Tick locators (linear, log, symlog, manual, datetime units, subticks)
    - Phase 15: Data loading (load_csv with headers, columns, converters)
    - Phase 16: Equation annotations ⭐ (Kleis-unique: $...$  in titles, labels, annotations)
  - Tests: `kleis test examples/plotting/basic_plots.kleis`
