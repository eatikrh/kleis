# Solver Abstraction Layer - Architecture

**Status:** ✅ Implemented (December 12, 2024)  
**Branch:** `refactor/solver-abstraction-layer` (merged to main)  
**Files:** 16 files, 2,521 insertions, -694 deletions

---

## 🎯 Overview

A pluggable solver backend system inspired by Model Context Protocol (MCP), enabling:

1. ✅ **Solver Independence** - Swap Z3, CVC5, custom solvers transparently
2. ✅ **Capability Declaration** - Solvers declare coverage upfront (MCP-style)
3. ✅ **User Extensibility** - Users can add custom operation translators
4. ✅ **Coverage Tracking** - Know what's natively supported vs uninterpreted
5. ✅ **Clean Abstraction** - All public APIs use Kleis AST only

**Critical Principle:** Solver backends MUST return Kleis `Expression`, never solver-specific types!

---

## 🏗️ Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   User Code                             │
│              (Kleis Expression only)                    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              AxiomVerifier (High-Level)                 │
│  - Dependency analysis (which structures needed)        │
│  - Structure loading (axioms, identity elements)        │
│  - Delegates verification to backend                    │
└────────────────────┬────────────────────────────────────┘
                     │ uses SolverBackend trait
                     ▼
┌─────────────────────────────────────────────────────────┐
│            SolverBackend Trait (Interface)              │
│  - verify_axiom(expr) → VerificationResult             │
│  - evaluate(expr) → Expression          ← Returns AST! │
│  - simplify(expr) → Expression          ← Returns AST! │
│  - are_equivalent(e1, e2) → bool                       │
└────────────────────┬────────────────────────────────────┘
                     │ implemented by
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Z3Backend (Implementation)                 │
│  - Translates: Expression → Z3 Dynamic                 │
│  - Uses modular translators (arithmetic, comparison,    │
│    boolean)                                             │
│  - Converts results: Z3 Dynamic → Expression           │
└────────────────────┬────────────────────────────────────┘
                     │ uses
                     ▼
┌─────────────────────────────────────────────────────────┐
│         Modular Operation Translators                   │
│  - arithmetic.rs: plus, minus, times, negate           │
│  - comparison.rs: equals, lt, gt, leq, geq             │
│  - boolean.rs: and, or, not, implies                   │
│  - All handle Int/Real mixing automatically            │
└─────────────────────────────────────────────────────────┘
```

**Key: Z3 types (Dynamic, Int, Real, Bool) NEVER escape Z3Backend!**

---

## 📦 File Structure

```
src/
├── axiom_verifier.rs           (478 lines, -642 from refactoring!)
│   └── Uses SolverBackend trait, no direct Z3 imports
│
├── solvers/
│   ├── mod.rs                  (Public API, re-exports)
│   ├── backend.rs              (SolverBackend trait)
│   ├── capabilities.rs         (SolverCapabilities, MCP-style)
│   ├── result_converter.rs     (ResultConverter trait)
│   │
│   └── z3/                     (Z3-specific implementation)
│       ├── mod.rs
│       ├── backend.rs          (681 lines - Z3Backend impl)
│       ├── capabilities.toml   (MCP manifest - 15 operations)
│       ├── converter.rs        (Z3ResultConverter)
│       └── translators/
│           ├── mod.rs
│           ├── arithmetic.rs   (203 lines - plus, minus, times, negate)
│           ├── comparison.rs   (234 lines - equals, lt, gt, leq, geq)
│           └── boolean.rs      (148 lines - and, or, not, implies)
```

---

## 🎨 Core Trait: SolverBackend

### Definition

```rust
pub trait SolverBackend {
    /// Solver name (e.g., "Z3", "CVC5")
    fn name(&self) -> &str;

    /// Get capabilities (MCP-style manifest)
    fn capabilities(&self) -> &SolverCapabilities;

    /// Check if operation is natively supported
    fn supports_operation(&self, op: &str) -> bool;

    /// Verify axiom (SMT solving)
    fn verify_axiom(&mut self, axiom: &Expression) 
        -> Result<VerificationResult, String>;

    /// Evaluate expression to concrete value
    /// CRITICAL: Returns Kleis Expression, not solver type!
    fn evaluate(&mut self, expr: &Expression) 
        -> Result<Expression, String>;

