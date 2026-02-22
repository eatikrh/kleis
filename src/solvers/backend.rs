//! Solver Backend Abstraction
//!
//! Defines the core trait for pluggable solver backends (Z3, CVC5, custom solvers).
//! Inspired by Model Context Protocol (MCP) - solvers declare capabilities upfront.
//!
//! **Design Principles:**
//! 1. **Solver Independence** - All public methods work with Kleis AST, not solver types
//! 2. **Capability Declaration** - Solvers declare what operations they natively support
//! 3. **Extensibility** - Users can add custom translators for operations
//! 4. **Coverage Tracking** - Know what's natively supported vs uninterpreted
//!
//! **CRITICAL: Return Type Contract**
//! - `evaluate()` and `simplify()` MUST return Kleis `Expression`, not solver types
//! - This maintains abstraction boundary and enables solver independence
//! - Internal translation to/from solver types is hidden in implementations
//!
//! See: docs/session-2024-12-12/SOLVER_ABSTRACTION_LAYER_DESIGN.md

use crate::ast::Expression;
use crate::kleis_ast::TypeExpr;
use crate::solvers::capabilities::SolverCapabilities;
use std::fmt;

/// A single binding in a witness: variable name → Kleis value.
///
/// When Z3 finds a counterexample or satisfying assignment, the witness
/// contains one `WitnessBinding` per quantified variable with its concrete value
/// expressed as a Kleis `Expression`.
///
/// # Example
/// For `∀(x : ℤ). x * x > 0`, a counterexample witness would contain:
/// ```ignore
/// WitnessBinding { name: "x".into(), value: Expression::Const("0".into()) }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct WitnessBinding {
    /// Original Kleis variable name (e.g., "x", not Z3's "x!0")
    pub name: String,
    /// Concrete value as a Kleis expression
    pub value: Expression,
}

/// Structured witness from a solver — a set of variable assignments that
/// either violate a universally quantified property (counterexample) or
/// satisfy an existentially quantified one (satisfying assignment).
///
/// The `bindings` field holds Kleis-native `Expression` values, enabling:
/// - **Round-tripping**: feed witness values back into `evaluate` for further reasoning
/// - **Composition**: use witness bindings in `let` expressions
/// - **Pretty-printing**: render in Kleis syntax via `PrettyPrinter`
/// - **CEGAR loops**: verify → get witness → refine property → verify again
///
/// The `raw` field preserves the original solver model output for debugging.
///
/// # Display
/// Formats as `x = 0, y = 42` — a comma-separated list of bindings.
/// For richer formatting (e.g., Kleis syntax with Unicode operators),
/// consumers should use `PrettyPrinter` on individual `binding.value` expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct Witness {
    /// Variable bindings: Kleis name → Kleis expression value
    pub bindings: Vec<WitnessBinding>,
    /// Original solver model output (for debugging / fallback display)
    pub raw: String,
}

impl Witness {
    /// Create a witness with no structured bindings, only a raw string.
    ///
    /// Used as a fallback when the solver model cannot be decomposed into
    /// individual variable assignments (e.g., uninterpreted function tables).
    pub fn from_raw(raw: String) -> Self {
        Witness {
            bindings: Vec::new(),
            raw,
        }
    }

    /// Check if the witness has any structured bindings.
    pub fn has_bindings(&self) -> bool {
        !self.bindings.is_empty()
    }
}

impl fmt::Display for WitnessBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Expression::Const(s) => write!(f, "{} = {}", self.name, s),
            Expression::String(s) => write!(f, "{} = \"{}\"", self.name, s),
            Expression::Object(s) => write!(f, "{} = {}", self.name, s),
            Expression::Operation { name, args, .. } => {
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| match a {
                        Expression::Const(s) => s.clone(),
                        Expression::Object(s) => s.clone(),
                        Expression::String(s) => format!("\"{}\"", s),
                        other => format!("{:?}", other),
                    })
                    .collect();
                write!(f, "{} = {}({})", self.name, name, arg_strs.join(", "))
            }
            other => write!(f, "{} = {:?}", self.name, other),
        }
    }
}

impl fmt::Display for Witness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bindings.is_empty() {
            // No structured bindings — fall back to raw solver output
            write!(f, "{}", self.raw)
        } else {
            let parts: Vec<String> = self.bindings.iter().map(|b| b.to_string()).collect();
            write!(f, "{}", parts.join(", "))
        }
    }
}

