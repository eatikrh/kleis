# ADR-013: Paper-Level Scope Hierarchy for Type System

## Status
Proposed

## Date
2025-12-05

## Context

Kleis aims to be a mathematical document editor that can type-check expressions and verify mathematical correctness. A key design question is: **how should types and definitions be scoped within a document?**

Academic papers have implicit scoping conventions:
- Notation defined at the start applies throughout
- Theorems reference earlier definitions
- Symbols are (usually) consistent within a paper
- But violations happen (same symbol, different meanings in different sections)

Kleis needs a scoping model that:
1. Matches how mathematicians actually think about papers
2. Catches errors (type mismatches, undefined symbols)
3. Allows flexibility where needed
4. Supports the POT (Projected Ontology Theory) use case

## Decision

We propose a **hierarchical scope model** with five levels:

```
┌─────────────────────────────────────────────────────────────┐
│  PACKAGE SCOPE (imported libraries)                         │
│  └─ std.quantum, std.pot, std.physics                       │
├─────────────────────────────────────────────────────────────┤
│  PAPER SCOPE (document-wide)                                │
│  └─ @paper { define ψ : ℋ, define Π : ... }                 │
├─────────────────────────────────────────────────────────────┤
│  SECTION SCOPE (optional overrides)                         │
│  └─ @section("Classical Limit") { shadow ψ : ℝ → ℝ }        │
├─────────────────────────────────────────────────────────────┤
│  BLOCK SCOPE (theorem/definition/proof)                     │
│  └─ @theorem { given: x : V, claim: ... }                   │
├─────────────────────────────────────────────────────────────┤
│  CELL SCOPE (innermost, temporary)                          │
│  └─ let temp = f(x)  // scratch work                        │
└─────────────────────────────────────────────────────────────┘
```

### Scope Rules

