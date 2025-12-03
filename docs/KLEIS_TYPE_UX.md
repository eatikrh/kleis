# Kleis Type System - User Experience Design

## Status
**Draft** - UX design for type inference, annotations, and context management

---

## Core UX Questions

1. **When/how does Kleis show inferred types?**
2. **When/how does it ask users to specify types?**
3. **Where is context stored?**
4. **How does context scope work?**
5. **What's the interaction model?**

---

## Proposed UX Model: Hybrid Document + Notebook

### Document Structure

A Kleis document consists of **cells** (like Jupyter) organized into **sections** with scoped contexts:

```
┌─────────────────────────────────────────┐
│ Document: Classical Mechanics           │
├─────────────────────────────────────────┤
│ Context: physics                        │
│   m: Scalar = 1.5  [kg]                 │
│   v: Vector(3) = [1, 2, 0]  [m/s]       │
│   t: Scalar  [s]                        │
├─────────────────────────────────────────┤
│ Cell 1: [Expression]                    │
│   E = ½mv²                              │
│                                         │
│   Inferred: E: Scalar  [J]              │
│   Result: 3.75 J                        │
├─────────────────────────────────────────┤
│ Cell 2: [Expression]                    │
│   p = mv                                │
│                                         │
│   Inferred: p: Vector(3)  [kg·m/s]      │
│   Result: [1.5, 3.0, 0] kg·m/s          │
└─────────────────────────────────────────┘
```

---

## Type Inference UI

### Mode 1: Implicit Inference (Default)

User writes expression; Kleis infers types automatically:

```
┌───────────────────────────────────────────┐
│ [Cell 1]                                  │
│   F = ma                                  │
│                                           │
│ ┌─ Type Info ─────────────────────────┐  │
│ │ Inferred types:                      │  │
│ │   m: Scalar  (assumed)               │  │
│ │   a: Vector(?) (from context)        │  │
│ │   F: Vector(?) (result of m × a)     │  │
│ │                                      │  │
│ │ ⚠️  Dimension unknown. Specify:       │  │
│ │   a: Vector(3) ✎                     │  │
│ └──────────────────────────────────────┘  │
└───────────────────────────────────────────┘
```

**Interaction:**
- Types shown in **collapsible side panel** or hover tooltip
- Warnings for ambiguous types (dimension unknown)
- Click `✎` to specify type explicitly

### Mode 2: Explicit Annotations

User can declare types upfront (like type hints in Python):

```kleis
// Declare before use
m: Scalar
a: Vector(3)
F: Vector(3)

// Now use them
F = ma

// Type checker: ✓ All types match
// Inferred: scalar_multiply(Scalar, Vector(3)) → Vector(3)
```

### Mode 3: Hybrid (Recommended)

```
┌─────────────────────────────────────────────────────┐
│ Context Panel (collapsible)                         │
│ ┌─────────────────────────────────────────────────┐ │
│ │ 📋 Active Context                                │ │
│ │                                                  │ │
│ │ Declared:                                        │ │
│ │   m: Scalar = 1.5                                │ │
│ │   v₀: Vector(3) = [0, 0, 0]                      │ │
│ │                                                  │ │
│ │ Inferred:                                        │ │
│ │   E: Scalar (from ½mv²)                          │ │
│ │   p: Vector(3) (from mv)                         │ │
│ │                                                  │ │
│ │ [+ Add Symbol]                                   │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ Cell 1: Newton's Second Law                         │
│                                                     │
│   F = ma              ← hover shows type info       │
│                                                     │
│   F: Vector(3)  ✓ (inferred)                        │
└─────────────────────────────────────────────────────┘
```

**Hover behavior:**
```
User hovers over 'm' in the equation:

┌───────────────────────┐
│ m: Scalar             │
│ Value: 1.5            │
│ Units: kg             │
│ Source: Context       │
└───────────────────────┘
```

---

## Context Management

### Context Scoping (Three Options)

#### Option A: Global Document Context

