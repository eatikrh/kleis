# Kleis Cross-Compiler Experiment

This directory contains an experimental cross-compiler from Kleis to Rust,
**written entirely in Kleis itself**.

## Directory Structure

```
experiments/cross-compiler/
├── README.md                     # This documentation
├── kleis/                        # Kleis source files
│   ├── kleis_in_kleis.kleis      # Core AST types (Expression, Pattern, Type, etc.)
│   ├── kleis_codegen_rust.kleis  # Kleis → Rust translation (89 functions)
│   ├── kleis_grammar_tests.kleis # Z3-verifiable grammar properties
│   ├── module_protocol.kleis     # Kleis ABI specification (48 functions)
│   ├── solver_api.kleis          # Solver abstraction + host utilities
│   ├── compiler_module.kleis     # Compiler as a Kleis Module
│   ├── symbolic.kleis            # Symbolic computation layer
│   ├── fibonacci.kleis           # Example: Fibonacci as AST
│   ├── compile_fib.kleis         # Compile Fibonacci to Rust
│   └── more_examples.kleis       # More Kleis constructs
│
└── rust/                         # Generated Rust code
    ├── Cargo.toml                # Rust project manifest
    ├── src/
    │   ├── lib.rs                # Runtime library entry point
    │   ├── symbolic.rs           # Sym<T> - symbolic values
    │   ├── solver.rs             # Z3 integration
    │   ├── module_abi.rs         # Kleis ABI implementation
    │   ├── fib_module.rs         # Example: Fibonacci as Kleis Module
    │   └── bin/
    │       ├── fib_symbolic.rs   # Symbolic Fibonacci demo
    │       └── loader_demo.rs    # Dynamic module loading demo
    ├── fib.rs                    # Generated fibonacci function
    ├── examples.rs               # Generated examples
    └── solver_api.rs             # Generated solver API
```

## Key Concepts

### 1. Self-Describing Grammar (`kleis_in_kleis.kleis`)

Kleis defines its own AST types:

```kleis
data Expression =
    EVariable(name: String)
  | ENumber(value: ℝ)
  | EOperation(name: String, args: List(Expression))
  | ELambda(params: List(Param), body: Expression)
  | EIf(cond: Expression, then_expr: Expression, else_expr: Expression)
  | ...
```

### 2. Code Generation (`kleis_codegen_rust.kleis`)

Functions that translate Kleis AST to Rust strings:

```kleis
define type_to_rust(t : Type) : String =
    match t {
        TPrimitive("ℝ") => "f64"
      | TPrimitive("ℕ") => "u64"
      | TParametric("List", args) => concat("Vec<", type_to_rust(head(args)), ">")
      | ...
    }

define expr_to_rust(e : Expression) : String =
    match e {
        EVariable(name) => name
      | EOperation("plus", args) => binary_op(args, "+")
      | EIf(cond, then_e, else_e) => concat("if ", expr_to_rust(cond), " { ... }")
      | ...
    }
```

### 3. Symbolic Computation (`symbolic.kleis` → `src/symbolic.rs`)

Variables are **symbolic**, not concrete:

```rust
use kleis_solver_api::{Sym, SymReal};

let x: SymReal = Sym::var("x");        // Symbolic variable
let y: SymReal = Sym::concrete(3.0);   // Concrete value
let expr = x + y;                       // → "(x + 3)" (symbolic expression)
```

### 4. Z3 Solver Integration (`solver_api.kleis` → `src/solver.rs`)

```rust
use kleis_solver_api::{KleisSolver, Sym};

let solver = KleisSolver::new();
let x: SymReal = Sym::var("x");

// Solve: x² = 9
let constraint = (x.clone() * x.clone()).sym_eq(Sym::concrete(9.0));
let solution = solver.solve(&constraint);  // → Some({"x": 3.0})
```

### 5. Kleis ABI (`module_protocol.kleis` → `src/module_abi.rs`)

**A Kleis Module** is a compiled shared library (`.dylib`/`.so`/`.dll`) that 
implements the **Kleis ABI** - a binary interface for discovery and invocation:

```
┌─────────────┐     dlopen/LoadLibrary     ┌──────────────────┐
│   REPL      │ ──────────────────────────▶│  my_module.dylib │
│             │                             │                  │
│  "load X"   │◀──── kleis_manifest() ─────│  [exported fns]  │
│             │                             │                  │
│  "fib(5)"   │──── kleis_call("fib",[5])──▶│                  │
│             │◀──── 5 ────────────────────│                  │
└─────────────┘                             └──────────────────┘
```

