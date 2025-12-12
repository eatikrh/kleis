# Next Session: Modularize Solver Interface Using MCP Pattern

**Date Created:** December 12, 2024  
**Priority:** High  
**Goal:** Refactor Z3 integration to use MCP-style modular architecture

---

## 🎯 Objective

Transform the hardcoded Z3 implementation into a **pluggable solver backend system** inspired by Model Context Protocol (MCP), enabling:

1. ✅ **Capability Declaration** - Solvers declare coverage upfront (like MCP resources)
2. ✅ **User Extensibility** - Users can add custom translators
3. ✅ **Multi-Solver Support** - Swap between Z3, CVC5, custom solvers
4. ✅ **Coverage Tracking** - Know what's natively supported vs uninterpreted
5. ✅ **Modular AST Conversion** - Solver-specific result converters

**⚠️ CRITICAL REQUIREMENT:** Solver module MUST return **Kleis AST**, not solver-specific types!
- ✅ Maintains abstraction boundary
- ✅ Enables solver independence
- ✅ Users work with Kleis expressions, not Z3/CVC5 internals

---

## 📚 Background Documents (Already Created)

**Design documents from this session:**
- `docs/session-2024-12-12/SOLVER_ABSTRACTION_LAYER_DESIGN.md` - Complete architecture
- `docs/session-2024-12-12/SOLVER_MCP_STYLE_CAPABILITIES.md` - MCP-inspired design
- `docs/session-2024-12-12/BUILTIN_FUNCTIONS_Z3_COVERAGE.md` - Current state analysis

**Key insights:**
- 133 builtin operations in Kleis
- Only 15 have Z3 translators (11% coverage)
- This is BY DESIGN (correct for theorem proving)
- But architecture needs to be modular and extensible

---

## 🏗️ Proposed Architecture

### Core Traits

**CRITICAL DESIGN REQUIREMENT:** Solver module MUST return Kleis AST, not solver-specific types!

```rust
/// Main solver abstraction
/// RETURNS: Kleis AST expressions, not solver-specific types
pub trait SolverBackend {
    fn name(&self) -> &str;
    fn capabilities(&self) -> &SolverCapabilities;
    fn supports_operation(&self, op: &str) -> bool;
    
    /// Verify axiom - returns verification result (not AST)
    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String>;
    
    /// Evaluate expression - MUST return Kleis AST! ← CRITICAL
    fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String>;
    
    /// Simplify expression - MUST return Kleis AST! ← CRITICAL
    fn simplify(&mut self, expr: &Expression) -> Result<Expression, String>;
}

/// Operation translator (internal to solver)
pub trait OperationTranslator {
    fn operation_name(&self) -> &str;
    fn arity(&self) -> usize;
    fn translate(&self, args: &[SolverExpression]) -> Result<SolverExpression, String>;
}

/// Result converter (internal - converts solver results to Kleis AST)
/// CRITICAL: This is how we maintain solver independence!
pub trait ResultConverter {
    /// Convert solver result to Kleis AST ← PRIMARY METHOD
    fn to_expression(&self, value: &SolverValue) -> Result<Expression, String>;
    
    /// Convenience methods (convert then extract)
    fn to_i64(&self, value: &SolverValue) -> Result<i64, String> {
        let expr = self.to_expression(value)?;
        match expr {
            Expression::Const(s) => s.parse().map_err(|_| "Not an integer".to_string()),
            _ => Err("Not a constant".to_string()),
        }
    }
}
```

---

## 📋 Implementation Tasks

### Phase 1: Core Abstraction (~4 hours)

**1.1 Create solver module structure**
- [ ] Create `src/solvers/` directory
- [ ] Create `src/solvers/mod.rs` with public API
- [ ] Create `src/solvers/backend.rs` - SolverBackend trait
- [ ] Create `src/solvers/capabilities.rs` - Capability structs
- [ ] Create `src/solvers/registry.rs` - TranslatorRegistry

**1.2 Define capability schema**
- [ ] Create `SolverCapabilities` struct
- [ ] Create `OperationSpec` struct
- [ ] Support TOML/JSON serialization (serde)
- [ ] Add validation logic

**1.3 Create Z3 backend directory**
- [ ] Move to `src/solvers/z3/`
- [ ] Create `src/solvers/z3/backend.rs` - Z3Backend impl
- [ ] Create `src/solvers/z3/capabilities.toml` - Manifest
- [ ] Extract translators to separate files

### Phase 2: Translator Registry (~3 hours)

**2.1 Extract current translators**
- [ ] Move arithmetic translators to `src/solvers/z3/translators/arithmetic.rs`
- [ ] Move comparison translators to `src/solvers/z3/translators/comparison.rs`
- [ ] Move boolean translators to `src/solvers/z3/translators/boolean.rs`
- [ ] Create translator registry system

