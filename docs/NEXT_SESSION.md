# Next Session: Equation Editor & Kleis Grammar Alignment

---
## 🚀 FUTURE VISION: Kleis Notebook (Dec 14, 2025)

The Equation Editor will evolve into a **Kleis Notebook** - a mathematically-aware 
editor that combines:

1. **File Loading** - Load `.kleis` files (axioms, structures, theorems)
2. **File Editing** - Edit `.kleis` files with palette-based math input
3. **Palette Input** - Click symbols (∀, ∂, ∫, Σ, Γ) to build expressions
4. **Verification** - Z3-backed theorem proving against loaded axioms
5. **Notebook Cells** - Multiple expressions, each independently verifiable

### Evolution Roadmap

| Stage | Name | Description |
|-------|------|-------------|
| ✅ Now | Equation Editor | Single expression, render to multiple formats |
| 🔄 Next | Kleis Editor | Load/edit `.kleis` files, palette for axiom authoring |
| 🔮 Future | Kleis Notebook | Multi-cell, proofs, dependency tracking, exports |

### Key Insight (Dec 14, 2025)

Now that `StructureRegistry.load_from_file()` and `Z3Backend.assert_axioms_from_registry()` 
work, the Equation Editor can load user-provided `.kleis` files and verify expressions 
against custom axioms. This is the foundation for the Kleis Notebook.

---
## ⚠️ CRITICAL ARCHITECTURE LESSONS (Dec 14, 2025)

**Read this first before making changes to AST or renderers.**

### Lesson 0: Equation Editor MUST NOT Hang

**The Equation Editor cannot tolerate operations that hang forever.**

When the user clicks "Verify" or "Evaluate", the response must come back in reasonable time.
This means we CANNOT use Z3 with universal quantifier axioms for concrete computation.

| Operation | Approach | Speed | Safe for Editor? |
|-----------|----------|-------|------------------|
| **Evaluation** (compute value) | Inline expansion in Rust | Fast | ✅ YES |
| **Evaluation** (compute value) | Z3 with ∀ axioms | HANGS | ❌ NO |
| **Verification** (sat/unsat) | Z3 with ∀ axioms | Usually fast | ⚠️ With timeout |
| **Satisfiability** (find model) | Z3 with ∀ axioms | Can be slow | ⚠️ With timeout |

**Why Z3 + ∀ axioms hangs on evaluation:**
```
Query: nth([1,2,3], 1) = ?

Axiom: ∀ x . ∀ xs . nth(cons(x, xs), 0) = x
Axiom: ∀ x . ∀ xs . ∀ n . nth(cons(x, xs), n+1) = nth(xs, n)

Z3 E-matching: Must try all instantiations of x, xs, n...
Result: Combinatorial explosion → HANGS FOREVER
```

**Rule:** For concrete computation in the Equation Editor, use **inline expansion** 
(direct Rust computation), NOT axiom-based Z3 reasoning.

Axioms are for VERIFICATION ("is this true?"), not EVALUATION ("what is the value?").

---

### Lesson 1: Two Different ASTs Exist

| AST Type | Created By | Purpose | Format |
|----------|-----------|---------|--------|
| **Editor AST** | `static/index.html` (JavaScript) | Internal representation | Semantic names: `gamma`, `riemann`, `index_mixed` |
| **Kleis AST** | `kleis_parser.rs` (Rust) | Language representation | Grammar-conforming: matches `kleis_grammar_v07.ebnf` |

**They are NOT the same thing.** The Editor AST is internal and can be richer.

### Lesson 2: The Three-Rung Ladder (Data Flow)

