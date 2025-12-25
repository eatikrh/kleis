# Next Session Notes

**Last Updated:** December 25, 2024

---

## 🎯 CRITICAL: Wire DAP to Real Evaluator with DebugHook

### Problem Statement

The DAP server currently **simulates stepping** by incrementing line numbers. It does NOT:
- Set the `DapDebugHook` on the evaluator
- Run actual evaluation
- Support cross-file debugging (stepping into imported files)

### Current State (What Works)

| Component | Status |
|-----------|--------|
| Parser populates `FullSourceLocation` (file + line + column) | ✅ |
| `ExampleStatement` carries location | ✅ |
| Evaluator calls `on_eval_start()` for every expression | ✅ |
| `DapDebugHook` exists with channel-based communication | ✅ |
| DAP returns stack traces with file paths | ✅ |
| VS Code shows debugger UI | ✅ |
| **DAP wires hook to evaluator** | ❌ NOT DONE |
| **Cross-file debugging** | ❌ NOT DONE |

### Architecture (from `REPL_ENHANCEMENTS.md`)

```
┌─────────────────────────────────────────────────────────────┐
│                     kleis server                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
│  │   LSP       │◄─►│  Shared     │◄─►│   DAP       │       │
│  │  Handler    │   │  Context    │   │  Handler    │       │
│  └─────────────┘   │ - Evaluator │   └─────────────┘       │
│                    │ - Types     │                          │
│                    │ - Structs   │                          │
│                    └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

**Key Design Points:**
- **RefCell** ensures zero overhead when not debugging (hook is `None`)
- **DapDebugHook** blocks in evaluator thread, communicates via channels
- **DapDebugController** held by DAP server, sends commands, receives events
- **DO NOT change RefCell** - it's there for a purpose!

### Implementation Plan

#### Step 1: Update `DapState` to Hold Controller

```rust
struct DapState {
    // ... existing fields ...
    
    /// Controller for channel-based communication with DebugHook
    controller: Option<DapDebugController>,
    
    /// Handle to evaluation thread
    eval_thread: Option<std::thread::JoinHandle<()>>,
    
    /// Parsed program (for finding example blocks)
    program: Option<Program>,
}
```

#### Step 2: Wire `launch` Handler

1. Parse file with `parse_kleis_program_with_file(source, canonical_path)`
2. Find first `ExampleBlock` to debug
3. Create `DapDebugHook` + `DapDebugController` via `DapDebugHook::new()`
4. Store controller in `DapState`
5. **Don't start evaluation yet** (wait for `configurationDone`)

#### Step 3: Wire `setBreakpoints` Handler

1. Create `Breakpoint { file, line, enabled: true }` for each
2. Store in `DapState.breakpoints`
3. Will be added to hook before evaluation starts

#### Step 4: Wire `configurationDone` Handler

1. Lock evaluator, set hook: `evaluator.set_debug_hook(hook)`
2. Spawn evaluation thread:
   ```rust
   thread::spawn(move || {
       evaluator.eval_example_block(&example);
       // Send terminated when done
   });
   ```
3. Wait for first `StopEvent` from `controller.event_rx`
4. Send `stopped` event to VS Code

#### Step 5: Wire Step Commands

| DAP Command | DebugAction |
|-------------|-------------|
| `next` | `StepOver` |
| `stepIn` | `StepInto` |
| `stepOut` | `StepOut` |
| `continue` | `Continue` |

1. Send via `controller.command_tx.send(action)`
2. Wait for `StopEvent` from `controller.event_rx`
3. Update `current_file` and `current_line` from event
4. Send `stopped` event to VS Code

#### Step 6: Wire `stackTrace` Handler

- Get stack from `StopEvent.stack`
- Store latest stack in `DapState`
- Return frames with `source.path` (absolute paths)

#### Step 7: Wire `variables` Handler

- Get bindings from top stack frame
- Return as DAP variables

#### Step 8: Handle Evaluation Complete

- Add `Terminated` variant to `StopEvent` (or use channel close)
- Send `terminated` event to VS Code

### Why This Works for Cross-File Debugging

The evaluator calls `on_eval_start` with whatever `SourceLocation` the AST has.
When stepping into a function from an imported file, the AST node has that file's path.
The hook receives it, checks breakpoints, sends stop event with the correct file.
**No per-construct hardcoding needed.**

### Files to Modify

| File | Changes |
|------|---------|
| `src/bin/kleis.rs` | Update `DapState`, wire handlers |
| `src/debug.rs` | Add `Terminated` event (if needed) |

### Technical Debt to Address

**1. Consolidate DAP Implementations**
- `src/dap.rs` — Library version (marked `#[deprecated]`)
- `src/bin/kleis.rs` — Used by `kleis server` (the active one)
- **Action:** Remove `src/dap.rs` after confirming `kleis server` works end-to-end