```
Document
  └─ One global context (like Jupyter kernel)
     └─ All cells share same namespace
     └─ Sequential execution updates context
```

**Pros:** Simple, familiar (Jupyter-like)  
**Cons:** No isolation, hard to reason about dependencies

#### Option B: Hierarchical Contexts (Recommended)

```
Document
  ├─ Context: physics
  │    ├─ Cell 1 (inherits physics context)
  │    ├─ Cell 2 (inherits + adds local bindings)
  │    └─ Subsection
  │         ├─ Context: quantum (extends physics)
  │         └─ Cells inherit quantum context
  │
  └─ Context: geometry (separate)
       └─ Cells (different namespace)
```

**Pros:** Modular, reusable, explicit scoping  
**Cons:** More complex to manage

#### Option C: Cell-Local with Imports

```
Cell 1:
  import physics.*
  E = mc²

Cell 2:
  import physics.{m, c}
  import geometry.{r}
  // Only imported symbols visible
```

**Pros:** Explicit dependencies, no hidden state  
**Cons:** Verbose for quick calculations

### Recommended Hybrid: Hierarchical + Auto-Propagation

```
Document
  ├─ Preamble Context (declarations)
  │    m: Scalar
  │    v: Vector(3)
  │
  ├─ Cell 1 (uses preamble context)
  │    E = ½mv²
  │    → Adds E: Scalar to context
  │
  └─ Cell 2 (sees preamble + Cell 1)
       F = ∇E
       → Can use E from Cell 1
```

**Execution model:** Top-to-bottom like Jupyter, but with explicit section breaks for context isolation.

---

## Storage & Persistence

### File Format (Option 1: JSON-based)

```json
{
  "document": {
    "title": "Classical Mechanics",
    "version": "1.0",
    "contexts": [
      {
        "name": "physics",
        "bindings": {
          "m": {"type": "Scalar", "value": 1.5, "units": "kg"},
          "v": {"type": "Vector(3)", "value": [1, 2, 0], "units": "m/s"}
        },
        "type_declarations": {
          "F": "Vector(3)",
          "E": "Scalar"
        }
      }
    ],
    "cells": [
      {
        "id": "cell-1",
        "context": "physics",
        "content": "E = ½mv²",
        "ast": {...},
        "inferred_type": "Scalar",
        "result": {"value": 3.75, "type": "Scalar", "units": "J"}
      },
      {
        "id": "cell-2",
        "context": "physics",
        "content": "p = mv",
        "ast": {...},
        "inferred_type": "Vector(3)",
        "result": {"value": [1.5, 3.0, 0], "type": "Vector(3)", "units": "kg·m/s"}
      }
    ]
  }
}
```

### File Format (Option 2: Kleis Native - Recommended)

```kleis
# Classical Mechanics
# Context declarations at top

context physics {
    // Type declarations
    m: Scalar = 1.5  // kg
    v: Vector(3) = [1, 2, 0]  // m/s
    t: Scalar  // s (unbound, symbolic)
    
    // Constants
    c: Scalar = 299792458  // m/s
    G: Scalar = 6.674e-11  // N·m²/kg²
}

---

## Cell 1: Kinetic Energy

using physics

E = ½mv²

// Kleis automatically adds:
// Inferred: E: Scalar
// Result: 3.75 J

---

## Cell 2: Momentum

using physics

p = mv

// Inferred: p: Vector(3)
// Result: [1.5, 3.0, 0] kg·m/s

---

## Cell 3: New Context

context quantum {
    using physics  // Inherit
    
    ℏ: Scalar = 1.054571e-34  // J·s
    ψ: Function(ℝ³ → ℂ)  // Wave function
}

using quantum

E·ψ = iℏ·∂ψ/∂t  // Schrödinger equation

// Type checker: ✓ All types compatible
```

**File extension:** `.kleis`

**Advantages:**
- Human-readable
- Git-friendly (plain text, diffable)
- Context explicitly scoped with `context { }` blocks
- Cells separated by `---` (like Markdown)
- Type info embedded naturally

---

