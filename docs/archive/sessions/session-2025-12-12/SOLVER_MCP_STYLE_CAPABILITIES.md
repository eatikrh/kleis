# Solver Modules with MCP-Style Capability Declaration

**Date:** December 12, 2025  
**Inspiration:** Model Context Protocol (MCP)  
**Idea:** Solvers declare their coverage like MCP servers declare resources

---

## 🎯 The MCP Pattern

**Model Context Protocol servers declare capabilities:**
```json
{
  "capabilities": {
    "resources": [...],
    "tools": [...],
    "prompts": [...]
  }
}
```

**We can apply this to solver modules!**

---

## 🏗️ Solver Capability Declaration

### Solver Manifest (JSON/TOML)

**File:** `src/solvers/z3/capabilities.toml`

```toml
[solver]
name = "Z3"
version = "4.12.0"
type = "smt"
description = "Z3 SMT Solver from Microsoft Research"

[capabilities]
# Theories supported
theories = ["arithmetic", "boolean", "equality", "uninterpreted_functions"]

# Native operation translators
[capabilities.operations]

[capabilities.operations.arithmetic]
plus = { arity = 2, theory = "Int/Real", native = true }
minus = { arity = 2, theory = "Int/Real", native = true }
times = { arity = 2, theory = "Int/Real", native = true }
divide = { arity = 2, theory = "Real", native = true, notes = "Not implemented yet" }
negate = { arity = 1, theory = "Int/Real", native = true, notes = "Could add" }

[capabilities.operations.comparison]
equals = { arity = 2, theory = "Any", native = true }
lt = { arity = 2, theory = "Int/Real", native = true }
gt = { arity = 2, theory = "Int/Real", native = true }
le = { arity = 2, theory = "Int/Real", native = true }
ge = { arity = 2, theory = "Int/Real", native = true }
neq = { arity = 2, theory = "Any", native = false, fallback = "uninterpreted" }

[capabilities.operations.boolean]
and = { arity = 2, theory = "Bool", native = true }
or = { arity = 2, theory = "Bool", native = true }
not = { arity = 1, theory = "Bool", native = true }
implies = { arity = 2, theory = "Bool", native = true }

[capabilities.operations.unsupported]
# Explicitly mark as uninterpreted (no native translator)
sin = { arity = 1, native = false, fallback = "uninterpreted", reason = "No symbolic math" }
cos = { arity = 1, native = false, fallback = "uninterpreted", reason = "No symbolic math" }
matrix_multiply = { arity = 2, native = false, fallback = "uninterpreted", reason = "Abstract reasoning" }
# ... 118 more

[capabilities.features]
quantifiers = true
uninterpreted_functions = true
recursive_functions = true
datatypes = true
arrays = false
bitvectors = false
floating_point = true

[capabilities.performance]
max_axioms = 10000
timeout_ms = 5000
parallel_solving = false

[capabilities.result_types]
verification = true  # Can verify axioms
evaluation = true    # Can evaluate expressions
simplification = false  # Cannot simplify symbolically
proof_generation = false  # Doesn't generate readable proofs
```

---

## 🔌 Solver Module Interface (MCP-Inspired)

### Rust API

```rust
/// Solver module descriptor (loaded from capabilities file)
pub struct SolverCapabilities {
    pub name: String,
    pub version: String,
    pub solver_type: SolverType,
    pub theories: Vec<String>,
    pub operations: OperationCapabilities,
    pub features: FeatureFlags,
    pub performance: PerformanceSpecs,
    pub result_types: ResultTypeSupport,
}

pub struct OperationCapabilities {
    /// Operations with native translators
    pub native: HashMap<String, OperationSpec>,
    
    /// Operations explicitly marked as unsupported
    pub unsupported: HashMap<String, UnsupportedSpec>,
}

pub struct OperationSpec {
    pub name: String,
    pub arity: usize,
    pub theory: String,
    pub native: bool,
    pub fallback: Option<String>,
    pub notes: Option<String>,
}

/// Solver module (like MCP server)
pub trait SolverModule {
    /// Get solver capabilities (like MCP list_resources)
    fn capabilities(&self) -> &SolverCapabilities;
    
    /// Check if operation is supported
    fn supports(&self, operation: &str) -> OperationSupport;
    
    /// Get coverage for a set of operations
    fn coverage(&self, operations: &[String]) -> CoverageReport;
    
    /// Translate expression (if supported)
    fn translate(&mut self, expr: &Expression) -> Result<SolverExpression, String>;
}

pub enum OperationSupport {
    Native { translator: String },
    Uninterpreted { reason: String },
    Unsupported { reason: String },
}
```