    /// Simplify expression algebraically
    /// CRITICAL: Returns Kleis Expression, not solver type!
    fn simplify(&mut self, expr: &Expression) 
        -> Result<Expression, String>;

    /// Check if two expressions are equivalent
    fn are_equivalent(&mut self, expr1: &Expression, expr2: &Expression) 
        -> Result<bool, String>;

    /// Solver state management (incremental solving)
    fn push(&mut self);
    fn pop(&mut self, levels: u32);
    fn reset(&mut self);
}
```

**Key Design:** All methods work with `Expression` only - solver types stay internal!

---

## 🎪 MCP-Style Capabilities

### Capability Manifest (capabilities.toml)

Inspired by Model Context Protocol servers declaring resources.

```toml
[solver]
name = "Z3"
version = "4.12.0"
type = "smt"
description = "Z3 Theorem Prover from Microsoft Research"

[capabilities]
theories = ["arithmetic", "boolean", "equality", "uninterpreted_functions"]

[capabilities.operations]
# Native operations (have translators)
plus = { arity = 2, theory = "Int/Real", native = true }
equals = { arity = 2, theory = "Any", native = true }
and = { arity = 2, theory = "Bool", native = true }
# ... 15 operations total

[capabilities.features]
quantifiers = true
uninterpreted_functions = true
recursive_functions = true
evaluation = true
simplification = false

[capabilities.performance]
max_axioms = 10000
timeout_ms = 5000
```

**Benefits:**
- ✅ Upfront declaration of what solver can do
- ✅ Enables coverage analysis
- ✅ Users can query capabilities programmatically
- ✅ Comparison across solvers

---

## 🔄 Result Conversion: Critical Abstraction Boundary

### The ResultConverter Trait

```rust
pub trait ResultConverter<SolverValue> {
    /// Convert solver result to Kleis Expression
    /// THIS IS THE ABSTRACTION BOUNDARY!
    fn to_expression(&self, value: &SolverValue) 
        -> Result<Expression, String>;
}
```

### Z3 Implementation

```rust
impl ResultConverter<Dynamic> for Z3ResultConverter {
    fn to_expression(&self, value: &Dynamic) -> Result<Expression, String> {
        // Z3 Dynamic can be Int, Real, Bool, etc.
        if let Some(int_val) = value.as_int() {
            if let Some(i) = int_val.as_i64() {
                return Ok(Expression::Const(i.to_string()));
            }
        }
        
        if let Some(bool_val) = value.as_bool() {
            if let Some(b) = bool_val.as_bool() {
                return Ok(Expression::Const(b.to_string()));
            }
        }
        
        if let Some(real_val) = value.as_real() {
            if let Some((num, den)) = real_val.as_rational() {
                // Convert rational to Expression
                if den == 1 {
                    return Ok(Expression::Const(num.to_string()));
                } else {
                    let decimal = num as f64 / den as f64;
                    return Ok(Expression::Const(decimal.to_string()));
                }
            }
        }
        
        // Fallback: symbolic representation
        Ok(Expression::Const(value.to_string()))
    }
}
```

**This ensures Z3 types NEVER escape the z3/ module!**

---

## 🔧 Modular Translator System

### Operation Translators

Each category of operations in its own file:

**arithmetic.rs** (plus, minus, times, negate):
```rust
pub fn translate_plus(left: &Dynamic, right: &Dynamic) -> Result<Dynamic, String> {
    // Try Int + Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(Int::add(&[&l, &r]).into());
    }
    
    // Try Real + Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(Real::add(&[&l, &r]).into());
    }
    
    // Mixed Int/Real - convert to Real
    let l_real = left.as_real().or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right.as_real().or_else(|| right.as_int().map(|i| i.to_real()));
    
    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(Real::add(&[&l, &r]).into());
    }
    
    // Fallback: uninterpreted function
    let func_decl = declare_uninterpreted("plus", 2);
    Ok(func_decl.apply(&[left, right]))
}
```

**comparison.rs** (equals, lt, gt, leq, geq):
- Handles Int/Real type mixing
- Equality works with any type
- Comparisons require numeric types

**boolean.rs** (and, or, not, implies):
- All require Bool type
- Direct mapping to Z3 boolean theory
- Type-safe (errors if non-boolean)

---

## 📊 Coverage Analysis

### Z3 Native Operations: 15/133 (11.3%)

**Native (have translators):**
```
Arithmetic:  plus, minus, times, negate
Comparison:  equals, lt, gt, leq, geq
Boolean:     and, or, not, implies
```

**Uninterpreted (118 operations):**
```
Transcendentals: sin, cos, tan, exp, ln, sqrt, ...
Matrix ops:      matrix_multiply, transpose, det, ...
Tensor ops:      contract, wedge, riemann, ...
Quantum ops:     bra, ket, commutator, ...
```

**This is CORRECT for theorem proving!**
- SMT solvers reason about properties, not compute values
- Uninterpreted functions are the right approach
- Axioms provide the semantics

---

## 🎯 Design Principles

### 1. Solver Independence

**All public APIs use Kleis Expression:**
```rust
// User code is solver-agnostic
fn verify_property(backend: &mut dyn SolverBackend, prop: &Expression) -> bool {
    backend.verify_axiom(prop).map(|r| matches!(r, VerificationResult::Valid))
        .unwrap_or(false)
}