```
┌─────────────────────────────────────────────────────────────────┐
│ RUNG 1: Equation Editor (JavaScript)                            │
│   User clicks button → generates Editor AST                     │
│   Example: { name: 'gamma', args: [base, λ, μ, ν] }            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ RUNG 2: Kleis Renderer (Rust: render.rs)                        │
│   Editor AST → visual output (per target)                       │
│   • Typst target: uses `gamma` template → Γ^λ_{μν}             │
│   • LaTeX target: uses `gamma` template → \Gamma^\lambda_{\mu\nu}│
│   • Kleis target: outputs xAct notation → Γ(λ, -μ, -ν)          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ RUNG 3: Kleis Language (grammar, parser, Z3)                    │
│   Kleis text → parsed to Kleis AST → verified/evaluated         │
│   Example: "Γ(λ, -μ, -ν)" parses to function call with negate() │
└─────────────────────────────────────────────────────────────────┘
```

### Lesson 3: Where xAct Notation Belongs

| Component | Uses xAct? | Why |
|-----------|------------|-----|
| Editor AST | ❌ NO | Uses semantic names (`gamma`) for template lookup |
| Kleis Renderer (Kleis target) | ✅ YES | Outputs `Γ(λ, -μ, -ν)` as Kleis text |
| Kleis Parser | ✅ YES | Parses `Γ(λ, -μ, -ν)` as function call |
| Typst/LaTeX templates | ❌ NO | Keyed by semantic name (`gamma`) |

**Mistake made (and fixed):** Changed Editor AST to use `Γ` instead of `gamma` → broke Typst rendering.

### Lesson 4: Changing Editor AST Requires Updating ALL Renderers

If you add metadata to Editor AST (e.g., `kind: 'tensor'`), you MUST update:
- `render.rs` (5 targets: Unicode, LaTeX, HTML, Typst, Kleis)
- `typst_renderer.rs`
- `index.html` (JavaScript AST generation)
- Server API endpoints
- All tests

---

## Priority Work Items

### 1. Three-Rung Ladder Architecture
Clarify and implement the separation between:
- **Rung 1: Equation Editor** - User-facing UI for building mathematical expressions
- **Rung 2: Kleis Renderer** - Visual rendering of Kleis AST to human-readable notation
- **Rung 3: Kleis Language** - The formal language with its grammar and semantics

### 2. Kleis Grammar v0.7 Alignment
- Review `docs/grammar/kleis_grammar_v07.ebnf` ✓ (exists)
- Ensure parser, renderer, and editor all conform to official grammar
- Document any deviations with rationale

### 3. Z3 Backend Testing
- Verify that grammar v0.7 expressions translate correctly to Z3
- Test edge cases: quantifiers, matrices, operations
- Ensure round-trip: Editor → AST → Z3 → Result → Renderer

### 4. Kleis Renderer vs Editor Differences
- What the **Editor** produces (AST from user interaction)
- What the **Renderer** displays (visual representation of AST)
- What **Kleis** accepts (grammar-conforming text/AST)
- Ensure bidirectional consistency: Editor → Kleis → Renderer → Editor

## Context from Previous Session
- Z3 integration complete (verify + satisfiability)
- Matrix operations working
- Power/exponentiation fixed
- Branch: `feature/kleis-renderer`

## Questions to Answer
- [x] Is grammar v0.7 the current official version? **Yes**
- [x] What are the key differences from v0.5? **Mathematica-style calculus: D(), Dt(), Integrate(), Limit()**
- [ ] Where does the 3-rung separation break down currently?

---

## Design Decision: xAct/xTensor-Style Tensor Notation

**Date:** Dec 14, 2025