---

## 📊 MCP-Style Capability Query

### CLI Interface (Like MCP)

```bash
# List available solvers (like MCP list servers)
$ kleis solver list
Available solvers:
  ✅ Z3 (v4.12.0) - SMT solver
  ⚠️  CVC5 (not installed)
  ⚠️  Custom (not configured)

# Get solver capabilities (like MCP describe server)
$ kleis solver capabilities Z3
Solver: Z3
Version: 4.12.0
Type: SMT
Theories: [arithmetic, boolean, equality, uninterpreted_functions]

Native Operations (15):
  ✅ plus (arity: 2, theory: Int/Real)
  ✅ minus (arity: 2, theory: Int/Real)
  ✅ times (arity: 2, theory: Int/Real)
  ✅ eq (arity: 2, theory: Any)
  ... (11 more)

Uninterpreted Operations (118):
  ⚠️  sin (reason: No symbolic math, fallback: uninterpreted)
  ⚠️  matrix_multiply (reason: Abstract reasoning)
  ... (116 more)

Features:
  ✅ Quantifiers
  ✅ Uninterpreted functions
  ✅ Recursive functions
  ❌ Symbolic simplification
  
Performance:
  Max axioms: 10,000
  Timeout: 5000ms

# Check coverage for specific program
$ kleis solver check-coverage --solver Z3 stdlib/prelude.kleis
Analyzing: stdlib/prelude.kleis
Solver: Z3

Operations used: 45
  Native: 15 (33.3%) ✅
  Uninterpreted: 30 (66.7%) ⚠️

Missing translators (could add):
  Priority HIGH: divide, negate (core arithmetic)
  Priority MED: abs, pow (numeric)
  Priority LOW: sin, cos (use axioms instead)

Recommendations:
  ✅ Current coverage sufficient for algebraic reasoning
  💡 Consider adding: divide, negate
  📚 Use axioms for: trig functions
```

---

## 🔧 Implementation: Capability Declaration

### Step 1: Define Capability Schema

**File:** `src/solvers/capabilities.rs`

```rust
use serde::{Deserialize, Serialize};

/// Solver capability declaration (MCP-inspired)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverCapabilities {
    pub solver: SolverInfo,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverInfo {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub solver_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub theories: Vec<String>,
    pub operations: OperationsCapabilities,
    pub features: FeaturesCapabilities,
    pub performance: PerformanceSpecs,
    pub result_types: ResultTypeSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsCapabilities {
    /// Categorized operations with specs
    pub arithmetic: HashMap<String, OperationSpec>,
    pub comparison: HashMap<String, OperationSpec>,
    pub boolean: HashMap<String, OperationSpec>,
    pub unsupported: HashMap<String, UnsupportedSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSpec {
    pub arity: usize,
    pub theory: String,
    pub native: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedSpec {
    pub arity: usize,
    pub native: bool,
    pub fallback: String,
    pub reason: String,
}
```

### Step 2: Load Capabilities at Runtime

```rust
impl Z3Backend {
    pub fn new() -> Result<Self, String> {
        // Load capabilities from file
        let capabilities = Self::load_capabilities()?;
        
        Ok(Z3Backend {
            solver: z3::Solver::new(),
            capabilities,
            registry: TranslatorRegistry::from_capabilities(&capabilities)?,
        })
    }
    
    fn load_capabilities() -> Result<SolverCapabilities, String> {
        // Load from embedded file or external config
        let manifest = include_str!("z3/capabilities.toml");
        toml::from_str(manifest)
            .map_err(|e| format!("Failed to parse capabilities: {}", e))
    }
}
```

### Step 3: Query Interface (MCP-style)

```rust
/// MCP-style capability queries
impl SolverModule for Z3Backend {
    fn list_operations(&self) -> Vec<OperationInfo> {
        let mut ops = Vec::new();
        
        // Native operations
        for (name, spec) in &self.capabilities.operations.arithmetic {
            ops.push(OperationInfo {
                name: name.clone(),
                support: OperationSupport::Native,
                arity: spec.arity,
                theory: spec.theory.clone(),
            });
        }
        
        // Uninterpreted operations
        for (name, spec) in &self.capabilities.operations.unsupported {
            ops.push(OperationInfo {
                name: name.clone(),
                support: OperationSupport::Uninterpreted(spec.reason.clone()),
                arity: spec.arity,
                theory: "Uninterpreted".to_string(),
            });
        }
        
        ops
    }
    
    fn supports(&self, operation: &str) -> OperationSupport {
        // Check if native
        if self.has_native_translator(operation) {
            return OperationSupport::Native;
        }
        
        // Check if explicitly unsupported
        if let Some(spec) = self.capabilities.operations.unsupported.get(operation) {
            return OperationSupport::Uninterpreted(spec.reason.clone());
        }
        
        // Unknown - will be uninterpreted
        OperationSupport::Uninterpreted("Not in registry".to_string())
    }
}
```