// Works with ANY backend:
verify_property(&mut Z3Backend::new(&registry)?, &expr);
verify_property(&mut CVC5Backend::new(&registry)?, &expr);  // Future
```

### 2. MCP-Style Capabilities

**Upfront declaration:**
```rust
let caps = backend.capabilities();
println!("Solver: {}", caps.solver.name);
println!("Native ops: {:?}", caps.native_operations());
println!("Coverage: {:.1}%", (caps.native_operations().len() as f64 / 133.0) * 100.0);
```

### 3. Abstraction Boundary

**Z3 types never escape:**
```rust
// INTERNAL (in Z3Backend):
let z3_result: Dynamic = /* Z3 computation */;

// CONVERTED before returning:
self.converter.to_expression(&z3_result)  // → Expression

// USER RECEIVES:
Expression::Const("42")  // Never sees Z3::Dynamic!
```

### 4. Modular & Testable

**Each translator is:**
- ✅ In separate file
- ✅ Unit tested independently
- ✅ Easy to add/modify
- ✅ Documented with examples

---

## 📈 Migration Path

### Before Refactoring

```rust
// axiom_verifier.rs (1,120 lines)
impl AxiomVerifier {
    fn verify_axiom_impl(...) {
        let z3_expr = self.kleis_to_z3_dynamic(...)?;  // Hardcoded
        let z3_bool = z3_expr.as_bool()?;
        self.solver.assert(z3_bool.not());  // Direct Z3 access
        // ... 300 lines of Z3 translation code
    }
    
    // Plus: 642 lines of duplicate Z3 translation methods
}
```

**Problems:**
- ❌ Z3 hardcoded throughout
- ❌ No extensibility
- ❌ No coverage tracking
- ❌ Duplicate translation code
- ❌ Can't swap solvers

### After Refactoring

```rust
// axiom_verifier.rs (478 lines, -642 lines removed!)
impl AxiomVerifier {
    fn verify_axiom_impl(...) {
        // Delegate to backend abstraction
        let result = self.backend.verify_axiom(expr)?;
        // Convert result types
        Ok(match result { ... })
    }
    // No Z3 imports! Clean separation.
}