**No central registry** - each module is self-describing. The REPL loads
modules directly via `dlopen`/`LoadLibrary`.

**Exported C functions:**
```c
// Discover available functions/axioms
const AbiManifest* kleis_manifest();

// Call a function by name
AbiResult kleis_call(const char* name, const AbiValue* args, size_t len);

// Get an axiom formula by name
const char* kleis_get_axiom(const char* name);

// List all function names (null-terminated array)
const char** kleis_function_names();
```

**Generated from Kleis:**
```kleis
// kleis_codegen_rust.kleis generates the ABI exports
define module_to_rust(name : String, decls : List(Declaration)) : String =
    concat(join_decls(decls),
           generate_abi_exports(name, decls))
```

### 6. Higher-Order Functions (HOFs)

Kleis fully supports passing functions as values across module boundaries:

```kleis
data FunctionRef = FunctionRef(
    module_name: String,       // Which module owns the function
    function_name: String,     // Function identifier
    arity: ℕ                  // Number of arguments
)

data PartialApp = PartialApp(
    func_ref: FunctionRef,     // Reference to the function
    bound_args: List(Value)    // Already-bound arguments
)
```

**Example:**
```kleis
// Higher-order: map takes a function
define map(f : α → β, xs : List(α)) : List(β) = ...

// Partial application: add(5) returns a function
define add(x : ℕ) : ℕ → ℕ = λy . x + y
let add5 = add(5)   // PartialApp(FunctionRef("math", "add", 2), [5])
```

### 7. Module Versioning

Every module includes version metadata for dependency tracking:

```kleis
data ModuleManifest = ModuleManifest(
    name: String,
    version: String,
    source_hash: String,       // SHA256 of .kleis source file
    signature_hash: String,    // Hash of exported signatures (API contract)
    compile_time: ℕ,          // Unix timestamp when compiled
    imports: List(ImportDescriptor)
)

data ImportDescriptor = ImportDescriptor(
    path: String,              // Module path
    expected_hash: String,     // Expected signature_hash
    required: Bool
)
```

**Recompilation triggers:**
- `source_hash` changed → source code modified, recompile
- `signature_hash` mismatch → API incompatible, recompile dependents
- Always stale if dependency newer than dependent

### 8. Solver as Host Gateway

The `Solver` abstraction provides host system utilities:

```kleis
structure Solver(Expr) {
    // Core SMT operations
    operation check_sat : Constraint → SatResult
    operation get_model : Unit → Option(Model)
    
    // Host system utilities (used by codegen)
    operation sha256 : String → String           // For source_hash
    operation current_time : Unit → ℕ           // For compile_time
    operation read_file : String → String        // Load source
    operation write_file : String → String → Unit
    operation real_to_string : ℝ → String
    operation nat_to_string : ℕ → String
}
```

This keeps Kleis pure while allowing the codegen to access host functionality.

### 9. Pass-by-Value/Address (Same-Platform ABI)

**Assumption:** All modules run on the same platform, compiled by the same
Rust compiler. No serialization or endianness concerns.

| Type | Passing | Reason |
|------|---------|--------|
| `ℕ`, `ℤ`, `ℝ`, `Bool` | By value | Small, fixed-size primitives |
| `String` | By reference (`*const c_char`) | Variable size |
| `List(T)` | By reference (`*const AbiValue`) | Variable size |
| ADT variants | By reference (`*const AbiValue`) | Discriminated union |
| `FunctionRef` | By value | Small, fixed-size struct |
| `PartialApp` | By reference | Contains variable-size bound_args |

**Rust representation:**
```rust
#[repr(C)]
pub enum AbiValue {
    Unit,
    Bool(bool),
    Nat(u64),
    Int(i64),
    Real(f64),
    String(*const c_char),
    List { ptr: *const AbiValue, len: usize },
    Variant { tag: u32, fields: *const AbiValue, field_count: usize },
    Function(FunctionRef),
    PartialApp { func: FunctionRef, args: *const AbiValue, arg_count: usize },
}

## Building

```bash
cd experiments/cross-compiler/rust
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h  # macOS ARM
cargo build
```

## Running Examples

```bash
# Concrete examples (no Z3 needed)
cargo run --bin fib
cargo run --bin more_examples

