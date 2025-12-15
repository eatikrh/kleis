# AST Translation Problem: Editor ↔ Kleis Isomorphism

## The Two-Layer Problem

There are actually **two distinct gaps**:

### Gap 1: Pattern Matching (Solvable)

LaTeX `\frac{\partial f}{\partial x}` can be pattern-matched and translated to `D(f, x)`.
This is a mechanical transformation we can implement in `template_inference.rs`.

### Gap 2: Missing Kleis Types (Deeper Issue)

Kleis has **type classes** for calculus (e.g., `Differentiable(F)` with operation `D`)
but lacks **concrete types** representing calculus expressions as first-class values:

| What We Have | What's Missing |
|--------------|----------------|
| `operation D : F -> Variable -> F` | `structure DefiniteIntegral(F, var, lower, upper)` |
| `operation Integrate : F -> Variable -> F` | `structure Sum(F, var, lower, upper)` |
| Axioms for derivative rules | `structure Limit(F, var, target)` |
| | `structure Product(F, var, lower, upper)` |

**The distinction:**
- `D(f, x)` is an *operation* that returns something of type `F`
- But `∫₀¹ x² dx` should be a *type* that represents "the definite integral from 0 to 1 of x²"

Without a proper `Integral` type, we can parse `\int_{0}^{1} x^2 dx` but can't represent it semantically in a way Z3 can verify properties about.

---

## The Problem

