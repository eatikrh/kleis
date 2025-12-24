# REPL Enhancements Plan

## Session Notes (Dec 23, 2024)

This document captures ideas discussed for enhancing the Kleis REPL toward a full IDE experience.

---

## 1. Provenance Tracking

**Problem:** Currently, the REPL only loads files but cannot unload them. Everything gets merged into global hashmaps with no record of where each definition came from.

**Solution:** Track which file each definition originated from.

```rust
struct DefinitionSet {
    functions: Vec<String>,
    data_types: Vec<String>,
    structures: Vec<String>,
    implements: Vec<(String, String)>,  // (structure_name, type)
}

// Track provenance
definitions_by_file: HashMap<PathBuf, DefinitionSet>
```

**Benefits:**
- Enable `:unload file.kleis` command
- Enable `:reload file.kleis` command  
- Enable `:reset` to clear all definitions
- Foundation for IDE integration

---

## 2. REPL as IDE Plugin

**Vision:** The Kleis REPL as a VS Code / IntelliJ plugin with rich IDE integration.

**Components:**

| Component | Technology | Status |
|-----------|------------|--------|
| LSP (Language Server) | `kleis-lsp` | ✅ Exists |
| DAP (Debug Adapter) | `kleis-debug` (new) | ❌ Not started |
| REPL Panel | VS Code Webview | ❌ Not started |
| Preview Panel | HTML/SVG render | ❌ Not started |
| Notebook Mode | Jupyter-like cells | ❌ Not started |

**Features needed:**

- **Unload/Reload files** — When user saves, update definitions
- **Incremental updates** — Only recompute affected parts
- **Multiple workspaces** — Different projects open simultaneously
- **Dependency tracking** — Know what depends on what

---

## 3. Debugging Support

**What a Kleis debugger would debug:**

| Mode | What You Step Through |
|------|----------------------|
| Evaluation (`:eval`) | Function application, pattern matching, substitution |
| Type Inference | Constraint generation, unification, solving |
| Verification | Which axioms Z3 uses, how it searches |
| Rendering | AST → template → output transformation |

**Architecture:** Use DAP (Debug Adapter Protocol) — the debugging equivalent of LSP.

```
┌─────────────────────────────────────────────────────────────┐
│                         EDITOR                               │
└─────────────────────────────────────────────────────────────┘
           │                              │
           │ LSP                          │ DAP
           ▼                              ▼
┌─────────────────────┐      ┌─────────────────────┐
│   kleis-lsp         │      │   kleis-debug       │
│   (editing)         │      │   (stepping)        │
└─────────────────────┘      └─────────────────────┘
```

**DAP features needed:**
- Launch/Attach
- Set breakpoints (on function calls, pattern matches)
- Step In / Step Out / Continue
- Inspect variables (bindings in current scope)
- Call stack (nested function applications)

---

## 4. New REPL Commands

**Proposed commands:**

```
:load file.kleis      # Load (existing)
:unload file.kleis    # NEW: Remove definitions from file
:reload file.kleis    # NEW: Unload + Load
:reset                # NEW: Clear all, back to empty
:deps func_name       # NEW: Show what depends on this
:sources              # NEW: Show which files are loaded
```

---

## 5. Implementation Phases

### Phase 1: Provenance Tracking ✅ DONE
- [x] Add `DefinitionSet` struct — `src/provenance.rs`
- [x] Add `ProvenanceTracker` with reverse lookups
- [x] Track file → definitions mapping in REPL
- [x] Update `load_file_recursive` to record provenance
- [x] Add `:sources` command to show loaded files

### Phase 2: Unload/Reload
- [ ] Add `remove_function()` method to Evaluator
- [ ] Add `remove_structure()` method to StructureRegistry  
- [ ] Add `remove_data_type()` method to Evaluator
- [ ] Implement `:unload` command (with warning)
- [ ] Implement `:reload` command (unload + load, with confirmation)
- [ ] Implement `:reset` command (clear all state)
- [ ] Handle TypeChecker cache invalidation

### Phase 3: Dependency Tracking (Optional Enhancement)
- [ ] Track which functions were called during `:let` evaluation
- [ ] On reload, identify which bindings may be stale
- [ ] Show specific warnings: "bindings X, Y reference 'multiply'"

### Phase 4: IDE Integration
- [ ] Create VS Code extension skeleton
- [ ] Add REPL panel (webview)
- [ ] Connect to kleis-lsp
- [ ] Add file change notifications

### Phase 5: Debugging
- [ ] Create `kleis-debug` binary
- [ ] Implement DAP protocol
- [ ] Add tracing to evaluator
- [ ] Step-through execution

---

## 6. Related Files

- `src/bin/repl.rs` — REPL implementation
- `src/bin/lsp.rs` — Language Server
- `src/evaluator.rs` — Function storage
- `src/structure_registry.rs` — Structure storage
- `src/type_checker.rs` — Type checking
- `src/type_context.rs` — Type context building

---

## 7. Key Insight from Session

> "While designing, it type-checks. If the type checking is correct, you can start rendering."

The REPL should provide real-time feedback as you design, catching errors immediately. This is the "smarts" in the equation editor — verification woven into the design process, not bolted on at the end.

---

## 8. Reload Philosophy: "REPL is a Workspace"

**Decision:** The REPL treats user state as valuable work. Never auto-reload; always ask.

### The Problem

When a user edits a `.kleis` file, what happens to their REPL state?

```
λ> :load matrices.kleis
λ> :let result = multiply(A, B)    # User's work-in-progress
λ> # ... user edits matrices.kleis in IDE ...
λ> # What now?
```

### Design Decision

| Trigger | Behavior |
|---------|----------|
| File changes on disk | **Notify only** — "matrices.kleis modified" |
| User types `:reload` | **Ask if bindings affected** — "Continue? [y/N]" |
| User types `:reset` | **Clear everything** — explicit nuclear option |

### Why Not Auto-Reload?

- User's `:let` bindings are **work in progress**
- Unexpected state loss is an "arrggh!" moment
- Jupyter/GHCi/Clojure all use explicit reload for this reason

### IDE Integration Flow

```
┌─────────────────────────────────────────────────────────────┐
│  IDE notifies: "matrices.kleis changed"                     │
│                                                             │
│  Kleis REPL shows:                                          │
│  📝 matrices.kleis modified on disk.                        │
│     Use :reload matrices.kleis to update.                   │
│     (5 functions, 2 structures would be replaced)           │
│                                                             │
│  Bindings are NOT touched until user says :reload           │
└─────────────────────────────────────────────────────────────┘
```

### Reload with Binding Warning

```
λ> :reload matrices.kleis
⚠️  Warning: 2 bindings may reference 'multiply' which will be replaced:
     - result
     - cached_inverse
   These bindings will be kept but may reference stale definitions.
   Continue? [y/N] y
✅ Reloaded: 5 functions, 2 structures
```

### Tracked Constructs

| Construct | Tracked in Provenance? |
|-----------|------------------------|
| Functions (`define`) | ✅ Yes |
| Data Types (`data`) | ✅ Yes |
| Structures (`structure`) | ✅ Yes |
| Type Aliases (`type`) | ✅ Yes |
| Implements blocks | ✅ Yes |
| Axioms/Theorems | Part of structures |
| `:let` bindings | Session state, not file-based |