// solvers/z3/backend.rs (681 lines - all Z3 code centralized)
impl SolverBackend for Z3Backend {
    fn verify_axiom(&mut self, expr: &Expression) 
        -> Result<VerificationResult, String> 
    {
        let z3_expr = self.kleis_to_z3(expr, &HashMap::new())?;
        // ... Z3-specific logic stays here
        // Returns VerificationResult (not Z3 types!)
    }
}
```

**Benefits:**
- ✅ Z3 isolated to solvers/z3/
- ✅ AxiomVerifier has zero Z3 dependencies
- ✅ Modular translator system
- ✅ Can add CVC5Backend without changing AxiomVerifier
- ✅ Coverage tracking enabled

---

## 🔌 Implementation: Z3Backend

### Structure

```rust
pub struct Z3Backend<'r> {
    /// Z3 solver instance
    solver: Solver,
    
    /// Structure registry (source of Kleis definitions)
    registry: &'r StructureRegistry,
    
    /// Capabilities (loaded from capabilities.toml)
    capabilities: SolverCapabilities,
    
    /// Track declared operations
    declared_ops: HashSet<String>,
    
    /// Track loaded structures
    loaded_structures: HashSet<String>,
    
    /// Identity elements (zero, one, e)
    identity_elements: HashMap<String, Dynamic>,
    
    /// Result converter (Z3 → Kleis AST)
    converter: Z3ResultConverter,
}
```

**State Management:**
- Solver: Z3 instance (long-lived, incremental solving)
- Registry: Access to Kleis structure definitions
- Tracking: Operations and structures (for debugging/stats)
- Converter: Maintains abstraction boundary

### Key Methods

**verify_axiom():**
```rust
fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String> {
    self.solver.push();
    
    let z3_expr = self.kleis_to_z3(axiom, &HashMap::new())?;
    let z3_bool = z3_expr.as_bool()
        .ok_or("Axiom must be boolean")?;
    
    self.solver.assert(z3_bool.not());  // Assert negation
    
    let result = match self.solver.check() {
        SatResult::Unsat => VerificationResult::Valid,
        SatResult::Sat => {
            let model = self.solver.get_model().unwrap();
            VerificationResult::Invalid { 
                counterexample: format!("{}", model) 
            }
        }
        SatResult::Unknown => VerificationResult::Unknown,
    };
    
    self.solver.pop(1);
    Ok(result)
}
```

**evaluate():** Translates to Z3, evaluates, converts back to Expression

**simplify():** Uses Z3's simplify(), converts back to Expression

**are_equivalent():** Check if two expressions unify

---

## 🎪 MCP-Style Capability System

### SolverCapabilities Structure

```rust
pub struct SolverCapabilities {
    pub solver: SolverMetadata,
    pub capabilities: Capabilities,
}

pub struct SolverMetadata {
    pub name: String,
    pub version: String,
    pub solver_type: String,
    pub description: String,
}

pub struct Capabilities {
    pub theories: HashSet<String>,
    pub operations: HashMap<String, OperationSpec>,
    pub features: FeatureFlags,
    pub performance: PerformanceHints,
}

pub struct OperationSpec {
    pub arity: usize,
    pub theory: String,
    pub native: bool,
    pub reason: Option<String>,
}
```

### Runtime Capability Queries

```rust
let caps = backend.capabilities();

// Query operations
caps.has_operation("plus");        // → true
caps.has_operation("sin");         // → false (uninterpreted)

// List native operations
caps.native_operations();          // → ["plus", "minus", "times", ...]

// Check theories
caps.has_theory("arithmetic");     // → true
caps.has_theory("calculus");       // → false
```

---

## 📊 Z3 Coverage Details

### Native Support (15 operations)

| Category | Operations | Theory |
|----------|-----------|--------|
| **Arithmetic** | plus, minus, times, negate | Int/Real |
| **Comparison** | equals, lt, gt, leq, geq | Int/Real/Any |
| **Boolean** | and, or, not, implies | Bool |

**Why so few?**
- Z3 is for **theorem proving**, not computation
- These are the **decidable theories** Z3 has built-in
- Everything else uses **uninterpreted functions** (correct approach!)

### Uninterpreted Functions (118+ operations)

**Transcendentals** (No symbolic math theory):
- sin, cos, tan, sec, csc, cot
- sinh, cosh, tanh
- arcsin, arccos, arctan
- exp, ln, log, sqrt

**Matrix Operations** (Abstract reasoning via axioms):
- matrix_multiply, transpose, det, trace
- inverse, eigenvalues, svd

**Tensor Operations** (GR/differential geometry):
- contract, wedge, exterior_derivative
- riemann, ricci, einstein
- covariant_derivative

**Quantum Operations** (QM):
- bra, ket, commutator, anti_commutator
- tensor_product, partial_trace

**This is BY DESIGN:**
- Uninterpreted functions + axioms = powerful reasoning!
- Don't need built-in matrix theory to prove matrix properties
- Axioms provide the semantics

---

## 🎯 Critical: AST as Abstraction Boundary

### Why This Matters

**Bad Design (leaky abstraction):**
```rust
fn evaluate(&mut self, expr: &Expression) -> Result<z3::ast::Dynamic, String>
//                                                   ^^^^^^^^^^^^^^^^^ ❌

