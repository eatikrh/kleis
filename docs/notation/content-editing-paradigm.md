# Content Editing Paradigm in Kleis

**Date:** December 6, 2024  
**Status:** Design Discussion (led to ADR-015)  
**Formal Decision:** [ADR-015: Text as Source of Truth](adr-015-text-as-source-of-truth.md)  
**Related:** Grammar v0.3, Equation Editor, Notebook System

> **Note:** This document captures the design discussion. For the formal decisions, see [ADR-015](adr-015-text-as-source-of-truth.md).

---

## The Core Challenge

Kleis has evolved to include multiple content types that require different editing approaches:

1. **Executable Kleis Code** - with mathematical operators and symbols
2. **Mathematical Equations** - for documentation and presentation (via visual equation editor)
3. **Rich Media** - tables, images, graphs, etc.

This creates a fundamental question: How do we provide a coherent editing experience across computational and presentational content?

---

## Current State

### Kleis Grammar Already Uses Unicode Mathematical Notation

The Kleis v0.3 grammar extensively uses Unicode mathematical symbols:

**Quantifiers (with text alternatives):**
- `∀` or `forall` - universal quantification  
- `∃` or `exists` - existential quantification

**Type symbols:**
- `ℝ`, `ℂ`, `ℤ`, `ℕ`, `ℚ` (alternatives: `Real`, `Complex`, `Integer`, `Nat`, `Rational`)

**Operators:**
- Arithmetic: `×`, `·`, `⊗`, `∘`, `^`
- Relations: `≠`, `≤`, `≥`, `≈`, `≡`, `∈`, `∉`, `⊂`, `⊆`
- Logic: `∧`, `∨`, `⟹`, `⟺`, `→`, `⇒`
- Calculus: `∇`, `∂`, `√`, `∫`, `†`, `ᵀ`

**Mathematical constructs:**
```kleis
Σ_{i=0}^{n} f(i)         // Summation
Π_{i=1}^{n} f(i)         // Product
∫_{a}^{b} f(x) dx        // Integral
∂f/∂x                    // Partial derivative
```

**Constants:**
- `π`, `e`, `i`, `ℏ`, `c`, `φ`, `∞`, `∅`

### The Keyboard Input Problem

The challenge: These mathematical symbols are not on standard keyboards. How do users type them?

---

## Proposed Design Philosophy

### Computational Layer vs. Presentation Layer

Rather than "two ways of editing," frame this as two distinct layers:

- **Computational Layer**: Kleis code cells (text-based, executable, with symbol input helpers)
- **Presentation Layer**: Documentation, equations, rich media (WYSIWYG, visual formatting)

### Key Insight: Kleis Code IS Already Mathematical

Kleis code doesn't use ASCII approximations like `sum(i, 0, n, f(i))`. It uses the actual mathematical notation: `Σ_{i=0}^{n} f(i)`.

This means:
- **The visual editor and code editor produce the same symbols**
- **Both can generate compatible ASTs**
- **The difference is in the INPUT method, not the output**

---

## The Palette-Assisted Code Editor Approach

### Proposed Solution

When editing Kleis code:
1. User types text that triggers a palette (e.g., "for", "sum", "int")
2. Palette shows relevant Unicode symbols
3. User selects symbol
4. Rich AST is created with full semantic structure

```
User types:  "sum"  [triggers palette]
                ↓
           Shows: Σ  ∑  Π  sum
                ↓
         User selects Σ
                ↓
          Σ_{i=0}^{n}
                ↓
   Rich AST: {type: "summation", variable: "i", from: 0, to: "n", ...}
```

### Benefits

- **Text-based code** (version control, search, copy-paste works)
- **Assisted input** (no hunting for Unicode characters)
- **Rich parsing** (semantic structure, not just string matching)
- **Visual rendering** (can display beautifully in notebook)

---

## The Critical Question: One-to-One Mapping

### Hypothesis

There exists a bijection between:
- **Kleis text syntax** (Unicode mathematical notation)
- **Visual representation** (2D typeset mathematics)

Such that:
```
Visual Editor → Rich AST → Kleis Text → Parser → Same Rich AST → Renderer → Same Visual
```

### Why This Matters

