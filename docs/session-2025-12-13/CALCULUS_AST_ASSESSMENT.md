# Calculus Operations: AST Assessment

## Overview

This document assesses how calculus operations (integrals, derivatives, limits, sums) are handled by the two parsing systems in Kleis:

1. **Equation Editor** (LaTeX input → `src/parser.rs` → template inference)
2. **Kleis Parser** (Kleis text syntax → `src/kleis_parser.rs`)

Both produce `Expression` ASTs defined in `src/ast.rs`, but they use different operation names and structures.

---

## The Shared AST Type

Both parsers produce the same `Expression` enum:

```rust
pub enum Expression {
    Const(String),           // Numeric constants
    Object(String),          // Variables, symbols
    Operation {              // Named operations with arguments
        name: String,
        args: Vec<Expression>
    },
    Placeholder { id, hint }, // Empty slots for editing
    // ... other variants
}
```

The key difference is in the **operation names** and **argument structures** used.

---

## Calculus Operations Comparison

### 1. Integrals

#### Equation Editor (LaTeX Path)

**Input:** `\int_{a}^{b} f \, dx` (from Calculus tab template)

**Initial Parse (flat AST):**
```
scalar_multiply(
    scalar_multiply(
        scalar_multiply(
            sub(Object("\\int"), Object("a")),
            sup(_, Object("b"))
        ),
        Object("f")
    ),
    mathrm(Object("d")),
    Object("x")
)
```

**After Template Inference:** (for double/triple integrals)
```
double_integral(integrand, region, var1, var2)
triple_integral(integrand, region, var1, var2, var3)
```

**Rendering Template Used:** `int_bounds`
```
int_bounds(integrand, lower, upper, variable)
```

#### Kleis Parser (Text Path)

**Input:** `∫f` (Unicode prefix operator)

**AST:**
```rust
Operation {
    name: "Integrate",
    args: [Object("f")]
}
```

**Input:** `Integrate(f, x, a, b)` (function-style)

**AST:**
```rust
Operation {
    name: "Integrate",
    args: [Object("f"), Object("x"), Object("a"), Object("b")]
}
```

#### Comparison Table: Integrals

| Feature | Equation Editor | Kleis Parser |
|---------|-----------------|--------------|
| **Indefinite integral** | `\int f dx` → flat chain | `∫f` → `Integrate(f)` |
| **Definite integral** | `\int_{a}^{b}` → sub/sup chain | `Integrate(f, x, a, b)` |
| **Double integral** | `\iint_{D}` → `double_integral` (inferred) | `∬f` → `DoubleIntegral(f)` |
| **Triple integral** | `\iiint_{V}` → `triple_integral` (inferred) | `∭f` → `TripleIntegral(f)` |
| **Line integral** | `\oint` → Object only | `∮f` → `LineIntegral(f)` |
| **Surface integral** | Not directly supported | `∯f` → `SurfaceIntegral(f)` |

---

### 2. Derivatives

#### Equation Editor (LaTeX Path)

**Input:** `\frac{\partial f}{\partial x}` (from Calculus tab)

**Initial Parse:**
```
scalar_divide(
    scalar_multiply(Object("\\partial"), Object("f")),
    scalar_multiply(Object("\\partial"), Object("x"))
)
```

**Rendering Template:** `d_part` (partial derivative)
```
d_part(numerator, denominator)
```

**LaTeX output:** `\frac{\partial\,{num}}{\partial {den}}`

#### Kleis Parser (Text Path)

**Input:** `D(f, x)` (Mathematica-style)

**AST:**
```rust
Operation {
    name: "D",
    args: [Object("f"), Object("x")]
}
```

**Semantics** (from `derivatives.kleis`):
- `D(f, x)` - Partial derivative (other variables are constants)
- `Dt(f, x)` - Total derivative (chain rule applies)
- `D(f, x, y)` - Mixed partial derivative
- `D(f, {x, n})` - nth derivative

#### Comparison Table: Derivatives