/// Result of axiom verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Axiom is valid (holds for all inputs)
    Valid,

    /// Axiom is valid AND we have a satisfying witness.
    /// Used for existential quantifiers: ∃(x). P(x) is valid, and here's an x that works.
    ValidWithWitness { witness: Witness },

    /// Axiom is invalid — the `witness` contains variable assignments
    /// that violate the property. The witness bindings are Kleis expressions.
    Invalid { witness: Witness },

    /// Solver couldn't determine (timeout, too complex, etc.)
    Unknown,
}

/// Result of satisfiability check
#[derive(Debug, Clone, PartialEq)]
pub enum SatisfiabilityResult {
    /// Expression is satisfiable — the `witness` contains variable assignments
    /// that make the expression true. The witness bindings are Kleis expressions.
    Satisfiable { witness: Witness },

    /// Expression is unsatisfiable (no assignment can make it true)
    Unsatisfiable,

    /// Solver couldn't determine (timeout, too complex, etc.)
    Unknown,
}

/// Main solver abstraction trait
///
/// Implementations wrap specific solvers (Z3, CVC5, etc.) and translate between
/// Kleis expressions and solver-specific representations.
///
/// # Example
/// ```ignore
/// let backend = Z3Backend::new()?;
/// let result = backend.verify_axiom(&axiom_expr)?;
/// match result {
///     VerificationResult::Valid => println!("✅ Verified!"),
///     VerificationResult::Invalid { witness } => {
///         println!("❌ Counterexample: {}", witness);
///         for binding in &witness.bindings {
///             println!("  {} = {:?}", binding.name, binding.value);
///         }
///     }
///     VerificationResult::Unknown => println!("⚠️ Unknown"),
/// }
/// ```
pub trait SolverBackend {
    /// Get solver name (e.g., "Z3", "CVC5")
    fn name(&self) -> &str;

    /// Get solver capabilities (declared upfront, MCP-style)
    fn capabilities(&self) -> &SolverCapabilities;

    /// Check if solver natively supports an operation
    ///
    /// Returns true if operation has a native translator, false if it will be
    /// treated as uninterpreted function.
    fn supports_operation(&self, op: &str) -> bool {
        self.capabilities().has_operation(op)
    }

    /// Verify an axiom using the solver (validity check)
    ///
    /// Checks if the axiom holds for ALL inputs by asserting its negation
    /// and checking satisfiability. If unsat, axiom is valid.
    ///
    /// Use this for: axioms, tautologies, universal truths
    ///
    /// # Arguments
    /// * `axiom` - Kleis expression (should be a boolean proposition)
    ///
    /// # Returns
    /// - `Valid` - Axiom holds for all inputs
    /// - `Invalid { witness }` - Found assignment that violates axiom (Kleis expressions)
    /// - `Unknown` - Solver couldn't determine (timeout, too complex)
    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String>;

    /// Check if an expression is satisfiable (existence check)
    ///
    /// Checks if there EXISTS an assignment that makes the expression true.
    ///
    /// Use this for: definitions, equations, "can this happen?" questions
    ///
    /// # Arguments
    /// * `expr` - Kleis expression (should be a boolean proposition)
    ///
    /// # Returns
    /// - `Satisfiable { witness }` - Found assignment that makes it true (Kleis expressions)
    /// - `Unsatisfiable` - No assignment can make it true
    /// - `Unknown` - Solver couldn't determine
    fn check_satisfiability(&mut self, expr: &Expression) -> Result<SatisfiabilityResult, String>;

    /// Evaluate an expression to a concrete value
    ///
    /// **CRITICAL:** MUST return Kleis Expression, not solver-specific type!
    ///
    /// # Arguments
    /// * `expr` - Kleis expression to evaluate
    ///
    /// # Returns
    /// Simplified Kleis expression (e.g., Expression::Const("42"))
    ///
    /// # Example
    /// ```ignore
    /// let expr = parse("2 + 3 * 4");
    /// let result = backend.evaluate(&expr)?;
    /// assert_eq!(result, Expression::Const("14".to_string()));
    /// ```
    fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String>;

    /// Simplify an expression using solver's rewrite rules
    ///
    /// **CRITICAL:** MUST return Kleis Expression, not solver-specific type!
    ///
    /// # Arguments
    /// * `expr` - Kleis expression to simplify
    ///
    /// # Returns
    /// Simplified Kleis expression
    ///
    /// # Example
    /// ```ignore
    /// let expr = parse("x + 0");
    /// let result = backend.simplify(&expr)?;
    /// assert_eq!(result, Expression::Object("x".to_string()));
    /// ```
    fn simplify(&mut self, expr: &Expression) -> Result<Expression, String>;

