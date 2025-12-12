# Solver Abstraction Layer - Extensible Architecture

**Date:** December 12, 2024  
**Issue:** Z3 translators are hardcoded, not modular or extensible  
**Goal:** Design pluggable solver backend system

---

## 🎯 Your Key Insights

1. ✅ **Translator Coverage Tracking** - Need to know what's supported
2. ✅ **User Extensibility** - Users should add their own translators
3. ✅ **Solver Independence** - Could be Z3, CVC5, Yices, or custom
4. ✅ **Modular AST Conversion** - Solver results → Kleis AST should be pluggable
5. ✅ **Future-proofing** - Architecture should support multiple solvers

**This is sophisticated systems design!** 🎯

---

## 🏗️ Proposed Architecture: Solver Backend Trait

### Core Abstraction

```rust
/// Trait for SMT solver backends
///
/// Implementations: Z3Backend, CVC5Backend, CustomBackend
pub trait SolverBackend {
    /// Name of the solver (e.g., "Z3", "CVC5")
    fn name(&self) -> &str;
    
    /// Check if operation has native translator
    fn supports_operation(&self, op_name: &str) -> bool;
    
    /// Get coverage statistics
    fn coverage(&self) -> SolverCoverage;
    
    /// Translate Kleis expression to solver-specific AST
    fn translate_expression(
        &mut self,
        expr: &Expression,
        context: &TranslationContext,
    ) -> Result<SolverExpression, String>;
    
    /// Verify an axiom using the solver
    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String>;
    
    /// Evaluate expression to get concrete result (optional)
    fn evaluate(&mut self, expr: &Expression) -> Result<SolverValue, String>;
    
    /// Convert solver result back to Kleis AST (optional)
    fn result_to_ast(&self, value: &SolverValue) -> Result<Expression, String>;
}

/// Solver-agnostic expression representation
pub enum SolverExpression {
    Z3(z3::ast::Dynamic),
    CVC5(String),  // SMT-LIB string
    Custom(Box<dyn Any>),
}

/// Solver-agnostic value representation
pub enum SolverValue {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Symbolic(String),  // For uninterpreted values
}

/// Coverage information
pub struct SolverCoverage {
    pub total_operations: usize,
    pub supported_operations: Vec<String>,
    pub unsupported_operations: Vec<String>,
    pub coverage_percentage: f64,
}
```

---

## 📊 Translator Registry System

### User-Extensible Translator Registry

```rust
/// Registry of operation translators for a solver
pub struct TranslatorRegistry {
    /// Builtin translators (shipped with Kleis)
    builtin: HashMap<String, Box<dyn OperationTranslator>>,
    
    /// User-registered translators (loaded at runtime)
    user: HashMap<String, Box<dyn OperationTranslator>>,
}

/// Trait for translating a single operation
pub trait OperationTranslator: Send + Sync {
    /// Operation name (e.g., "sin", "plus", "matrix_multiply")
    fn operation_name(&self) -> &str;
    
    /// Arity (number of arguments)
    fn arity(&self) -> usize;
    
    /// Translate to Z3 (or other solver)
    fn translate(
        &self,
        args: &[SolverExpression],
        solver: &dyn SolverBackend,
    ) -> Result<SolverExpression, String>;
    
    /// Can this translator handle the operation?
    fn can_translate(&self, op_name: &str) -> bool {
        self.operation_name() == op_name
    }
}

impl TranslatorRegistry {
    /// Register a user-defined translator
    pub fn register_translator(&mut self, translator: Box<dyn OperationTranslator>) {
        let name = translator.operation_name().to_string();
        self.user.insert(name, translator);
    }
    
    /// Check coverage
    pub fn coverage(&self, operations: &[String]) -> SolverCoverage {
        let supported: Vec<String> = operations
            .iter()
            .filter(|op| self.has_translator(op))
            .cloned()
            .collect();
            
        let unsupported: Vec<String> = operations
            .iter()
            .filter(|op| !self.has_translator(op))
            .cloned()
            .collect();
            
        SolverCoverage {
            total_operations: operations.len(),
            supported_operations: supported.clone(),
            unsupported_operations: unsupported,
            coverage_percentage: (supported.len() as f64 / operations.len() as f64) * 100.0,
        }
    }
    
    fn has_translator(&self, op: &str) -> bool {
        self.builtin.contains_key(op) || self.user.contains_key(op)
    }
}
```

