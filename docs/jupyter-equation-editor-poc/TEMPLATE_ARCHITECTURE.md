# Template Architecture: Unified in Kleis

**Date:** January 2, 2026  
**Status:** Design  
**Principle:** "Everything is a structure. Templates are structures."

---

## Decision: Unify .kleist into .kleis

**The current .kleist system is unused and can be replaced.**

Instead of maintaining two formats:
- `.kleist` - Templates (separate parser, no verification)
- `.kleis` - Language (full parser, axioms, types)

We unify into ONE format:
- `.kleis` - Templates ARE structures with axioms

---

## The Insight

> "Everybody needs some math at some point!"

Core mathematical notation (fractions, integrals, tensors, Greek letters) is universal. Every domain—physics, music theory, chemistry, finance—builds ON TOP of this foundation.

```
┌─────────────────────────────────────────────────────────────────┐
│                     DOMAIN-SPECIFIC                              │
│  Music: chord symbols, figured bass, counterpoint notation       │
│  Chemistry: reaction arrows, molecular structures                │
│  Finance: currency symbols, accounting notation                  │
│  Physics: Feynman diagrams, circuit elements                     │
└─────────────────────────────────────────────────────────────────┘
                              ↑
                        Built on top of
                              ↑
┌─────────────────────────────────────────────────────────────────┐
│                     CORE MATHEMATICS                             │
│  Fractions, integrals, sums, products, matrices                 │
│  Greek letters, operators, relations                             │
│  Tensors with index structures                                   │
│  54+ templates in render_editor.rs                               │
└─────────────────────────────────────────────────────────────────┘
```

## Hybrid Architecture

### Layer 1: Core Math Templates (Rust)

Located in `src/render_editor.rs`, these are:
- **Performance-optimized** (no evaluator callback)
- **Well-tested** (used by Equation Editor)
- **Multi-target** (Unicode, LaTeX, Typst, HTML, Kleis)

```rust
// Built-in: everyone needs fractions
"frac" => format!("({})/({})", render(args[0]), render(args[1])),

// Built-in: tensors with index structure
"tensor" => render_tensor_with_indices(name, args, metadata),
```

### Layer 2: User Templates (Kleis)

Defined in Kleis structures, following the philosophy:

```kleis
// Domain: Music Theory
structure MusicNotation {
    // Chord symbol: C7, Dm, G#dim
    operation chord(root: Symbol, quality: Symbol)
    
    // Multi-target templates
    template unicode = "${root}${quality}"
    template latex = "\\chord{${root}}{${quality}}"
    template typst = "\"${root}\"#super[${quality}]"
}

// Domain: Chemistry
structure ChemNotation {
    // Reaction arrow: A → B
    operation reaction(reactants: Expr, products: Expr)
    
    template unicode = "${reactants} → ${products}"
    template latex = "${reactants} \\rightarrow ${products}"
    template typst = "${reactants} arrow.r ${products}"
}
```

### Layer 3: The Bridge

The Kleis evaluator registers user templates with the Rust renderer:

```rust
// In evaluator.rs
fn register_structure_templates(structure: &Structure, renderer: &mut Renderer) {
    for operation in &structure.operations {
        if let Some(templates) = &operation.templates {
            renderer.register_user_template(
                &operation.name,
                templates.clone()
            );
        }
    }
}
```

### Lookup Order

When `render_to_typst(node)` is called:

```
1. User templates (from Kleis structures)
   ↓ not found
2. Core math templates (Rust built-ins)
   ↓ not found
3. Default: op_name(arg1, arg2, ...)
```

---

## API Design

### Defining Templates in Kleis

