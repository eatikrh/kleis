# ADR-011: Kleis Notebook Environment

## Status
**Proposed** - Ready for implementation

## Context

Kleis needs a computational environment for:
1. **Editing .kleis files** (mathematical definitions)
2. **Type inference** with context management
3. **Cell-based execution** (like Jupyter)
4. **Live rendering** of mathematical notation
5. **Integration** with the structural editor (v2.2)

## Vision

> A hybrid notebook environment where mathematical expressions are edited structurally, executed symbolically, and verified by type system - all in a visual, interactive interface.

---

## Architecture: Three-Layer Model

```
┌──────────────────────────────────────────┐
│         Notebook Interface (Web)         │
│  - Cell editor (structural + text modes) │
│  - Context panel                         │
│  - Output rendering                      │
└──────────────┬───────────────────────────┘
               │
┌──────────────┴───────────────────────────┐
│         Kleis Runtime (Rust)             │
│  - Parser                                │
│  - Type inference engine                 │
│  - Evaluator                             │
│  - Context manager                       │
└──────────────┬───────────────────────────┘
               │
┌──────────────┴───────────────────────────┐
│         Storage Layer                    │
│  - .kleis files (source)                 │
│  - .kleis-nb files (notebook state)      │
│  - Package registry                      │
└──────────────────────────────────────────┘
```

---

## File Format: `.kleis` vs `.kleis-nb`

### `.kleis` Files (Source Code)

Plain text, Git-friendly, human-readable:

```kleis
# physics.kleis - Classical Mechanics Definitions

context physics {
    // Constants
    c: Scalar = 299792458  // m/s - Speed of light
    G: Scalar = 6.674e-11  // N·m²/kg² - Gravitational constant
    
    // Variables
    m: Scalar  // kg - Mass
    v: Vector(3)  // m/s - Velocity
    F: Vector(3)  // N - Force
    
    // Derived
    p: Vector(3) = m * v  // Momentum
    E: Scalar = ½ * m * v²  // Kinetic energy
}

---

## Newton's Second Law

using physics

F = m * a

// Type: Vector(3) = Scalar * Vector(3)
// Checks: ✓ Types compatible
```

### `.kleis-nb` Files (Notebook State)

JSON format with execution results, saved outputs:

```json
{
  "version": "1.0",
  "kernel": "kleis-0.1",
  "metadata": {
    "title": "Classical Mechanics",
    "author": "user",
    "created": "2025-12-03"
  },
  "contexts": {
    "physics": {
      "bindings": {
        "m": {"type": "Scalar", "value": 1.5, "units": "kg"},
        "v": {"type": "Vector(3)", "value": [1, 2, 0], "units": "m/s"}
      }
    }
  },
  "cells": [
    {
      "id": "cell-1",
      "type": "code",
      "source": "F = m * a",
      "ast": {"Operation": {...}},
      "outputs": [
        {
          "type": "expression",
          "latex": "F = ma",
          "svg": "<svg>...</svg>",
          "inferred_type": "Vector(3) = Scalar * Vector(3)",
          "type_check": "success"
        }
      ],
      "execution_count": 1
    }
  ]
}
```

---

## Notebook UI Design

### Layout (Inspired by Jupyter + Mathematica)

```
┌────────────────────────────────────────────────────────┐
│ 📘 Kleis Notebook: Classical Mechanics                │
│ ─────────────────────────────────────────────────────  │
│ [▶ Run All] [+ Cell] [⬆⬇] [💾 Save] [📤 Export]      │
└────────────────────────────────────────────────────────┘

┌─ CONTEXT PANEL ──────────────┐  ┌─ NOTEBOOK ─────────┐
│ 📦 physics                    │  │                     │
│                               │  │ ┌─ Cell 1 ─────┐  │
│ Variables:                    │  │ │ F = ma        │  │
│  m: Scalar = 1.5 kg           │  │ │               │  │
│  v: Vector(3) = [1,2,0] m/s   │  │ │ Out: F = ma   │  │
│  F: Vector(3) [unbound]       │  │ │ Type: ✓       │  │
│                               │  │ └───────────────┘  │
│ Constants:                    │  │                     │
│  c: 299792458 m/s             │  │ ┌─ Cell 2 ─────┐  │
│  G: 6.674e-11 N·m²/kg²        │  │ │ E = ½mv²      │  │
│                               │  │ │               │  │
│ [+ Add Variable]              │  │ │ Out: 3.75 J   │  │
│ [Import Context]              │  │ │ Type: ✓       │  │
│                               │  │ └───────────────┘  │
└───────────────────────────────┘  │                     │
                                   │ [+ Add Cell]        │
                                   └─────────────────────┘
```