**2.2 Implement uninterpreted fallback**
- [ ] Create generic uninterpreted function handler
- [ ] Log when falling back to uninterpreted
- [ ] Track uninterpreted operations used

**2.3 User extensibility hooks**
- [ ] Add `register_translator()` method
- [ ] Support dynamic translator loading
- [ ] Load from config file

### Phase 3: Coverage Tracking (~2 hours)

**3.1 Coverage analysis**
- [ ] Implement `CoverageAnalyzer` struct
- [ ] Method: `analyze(program)` - extract operations
- [ ] Method: `report()` - generate coverage report
- [ ] Compare coverage across solvers

**3.2 CLI commands**
- [ ] Add `kleis solver list` - List available solvers
- [ ] Add `kleis solver capabilities <name>` - Show solver info
- [ ] Add `kleis solver check-coverage <file>` - Analyze coverage
- [ ] Pretty formatting for reports

**3.3 Programmatic API**
- [ ] `SolverDiscovery::list_solvers()`
- [ ] `SolverDiscovery::get_capabilities(name)`
- [ ] `SolverDiscovery::check_coverage(name, ops)`

### Phase 4: Refactor AxiomVerifier (~3 hours)

**4.1 Use SolverBackend trait**
- [ ] Change `AxiomVerifier` to use `&dyn SolverBackend`
- [ ] Remove Z3-specific hardcoded logic
- [ ] Use backend methods for translation
- [ ] Maintain backward compatibility

**4.2 Update tests**
- [ ] Ensure all existing tests still pass
- [ ] Add tests for multi-solver support
- [ ] Add tests for coverage tracking
- [ ] Add tests for user translators

**4.3 Documentation**
- [ ] Update API docs
- [ ] Migration guide for users
- [ ] Plugin development guide

---

## 📁 Proposed File Structure

```
src/
├── axiom_verifier.rs           # Uses SolverBackend trait (refactored)
├── solvers/
│   ├── mod.rs                  # Public API, re-exports
│   ├── backend.rs              # SolverBackend trait
│   ├── capabilities.rs         # Capability structs (serde)
│   ├── registry.rs             # TranslatorRegistry
│   ├── coverage.rs             # CoverageAnalyzer
│   ├── result_converter.rs    # ResultConverter trait
│   ├── discovery.rs            # SolverDiscovery API
│   │
│   ├── z3/
│   │   ├── mod.rs
│   │   ├── backend.rs          # Z3Backend implementation
│   │   ├── capabilities.toml   # ← MCP-style manifest!
│   │   ├── converter.rs        # Z3ResultConverter
│   │   └── translators/
│   │       ├── mod.rs
│   │       ├── arithmetic.rs   # plus, minus, times
│   │       ├── comparison.rs   # eq, lt, gt, le, ge
│   │       ├── boolean.rs      # and, or, not, implies
│   │       └── registry.rs     # Z3 translator registry
│   │
│   └── plugins/
│       ├── mod.rs
│       ├── loader.rs           # Dynamic library loading
│       └── api.rs              # Plugin API for users

src/bin/
├── kleis_solver.rs            # CLI: solver list/capabilities/coverage

tests/
├── solver_capabilities_test.rs
├── solver_coverage_test.rs
└── solver_multi_backend_test.rs
```

---

## 📝 MCP-Style Capability Manifest

### Example: `src/solvers/z3/capabilities.toml`

```toml
[solver]
name = "Z3"
version = "4.12.0"
type = "smt"
description = "Z3 SMT Solver from Microsoft Research"

[capabilities]
theories = ["arithmetic", "boolean", "equality", "uninterpreted_functions"]

[capabilities.operations.arithmetic]
plus = { arity = 2, theory = "Int/Real", native = true }
minus = { arity = 2, theory = "Int/Real", native = true }
times = { arity = 2, theory = "Int/Real", native = true }

[capabilities.operations.comparison]
equals = { arity = 2, theory = "Any", native = true }
lt = { arity = 2, theory = "Int/Real", native = true }
gt = { arity = 2, theory = "Int/Real", native = true }
le = { arity = 2, theory = "Int/Real", native = true }
ge = { arity = 2, theory = "Int/Real", native = true }

[capabilities.operations.boolean]
and = { arity = 2, theory = "Bool", native = true }
or = { arity = 2, theory = "Bool", native = true }
not = { arity = 1, theory = "Bool", native = true }
implies = { arity = 2, theory = "Bool", native = true }

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

---

## 🔧 Migration Strategy

### Approach: Incremental Refactoring

**Step 1: Create new structure alongside old**
- Create `src/solvers/` with new architecture
- Keep `src/axiom_verifier.rs` working as-is
- Implement new Z3Backend

**Step 2: Add feature flag**
```rust
#[cfg(feature = "new-solver-api")]
use solvers::z3::Z3Backend;