---

## 🌐 User Extensibility Pattern

### Users Can Register Custom Translators

**Config file:** `~/.kleis/solver_config.toml`

```toml
[[solvers]]
name = "Z3"
enabled = true

# User adds custom translators
[[solvers.Z3.translators]]
operation = "sin"
type = "taylor_approximation"
order = 5
code = """
  # Taylor series: sin(x) ≈ x - x³/6 + x⁵/120
  (- x (/ (* x x x) 6))
"""

[[solvers.Z3.translators]]
operation = "my_custom_op"
type = "plugin"
path = "~/.kleis/plugins/my_op.so"
entry_point = "translate_my_op"

[[solvers.Z3.translators]]
operation = "matrix_det"
type = "smt_lib_template"
template = "(det {0})"  # Pass to Z3's array theory
```

**Load at runtime:**
```rust
let config = SolverConfig::load_from_file("~/.kleis/solver_config.toml")?;
let mut z3_backend = Z3Backend::new()?;

// Register user translators
for translator_spec in config.user_translators {
    let translator = load_translator(&translator_spec)?;
    z3_backend.register_translator(translator);
}

// Now coverage increased!
let coverage = z3_backend.coverage();
println!("Coverage: {:.1}%", coverage.percentage);
```

---

## 📊 Coverage Report API (MCP-Inspired)

### Programmatic Interface

```rust
/// MCP-style capability discovery
pub struct SolverDiscovery;

impl SolverDiscovery {
    /// List all available solvers (like MCP list_resources)
    pub fn list_solvers() -> Vec<SolverInfo> {
        vec![
            SolverInfo {
                name: "Z3".to_string(),
                version: "4.12.0".to_string(),
                available: true,
                path: Some("/opt/homebrew/lib/libz3.dylib".to_string()),
            },
            SolverInfo {
                name: "CVC5".to_string(),
                version: "1.0.0".to_string(),
                available: false,
                path: None,
            },
        ]
    }
    
    /// Get capabilities for a solver (like MCP describe_resource)
    pub fn get_capabilities(solver_name: &str) -> Result<SolverCapabilities, String> {
        match solver_name {
            "Z3" => Z3Backend::load_capabilities(),
            "CVC5" => CVC5Backend::load_capabilities(),
            _ => Err(format!("Unknown solver: {}", solver_name)),
        }
    }
    
    /// Check coverage for operations (like MCP resource query)
    pub fn check_coverage(
        solver_name: &str,
        operations: &[String],
    ) -> Result<CoverageReport, String> {
        let caps = Self::get_capabilities(solver_name)?;
        Ok(caps.coverage_for_operations(operations))
    }
}
```

---

## 🎯 JSON API (MCP Protocol Style)

### Endpoint 1: List Solvers

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "solver/list",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "solvers": [
      {
        "name": "Z3",
        "version": "4.12.0",
        "type": "smt",
        "available": true,
        "description": "Z3 SMT Solver"
      },
      {
        "name": "CVC5",
        "version": "1.0.0",
        "type": "smt",
        "available": false,
        "description": "CVC5 SMT Solver (not installed)"
      }
    ]
  }
}
```

### Endpoint 2: Get Capabilities

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "solver/capabilities",
  "params": {
    "solver": "Z3"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "solver": {
      "name": "Z3",
      "version": "4.12.0",
      "type": "smt"
    },
    "capabilities": {
      "theories": ["arithmetic", "boolean", "equality"],
      "operations": {
        "native": [
          {"name": "plus", "arity": 2, "theory": "Int/Real"},
          {"name": "minus", "arity": 2, "theory": "Int/Real"},
          {"name": "times", "arity": 2, "theory": "Int/Real"},
          {"name": "eq", "arity": 2, "theory": "Any"},
          {"name": "lt", "arity": 2, "theory": "Int/Real"},
          ...
        ],
        "uninterpreted": [
          {"name": "sin", "arity": 1, "reason": "No symbolic math"},
          {"name": "cos", "arity": 1, "reason": "No symbolic math"},
          ...
        ]
      },
      "features": {
        "quantifiers": true,
        "uninterpreted_functions": true,
        "evaluation": true,
        "simplification": false
      }
    }
  }
}
```