### Cell Types

**1. Code Cell (Expression)**
```
┌─ Cell 1 [Expression] ──────────────────────┐
│ ┌─ Input ─────────────────────────────────┐│
│ │ F = ma                                  ││  ← Structural editor!
│ └─────────────────────────────────────────┘│
│ ┌─ Output ────────────────────────────────┐│
│ │ F = ma                                  ││  ← Rendered
│ │ Type: Vector(3) = Scalar × Vector(3) ✓ ││  ← Type info
│ └─────────────────────────────────────────┘│
│ [▶ Run] [Structural] [Text] [🐛 Debug AST]│
└────────────────────────────────────────────┘
```

**2. Context Cell (Definitions)**
```
┌─ Cell 0 [Context] ─────────────────────────┐
│ context physics {                          │
│     m: Scalar = 1.5  // kg                 │
│     v: Vector(3) = [1, 2, 0]  // m/s       │
│     F: Vector(3)  // N (unbound)           │
│ }                                          │
│ [▶ Load Context]                           │
└────────────────────────────────────────────┘
```

**3. Markdown Cell (Documentation)**
```
┌─ Cell 2 [Markdown] ────────────────────────┐
│ ## Newton's Second Law                     │
│                                            │
│ Force equals mass times acceleration...   │
└────────────────────────────────────────────┘
```

---

## Cell Editing Modes

### Mode 1: Structural Editor (Default)

Using the v2.2 inline editing system!

```
Input Cell:
┌──────────────────────────────────┐
│  F  =  m  ×  a                   │
│  □      □      □  ← Click to edit│
└──────────────────────────────────┘

Click placeholder → Inline input appears
Type or click symbols → Natural workflow
Press Enter → Value committed
```

**Benefits:**
- Uses existing structural editor (v2.2)
- Inline editing already works!
- Template buttons available
- Type inference can happen live

### Mode 2: Text Editor (Alternative)

Traditional LaTeX input with live preview:

```
┌─ Input ─────────────────────────┐
│ F = ma                          │  ← Text input
└─────────────────────────────────┘
┌─ Preview ───────────────────────┐
│ F = ma                          │  ← Live render
└─────────────────────────────────┘
```

### Mode Toggle

Each cell has a toggle: **[📐 Structural] [📝 Text]**

---

## Context Management

### Context Panel (Left Sidebar)

```
┌─ CONTEXTS ──────────────────────┐
│ 📦 Active: physics              │
│                                 │
│ Variables:                      │
│  m: Scalar = 1.5                │ ← Hover shows units
│  v: Vector(3) = [1,2,0]         │ ← Click to edit
│  F: Vector(3) [unbound]         │ ← Symbolic
│                                 │
│ [+ Add Variable]                │
│ [📤 Export Context]             │
│ [📥 Import Context]             │
│                                 │
│ Available:                      │
│  □ std.calculus                 │ ← Checkbox to import
│  □ std.linear_algebra           │
│  □ std.quantum                  │
│                                 │
│ Custom:                         │
│  ✓ physics (current)            │
│  □ cosmology (from file)        │
└─────────────────────────────────┘
```

### Context Loading

```javascript
// Load context from .kleis file
await loadContext('kleis/physics.kleis');

// Contexts are additive (can import multiple)
contexts.push(physicsContext);
contexts.push(calculusContext);

// Cell execution uses merged context
const mergedContext = mergeContexts(contexts);
```

---

## Type Inference Integration

### Real-Time Type Checking

**As you edit:**

```
Cell input: F = m × v

Type inference runs:
  m: Scalar (from context)
  v: Vector(3) (from context)
  m × v: ???

Type checker:
  scalar_multiply: (Scalar, Vector(n)) → Vector(n)
  Result: F: Vector(3) ✓

Display under cell:
  ✅ F: Vector(3)  [N]
```

### Type Error Display