Kleis will implement **xAct/xTensor-style** tensor notation (from Mathematica's tensor calculus package).

### Syntax:
```kleis
T(μ, -ν)              // T^μ_ν  (positive = contravariant, negative = covariant)
g(-μ, -ν)             // g_μν   (metric tensor)
R(ρ, -σ, -μ, -ν)      // R^ρ_σμν (Riemann tensor)
Γ(λ, -μ, -ν)          // Γ^λ_μν (Christoffel symbol)

// Einstein summation (automatic contraction on repeated indices)
T(μ, -ν) × V(ν)       // = T^μ_ν V^ν (contracts on ν)
```

### Why xAct-style:
- ✅ **No grammar change** - Already parses as function call with unary minus
- ✅ **Simple** - Sign convention is unambiguous (+ = up, - = down)
- ✅ **No backslashes** - Fits Kleis Unicode aesthetic
- ✅ **Proven** - Used by physicists in Mathematica for GR calculations

### Implementation Status (Dec 14, 2025):
1. ✅ **Renderer**: xAct detection implemented - `T(μ, -ν)` displays as `T^μ_ν` (8 tests)
2. ❌ **Type system**: Using generic `Data` types (consistent with matrices, per ADR-021)
3. ✅ **stdlib/tensors.kleis**: Axioms added (metric symmetry, Christoffel symmetry, Riemann antisymmetry)
4. ⏳ **Z3 axiom loading**: Axioms exist in Kleis but Z3 doesn't load them yet (see below)

### Alternatives rejected:
- Cadabra-style `T^{μ}_{ν}` - Requires backslashes, grammar change
- SymPy-style `T(mu, -nu)` - Similar but uses ASCII, less visual

---

## TODO: Equation Editor Tensor Handling

**Problem:** The Equation Editor palette still creates **old template-based** tensors, not xAct-style.

| Palette Button | Current AST | Should be (xAct) |
|----------------|-------------|------------------|
| Mixed tensor | `index_mixed(T, μ, ν)` | `T(μ, -ν)` |
| Metric | `tensor_lower_pair(g, μ, ν)` | `g(-μ, -ν)` |
| Christoffel | `gamma('', λ, μ, ν)` | `Γ(λ, -μ, -ν)` |
| Riemann | `riemann('', ρ, σ, μ, ν)` | `R(ρ, -σ, -μ, -ν)` |

**Challenge:** In xAct-style, the tensor name IS the operation name (dynamic). 
In current templates, the tensor name is a placeholder argument (operation name is fixed).

**Location:** `static/index.html` lines ~2098-2203 (`templateToAST`)

**Options:**
1. Keep both systems (old for editor, xAct for programmatic)
2. Update palette templates to generate xAct-style AST
3. Add tensor-specific palette builder (like matrix builder)

---

## TODO: Z3 Axiom Loading from Kleis Files

**Problem:** Tensor axioms are defined in `stdlib/tensors.kleis` but Z3 doesn't know about them.

**Current state:**
- `load_structure_axioms()` in `src/solvers/z3/backend.rs` is a TODO placeholder
- Z3 treats tensor operations as unconstrained uninterpreted functions
- Tensor symmetry checks return "Satisfiable" with arbitrary assignments

**What was built (Dec 14, 2025):**
1. ✅ `StructureRegistry.load_from_file()` - Parses Kleis files into registry
2. ✅ `StructureRegistry.load_stdlib()` - Loads all stdlib files
3. ✅ `AxiomVerifier` already has `load_axioms_recursive()` that asserts axioms in Z3
4. ⏳ Parser limitation: Doesn't fully support `∀` quantifier syntax in axioms

**Remaining blocker:**
- Parser fails on `∀ var : Type . expr` syntax in axioms
- Error: "Expected '(' after quantifier"
- Once parser is enhanced, axioms will flow from Kleis → Registry → Z3

**Example axiom in Kleis:**
```kleis
axiom metric_symmetric : ∀ g : Tensor(0, 2, dim, ℝ) .
    ∀ μ : Nat . ∀ ν : Nat .
    component(g, μ, ν) = component(g, ν, μ)
```

**Should become in Z3:**
```rust
let metric_sym = forall_const(&[&mu, &nu], &[], &g_mu_nu._eq(&g_nu_mu));
solver.assert(&metric_sym);
```

**Key principle (ADR-015):** Axioms should be defined in Kleis files, NOT hardcoded in Rust.

---

## ✅ FIXED: Editor AST Uses Semantic Names (Dec 14, 2025)

**Issue:** Editor AST was mistakenly changed to xAct-style (wrong layer).

**Resolution:** Reverted tensor templates to use semantic operation names:
- `christoffel` → `gamma` (operation name)
- `riemann` → `riemann` (operation name)
- `tensor_mixed` → `index_mixed`
- etc.

**Correct architecture:**
```
Editor AST: { name: 'gamma', args: [base, upper, lower1, lower2] }
    ↓
Kleis Renderer:
    → Kleis target: "Γ(λ, -μ, -ν)"     ← xAct style output
    → Typst target: uses gamma template
    → LaTeX target: uses gamma template
```

**Key insight:** xAct notation belongs in **Kleis text output**, not internal AST.

---

## Architecture Note: Editor Semantic AST Can Be Richer

**Key insight:** The AST created by the Equation Editor is **internal** - it doesn't need to match Kleis grammar exactly. We can embed additional semantic information.

**Current problem:** 
- `try_render_xact_tensor` uses heuristics (presence of `negate()`) to detect tensors
- This is fragile - we're inferring intent from structure

**Better approach:** Make the Editor AST explicitly semantic:

```javascript
// CURRENT: Implicit tensor (detected by negate pattern)
{ Operation: { name: 'gamma', args: [...] } }

// BETTER: Explicit tensor metadata
{ 
  Operation: { 
    name: 'gamma',
    args: [...],
    metadata: {
      kind: 'tensor',
      symbolicName: 'Γ',
      indexStructure: ['upper', 'lower', 'lower']
    }
  } 
}
```

**Critical constraint:** Any change to the internal AST format must be coordinated across ALL renderers:
- ✅ `render.rs` - LaTeX, Typst, HTML, Unicode, Kleis targets
- ✅ `typst_renderer.rs` - Typst compiler
- ✅ JavaScript in `index.html` - Editor AST generation
- ✅ Server API endpoints - AST serialization/deserialization

**Process for AST changes:**
1. Define the new structure
2. Update ALL renderers to handle it
3. Update Editor to generate it
4. Update tests
5. Ensure backwards compatibility during transition

**Not urgent - current heuristic works. Flagging for future enhancement.**

---

## TODO: Simplify templateMap in Equation Editor (Structural Mode)

**Location:** `static/index.html` lines ~1961-2085 (`templateMap`)

**Problem:** Unnecessary indirection in Structural Mode.

Current flow:
```
Button onclick="insertTemplate('\\frac{□}{□}')" 
  → templateMap['\\frac{□}{□}'] → 'fraction'
  → astTemplates['fraction'] → { Operation: { name: 'scalar_divide', ... } }
```

**Key insight:** In Structural Mode, we know EXACTLY what the user clicked. There is no ambiguity:
- User clicks "Fraction" → they want `scalar_divide`
- User clicks "Christoffel" → they want `Γ(λ, -μ, -ν)`

**The fix is simple:**
```javascript
// BEFORE: Two-step lookup via LaTeX pattern
onclick="insertTemplate('\\frac{□}{□}')"

// AFTER: Direct template name
onclick="insertTemplate('fraction')"
```

Then `insertTemplate()` just does: `astTemplates[name]` - one step.

**Why LaTeX patterns exist at all:**
- Legacy from Text Mode (LaTeX input)
- Text Mode will likely be **deprecated** in favor of Structural Mode
- Once deprecated, `templateMap` can be removed entirely

**Action items:**
1. Change button `onclick` to use template names directly (e.g., `'fraction'`, `'christoffel'`)
2. Update `insertTemplate()` to accept template name (not LaTeX pattern)
3. Keep LaTeX pattern lookup as fallback for Text Mode (until deprecated)
4. Eventually remove `templateMap` entirely

**Not blocking - cosmetic cleanup. Current system works.**

---

## ✅ RESOLVED: Two-AST Architecture (Dec 15, 2025)

**Decision:** Separate Kleis Core AST (`Expression`) from Visual Editor AST (`EditorNode`)

### The Problem (Was)

Initially tried to add `kind` and `metadata` fields to `Expression::Operation`. 
This polluted the Kleis Core AST with Editor-specific concerns.

**Key insight from discussion:**
> "I would want to have a clean AST for the Kleis grammar and structures. If we 
> were to define Kleis AST in Kleis these elements that are related to Equation 
> Editor and not to Kleis core will create some problems at least conceptually."

### The Solution: Two Separate AST Types

| Type | Location | Purpose | Contains |
|------|----------|---------|----------|
| **`Expression`** | `src/ast.rs` | Kleis Core (language semantics) | Pure, grammar-based |
| **`EditorNode`** | `src/editor_ast.rs` | Visual Editor (rendering) | `kind`, `metadata`, display hints |

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Kleis Core AST (Expression)                 │
│  • Pure, grammar-based                                          │
│  • No rendering concerns                                        │
│  • Source: Kleis Parser                                         │
│  • Used for: Verification, Evaluation, Type Checking            │
└─────────────────────────────────────────────────────────────────┘
                              ↓ translate_to_editor()
┌─────────────────────────────────────────────────────────────────┐
│                     Visual AST (EditorNode)                     │
│  • Rich, with rendering metadata                                │
│  • kind, indexStructure, display hints                          │
│  • Source: Editor palette OR translated from Core AST           │
│  • Used for: Visual Rendering, Visual Editing                   │
└─────────────────────────────────────────────────────────────────┘
```

### EditorNode Structure

```rust
// src/editor_ast.rs
pub enum EditorNode {
    Object { object: String },
    Const { value: String },
    Placeholder { placeholder: PlaceholderData },
    Operation { operation: OperationData },
    List { list: Vec<EditorNode> },
}

pub struct OperationData {
    pub name: String,                           // Display symbol: 'Γ', '∫'
    pub args: Vec<EditorNode>,                  // Arguments
    pub kind: Option<String>,                   // 'tensor', 'integral', etc.
    pub metadata: Option<HashMap<String, Value>>, // indexStructure, etc.
}
```

### Translation: Kleis Core → Visual AST

When visually editing parsed Kleis code:

```rust
// src/editor_ast.rs
pub fn translate_to_editor(expr: &Expression) -> EditorNode {
    // 1. Recognize known tensor symbols (Γ, R, g)
    // 2. Infer index structure from negate() wrappers
    // 3. Add kind: 'tensor', metadata: { indexStructure: [...] }
    // ...
}
```

**Example:**
```kleis
Γ(λ, -μ, -ν)  // Kleis text
```
→ Parser → `Expression::Operation { name: "Γ", args: [λ, negate(μ), negate(ν)] }`
→ `translate_to_editor()` →
```rust
EditorNode::Operation {
    name: "Γ",
    args: [...],
    kind: Some("tensor"),
    metadata: Some({ indexStructure: ["up", "down", "down"] })
}
```

### Benefits

1. **Kleis Core stays pure** - Self-hosting ready, no rendering pollution
2. **Editor AST can evolve independently** - Add UI concerns without affecting language
3. **Translation is explicit** - Clear where semantic enrichment happens
4. **One renderer** - Only handles `EditorNode` (not two types)

### Implementation Status

| Step | Status |
|------|--------|
| Create `EditorNode` type | ✅ Done |
| Create `translate_to_editor()` | ✅ Done |
| Update renderer to use `EditorNode` | ✅ Done (mixed-mode) |
| Update server API to accept `EditorNode` | ✅ Done |
| Update palette buttons to generate `EditorNode` | ✅ Done (tensors) |
| Make tensor symbol editable (args[0]) | ✅ Done |

### Files Created/Modified

- ✅ **NEW:** `src/editor_ast.rs` - EditorNode type + translation
- ✅ `src/lib.rs` - Added `editor_ast` module
- ✅ `src/render.rs` - Uses EditorNode (mixed-mode: tensor special, rest via Expression)
- ✅ `src/bin/server.rs` - Handles EditorNode for rendering and type checking
- ✅ `static/index.html` - Tensor palette generates `kind: "tensor"`, `metadata: {indexStructure: [...]}`

### What We're NOT Changing

- `Expression` in `src/ast.rs` - stays clean
- Kleis parser (`kleis_parser.rs`) - untouched
- Kleis grammar (`kleis_grammar_v07.ebnf`) - untouched
- Matrix handling - untouched (it works!)

---

## ✅ RESOLVED: Mixed-Mode Rendering Strategy (Dec 15, 2025)

**Decision:** EditorNode rendering uses Expression renderer for most operations, with special handling only for tensors.

### The Problem

When we created `EditorNode`, we initially tried to create a separate rendering path (`render_editor_node_internal`). This led to:

1. **Ad-hoc fixes** - Adding `{content}`, `{argument}`, `{value}` substitutions one by one
2. **Duplicate code** - Reimplementing matrix handling, piecewise functions, etc.
3. **Breaking changes** - Every palette button broke until we fixed its specific template

### The Solution: EditorNode → Expression → Render

For operations that don't need `EditorNode`-specific metadata:

```rust
fn render_operation(op: &OperationData, ...) -> String {
    match op.kind.as_deref() {
        Some("tensor") => {
            // Tensors NEED indexStructure metadata - special handling
            render_tensor(op, &rendered_args, target)
        }
        _ => {
            // Convert to Expression, use battle-tested renderer
            let expr = editor_node_to_expression(&EditorNode::Operation { operation: op.clone() });
            render_expression_internal(&expr, ctx, target, node_id, node_id_to_uuid)
        }
    }
}
```

### Why This Works

| Operation Type | Loses Fidelity? | Why |
|---------------|-----------------|-----|
| Matrix, sin, parens, etc. | ❌ NO | `kind` = None, `metadata` = None |
| Tensors with `indexStructure` | ⚠️ YES | `metadata` is lost in conversion |

For tensors, we use special handling. For everything else, we convert losslessly.

### Benefits

1. **No code duplication** - Matrix, trig, brackets, etc. use existing renderer
2. **Battle-tested** - Expression renderer handles 100+ templates correctly
3. **Easy to extend** - Just add `kind` cases for operations that need metadata

### Code Removed

- `render_matrix_editor` (98 lines) - was duplicating Expression matrix logic
- `render_integral` - unused placeholder
- `render_derivative` - unused placeholder
- `render_by_template` (131 lines) - incomplete reimplementation

### ⚠️ FUTURE WARNING: translate_to_editor() Complications

When we implement `translate_to_editor(Expression) -> EditorNode` for visual editing of parsed Kleis code, this mixed-mode rendering may cause issues:

1. **Round-trip problem:** 
   - Parse Kleis → `Expression`
   - Translate → `EditorNode` (adds `kind`, `metadata`)
   - Render → calls `editor_node_to_expression()` (loses `kind`, `metadata`)
   - Now we're back to `Expression` anyway!

2. **Double work:**
   - `translate_to_editor()` enriches AST with tensor metadata
   - `render_operation()` converts back to Expression for non-tensors
   - Wasted enrichment

3. **Inconsistent paths:**
   - Tensors: EditorNode → render_tensor (uses metadata)
   - Everything else: EditorNode → Expression → render_expression (ignores metadata)

### Future Option: Full EditorNode Rendering

When ready, we could implement full `EditorNode` rendering:

```rust
// FUTURE: All operations rendered from EditorNode directly
fn render_operation(op: &OperationData, ...) -> String {
    match op.kind.as_deref() {
        Some("tensor") => render_tensor(op, ...),
        Some("integral") => render_integral(op, ...),  // Uses bounds metadata
        Some("matrix") => render_matrix(op, ...),       // Uses dimensions metadata
        _ => render_generic_operation(op, ...)          // Template lookup
    }
}
```

**Not doing this now because:**
- Most operations don't need metadata
- Expression renderer is comprehensive and tested
- Would require porting 500+ lines of template handling

**Flagged for future consideration when we need operation-specific metadata beyond tensors.**

---
*Resolved: Dec 15, 2025*