### Endpoint 3: Check Coverage

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "solver/check_coverage",
  "params": {
    "solver": "Z3",
    "operations": ["plus", "minus", "sin", "cos", "matrix_multiply"]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "solver": "Z3",
    "total": 5,
    "native": 2,
    "uninterpreted": 3,
    "coverage_percentage": 40.0,
    "operations": [
      {"name": "plus", "support": "native"},
      {"name": "minus", "support": "native"},
      {"name": "sin", "support": "uninterpreted", "reason": "No symbolic math"},
      {"name": "cos", "support": "uninterpreted", "reason": "No symbolic math"},
      {"name": "matrix_multiply", "support": "uninterpreted", "reason": "Abstract reasoning"}
    ],
    "recommendations": [
      "sin, cos: Consider adding Taylor series approximation",
      "matrix_multiply: Use axioms for abstract reasoning"
    ]
  }
}
```

---

## 🔌 Plugin Registration (MCP-Style)

### Solver Plugin Manifest

**File:** `~/.kleis/plugins/my_solver/plugin.toml`

```toml
[plugin]
name = "MySolver"
version = "1.0.0"
type = "solver"
author = "User Name"

[capabilities]
theories = ["custom_theory"]

[capabilities.operations.custom]
my_operation = { arity = 2, native = true }
another_op = { arity = 1, native = true }

[plugin.rust_api]
library_path = "./libmy_solver.so"
init_function = "initialize_solver"
```

**Load dynamically:**
```rust
// Kleis loads plugins like MCP loads servers
let plugin = SolverPlugin::load_from_manifest("~/.kleis/plugins/my_solver/plugin.toml")?;
let backend = plugin.create_backend()?;

// Now available to Kleis!
solvers.register("MySolver", backend);
```

---

## 📋 Implementation Roadmap

### Phase 1: Capability Declaration (Week 1)

- [ ] Define `SolverCapabilities` struct
- [ ] Create `src/solvers/z3/capabilities.toml`
- [ ] Implement capability loading
- [ ] Add `capabilities()` method to Z3Backend

### Phase 2: Coverage Tracking (Week 1)

- [ ] Implement `CoverageReport` struct
- [ ] Add `coverage()` method
- [ ] CLI command: `kleis solver capabilities`
- [ ] CLI command: `kleis solver check-coverage`

### Phase 3: Plugin System (Week 2)

- [ ] Define `SolverModule` trait
- [ ] Implement plugin loader
- [ ] Support user-registered translators
- [ ] Document plugin API

### Phase 4: Multi-Solver Support (Week 3)

- [ ] Abstract `AxiomVerifier` to use `SolverBackend` trait
- [ ] Implement CVC5Backend or SMT-LIB generic backend
- [ ] Support solver selection in config
- [ ] Comparative coverage reports

---

## 🎯 File Structure

```
src/solvers/
├── mod.rs                       # Public API
├── capabilities.rs              # Capability structs
├── backend.rs                   # SolverBackend trait
├── registry.rs                  # Solver registry
├── coverage.rs                  # Coverage analysis
├── plugin.rs                    # Plugin loading
│
├── z3/
│   ├── mod.rs
│   ├── backend.rs               # Z3Backend
│   ├── capabilities.toml        # ← MCP-style manifest!
│   ├── translators/
│   │   ├── mod.rs
│   │   ├── arithmetic.rs
│   │   ├── comparison.rs
│   │   └── boolean.rs
│   └── converter.rs             # Result → AST
│
├── cvc5/                        # Future
│   ├── backend.rs
│   └── capabilities.toml
│
└── smt_lib/                     # Generic SMT-LIB backend
    ├── backend.rs
    └── capabilities.toml

tests/
├── solver_capabilities_test.rs  # Test capability loading
└── solver_coverage_test.rs      # Test coverage tracking
```

---

## 📝 Example: Complete User Story

### User Wants Better Coverage for Trigonometry

**Step 1: Check current coverage**
```bash
$ kleis solver check-coverage --solver Z3 my_trig_axioms.kleis
Operations used: sin, cos, tan
Native: 0/3 (0%)
Uninterpreted: 3/3 (100%)