#[cfg(not(feature = "new-solver-api"))]
// Use old axiom_verifier.rs
```

**Step 3: Migrate gradually**
- Get new API working
- Test thoroughly
- Switch default
- Remove old code

**Benefits:**
- ✅ No breaking changes
- ✅ Can test new API without breaking existing
- ✅ Safe migration path

---

## 🎯 Expected Outcomes

### After Refactoring:

**1. MCP-Style Capability Query**
```bash
$ kleis solver capabilities Z3
Solver: Z3 v4.12.0
Coverage: 15/133 operations (11.3%)
Native: plus, minus, times, eq, lt, gt, le, ge, and, or, not, implies, ...
Uninterpreted: sin, cos, matrix_multiply, ... (118 operations)
```

**2. User Extensibility**
```rust
// User adds custom translator
registry.register_translator(Box::new(MySinTranslator));

// Now sin is natively supported!
```

**3. Multi-Solver Support**
```rust
// Switch solvers easily
let verifier = AxiomVerifier::with_solver(Box::new(Z3Backend::new()));
// or
let verifier = AxiomVerifier::with_solver(Box::new(CVC5Backend::new()));
```

**4. Coverage Tracking**
```bash
$ kleis solver check-coverage stdlib/prelude.kleis
Analyzing 45 operations...
Z3: 15 native (33%), 30 uninterpreted (67%)
```

---

## ⚠️ Challenges to Address

### Challenge 1: Dynamic Type Handling

**Current issue:** Z3 returns `Dynamic`, needs conversion
```rust
let result = func_decl.apply(&[&x]);  // Returns Dynamic
let result_int = result.as_int()?;    // Convert to Int for arithmetic
```

**Solution:** ResultConverter trait handles per-solver

### Challenge 2: Backward Compatibility

**Must ensure:** All existing code continues to work

**Strategy:** Feature flags + gradual migration

### Challenge 3: Plugin Safety

**Security concern:** Loading external .so files

**Solution:**
- Sandbox plugins
- Verify signatures
- Or only support "declarative" plugins (TOML/SMT-LIB templates)

---

## 📊 Estimated Timeline

| Phase | Tasks | Time | Deliverable |
|-------|-------|------|-------------|
| **Phase 1** | Core abstraction | 4 hours | Traits + directory structure |
| **Phase 2** | Translator registry | 3 hours | Modular translators |
| **Phase 3** | Coverage tracking | 2 hours | CLI commands + reports |
| **Phase 4** | Refactor verifier | 3 hours | Uses SolverBackend trait |
| **Testing** | Integration tests | 2 hours | All tests passing |
| **Documentation** | ADR-023, guides | 2 hours | Complete docs |
| **Total** | | **16 hours** | **~2 days focused work** |

---

## 🎯 Success Criteria

**Definition of Done:**
- [ ] `SolverBackend` trait defined and documented
- [ ] `capabilities.toml` loaded at runtime
- [ ] Coverage tracking API working
- [ ] CLI commands: `solver list`, `capabilities`, `check-coverage`
- [ ] User can register custom translators
- [ ] All 600+ tests still passing
- [ ] ADR-023 created and approved
- [ ] Migration guide for future solvers
- [ ] Quality gates passing

---

## 📖 References

**Design Documents (This Session):**
- `SOLVER_ABSTRACTION_LAYER_DESIGN.md` - Complete architecture
- `SOLVER_MCP_STYLE_CAPABILITIES.md` - MCP-inspired design
- `BUILTIN_FUNCTIONS_Z3_COVERAGE.md` - Current coverage analysis

**Related ADRs:**
- ADR-022: Z3 Integration (current implementation)
- ADR-023: Solver Abstraction Layer (to be created)

**Inspiration:**
- Model Context Protocol (MCP) - Server capability declaration
- Rust trait system - Pluggable backends
- LSP (Language Server Protocol) - Similar abstraction pattern

---

## 🚀 Getting Started (Next Session)

### Step 1: Review Current State
```bash
# Count Z3-specific code
grep -r "z3::" src/axiom_verifier.rs | wc -l

