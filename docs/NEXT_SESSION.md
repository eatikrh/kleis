# Next Session Notes

**Last Updated:** December 26, 2024 (Late Evening)

---

## ✅ DONE: DAP Debugger Fully Working! (Dec 26, 2024)

### What Works
- ✅ Cross-file debugging (VS Code opens imported files)
- ✅ Correct line numbers for ALL operation types (arithmetic, logical, comparison)
- ✅ Breakpoints work in both main and imported files
- ✅ Variables panel shows AST expressions (symbolic representation!)
- ✅ Stack frames tracked correctly
- ✅ Step over, step into, step out all work
- ✅ **assert() uses Z3 for symbolic verification!**

### Key Insight: DAP as a Window to Kleis Internals
The debugger shows variables as **AST expressions**, not evaluated values:
```
doubled = Operation { name: "plus", args: [Object("x"), Object("x")], span: ... }
x = Const("10")
```

This is **exactly right** for a symbolic mathematics system! Variables hold
symbolic expressions that can be passed to Z3 for verification.

### Fixes Applied (Dec 26, 2024)
1. **Skip expressions without spans** - No more line 1 spurious stops
2. **Parser span capture at START** - Fixed 8 parsing functions to capture span
   before parsing, not after (parse_arithmetic, parse_term, parse_factor,
   parse_comparison, parse_conjunction, parse_disjunction, parse_implication,
   parse_biconditional, parse_where_term)
3. **Fixed double pop_frame bug** - Removed redundant pop_frame() call
4. **Custom operator spans** - Fixed parse_where_term

### Future Ideas

#### 1. Eval Command in Debug Panel
Add ability to evaluate an AST expression to a concrete value during debugging.
The infrastructure exists (`evaluator.eval()`).

#### 2. Extend `example` Block Grammar
Current grammar only allows: `let`, `assert`, expressions.

**Could add:**
```kleis
example "test" {
    define local_fn(x) = x + 1   // Local function definition
    let y = local_fn(5)
    assert(y = 6)
}
```

**Pros:** Self-contained test cases, useful for testing helpers
**Cons:** `example` is for testing not defining; functions can be top-level

#### 3. ✅ Wire assert() to Z3 - DONE! (Dec 26, 2024)
**IMPLEMENTED!** `assert()` in example blocks now uses Z3 for symbolic verification:

```kleis
structure CommutativeRing(R) {
    operation (+) : R × R → R
    axiom commutativity: ∀(a b : R). a + b = b + a
}

example "test commutativity" {
    assert(x + y = y + x)  // ✅ Z3 verifies this using the commutativity axiom!
}
```

**How it works:**
1. `eval_assert()` checks if expressions are symbolic (`is_symbolic()`)
2. If symbolic → calls `verify_with_z3()` using `AxiomVerifier`
3. Z3 loads structure axioms and verifies/disproves the assertion
4. Results: `Verified`, `Disproved { counterexample }`, or `Unknown`

**Test cases added:**
- `test_assert_with_z3_verification` - Verifies commutativity axiom
- `test_assert_associativity` - Verifies associativity axiom  
- `test_assert_invalid_symbolic` - Z3 correctly disproves `x + y = y + y`
- `test_assert_concrete_values` - Structural equality for bound variables

---

---

## ✅ DONE: Type Promotion (Lift) Implemented (Dec 26, 2024)

### What Was Fixed

The type checker now correctly promotes types through the `Promotes` structure.

**Before:** `:type 1 + sin(x)` → `Int` ❌
**After:** `:type 1 + sin(x)` → `Scalar` ✅

### Bugs Fixed

1. **OperationRegistry.merge() missing fields**
   - Added merge for `structure_extends` and `type_promotions`
   - Without this, promotions registered in stdlib weren't available to type checker