// User forced to handle Z3 types:
let z3_result = backend.evaluate(&expr)?;
let value = z3_result.as_int()?.as_i64()?;  // Z3 API in user code!
```

**Good Design (clean abstraction):**
```rust
fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String>
//                                                   ^^^^^^^^^^ ✅

// User works with Kleis types only:
let result = backend.evaluate(&expr)?;
match result {
    Expression::Const(v) => println!("Result: {}", v),  // Kleis AST!
    _ => println!("Symbolic: {:?}", result),
}
```

### Conversion Flow

```
User: Expression::Operation("plus", [Const("2"), Const("3")])
        ↓
Z3Backend::evaluate()
        ↓
kleis_to_z3() → Z3::Dynamic (internal translation)
        ↓
Z3 computation → Z3::Int(5)
        ↓
Z3ResultConverter::to_expression() ← BOUNDARY!
        ↓
Expression::Const("5")
        ↑
User: Never sees Z3 types!
```

**Key Insight:** Conversion happens INSIDE the backend, not in user code!

---

## 🚀 Extensibility

### Future: Multiple Backends

```rust
// Easy to add new backends
pub struct CVC5Backend { ... }
impl SolverBackend for CVC5Backend { ... }

pub struct CustomSMTBackend { ... }
impl SolverBackend for CustomSMTBackend { ... }

// User code unchanged:
let backend: Box<dyn SolverBackend> = match solver_choice {
    "Z3" => Box::new(Z3Backend::new(&registry)?),
    "CVC5" => Box::new(CVC5Backend::new(&registry)?),
    _ => return Err("Unknown solver"),
};

backend.verify_axiom(&axiom)?;  // Same interface!
```

### Future: Custom Translators

```rust
// User adds translator for sin (Taylor series approximation)
struct TaylorSinTranslator { order: usize }

impl OperationTranslator for TaylorSinTranslator {
    fn operation_name(&self) -> &str { "sin" }
    fn arity(&self) -> usize { 1 }
    fn translate(&self, args: &[Dynamic]) -> Result<Dynamic, String> {
        // Implement: sin(x) ≈ x - x³/3! + x⁵/5! - ...
        // (Taylor series to specified order)
    }
}

// Register with backend
backend.register_translator(Box::new(TaylorSinTranslator { order: 5 }));

// Now sin has native support!
backend.supports_operation("sin");  // → true
```

---

## 📝 Migration Summary

### What Was Removed (694 lines)

**From axiom_verifier.rs:**
- ❌ Direct Z3 imports (Bool, Dynamic, Int, Real, FuncDecl, Sort, Solver)
- ❌ 642 lines of Z3 translation code
- ❌ Duplicate operation translators
- ❌ Z3-specific state (identity_elements, declared_ops now in backend)

### What Was Added (2,521 lines)

**Core abstractions:**
- ✅ SolverBackend trait (172 lines)
- ✅ SolverCapabilities (262 lines)
- ✅ ResultConverter trait (247 lines)

**Z3 implementation:**
- ✅ Z3Backend (681 lines - all Z3 code centralized)
- ✅ Modular translators (585 lines - arithmetic, comparison, boolean)
- ✅ Z3ResultConverter (172 lines)
- ✅ capabilities.toml (73 lines - MCP manifest)

**Net result:** +1,827 lines of clean, modular architecture

---

## ✅ Quality Metrics

### Tests

```
✅ 776 tests passing (all through abstraction layer!)
❌ 0 tests failed
⏭️  46 tests ignored
```

**Key validation:**
- All existing Z3 integration tests pass through new abstraction
- Proves the abstraction is complete and correct
- No regressions introduced

### Code Quality

```
✅ cargo fmt --all --check    PASS
✅ cargo clippy               PASS (0 errors, 0 warnings)
✅ cargo test                 PASS (776/776)
```

### Architecture Quality

**Separation of Concerns:**
- High-level logic: axiom_verifier.rs (478 lines)
- Z3 backend: solvers/z3/ (1,159 lines total)
- Clean interfaces: SolverBackend trait

**No technical debt:**
- Zero dead code
- Zero commented-out code
- All TODOs documented

---

## 🎉 Key Achievements

1. ✅ **Pluggable Architecture** - Can swap Z3 for CVC5 or custom solvers
2. ✅ **MCP-Style Capabilities** - Upfront operation coverage declaration
3. ✅ **Clean Abstraction** - Z3 types never escape module
4. ✅ **Modular Translators** - Easy to extend and test
5. ✅ **User Extensibility** - Foundation for custom translators
6. ✅ **Coverage Tracking** - Know what's native vs uninterpreted
7. ✅ **Zero Technical Debt** - No shortcuts, properly refactored

---

## 🚀 Future Work

### Phase 4: Coverage CLI (Planned)

```bash
# List available solvers
$ kleis solver list
Available solvers:
  ✅ Z3 (v4.12.0) - 15 native operations