# Symbolic examples (requires Z3)
cargo run --bin solver_api
cargo run --bin fib_symbolic
```

## Two Modes of Kleis Modules

Kleis Modules serve two fundamentally different purposes:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MODE 1: SYMBOLIC (Container Mode)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Purpose: Deliver Kleis code for symbolic reasoning                          │
│                                                                             │
│  user.kleis  →  user_module.dylib  →  REPL loads  →  Extracts AST/axioms   │
│                                              ↓                              │
│                                       Module job done!                      │
│                                              ↓                              │
│                                   Kleis interpreter + Z3 verify             │
│                                                                             │
│ • Module is a "container" - delivers payload once                           │
│ • No execution via kleis_call() - AST is interpreted                        │
│ • Performance of module irrelevant                                          │
│ • Z3 verifies axioms symbolically                                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                    MODE 2: NUMERICAL (Execution Mode)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ Purpose: Provide high-performance numerical computation                     │
│                                                                             │
│  lapack.rs  →  lapack_module.dylib  →  REPL loads  →  Calls via ABI        │
│                                              ↓                              │
│                               kleis_call("solve", [A, b])                   │
│                                              ↓                              │
│                                   Native BLAS/LAPACK execution              │
│                                                                             │
│ • Module actually executes code via kleis_call()                            │
│ • Performance critical - compiled for speed                                 │
│ • No symbolic verification - just computation                               │
│ • Perfect for numerical libraries, FFI bindings                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Why both modes use the same ABI:**
- Uniform discovery mechanism (`kleis_manifest()`)
- Same versioning and dependency tracking
- REPL doesn't need to know which mode a module uses
- Numerical modules can still export axioms about their behavior

**Example - Best of Both Worlds:**
```kleis
import "lapack_bindings"  -- Mode 2: Native numerical library

-- These call directly into compiled LAPACK via ABI (fast!)
let (eigenvalues, eigenvectors) = lapack.eig(matrix)

-- But THIS axiom is verified symbolically by Z3
axiom eigenvalue_property : ∀ A v λ . 
    is_eigenpair(A, v, λ) ⟹ A × v = λ × v
```

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KLEIS CROSS-COMPILER                                │
└─────────────────────────────────────────────────────────────────────────────┘

KLEIS SOURCE FILES (kleis/):
┌─────────────────────────┬───────────────────────────────────────────────────┐
│ kleis_in_kleis.kleis    │ AST types (Expression, Pattern, Type, etc.)      │
│ kleis_codegen_rust.kleis│ Kleis → Rust translation (89 functions)          │
│ module_protocol.kleis   │ Kleis ABI specification (48 functions)           │
│ solver_api.kleis        │ Solver + host system utilities                   │
│ compiler_module.kleis   │ Compiler as a Kleis Module                       │
└─────────────────────────┴───────────────────────────────────────────────────┘

RUST RUNTIME (rust/):
┌─────────────────────────┬───────────────────────────────────────────────────┐
│ src/module_abi.rs       │ Kleis ABI implementation                         │
│ src/symbolic.rs         │ Sym<T> for symbolic execution                    │
│ src/solver.rs           │ Z3 backend                                       │
│ src/fib_module.rs       │ Example generated module                         │
└─────────────────────────┴───────────────────────────────────────────────────┘

KEY FEATURES:
┌─────────────────────────────────────────────────────────────────────────────┐
│ ✓ Kleis ABI         - C-compatible discovery & invocation                  │
│ ✓ Kleis Modules     - Self-describing .dylib/.so/.dll                      │
│ ✓ HOF Support       - FunctionRef + partial application                    │
│ ✓ Versioning        - source_hash, signature_hash, compile_time            │
│ ✓ Solver Gateway    - sha256, current_time, file I/O via solver            │
│ ✓ No Central Registry - Modules loaded directly via dlopen                 │
│ ✓ Same-Platform     - No serialization overhead                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

## The Vision

1. **Write Kleis** → Kleis source code
2. **Parse to AST** → `kleis_in_kleis.kleis` types
3. **Translate to Rust** → `kleis_codegen_rust.kleis` (written in Kleis!)
4. **Compile with rustc** → Native shared library (Kleis Module)
5. **Load dynamically** → REPL uses `dlopen` + Kleis ABI
6. **Verify with Z3** → Axioms extracted and checked

This enables:
- **Verified cross-compilation**: Kleis proves properties about generated code
- **Self-hosting**: The cross-compiler can compile itself
- **Performance**: Generated Rust runs at native speed
- **Dynamic composition**: Load/unload modules at runtime
- **Dependency management**: Version hashes ensure compatibility