    /// Check if two expressions are equivalent
    ///
    /// Uses solver to determine if expr1 ≡ expr2 for all variable assignments.
    ///
    /// # Arguments
    /// * `expr1` - First Kleis expression
    /// * `expr2` - Second Kleis expression
    ///
    /// # Returns
    /// true if expressions are equivalent, false otherwise
    fn are_equivalent(&mut self, expr1: &Expression, expr2: &Expression) -> Result<bool, String>;

    /// Load a structure's axioms into the solver
    ///
    /// This is called on-demand when the solver needs axioms for reasoning.
    /// Structures are loaded with their dependencies (extends, over, where clauses).
    ///
    /// # Arguments
    /// * `structure_name` - Name of structure to load
    /// * `axioms` - List of axiom expressions from the structure
    fn load_structure_axioms(
        &mut self,
        structure_name: &str,
        axioms: &[Expression],
    ) -> Result<(), String>;

    /// Check if currently loaded axioms are consistent (satisfiable).
    ///
    /// Performs a bare `solver.check()` with no additional assertions.
    /// If the axioms alone are UNSAT, the theory is inconsistent and
    /// all verification results would be vacuously true.
    ///
    /// # Returns
    /// - `Ok(true)` if axioms are satisfiable (consistent)
    /// - `Ok(false)` if axioms are unsatisfiable (inconsistent)
    /// - `Err` if the check itself failed
    fn check_consistency(&mut self) -> Result<bool, String>;

    /// Push a new assertion scope (for incremental solving)
    ///
    /// Creates a checkpoint that can be restored with `pop()`.
    /// Useful for temporary assumptions in proofs.
    fn push(&mut self);

    /// Pop assertion scope (restore to previous checkpoint)
    ///
    /// # Arguments
    /// * `levels` - Number of scopes to pop (default 1)
    fn pop(&mut self, levels: u32);

    /// Reset solver to initial state
    ///
    /// Clears all assertions and loaded structures.
    fn reset(&mut self);

    /// Load an identity element (nullary operation) as a distinct constant
    ///
    /// Identity elements like `zero`, `one`, `e` are symbolic constants that
    /// must be mutually distinct. The solver asserts distinctness constraints
    /// between all loaded identity elements.
    ///
    /// # Arguments
    /// * `name` - Name of the identity element (e.g., "zero", "one")
    /// * `type_expr` - The type of the identity element
    ///
    /// # Example
    /// ```ignore
    /// backend.load_identity_element("zero", &TypeExpr::Named("M".to_string()));
    /// backend.load_identity_element("one", &TypeExpr::Named("M".to_string()));
    /// // Now zero ≠ one is asserted in the solver
    /// ```
    fn load_identity_element(&mut self, name: &str, type_expr: &TypeExpr);

    /// Check if a name is a constructor in a declared data type
    ///
    /// Returns true if the name matches a nullary constructor in any
    /// declared data type. Used to avoid loading ADT constructors as
    /// separate identity elements when they're already part of a data type.
    fn is_declared_constructor(&self, name: &str) -> bool;

    /// Assert a Kleis expression as a boolean constraint
    ///
    /// The expression must evaluate to a boolean in the solver.
    /// Used for asserting axioms, assumptions, and constraints.
    ///
    /// # Arguments
    /// * `expr` - Kleis expression (must be boolean)
    ///
    /// # Returns
    /// * `Ok(())` if assertion succeeded
    /// * `Err` if expression couldn't be translated or isn't boolean
    fn assert_expression(&mut self, expr: &Expression) -> Result<(), String>;

    /// Define a function with parameters and body
    ///
    /// Creates a function declaration and asserts its definitional equality:
    /// `∀ params. f(params) = body`
    ///
    /// # Arguments
    /// * `name` - Function name
    /// * `params` - Parameter names
    /// * `body` - Function body expression
    ///
    /// # Example
    /// ```ignore
    /// // Define: abs(x) = if x < 0 then -x else x
    /// backend.define_function("abs", &["x".to_string()], &abs_body)?;
    /// ```
    fn define_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &Expression,
    ) -> Result<(), String>;
}

/// Statistics about solver backend state
#[derive(Debug, Clone)]
pub struct SolverStats {
    /// Number of structures whose axioms are currently loaded
    pub loaded_structures: usize,
    /// Number of operations declared (native + uninterpreted)
    pub declared_operations: usize,
    /// Number of assertions currently in solver
    pub assertion_count: usize,
}