**2. Review DebugHook Implementations**
We have 3 implementations in `src/debug.rs`:
- `NoOpDebugHook` — Zero overhead when not debugging (KEEP)
- `InteractiveDebugHook` — For REPL `:debug` command (KEEP for REPL)
- `DapDebugHook` — For VS Code DAP integration (KEEP for DAP)

**Action:** After wiring is complete, review if `InteractiveDebugHook` and `DapDebugHook` can share more code or if the separation is justified.

**3. Squash Commits Before Merging**
The `feature/debugger-dap` branch has 63+ incremental commits. Before merging to `main`, squash into logical commits:
- "Add example blocks and assert to grammar (v0.93)"
- "Implement REPL :debug command"  
- "Add DAP infrastructure for VS Code debugging"
- "Add source location tracking to parser"
- "Wire DAP to evaluator with DapDebugHook"

**Command:** `git rebase -i origin/main` then squash/fixup related commits.

### Test Plan

1. Set breakpoint in `examples/debug_main.kleis` on line 8
2. Set breakpoint in `examples/debug_helper.kleis` on line 6
3. Start debugging `debug_main.kleis`
4. Should stop at line 8
5. Step over to line 11 (`let doubled = double(x)`)
6. Step into → should jump to `debug_helper.kleis` line 6
7. Step out → should return to `debug_main.kleis`

### Key Documents

1. **`docs/plans/REPL_ENHANCEMENTS.md`** — Master plan, Phase 6 (Debugging)
2. **`docs/plans/EXPRESSION_SPANS.md`** — Future: spans on all Expressions
3. **`src/debug.rs`** — DebugHook trait and DapDebugHook implementation

---

## 📋 Previous: Debugger Status Before Wiring

| Feature | Status |
|---------|--------|
| Launch/attach | ✅ |
| Breakpoints (set) | ✅ |
| Breakpoints (hit) | ⚠️ Simulated, not real |
| Step in/over/out | ⚠️ Simulated line increment |
| Continue | ⚠️ Simulated |
| Stack trace | ✅ Correct file paths |
| Variables | ✅ From evaluator |
| Cross-file | ❌ Not working |

### Files to Review

- `src/bin/kleis.rs` — Unified binary (DAP implementation here)
- `src/debug.rs` — DebugHook trait and DapDebugHook
- `src/evaluator.rs` — Calls debug hooks at key points
- `vscode-kleis/src/extension.ts` — VS Code integration

---

## ✅ DONE: Matrix Arithmetic Type Inference Fix

**Problem:** `minus(Matrix, Matrix)` was incorrectly returning `Scalar` type.

**Root Cause:** The hardcoded type hierarchy in `type_inference.rs` (lines 1401-1489) checked for Complex, Rational, Scalar, Int, Nat but **never checked for Matrix**. If nothing matched, it defaulted to Scalar.

**Fix:** Added Matrix handling before the default fallback (lines 1474-1485):
```rust
// Check for Matrix - if either arg is Matrix, return that Matrix type
if let Type::Data { constructor, .. } = &t1 {
    if constructor == "Matrix" {
        return Ok(t1.clone());
    }
}
// ... similar for t2
```