```
Cell input: E = F + m

Type inference:
  F: Vector(3)
  m: Scalar
  F + m: ???

Type checker:
  plus: requires compatible types
  Vector(3) + Scalar → ERROR

Display under cell:
  ❌ Type mismatch: Cannot add Vector(3) + Scalar
  Suggestion: Did you mean F + m·v?
```

### Inline Type Hints

During inline editing:

```
You type: "F"
Tooltip appears: F: Vector(3) [N]  ← From context

You type: "m"  
Tooltip appears: m: Scalar [kg]

As you build: F = m × a
Live type: Vector(3) = Scalar × Vector(3) ✓
```

---

## File Editing Workflow

### Opening .kleis Files

**Option A: Import into Notebook**
```
1. Click "📂 Open .kleis"
2. Select physics.kleis
3. Parses context block
4. Loads into context panel
5. Can now use physics symbols in cells
```

**Option B: Edit .kleis Directly**
```
1. Click "📝 Edit Source"
2. Opens physics.kleis in text editor
3. Edit context definitions
4. Save → Auto-reloads context
5. All cells re-type-check
```

### Saving Notebooks

**Two file types:**

1. **Source only (.kleis)** - Version control friendly
   ```
   File → Save As → Source (.kleis)
   Saves: contexts + cell sources
   Omits: outputs, execution state
   ```

2. **Full notebook (.kleis-nb)** - Complete state
   ```
   File → Save Notebook (.kleis-nb)
   Saves: everything including outputs
   Like: .ipynb format
   ```

---

## Integration with Structural Editor

### The Power Combo

**Current v2.2 structural editor** becomes the **cell editor**!

```
Notebook Cell:
┌──────────────────────────────────────────┐
│ [▶ Run]  [Structural ✓] [Text]          │
├──────────────────────────────────────────┤
│ Structural Editor (inline editing):      │
│                                          │
│   E = ½ m v²                             │
│        ↑  ↑  ↑ ← Click to inline edit!   │
│                                          │
├──────────────────────────────────────────┤
│ Output:                                  │
│   E = 3.75 J                             │
│   Type: Scalar ✓                         │
└──────────────────────────────────────────┘
```

**Features carry over:**
- ✅ Inline editing (v2.2)
- ✅ Symbol buttons
- ✅ Template buttons
- ✅ MathJax rendering
- ✅ 137 beautiful buttons

---

## Technical Implementation

### Backend: Rust Server

**New endpoints needed:**

```rust
POST /api/notebook/create
POST /api/notebook/load { path: "physics.kleis" }
POST /api/notebook/save { notebook: {...} }

POST /api/cell/execute { cell_id, ast, context }
POST /api/cell/typecheck { ast, context }

POST /api/context/load { path: "kleis/physics.kleis" }
POST /api/context/merge { contexts: [...] }
POST /api/context/infer { symbol, context }
```

### Frontend: Notebook UI

**New components:**

```javascript
class KleisNotebook {
    cells: Cell[]
    contexts: Context[]
    activeCell: Cell | null
    
    addCell(type: 'code' | 'context' | 'markdown')
    runCell(cellId: string)
    runAll()
    
    loadContext(path: string)
    mergeContexts()
}

class Cell {
    id: string
    type: 'code' | 'context' | 'markdown'
    source: string | AST
    outputs: Output[]
    executionCount: number
    
    // Embeds structural editor (v2.2)
    editor: StructuralEditor
}

class Context {
    name: string
    bindings: Map<string, Binding>
    types: Map<string, Type>
    
    lookup(symbol: string): Binding | null
    typeOf(symbol: string): Type | null
}
```

---

## User Workflows

### Workflow 1: Create New Notebook

```
1. Click "📘 New Notebook"
2. Choose template:
   - Blank
   - Physics
   - Quantum Mechanics
   - General Relativity
   - Custom...

3. Notebook opens with:
   - Context cell (pre-filled if template)
   - Empty code cell
   - Context panel (left sidebar)

4. Start editing cells with structural editor!
```

### Workflow 2: Load Existing .kleis File

```
1. Click "📂 Open"
2. Select "kleis/physics.kleis"
3. Parser extracts:
   - Context block → Loaded into context panel
   - Definitions → Converted to cells
   - Comments → Markdown cells

4. Notebook displays with all contexts loaded
5. Edit cells using structural editor (v2.2)
6. Run cells to execute/type-check
```