---

## 🔌 User Extensibility Pattern

### Example: User Adds Custom Translator

```kleis
// In user's .kleis file:
@solver_translator("Z3", "my_special_func")
define_translator my_special_func(x) {
    // Could be:
    // 1. Kleis code that defines the translation
    // 2. Reference to external plugin
    // 3. SMT-LIB template
}
```

**Or in Rust plugin:**
```rust
// User creates translator plugin
struct MySinTranslator;

impl OperationTranslator for MySinTranslator {
    fn operation_name(&self) -> &str { "sin" }
    fn arity(&self) -> usize { 1 }
    
    fn translate(
        &self,
        args: &[SolverExpression],
        solver: &dyn SolverBackend,
    ) -> Result<SolverExpression, String> {
        // Custom translation logic
        // Could use Taylor series, lookup table, etc.
    }
}

// Register at runtime
registry.register_translator(Box::new(MySinTranslator));
```

---

## 🔄 Multi-Solver Support

### Solver Backend Implementations

```rust
/// Z3 Backend
pub struct Z3Backend {
    solver: z3::Solver,
    registry: TranslatorRegistry,
}

impl SolverBackend for Z3Backend {
    fn name(&self) -> &str { "Z3" }
    
    fn supports_operation(&self, op: &str) -> bool {
        // Check if we have Z3-specific translator
        self.registry.has_translator(op)
    }
    
    fn translate_expression(
        &mut self,
        expr: &Expression,
        context: &TranslationContext,
    ) -> Result<SolverExpression, String> {
        // Use registry to find translator
        // Or fall back to uninterpreted
    }
}

/// CVC5 Backend (future)
pub struct CVC5Backend {
    // CVC5-specific state
    registry: TranslatorRegistry,
}

impl SolverBackend for CVC5Backend {
    fn name(&self) -> &str { "CVC5" }
    
    fn translate_expression(
        &mut self,
        expr: &Expression,
        context: &TranslationContext,
    ) -> Result<SolverExpression, String> {
        // CVC5-specific translation
        // Could generate SMT-LIB strings
    }
}

/// Custom/User Backend
pub struct CustomSolverBackend {
    // User-provided implementation
    translation_fn: Box<dyn Fn(&Expression) -> Result<String, String>>,
    verification_fn: Box<dyn Fn(&str) -> Result<VerificationResult, String>>,
}
```

---

## 📊 Coverage Tracking System

### Built-in Coverage Reporter

```rust
/// Coverage analyzer
pub struct CoverageAnalyzer {
    solvers: Vec<Box<dyn SolverBackend>>,
}

impl CoverageAnalyzer {
    /// Analyze coverage for a Kleis program
    pub fn analyze(&self, program: &Program) -> CoverageReport {
        // 1. Extract all operations used
        let operations = self.extract_operations(program);
        
        // 2. Check each solver's coverage
        let mut solver_coverage = HashMap::new();
        for solver in &self.solvers {
            let coverage = solver.coverage();
            solver_coverage.insert(solver.name(), coverage);
        }
        
        CoverageReport {
            total_operations: operations.len(),
            solver_coverage,
            recommendations: self.generate_recommendations(&operations),
        }
    }
    
    /// Generate coverage report
    pub fn report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Solver Coverage Report ===\n\n");
        
        for solver in &self.solvers {
            let cov = solver.coverage();
            report.push_str(&format!(
                "Solver: {}\n",
                solver.name()
            ));
            report.push_str(&format!(
                "  Supported: {}/{} ({:.1}%)\n",
                cov.supported_operations.len(),
                cov.total_operations,
                cov.coverage_percentage
            ));
            report.push_str("\n  Supported operations:\n");
            for op in &cov.supported_operations {
                report.push_str(&format!("    ✅ {}\n", op));
            }
            report.push_str("\n  Unsupported (using uninterpreted):\n");
            for op in cov.unsupported_operations.iter().take(10) {
                report.push_str(&format!("    ⚠️  {}\n", op));
            }
            report.push_str("\n");
        }
        
        report
    }
}

pub struct CoverageReport {
    pub total_operations: usize,
    pub solver_coverage: HashMap<String, SolverCoverage>,
    pub recommendations: Vec<String>,
}
```

