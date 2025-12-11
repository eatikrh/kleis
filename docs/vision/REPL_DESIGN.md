# Kleis REPL - Design Document

**Date:** December 11, 2024  
**Status:** Proposal - Pre-cursor to Kleis Notebook  
**Priority:** After documentation consolidation

---

## Vision

A **Read-Eval-Print Loop** for interactive Kleis development, serving as the stepping stone to the full Kleis Notebook environment.

---

## Why REPL Before Notebook?

**Simpler to implement:**
- ✅ No web UI needed (terminal-based)
- ✅ No cell management complexity
- ✅ No persistence layer
- ✅ Just: parse → execute → print loop

**Provides immediate value:**
- Interactive theory testing
- Quick axiom verification with Z3
- Learn Kleis language interactively
- Debug structures and implementations

**Foundation for notebook:**
- Same execution engine
- Same context management
- Same evaluation logic
- Notebook = REPL + web UI + cells + persistence

**Implementation time:**
- REPL: 2-3 days
- Full Notebook: Weeks

**Smart phasing!**

---

## Example Session

```bash
$ kleis repl
Kleis REPL v0.1.0 
Type :help for commands, :quit to exit

kleis> structure Ring(R) {
  ...>   operation (+) : R → R → R
  ...>   operation (×) : R → R → R
  ...>   axiom commutativity: ∀(x y : R). x + y = y + x
  ...> }
✅ Structure 'Ring' registered

kleis> implements Ring(ℝ) {
  ...>   operation (+) = builtin_add
  ...>   operation (×) = builtin_mul
  ...> }
✅ Implementation 'Ring(ℝ)' loaded

kleis> :verify commutativity for Ring(ℝ)
🔍 Verifying with Z3...
✅ Proven: Valid (0.03s)

kleis> let x = 2 + 3
x : ℝ = 5

kleis> :type x
x : ℝ

kleis> :load theory/pot.kleis
📂 Loading theory/pot.kleis...
✅ Loaded 12 structures
✅ Loaded 8 implementations
✅ Loaded 24 axioms

kleis> let galaxy = ProjectedOntology(Minkowski, SpiralGalaxy)
galaxy : ProjectedOntology(Spacetime, MassDistribution)

kleis> let velocities = orbital_velocity_curve(galaxy, 0..50)
[Computes...]
velocities : List(Tuple(ℝ, ℝ)) = [(0, 0), (5, 180), ...]

kleis> :export velocities to "galaxy_data.csv"
✅ Exported 50 data points to galaxy_data.csv

kleis> :doc Ring
╔══════════════════════════════════════╗
║ Structure: Ring(R)                   ║
╠══════════════════════════════════════╣
║ Operations:                          ║
║   (+) : R → R → R                    ║
║   (×) : R → R → R                    ║
║                                      ║
║ Axioms:                              ║
║   commutativity:                     ║
║     ∀(x y : R). x + y = y + x        ║
╚══════════════════════════════════════╝

kleis> :quit
Goodbye!
```

---

## Core Features

### 1. Expression Evaluation
```
kleis> 2 + 3
5 : ℝ

kleis> Matrix(2, 2, [1, 2, 3, 4])
[[1, 2], [3, 4]] : Matrix(2, 2, ℝ)
```

### 2. Structure Definition
```
kleis> structure Monoid(M) {
  ...>   operation (•) : M → M → M
  ...>   operation e : M
  ...>   axiom identity: ∀(x : M). e • x = x
  ...> }
✅ Structure registered
```

### 3. Loading Files
```
kleis> :load stdlib/quantum.kleis
✅ Loaded quantum structures

kleis> :load theory/pot.kleis
✅ Loaded POT theory
```

### 4. Z3 Verification
```
kleis> :verify associativity for Ring(ℝ)
🔍 Verifying with Z3...
✅ Proven: Valid

kleis> :verify my_conjecture for CustomStructure
🔍 Verifying with Z3...
❌ Invalid: Counterexample found
  x = 1, y = 2 violates the axiom
```

### 5. Context Inspection
```
kleis> :structures
Ring, Field, VectorSpace, Monoid, Group, ...

kleis> :types
ℝ : Field
ℂ : Field
Matrix(m, n, T) : Type

kleis> :axioms Ring
- commutativity: ∀(x y : R). x + y = y + x
- associativity: ∀(x y z : R). (x + y) + z = x + (y + z)
- ...
```

### 6. Type Checking
```
kleis> :type Matrix(2, 2, [1, 2, 3, 4])
Matrix(2, 2, ℝ)

kleis> :check 2 + Matrix(2, 2, [1,2,3,4])
❌ Type error: Cannot add Scalar to Matrix(2, 2, ℝ)
💡 Suggestion: Use scalar_multiply for scalar-matrix operations
```

### 7. Documentation
```
kleis> :doc Matrix
[Renders structure documentation]

kleis> :doc dot
[Shows operation signature and implementations]
```

### 8. Data Export
```
kleis> :export result to "data.csv"
✅ Exported to data.csv

kleis> :export Ring to "ring_docs.html"
✅ Generated documentation at ring_docs.html
```

---

## REPL Commands

