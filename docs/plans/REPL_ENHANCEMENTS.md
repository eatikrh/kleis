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

### Phase 1: Provenance Tracking (This Branch)
- [ ] Add `DefinitionSet` struct
- [ ] Track file → definitions mapping in REPL
- [ ] Update `load_file_recursive` to record provenance
- [ ] Add `:sources` command to show loaded files

### Phase 2: Unload/Reload
- [ ] Add `unload_file` method to Evaluator
- [ ] Add `remove` method to StructureRegistry
- [ ] Implement `:unload` command
- [ ] Implement `:reload` command
- [ ] Implement `:reset` command

### Phase 3: Dependency Tracking
- [ ] Build dependency graph during load
- [ ] Detect what needs recomputation on change
- [ ] Warn about broken dependencies on unload

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