Recommendation: Add Taylor series approximations for better Z3 reasoning
```

**Step 2: User creates translator plugin**
```rust
// ~/.kleis/plugins/trig/src/lib.rs
use kleis_solver_api::*;

#[kleis_translator_plugin]
pub struct TrigTranslator;

impl OperationTranslator for TrigTranslator {
    fn declare_capabilities(&self) -> Vec<OperationSpec> {
        vec![
            OperationSpec {
                name: "sin".to_string(),
                arity: 1,
                native: true,
                theory: "Real".to_string(),
                notes: Some("5th order Taylor series".to_string()),
            },
            OperationSpec {
                name: "cos".to_string(),
                arity: 1,
                native: true,
                theory: "Real".to_string(),
                notes: Some("5th order Taylor series".to_string()),
            },
        ]
    }
    
    fn translate(&self, op: &str, args: &[SolverExpression]) -> Result<SolverExpression, String> {
        match op {
            "sin" => taylor_sin(args[0], 5),
            "cos" => taylor_cos(args[0], 5),
            _ => Err(format!("Unsupported: {}", op)),
        }
    }
}
```

**Step 3: Register plugin**
```toml
# ~/.kleis/config.toml
[[solver.Z3.plugins]]
name = "TrigTranslator"
path = "~/.kleis/plugins/trig/libtrig_translator.so"
```

**Step 4: Verify improved coverage**
```bash
$ kleis solver check-coverage --solver Z3 my_trig_axioms.kleis
Operations used: sin, cos, tan
Native: 2/3 (66.7%) ✅ Improved!
Uninterpreted: 1/3 (33.3%)

✅ sin (plugin: TrigTranslator)
✅ cos (plugin: TrigTranslator)
⚠️  tan (uninterpreted)
```

---

## 🎯 Benefits of MCP-Style Design

### 1. Discoverability ✅
```rust
// Users can query: "What can Z3 do?"
let caps = Z3Backend::capabilities();
for op in caps.native_operations() {
    println!("✅ {}", op);
}
```

### 2. Extensibility ✅
```rust
// Users add translators without modifying Kleis
registry.register_translator(MyTranslator::new());
```

### 3. Multi-Solver ✅
```rust
// Compare solvers
for solver in ["Z3", "CVC5", "Custom"] {
    let coverage = check_coverage(solver, &my_operations);
    println!("{}: {:.1}%", solver, coverage.percentage);
}

// Pick best solver for the job
let best = pick_solver_with_best_coverage(&my_operations);
```

### 4. Debugging ✅
```bash
$ kleis why-uninterpreted sin
Operation: sin
Solver: Z3
Status: Uninterpreted
Reason: No symbolic mathematics support

Suggestions:
  1. Add Taylor series approximation (plugin available)
  2. Use axioms to constrain behavior
  3. Switch to solver with trig support (none available)
```

---

## ✅ Immediate Action

### For TODO #57: Document Intent

**Add comment to axiom_verifier.rs:**
```rust
// TODO (Future - ADR-023): Refactor to use SolverBackend trait
// This hardcoded implementation should become:
//   1. Capability declaration (capabilities.toml)
//   2. Pluggable translator registry
//   3. User-extensible translators
//   4. Multi-solver support
// See: docs/session-2025-12-12/SOLVER_ABSTRACTION_LAYER_DESIGN.md
```

### Create ADR-023 Draft

Document the planned architecture for future implementation.

---

## 🎯 Answer to Your Questions

**Q1: How to track Z3 translator coverage?**  
**A:** MCP-style capability declaration in `capabilities.toml` ✅

**Q2: User extensibility for translators?**  
**A:** Plugin system with `OperationTranslator` trait + registry ✅

**Q3: Support other solvers?**  
**A:** `SolverBackend` trait, each solver has capabilities manifest ✅

**Q4: Modular AST conversion?**  
**A:** `ResultConverter` trait, solver-specific converters ✅

**Q5: MCP-style declaration?**  
**A:** YES! ✅ Each solver declares capabilities upfront, discoverable at runtime!

---

## 📝 **Recommended Path Forward**

1. ✅ **Commit TODO #57** (current implementation works)
2. 📝 **Create ADR-023** (document this architecture)
3. 🏗️ **Implement in v0.7** (1-2 weeks of work)

**The MCP-style design is EXCELLENT!** Would you like me to:
- Commit TODO #57 with notes about future refactoring?
- Create ADR-023 draft for solver abstraction?
- Start the refactoring now?