### Workflow 3: Edit Cell with Inline Editing

```
1. Click cell to focus
2. Cell shows structural editor
3. Click placeholder → Inline input appears ✨ (v2.2)
4. Type or click symbols
5. Press Enter → Commits
6. Click "▶ Run" → Executes cell
7. Output appears below with:
   - Rendered equation
   - Inferred type
   - Numerical result (if evaluable)
   - Type check status
```

### Workflow 4: Context Management

```
1. Click "+ Add Variable" in context panel
2. Dialog appears:
   Name: [E____]
   Type: [Scalar ▼]
   Value: [______] (optional)
   Units: [J____]

3. Click "Add"
4. Variable appears in context panel
5. All cells re-type-check automatically
6. New variable available in all cells below
```

---

## Cell Execution Model

### Execution Order

```
Context Cells (top)
      ↓
Code Cell 1 (uses context)
      ↓
Code Cell 2 (uses previous + context)
      ↓
Code Cell 3 (uses all previous)
```

### Type Inference Flow

```
1. User edits cell: F = ma
2. Parser → AST: equals(Object("F"), scalar_multiply(...))
3. Type inference:
   - Lookup m in context: Scalar
   - Lookup a in context: Vector(3)
   - Infer: scalar_multiply(Scalar, Vector(3)) → Vector(3)
   - Infer: F: Vector(3)
4. Display type info below cell
5. If F not in context, add it with inferred type
```

### Evaluation vs Type Check

```
Cell: E = ½mv²

Type Check (always runs):
  m: Scalar
  v: Vector(3)
  v²: Scalar (dot product)
  ½mv²: Scalar
  E: Scalar ✓

Evaluation (if values bound):
  m = 1.5
  v = [1, 2, 0]
  v² = 1² + 2² + 0² = 5
  ½ × 1.5 × 5 = 3.75
  Result: E = 3.75 J ✓
```

---

## Storage Strategy

### Git-Friendly Source Files

**Store in repo:**
```
kleis/
  ├── physics.kleis       (context definitions)
  ├── cosmology.kleis     (cosmology context)
  └── axioms.kleis        (foundational axioms)

notebooks/
  ├── classical_mechanics.kleis     (executable notebook)
  ├── quantum_field_theory.kleis
  └── general_relativity.kleis
```

**Don't store:**
- Compiled outputs (regenerate)
- Execution state (.kleis-nb files are gitignored)
- Cached type info

### Notebook State (Local Only)

**Store in .kleis-nb:**
```
.kleis-nb/
  ├── classical_mechanics.kleis-nb  (with outputs)
  ├── quantum_field_theory.kleis-nb
  └── .cache/
      └── type_inference_cache.json
```

---

## Editor Integration

### The Structural Editor IS the Cell Editor

**What we have (v2.2):**
- ✅ Inline editing
- ✅ Symbol buttons
- ✅ Template buttons
- ✅ 137 buttons classified
- ✅ Keyboard shortcuts
- ✅ Beautiful rendering

**What we add:**
- ✅ Embed in cell
- ✅ Context-aware type hints
- ✅ Run button triggers execution
- ✅ Output rendering below cell

**Minimal changes needed!**

The v2.2 editor already has everything we need - just wrap it in a cell container!

---

## Implementation Phases

### Phase 1: Basic Notebook (2-3 weeks)

**Components:**
- [ ] Notebook container (HTML/JS)
- [ ] Cell management (add/delete/move)
- [ ] Context panel UI
- [ ] Load/save .kleis files
- [ ] Embed structural editor in cells
- [ ] Basic execution (parse + type check)

**Deliverable:** Can create cells, edit with v2.2 editor, see type info

### Phase 2: Context System (2 weeks)

**Components:**
- [ ] Context parser (extract from .kleis)
- [ ] Type inference engine integration
- [ ] Variable lookup
- [ ] Context merging
- [ ] Import mechanism

**Deliverable:** Full type checking with contexts

### Phase 3: Execution Engine (3 weeks)

**Components:**
- [ ] Expression evaluator
- [ ] Numeric computation
- [ ] Symbolic simplification
- [ ] Result rendering