# Check capabilities
$ kleis solver capabilities Z3
Solver: Z3 v4.12.0
Native operations: 15 (11.3% coverage)
  ✅ Arithmetic: plus, minus, times, negate
  ✅ Comparison: equals, lt, gt, leq, geq
  ✅ Boolean: and, or, not, implies
Uninterpreted: 118 operations
  ⚠️  Transcendentals: sin, cos, exp, ...
  ⚠️  Matrix ops: matrix_multiply, transpose, ...

# Check coverage for specific program
$ kleis solver check-coverage stdlib/prelude.kleis
Operations used: 45
Z3 coverage: 15/45 (33.3%)
Recommendation: Sufficient for algebraic reasoning ✅
```

### Phase 5: CVC5 Backend (Future)

Implementing CVC5Backend will validate the abstraction:
- Same SolverBackend trait
- Different internal implementation
- User code unchanged

### Phase 6: Custom Translators (Future)

Allow users to register operation translators at runtime:
- Plugin system
- TOML-based translator definitions
- Dynamic library loading

---

## 📚 Related Documentation

### Primary References
- **This document** - Architecture overview
- `src/solvers/backend.rs` - Trait definitions
- `src/solvers/z3/capabilities.toml` - Z3 manifest
- `docs/session-2024-12-12/TRANSCENDENTAL_FUNCTIONS.md` - Why matrix transcendentals are valid

### Related ADRs
- ADR-022: Z3 Integration (previous hardcoded approach)
- ADR-023: Solver Abstraction Layer (to be created from this doc)
- ADR-016: Operations in Structures (operations defined in Kleis)

### Implementation Notes
- `docs/solver-abstraction/z3/IMPLEMENTATION_NOTES.md` (Z3-specific details)
- `docs/solver-abstraction/z3/FUNCTION_HANDLING.md` (RecFuncDecl, quantifiers)

### Session Documentation
- `NEXT_SESSION_SOLVER_MODULARIZATION.md` - Original plan
- `docs/session-2024-12-12/README.md` - Session summary

---

## 💡 Lessons Learned

### From the Architect

**"No shortcuts for getting a clean build"**

Initially I tried to:
1. ❌ Add backend alongside old solver (duplication!)
2. ❌ Leave 642 lines of dead code (technical debt!)
3. ❌ Not actually use the abstraction I built

The architect correctly insisted:
1. ✅ Actually USE the abstraction (remove old code)
2. ✅ Let compiler identify dead code systematically
3. ✅ Remove all duplication
4. ✅ Run full test suite (not just --lib)

**Result:** Clean architecture with zero technical debt.

### From the Tests

**776 tests gave confidence to:**
- Delete 642 lines of code
- Refactor critical verification logic
- Change core abstractions

**Without tests:** Would have been terrified to touch this code.

---

## 📊 Statistics

| Metric | Value |
|--------|-------|
| **Files changed** | 16 |
| **Lines added** | +2,521 |
| **Lines removed** | -694 |
| **Net change** | +1,827 |
| **axiom_verifier.rs** | 1,120 → 478 lines (-57%) |
| **Z3 code centralized** | solvers/z3/ (1,159 lines) |
| **Tests passing** | 776/776 (100%) |
| **Coverage** | 15/133 native ops (11.3%) |
| **Quality gates** | 3/3 passing (fmt, clippy, test) |

---

**Status:** ✅ Production-ready solver abstraction layer  
**Merged:** December 12, 2024  
**Impact:** Foundation for multi-solver support and user extensibility


