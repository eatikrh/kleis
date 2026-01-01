# Kleis Plotting Roadmap

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

### Phase 1: Core 2D Plots ✅ DONE

| Function | Status | Description |
|----------|--------|-------------|
| `plot(x, y)` | ✅ | Line plot from data points |
| `scatter(x, y)` | ✅ | Scatter plot with markers |

### Phase 2: Function Plotting 🔜 NEXT

| Function | Status | Description |
|----------|--------|-------------|
| `fplot(f, x_min, x_max)` | ⏳ | Plot function with auto-sampling |
| `fplot([f, g], x_min, x_max)` | ⏳ | Multiple functions |
| `fplot(f, domain)` | ⏳ | Domain syntax: `0..2*pi` |

**Implementation notes:**
- Need to evaluate Kleis function at sample points
- Sample count configurable (default: 100)
- Handle singularities/discontinuities

### Phase 3: Statistical & Categorical

| Function | Status | Description |
|----------|--------|-------------|
| `bar(x, heights)` | ⏳ | Bar chart |
| `histogram(data, bins)` | ⏳ | Histogram |
| `boxplot(data...)` | ⏳ | Box and whisker |
| `pie(values, labels)` | ⏳ | Pie chart |

### Phase 4: 3D & Surface Plots

| Function | Status | Description |
|----------|--------|-------------|
| `surf(X, Y, Z)` | ⏳ | 3D surface |
| `mesh(X, Y, Z)` | ⏳ | 3D wireframe |
| `contour(f, x_range, y_range)` | ⏳ | Contour lines |
| `contourf(...)` | ⏳ | Filled contours |

**Note:** Lilaq has limited 3D support. May need:
- Typst + CeTZ for 3D
- Or generate Plotly JSON for interactive 3D

### Phase 5: Scientific/Engineering

| Function | Status | Description |
|----------|--------|-------------|
| `quiver(X, Y, U, V)` | ⏳ | Vector/arrow field |
| `polar(theta, r)` | ⏳ | Polar coordinates |
| `semilogy(x, y)` | ⏳ | Log Y axis |
| `loglog(x, y)` | ⏳ | Log both axes |
| `bode(tf)` | ⏳ | Bode plot for control systems |
| `nyquist(tf)` | ⏳ | Nyquist diagram |

### Phase 6: Matrix Visualization

| Function | Status | Description |
|----------|--------|-------------|
| `heatmap(matrix)` | ⏳ | Color-coded matrix values |
| `imagesc(matrix)` | ⏳ | Scaled image of matrix |
| `spy(matrix)` | ⏳ | Sparsity pattern |

**Quantum computing use case:**
```kleis
// Visualize density matrix
heatmap(density_matrix)

// Probability distribution
bar([0, 1, 2, 3], probabilities(quantum_state))
```

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

- **Dec 31, 2024**: Initial implementation
  - Added `plot()` and `scatter()` built-ins
  - Created `src/plotting.rs` module
  - Lilaq/Typst → SVG pipeline working
  - 9 plotting examples passing

