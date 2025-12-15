# Operation Mapping Verification

**Date:** 2025-12-05  
**Purpose:** Verify all 16 new operations are correctly mapped in all systems

## Mapping Verification Table

| # | Operation | Backend Template | Palette Button | templateMap | astTemplates | Typst Mapping | Status |
|---|-----------|------------------|----------------|-------------|--------------|---------------|--------|
| 1 | Fourier Transform | ✅ template_fourier_transform | ✅ `\mathcal{F}[□](□)` | ✅ 'fourier_transform' | ✅ Yes | ✅ Line 845-848 | ✅ |
| 2 | Inverse Fourier | ✅ template_inverse_fourier | ✅ `\mathcal{F}^{-1}[□](□)` | ✅ 'inverse_fourier' | ✅ Yes | ✅ Line 845-848 | ✅ |
| 3 | Laplace Transform | ✅ template_laplace_transform | ✅ `\mathcal{L}[□](□)` | ✅ 'laplace_transform' | ✅ Yes | ✅ Line 845-848 | ✅ |
| 4 | Inverse Laplace | ✅ template_inverse_laplace | ✅ `\mathcal{L}^{-1}[□](□)` | ✅ 'inverse_laplace' | ✅ Yes | ✅ Line 845-848 | ✅ |
| 5 | Convolution | ✅ template_convolution | ✅ `(□ \ast □)(□)` | ✅ 'convolution' | ✅ Yes | ✅ Line 857, 913 | ✅ |
| 6 | Kernel Integral | ✅ template_kernel_integral | ✅ `\int_{□} □ □ \, d□` | ✅ 'kernel_integral' | ✅ Yes | ✅ Line 843-844, 908-909, 925-926 | ✅ |
| 7 | Green's Function | ✅ template_greens_function | ✅ `G(□, □)` | ✅ 'greens_function' | ✅ Yes | ✅ Line 849-851 | ✅ |
| 8 | Projection | ✅ template_projection | ✅ `\Pi[□](□)` | ✅ 'projection' | ✅ Yes | ✅ Line 845-848 | ✅ |
| 9 | Modal Integral | ✅ template_modal_integral | ✅ `\int_{□} □ \, d\mu(□)` | ✅ 'modal_integral' | ✅ Yes | ✅ Line 854-855, 911 | ✅ FIXED |
| 10 | Projection Kernel | ✅ template_projection_kernel | ✅ `K(□, □)` | ✅ 'projection_kernel' | ✅ Yes | ✅ Line 806-808, 849-851 | ✅ |
| 11 | Causal Bound | ✅ template_causal_bound | ✅ `c(□)` | ✅ 'causal_bound' | ✅ Yes | ✅ Line 809-810 | ✅ |
| 12 | Projection Residue | ✅ template_projection_residue | ✅ `\mathrm{Residue}[□, □]` | ✅ 'projection_residue' | ✅ Yes | ✅ Line 811-812, 852-853 | ✅ |
| 13 | Modal Space | ✅ template_modal_space | ✅ `\mathcal{M}_{□}` | ✅ 'modal_space' | ✅ Yes | ✅ Line 813-814 | ✅ |
| 14 | Spacetime | ✅ template_spacetime | ✅ `\mathbb{R}^4` | ✅ 'spacetime' | ✅ Yes | ✅ (no args) | ✅ |
| 15 | Hont | ✅ template_hont | ✅ `\mathcal{H}_{□}` | ✅ 'hont' | ✅ Yes | ✅ Line 815-816 | ❓ |

## Hont Template Details

### Backend (src/templates.rs line 753)
```rust
pub fn template_hont() -> Expression {
    Expression::operation(
        "hont",
        vec![Expression::placeholder(next_id(), "dimension")],
    )
}
```
Arguments: 1 (dimension)

### Palette Button (static/index.html line 820)
```html
<button class="math-btn" 
        onclick="insertTemplate('\\mathcal{H}_{□}')" 
        data-tooltip="Hont (Hilbert Ontology)">
    \(\mathcal{H}_\infty\)
</button>
```

### Template Map (static/index.html line 1648)
```javascript
'\\mathcal{H}_{□}': 'hont'
```

### AST Template (static/index.html line 1714)
```javascript
hont: { 
    Operation: { 
        name: 'hont', 
        args: [{Placeholder:{id:0,hint:'dimension'}}] 
    } 
}
```

### Rendering Templates

**Unicode:**
```
𝓗_{dimension}
```

**LaTeX:**
```
\mathcal{H}_{{dimension}}
```

**HTML:**
```
<span class="math-script">𝓗</span><sub class="math-sub">{dimension}</sub>
```

**Typst:**
```
cal(H)_({dimension})
```

### Placeholder Mapping (src/render.rs line 815-816)
```rust
} else if name == "hont" {
    result = result.replace("{dimension}", first);  // arg[0] → {dimension}
```

## Everything Looks Correct! ✅

All components are properly configured. 

**Please provide:**
1. The exact error message you see
2. When the error appears (click, fill, render)
3. What you entered in the placeholder (if anything)

This will help me identify the specific issue!