## Interactive Type Prompts

### Scenario 1: Ambiguous Dimension

```
User types: F = ma

┌─────────────────────────────────────────┐
│ ⚠️  Type Ambiguity                       │
├─────────────────────────────────────────┤
│ Cannot infer vector dimension.          │
│                                         │
│ Please specify:                         │
│   a: Vector( [?] )                      │
│        ↑ Enter dimension: 3_            │
│                                         │
│ Or declare in context:                  │
│   context { a: Vector(3) }              │
│                                         │
│ [Apply] [Cancel]                        │
└─────────────────────────────────────────┘
```

### Scenario 2: Type Mismatch

```
User types: E = ½mv² where v: Scalar

┌─────────────────────────────────────────┐
│ ❌ Type Error                            │
├─────────────────────────────────────────┤
│ Expression: E = ½mv²                    │
│ Issue: v² requires v: Scalar, but       │
│        kinetic energy formula expects   │
│        v: Vector(n)                     │
│                                         │
│ Did you mean:                           │
│   • E = ½m‖v‖²  (magnitude squared)     │
│   • E = ½m·v·v  (if v is 1D speed)      │
│                                         │
│ [Apply Suggestion] [Edit Type] [Cancel] │
└─────────────────────────────────────────┘
```

### Scenario 3: First Use of Symbol

```
User types: F = ma

┌─────────────────────────────────────────┐
│ 🔍 New Symbols Detected                  │
├─────────────────────────────────────────┤
│ Kleis hasn't seen these symbols before: │
│                                         │
│ F: [Infer from equation ✓] [Declare...] │
│ m: [Scalar ▼] = [1.5____]               │
│ a: [Vector(3) ▼] = [______]             │
│                                         │
│ [Add to Context] [Skip]                 │
└─────────────────────────────────────────┘
```

---

## Context Storage Options

### Option 1: Cell Metadata (Jupyter-style)

```json
{
  "cells": [
    {
      "type": "context",
      "declarations": {
        "m": {"type": "Scalar", "value": 1.5}
      }
    },
    {
      "type": "expression",
      "content": "E = ½mv²",
      "context_snapshot": {...},  // Inherited context
      "inferred_types": {"E": "Scalar"}
    }
  ]
}
```

### Option 2: Separate Context File (Recommended)

```
project/
  physics.kleis          # Main document
  physics.context.json   # Context storage
  geometry.kleis
  geometry.context.json
```

**`physics.context.json`:**
```json
{
  "version": "1.0",
  "contexts": {
    "physics": {
      "parent": null,
      "bindings": {
        "m": {"type": "Scalar", "value": 1.5, "units": "kg"},
        "c": {"type": "Scalar", "value": 299792458, "units": "m/s"}
      },
      "type_declarations": {
        "F": "Vector(3)",
        "E": "Scalar"
      },
      "inferred_types": {
        "p": "Vector(3)",
        "v": "Vector(3)"
      }
    },
    "quantum": {
      "parent": "physics",  // Inheritance
      "bindings": {
        "ℏ": {"type": "Scalar", "value": 1.054571e-34}
      }
    }
  }
}
```

### Option 3: Embedded Context (All-in-One)

```kleis
# physics.kleis

context physics {
    // All context info embedded at top
    m: Scalar = 1.5
    v: Vector(3)
}

---

E = ½mv²
// Context + types + results all in same file
```

**Recommended:** Option 3 for simplicity, with Option 2 as compilation cache.

---

## Context Lifecycle

### Initialization (When Document Opens)

```
1. Parse document top-to-bottom
2. Extract context declarations
3. Build initial context
4. Run type inference on all cells
5. Show any ambiguities/errors
```

```
┌──────────────────────────────────────────┐
│ 📂 Opening: physics.kleis                 │
│                                          │
│ ✓ Parsed 5 cells                         │
│ ✓ Loaded context "physics" (3 symbols)   │
│ ⚠️  2 type ambiguities found              │
│                                          │
│ Cell 3: Cannot infer dimension of v      │
│   Quick fix: Declare v: Vector(3)        │
│                                          │
│ [Review] [Auto-fix] [Ignore]             │
└──────────────────────────────────────────┘
```