---

## 🔧 Modular AST Conversion

### Solver Result Converter Trait

```rust
/// Trait for converting solver results back to Kleis AST
pub trait ResultConverter {
    /// Convert solver value to Kleis expression
    fn to_expression(&self, value: &SolverValue) -> Result<Expression, String>;
    
    /// Convert solver value to Rust type
    fn to_i64(&self, value: &SolverValue) -> Result<i64, String>;
    fn to_f64(&self, value: &SolverValue) -> Result<f64, String>;
    fn to_bool(&self, value: &SolverValue) -> Result<bool, String>;
}

/// Z3-specific converter
pub struct Z3ResultConverter;

impl ResultConverter for Z3ResultConverter {
    fn to_expression(&self, value: &SolverValue) -> Result<Expression, String> {
        match value {
            SolverValue::Integer(i) => Ok(Expression::Const(i.to_string())),
            SolverValue::Real(r) => Ok(Expression::Const(r.to_string())),
            SolverValue::Boolean(b) => {
                let const_name = if *b { "True" } else { "False" };
                Ok(Expression::Object(const_name.to_string()))
            }
            SolverValue::Symbolic(s) => {
                // Parse the symbolic representation
                // Could be: "(+ x y)" → parse back to AST
                parse_solver_expression(s)
            }
        }
    }
    
    fn to_i64(&self, value: &SolverValue) -> Result<i64, String> {
        match value {
            SolverValue::Integer(i) => Ok(*i),
            _ => Err("Not an integer value".to_string()),
        }
    }
}

/// CVC5-specific converter (future)
pub struct CVC5ResultConverter;

impl ResultConverter for CVC5ResultConverter {
    // CVC5-specific conversion logic
}
```

---

## 📝 Configuration File for Coverage

### `.kleis_solver_config.toml`

```toml
[solvers]
default = "Z3"

[solvers.Z3]
enabled = true
version = "4.12.0"

# Coverage tracking
[solvers.Z3.translators]
# Core arithmetic (100% coverage)
plus = "native"
minus = "native"
times = "native"
divide = "native"  # Could add!
negate = "native"  # Could add!

# Comparisons (75% coverage)
equals = "native"
lt = "native"
gt = "native"
le = "native"
ge = "native"
neq = "uninterpreted"  # Could add!

# Boolean logic (100% coverage)
and = "native"
or = "native"
not = "native"
implies = "native"

# Mathematics (0% coverage - intentional)
sin = "uninterpreted"
cos = "uninterpreted"
tan = "uninterpreted"
# ... etc

# User can add:
[solvers.Z3.user_translators]
my_custom_op = { type = "plugin", path = "./plugins/my_op.so" }
my_special_func = { type = "smt-lib", template = "(custom-smt ...)" }

[solvers.CVC5]
enabled = false
# Future: CVC5-specific configuration
```

---

## 🔌 Plugin System for User Translators

### Dynamic Loading Architecture

```rust
/// Plugin system for user translators
pub struct TranslatorPluginSystem {
    plugins: HashMap<String, Box<dyn OperationTranslator>>,
}

impl TranslatorPluginSystem {
    /// Load translators from configuration
    pub fn load_from_config(&mut self, config: &SolverConfig) -> Result<(), String> {
        for (op_name, plugin_spec) in &config.user_translators {
            match plugin_spec.plugin_type.as_str() {
                "builtin" => {
                    // Load from Kleis standard library
                    self.load_builtin_translator(op_name)?;
                }
                "plugin" => {
                    // Load shared library (.so/.dylib/.dll)
                    self.load_dynamic_translator(&plugin_spec.path)?;
                }
                "smt-lib" => {
                    // Use SMT-LIB template
                    self.load_template_translator(op_name, &plugin_spec.template)?;
                }
                "kleis" => {
                    // Defined in Kleis code itself!
                    self.load_kleis_translator(op_name, &plugin_spec.code)?;
                }
                _ => return Err(format!("Unknown translator type: {}", plugin_spec.plugin_type)),
            }
        }
        Ok(())
    }
}
```

