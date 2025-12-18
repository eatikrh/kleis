# Next Session: PatternFly Equation Editor Migration

---

## 🎯 IMMEDIATE TASK: PatternFly Equation Editor (Dec 15, 2025)

### The Goal

Create a PatternFly/React version of the Equation Editor while keeping `static/index.html` intact as the reference implementation.

### Strategy: Web of Trust for Migration

```
┌─────────────────────────────────────────────────────────────┐
│  Reference Implementation (static/index.html)               │
│  ├─ 3,860 lines vanilla JS                                  │
│  ├─ Known working state                                     │
│  └─ FROZEN - do not modify during migration                 │
└─────────────────────────────────────────────────────────────┘
                              ↕  Compare outputs
┌─────────────────────────────────────────────────────────────┐
│  New Implementation (patternfly-editor/)                    │
│  ├─ React + PatternFly                                      │
│  ├─ Component-based architecture                            │
│  └─ Built incrementally, verified against reference         │
└─────────────────────────────────────────────────────────────┘
```

### Proposed Directory Structure

```
kleis/
├── static/index.html          ← FROZEN reference
├── patternfly-editor/         ← NEW React app
│   ├── package.json
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── Palette/
│   │   │   │   ├── PaletteTabs.tsx
│   │   │   │   ├── PaletteButton.tsx
│   │   │   │   └── buttonConfigs.ts    ← Data from astTemplates
│   │   │   ├── Editor/
│   │   │   │   ├── StructuralEditor.tsx
│   │   │   │   ├── InlineEditor.tsx
│   │   │   │   └── SvgOverlay.tsx
│   │   │   └── Preview/
│   │   │       └── MathPreview.tsx
│   │   ├── hooks/
│   │   │   ├── useAST.ts
│   │   │   ├── useTypeCheck.ts
│   │   │   └── useUndoRedo.ts
│   │   └── api/
│   │       └── kleis.ts          ← Same API calls to Rust backend
│   └── tests/
│       └── comparison.test.ts    ← Verify outputs match reference
```

### Verification Checklist

| Test | Reference Output | New Implementation |
|------|------------------|-------------------|
| Click "fraction" button | `{Operation: {name: 'scalar_divide', ...}}` | Same AST |
| Click "Christoffel" button | `{Operation: {name: 'tensor', kind: 'tensor', ...}}` | Same AST |
| Render 2×2 matrix | SVG with placeholders | Identical SVG |
| Type check Γ tensor | `Tensor(1, 2, dim, ℝ)` | Same type result |
| Fill placeholder with "α" | Green box, value updated | Same behavior |
| Undo/Redo | Stack works correctly | Same behavior |

### Milestones

| Milestone | Description | Verification |
|-----------|-------------|--------------|
| **M1: Scaffold** | PatternFly app renders, connects to API | API calls work |
| **M2: One Button** | Fraction button works | AST matches reference |
| **M3: Palette Tabs** | All tabs render | Visual parity |
| **M4: All Buttons** | All 54+ templates work | All ASTs match |
| **M5: SVG Rendering** | Typst SVG displays | Identical output |
| **M6: Overlays** | Clickable markers work | Same UX |
| **M7: Inline Editor** | Type in placeholders | Same behavior |
| **M8: Type Checking** | Live type feedback | Same results |
| **M9: Undo/Redo** | History works | Same behavior |
| **M10: Parity** | Feature-complete | Ready to replace |

### Benefits of React/PatternFly

1. **Component Testing** - Safety net for visual bugs (currently missing)
2. **Flexible Tabs** - Move buttons between tabs = move line in array
3. **State Management** - Clean, predictable, debuggable
4. **Design System** - Professional UX out of the box
5. **Future: Kleis Notebook** - Multi-cell support becomes feasible

### Branch

```
feature/patternfly-editor
```

### First Session Tasks

1. [ ] Create `patternfly-editor/` directory
2. [ ] Initialize React + TypeScript + PatternFly
3. [ ] Create basic App component with header
4. [ ] Add one palette button (fraction)
5. [ ] Verify AST output matches `static/index.html`
6. [ ] Commit: "feat: PatternFly editor scaffold with first button"

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