2. **Unicode type names not normalized when registering**
   - `implements Promotes(ℕ, ℤ)` was registering as `("ℕ", "ℤ")`
   - But `has_promotion` and `find_common_supertype` normalize to `("Nat", "Int")`
   - Fix: Normalize in `register_implements` before storing

3. **Top-level operations not registered**
   - Operations like `operation sin : ℝ → ℝ` were ignored (TODO stub)
   - Added `toplevel_operation_types` to `OperationRegistry`
   - Type inference now queries these for function return types

4. **Added type_expr_to_type helper**
   - Converts `TypeExpr` to `Type` for return type extraction
   - Handles Function, Named, Parametric, Product, ForAll, DimExpr

### Test Results

All 8 type promotion tests pass:
- `:type sin(x) = Scalar` ✅ (was `Var(TypeVar(0))`)
- `:type 1 + sin(x) = Scalar` ✅ (was `Int`)
- `:type (1 + sin(x)) / 2 = Scalar` ✅ (was `Int`)
- `:type 1 + 3.14 = Scalar` ✅
- Promotions registered: Nat→Int, Int→Scalar, etc. ✅

### Files Modified
- `src/type_context.rs` - Major fixes to registry and type lookup
- `tests/type_promotion_test.rs` - New test file with 8 tests

---

## ✅ DONE: First-Class Function Types Implemented (Dec 26, 2024)

### What Was Implemented

Added `Type::Function(Box<Type>, Box<Type>)` variant to the type system:

```rust
pub enum Type {
    // ...
    /// Function type: A → B
    Function(Box<Type>, Box<Type>),
    // ...
}
```

### Files Modified
- `src/type_inference.rs` - Added Function variant, updated unify(), occurs(), apply()
- `src/type_context.rs` - Updated type_expr_to_type() and interpret_toplevel_operation_type()
- `tests/function_type_test.rs` - New test file with 9 tests

### What Works Now
- **Display:** `sin : Scalar → Scalar` displays correctly with arrow
- **Unification:** Function types unify properly (same domains/codomains)
- **Occurs check:** Prevents infinite types like `α = α → ℝ`
- **Higher-order functions:** Can represent `(T → U) → List(T) → List(U)`
- **Curried functions:** Can represent `ℝ → ℝ → ℝ`

### Still TODO: Product Types

Product types still need proper support (lines ~1175 in type_context.rs):

```rust
TypeExpr::Product(types) => {
    // Product type - for now return first type
    // TODO: Proper tuple/product type support
    self.type_expr_to_type(&types[0])
}
```

Returns first element only instead of proper tuple type.

---

## 🔴 Tech Debt: N-ary Product Types Not Supported

### Problem
The parser only supports **binary product types** (`A × B`), not n-ary products.

This fails to parse:
```kleis
operation mass_at : GreenKernel × Flow × Event → ℝ  // ❌ Parse error
```

### Root Cause
The type parser in `kleis_parser.rs` parses product types as binary operations:
```
type → base_type ('×' base_type)?
```

When it sees `A × B × C × D`, the grammar only handles one `×` without proper
right-associativity or n-ary grouping.

### Options

**Option 1: Make `×` right-associative**
- `A × B × C` parses as `A × (B × C)`
- Minimal parser change
- Semantically correct (nested pairs)

**Option 2: Add n-ary tuple syntax**
- `(A, B, C, D)` as a first-class n-tuple type
- More parser work but cleaner syntax
- Matches common mathematical notation

**Option 3: Keep binary, use structures (current workaround)**
- Bundle multi-arg data into structures (`ResiduePair`, `SourceSpec`)
- Verbose but works now
- Used in POT formalization

### Workarounds (what we did for POT)
1. Bundle data into structures: `ResiduePair { field: FieldR4, event: Event }`
2. Use `make_pair` operations: `operation make_pair : FieldR4 × Event → ResiduePair`
3. Break multi-arg operations into smaller pieces

### Files to Modify
- `src/kleis_parser.rs` - `parse_type()` function