**Future Work (TODO #10):** Per ADR-016, all ~400 lines of hardcoded type logic should move to `stdlib/prelude.kleis` structures and be queried from the registry. Current approach works but isn't self-hosting.

---

## ✅ DONE: Equation Editor `let x =` Template

Added `let_simple` template for 2-argument let bindings:
- Button in "Logic & Set Theory" palette
- Template in `std_template_lib/logic.kleist`
- Implemented for ℝ, Matrix, and Bool types in `stdlib/prelude.kleis`

---

## 🎯 Equation Editor: Add `let x =` Template

The equation editor needs a template for let bindings:

```
let x = [value] in [body]
```

This allows users to define local variables in the visual editor.

**Files to modify:**
- `static/index.html` - Add button/template
- Template structure: `Let { pattern: "x", value: Placeholder, body: Placeholder }`

---

## 🎯 Equation Editor: Set Type Templates

The Equation Editor should support Set operations with proper type inference.

**Current Status:**
- ✅ REPL can infer Set types: `insert(5, empty_set)` → `Set(Int)`
- ❌ Equation Editor doesn't have Set operation templates

**Needed templates:**
- `in_set(x, S)` - membership test (x ∈ S)
- `union(A, B)` - set union (A ∪ B)
- `intersect(A, B)` - intersection (A ∩ B)
- `difference(A, B)` - difference (A \ B)
- `subset(A, B)` - subset test (A ⊆ B)
- `empty_set` - empty set (∅)
- `singleton(x)` - singleton set ({x})
- `insert(x, S)` - add element

**Files to modify:**
- `static/index.html` - Add buttons to palette
- `std_template_lib/sets.kleist` - Template definitions
- `src/render_editor.rs` - Rendering templates
- `patternfly-editor/` - PatternFly integration

**Leave for future branch:** `feature/equation-editor-sets`

---

## ⚠️ Program Synthesis: Documented Limitation

**The Dream:** `spec → Z3 → program`

**The Reality:** Z3 cannot synthesize recursive programs from grammar. We tried and documented the failure in `feature/program-synthesis` branch.

**What works:**
- Sketch-based synthesis (human provides template, Z3 fills parameters)
- Bounded verification (sort 2-3 elements)
- LLM proposes, Z3 verifies

**Architecture going forward:**
```
LLM → proposes program → Z3 → verifies properties
                              ✓ or counterexample
```

See `docs/vision/VERIFIED_SOFTWARE_DREAM.md` (in abandoned branch) for full analysis.

---

## ✅ DONE: LISP Interpreter in Kleis

- ✅ Parser (recursive descent, S-expressions)
- ✅ Evaluator (arithmetic, lambda, let, letrec)  
- ✅ Recursion: `fib(10) = 55`, `fact(5) = 120`
- ✅ Documented in manual appendix
- ✅ `:eval` command for concrete execution
- ❌ `(verify ...)` form — **CANCELLED** (program synthesis doesn't work as envisioned)

---

## ✅ DONE: LISP Interpreter Uses stdlib Ordering Operations

The LISP interpreter (`docs/grammar/lisp_parser.kleis`) already:
1. ✅ Imports `stdlib/prelude.kleis`
2. ✅ Uses `le`, `lt`, `gt`, `ge`, `eq` from stdlib `Ordered(T)` structure

No changes needed - this was already working correctly.

---

## ✅ DONE: Type Inference for User-Defined Types

Fixed Dec 21, 2024:
- `:load` now registers data types with TypeChecker
- `:type VNum(42)` → `VNum(Scalar)` ✅
- `:type SAtom("hello")` → `SAtom("hello")` ✅

---

## 📝 Key Learnings (Dec 21, 2024)

1. **Kleis is Turing complete** — proved by implementing LISP interpreter
2. **Data constructors create concrete objects** — not just symbols
3. **Z3 cannot unroll recursion over unbounded ADTs** — fundamental limitation
4. **`:eval` enables execution** — concrete evaluation in Rust
5. **Verification ≠ Synthesis** — Z3 verifies, LLMs synthesize

---

## 🚫 CANCELLED: Implement `(verify ...)` in LISP Interpreter

**Reason:** The program synthesis vision didn't work. Z3 can't evaluate LISP programs symbolically, so `(verify ...)` can't use Z3 the way we hoped.

### What We Have
- ✅ LISP parser (recursive descent, S-expressions)
- ✅ LISP evaluator (arithmetic, comparisons, lambda, let, letrec)
- ✅ Recursion working: `fib(10) = 55`, `fact(5) = 120`
- ✅ Documented in manual appendix

### What We Need to Design
1. **How does `(verify expr)` call Z3?**
   - Option A: Translate LISP → Kleis expression → Z3
   - Option B: Direct LISP → Z3 (bypass Kleis translation)
   - Option C: Add Z3 access to Rust evaluator as a built-in

2. **What syntax for quantifiers?**
   - `(forall (x) (= (+ x 0) x))` - LISP-style
   - How to specify types for quantified variables?

3. **Return value on failure?**
   - `VBool(false)` vs `VSym("Counterexample: x = 42")`

### Why This Matters
See `docs/vision/VERIFIED_SOFTWARE_VISION.md` — this enables:
- Programs with embedded proofs
- Design-by-contract with verification
- The path to "correct by construction" software

### Files to Modify
- `docs/grammar/lisp_parser.kleis` - Add verify form
- `src/evaluator.rs` - May need Z3 integration
- `docs/manual/src/appendix/lisp-interpreter.md` - Update with new code

---

## 🎯 PRIORITY: Bourbaki Compliance Roadmap

Based on capability assessment (Dec 19, 2025), here's what's needed to increase Bourbaki coverage from ~15-20% to higher levels.

### Priority 1: Parser Fixes ✅ COMPLETE (Grammar v0.9)

**Status: DONE** (Dec 22, 2025) - All parser issues resolved!

| Issue | Status | Verified By |
|-------|--------|-------------|
| **∀ inside ∧** | ✅ Works | `tests/grammar_v09_test.rs::test_quantifier_in_conjunction` |
| **Function types in quantifiers** | ✅ Works | `tests/grammar_v09_test.rs::test_function_type_with_nested_quantifier` |
| **→ as implication** | ✅ Works | Used throughout axiom definitions |
| **ε-δ limit definition** | ✅ Works | `tests/grammar_v09_test.rs::test_epsilon_delta_limit` |

**Impact:** Full ε-δ analysis definitions, nested quantifiers, and function types in quantifiers all work.

**Next Steps:** Priorities 2-5 are pure Kleis stdlib code (no more Rust changes needed).

### Priority 2: Set Theory in stdlib (Foundation) 📚

Set(T) exists but operations need defining:

```kleis
// Add to stdlib/sets.kleis:
structure SetTheory(X) {
    operation (⊆) : Set(X) × Set(X) → Bool
    operation (∪) : Set(X) × Set(X) → Set(X)
    operation (∩) : Set(X) × Set(X) → Set(X)
    operation 𝒫 : Set(X) → Set(Set(X))
    element ∅ : Set(X)
    
    axiom subset_def: ∀(A B : Set(X)). A ⊆ B ↔ ∀(x : X). in_set(x, A) → in_set(x, B)
    axiom union_def: ∀(A B : Set(X), x : X). in_set(x, A ∪ B) ↔ in_set(x, A) ∨ in_set(x, B)
    axiom power_set_def: ∀(S A : Set(X)). in_set(A, 𝒫(S)) ↔ A ⊆ S
}
```

**Impact:** Enables Bourbaki Vol I (Set Theory foundations).

### Priority 3: Topology in stdlib 🌐

Now verified to be expressible:

```kleis
// Add to stdlib/topology.kleis:
structure TopologicalSpace(X) {
    element tau : Set(Set(X))
    
    axiom empty_open: in_set(∅, tau)
    axiom full_open: in_set(X, tau)
    axiom union_closed: ∀(U V : Set(X)). in_set(U, tau) ∧ in_set(V, tau) → in_set(union(U, V), tau)
    axiom intersection_closed: ∀(U V : Set(X)). in_set(U, tau) ∧ in_set(V, tau) → in_set(intersect(U, V), tau)
}

structure Continuous(X, Y) over TopologicalSpace(X), TopologicalSpace(Y) {
    operation f : X → Y
    axiom continuity: ∀(V : Set(Y)). in_set(V, tau_Y) → in_set(preimage(f, V), tau_X)
}
```

**Impact:** Enables Bourbaki Vol III (Topology).

### Priority 4: Analysis Structures 📈

```kleis
// Add to stdlib/analysis.kleis:
structure MetricSpace(X) {
    operation d : X × X → ℝ
    
    axiom non_negative: ∀(x y : X). d(x, y) >= 0
    axiom identity: ∀(x y : X). d(x, y) = 0 ↔ x = y
    axiom symmetry: ∀(x y : X). d(x, y) = d(y, x)
    axiom triangle: ∀(x y z : X). d(x, z) <= d(x, y) + d(y, z)
}

structure Limit {
    // Requires parser fix for nested quantifiers
    axiom epsilon_delta: ∀(L a : ℝ, epsilon : ℝ) where epsilon > 0.
        ∃(delta : ℝ). delta > 0
}
```

**Impact:** Enables Bourbaki Vol IV (Analysis), after parser fixes.

### Priority 5: ZFC Axioms (Long-term) 🏛️

```kleis
// Add to stdlib/foundations/zfc.kleis:
structure ZFC {
    // Extensionality
    axiom extensionality: ∀(A B : Set). (∀(x). in_set(x, A) ↔ in_set(x, B)) → A = B
    
    // Pairing
    axiom pairing: ∀(a b). ∃(c : Set). in_set(a, c) ∧ in_set(b, c)
    
    // Union
    axiom union: ∀(F : Set(Set)). ∃(U : Set). ∀(x). in_set(x, U) ↔ ∃(A : Set). in_set(A, F) ∧ in_set(x, A)
    
    // Power Set
    axiom power: ∀(A : Set). ∃(P : Set). ∀(B : Set). in_set(B, P) ↔ B ⊆ A
    
    // Infinity (requires ordinals)
    // axiom infinity: ...
}
```

**Impact:** Full foundational rigor, but Z3 verification may struggle with some axioms.

---

## ⚠️ Z3 Capabilities (Clarified Dec 19, 2025)

**Z3 CAN verify (no Kleis implementation needed):**
- Arithmetic: `∀(n : ℕ). n + 0 = n` ✅
- Algebra: `∀(a b : ℝ). (a-b)*(a+b) = a²-b²` ✅
- Logic: De Morgan, distributivity ✅
- Most Bourbaki-style axioms about ℝ, ℂ, topology ✅

**Z3 struggles with:**

| Limitation | Example | Status |
|------------|---------|--------|
| **Structural induction** | `length(xs ++ ys) = length(xs) + length(ys)` | May timeout |
| **Limits/Convergence** | ε-δ proofs with nested quantifiers | May timeout |
| **Type-level arithmetic** | `Vec(m+n)` from `Vec(m) ++ Vec(n)` | Not expressible |

**Key insight:** Bourbaki is mostly continuous math (ℝ, ℂ, topology) where Z3 works well. Structural induction on lists/trees is rare in Bourbaki.

---

## ✅ Recently Completed

### Operator Overloading (Dec 19, 2025)
- Natural arithmetic: `3 + 4*i = complex(3, 4)` ✅
- Type-directed lowering working
- 17 integration tests

### Capability Assessment (Dec 19, 2025)
- Verified Kleis capabilities against Bourbaki
- Found more works than expected (~15-20% not 5%)
- Documented real limitations

---

## 📊 Current Stats

| Metric | Value |
|--------|-------|
| Tests | 663+ passing |
| Commits | 840+ |
| ADRs | 23 |
| Grammar | v0.8 |
| Unique Cloners | 505+ |
| Bourbaki Coverage | ~15-20% (axiomatic) |

---

## 🏗️ Architecture Notes

### Operator Overloading Pipeline

```
Parser → Type Inference → Lowering → Z3 Backend
                              ↓
              Rewrites: plus(ℂ, ℂ) → complex_add
                        times(ℝ, ℂ) → complex_mul(lift, _)
```

### Bourbaki Coverage Path

```
Current: Basic Algebra (Groups, Rings, Fields, Vector Spaces)
    ↓ Priority 1-2 (parser + set theory)
Next: Set Theory foundations
    ↓ Priority 3
Next: Topology (open sets, continuity)
    ↓ Priority 4
Next: Analysis (limits, metric spaces)
    ↓ Priority 5
Long-term: ZFC foundations
    ↓ New backend
Ultimate: Induction, transfinite, category theory
```

---

*See `docs/CAPABILITY_ASSESSMENT.md` for full analysis.*