## BUG: Tensor Indices Become All Contravariant in Equations (Dec 16, 2025)

**Symptom:** When a tensor is part of an equation (e.g., `□ = R^{μνρσ}`), all indices render as upper (contravariant), even though `EditorNode` contains correct `indexStructure: ['up', 'down', 'down', 'down']`.

**Works:** Tensor alone in AST renders correctly with mixed indices.
**Broken:** Tensor nested inside `equals` operation loses index variance.

**Root Cause:** In `src/render.rs:6664-6675`, `render_operation` handles tensors specially, BUT for non-tensor operations (like `equals`), it converts children to `Expression` via `editor_node_to_expression`, which **loses** the `kind` and `metadata` fields.

```rust
// This loses tensor metadata!
let expr = Expression::Operation {
    name: op.name.clone(),
    args: op.args.iter().map(editor_node_to_expression).collect(),  // ← metadata lost here
};
```

**Proper Fix:** Full EditorNode rendering architecture:
1. Make `render_editor_node_internal` handle ALL operations directly
2. Never convert EditorNode to Expression for rendering
3. Expression rendering should convert TO EditorNode first, or EditorNode should be primary

**Scope:** ~500-1000 lines refactor of render.rs
**Workaround:** None (tensors in equations will show all upper indices)

**Files:**
- `src/render.rs` - `render_operation`, `editor_node_to_expression`
- `src/editor_ast.rs` - `EditorNode`, `OperationData` (has `kind`, `metadata`)

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
pub fn translate_to_editor(expr: &Expression, type_ctx: &TypeContext) -> EditorNode {
    // 1. Query TypeContext for the type of this expression
    // 2. If type is Tensor(up, down, dim), add kind: 'tensor' and derive indexStructure
    // 3. Infer index positions from negate() wrappers (covariant indices)
    // Note: No Greek symbol heuristics needed - the type system knows what's a tensor
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

---
## 📋 TODO: Parser Feature Gaps (Dec 15, 2025)

The current Kleis parser implements ~30% of the v0.7 grammar. Here are the notable gaps:

### Missing Top-Level Declarations

| Feature | Grammar v0.7 | Parser | Notes |
|---------|--------------|--------|-------|
| `import` / `include` | ❌ Not in grammar | ❌ Not implemented | **Priority: HIGH** - need for modular files |
| Top-level `axiom` | ✅ | ❌ | Axioms only work inside structures |
| Top-level `let` | ✅ | ❌ | Let bindings only in expressions |
| Top-level `verify` | ✅ | ❌ | Verification statements |

### Comment Syntax Discrepancy

| Style | Grammar | Parser |
|-------|---------|--------|
| `-- comment` | ✅ Defined | ❌ Not recognized |
| `// comment` | ✅ Defined | ✅ Works |
| `/* block */` | ✅ Defined | ✅ Works |

**Action:** Examples should use `//` comments for parser compatibility.

### What Works

| Feature | Status |
|---------|--------|
| `structure` with `element`, `operation`, `axiom` | ✅ |
| Parameterized structures `(n: Nat, T: Type)` | ✅ |
| `data` type definitions with variants | ✅ |
| `implements` blocks | ✅ |
| Top-level `operation` declarations | ✅ |
| `define` function definitions | ✅ |

### Missing Pattern Features

| Pattern | Example | Status |
|---------|---------|--------|
| Wildcard | `_` | ✅ Works |
| Variable | `x`, `head` | ✅ Works |
| Constructor | `Some(x)`, `Cons(h, t)` | ✅ Works |
| Nested | `Ok(Some(x))` | ✅ Works |
| Constant | `0`, `42` | ✅ Works |
| **As-pattern** | `Cons(h, t) as whole` | ❌ NOT IMPLEMENTED |
| **Pattern guard** | `x if x < 0 => ...` | ❌ NOT IMPLEMENTED |
| **Let destructuring** | `let Point(x, y) = p in ...` | ❌ NOT IMPLEMENTED |

---

**Pattern guards** allow conditional matching beyond structure:
- Haskell: `x | x < 0 -> "negative"`
- Rust: `x if x < 0 => "negative"`
- OCaml: `x when x < 0 -> "negative"`

**Use case:**
```kleis
define sign(n) =
    match n {
        x if x < 0 => "negative"
        x if x > 0 => "positive"  
        _ => "zero"
    }
```

**To implement:**
1. Add `guard: Option<Expression>` field to `MatchCase` in `src/ast.rs`
2. After parsing pattern, check for `if` keyword before `=>`
3. If found, parse guard expression
4. Update evaluator: check guard after pattern matches, before executing body

**Current workaround:** Use nested if-then-else:
```kleis
define sign(n) =
    if n < 0 then "negative"
    else if n > 0 then "positive"
    else "zero"
```

---

**Let destructuring** allows pattern matching in let bindings:
- Haskell: `let (x, y) = point in ...`
- Rust: `let Point { x, y } = point;`
- OCaml: `let (x, y) = point in ...`

**Use cases:**
```kleis
define distance_squared(origin) =
    let Point(x, y) = origin in x^2 + y^2

define sum_first_two(triple) =
    let (first, second, _) = triple in first + second
```

**To implement:**
1. Change `Let.name: String` to `Let.pattern: Pattern` in `src/ast.rs`
2. Update `parse_let_binding()` to call `parse_pattern()` instead of `parse_identifier()`
3. Update evaluator: match value against pattern, bind all extracted variables
4. Type inference: infer types for all bound variables from pattern structure

**Current workaround:** Use explicit match:
```kleis
define distance_squared(origin) =
    match origin {
        Point(x, y) => x^2 + y^2
    }
```

---

**As-pattern (alias binding)** is a common feature in functional languages:
- Haskell: `list@(x:xs)` 
- Rust: `list @ [head, ..]`
- OCaml: `(head :: tail) as list`

**Use case:**
```kleis
define filter_head(list) =
    match list {
        Cons(h, t) as whole =>   // Bind 'whole' to the entire list
            if h > 10 then whole  // Return entire list unchanged
            else t                // Or return just the tail
        Nil => Nil
    }
```

**To implement:**
1. Add `As { pattern: Box<Pattern>, binding: String }` to `Pattern` enum in `src/ast.rs`
2. After parsing a pattern in `parse_pattern()`, check for `as` keyword + identifier
3. Update evaluator's pattern matcher to bind both destructured parts AND the alias

### Next Steps

1. **Add `import`/`include` support** - Allow loading other .kleis files
2. **Add `--` comment support** - Match grammar specification
3. **Add top-level `axiom`** - For standalone axiom declarations
4. **Add top-level `let`/`verify`** - For example files and notebooks
5. **Add `as` pattern support** - Alias binding in pattern matching
6. **Add pattern guards** - Conditional matching (`x if x < 0 => ...`)
7. **Add let destructuring** - Pattern matching in let bindings (`let Point(x, y) = p in ...`)

---
*Noted: Dec 15, 2025*
*Updated: Dec 18, 2025*

---

## 📋 ROADMAP: Kleis Notebook Prerequisites (Dec 15, 2025)

### The Dependency Chain

```
┌─────────────────────────────────────────────────────────────┐
│  PatternFly Equation Editor                                 │ ← FIRST
│  • React component architecture                             │
│  • Proper state management                                  │
│  • Testable components (safety net for visual bugs)         │
│  • Flexible palette organization                            │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Complete AST Separation                                    │
│  • Full EditorNode renderer (no dispatch to Expression)     │
│  • Robust translate_to_editor() for parsed Kleis            │
│  • translate_to_expression() for verification               │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Complete Equation Editor Palette                           │
│  • Functions: f(x), λ x . body                              │
│  • Let bindings: let x = ... in ...                         │
│  • Conditionals: if/then/else                               │
│  • Quantifiers: ∀, ∃                                        │
│  • Sets: { x | condition }                                  │
│  • Pattern matching: match ... with ...                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Structure Editors (NEW)                                    │
│  • structure { ... } editor                                 │
│  • implements { ... } editor                                │
│  • data Type = ... editor                                   │
│  • operation name : Type editor                             │
│  • axiom name : Expression editor                           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Kleis Notebook                                             │ ← GOAL
│  • Multi-cell document                                      │
│  • Cell types: Expression, Structure, Verify, Output        │
│  • Dependency tracking between cells                        │
│  • Export to .kleis files                                   │
└─────────────────────────────────────────────────────────────┘
```

### Why PatternFly Migration is Critical

**Current state (vanilla JS):**
```
static/index.html
├─ 3,860 lines of vanilla JavaScript
├─ DOM manipulation everywhere
├─ State scattered across global variables
├─ No component reuse
├─ Fragile event handling
└─ Moving buttons between tabs = edit HTML + handlers + test
```

**Target (PatternFly/React):**
```jsx
// Palette buttons as data
const tensorButtons = [
  { symbol: 'Γ', name: 'christoffel', kind: 'tensor', indices: ['up','down','down'] },
  { symbol: 'R', name: 'riemann', kind: 'tensor', indices: ['up','down','down','down'] },
  { symbol: 'g', name: 'metric', kind: 'tensor', indices: ['down','down'] },
];

// Tabs are just configuration
<PaletteTabs>
  <Tab title="Tensors" buttons={tensorButtons} />
  <Tab title="Physics" buttons={physicsButtons} />
</PaletteTabs>

// Moving a button = move one line in the array
```

### ⚠️ Equation Editor Fragility Warning

**From this session (Dec 15, 2025):**

The Equation Editor is a **high-coupling, low-tolerance system**:

```
Templates ←→ Renderer ←→ Typst ←→ SVG extraction
    ↕            ↕          ↕           ↕
Placeholders ←→ UUIDs ←→ Coordinates ←→ Overlays
    ↕            ↕          ↕           ↕
Click handlers ←→ Inline editor ←→ AST updates

Change one thing → 5 things break
```

**What happened this session:**
- Changed tensor structure → broke `parens`, `sin`, `Matrix`
- Fixed rendering → broke coordinate extraction
- Fixed coordinates → edit markers shifted
- Fixed markers → piecewise builder cleared expressions

**The prudent approach:**

| ❌ Don't | ✅ Do |
|----------|------|
| "Let's overhaul the AST" | One palette button at a time |
| "Add all missing operations" | Add one, test thoroughly |
| "Refactor rendering" | Surgical changes with full test coverage |
| Big bang changes | Incremental with verification at each step |

**Key insight:** Visual bugs have no automated safety net (unlike parser/type errors).
The "web of trust" doesn't fully cover the UI layer yet. PatternFly + React component
testing will provide that missing safety net.

### Proposed Tab Structure (Post-PatternFly)

| Tab | Contents |
|-----|----------|
| **Tensors** | Γ, R, g, generic tensor, index operations |
| **Matrices** | Matrix builder, determinant, trace, inverse |
| **Calculus** | ∂, ∫, Σ, Π, limits |
| **Functions** | sin, cos, exp, log, custom f(x) |
| **Logic** | ∀, ∃, ∧, ∨, →, = |
| **Physics** | ℏ, c, α, Schrödinger, Dirac |
| **Greek** | α, β, γ, ... (input helpers) |

Users could customize their own tabs (not feasible with vanilla JS).

---

## 🔧 Type Inference Issues (Dec 17, 2025)

### Issue: Comparison Operators Not Returning Bool

**Found during render_editor.rs testing:**

When using comparison operators (`neq`, `less_than`, etc.) in expressions like:
```
x ≠ Γ^λ_{μν}
```

The type checker returns the **RHS type** (Tensor) instead of **Bool**.

**Root cause:** In `src/type_context.rs`:
```rust
// Line 725 - matches "not_equals" but NOT "neq"
"equals" | "not_equals" => {
    Ok(arg_types[1].clone())  // Returns RHS type, not Bool!
}
```

**Problems:**
1. `"neq"` is not in the match pattern (only `"not_equals"`)
2. Even when matched, returns RHS type instead of `Bool`
3. No type mismatch error when comparing scalar `x` with tensor `Γ`

**Why this matters for Equation Editor:**
- Piecewise functions show "if" before conditions
- "if x < 0" implies what follows should be a **Boolean expression**
- Type checker should validate this semantic constraint
- Currently: `if x ≠ Tensor` is accepted when it should be an error

**Operations affected:**
- `neq` (inequality)
- `less_than`, `greater_than` (should return Bool)
- `logical_and`, `logical_or` (should require Bool args)
- Any comparison used in Piecewise conditions

### Issue: Arithmetic on Incompatible Types

**Found during testing:**

`1 + Γ^λ_{μν}` (scalar + tensor) is accepted with inferred type `Tensor(1, 2, dim, ℝ)`.

**Expected behavior:** Type error - cannot add scalar to tensor without explicit broadcasting.

### Issue: Matrix Multiply Type Inference Fails (REGRESSION)

**Found during Equation Editor testing:**

```
✗ Operation 'multiply' found in structure 'MatrixMultipliable' but type inference failed: 
  Unbound parameter: m
```

### Issue: Negate Type Inference Fails

**Found during Equation Editor testing (Dec 17, 2025):**

```
✗ Operation 'negate' found in structure 'Ring' but type inference failed: 
  Operation 'negate' not found in structure 'Ring'
```

**Note:** The error message is contradictory - says operation is found but then says not found.
This suggests a bug in the type inference error reporting or structure traversal.

**This is likely a REGRESSION** - the type checker was previously loading stdlib and matrices
correctly. Something in the render_editor.rs changes may have broken the type context loading.

**Symptoms:**
- Operations like `multiply` are "found" in structures but fail type inference
- Many operations show "Unknown operation" despite being in stdlib
- Type parameters (`m`, `n`) are reported as unbound

**Investigation needed:**
- Check if server.rs TypeChecker initialization changed
- Verify stdlib files are being parsed and loaded correctly
- Compare with working state before render_editor.rs refactoring

### Issue: Palette Button Generates Wrong AST

**Found during Equation Editor testing:**

The "logical not" palette button generates:
```json
{"Operation": {"name": "minus", "args": [{"Const": "0"}, ...]}}
```

Instead of:
```json
{"Operation": {"name": "logical_not", "args": [...]}}
```

**Location:** `static/index.html` - palette button configurations

**Buttons needing fixes:**
1. ~~"Logical not" - generates `minus(0, x)` instead of `logical_not(x)`~~ - **FIXED**: Was already correct
2. ~~"Arithmetic negation" - generates `minus(0, x)` instead of `negate(x)`~~ - **FIXED**: Updated `static/index.html` line 2132

**Status:** Both palette buttons now generate correct AST operations

**Operations affected:**
- `plus` - should require compatible types
- `minus` - same issue
- `multiply` - scalar × tensor is valid, but tensor × scalar positioning matters
- Any binary arithmetic operation

**Proposed fix:**
```rust
// Add neq to the match
"equals" | "not_equals" | "neq" => {
    self.check_binary_args(op_name, arg_types)?;
    // For comparisons, verify types are compatible
    // Then return Bool, not RHS type
    Ok(Type::Bool)
}
```

**Note:** This is a semantic design question - `equals` serves double duty:
- **Definition:** `x = 5` means x has type of 5
- **Assertion:** `x = 5` is a Bool predicate

The type system currently assumes definition semantics. For Piecewise conditions,
we need assertion/predicate semantics (returns Bool).

### Issue: Missing Operations in Stdlib

**Found during Equation Editor testing:**

Many operations have rendering templates but are not defined in stdlib, causing type checker warnings:

```
✗ Unknown operation: 'norm'
Hint: This operation is not defined in any loaded structure.
```

**Operations missing from stdlib (have render templates but no type definitions):**
- `norm` - vector/matrix norm → should return Scalar
- `abs` - absolute value → should return Scalar  
- `floor`, `ceiling` - rounding → should return Scalar (or same type as input)
- `binomial` - binomial coefficient → should return Nat
- `nth_root` - nth root → should return Scalar
- `factorial` - factorial → should return Nat
- `dot_accent`, `ddot_accent`, `bar`, `hat`, `tilde`, `overline` - accents (physics notation)
- `sum_bounds`, `prod_bounds`, `int_bounds` - bounded summation/product/integral
- `lim`, `d_dt`, `d_part` - limits and derivatives
- `kernel_integral`, `convolution`, `greens_function` - advanced integrals and transforms
- `cross`, `dot` - vector operations
- Various quantum operators (`ket`, `bra`, `inner`, `outer`, etc.)
- Various trig/calculus (`arcsin`, `arccos`, `arctan`, `sinh`, `cosh`, `tanh`, etc.)

**Impact:**
- Rendering works (templates exist in render_editor.rs)
- Type checking fails (operations not in stdlib structures)
- Users see warning but equation still renders

**Operations missing render templates in render_editor.rs (Typst compile fails):**
- `ket` - Dirac ket notation |ψ⟩ → needs template like `lr(| {arg} angle.r)`
- `bra` - Dirac bra notation ⟨ψ| → needs template like `lr(angle.l {arg} |)`
- Other quantum operations may also be missing

**Fix needed:**
Add these operations to appropriate stdlib files:
- `stdlib/vectors.kleis` - norm, cross, dot
- `stdlib/scalars.kleis` - abs, floor, ceil, nth_root
- `stdlib/combinatorics.kleis` - binomial, factorial
- `stdlib/quantum.kleis` - ket, bra, inner, outer, etc.

---

## ✅ render_editor.rs Implementation (Dec 17, 2025)

### What Was Done

Created `src/render_editor.rs` - a pure EditorNode renderer that fixes the tensor
index bug by never converting EditorNode to Expression.

**Branch:** `refactor/editor-node-rendering`

**The Bug Fixed:**
```
Before: Tensor inside equals → R^{μ ν ρ} = 0 (all upper - WRONG)
After:  Tensor inside equals → R^{μ}_{ν ρ} = 0 (correct indices)
```

**Key Changes:**
1. `src/render_editor.rs` - New 2000+ line renderer
2. `src/bin/server.rs` - Wired to use render_editor for /api/render_ast
3. `src/math_layout/typst_compiler.rs` - Wired to use render_editor for Typst

**Templates Added (80+):**
- All comparison operators (lt, gt, leq, geq, neq, less_than, greater_than)
- All logical operators (and, or, not, logical_and, logical_or, logical_not)
- Quantum operators (ket, bra, inner, outer, commutator, expectation)
- Vectors (vector_bold, vector_arrow, dot, cross, norm)
- Calculus (sqrt, int_bounds, sum_bounds, prod_bounds, lim, d_dt, d_part)
- Transforms (fourier, laplace, inverse variants, convolution)
- Accents (dot_accent, ddot_accent)
- And more...

**Status:** Functional, under testing. Not yet merged to main.

### Future: User-Defined Templates

Currently `render_editor.rs` has all 80+ templates hardcoded in `load_templates()`. For user-defined 
notations (per ADR-009 and KLEIS_ECOSYSTEM_TOOLBOXES.md), we need dynamic template loading.

**Vision:**
```kleis
// User defines in my-algebra.kleis:
operation tensor_contract : (Tensor, Index, Index) -> Tensor
template tensor_contract {
    glyph: "⊗",
    latex: "{tensor}^{{upper}}_{{{lower}}}",
    typst: "{tensor}^({upper})_({lower})",
    unicode: "{tensor}^{upper}_{lower}"
}
```

**What `render_editor.rs` needs:**

| Current | Needed |
|---------|--------|
| `EditorRenderContext::new()` creates fixed templates | `EditorRenderContext::from_registry(registry)` loads from registry |
| Templates hardcoded in Rust | Templates in `.kleis` files, parsed at load time |
| Palette buttons hardcoded in HTML/TS | Palette auto-generated from template metadata |

**The bridge (future implementation):**
```rust
impl EditorRenderContext {
    pub fn from_template_registry(registry: &TemplateRegistry) -> Self {
        let mut ctx = EditorRenderContext::empty();
        for (name, metadata) in registry.iter() {
            ctx.add_template(
                &name,
                &metadata.unicode, &metadata.latex,
                &metadata.html, &metadata.typst, &metadata.kleis,
            );
        }
        ctx
    }
}
```

**Good news:** The HashMap-based structure in `EditorRenderContext` is already right. We just need:
1. Parse `@template` blocks from `.kleist` files
2. Populate a `TemplateRegistry`
3. Pass registry to `EditorRenderContext` at startup

### File Extensions

| Extension | Purpose | Example |
|-----------|---------|---------|
| `.kleis` | Kleis programs (structures, axioms, proofs) | `stdlib/ring.kleis` |
| `.kleist` | Templates AND palette layout | `std_template_lib/calculus.kleist` |

**Why `.kleist`?**
- Clearly related to "Kleis" (kleis + t)
- Not used by any existing software
- Memorable, sounds like a word
- Literary connection: Heinrich von Kleist (German dramatist)

**Block types within `.kleist` files:**
- `@template` - Rendering templates (how to display operations)
- `@palette` - UI layout (which buttons, which tabs, what order)

### Standard Template Library Structure

```text
std_template_lib/
├── basic.kleist        # Templates: +, -, ×, ÷, =, ^, _
├── calculus.kleist     # Templates: ∫, Σ, Π, lim, d/dx
├── quantum.kleist      # Templates: |ψ⟩, ⟨φ|, [A,B]
├── tensors.kleist      # Templates: Γ, R, g indices
├── transforms.kleist   # Templates: ℱ, ℒ, convolution
├── pot.kleist          # Templates: Π, modal space
└── palette.kleist      # Palette layout (tabs, groups, order)
```

**Combined example (`calculus.kleist`):**
```kleist
// ============ TEMPLATES ============

@template integral {
    pattern: int_bounds(integrand, from, to, variable)
    unicode: "∫_{from}^{to} {integrand} d{variable}"
    latex: "\\int_{{{from}}}^{{{to}}} {integrand} \\, \\mathrm{d}{variable}"
    typst: "integral_({from})^({to}) {integrand} dif {variable}"
}

@template derivative {
    pattern: d_dt(function, variable)
    unicode: "d{function}/d{variable}"
    latex: "\\frac{d\\,{function}}{d{variable}}"
    typst: "(d {function})/(d {variable})"
}

@template partial {
    pattern: d_part(function, variable)
    unicode: "∂{function}/∂{variable}"
    latex: "\\frac{\\partial\\,{function}}{\\partial {variable}}"
    typst: "(diff {function})/(diff {variable})"
}
```

**Palette layout (`palette.kleist`):**
```kleist
@palette {
    tab "Basics" {
        group "Arithmetic" {
            plus
            minus
            multiply
            divide
            power
        }
        
        group "Comparison" {
            equals
            lt  gt
            leq geq
            neq
        }
        
        separator
        
        group "Brackets" {
            parens
            brackets
            braces
            abs
            norm
        }
    }
    
    tab "Calculus" {
        group "Derivatives" {
            derivative    shortcut: "Ctrl+D"
            partial       shortcut: "Ctrl+Shift+D"
        }
        
        group "Integrals" {
            integral      shortcut: "Ctrl+I"
        }
        
        group "Limits & Sums" {
            lim
            sum_bounds
            prod_bounds
        }
    }
    
    tab "Quantum" {
        ket           shortcut: "Ctrl+K"
        bra
        inner
        outer
        commutator
        expectation
    }
    
    tab "POT" {
        projection
        modal_integral
        causal_bound
        hont
    }
}
```

**`render_editor.rs` becomes thin:**
```rust
impl EditorRenderContext {
    pub fn from_std_template_lib(lib: &StdTemplateLib) -> Self {
        let mut ctx = EditorRenderContext::empty();
        for template in lib.templates() {
            ctx.add_template(/* from parsed template */);
        }
        ctx
    }
}
```

**Related docs:**
- ADR-009: WYSIWYG Structural Editor (glyph/template specs)
- KLEIS_ECOSYSTEM_TOOLBOXES.md (template externalization)
- docs/archive/template-implementation-strategy.md (detailed plan)

---

## 📚 Documentation vs Reality Gaps (Dec 18, 2025)

### ⚠️ REVIEW NEEDED: Pattern Matching Chapter

**File:** `docs/manual/src/chapters/05-pattern-matching.md`

This chapter needs strict review against actual implementation. Many features commonly shown in pattern matching tutorials are NOT implemented in Kleis:

| Feature | Documented? | Implemented? |
|---------|-------------|--------------|
| Basic patterns (`_`, `x`, `Cons(h,t)`) | ✅ | ✅ |
| Nested patterns | ✅ | ✅ |
| Exhaustiveness checking | ✅ | ✅ |
| **As-patterns** (`Cons(h,t) as whole`) | ❓ | ❌ |
| **Pattern guards** (`x if x < 0 => ...`) | ❓ | ❌ |
| **Let destructuring** (`let Point(x,y) = p in ...`) | ❓ | ❌ |

**Action:** Review chapter to ensure it doesn't show examples that won't parse. Add "Not Yet Implemented" section if aspirational features are mentioned.

---

### Issue: Complex Numbers Not Instantiable

**The manual claims** (`01-starting-out.md`):
```
ℂ (or Complex)    Complex numbers    3 + 4i, i
```

**Reality:**
- `Complex` is a type **tag** in `stdlib/types.kleis`: `| Complex` (nullary variant)
- No **data constructor** exists: `Complex(real: ℝ, imag: ℝ)`
- No **literal syntax** in parser: `3 + 4i` doesn't parse
- No **imaginary unit**: `i` is not defined
- The only TODO is in `stdlib/prelude.kleis`: `// define i : ℂ = ... (TODO: needs complex literal syntax)`

**Manual example** (`09-structures.md`) shows aspirational code:
```kleis
structure Complex {
    re : ℝ  // real part
    im : ℝ  // imaginary part
}
```

But this is NOT in stdlib. You **cannot create complex values** currently.

**Options:**
1. Fix the documentation to say "planned, not implemented"
2. Implement complex numbers properly:
   - Add `data ℂ = Complex(re: ℝ, im: ℝ)` to stdlib
   - Add literal syntax `3 + 4i` to parser
   - Define `i : ℂ = Complex(0, 1)`

### Issue: ASCII Alternatives Not All Implemented

**The manual claims** (`01-starting-out.md`):

| Unicode | ASCII Alternative |
|---------|-------------------|
| `∀`     | `forall`          |
| `∃`     | `exists`          |
| `→`     | `->`              |
| `×`     | `*`               |
| `ℝ`     | `Real`            |
| `ℕ`     | `Nat`             |

**Actual parser support:**

| Claim | Status | Notes |
|-------|--------|-------|
| `forall` | ✅ Works | Parser line 891 |
| `exists` | ✅ Works | Parser line 914 |
| `->` | ✅ Works | Parser line 1337 |
| `*` for `×` | ❌ Different | `*` = multiply, `×` = product type |
| `Real` for `ℝ` | ❌ Not aliased | Just different identifiers |
| `Nat` for `ℕ` | ❌ Not aliased | Just different identifiers |

**Fix needed:** Either implement the aliases or correct the documentation.

---
*Recorded: Dec 17, 2025*
*Updated: Dec 18, 2025*