# List all operations with translators
grep "\"plus\"|\"minus\"|\"times\"" src/axiom_verifier.rs
```

### Step 2: Create Directory Structure
```bash
mkdir -p src/solvers/z3/translators
touch src/solvers/mod.rs
touch src/solvers/backend.rs
touch src/solvers/z3/capabilities.toml
```

### Step 3: Define Traits
```rust
// Start with SolverBackend trait
// Implement for Z3Backend
// Test with existing functionality
```

### Step 4: Extract Translators
```rust
// Move arithmetic translators out of axiom_verifier.rs
// Create modular translator files
// Register in TranslatorRegistry
```

---

## 💡 Quick Wins

**Easy tasks to start with:**

1. **Create capabilities.toml** (30 minutes)
   - List all 15 native operations
   - Mark 118 as uninterpreted
   - Add metadata (arity, theory, reason)

2. **Define SolverBackend trait** (1 hour)
   - Core methods only
   - Simple trait definition
   - Document with examples

3. **Implement coverage reporting** (2 hours)
   - Extract operations from Expression
   - Check against capabilities
   - Generate report string

---

## 🎯 Vision: What Users Will See

### Command Line Interface

```bash
# Discover solvers
$ kleis solver list
Available solvers:
  ✅ Z3 (v4.12.0) - 15 native operations
  ⚠️  CVC5 (not installed)

# Check capabilities
$ kleis solver capabilities Z3
Solver: Z3 v4.12.0
Native operations: 15
  ✅ plus, minus, times
  ✅ eq, lt, gt, le, ge
  ✅ and, or, not, implies
Uninterpreted: 118
  ⚠️  sin, cos, tan (reason: No symbolic math)
  ⚠️  matrix_multiply (reason: Use axioms)
  
# Check coverage for specific program
$ kleis solver check-coverage stdlib/prelude.kleis
Operations used: 45
Z3 coverage: 15/45 (33.3%)
  ✅ plus, minus, times, eq, lt, ... (15 operations)
  ⚠️  sin, cos, gamma, ... (30 operations - uninterpreted)
  
Recommendation: Coverage sufficient for algebraic reasoning ✅
```

### Configuration File

```toml
# ~/.kleis/config.toml
[solver]
default = "Z3"
timeout_ms = 5000

[[solver.translators]]
operation = "sin"
type = "taylor"
order = 5

[[solver.translators]]
operation = "my_custom_op"
type = "plugin"
path = "~/.kleis/plugins/my_op.so"
```

---

## ✅ Preparation Checklist

**Before next session:**
- [x] Current implementation working and pushed ✅
- [x] Design documents created ✅
- [x] Architecture planned ✅
- [x] File structure proposed ✅
- [x] Timeline estimated ✅
- [ ] Review MCP specification (optional)
- [ ] Review serde TOML parsing examples (optional)

## 🔍 CRITICAL: Reference Z3 Source Code

**⚠️ IMPORTANT: Read the code at `/Users/eatik_1/Documents/git/cee/Z3` while modularizing the solver**

**Why this is critical:**
- Z3 source has real-world examples of proper API usage
- `/Users/eatik_1/Documents/git/cee/Z3/z3.rs/z3/tests/lib.rs` - Complete test examples
- Shows `RecFuncDecl` usage, quantifier patterns, model evaluation
- We just learned: RecFuncDecl (not FuncDecl) is correct for defined functions
- More discoveries likely as we refactor!

**Useful files:**
- `Z3/z3.rs/z3/tests/lib.rs` - Rust API examples
- `Z3/z3.rs/z3/src/` - Implementation details
- `Z3/z3/doc/` - Z3 documentation
- `Z3/z3/examples/` - C++/Python examples for reference

**Today's lesson:** We would have avoided test failures if we'd read the Z3 tests first!  
**Next session:** Reference Z3 source code as we design the abstraction layer.

---

## 🎉 Current State (End of This Session)

**What works:**
- ✅ Grammar v0.6 complete
- ✅ Functions in structures
- ✅ Z3 integration (hardcoded)
- ✅ Evaluator integration
- ✅ 600+ tests passing

**What needs refactoring:**
- ⚠️ Z3-specific code in axiom_verifier.rs (300+ lines)
- ⚠️ No capability declaration
- ⚠️ No coverage tracking
- ⚠️ No user extensibility
- ⚠️ Tied to single solver

**Goal for next session:**
- ✅ Modular solver backends
- ✅ MCP-style capabilities
- ✅ User extensibility
- ✅ Coverage tracking
- ✅ Foundation for multi-solver support

---

## 🚀 Let's Build This!

**Next session agenda:**
1. Create `src/solvers/` module structure
2. Define core traits
3. Create Z3 `capabilities.toml`
4. Implement coverage tracking
5. Begin extracting translators
6. Create ADR-023

**This will be a significant architectural improvement!** 🏗️

---

**Status:** Ready for next session ✅  
**Design:** Complete and documented ✅  
**Path:** Clear and achievable ✅