### Incremental Updates (As User Types)

```
User types new cell: F = ∇E

1. Kleis looks up E in context
   → E: Scalar (from Cell 1)

2. Infers ∇E type
   → grad(Scalar) requires E: Field(ℝ³, Scalar)
   → Result: ∇E: Field(ℝ³, Vector(3))

3. Shows type info in real-time:
   
   ┌─────────────────────────────┐
   │ F = ∇E                      │
   │       ↑                     │
   │       │ Vector(3) field     │
   └─────────────────────────────┘

4. Adds F to context if expression evaluates successfully
```

---

## UI Mockups

### Structural Editor with Type Panel

```
┌────────────────────────────────────────────────────────────┐
│ 📝 Structural Editor                    📋 Context         │
├────────────────────────────────┬───────────────────────────┤
│                                │ physics                   │
│   E = ½mv²                     │ ─────────────────────     │
│                                │ m: Scalar = 1.5           │
│   [edit markers...]            │ v: Vector(3) = ?          │
│                                │ E: Scalar = ?  [inferred] │
│                                │                           │
│                                │ [+ Add Variable]          │
├────────────────────────────────┴───────────────────────────┤
│ Type Info: E = ½mv²                                        │
│   E: Scalar (kinetic energy, inferred from ½·Scalar·‖V‖²)  │
│   ✓ Type check passed                                      │
└────────────────────────────────────────────────────────────┘
```

### Inline Type Annotations (VS Code Style)

```
┌──────────────────────────────────────────────┐
│ F = ma                                       │
│ │   │└─ a: Vector(3)                         │
│ │   └── m: Scalar                            │
│ └────── F: Vector(3) [inferred]              │
│                                              │
│ [✓] Enable inline type hints                 │
└──────────────────────────────────────────────┘
```

### Type Declaration Dialog

```
When user types undefined symbol 'a':

┌──────────────────────────────────────────┐
│ Declare Symbol: a                        │
├──────────────────────────────────────────┤
│ Type:  [Scalar ▼]                        │
│        • Scalar                          │
│        • Complex                         │
│        • Vector(n) ──→ Dimension: [3__]  │
│        • Matrix(m,n)                     │
│        • Field                           │
│        • Custom...                       │
│                                          │
│ Value (optional):                        │
│   [9.8_________]                         │
│                                          │
│ Units (optional):                        │
│   [m/s²________]                         │
│                                          │
│ [Add to Context] [Infer Later] [Cancel]  │
└──────────────────────────────────────────┘
```

---

## Context Persistence

### When to Save Context

1. **Auto-save:** After each successful cell evaluation
2. **Explicit save:** When user clicks "Save" (saves document + context)
3. **On close:** Prompt if context has unsaved changes

### Context Versioning

```json
{
  "context_history": [
    {
      "timestamp": "2025-12-03T10:00:00Z",
      "snapshot": {...},
      "description": "Added velocity vector"
    },
    {
      "timestamp": "2025-12-03T10:05:00Z",
      "snapshot": {...},
      "description": "Updated mass to 2.0 kg"
    }
  ]
}
```

Users can **revert context** to earlier state if type errors cascade.

---

## Interactive Type Inference Flow

### Flow 1: Writing a New Expression

```
Step 1: User types: F = ma
┌────────────────────────────────────┐
│ F = ma                             │
│ ⚠️  Unknown symbols: F, m, a        │
└────────────────────────────────────┘

Step 2: Kleis attempts inference
┌────────────────────────────────────┐
│ 🔍 Analyzing...                     │
│   • Recognized pattern: F = ma      │
│   • Newton's 2nd law template?     │
│   • Suggest: F,a: Vector | m: Scalar│
└────────────────────────────────────┘

Step 3: Kleis asks for confirmation
┌────────────────────────────────────┐
│ 💡 Inferred Types (review):         │
│                                    │
│   m: Scalar  [✓]                   │
│   a: Vector(?) [specify dimension] │
│   F: Vector(?) [matches a]         │
│                                    │
│ Dimension: [3__] ← enter            │
│                                    │
│ [Accept & Add to Context] [Edit]   │
└────────────────────────────────────┘

Step 4: Context updated
┌────────────────────────────────────┐
│ ✅ Added to context:                │
│   m: Scalar                         │
│   a: Vector(3)                      │
│   F: Vector(3)                      │
└────────────────────────────────────┘
```

