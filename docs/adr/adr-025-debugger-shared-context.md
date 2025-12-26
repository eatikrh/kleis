# ADR-025: Debugger Shared Context Architecture

## Status
Accepted

## Date
2024-12-25

## Context

Kleis needs a debugger that integrates with VS Code via the Debug Adapter Protocol (DAP).
The debugger must:

1. **Share parsed ASTs with LSP** - Avoid re-parsing files that LSP already parsed
2. **Support cross-file debugging** - Step into imported files
3. **Handle concurrent access** - LSP and DAP run on different threads
4. **Maintain consistency** - When a file changes, dependents must be re-parsed

The challenge: `Evaluator` contains `RefCell<Option<Box<dyn DebugHook>>>` which is not `Sync`.
This prevents sharing the evaluator directly across threads.

## Decision

### Thread-Safe AST Cache Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Thread-Safe AST Cache                         │
│     Arc<RwLock<HashMap<PathBuf, CachedDocument>>>               │
│                                                                  │
│  CachedDocument {                                                │
│    source: String,                                               │
│    program: Option<Program>,  // The AST                         │
│    imports: HashSet<PathBuf>, // Dependencies                    │
│    dirty: bool,               // Needs re-parse?                 │
│  }                                                               │
└─────────────────────────────────────────────────────────────────┘
           ↑                              ↑
           │ write                        │ read (or write if miss)
           │                              │
    ┌──────┴───────┐               ┌──────┴───────┐
    │     LSP      │               │     DAP      │
    │  (Thread 1)  │               │  (Thread 2)  │
    │              │               │              │
    │  Evaluator   │               │  Evaluator   │
    │  (own copy)  │               │  (own copy)  │
    └──────────────┘               └──────────────┘
```

### Key Design Decisions

| Component | Design | Reason |
|-----------|--------|--------|
| AST Cache | `Arc<RwLock<HashMap<PathBuf, CachedDocument>>>` | Thread-safe, shared between LSP/DAP |
| Evaluator | Per-thread copy | Has `RefCell` (not `Sync`) |
| `imports` | `HashSet<PathBuf>` | Fast lookup for dependency tracking |
| Eviction | Mark dirty, lazy re-parse | Only parse when needed |

### Cascade Invalidation Algorithm

When a file changes:

```rust
fn invalidate_dependents(cache: &mut HashMap<PathBuf, CachedDocument>, path: &PathBuf) {
    // Mark the changed document as dirty
    if let Some(doc) = cache.get_mut(path) {
        doc.dirty = true;
    }

    // Keep iterating until no new documents are marked dirty
    loop {
        let mut newly_dirtied = false;
        let dirty_paths: HashSet<PathBuf> = cache
            .iter()
            .filter(|(_, doc)| doc.dirty)
            .map(|(p, _)| p.clone())
            .collect();

        for (_, doc) in cache.iter_mut() {
            if !doc.dirty && doc.imports.iter().any(|imp| dirty_paths.contains(imp)) {
                doc.dirty = true;
                newly_dirtied = true;
            }
        }

        if !newly_dirtied { break; }
    }
}
```

This is analogous to incremental compilation: if B changes, all files that import B must be re-analyzed.

### Eviction Policy

Dirty ASTs are **replaced** (re-parsed on access), not **ejected** (removed).

Safe to evict only when:
- File is closed in editor AND
- No other cached file imports it

For typical Kleis projects (10-50 files), no eviction is needed.

## Consequences

### Positive
- **Performance**: Parse each file once, reuse across LSP and DAP
- **Consistency**: Single source of truth for ASTs
- **Cross-file debugging**: Breakpoints work across imports
- **Incremental**: Only re-parse what changed

### Negative
- **Complexity**: Must maintain dependency graph
- **Memory**: Cache grows with open files (acceptable for small projects)

### Neutral
- Each thread still has its own `Evaluator` (necessary due to `RefCell`)

## Process Architecture: Why Same-Process Matters

```
VS Code
   │
   ├── Spawns: "kleis server" (ONE process)
   │           │
   │           ├── LSP handler (main thread)
   │           │       │
   │           │       └── Arc<RwLock<AstCache>> ◄────┐
   │           │                                      │ SHARED
   │           └── DAP handler (separate thread)      │
   │                   │                              │
   │                   └── Arc<RwLock<AstCache>> ◄────┘
   │
   └── Connects to DAP via TCP (port from LSP command)
```

**Critical design decision:** The extension uses `DebugAdapterServer` (TCP connection to existing process)
instead of `DebugAdapterExecutable` (spawn new process).

If DAP ran as a separate process, there would be no shared memory and the AST cache
couldn't be reused. Our unified server approach ensures:

1. `kleis server` is spawned once by VS Code for LSP
2. When debugging starts, the extension sends `kleis.startDebugSession` command via LSP
3. The server spawns DAP handler **in the same process, different thread**
4. Returns TCP port, VS Code connects via `DebugAdapterServer(port)`

This is why we use `Arc<RwLock<...>>` (thread-safe) rather than `Rc<RefCell<...>>` (single-threaded).

## DAP-Evaluator Communication via Channels

The DAP server and evaluator run on different threads. They communicate via channels:

```
┌──────────────────┐                    ┌─────────────────────┐
│   DAP Server     │                    │  Evaluator Thread   │
│   (DapState)     │                    │                     │
│                  │                    │                     │
│  controller ─────┼─── command_tx ───► │  DapDebugHook       │
│                  │    DebugAction     │  (blocks in         │
│                  │    {Continue,      │   wait_for_command) │
│                  │     StepInto,      │                     │
│                  │     StepOver,      │                     │
│                  │     StepOut}       │                     │
│                  │                    │                     │
│                  │◄── event_rx ─────  │  DapDebugHook       │
│                  │    StopEvent       │  (sends when        │
│                  │    {location,      │   should_stop=true) │
│                  │     stack,         │                     │
│                  │     reason}        │                     │
└──────────────────┘                    └─────────────────────┘
```

**Key types:**

```rust
pub struct DapDebugHook {
    stack: Vec<StackFrame>,           // Stack frames for IDE
    breakpoints: Vec<Breakpoint>,     // Breakpoints
    command_rx: Receiver<DebugAction>, // Receives from DAP
    event_tx: Sender<StopEvent>,       // Sends to DAP
}

pub struct DapDebugController {
    pub command_tx: Sender<DebugAction>,  // DAP sends commands
    pub event_rx: Receiver<StopEvent>,    // DAP receives events
}
```

**Flow:**
1. `launch`: DAP creates `DapDebugHook` + `DapDebugController`
2. `configurationDone`: DAP sets hook on evaluator, spawns eval thread
3. Evaluator calls `hook.on_eval_start()` at each expression
4. If `should_stop()` is true, hook sends `StopEvent` and blocks on `command_rx`
5. DAP receives `StopEvent`, updates UI, waits for user action
6. User clicks "Step Over" → DAP sends `DebugAction::StepOver` via `command_tx`
7. Hook unblocks, returns `StepOver` to evaluator
8. Evaluator continues until next stop condition

## Implementation

- `src/context.rs` - `Document` struct with `imports: HashSet<PathBuf>` and `dirty: bool`
- `src/bin/kleis.rs` - `AstCache` type alias and `invalidate_dependents()` function
- `vscode-kleis/src/extension.ts` - `KleisDebugAdapterFactory` uses LSP command, not executable spawn
- Tests in `src/context.rs` verify cascade invalidation

## Related ADRs

- ADR-014: Hindley-Milner Type System (type inference architecture)
- ADR-016: Operations in Structures (where Evaluator fits)