### Example: User-Defined Translator in Kleis

```kleis
// User can define how to translate operations!
@solver_translator("Z3")
define_translator sin_approximation(x) {
  // Use Taylor series approximation
  // sin(x) ≈ x - x³/6 + x⁵/120
  translator_code {
    smt: "(- x (/ (* x x x) 6))"
    accuracy: "3rd order"
  }
}
```

---

## 🗺️ Proposed File Structure

```
src/
├── solvers/
│   ├── mod.rs                    # Solver abstraction traits
│   ├── backend.rs                # SolverBackend trait
│   ├── registry.rs               # TranslatorRegistry
│   ├── coverage.rs               # Coverage tracking
│   ├── result_converter.rs       # ResultConverter trait
│   │
│   ├── z3/
│   │   ├── mod.rs
│   │   ├── backend.rs            # Z3Backend implementation
│   │   ├── translators/
│   │   │   ├── arithmetic.rs     # plus, minus, times
│   │   │   ├── comparison.rs     # eq, lt, gt
│   │   │   ├── boolean.rs        # and, or, not
│   │   │   └── registry.rs       # Z3 translator registry
│   │   └── converter.rs          # Z3ResultConverter
│   │
│   ├── cvc5/                     # Future
│   │   ├── backend.rs
│   │   └── translators/
│   │
│   └── plugins/
│       ├── loader.rs             # Dynamic plugin loading
│       └── api.rs                # Plugin API for users
│
└── axiom_verifier.rs            # Uses SolverBackend trait

tests/
└── solver_coverage_test.rs      # Test coverage reporting
```

---

## 📋 Coverage Tracking Implementation

### Automatic Coverage Report

```rust
/// Generate coverage report for loaded program
pub fn generate_coverage_report(
    program: &Program,
    solver: &dyn SolverBackend,
) -> String {
    // 1. Extract all operations from program
    let operations = extract_all_operations(program);
    
    // 2. Check which are supported
    let mut supported = Vec::new();
    let mut unsupported = Vec::new();
    
    for op in &operations {
        if solver.supports_operation(op) {
            supported.push(op.clone());
        } else {
            unsupported.push(op.clone());
        }
    }
    
    // 3. Generate report
    format!(
        "Solver: {}\n\
         Total operations: {}\n\
         Supported: {} ({:.1}%)\n\
         Unsupported: {} (will use uninterpreted functions)\n\n\
         Supported operations:\n{}\n\n\
         Unsupported operations:\n{}",
        solver.name(),
        operations.len(),
        supported.len(),
        (supported.len() as f64 / operations.len() as f64) * 100.0,
        unsupported.len(),
        supported.iter().map(|s| format!("  ✅ {}", s)).collect::<Vec<_>>().join("\n"),
        unsupported.iter().map(|s| format!("  ⚠️  {}", s)).collect::<Vec<_>>().join("\n"),
    )
}

/// CLI command
pub fn kleis_solver_coverage(program_file: &str) {
    let program = parse_kleis_file(program_file)?;
    let solver = Z3Backend::new();
    let report = generate_coverage_report(&program, &solver);
    println!("{}", report);
}
```

**Usage:**
```bash
$ kleis solver-coverage stdlib/prelude.kleis

Solver: Z3
Total operations: 45
Supported: 15 (33.3%)
Unsupported: 30 (will use uninterpreted functions)

Supported operations:
  ✅ plus
  ✅ minus
  ✅ times
  ✅ eq
  ✅ lt
  ...

Unsupported operations:
  ⚠️  sin (suggestion: add Taylor series approximation)
  ⚠️  matrix_multiply (suggestion: use abstract reasoning)
  ⚠️  gamma_func (suggestion: use uninterpreted + axioms)
  ...
```

---

## 🎯 Migration Path

### Phase 1: Extract Current Code (Refactor)

**Move from:**
```rust
// axiom_verifier.rs (monolithic)
fn operation_to_z3_dynamic(&mut self, name: &str, ...) {
    match name {
        "plus" => /* Z3-specific code */,
        "times" => /* Z3-specific code */,
        // ... 100+ lines of hardcoded translations
    }
}
```