| Feature | Equation Editor | Kleis Parser |
|---------|-----------------|--------------|
| **Partial ∂f/∂x** | `\frac{\partial f}{\partial x}` → fraction structure | `D(f, x)` → operation |
| **Total df/dx** | `\frac{d f}{d x}` → `d_dt` operation | `Dt(f, x)` → operation |
| **Mixed ∂²f/∂x∂y** | Complex fraction nesting | `D(f, x, y)` → 3-arg operation |
| **nth derivative** | `\frac{d^n f}{dx^n}` | `D(f, {x, n})` → list notation |
| **Gradient ∇f** | `\nabla f` → Object | `∇f` → `gradient(f)` |

---

### 3. Limits

#### Equation Editor (LaTeX Path)

**Input:** `\lim_{x \to a} f(x)` (from Calculus tab)

**Initial Parse:**
```
scalar_multiply(
    sub(Object("\\lim"), ...arrow expression...),
    function_call(f, x)
)
```

**Rendering Template:** `lim`
```
lim(body, var, target)
```

**Output:** `\lim_{ {var} \to {target} } {body}`

#### Kleis Parser (Text Path)

**Currently:** Limits are not directly parsed as a prefix operator.

**Potential:** Could use `lim(f, x, a)` function-style if implemented.

#### Comparison Table: Limits

| Feature | Equation Editor | Kleis Parser |
|---------|-----------------|--------------|
| **Standard limit** | `\lim_{x \to a}` → sub with arrow | Not implemented |
| **limsup** | `\limsup_{x \to a}` → `limsup` op | Not implemented |
| **liminf** | `\liminf_{x \to a}` → `liminf` op | Not implemented |

---

### 4. Summation & Products

#### Equation Editor (LaTeX Path)

**Input:** `\sum_{i=1}^{n} a_i` (from Calculus tab)

**Initial Parse:**
```
scalar_multiply(
    sup(
        sub(Object("\\sum"), equals(i, 1)),
        Object("n")
    ),
    sub(Object("a"), Object("i"))
)
```

**Rendering Template:** `sum_bounds`
```
sum_bounds(body, from, to)
```

**Output:** `\sum_{ {from} }^{ {to} } {body}`

#### Kleis Parser (Text Path)

**Currently:** No direct Unicode Σ parsing as prefix operator.

**Could use:** Function-style `Sum(body, i, 1, n)` if implemented.

#### Comparison Table: Sums/Products

| Feature | Equation Editor | Kleis Parser |
|---------|-----------------|--------------|
| **Summation** | `\sum_{i=1}^{n}` → sub/sup chain | Not implemented |
| **Product** | `\prod_{i=1}^{n}` → sub/sup chain | Not implemented |

---

## Rendering Convergence

Despite different AST structures, both paths converge at rendering through shared templates in `src/render.rs`:

### LaTeX Templates (excerpt)

```rust
"int_bounds" → "\\int_{ {from} }^{ {to} } {integrand} \\, \\mathrm{d}{int_var}"
"sum_bounds" → "\\sum_{ {from} }^{ {to} } {body}"
"prod_bounds" → "\\prod_{ {from} }^{ {to} } {body}"
"lim" → "\\lim_{ {var} \\to {target} } {body}"
"d_part" → "\\frac{\\partial\\,{num}}{\\partial {den}}"
"d_dt" → "\\frac{d\\,{num}}{d{den}}"
```

### Unicode Templates (excerpt)

```rust
"int_bounds" → "∫_{ {from} }^{ {to} } {integrand} d{int_var}"
"sum_bounds" → "Σ_{ {from} }^{ {to} } {body}"
"lim" → "lim_{ {var}→{target} } {body}"
```

---

## Template Inference Layer

`src/template_inference.rs` bridges the gap between flat LaTeX parse and structured operations:

### Patterns Recognized

| Pattern | Inferred Operation |
|---------|-------------------|
| `\iint_{D} f \mathrm{d}x \mathrm{d}y` | `double_integral(f, D, x, y)` |
| `\iiint_{V} f \mathrm{d}x \mathrm{d}y \mathrm{d}z` | `triple_integral(f, V, x, y, z)` |
| `P \Rightarrow Q` | `implies(P, Q)` |
| `\forall x : x \in S` | `forall(x, in_set(x, S))` |
| `\nabla \times \mathbf{B}` | `curl(B)` |
| `\mathrm{Var}(X)` | `variance(X)` |