1. **Inner scopes inherit from outer scopes** (lexical scoping)
2. **Shadowing requires explicit `shadow` keyword** and triggers warning
3. **Paper scope is immutable after definition** (no redefinition)
4. **Cell scope is ephemeral** (cleared when cell deleted/reset)
5. **Exports must be explicit** (section doesn't auto-export to paper)

### Syntax Examples

#### Package Import
```kleis
import std.quantum as Q       // Namespaced
import std.pot                 // Direct import
use std.physics.{tensor, metric}  // Selective import
```

#### Paper Scope (Notation Section)
```kleis
@paper("Projected Ontology Theory") {
  // Spaces
  define ℋ : HilbertSpace(ℂ, countable)
  define M : Manifold(ℝ, 4, signature=(-,+,+,+))
  
  // Core objects
  define Π : Functional(ℋ) → Field(M)
  define K : M × ℋ → ℂ
  
  // Constraints
  axiom hermitian_kernel: K(x,m)* = K(x,m)
  axiom normalization: ∫_M |K(x,m)|² dx = 1
  
  // Projection residues
  define c : Field(M, ℝ⁺)    // variable speed of light
  define ℏ : Field(M, ℝ⁺)    // variable Planck constant
}
```

#### Section Scope
```kleis
@section("Quantum Regime") {
  // Inherits all paper definitions
  // Can add section-local definitions
  define ρ : DensityMatrix(ℋ)
}

@section("Classical Limit") {
  // WARNING: This shadows paper-level ψ
  shadow ψ : M → ℝ  
  
  // Section-local, no conflict
  define S : M → ℝ  // Hamilton-Jacobi action
}
```

#### Block Scope (Theorems)
```kleis
@theorem("Projection Uniqueness") {
  given:
    ψ₁, ψ₂ : ℋ
    Π[ψ₁] = Π[ψ₂]  // same projection
  claim:
    ψ₁ = e^{iθ} ψ₂  // equal up to phase
  proof:
    // ... proof steps, can use local variables
}

@definition("Causal Bound") {
  let B_c(x, r) = { y ∈ M : d(x,y) < r/c(x) }
  // B_c is now available in paper scope? Or must export?
}
```

#### Cell Scope
```kleis
// Scratch calculation, not part of paper
let temp = Π_ψ(x₀)
eval(temp)  // → 0.73 + 0.21i
// `temp` disappears when cell is cleared
```

### Execution Scope (Orthogonal)

Separate from lexical scope, **execution scope** tracks evaluation state:

```kleis
@execution {
  mode: symbolic | numeric | mixed
  precision: arbitrary | float64 | interval
  assumptions: [x > 0, ψ ∈ domain(Π)]
}
```

Execution scope affects HOW expressions evaluate, not WHAT types they have.

## Consequences

### Pros

| Benefit | Description |
|---------|-------------|
| **Matches academic convention** | Paper-wide notation is how mathematicians actually write |
| **Prevents silent errors** | Can't accidentally use undefined symbol or wrong type |
| **Explicit shadowing** | Must acknowledge when reusing symbol with different meaning |
| **Theorem locality** | `given:` clauses create clean local contexts |
| **Import system** | Can build on others' type definitions (arXiv vision) |
| **Gradual adoption** | Can start with just cell scope, add paper scope later |
| **IDE support** | Scopes enable autocomplete, go-to-definition, refactoring |
| **Verification** | Can type-check entire paper for consistency |

### Cons

| Drawback | Description |
|----------|-------------|
| **Overhead for simple docs** | Quick notes don't need full paper scope ceremony |
| **Learning curve** | Mathematicians must learn scoping rules |
| **Shadowing friction** | Forcing explicit `shadow` may feel pedantic |
| **Rigidity** | Some papers intentionally reuse symbols loosely |
| **Migration cost** | Converting existing LaTeX papers requires effort |
| **Cross-paper complexity** | Importing from other papers has versioning issues |
| **Axiom trust** | Who verifies the axioms themselves are consistent? |
| **Scope creep** | Could become too complex (modules, visibility, etc.) |

### Mitigations

| Con | Mitigation |
|-----|------------|
| Overhead for simple docs | Default "notebook mode" with just cell scope |
| Learning curve | Good defaults, progressive disclosure |
| Shadowing friction | Warning, not error; can disable per-section |
| Rigidity | "Loose mode" flag for exploratory work |
| Migration cost | LaTeX import tool with scope inference |
| Cross-paper complexity | Semantic versioning, immutable published types |
| Axiom trust | Mark axioms as "assumed" vs "proven" |
| Scope creep | Start minimal, add features based on need |

## Alternatives Considered

### Alternative 1: Flat Scope (Everything Global)
- All definitions in one namespace
- Simple but doesn't scale
- No theorem-local variables
- **Rejected:** Doesn't match paper structure

### Alternative 2: Module System (Like Haskell/ML)
- Full module system with exports, hiding, functors
- Very powerful but complex
- **Deferred:** May add later for large projects, not MVP

### Alternative 3: Dynamic Scope (Like TeX)
- Definitions apply from point of definition onward
- Easy to implement but confusing semantics
- **Rejected:** Too error-prone, not declarative

### Alternative 4: No Sections, Just Paper + Cell
- Only two levels: document and cell
- Simpler but loses theorem locality
- **Rejected:** `given:` clauses are too useful

## Implementation Notes

### Phase 1: Cell Scope Only
- Current Kleis behavior
- Variables exist in cell, cleared on reset
- No persistence

### Phase 2: Add Paper Scope
- `@paper { }` block at document start
- Definitions persist across cells
- Type-checking against paper definitions

### Phase 3: Add Block Scope
- `@theorem`, `@definition`, `@proof` blocks
- `given:` clause for local assumptions
- Scoped variables

### Phase 4: Add Section Scope
- `@section` blocks
- Explicit `shadow` for overrides
- Warning system

### Phase 5: Add Package Scope
- `import` statements
- Standard library (std.quantum, std.pot, etc.)
- Cross-paper imports (arXiv integration)

## Related ADRs

- ADR-005: Visual Authoring (custom types)
- ADR-010: Type System Design (algebraic hierarchy)
- ADR-011: Notebook Environment (execution model)
- ADR-012: Document Authoring (rendering pipeline)

## References

- Lean 4 namespace system
- Agda module system  
- LaTeX \newcommand scoping
- Jupyter notebook variable persistence
- Mathematica scoping constructs (Module, Block, With)

## Open Questions

1. **Should definitions auto-export from theorems?**
   - If you define something in a proof, is it paper-visible?
   - Suggestion: No, require explicit `export`

2. **How to handle mutual recursion?**
   - Definition A references B, B references A
   - Suggestion: Allow within same scope block

3. **Section ordering constraints?**
   - Can Section 3 reference Section 5?
   - Suggestion: Yes, scopes are declarative not sequential

4. **Versioning for imports?**
   - What if imported paper changes?
   - Suggestion: Lock to specific version/commit

5. **Type inference vs annotation?**
   - Must all definitions have explicit types?
   - Suggestion: Inference with optional annotation, warning if ambiguous

---

*This ADR establishes the foundation for Kleis's type system scoping. Implementation should proceed incrementally, validating each phase with real use cases (especially POT formalization).*

