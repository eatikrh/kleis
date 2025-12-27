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
use crate::solvers::capabilities::SolverCapabilities;

/// Result of axiom verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Axiom is valid (holds for all inputs)
    Valid,

    /// Axiom is invalid (counterexample found)
    Invalid { counterexample: String },

    /// Solver couldn't determine (timeout, too complex, etc.)
    Unknown,
}

/// Result of satisfiability check
#[derive(Debug, Clone, PartialEq)]
pub enum SatisfiabilityResult {
    /// Expression is satisfiable (there exists an assignment that makes it true)
    Satisfiable { example: String },

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
///     VerificationResult::Invalid { counterexample } => {
///         println!("❌ Counterexample: {}", counterexample)
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
    /// - `Invalid { counterexample }` - Found assignment that violates axiom
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
    /// - `Satisfiable { example }` - Found assignment that makes it true
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
    ///
    /// # Example
    /// ```ignore
    /// backend.load_identity_element("zero");
    /// backend.load_identity_element("one");
    /// // Now zero ≠ one is asserted in the solver
    /// ```
    fn load_identity_element(&mut self, name: &str);

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
