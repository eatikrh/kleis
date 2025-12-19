# ADR-023: Kleis ABI and Kleis Modules

## Status

Accepted

## Context

The Kleis cross-compiler (written in Kleis itself) generates Rust code that compiles
to native shared libraries. We need a standardized way for:

1. A REPL to dynamically load compiled Kleis modules
2. Discovery of available functions and axioms without recompilation
3. Calling functions across module boundaries
4. Passing higher-order functions between modules
5. Managing module versions and dependencies

### Alternatives Considered

1. **Static Linking**: Compile everything together
   - Rejected: No dynamic loading, no REPL flexibility
   
2. **FFI with Manual Registration**: Modules call a registration function
   - Rejected: Requires central registry, complex initialization order
   
3. **Plugin System with Traits**: Define Rust traits that modules implement
   - Rejected: Rust's lack of stable ABI makes this fragile

4. **C-Compatible ABI (COM-like)**: Self-describing modules with C exports
   - Chosen: Maximum compatibility, no central registry needed

## Decision

We define the **Kleis ABI** (Application Binary Interface) and **Kleis Module** concepts.

### Kleis Module

A **Kleis Module** is a compiled shared library (`.dylib`/`.so`/`.dll`) that:
- Exports C-compatible functions for discovery and invocation
- Contains its own manifest (name, version, exports, axioms, imports)
- Is self-describing—no external metadata files required

### Kleis ABI

The **Kleis ABI** is the binary-level contract that Kleis Modules implement:

```c
// Module identification
extern "C" const char* kleis_module_name();
extern "C" const char* kleis_module_version();

// Discovery
extern "C" const ModuleManifest* kleis_manifest();
extern "C" const char** kleis_function_names();  // null-terminated
extern "C" const char** kleis_axiom_names();     // null-terminated

// Invocation
extern "C" AbiResult kleis_call(const char* name, const AbiValue* args, size_t len);
extern "C" const char* kleis_get_axiom(const char* name);  // Z3 SMT-LIB string
```

### AbiValue - Universal Value Type

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
```

### Higher-Order Functions

Functions can be passed across module boundaries using `FunctionRef`:

```rust
#[repr(C)]
pub struct FunctionRef {
    module_name: *const c_char,
    function_name: *const c_char,
    arity: u32,
}
```

The REPL resolves `FunctionRef` to the actual function at call time, enabling
cross-module HOFs like `map(f, xs)` where `f` comes from a different module.

### Versioning

Each module manifest includes version tracking:

```rust
pub struct ModuleManifest {
    name: String,
    version: String,
    source_hash: String,      // SHA256 of .kleis source
    signature_hash: String,   // Hash of exported signatures (API contract)
    compile_time: u64,        // Unix timestamp
    imports: Vec<ImportDescriptor>,
}

pub struct ImportDescriptor {
    path: String,
    expected_hash: String,    // Expected signature_hash
    required: bool,
}
```

Recompilation is triggered when:
- `source_hash` differs from cached value (source changed)
- `signature_hash` of dependency doesn't match `expected_hash` (API changed)

### Same-Platform Assumption

The Kleis ABI assumes all modules run on the same platform, compiled by the
same Rust compiler version. This simplifies:
- No endianness handling
- No serialization of primitives
- Direct pointer passing for strings and lists
- No cross-platform numeric format concerns

## Two Modes of Operation

Kleis Modules serve two fundamentally different purposes:

### Mode 1: Symbolic (Container Mode)

For Kleis code that will be reasoned about symbolically:

```
user.kleis → user_module.dylib → REPL loads → Extracts AST/axioms → Done
                                                      ↓
                                        Kleis interpreter + Z3 verify
```

- Module is a **container** - delivers payload once, then unused
- `kleis_call()` not used - REPL interprets the AST directly
- Performance of module code is **irrelevant**
- Z3 verifies axioms symbolically

### Mode 2: Numerical (Execution Mode)

For high-performance numerical libraries:

```
lapack.rs → lapack_module.dylib → REPL loads → kleis_call("solve", [A,b])
                                                      ↓
                                           Native BLAS/LAPACK execution
```

- Module **executes** code via `kleis_call()`
- Performance is **critical** - compiled for speed
- No symbolic verification - just computation
- Perfect for: BLAS, LAPACK, numerical solvers, FFI bindings

### Why One ABI for Both

- Uniform discovery (`kleis_manifest()`)
- Same versioning and dependency tracking
- REPL doesn't need to know which mode
- Numerical modules can still export axioms about their behavior

## Consequences

### Positive

- **No Central Registry**: Each module is self-describing
- **Dynamic Loading**: REPL can load/unload modules at runtime
- **Cross-Module HOFs**: Functions are first-class across module boundaries
- **Version Safety**: Hash-based dependency tracking prevents stale binaries
- **C Compatibility**: Can be called from any language with C FFI
- **Dual Purpose**: Same ABI works for symbolic containers AND numerical execution

### Negative

- **Same Platform Only**: Not suitable for distributed systems
- **Rust Compiler Coupling**: ABI may break across Rust versions
- **Memory Management**: Caller must understand lifetime of returned pointers
- **Mode Confusion**: Must document which modules are containers vs executors

### Implementation Location

- Kleis definition: `experiments/cross-compiler/kleis/module_protocol.kleis`
- Rust implementation: `experiments/cross-compiler/rust/src/module_abi.rs`
- Example module: `experiments/cross-compiler/rust/src/fib_module.rs`
- Loading demo: `experiments/cross-compiler/rust/src/bin/loader_demo.rs`

## Related ADRs

- ADR-003: Self-Hosting Strategy (cross-compiler in Kleis)
- ADR-021: Algebraic Data Types (ADT representation in ABI)
- ADR-022: Z3 Integration (axiom verification via `kleis_get_axiom`)

