# Next Session: Equation Editor & Kleis Grammar Alignment

---
## ⚠️ CRITICAL ARCHITECTURE LESSONS (Dec 14, 2024)

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

**Date:** Dec 14, 2024

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

### Implementation Status (Dec 14, 2024):
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

**What was built (Dec 14, 2024):**
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

## ✅ FIXED: Editor AST Uses Semantic Names (Dec 14, 2024)

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
*Created: Dec 14, 2024*