We have two AST representations that serve different purposes:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        EQUATION EDITOR                               │
│  ┌─────────────┐     ┌──────────────┐     ┌────────────────────┐   │
│  │ LaTeX Input │ ──► │ Flat/Visual  │ ──► │ Beautiful Render   │   │
│  │ \int_{a}^b  │     │ AST (sub,sup)│     │ on Screen/PDF      │   │
│  └─────────────┘     └──────────────┘     └────────────────────┘   │
│                              │                                       │
│                              │ ??? (missing translation)             │
│                              ▼                                       │
│  ┌─────────────┐     ┌──────────────┐     ┌────────────────────┐   │
│  │ Kleis Input │ ──► │ Semantic AST │ ──► │ Z3 Verification    │   │
│  │ Integrate() │     │ (D, Integrate│     │ (validity check)   │   │
│  └─────────────┘     └──────────────┘     └────────────────────┘   │
│                        KLEIS PARSER                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Current State

| Capability | Equation Editor | Kleis Parser |
|------------|-----------------|--------------|
| Rich visual editing | ✅ | ❌ |
| LaTeX/PDF rendering | ✅ | ✅ (via templates) |
| Semantic structure | ❌ (flat) | ✅ |
| Z3 verification | ❌ | ✅ |
| **Bidirectional translation** | ❌ | ❌ |

### The Gap

**User wants to:**
1. Edit an equation visually in the Equation Editor
2. Click "Verify" to check if the equation is valid
3. Get Z3 verification results

**But currently:**
1. Editor produces flat AST with `sub`, `sup`, `scalar_multiply` chains
2. Z3 backend expects semantic AST with `D`, `Integrate`, `forall`
3. No reliable translation exists between them

---

## Why This Is Hard

### 1. Information Loss in Flat AST

The flat AST loses semantic intent:

```latex
\frac{\partial f}{\partial x}
```

Parses to:
```
scalar_divide(
    scalar_multiply(Object("\\partial"), Object("f")),
    scalar_multiply(Object("\\partial"), Object("x"))
)
```

**Lost information:**
- This is a *derivative*, not just a fraction of products
- The `∂` symbols are paired with specific meanings
- `f` is a function, `x` is the differentiation variable

### 2. Ambiguity in Visual Notation

Visual math is inherently ambiguous:

| Notation | Could mean |
|----------|------------|
| `fg` | `f * g` or `f(g)` (function application) |
| `x^2` | Power or superscript index |
| `A_i` | Subscript index or variable named `A_i` |
| `dx` | Differential or `d * x` |

Kleis resolves this through explicit syntax; LaTeX does not.

### 3. Template Inference Is Incomplete

`template_inference.rs` handles some patterns:
- ✅ `\iint_{D} ... \mathrm{d}x \mathrm{d}y` → `double_integral`
- ✅ `P \Rightarrow Q` → `implies`
- ❌ `\int_{a}^{b} f dx` → still flat
- ❌ `\frac{\partial f}{\partial x}` → still flat
- ❌ `\sum_{i=1}^{n} a_i` → still flat

---

## Proposed Solution: Semantic Translation Layer

### Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   EQUATION EDITOR                                                   │
│   ┌──────────────┐                                                 │
│   │ Visual AST   │                                                 │
│   │ (sub, sup,   │                                                 │
│   │  frac, etc.) │                                                 │
│   └──────┬───────┘                                                 │
│          │                                                          │
│          ▼                                                          │
│   ┌──────────────────────────────────────────────────────────┐     │
│   │           SEMANTIC TRANSLATION LAYER (NEW)               │     │
│   │                                                          │     │
│   │  visual_to_semantic(visual_ast) -> Result<KleisAST>     │     │
│   │  semantic_to_visual(kleis_ast) -> VisualAST             │     │
│   │                                                          │     │
│   │  • Pattern recognition (extended template_inference)     │     │
│   │  • Ambiguity resolution (with user hints if needed)      │     │
│   │  • Bidirectional mapping registry                        │     │
│   └──────────────────────────────────────────────────────────┘     │
│          │                          ▲                               │
│          ▼                          │                               │
│   ┌──────────────┐           ┌──────────────┐                      │
│   │ Kleis AST    │ ◄──────── │ Kleis Text   │                      │
│   │ (semantic)   │           │ Parser       │                      │
│   └──────┬───────┘           └──────────────┘                      │
│          │                                                          │
│          ▼                                                          │
│   ┌──────────────┐                                                 │
│   │ Z3 Backend   │                                                 │
│   │ (verify)     │                                                 │
│   └──────────────┘                                                 │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

### Key Components

#### 1. Extended Pattern Recognition

Expand `template_inference.rs` to recognize more patterns:

```rust
// Current: only double/triple integrals, implications, quantifiers
// Proposed: all calculus operations

fn try_infer_definite_integral(expr: &Expression) -> Option<Expression> {
    // Pattern: sup(sub(\int, lower), upper) * integrand * d * var
    // → Integrate(integrand, var, lower, upper)
}

fn try_infer_partial_derivative(expr: &Expression) -> Option<Expression> {
    // Pattern: scalar_divide(∂ * f, ∂ * x)
    // → D(f, x)
}

fn try_infer_summation(expr: &Expression) -> Option<Expression> {
    // Pattern: sup(sub(\sum, i=start), end) * body
    // → Sum(body, i, start, end)
}
```

#### 2. Canonical Operation Names

Define a canonical set of semantic operation names:

```rust
// Calculus
const OP_INTEGRATE: &str = "Integrate";      // ∫
const OP_PARTIAL: &str = "D";                // ∂/∂x
const OP_TOTAL_DERIV: &str = "Dt";           // d/dx
const OP_GRADIENT: &str = "gradient";        // ∇
const OP_SUM: &str = "Sum";                  // Σ
const OP_PRODUCT: &str = "Product";          // Π
const OP_LIMIT: &str = "Limit";              // lim

// Rendering uses aliases
"int_bounds" -> OP_INTEGRATE  // for rendering
"d_part" -> OP_PARTIAL        // for rendering
```

#### 3. Bidirectional Mapping

```rust
struct SemanticMapping {
    // Visual → Semantic
    visual_patterns: Vec<(Pattern, SemanticOp)>,
    
    // Semantic → Visual (for rendering)
    semantic_to_visual: HashMap<String, VisualTemplate>,
}

impl SemanticMapping {
    fn to_semantic(&self, visual: &Expression) -> Result<Expression, AmbiguityError>;
    fn to_visual(&self, semantic: &Expression) -> Expression;
}
```

#### 4. Ambiguity Resolution

For genuinely ambiguous cases, provide hints:

```rust
enum Hint {
    IsDerivative,      // fg means df/dg, not f*g
    IsSubscriptIndex,  // A_i is indexing, not naming
    IsDifferential,    // dx is differential, not d*x
}

fn to_semantic_with_hints(
    visual: &Expression, 
    hints: &[Hint]
) -> Result<Expression, AmbiguityError>;
```

---

## Implementation Phases

### Phase 1: Extend Template Inference (Low Risk)

Add more patterns to `template_inference.rs`:
- Single definite integrals
- Partial/total derivatives (fraction form)
- Summations with bounds
- Products with bounds
- Limits

**Estimated effort:** 2-3 days
**Risk:** Low (additive change)

### Phase 2: Canonical Names (Medium Risk)

Unify operation names:
- Create mapping between visual and semantic names
- Update Z3 backend to recognize both
- Keep rendering templates working

**Estimated effort:** 1-2 days
**Risk:** Medium (touches multiple files)

### Phase 3: Verify Button in Editor (Integration)

Add UI integration:
- "Verify" button in Equation Editor
- Calls `visual_to_semantic()` 
- Sends to Z3 backend
- Displays result

**Estimated effort:** 1 day
**Risk:** Low (new feature)

### Phase 4: Full Bidirectional (Future)

Complete isomorphism:
- Handle all operation types
- Ambiguity detection and user prompts
- Round-trip testing

**Estimated effort:** 1-2 weeks
**Risk:** Higher (complex patterns)

---

## Specific Translations Needed

### Definite Integral

```
Visual:  sup(sub(Object("\\int"), a), b) * f * mathrm(d) * x
         ↓
Semantic: Integrate(f, x, a, b)
         ↓
Z3:      uninterpreted_function("Integrate", [f, x, a, b])
```

### Partial Derivative

```
Visual:  scalar_divide(
           scalar_multiply(Object("\\partial"), f),
           scalar_multiply(Object("\\partial"), x)
         )
         ↓
Semantic: D(f, x)
         ↓
Z3:      uninterpreted_function("D", [f, x])
```

### Summation

```
Visual:  sup(sub(Object("\\sum"), equals(i, 1)), n) * a_i
         ↓
Semantic: Sum(sub(a, i), i, 1, n)
         ↓
Z3:      uninterpreted_function("Sum", [a_i, i, 1, n])
```

### Limit

```
Visual:  sub(Object("\\lim"), arrow(x, a)) * f
         ↓
Semantic: Limit(f, x, a)
         ↓
Z3:      uninterpreted_function("Limit", [f, x, a])
```

---

## Testing Strategy

### Round-Trip Tests

```rust
#[test]
fn roundtrip_integral() {
    let latex = r"\int_{0}^{1} x^2 \, dx";
    let visual = parse_latex(latex).unwrap();
    let semantic = visual_to_semantic(&visual).unwrap();
    let back_to_visual = semantic_to_visual(&semantic);
    let rendered = render(&back_to_visual, RenderTarget::LaTeX);
    
    // Should produce equivalent LaTeX
    assert_eq!(rendered, r"\int_{0}^{1} x^{2} \, \mathrm{d}x");
}
```

### Verification Integration Tests

```rust
#[test]
fn verify_from_editor() {
    let latex = r"\forall x : x + 0 = x";
    let visual = parse_latex(latex).unwrap();
    let semantic = visual_to_semantic(&visual).unwrap();
    
    let result = verify_with_z3(&semantic);
    assert!(result.is_valid());
}
```

---

---

## Gap 2 Detail: Missing Kleis Calculus Types

### What Exists Today

In `examples/calculus/derivatives.kleis`:

```kleis
structure Differentiable(F) {
    operation D : F -> Variable -> F      // Returns F, not a "Derivative type"
    operation Dt : F -> Variable -> F
    
    axiom D_linear_add: ∀(f g : F, x : Variable). D(f + g, x) = D(f, x) + D(g, x)
    // ... more axioms
}

structure Integrable(F) {
    operation Integrate : F -> Variable -> F   // Returns F, not an "Integral type"
    
    axiom FTC1: ∀(f : F, x : Variable). D(Integrate(f, x), x) = f
}
```

**The limitation:** These define `D` and `Integrate` as *operations* that transform `F → F`.
They don't represent the integral/derivative *itself* as a structured mathematical object.

### What's Missing: First-Class Calculus Types

```kleis
// PROPOSED: Definite Integral as a type
structure DefiniteIntegral(F, var: Variable, lower: ℝ, upper: ℝ) {
    // The integrand
    field integrand : F
    
    // Evaluation (when computable)
    operation evaluate : Self -> ℝ
    
    // Properties
    axiom additivity: ∀(f : F, a b c : ℝ, x : Variable).
        evaluate(DefiniteIntegral(f, x, a, b)) + evaluate(DefiniteIntegral(f, x, b, c)) 
        = evaluate(DefiniteIntegral(f, x, a, c))
    
    axiom reversal: ∀(f : F, a b : ℝ, x : Variable).
        evaluate(DefiniteIntegral(f, x, a, b)) = -evaluate(DefiniteIntegral(f, x, b, a))
    
    axiom FTC2: ∀(f : F, F_antideriv : F, a b : ℝ, x : Variable).
        D(F_antideriv, x) = f ⟹ 
        evaluate(DefiniteIntegral(f, x, a, b)) = F_antideriv(b) - F_antideriv(a)
}

// PROPOSED: Summation as a type  
structure FiniteSum(F, var: Variable, lower: ℤ, upper: ℤ) {
    field body : F
    
    operation evaluate : Self -> ℝ
    
    axiom split: ∀(f : F, i : Variable, a b c : ℤ).
        b < c ⟹
        evaluate(FiniteSum(f, i, a, c)) = 
        evaluate(FiniteSum(f, i, a, b)) + evaluate(FiniteSum(f, i, b+1, c))
}

// PROPOSED: Limit as a type
structure Limit(F, var: Variable, target: ℝ) {
    field expression : F
    
    operation value : Self -> Option(ℝ)  // May not exist
    
    axiom sum_of_limits: ∀(f g : F, x : Variable, a : ℝ).
        value(Limit(f, x, a)) + value(Limit(g, x, a)) = value(Limit(f + g, x, a))
}
```

### Why This Matters for Verification

**Without types:**
```
\int_{0}^{1} x^2 dx  →  ??? (no semantic representation)
                     →  Can't verify "∫₀¹ x² dx = 1/3"
```

**With types:**
```
\int_{0}^{1} x^2 dx  →  DefiniteIntegral(x^2, x, 0, 1)
                     →  Z3 can use axioms to verify properties
                     →  Can verify "evaluate(...) = 1/3"
```

### The Translation Path (with types)

```
LaTeX Input                  Pattern Match              Kleis Type
─────────────────────────────────────────────────────────────────────
\int_{0}^{1} x^2 dx    →    recognize integral    →    DefiniteIntegral(
                                                         x^2, x, 0, 1)

\sum_{i=1}^{n} i^2     →    recognize sum         →    FiniteSum(
                                                         i^2, i, 1, n)

\lim_{x \to 0} sin(x)/x →   recognize limit       →    Limit(
                                                         sin(x)/x, x, 0)

\frac{\partial f}{\partial x} → recognize deriv   →    D(f, x)
                                                       (operation, not type)
```

---

## Revised Solution: Two-Phase Approach

### Phase A: Pattern Matching (Gap 1)

Extend `template_inference.rs` to recognize calculus patterns:

```rust
fn try_infer_definite_integral(expr: &Expression) -> Option<Expression> {
    // \int_{a}^{b} f dx → Operation("DefiniteIntegral", [f, x, a, b])
}
```

**This works now** - we can produce `Operation { name: "DefiniteIntegral", args: [...] }`.

### Phase B: Kleis Type Definitions (Gap 2)

Define the actual types in stdlib:

1. Create `stdlib/calculus.kleis` with:
   - `structure DefiniteIntegral`
   - `structure FiniteSum`
   - `structure Limit`
   - Appropriate axioms

2. Update Z3 backend to handle these types

3. Enable verification of calculus expressions

**This requires new stdlib development.**

---

## Summary

| Current State | Desired State |
|---------------|---------------|
| Two separate AST systems | Unified with translation layer |
| Editor can't verify | Editor can verify via translation |
| Template inference partial | Template inference complete |
| No round-trip guarantee | Bidirectional isomorphism |
| **Calculus as operations only** | **Calculus as first-class types** |
| `Integrate(f, x)` returns `F` | `DefiniteIntegral(f, x, a, b)` IS a type |

**The core insight:** We don't need to change how either parser works internally. We need a **translation layer** that maps between them, with the understanding that some visual patterns are ambiguous and may require hints.

**Recommended starting points:**
1. **Phase A (Quick Win):** Extend `template_inference.rs` to recognize calculus patterns
2. **Phase B (Foundational):** Define calculus types in `stdlib/calculus.kleis`

Phase A gives immediate pattern recognition. Phase B enables true semantic verification.