### Not Yet Inferred

- Single integrals with bounds → still flat structure
- Summations/products with bounds → still flat structure
- Limits → still flat structure

---

## Z3 Solver Handling

`src/solvers/z3/backend.rs` interprets Kleis-style operation names:

```rust
match op_name {
    "D" | "partial" => {
        // Partial derivative - uninterpreted function
        declare_uninterpreted("D", args.len())
    }
    "Dt" => {
        // Total derivative - uninterpreted function
        declare_uninterpreted("Dt", args.len())
    }
    "Integrate" | "integral" => {
        // Integration - uninterpreted function
        declare_uninterpreted("Integrate", args.len())
    }
    // ...
}
```

The Z3 backend recognizes the Kleis-style names (`D`, `Dt`, `Integrate`) rather than the rendering template names (`d_part`, `d_dt`, `int_bounds`).

---

## Gaps and Recommendations

### Current Gaps

1. **Naming inconsistency:**
   - Kleis uses: `Integrate`, `D`, `Dt`, `DoubleIntegral`
   - Templates use: `int_bounds`, `d_part`, `d_dt`, `double_integral`

2. **Missing Kleis features:**
   - No summation prefix operator (Σ)
   - No product prefix operator (Π)
   - No limit parsing

3. **Template inference gaps:**
   - Single definite integrals not converted to `int_bounds`
   - Summations/products remain as flat chains

### Potential Alignment Strategy

If alignment is desired in the future:

**Option A: Normalize to Kleis names**
- Change templates from `int_bounds` → `Integrate`
- Change `d_part` → `D`
- Pro: Matches Mathematica convention
- Con: Breaking change to existing templates

**Option B: Add mapping layer**
- Keep both naming conventions
- Add translation in render context
- Pro: Non-breaking
- Con: More complexity

**Option C: Keep as-is (current state)**
- Equation Editor uses rendering names
- Kleis uses semantic names
- Z3 understands both
- Pro: No changes needed
- Con: Cognitive overhead

---

## Summary Matrix

| Operation | Eq. Editor Input | Eq. Editor AST | Kleis Input | Kleis AST | Render Template |
|-----------|------------------|----------------|-------------|-----------|-----------------|
| Definite integral | `\int_{a}^{b} f dx` | sub/sup chain | `Integrate(f,x,a,b)` | `Integrate(...)` | `int_bounds` |
| Indefinite integral | `\int f dx` | flat multiply | `∫f` | `Integrate(f)` | — |
| Double integral | `\iint_{D}` | → inferred | `∬f` | `DoubleIntegral(f)` | `double_integral` |
| Partial derivative | `\frac{\partial f}{\partial x}` | fraction | `D(f, x)` | `D(f, x)` | `d_part` |
| Total derivative | `\frac{df}{dx}` | fraction | `Dt(f, x)` | `Dt(f, x)` | `d_dt` |
| Gradient | `\nabla f` | Object + multiply | `∇f` | `gradient(f)` | `grad` |
| Limit | `\lim_{x \to a} f` | sub chain | — | — | `lim` |
| Summation | `\sum_{i=1}^{n} a_i` | sub/sup chain | — | — | `sum_bounds` |

---

## Conclusion

The two parsing systems produce **related but structurally different ASTs** for calculus operations:

- **Equation Editor** creates flat, LaTeX-faithful structures that get post-processed
- **Kleis Parser** creates semantic, Mathematica-inspired structures directly
- **Rendering** provides a common output layer with shared templates
- **Z3 Backend** understands the Kleis semantic names

This design allows:
1. Visual editing with familiar LaTeX notation
2. Semantic reasoning with clean operation names
3. Consistent rendering across both paths

The trade-off is cognitive overhead in understanding two naming conventions, but this is manageable given the clear separation of concerns.