**Deliverable:** Can run cells and get numerical results

### Phase 4: Polish (1 week)

**Components:**
- [ ] Keyboard shortcuts (Cmd+Enter to run)
- [ ] Cell drag-and-drop reordering
- [ ] Export to PDF/HTML
- [ ] Import from Jupyter (.ipynb)
- [ ] Syntax highlighting for .kleis files

**Deliverable:** Production-ready notebook environment

---

## Key Design Decisions

### Decision 1: Structural Editor for Cells

**Why:** The v2.2 inline editing is perfect for notebook cells!
- Natural typing workflow
- Symbol buttons for non-LaTeX users
- Template buttons for complex structures
- Already debugged and working

**Alternative considered:** Separate Monaco/CodeMirror editor
**Rejected:** Reinventing the wheel, less visual

### Decision 2: .kleis Files for Contexts

**Why:** Plain text, Git-friendly, human-readable
- Easy to version control
- Can edit in any text editor
- Import into notebooks
- Share as packages

**Alternative considered:** Binary format
**Rejected:** Not Git-friendly, not human-readable

### Decision 3: Separate .kleis-nb for State

**Why:** Don't pollute version control with execution outputs
- Source (.kleis) is tracked
- State (.kleis-nb) is gitignored
- Similar to .ipynb vs .py split

**Alternative considered:** Single file format
**Rejected:** Noisy diffs, large files

### Decision 4: Type Inference in Frontend

**Why:** Instant feedback, no network latency
- Type rules can be compiled to WASM
- Run in browser for immediate hints
- Server validates on execution

**Alternative considered:** Server-only type checking
**Rejected:** Slow, network dependency

---

## Example: Complete Workflow

### Step 1: Create Notebook

```
Click "📘 New Notebook" → "Physics Template"
```

### Step 2: Edit Context

```
Context cell (auto-created):
context physics {
    m: Scalar = 1.5  // kg
    v: Vector(3) = [1, 2, 0]  // m/s
}
```

Context panel updates automatically.

### Step 3: Write Equation in Cell

```
Code Cell 1:
Click in cell → Structural editor appears
Click "=" template
Click left placeholder → Inline editor appears
Type "E"
Press Enter
Click right placeholder
Click "fraction" template
Fill numerator: "1" and click "2"
Fill denominator: click "m", "v", "²"
```

Result: `E = ½mv²`

### Step 4: Run Cell

```
Click "▶ Run"

Backend:
1. Receives AST
2. Infers types with context
3. Evaluates expression
4. Returns result

Frontend displays:
┌─ Output ────────────────┐
│ E = ½mv²                │  ← Rendered
│ Type: Scalar ✓          │  ← Type check passed
│ Value: 3.75 J           │  ← Numerical result
└─────────────────────────┘
```

### Step 5: Save

```
File → Save
Writes: classical_mechanics.kleis (source)
Writes: .kleis-nb/classical_mechanics.kleis-nb (state)
```

---

## Mobile/Touch Considerations

### Responsive Design

```css
/* Desktop: Side-by-side layout */
@media (min-width: 1024px) {
    .notebook-container {
        display: grid;
        grid-template-columns: 300px 1fr;
    }
}

/* Tablet: Collapsible sidebar */
@media (max-width: 1023px) {
    .context-panel {
        position: absolute;
        transform: translateX(-100%);
    }
    .context-panel.open {
        transform: translateX(0);
    }
}

/* Mobile: Full-width cells */
@media (max-width: 768px) {
    .cell {
        width: 100%;
    }
    /* Use dialog mode for editing (not inline) */
}
```

---

## Package System

### Importing Contexts

```kleis
# In notebook or .kleis file

import std.physics
import std.calculus
import custom.my_algebra from "./algebras/custom.kleis"

// All symbols from these contexts now available
F = ma  // m, a recognized from std.physics
∇φ = 0  // ∇ recognized from std.calculus
```

### Standard Library Structure