### Flow 2: Using Existing Symbol with Wrong Type

```
Context has: m: Scalar

User types: m + v  where v: Vector(3)

┌────────────────────────────────────┐
│ ❌ Type Error in: m + v             │
│                                    │
│   m: Scalar                        │
│   v: Vector(3)                     │
│   ↑                                │
│   Cannot add Scalar + Vector       │
│                                    │
│ Did you mean:                      │
│   • m·v  (scale vector by scalar)  │
│   • m + ‖v‖  (scalar + magnitude)  │
│                                    │
│ [Apply Fix] [Change Types] [Cancel]│
└────────────────────────────────────┘
```

---

## Comparison to Other Systems

| Feature | Jupyter | Mathematica | Kleis (Proposed) |
|---------|---------|-------------|------------------|
| **Context scope** | Global kernel | Global session | Hierarchical + scoped |
| **Type display** | None (dynamic) | On demand (`:= Head`) | Always visible (hover/panel) |
| **Type checking** | Runtime only | Mostly dynamic | Static + runtime hybrid |
| **Context storage** | Memory only | Session file | Document-embedded + cache |
| **Type annotations** | Not applicable | Optional | Inferred + declarable |
| **Error recovery** | Clear cell | Undo | Undo + context revert |

---

## Recommended Implementation

### Phase 1: Minimal Viable UX
1. **Global document context** (like Jupyter)
2. **Hover to show types** (like VS Code)
3. **Error messages with type info**
4. **Manual type declarations** via context panel

### Phase 2: Smart Inference
1. **Auto-suggest types** for common patterns
2. **Template recognition** (F=ma → suggest types)
3. **Quick-fix actions** for type errors
4. **Inline type hints** (toggleable)

### Phase 3: Advanced Features
1. **Hierarchical contexts** with inheritance
2. **Context versioning** and revert
3. **Type exploration tools** (show all possible types for ambiguous expr)
4. **Proof obligations** (show preconditions that must hold)

---

## Context Start Points

### Option A: Empty Document
```
New document starts with:
  - Empty global context
  - User builds context incrementally
  - Each symbol declared on first use
```

### Option B: Template Contexts (Recommended)
```
User creates new document from template:

Templates:
  • Blank Document (empty context)
  • Classical Mechanics (preloaded: F, m, a, v, E, p)
  • Quantum Mechanics (preloaded: ψ, ℏ, H, E)
  • Linear Algebra (preloaded: matrix ops, vector spaces)
  • Calculus (preloaded: ∂, ∇, ∫, d/dx)
  • General Relativity (preloaded: g_μν, R, T, Λ)
```

### Option C: Import Standard Library
```kleis
// Start blank, import what you need
using std.physics
using std.linear_algebra

// Now have access to standard symbols
F = ma  // F, m, a recognized from std.physics
```

---

## Summary

**Recommended UX:**
1. **Document = Context + Cells** (Jupyter-like structure)
2. **Context panel** shows all bindings + inferred types
3. **Hover/inline hints** show type info while editing
4. **Type prompts** for ambiguous/new symbols
5. **Storage:** `.kleis` file with embedded context
6. **Scoping:** Top-to-bottom execution with hierarchical contexts
7. **Templates:** Pre-configured contexts for common domains

This balances **power** (explicit control) with **convenience** (smart inference) while keeping the UX familiar to users of Jupyter, Mathematica, and VS Code.

---

**Next Step:** Prototype the context panel UI in the structural editor?

