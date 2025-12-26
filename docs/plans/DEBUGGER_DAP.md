# Kleis Debugger — DAP Implementation Plan

## Overview

Implement a Debug Adapter Protocol (DAP) server for Kleis, enabling step-through debugging in VS Code and other DAP-compatible editors.

## What DAP Enables

| Feature | Description |
|---------|-------------|
| **Breakpoints** | Pause execution at specific lines/functions |
| **Step In/Out/Over** | Navigate through evaluation |
| **Variable Inspection** | See bindings and their values |
| **Call Stack** | View nested function applications |
| **Watch Expressions** | Evaluate expressions in current context |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         EDITOR                               │
│                   (VS Code, IntelliJ, etc.)                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ DAP (JSON over stdio)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      kleis-debug                             │
│                   (Debug Adapter Server)                     │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ DAP Handler │  │ Breakpoint  │  │ Stepping Controller │  │
│  │             │  │ Manager     │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                              │                               │
│                              ▼                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Instrumented Evaluator                      │ │
│  │   (Evaluator with hooks for pause/step/inspect)         │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## DAP Protocol Basics

DAP uses JSON-RPC over stdio. Key message types:

### Requests (Editor → Debugger)
- `initialize` — Capabilities exchange
- `launch` / `attach` — Start debugging
- `setBreakpoints` — Set breakpoints in a file
- `continue` — Resume execution
- `next` — Step over
- `stepIn` — Step into function
- `stepOut` — Step out of function
- `threads` — List threads (we have 1)
- `stackTrace` — Get call stack
- `scopes` — Get variable scopes
- `variables` — Get variables in a scope

### Events (Debugger → Editor)
- `initialized` — Ready for breakpoints
- `stopped` — Execution paused (breakpoint/step)
- `terminated` — Debugging ended
- `output` — Console output

---

## What to Debug in Kleis

Kleis evaluation involves several interesting points:

### 1. Expression Evaluation
```kleis
:eval multiply(A, B)
       ^-- Step into multiply
       ^-- See A and B values
       ^-- Watch the result form
```

### 2. Pattern Matching
```kleis
define fib(0) = 0
define fib(1) = 1
define fib(n) = fib(n-1) + fib(n-2)
       ^-- Which pattern matched?
       ^-- What is n?
```

### 3. Let Bindings
```kleis
let x = expensive_computation in
    x + x
^-- Pause after x is computed
```

### 4. Function Application
```kleis
f(g(x))
  ^-- Step into g first
^-- Then step into f
```

---

## Implementation Phases

### Phase 1: DAP Server Skeleton
- [ ] Create `src/bin/debug.rs`
- [ ] Add DAP protocol types (or use `dap` crate)
- [ ] Implement `initialize` / `launch` / `terminate`
- [ ] Handle JSON-RPC over stdio

### Phase 2: Instrumented Evaluator
- [ ] Add `EvaluatorState` enum (Running, Paused, Stepping)
- [ ] Add hooks in `eval()` for breakpoint checking
- [ ] Track source locations through evaluation
- [ ] Capture variable bindings at each step

### Phase 3: Breakpoints
- [ ] Implement `setBreakpoints` handler
- [ ] Map source locations to AST nodes
- [ ] Check breakpoints during evaluation
- [ ] Send `stopped` event when hit

### Phase 4: Stepping
- [ ] Implement `next` (step over)
- [ ] Implement `stepIn` (step into function)
- [ ] Implement `stepOut` (step out of function)
- [ ] Track call depth for step over/out

### Phase 5: Variable Inspection
- [ ] Implement `scopes` (local, global)
- [ ] Implement `variables` (list bindings)
- [ ] Format complex values (matrices, lists)
- [ ] Support nested inspection (expand structures)

### Phase 6: VS Code Integration
- [ ] Add debug configuration to `vscode-kleis`
- [ ] Create `launch.json` template
- [ ] Connect extension to `kleis-debug`

---

## Files to Create

| File | Purpose |
|------|---------|
| `src/bin/debug.rs` | DAP server main |
| `src/debug/mod.rs` | Debug module |
| `src/debug/dap_types.rs` | DAP protocol types |
| `src/debug/handler.rs` | Request handlers |
| `src/debug/breakpoints.rs` | Breakpoint management |
| `src/debug/evaluator.rs` | Instrumented evaluator |
| `vscode-kleis/src/debugProvider.ts` | VS Code debug config |

---

## Crate Options

### Option A: Use `dap` crate
```toml
[dependencies]
dap = "0.4"
```
Provides protocol types and server framework.

### Option B: Use `debug-adapter-protocol` crate
```toml
[dependencies]
debug-adapter-protocol = "0.1"
```
Just the types, we handle I/O.

### Option C: Hand-roll (educational but slower)
Define our own types matching DAP spec.

**Recommendation:** Start with `dap` crate for faster development.

---

## Example Debug Session

```
1. User opens prelude.kleis in VS Code
2. User sets breakpoint on line 10 (inside `fib`)
3. User starts debugging with :eval fib(5)
4. Debugger loads file, sets breakpoints
5. Evaluation begins
6. When fib is called, breakpoint hits
7. Editor shows: n = 5, call stack, locals
8. User steps through recursion
9. User inspects intermediate values
10. Evaluation completes, debug session ends
```

---

## Challenges

1. **Source Mapping** — AST nodes need source locations
2. **Lazy Evaluation** — When does evaluation "happen"?
3. **Recursion** — Deep stacks in functional code
4. **Performance** — Debugging overhead must be minimal
5. **Multiple Files** — Breakpoints across imports

---

## References

- [DAP Specification](https://microsoft.github.io/debug-adapter-protocol/)
- [dap crate](https://crates.io/crates/dap)
- [VS Code Debug Extension Guide](https://code.visualstudio.com/api/extension-guides/debugger-extension)