### Impact
- Equation Editor type display will be correct
- REPL `:type` command will show correct types
- Better user experience when mixing numeric types

---

## ✅ DONE: assert() Uses Z3 Verification (Dec 26, 2024)

**Implemented!** `assert()` in example blocks now uses Z3 for symbolic verification.

### Changes Made
- Added `is_symbolic()` to detect if expressions contain unbound variables
- Added `verify_with_z3()` to call `AxiomVerifier.verify_axiom()`
- Modified `eval_equality_assert()` to try Z3 when expressions are symbolic
- Added `AssertResult::Verified` and `AssertResult::Disproved` variants

### Tests Added (`tests/crossfile_debug_test.rs`)
- `test_assert_with_z3_verification` - Verifies commutativity axiom
- `test_assert_associativity` - Verifies associativity axiom
- `test_assert_invalid_symbolic` - Z3 correctly disproves `x + y = y + y`
- `test_assert_concrete_values` - Structural equality for bound variables

### How It Works
```kleis
assert(x + y = y + x)   // ✅ Z3 verifies via commutativity axiom
assert(x + y = y + y)   // ❌ Z3 disproves: "Counterexample: y!1 -> 1, x!0 -> 0"
assert(4 = 4)           // ✅ Concrete equality (no Z3 needed)
```

---

## ✅ DONE: Thread-Safe AST Cache (ADR-025)

**See:** `docs/adr/adr-025-debugger-shared-context.md`

Implemented thread-safe AST cache shared between LSP and DAP:

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

**Key features:**
- LSP updates cache when documents change
- DAP reads from cache (or parses and caches if missing/dirty)
- Cascade invalidation: dirty files propagate to dependents
- Each thread has its own `Evaluator` (because `RefCell` is not `Sync`)

---

## ✅ DONE: DAP Line Number Issues FIXED! (Dec 26, 2024)

### What Was Fixed

1. **Parser span capture at START of operations** - Fixed 8 parsing functions
2. **Skip expressions without spans** - No more line 1 spurious stops
3. **Custom operator spans** - Fixed parse_where_term

### Current State (ALL WORKING!)

| Component | Status |
|-----------|--------|
| Parser populates `SourceSpan` with file path | ✅ |
| `ExampleStatement` carries location | ✅ |
| Evaluator calls `on_eval_start()` for every expression | ✅ |
| `DapDebugHook` exists with channel-based communication | ✅ |
| DAP returns stack traces with file paths | ✅ |
| VS Code shows debugger UI | ✅ |
| DAP wires hook to evaluator | ✅ |
| Cross-file debugging (file switching) | ✅ |
| **Line numbers accurate in cross-file stepping** | ✅ FIXED! |

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

---

## 🧠 CRITICAL ARCHITECTURE: SharedContext AST Cache

### The Insight

**LSP already parses every file the user has open.** It re-parses on every edit.
DAP should NOT parse files separately — it should use the SAME cached AST.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              SharedContext.documents                         │
│                                                              │
│   HashMap<PathBuf, CachedDocument>                          │
│                                                              │
│   "/path/to/main.kleis"    → AST (parsed by LSP on open)    │
│   "/path/to/helper.kleis"  → AST (parsed by LSP on open)    │
│   "/path/to/stdlib/prelude" → AST (parsed by DAP if needed) │
│                                                              │
└─────────────────────────────────────────────────────────────┘
        ↑                              ↑
   LSP updates on edit             DAP reads (parses only if missing)
```

### The Rule

1. **DAP checks cache first** before parsing any file
2. **If found** → use it (FREE, already parsed by LSP)
3. **If not found** → parse, then ADD to cache for future use
4. **Both LSP and DAP use the same cache**

### Cache Invalidation (CRITICAL)

**When a file changes, all files that IMPORT it must be evicted from cache.**

Example:
```
main.kleis imports helper.kleis
helper.kleis imports stdlib/prelude.kleis