**To:**
```rust
// solvers/z3/backend.rs (modular)
impl Z3Backend {
    fn translate(&mut self, expr: &Expression) -> Result<SolverExpression, String> {
        // Use registry
        if let Some(translator) = self.registry.get(op_name) {
            translator.translate(args, self)
        } else {
            // Fallback to uninterpreted
            self.declare_uninterpreted(op_name, arity)
        }
    }
}
```

**Estimated:** 1 day to refactor

### Phase 2: Add Coverage Tracking

- Implement `SolverCoverage` struct
- Add `coverage()` method to backends
- Create coverage report generator
- Add CLI command: `kleis solver-coverage`

**Estimated:** Half day

### Phase 3: Plugin System

- Define `OperationTranslator` trait
- Implement dynamic loading
- Support user-registered translators
- Document plugin API

**Estimated:** 1-2 days

### Phase 4: Multi-Solver Support

- Abstract away Z3-specific code
- Implement CVC5 backend (or other)
- Support solver selection
- Comparative coverage reports

**Estimated:** 2-3 days

---

## 📝 Example: Complete User Story

### User Wants to Add `sin` Translator

**Step 1: Create translator plugin**
```rust
// ~/.kleis/plugins/trig_translator.rs
use kleis_solver_api::*;

#[kleis_translator]
pub struct TrigTranslator;

impl OperationTranslator for TrigTranslator {
    fn operation_name(&self) -> &str { "sin" }
    
    fn translate(&self, args: &[SolverExpression], solver: &dyn SolverBackend) -> Result<SolverExpression, String> {
        // Use Taylor series or lookup table
        // sin(x) ≈ x - x³/6 + x⁵/120
        taylor_approximation(args[0], 3)
    }
}
```

**Step 2: Register in config**
```toml
[solvers.Z3.user_translators]
sin = { type = "plugin", path = "~/.kleis/plugins/libtrig_translator.so" }
```

**Step 3: Use in Kleis**
```kleis
axiom: ∀(x). sin(-x) = -sin(x)
// Now Z3 uses the custom translator! ✅
```

**Step 4: Check coverage**
```bash
$ kleis solver-coverage my_program.kleis
Solver: Z3
✅ sin (custom translator loaded)
✅ cos (custom translator loaded)
Coverage: 17/45 (37.8%) ← Improved from 33.3%!
```

---

## ✅ Immediate Action Items

### For Current Implementation (TODO #57)

**Keep it simple for now:**
- ✅ Current hardcoded translators work
- ✅ Uninterpreted fallback works
- ✅ 11% coverage is sufficient
- ✅ Document as "Phase 1" design

**Mark as "Refactoring Opportunity":**
- Create ADR for solver abstraction
- Plan refactoring for v0.7 or v1.0
- Document the architecture you've outlined

### For Future Sessions

1. **Create ADR-023:** Solver Abstraction Layer
2. **Refactor:** Extract Z3-specific code to backend
3. **Implement:** Translator registry
4. **Add:** Coverage tracking
5. **Design:** Plugin API for users
6. **Support:** Multiple solvers

**Estimated total:** 1-2 weeks of focused work

---

## 🎯 Answers to Your Questions

### Q1: How to track Z3 translator coverage?

**A:** Create `SolverCoverage` system:
- Extract operations from program
- Check which have translators
- Generate report with percentages
- CLI tool: `kleis solver-coverage`

### Q2: User extensibility for translators?

**A:** Plugin system:
- `OperationTranslator` trait
- User registers translators
- Load from `.so` plugins or Kleis code
- Configuration file specifies custom translators

### Q3: Support other solvers?

**A:** `SolverBackend` trait:
- Z3Backend, CVC5Backend, CustomBackend
- Each implements translation for their solver
- Kleis code stays solver-agnostic
- User selects solver via config

### Q4: Modular AST conversion?

**A:** `ResultConverter` trait:
- Solver-specific converters
- to_expression(), to_i64(), to_bool()
- Pluggable for each backend

---

## ✅ Recommendation

**For TODO #57 (now):** Keep current design, it works! ✅

**For future:** Implement solver abstraction layer (ADR-023)

**Would you like me to:**
1. Commit TODO #57 as-is (works, can refactor later)?
2. Create ADR-023 draft for solver abstraction?
3. Start refactoring now (will take time)?