```
stdlib/
  ├── physics.kleis
  │   - Classical mechanics symbols (m, v, F, E, p)
  │   - Constants (c, G, ℏ, k_B)
  │
  ├── calculus.kleis
  │   - Operators (∇, ∂, ∫, Σ, ∏, lim)
  │   - Functions (sin, cos, exp, ln)
  │
  ├── linear_algebra.kleis
  │   - Matrix operations (det, tr, ⊗, ·, ×)
  │   - Vector spaces
  │
  ├── quantum.kleis
  │   - Dirac notation (|ψ⟩, ⟨φ|, ⟨φ|ψ⟩)
  │   - Operators (Ĥ, â, â†)
  │
  └── geometry.kleis
      - Manifolds, tensors (g_μν, R^μν_ρσ, Γ)
      - Differential operators
```

---

## Example .kleis File

```kleis
# physics.kleis - Classical Mechanics Context

context physics {
    // ===== Constants =====
    c: Scalar = 299792458           // m/s - Speed of light
    G: Scalar = 6.674e-11          // N·m²/kg² - Gravitational constant
    k_B: Scalar = 1.380649e-23     // J/K - Boltzmann constant
    ℏ: Scalar = 1.054571817e-34    // J·s - Reduced Planck constant
    
    // ===== Variables =====
    m: Scalar                       // kg - Mass
    v: Vector(3)                    // m/s - Velocity
    a: Vector(3)                    // m/s² - Acceleration
    F: Vector(3)                    // N - Force
    x: Vector(3)                    // m - Position
    t: Scalar                       // s - Time
    
    // ===== Derived Quantities =====
    p: Vector(3) = m * v           // kg·m/s - Momentum
    E_k: Scalar = ½ * m * |v|²     // J - Kinetic energy
    E_p: Scalar                     // J - Potential energy (context-dependent)
    E: Scalar = E_k + E_p          // J - Total energy
    
    // ===== Laws (for verification) =====
    law newtons_second { F = m * a }
    law energy_conservation { d(E)/dt = 0 }
}

export physics
```

---

## Comparison with Jupyter

| Feature | Jupyter | Kleis Notebook |
|---------|---------|----------------|
| **Cell types** | Code, Markdown | Code, Context, Markdown |
| **Language** | Python, Julia, R | Kleis (mathematical expressions) |
| **Editor** | Monaco (text) | Structural + Inline (v2.2) |
| **Type system** | Dynamic | Static with inference |
| **Rendering** | Matplotlib | Built-in Typst |
| **Context** | Global scope | Explicit contexts |
| **File format** | .ipynb (JSON) | .kleis (text) + .kleis-nb (state) |
| **Version control** | Messy (outputs) | Clean (.kleis only) |

---

## Implementation Estimate

### Total Effort: 8-10 weeks

**Phase 1: Basic Notebook** (2-3 weeks)
- Notebook UI shell
- Cell management
- Structural editor integration
- Load/save .kleis files

**Phase 2: Context System** (2 weeks)
- Context parser
- Type inference integration
- Context panel UI
- Variable lookup

**Phase 3: Execution** (3 weeks)
- Evaluator backend
- Expression execution
- Result rendering
- Error handling

**Phase 4: Polish** (1 week)
- Keyboard shortcuts
- Export/import
- Standard library
- Documentation

---

## Next Steps

### Immediate (This Week):
1. ✅ Create ADR-011 (this document)
2. Create mockups for notebook UI
3. Design context file parser
4. Prototype single-cell notebook

### Short-term (Next Month):
1. Implement basic notebook shell
2. Integrate v2.2 structural editor as cell editor
3. Load .kleis files into context
4. Basic type inference

### Long-term (Q1 2025):
1. Full execution engine
2. Standard library contexts
3. Package system
4. Public beta release

---

## Open Questions

1. **Cell output format:** Show just result, or full derivation steps?
2. **Context inheritance:** Hierarchical or flat?
3. **Version compatibility:** How to handle .kleis format changes?
4. **Collaboration:** Real-time editing like Google Colab?
5. **Performance:** How many cells before lag?

---

## Decision

**Recommendation:** Build notebook environment in Q1 2025

**Rationale:**
- v2.2 structural editor is perfect foundation
- Type system docs already exist
- .kleis file format partially defined
- Clear user need for computational environment

**Priority:** High - This completes the Kleis vision
**Complexity:** Medium - Can reuse existing components
**Timeline:** 8-10 weeks for v1.0

---

**Status:** ✅ **Fully Specified - Ready for Prototyping**

Next: Create UI mockups and begin Phase 1 implementation.