```kleis
// Option A: In structure definition
structure Physics {
    operation christoffel(base, up, lo1, lo2)
    
    // Templates are structure members
    template christoffel.typst = "${base}^(${up})_(${lo1} ${lo2})"
    template christoffel.latex = "${base}^{${up}}_{${lo1}${lo2}}"
}

// Option B: Standalone template definition
template christoffel(base, up, lo1, lo2) {
    unicode: "Γ^${up}_{${lo1}${lo2}}"
    latex: "\\Gamma^{${up}}_{${lo1}${lo2}}"
    typst: "Γ^(${up})_(${lo1} ${lo2})"
}

// Option C: Template as function (maximum flexibility)
define render_christoffel(args: List(EditorNode), target: RenderTarget) : String =
    let base = render_to_target(nth(args, 0), target) in
    let up = render_to_target(nth(args, 1), target) in
    let lo1 = render_to_target(nth(args, 2), target) in
    let lo2 = render_to_target(nth(args, 3), target) in
    match target {
        Typst => concat5(base, "^(", up, ")_(", concat3(lo1, " ", lo2, ")"))
      | LaTeX => concat5(base, "^{", up, "}_{", concat(lo1, lo2, "}"))
      | _ => concat5(base, "^", up, "_", concat(lo1, lo2))
    }
```

### Using Templates

```kleis
// Import user templates
import "physics_templates.kleis"

// Define equation using user template
let gamma = christoffel("Γ", "λ", "μ", "ν")

// Render using hybrid system
let typst_code = render_to_typst(gamma)
// Output: "Γ^(λ)_(μ ν)"

// Core math templates work automatically
let frac = frac(sym("a"), sym("b"))
let typst_code = render_to_typst(frac)
// Output: "(a)/(b)"
```

---

## Implementation Path

### Phase 1: Expose Core Renderer as Builtin
```kleis
// Works with built-in 54+ templates
let typst = render_to_typst(frac(sym("a"), sym("b")))
```

### Phase 2: Template Registration API
```kleis
// Register single template
register_template("myop", 
    typst = "${0} ⊗ ${1}",
    latex = "${0} \\otimes ${1}"
)

// Templates accessible to render_to_typst
let typst = render_to_typst(myop(sym("a"), sym("b")))
```

### Phase 3: Templates in Structures
```kleis
structure MyDomain {
    operation myop(a, b)
    template myop.typst = "${0} ⊗ ${1}"
    template myop.latex = "${0} \\otimes ${1}"
}

// Automatically registered when structure is defined
let typst = render_to_typst(myop(sym("a"), sym("b")))
```

### Phase 4: Template Inheritance
```kleis
// Extend core math with domain-specific
structure Physics extends MathCore {
    operation christoffel(base, up, lo1, lo2)
    template christoffel.typst = "..."
    
    // Inherits all math templates: frac, integral, sum, etc.
}
```

---

## Benefits

1. **Everyone gets math for free** - 54+ core templates
2. **Domains can extend** - Add notation without modifying Rust
3. **Kleis is source of truth** - Templates are code, version-controlled
4. **Performance where needed** - Core templates are Rust, fast
5. **Flexibility where needed** - User templates can be complex Kleis functions

---

## Unified Design: Templates as Structures

### Before (.kleist - DEPRECATED)

```kleist
@template christoffel {
    pattern: "christoffel(base, up, lo1, lo2)"
    unicode: "Γ^{up}_{lo1 lo2}"
    latex: "\\Gamma^{{up}}_{{{lo1}{lo2}}}"
    typst: "Γ^({up})_({lo1} {lo2})"
    category: "tensors"
}
```

### After (.kleis - UNIFIED)

```kleis
structure TensorTemplates {
    // Operations with types
    operation christoffel(base: Symbol, up: Index, lo1: Index, lo2: Index) : Tensor
    
    // Multi-target rendering
    template christoffel {
        unicode = "{base}^{up}_{lo1 lo2}"
        latex = "{base}^{{{up}}}_{{{lo1}{lo2}}}"
        typst = "{base}^({up})_({lo1} {lo2})"
    }
    
    // Axioms for verification!
    axiom symmetry_lower : ∀ base up lo1 lo2 . 
        christoffel(base, up, lo1, lo2) = christoffel(base, up, lo2, lo1)
}
```