If we can prove 1:1 mapping:
- **Interoperability**: Visual and text editors are truly equivalent
- **No information loss**: Round-tripping preserves semantics
- **Unified type inference**: Same AST regardless of input method
- **Consistent rendering**: Code always displays the same way

### What Needs to be Proven

For each mathematical construct:

1. **Canonical text form** - unambiguous Unicode representation
2. **Parsing rules** - deterministic AST generation
3. **Rendering rules** - deterministic visual layout
4. **Invertibility** - can reconstruct text from visual (and vice versa)

---

## AST Design Implications

### AST Hierarchy

```
Visual Editor AST (Rich)
    ↓ (preserves all information)
Kleis Code AST (Core)
    ↓ (adds default rendering hints)
Visual Representation
```

### Rich AST Example

```typescript
// Core structure (minimal, executable)
{
  type: "summation",
  variable: "i",
  lowerBound: 0,
  upperBound: "n",
  expression: { type: "identifier", name: "f" }
}

// Rich AST (extended with metadata)
{
  type: "summation",
  variable: "i",
  lowerBound: 0,
  upperBound: "n",
  expression: { type: "identifier", name: "f" },
  // Additional metadata:
  inferredTypes: { 
    i: "Integer", 
    n: "Integer",
    result: "Real"
  },
  visualLayout: "display",  // vs "inline"
  renderingHints: {
    size: "large",
    symbolStyle: "traditional"
  }
}
```

### Type Inference Advantage

The visual editor becomes a **type annotation tool**:
- User creates matrix visually → dimensions and element types are immediately known
- User writes matrix in code → types must be inferred from context
- Visual AST can carry explicit type information that guides inference

---

## Design Decisions → See ADR-015

**The formal decisions are documented in [ADR-015: Text as Source of Truth](adr-015-text-as-source-of-truth.md).**

This section captures the design discussion that led to those decisions.

### 1. Source of Truth: TEXT ✅

**Design Question:** Should we store text, AST, or both?

**Discussion:**
- Storing AST (JSON) makes git diffs unreadable
- Storing text makes version control natural
- But what should the text format be?
- Need canonical forms for consistent diffs

**Decision (see ADR-015):**
Store Kleis code as Unicode text in `.kleis` files. AST is derived.

### 2. Display Mode: Same Semantics, Different Syntax ✅

**Design Question:** Should `frac(a,b)` and `a/b` be different operations?

**Discussion:**
- They're mathematically identical (both are division)
- But users want different display styles
- Options: (a) Same operation with metadata, (b) Different operations

**Decision (see ADR-015):**
Different syntax, same semantics. `frac(a,b)` signals display mode, `/` is inline.

### 3. Ambiguous Notation: Require Explicit Forms ✅

**Design Question:** How to handle `|x|` which could mean abs, cardinality, or norm?

**Discussion:**
- Option A: Allow `|x|`, resolve via types (requires 2-pass parsing)
- Option B: Require explicit forms: `abs(x)`, `card(S)`, `norm(v)`
- Option C: Different delimiters: `|x|` vs `||x||`

**Decision (see ADR-015):**
Require explicit forms in text. Visual display can use traditional notation.

**Rationale:**
- Git diffs are clear: `abs(x)` → `card(S)` shows operation change
- Error messages are helpful: "card() expects Set, got Number"
- Single-pass parsing (no type context needed)
- Visual can still be beautiful (renders as |x|)

## Open Questions

### 1. User Refinement
- Can users customize visual layout while preserving semantics?
- Example: Change `Σ` display from inline to display mode
- Does this affect the AST or just rendering hints?

### 2. Partial Expressions
- How do we handle incomplete expressions during editing?
- Visual editor: user is building structure step-by-step
- Text editor: code may be syntactically invalid mid-edit

---

## Next Steps

1. **Catalog all mathematical notations** we want to support
2. **Define canonical forms** for each notation (text and visual)
3. **Test for ambiguities** - try to find counterexamples to 1:1 mapping
4. **Design AST structure** that captures all semantic information
5. **Implement round-trip tests** to validate bijection
6. **Design palette UI** for assisted Unicode input

---

## Test Cases

See [notation-mapping-tests.md](notation-mapping-tests.md) for specific test cases validating the one-to-one mapping hypothesis.