**Evaluation:**
- `<expression>` - Evaluate expression
- `let x = <expr>` - Bind variable
- `:type <expr>` - Show type
- `:check <expr>` - Type check without evaluation

**Loading:**
- `:load <file.kleis>` - Load Kleis file
- `:reload` - Reload all files
- `:use <module>` - Import module

**Inspection:**
- `:structures` - List all structures
- `:types` - List known types
- `:axioms <structure>` - Show axioms
- `:doc <name>` - Show documentation
- `:context` - Show current bindings

**Verification:**
- `:verify <axiom> for <structure>` - Z3 verification
- `:prove <proposition>` - Prove statement

**Export:**
- `:export <var> to <file>` - Export data
- `:export <structure> to <file>` - Generate docs

**Utility:**
- `:help` - Show help
- `:clear` - Clear context
- `:history` - Show command history
- `:quit` - Exit REPL

---

## Architecture

### Components Needed

**1. REPL Loop** (new)
```rust
// src/bin/repl.rs
loop {
    print!("kleis> ");
    let input = read_line()?;
    
    match parse_repl_command(input) {
        Command::Expression(expr) => evaluate_and_print(expr),
        Command::LoadFile(path) => load_kleis_file(path),
        Command::Verify(axiom) => verify_with_z3(axiom),
        Command::Help => show_help(),
        Command::Quit => break,
    }
}
```

**2. Context Manager** (new)
```rust
// src/repl_context.rs
struct ReplContext {
    type_checker: TypeChecker,
    bindings: HashMap<String, (Expression, Type)>,
    loaded_files: Vec<String>,
}
```

**3. Evaluator** (enhance existing)
```rust
// src/evaluator.rs - already exists!
// Just needs integration with REPL context
```

**4. Output Formatter** (new)
```rust
// Format results for terminal
// Unicode rendering + type annotations
```

### Existing Components (Reuse)

- ✅ Parser (kleis_parser.rs)
- ✅ Type checker (type_checker.rs)
- ✅ Z3 integration (axiom_verifier.rs)
- ✅ Renderer (render.rs with Unicode output)
- ✅ Structure registry
- ✅ Evaluator (basic structure exists)

**Mostly assembly work, not building from scratch!**

---

## Implementation Plan

### Phase 1: Basic REPL (4-6 hours)
1. Create `src/bin/repl.rs`
2. Implement read-eval-print loop
3. Parse expressions
4. Evaluate simple expressions
5. Print results with types

**Deliverable:** Can evaluate `2 + 3` and see result

### Phase 2: Context Management (3-4 hours)
1. Variable bindings (`let x = 5`)
2. Persistent type checker
3. Structure registration
4. Implementation loading

**Deliverable:** Can define structures and use them

### Phase 3: File Loading (2-3 hours)
1. `:load` command
2. Parse and register .kleis files
3. Error handling
4. Multiple file support

**Deliverable:** Can load stdlib and custom theories

### Phase 4: Z3 Integration (2-3 hours)
1. `:verify` command
2. Axiom verification workflow
3. Pretty-print proof results
4. Counterexample display

**Deliverable:** Interactive theorem proving

### Phase 5: Polish (2-3 hours)
1. Command history (readline)
2. Tab completion
3. Better error messages
4. Help system
5. Syntax highlighting (optional)

**Deliverable:** Professional REPL experience

**Total:** 13-19 hours (~2-3 days of focused work)

---

## Use Cases

### 1. Theory Development
```
# Edit theory/pot.kleis in editor
# Switch to REPL
kleis> :reload
kleis> :verify causality for Hont
# See if axiom holds
# Back to editor to fix
# Iterate rapidly
```

### 2. Learning Kleis
```
kleis> :load stdlib/minimal_prelude.kleis
kleis> :structures
kleis> :doc Ring
# Interactive exploration
```

### 3. Quick Calculations
```
kleis> Matrix(2,2,[1,2,3,4]) + Matrix(2,2,[5,6,7,8])
[[6,8],[10,12]] : Matrix(2,2,ℝ)
```

### 4. Axiom Verification
```
kleis> :load theory/my_theory.kleis
kleis> :verify my_axiom for MyStructure
# Fast feedback loop
```

### 5. Data Generation
```
kleis> let curve = orbital_velocities(galaxy, 0..50)
kleis> :export curve to "data.csv"
# Feed to matplotlib/gnuplot
```

---

## Path to Notebook

**REPL (2-3 days) →** Command-line interaction  
**Notebook (2-3 weeks) →** Add:
- Web UI for cells
- Rich output (rendered math, graphs)
- Cell persistence (save/load notebooks)
- Execution order management
- Markdown cells

**But REPL gives immediate value** while building toward notebook!

---

## Next Steps

**After doc consolidation (next session):**
1. Design REPL architecture (1-2 hours)
2. Implement Phase 1: Basic REPL (4-6 hours)
3. Test and iterate

**Milestone:** Working REPL for theory development

**Then:** Notebook becomes "REPL + web UI"

---

**This is the right progression:** 
CLI REPL → Web Notebook → Full Theory Development Environment

Each step provides value, each step builds on the previous.