If stdlib/prelude.kleis changes:
  → Evict helper.kleis (imports stdlib)
  → Evict main.kleis (imports helper which imports stdlib)
```

This requires **dependency tracking**:
```rust
struct CachedDocument {
    ast: Program,
    imports: Vec<PathBuf>,        // Files this doc imports
    imported_by: Vec<PathBuf>,    // Files that import this doc (reverse)
}
```

When file X changes:
1. Evict X from cache
2. For each file that imports X, recursively evict

### Performance Impact

| Without Cache | With Cache |
|---------------|------------|
| Debug start: parse file (50ms) | 0ms (already parsed) |
| Step into import: parse (50ms) | 0ms if open in editor |
| Edit during debug: parse twice | Parse once (LSP only) |

### Why This Matters

> **The user's editor IS the source of truth.**
> LSP sees what user sees. DAP uses what LSP sees.
> No stale ASTs. No duplicate parsing.

### The Algorithm (Classic Incremental Compilation)

This is the same algorithm used by `make`, `cargo`, Webpack, and TypeScript.

**1. Build Dependency Graph (on parse):**
```rust
fn on_parse(file: &Path, ast: &Program) {
    for import_path in ast.imports() {
        // Forward edge: file imports import_path
        cache[file].imports.push(import_path);
        // Reverse edge: import_path is imported_by file
        cache[import_path].imported_by.push(file);
    }
}
```

**2. Invalidation (on file change) — propagate UP the tree:**
```rust
fn invalidate(file: &Path) {
    if let Some(doc) = cache.remove(file) {
        // Recursively invalidate all dependents
        for dependent in doc.imported_by {
            invalidate(&dependent);
        }
    }
}
```

**3. Lazy Re-parse (on demand) — parse dependencies FIRST:**
```rust
fn get_ast(file: &Path) -> &Program {
    if cache.contains(file) {
        return &cache[file].ast;
    }
    
    // Parse the file
    let ast = parse(file);
    
    // Ensure all imports are in cache first (topological order)
    for import_path in ast.imports() {
        get_ast(&import_path);  // Recursive
    }
    
    // Store and return
    cache.insert(file, CachedDocument { ast, ... });
    &cache[file].ast
}
```

**Visual Example:**
```
stdlib/prelude.kleis CHANGES
         ↓ invalidate
    helper.kleis (imports stdlib) → EVICTED
         ↓ invalidate  
    main.kleis (imports helper) → EVICTED

Later, when DAP needs main.kleis:
    get_ast(main.kleis)
        → get_ast(helper.kleis)  // dependency first
            → get_ast(stdlib/prelude.kleis)  // leaf first
            ← parse stdlib, cache it
        ← parse helper, cache it
    ← parse main, cache it
```

**Key Properties:**
- Parse each file at most once per change
- Dependencies parsed before dependents (topological order)
- Lazy: only re-parse when actually needed
- Minimal work: only affected files re-parsed

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

## ✅ Current Debugger Status (ALL WORKING!)

| Feature | Status |
|---------|--------|
| Launch/attach | ✅ |
| Breakpoints (set) | ✅ |
| Breakpoints (hit) | ✅ Real, wired to evaluator |
| Breakpoints in imported files | ✅ Works! |
| Step in/over/out | ✅ Real evaluation |
| Continue | ✅ Real evaluation |
| Stack trace | ✅ Correct file paths |
| Variables | ✅ Shows AST expressions |
| Cross-file (file switching) | ✅ Works |
| Cross-file (line numbers) | ✅ **FIXED!** All operations correct |
| assert() with Z3 | ✅ **NEW!** Symbolic verification |

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
| Tests | 755+ passing |
| Commits | 850+ |
| ADRs | 25 |
| Grammar | v0.93 |
| Unique Cloners | 505+ |
| Bourbaki Coverage | ~15-20% (axiomatic) |
| DAP Debugger | ✅ Fully working! |
| Z3 Assert Verification | ✅ Implemented! |

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