---

## Migration Plan

### Phase 1: Define Template Syntax in Kleis Grammar

Add to `kleis_grammar.ebnf`:

```ebnf
template_def = "template" identifier "{" template_body "}" ;
template_body = (target_def)+ ;
target_def = ("unicode" | "latex" | "typst" | "html") "=" string ;
```

### Phase 2: Migrate std_template_lib/ to stdlib/templates/

| Old (.kleist) | New (.kleis) |
|---------------|--------------|
| `std_template_lib/basic.kleist` | `stdlib/templates/basic.kleis` |
| `std_template_lib/tensors.kleist` | `stdlib/templates/tensors.kleis` |
| `std_template_lib/calculus.kleist` | `stdlib/templates/calculus.kleis` |
| ... | ... |

### Phase 3: Update Renderer to Query Structures

```rust
// Old: Load .kleist files
let templates = kleist_parser::parse_kleist_file("tensors.kleist")?;

// New: Query loaded Kleis structures
let templates = evaluator.get_structure_templates("TensorTemplates")?;
```

### Phase 4: Deprecate kleist_parser.rs

- Remove `src/kleist_parser.rs` (900+ lines)
- Remove `.kleist` grammar
- All templates defined in `.kleis` files

---

## Benefits of Unification

| Feature | .kleist (Old) | .kleis (New) |
|---------|---------------|--------------|
| Parser | Separate | Unified |
| Types | No | Yes |
| Axioms | No | Yes |
| Verification | No | Z3 |
| Imports | Limited | Full |
| Extensibility | Manual | Inheritance |
| Version Control | Yes | Yes |
| Documentation | Manual | Self-documenting |

---

## Example: Complete Math Library

```kleis
// stdlib/templates/math.kleis

structure BasicMath {
    operation plus(a: Expr, b: Expr) : Expr
    operation minus(a: Expr, b: Expr) : Expr
    operation times(a: Expr, b: Expr) : Expr
    operation frac(num: Expr, denom: Expr) : Expr
    
    template plus { typst = "{a} + {b}" }
    template minus { typst = "{a} - {b}" }
    template times { typst = "{a} {b}" }
    template frac { typst = "({num})/({denom})" }
    
    axiom plus_commutative : ∀ a b . plus(a, b) = plus(b, a)
    axiom times_commutative : ∀ a b . times(a, b) = times(b, a)
}

structure Calculus extends BasicMath {
    operation integral(lower: Expr, upper: Expr, body: Expr, var: Symbol) : Expr
    operation derivative(f: Expr, var: Symbol) : Expr
    
    template integral { 
        typst = "integral_{lower}^{upper} {body} d {var}" 
        latex = "\\int_{{lower}}^{{upper}} {body} \\, d{var}"
    }
    
    axiom fundamental_theorem : ∀ f a b . 
        integral(a, b, derivative(f, x), x) = minus(f(b), f(a))
}

structure TensorCalculus extends Calculus {
    operation christoffel(base: Symbol, up: Index, lo1: Index, lo2: Index) : Tensor
    operation riemann(base: Symbol, up: Index, lo1: Index, lo2: Index, lo3: Index) : Tensor
    
    template christoffel { typst = "{base}^({up})_({lo1} {lo2})" }
    template riemann { typst = "{base}^({up})_({lo1} {lo2} {lo3})" }
    
    axiom bianchi_identity : ∀ R μ ν ρ σ .
        plus(plus(covariant_derivative(R, ρ, μ, ν, σ), 
                  covariant_derivative(R, σ, μ, ν, ρ)),
             covariant_derivative(R, ν, μ, σ, ρ)) = 0
}
```

---

## Open Questions

1. **Template syntax in grammar** - Exact syntax for multi-target templates?
2. **Argument binding** - `{a}` vs `${a}` vs pattern matching?
3. **Palette/UI info** - How to specify glyphs, shortcuts, categories?
4. **Tool builders** - How to define interactive tools (matrix builder)?

