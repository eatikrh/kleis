//! Axiom Verification using Z3 Theorem Prover
//!
//! This module provides verification of Kleis axioms by translating them to Z3
//! and checking if they're satisfiable/valid.
//!
//! **Architecture: Incremental Z3 Solving with Smart Caching**
//! - Long-lived Solver instance with push/pop for efficiency
//! - Axiom filtering: Only loads relevant axioms for each query
//! - Structure dependency analysis: Understands type relationships
//! - Uninterpreted functions: Custom operations declared in Z3
//! - Scales to thousands of axioms efficiently
//!
//! **Key Design Decisions:**
//! - Z3 Rust bindings use global context internally (no lifetime management)
//! - Solver persists across queries
//! - Each verify_axiom() uses push/pop (lightweight, ~1ms)
//! - Axioms loaded on-demand based on expression analysis
//! - Background theory cached per structure combination
//!
//! **Usage:**
//! ```ignore
//! let registry = StructureRegistry::new();
//! let verifier = AxiomVerifier::new(&registry)?;
//! let result = verifier.verify_axiom(&axiom)?;
//! ```
use crate::ast::Expression;
use crate::solvers::backend::Witness;
use crate::structure_registry::StructureRegistry;
use std::collections::HashSet;

#[cfg(feature = "axiom-verification")]
use crate::solvers::backend::SolverBackend;
#[cfg(feature = "axiom-verification")]
use crate::solvers::z3::Z3Backend;

/// Result of axiom verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Axiom is valid (holds for all inputs)
    Valid,

    /// Axiom is valid AND we have a satisfying witness.
    /// Used for existential quantifiers: ∃(x). P(x) is valid, and here's an x that works.
    ValidWithWitness { witness: Witness },

    /// Axiom is invalid — the `witness` contains variable assignments (as Kleis expressions)
    /// that violate the property.
    Invalid { witness: Witness },

    /// Z3 couldn't determine (timeout, too complex, etc.)
    Unknown,

    /// Feature not enabled
    Disabled,

    /// Loaded axioms are mutually inconsistent (unsatisfiable).
    /// All verification results would be vacuously true — this is a theory bug.
    InconsistentAxioms,
}

/// Axiom verifier using Z3 with incremental solving and smart caching
///
/// This struct maintains a long-lived Z3 solver and intelligently loads
/// only relevant axioms for each verification query.
pub struct AxiomVerifier<'r> {
    #[cfg(feature = "axiom-verification")]
    /// Solver backend (Z3, CVC5, etc.) - handles low-level Z3 operations
    backend: Z3Backend<'r>,

    #[cfg(feature = "axiom-verification")]
    /// Structure registry - source of all Kleis structure definitions (CRITICAL!)
    registry: &'r StructureRegistry,

    #[cfg(feature = "axiom-verification")]
    /// Track which structures' axioms are currently loaded
    loaded_structures: HashSet<String>,

    #[cfg(feature = "axiom-verification")]
    /// Whether axiom consistency has been checked (avoids redundant checks)
    consistency_checked: bool,

    #[cfg(feature = "axiom-verification")]
    /// Result of last consistency check (None = not yet checked)
    axioms_consistent: Option<bool>,

    #[cfg(not(feature = "axiom-verification"))]
    _phantom: std::marker::PhantomData<&'r ()>,
}

impl<'r> AxiomVerifier<'r> {
    /// Create a new axiom verifier with structure registry context
    ///
    /// Initializes a long-lived Z3 solver. Axioms are loaded on-demand
    /// based on what structures are actually used in queries.
    ///
    /// # Arguments
    /// * `registry` - Structure registry containing operations and axioms
    ///
    /// # Example
    /// ```ignore
    /// let registry = StructureRegistry::new();
    /// let verifier = AxiomVerifier::new(&registry)?;
    /// ```
    #[cfg(feature = "axiom-verification")]
    pub fn new(registry: &'r StructureRegistry) -> Result<Self, String> {
        let mut backend = Z3Backend::new(registry)?;

        // Initialize Z3 with registry data (data types, axioms)
        // This must be done before any verification
        backend.initialize_from_registry()?;

        Ok(Self {
            backend,
            registry,
            loaded_structures: HashSet::new(),
            consistency_checked: false,
            axioms_consistent: None,
        })
    }

    /// Create verifier without axiom-verification feature
    #[cfg(not(feature = "axiom-verification"))]
    pub fn new(_registry: &'r StructureRegistry) -> Result<Self, String> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Load top-level function definitions from a program into Z3
    ///
    /// This enables user-defined functions to be used in axiom verification,
    /// including functions that use `if/then/else` conditionals.
    ///
    /// # Arguments
    /// * `program` - Parsed Kleis program containing function definitions
    ///
    /// # Example
    /// ```ignore
    /// let code = r#"
    ///     define abs(x) = if x > 0 then x else negate(x)
    ///     define clamp(x, lo, hi) = if x < lo then lo else if x > hi then hi else x
    /// "#;
    /// let program = parse_kleis_program(code)?;
    ///
    /// let mut verifier = AxiomVerifier::new(&registry)?;
    /// verifier.load_program_functions(&program)?;
    ///
    /// // Now abs() and clamp() can be used in verification
    /// let result = verifier.verify_axiom(&axiom_using_abs)?;
    /// ```
    #[cfg(feature = "axiom-verification")]
    pub fn load_program_functions(
        &mut self,
        program: &crate::kleis_ast::Program,
    ) -> Result<(), String> {
        use crate::kleis_ast::TopLevel;

        let mut count = 0;
        for item in &program.items {
            if let TopLevel::FunctionDef(func_def) = item {
                self.backend
                    .define_function(&func_def.name, &func_def.params, &func_def.body)?;
                println!(
                    "   ✅ Top-level function '{}' loaded into Z3",
                    func_def.name
                );
                count += 1;
            }
        }

        if count > 0 {
            println!("   📦 Loaded {} top-level function(s)", count);
        }

        Ok(())
    }

    /// Load program functions (stub when feature disabled)
    #[cfg(not(feature = "axiom-verification"))]
    pub fn load_program_functions(
        &mut self,
        _program: &crate::kleis_ast::Program,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Load ADT constructor names as identity elements
    ///
    /// Nullary constructors (like TCP, UDP, ICMP) are values, not functions.
    /// They need to be registered as Z3 constants so expressions like
    /// `Packet(4, 5, 100, 64, TCP, src, dst)` can be translated.
    ///
    /// NOTE: This skips constructors that are already declared in Z3 data types.
    /// Those are handled via `get_nullary_constructor` instead.
    #[cfg(feature = "axiom-verification")]
    pub fn load_adt_constructors<I, S>(&mut self, constructors: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        use crate::kleis_ast::TypeExpr;
        // ADT constructors without explicit types default to Int
        let default_type = TypeExpr::Named("Int".to_string());
        for name in constructors {
            let name_str = name.as_ref();
            // Skip if this is already a declared data type constructor
            if self.backend.is_declared_constructor(name_str) {
                continue;
            }
            self.backend.load_identity_element(name_str, &default_type);
        }
    }

    /// Load ADT constructors (stub when feature disabled)
    #[cfg(not(feature = "axiom-verification"))]
    pub fn load_adt_constructors<I, S>(&mut self, _constructors: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
    }

    /// Pre-seed the consistency check result from an external cache.
    /// This avoids redundant Z3 consistency checks when the evaluator
    /// already knows the result from a previous verifier instance.
    #[cfg(feature = "axiom-verification")]
    pub fn set_consistency_cache(&mut self, cached: Option<bool>) {
        self.consistency_checked = true;
        self.axioms_consistent = cached;
    }

    #[cfg(not(feature = "axiom-verification"))]
    pub fn set_consistency_cache(&mut self, _cached: Option<bool>) {}

    /// Get the consistency check result (for external caching).
    /// Returns None if not yet checked, Some(Some(true/false)) if checked.
    #[cfg(feature = "axiom-verification")]
    pub fn get_consistency_result(&self) -> Option<Option<bool>> {
        if self.consistency_checked {
            Some(self.axioms_consistent)
        } else {
            None
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    pub fn get_consistency_result(&self) -> Option<Option<bool>> {
        None
    }

    /// Analyze expression to find which structures it depends on
    ///
    /// This enables smart axiom loading - only load axioms for structures
    /// that are actually used in the expression being verified.
    #[cfg(feature = "axiom-verification")]
    fn analyze_dependencies(&self, expr: &Expression) -> HashSet<String> {
        let mut structures = HashSet::new();

        match expr {
            Expression::Operation { name, args, .. } => {
                // Check if this operation belongs to a known structure
                if let Some(owners) = self.registry.get_operation_owners(name) {
                    structures.extend(owners);
                }

                // Recursively analyze arguments
                for arg in args {
                    structures.extend(self.analyze_dependencies(arg));
                }
            }

            Expression::Quantifier {
                body, where_clause, ..
            } => {
                structures.extend(self.analyze_dependencies(body));
                if let Some(condition) = where_clause {
                    structures.extend(self.analyze_dependencies(condition));
                }
            }

            Expression::Object(name) => {
                // Check if this object is actually a nullary operation (like e, zero, one)
                // Nullary operations appear as Object when used in expressions
                if let Some(owners) = self.registry.get_operation_owners(name) {
                    structures.extend(owners);
                }
                // Otherwise it's a true variable and introduces no dependencies
            }

            Expression::Const(_) | Expression::String(_) => {
                // Constants and strings don't introduce dependencies
            }

            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Analyze all three parts of the conditional
                structures.extend(self.analyze_dependencies(condition));
                structures.extend(self.analyze_dependencies(then_branch));
                structures.extend(self.analyze_dependencies(else_branch));
            }

            Expression::Match {
                scrutinee, cases, ..
            } => {
                // Analyze scrutinee and all case bodies
                structures.extend(self.analyze_dependencies(scrutinee));
                for case in cases {
                    structures.extend(self.analyze_dependencies(&case.body));
                }
            }

            Expression::List(elements) => {
                // Analyze all list elements
                for elem in elements {
                    structures.extend(self.analyze_dependencies(elem));
                }
            }

            Expression::Placeholder { .. } => {
                // Placeholders don't introduce dependencies
            }

            Expression::Let { value, body, .. } => {
                // Analyze both the value and body of let binding
                structures.extend(self.analyze_dependencies(value));
                structures.extend(self.analyze_dependencies(body));
            }
            Expression::Ascription { expr, .. } => {
                // Analyze the inner expression
                structures.extend(self.analyze_dependencies(expr));
            }
            Expression::Lambda { body, .. } => {
                // Analyze the body of the lambda
                structures.extend(self.analyze_dependencies(body));
            }
        }

        structures
    }

    /// Load axioms for a specific structure if not already loaded
    ///
    /// This is called on-demand when we detect a structure is needed.
    /// Loads:
    /// 1. Identity elements (zero, one, e) as Z3 constants
    /// 2. Operations (for future uninterpreted functions)
    /// 3. Axioms as background assumptions
    /// 4. Where constraint structures (e.g., if implements X where Y, also load Y)
    #[cfg(feature = "axiom-verification")]
    fn ensure_structure_loaded(&mut self, structure_name: &str) -> Result<(), String> {
        // Already loaded?
        if self.loaded_structures.contains(structure_name) {
            return Ok(());
        }

        // FIRST: Load structures from where constraints
        // This ensures constrained structure axioms are available as assumptions
        let where_constraints = self.registry.get_where_constraints(structure_name);
        for constraint in where_constraints {
            // Recursively load constrained structures
            // Example: where Semiring(T) → load Semiring axioms
            println!(
                "   🔗 Loading where constraint: {}",
                constraint.structure_name
            );
            self.ensure_structure_loaded(&constraint.structure_name)?;
        }

        // Get structure definition from registry
        let structure = self
            .registry
            .get(structure_name)
            .ok_or_else(|| format!("Structure not found: {}", structure_name))?;

        // SECOND: Load parent structure if extends clause present (inheritance!)
        // This ensures parent structure axioms are available
        if let Some(extends_type) = &structure.extends_clause {
            // Extract parent structure name
            let parent_name = match extends_type {
                crate::kleis_ast::TypeExpr::Named(name) => name.clone(),
                crate::kleis_ast::TypeExpr::Parametric(name, _) => name.clone(),
                _ => return Err("Invalid extends clause type".to_string()),
            };

            println!("   🔗 Loading parent structure: {}", parent_name);
            self.ensure_structure_loaded(&parent_name)?;
        }

        // THIRD: Load field structure if over clause present
        // This ensures field axioms are available for vector space reasoning
        if let Some(over_type) = &structure.over_clause {
            // Extract field structure name
            // Example: over Field(ℝ) → load Field
            let field_name = match over_type {
                crate::kleis_ast::TypeExpr::Named(name) => name.clone(),
                crate::kleis_ast::TypeExpr::Parametric(name, _) => name.clone(),
                _ => return Err("Invalid over clause type".to_string()),
            };

            println!("   🔗 Loading over clause: {}", field_name);
            self.ensure_structure_loaded(&field_name)?;
        }

        // Phase 1: Load identity elements (nullary operations: zero, one, e, etc.)
        // This includes identity elements in nested structures!
        self.load_identity_elements_recursive(&structure.members);

        // Phase 2: Get and load axioms (including from nested structures)
        // Use push/pop so that if axiom loading fails partway through,
        // the partially-asserted axioms are rolled back. Without this,
        // a type mismatch in one axiom leaves earlier axioms asserted,
        // potentially making the solver state UNSAT (inconsistent).
        //
        // On success we pop and re-load at the base level. This avoids
        // accumulating push levels (one per loaded structure), which can
        // degrade solver performance over many structures.
        println!("   Loading axioms for {}...", structure_name);
        self.backend.push();
        match self.load_axioms_recursive(&structure.members) {
            Err(e) => {
                eprintln!("   ❌ ERROR loading axioms: {}", e);
                self.backend.pop(1);
                return Err(e);
            }
            Ok(()) => {
                // Axioms loaded successfully in the trial scope.
                // Pop the trial scope and re-assert at the base level so we
                // don't accumulate push levels across many structure loads.
                self.backend.pop(1);
                // Re-load is safe: we know it succeeded once, so it won't fail.
                if let Err(e) = self.load_axioms_recursive(&structure.members) {
                    eprintln!(
                        "   ❌ Unexpected error re-loading axioms for {}: {}",
                        structure_name, e
                    );
                    return Err(e);
                }
            }
        }
        println!("   ✅ Axioms loaded successfully");

        // Mark as loaded
        self.loaded_structures.insert(structure_name.to_string());
        println!("   ✅ Marked {} as loaded", structure_name);

        Ok(())
    }

    /// Recursively load identity elements from structure members
    /// Handles nested structures automatically
    #[cfg(feature = "axiom-verification")]
    fn load_identity_elements_recursive(&mut self, members: &[crate::kleis_ast::StructureMember]) {
        use crate::kleis_ast::{StructureMember, TypeExpr};

        for member in members {
            match member {
                StructureMember::Operation {
                    name,
                    type_signature,
                } => {
                    // Check if nullary (identity element)
                    let is_nullary = !matches!(type_signature, TypeExpr::Function(..));

                    if is_nullary {
                        // Delegate to backend to load identity element with its type
                        self.backend.load_identity_element(name, type_signature);
                    }
                }
                StructureMember::NestedStructure { members, .. } => {
                    // Recursively process nested structure members
                    self.load_identity_elements_recursive(members);
                }
                _ => {
                    // Field or Axiom - not an identity element
                }
            }
        }
    }

    /// Recursively load axioms from structure members
    /// Handles axioms and function definitions in nested structures
    ///
    /// Grammar v0.6: Also loads function definitions as axioms.
    /// Example: define (-)(x, y) = x + negate(y)
    /// Becomes Z3 axiom: ∀(x y). minus(x, y) = plus(x, negate(y))
    #[cfg(feature = "axiom-verification")]
    fn load_axioms_recursive(
        &mut self,
        members: &[crate::kleis_ast::StructureMember],
    ) -> Result<(), String> {
        use crate::kleis_ast::StructureMember;

        for member in members {
            match member {
                StructureMember::Axiom { proposition, .. } => {
                    // Delegate to backend for assertion
                    self.backend.assert_expression(proposition)?;
                }
                StructureMember::FunctionDef(func_def) => {
                    // Grammar v0.6: Load function definition via backend
                    self.backend.define_function(
                        &func_def.name,
                        &func_def.params,
                        &func_def.body,
                    )?;
                    println!("   ✅ Function {} loaded via backend", func_def.name);
                }
                StructureMember::NestedStructure { members, .. } => {
                    // Recursively load axioms from nested structure
                    self.load_axioms_recursive(members)?;
                }
                _ => {
                    // Operation or Field - not an axiom
                }
            }
        }

        Ok(())
    }

    /// Verify a Kleis axiom using Z3 with incremental solving
    ///
    /// Uses push/pop to avoid polluting the global solver state.
    /// Automatically loads relevant axioms based on expression analysis.
    ///
    /// # How it works
    /// 1. Analyze expression to find dependent structures
    /// 2. Load axioms for those structures (cached)
    /// 3. Push a new assertion scope
    /// 4. Assert the NEGATION of the axiom
    /// 5. Check satisfiability
    /// 6. Pop the assertion scope (cleanup)
    ///
    /// If the negation is UNSAT, the axiom is valid!
    ///
    /// # Example
    /// ```ignore
    /// // axiom identity: ∀(x : M). x + 0 = x
    /// let verifier = AxiomVerifier::new(&registry)?;
    /// let result = verifier.verify_axiom(&axiom_expr)?;
    /// match result {
    ///     VerificationResult::Valid => println!("✅ Axiom verified!"),
    ///     VerificationResult::Invalid { witness } => {
    ///         println!("❌ Counterexample: {}", witness)
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub fn verify_axiom(&mut self, expr: &Expression) -> Result<VerificationResult, String> {
        #[cfg(feature = "axiom-verification")]
        {
            self.verify_axiom_impl(expr)
        }

        #[cfg(not(feature = "axiom-verification"))]
        {
            let _ = expr; // Suppress unused variable warning
            Ok(VerificationResult::Disabled)
        }
    }

    #[cfg(feature = "axiom-verification")]
    fn verify_axiom_impl(&mut self, expr: &Expression) -> Result<VerificationResult, String> {
        // Step 1: Analyze dependencies
        let dependencies = self.analyze_dependencies(expr);

        // Step 2: Ensure all required axioms are loaded
        for structure in &dependencies {
            self.ensure_structure_loaded(structure)?;
        }

        // Step 2b: Load ALL structure axioms from registry
        // This ensures axioms for uninterpreted functions (like complex operations)
        // are available even when analyze_dependencies can't find the connection
        let all_structures: Vec<String> = self
            .registry
            .structures_with_axioms()
            .iter()
            .map(|s| (*s).clone())
            .collect();

        for structure in &all_structures {
            if !self.loaded_structures.contains(structure) {
                // Continue even if one structure fails to load
                // This allows complex axioms to work even if Field fails
                if let Err(e) = self.ensure_structure_loaded(structure) {
                    eprintln!("   ⚠️ Warning: Failed to load {}: {}", structure, e);
                }
            }
        }

        // Step 2c: Check axiom consistency (once per verifier lifetime)
        if !self.consistency_checked {
            self.consistency_checked = true;
            match self.backend.check_consistency() {
                Ok(true) => {
                    self.axioms_consistent = Some(true);
                }
                Ok(false) => {
                    self.axioms_consistent = Some(false);
                    return Ok(VerificationResult::InconsistentAxioms);
                }
                Err(_e) => {
                    self.axioms_consistent = None;
                }
            }
        } else if self.axioms_consistent == Some(false) {
            return Ok(VerificationResult::InconsistentAxioms);
        }

        // Step 3: Delegate to backend for verification (uses solver abstraction layer!)
        use crate::solvers::backend::VerificationResult as BackendResult;

        let backend_result = self.backend.verify_axiom(expr)?;

        // Convert backend result to AxiomVerifier result
        Ok(match backend_result {
            BackendResult::Valid => VerificationResult::Valid,
            BackendResult::ValidWithWitness { witness } => {
                VerificationResult::ValidWithWitness { witness }
            }
            BackendResult::Invalid { witness } => VerificationResult::Invalid { witness },
            BackendResult::Unknown => VerificationResult::Unknown,
        })
    }

    /// Check if an expression is satisfiable with axioms loaded
    ///
    /// Unlike the raw `Z3Backend::check_satisfiability`, this method:
    /// 1. Analyzes dependencies to find relevant structures
    /// 2. Loads all structure axioms to constrain uninterpreted functions
    /// 3. Then checks satisfiability
    ///
    /// This is essential for "computation via satisfiability" - finding values
    /// that satisfy constraints defined by axioms.
    ///
    /// # Example
    /// ```ignore
    /// // With fib axioms loaded:
    /// // axiom fib_zero: fib(0) = 0
    /// // axiom fib_one: fib(1) = 1  
    /// // axiom fib_rec: ∀n. fib(n+2) = fib(n+1) + fib(n)
    ///
    /// let result = verifier.check_satisfiability(&parse("fib(5) = x"))?;
    /// // result: Satisfiable { example: "x = 5" }
    /// ```
    pub fn check_satisfiability(
        &mut self,
        expr: &Expression,
    ) -> Result<crate::solvers::backend::SatisfiabilityResult, String> {
        #[cfg(feature = "axiom-verification")]
        {
            self.check_satisfiability_impl(expr)
        }

        #[cfg(not(feature = "axiom-verification"))]
        {
            let _ = expr; // Suppress unused variable warning
            Err("Axiom verification feature not enabled".to_string())
        }
    }

    #[cfg(feature = "axiom-verification")]
    fn check_satisfiability_impl(
        &mut self,
        expr: &Expression,
    ) -> Result<crate::solvers::backend::SatisfiabilityResult, String> {
        use crate::solvers::backend::SolverBackend;

        // Step 1: Analyze dependencies
        let dependencies = self.analyze_dependencies(expr);

        // Step 2: Ensure all required axioms are loaded
        for structure in &dependencies {
            self.ensure_structure_loaded(structure)?;
        }

        // Step 3: Load ALL structure axioms from registry (same as verify_axiom_impl)
        let all_structures: Vec<String> = self
            .registry
            .structures_with_axioms()
            .iter()
            .map(|s| (*s).clone())
            .collect();

        for structure in &all_structures {
            if !self.loaded_structures.contains(structure) {
                if let Err(e) = self.ensure_structure_loaded(structure) {
                    eprintln!("   ⚠️ Warning: Failed to load {}: {}", structure, e);
                }
            }
        }

        // Step 4: Delegate to backend for satisfiability check
        self.backend.check_satisfiability(expr)
    }

    /// Simplify an expression using Z3's simplification engine.
    ///
    /// This is the core of the `eval()` operation - it reduces ground terms
    /// (expressions with no free variables) to concrete values.
    ///
    /// # Examples
    /// ```ignore
    /// // Arithmetic simplification
    /// let result = verifier.simplify(&parse("1 + 2 * 3")?)?;
    /// // result: Const("7")
    ///
    /// // Conditional evaluation
    /// let result = verifier.simplify(&parse("if 5 <= 3 then 1 else 2")?)?;
    /// // result: Const("2")
    ///
    /// // Boolean simplification
    /// let result = verifier.simplify(&parse("true ∧ false")?)?;
    /// // result: Const("false")
    /// ```
    ///
    /// # Note
    /// This should only be called on ground terms (no free variables).
    /// For symbolic expressions, use `verify_axiom()` instead.
    pub fn simplify(&mut self, expr: &Expression) -> Result<Expression, String> {
        #[cfg(feature = "axiom-verification")]
        {
            use crate::solvers::backend::SolverBackend;
            self.backend.simplify(expr)
        }

        #[cfg(not(feature = "axiom-verification"))]
        {
            let _ = expr; // Suppress unused variable warning
            Err("eval() requires axiom-verification feature (Z3 backend)".to_string())
        }
    }

    /// Check if two expressions are equivalent
    ///
    /// Uses Z3 to determine if expr1 ≡ expr2 for all variable assignments.
    /// This is key for simplification and optimization!
    pub fn are_equivalent(
        &mut self,
        expr1: &Expression,
        expr2: &Expression,
    ) -> Result<bool, String> {
        #[cfg(feature = "axiom-verification")]
        {
            // Load relevant axioms for both expressions
            let deps1 = self.analyze_dependencies(expr1);
            let deps2 = self.analyze_dependencies(expr2);

            for structure in deps1.union(&deps2) {
                self.ensure_structure_loaded(structure)?;
            }

            // Delegate to backend
            self.backend.are_equivalent(expr1, expr2)
        }

        #[cfg(not(feature = "axiom-verification"))]
        {
            let _ = (expr1, expr2); // Suppress warnings
            Err("Axiom verification feature not enabled".to_string())
        }
    }

    /// Get statistics about the verifier state
    ///
    /// Useful for debugging and performance monitoring.
    #[cfg(feature = "axiom-verification")]
    pub fn stats(&self) -> VerifierStats {
        let backend_stats = self.backend.stats();
        VerifierStats {
            loaded_structures: self.loaded_structures.len(),
            declared_operations: backend_stats.declared_operations,
        }
    }
}

/// Statistics about the verifier's current state
#[derive(Debug, Clone)]
pub struct VerifierStats {
    /// Number of structures whose axioms are currently loaded
    pub loaded_structures: usize,
    /// Number of operations declared as uninterpreted functions
    pub declared_operations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result_types() {
        let valid = VerificationResult::Valid;
        let invalid = VerificationResult::Invalid {
            witness: Witness::from_raw("x=1".to_string()),
        };
        let unknown = VerificationResult::Unknown;

        assert!(matches!(valid, VerificationResult::Valid));
        assert!(matches!(invalid, VerificationResult::Invalid { .. }));
        assert!(matches!(unknown, VerificationResult::Unknown));
    }

    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_verifier_creation_with_registry() {
        let registry = StructureRegistry::new();
        let verifier = AxiomVerifier::new(&registry);
        assert!(verifier.is_ok(), "Verifier creation should succeed");
    }

    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_dependency_analysis() {
        use crate::ast::Expression;

        let registry = StructureRegistry::new();
        let verifier = AxiomVerifier::new(&registry).unwrap();

        // Simple expression with no operations
        let expr = Expression::Const("5".to_string());
        let deps = verifier.analyze_dependencies(&expr);
        assert_eq!(deps.len(), 0, "Constant should have no dependencies");

        // Expression with operation
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("0".to_string()),
            ],
            span: None,
        };
        let deps = verifier.analyze_dependencies(&expr);
        // Dependencies depend on registry content
        println!("Dependencies for plus operation: {:?}", deps);
    }

    /// Helper: build a StructureDef with given members
    #[cfg(feature = "axiom-verification")]
    fn make_structure(
        name: &str,
        members: Vec<crate::kleis_ast::StructureMember>,
    ) -> crate::kleis_ast::StructureDef {
        crate::kleis_ast::StructureDef {
            name: name.to_string(),
            type_params: vec![],
            members,
            extends_clause: None,
            over_clause: None,
        }
    }

    /// Helper: build an axiom StructureMember from an Expression
    #[cfg(feature = "axiom-verification")]
    fn make_axiom(name: &str, prop: Expression) -> crate::kleis_ast::StructureMember {
        crate::kleis_ast::StructureMember::Axiom {
            name: name.to_string(),
            proposition: prop,
        }
    }

    /// Helper: build an operation StructureMember
    #[cfg(feature = "axiom-verification")]
    fn make_operation(
        name: &str,
        type_sig: crate::kleis_ast::TypeExpr,
    ) -> crate::kleis_ast::StructureMember {
        crate::kleis_ast::StructureMember::Operation {
            name: name.to_string(),
            type_signature: type_sig,
        }
    }

    /// Test that a successfully loaded structure keeps the solver consistent.
    /// An existential tautology ∃(x:Int). x = x should be satisfiable.
    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_successful_structure_load_keeps_consistency() {
        use crate::ast::Expression;
        use crate::kleis_ast::TypeExpr;

        let mut registry = StructureRegistry::new();

        // Register a simple structure: operation f : ℤ → ℤ, axiom: ∀x. f(x) >= 0
        let structure = make_structure(
            "SimpleStruct",
            vec![
                make_operation(
                    "f",
                    TypeExpr::Function(
                        Box::new(TypeExpr::Named("ℤ".to_string())),
                        Box::new(TypeExpr::Named("ℤ".to_string())),
                    ),
                ),
                make_axiom(
                    "f_nonneg",
                    Expression::Quantifier {
                        quantifier: crate::ast::QuantifierKind::ForAll,
                        variables: vec![crate::ast::QuantifiedVar {
                            name: "x".to_string(),
                            type_annotation: Some("ℤ".to_string()),
                        }],
                        body: Box::new(Expression::Operation {
                            name: "geq".to_string(),
                            args: vec![
                                Expression::Operation {
                                    name: "f".to_string(),
                                    args: vec![Expression::Object("x".to_string())],
                                    span: None,
                                },
                                Expression::Const("0".to_string()),
                            ],
                            span: None,
                        }),
                        where_clause: None,
                    },
                ),
            ],
        );
        registry.register(structure).unwrap();

        let mut verifier = AxiomVerifier::new(&registry).unwrap();
        let result = verifier.ensure_structure_loaded("SimpleStruct");
        assert!(result.is_ok(), "SimpleStruct should load successfully");

        // Existential tautology: ∃(x:ℤ). x = x
        let tautology = Expression::Quantifier {
            quantifier: crate::ast::QuantifierKind::Exists,
            variables: vec![crate::ast::QuantifiedVar {
                name: "x".to_string(),
                type_annotation: Some("ℤ".to_string()),
            }],
            body: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("x".to_string()),
                ],
                span: None,
            }),
            where_clause: None,
        };
        let result = verifier.verify_axiom(&tautology).unwrap();
        assert!(
            matches!(
                result,
                VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
            ),
            "Existential tautology should hold after successful structure load: got {:?}",
            result
        );
    }

    /// Test that a failed structure load rolls back partial axioms.
    /// We create a structure where the second axiom will fail (type mismatch).
    /// After the failure, the solver should remain consistent — an existential
    /// tautology should still be satisfiable, NOT return UNSAT.
    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_failed_structure_load_rolls_back() {
        use crate::ast::Expression;

        let mut registry = StructureRegistry::new();

        // A constraining axiom that narrows the model (1 = 1, always true)
        let good_axiom = make_axiom(
            "trivial",
            Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Const("1".to_string()),
                ],
                span: None,
            },
        );

        // Two axioms that use the same operation name "clash_op" with
        // incompatible argument types. The first call declares it as
        // Int×Int → Int; the second passes a String arg, triggering a
        // type mismatch error from the Z3 backend.
        let first_use = make_axiom(
            "setup_type",
            Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "clash_op".to_string(),
                        args: vec![
                            Expression::Const("1".to_string()),
                            Expression::Const("2".to_string()),
                        ],
                        span: None,
                    },
                    Expression::Const("0".to_string()),
                ],
                span: None,
            },
        );
        let conflicting_use = make_axiom(
            "will_fail",
            Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "clash_op".to_string(),
                        args: vec![
                            Expression::String("not_an_int".to_string()),
                            Expression::Const("2".to_string()),
                        ],
                        span: None,
                    },
                    Expression::Const("0".to_string()),
                ],
                span: None,
            },
        );

        let structure = make_structure("BadStruct", vec![good_axiom, first_use, conflicting_use]);
        registry.register(structure).unwrap();

        let mut verifier = AxiomVerifier::new(&registry).unwrap();

        // Loading should fail due to type conflict on "clash_op"
        let result = verifier.ensure_structure_loaded("BadStruct");
        assert!(
            result.is_err(),
            "BadStruct should fail to load due to type mismatch"
        );

        // After the failure, the solver should still be consistent.
        // ∃(x:ℤ). x = x should be satisfiable.
        let tautology = Expression::Quantifier {
            quantifier: crate::ast::QuantifierKind::Exists,
            variables: vec![crate::ast::QuantifiedVar {
                name: "x".to_string(),
                type_annotation: Some("ℤ".to_string()),
            }],
            body: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("x".to_string()),
                ],
                span: None,
            }),
            where_clause: None,
        };
        let result = verifier.verify_axiom(&tautology).unwrap();
        assert!(
            matches!(result, VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }),
            "Existential tautology should hold after failed structure load (rollback must work): got {:?}", result
        );
    }

    /// Test that after a failed structure load, other structures can still
    /// load and verify correctly — the failure doesn't poison the solver.
    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_failed_load_does_not_poison_subsequent_loads() {
        use crate::ast::Expression;
        use crate::kleis_ast::TypeExpr;

        let mut registry = StructureRegistry::new();

        // Bad structure: two axioms using the same op name with conflicting types
        let first_use = make_axiom(
            "setup_type",
            Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "clash_op2".to_string(),
                        args: vec![Expression::Const("1".to_string())],
                        span: None,
                    },
                    Expression::Const("0".to_string()),
                ],
                span: None,
            },
        );
        let conflicting_use = make_axiom(
            "will_fail",
            Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "clash_op2".to_string(),
                        args: vec![Expression::String("not_an_int".to_string())],
                        span: None,
                    },
                    Expression::Const("0".to_string()),
                ],
                span: None,
            },
        );
        registry
            .register(make_structure(
                "BadStruct",
                vec![first_use, conflicting_use],
            ))
            .unwrap();

        // Good structure: operation g : ℤ → ℤ, axiom: g(0) = 42
        let good_structure = make_structure(
            "GoodStruct",
            vec![
                make_operation(
                    "g",
                    TypeExpr::Function(
                        Box::new(TypeExpr::Named("ℤ".to_string())),
                        Box::new(TypeExpr::Named("ℤ".to_string())),
                    ),
                ),
                make_axiom(
                    "g_zero",
                    Expression::Operation {
                        name: "equals".to_string(),
                        args: vec![
                            Expression::Operation {
                                name: "g".to_string(),
                                args: vec![Expression::Const("0".to_string())],
                                span: None,
                            },
                            Expression::Const("42".to_string()),
                        ],
                        span: None,
                    },
                ),
            ],
        );
        registry.register(good_structure).unwrap();

        let mut verifier = AxiomVerifier::new(&registry).unwrap();

        // Load bad structure — should fail
        let result = verifier.ensure_structure_loaded("BadStruct");
        assert!(result.is_err(), "BadStruct should fail to load");

        // Load good structure — should succeed
        let result = verifier.ensure_structure_loaded("GoodStruct");
        assert!(
            result.is_ok(),
            "GoodStruct should load successfully after BadStruct failure"
        );

        // Verify that GoodStruct's axiom is usable: g(0) = 42
        let check = Expression::Operation {
            name: "equals".to_string(),
            args: vec![
                Expression::Operation {
                    name: "g".to_string(),
                    args: vec![Expression::Const("0".to_string())],
                    span: None,
                },
                Expression::Const("42".to_string()),
            ],
            span: None,
        };
        let result = verifier.verify_axiom(&check).unwrap();
        assert!(
            matches!(
                result,
                VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
            ),
            "g(0) = 42 should be verifiable after BadStruct failure + GoodStruct load: got {:?}",
            result
        );
    }

    /// Helper: build an existential ∃(x:ℤ). body expression.
    #[cfg(feature = "axiom-verification")]
    fn exists_int(name: &str, body: Expression) -> Expression {
        Expression::Quantifier {
            quantifier: crate::ast::QuantifierKind::Exists,
            variables: vec![crate::ast::QuantifiedVar {
                name: name.to_string(),
                type_annotation: Some("ℤ".to_string()),
            }],
            body: Box::new(body),
            where_clause: None,
        }
    }

    /// Helper: build a universal ∀(x:ℤ). body expression.
    #[cfg(feature = "axiom-verification")]
    fn forall_int(name: &str, body: Expression) -> Expression {
        Expression::Quantifier {
            quantifier: crate::ast::QuantifierKind::ForAll,
            variables: vec![crate::ast::QuantifiedVar {
                name: name.to_string(),
                type_annotation: Some("ℤ".to_string()),
            }],
            body: Box::new(body),
            where_clause: None,
        }
    }

    /// Helper: build `a = b` expression.
    #[cfg(feature = "axiom-verification")]
    fn eq(a: Expression, b: Expression) -> Expression {
        Expression::Operation {
            name: "equals".to_string(),
            args: vec![a, b],
            span: None,
        }
    }

    /// Verify that a successfully loaded structure's axioms are actually
    /// usable for proving things — not just that the solver "stays consistent."
    /// If axioms were silently dropped, the verification would fail.
    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_loaded_axioms_are_usable_for_verification() {
        use crate::ast::Expression;
        use crate::kleis_ast::TypeExpr;

        let mut registry = StructureRegistry::new();

        // Structure: operation h : ℤ → ℤ, axiom: h(0) = 99
        let structure = make_structure(
            "UsableStruct",
            vec![
                make_operation(
                    "h",
                    TypeExpr::Function(
                        Box::new(TypeExpr::Named("ℤ".to_string())),
                        Box::new(TypeExpr::Named("ℤ".to_string())),
                    ),
                ),
                make_axiom(
                    "h_zero",
                    eq(
                        Expression::Operation {
                            name: "h".to_string(),
                            args: vec![Expression::Const("0".to_string())],
                            span: None,
                        },
                        Expression::Const("99".to_string()),
                    ),
                ),
            ],
        );
        registry.register(structure).unwrap();

        let mut verifier = AxiomVerifier::new(&registry).unwrap();
        verifier.ensure_structure_loaded("UsableStruct").unwrap();

        // h(0) = 99 should be provable (it's an axiom we loaded)
        let provable = eq(
            Expression::Operation {
                name: "h".to_string(),
                args: vec![Expression::Const("0".to_string())],
                span: None,
            },
            Expression::Const("99".to_string()),
        );
        let result = verifier.verify_axiom(&provable).unwrap();
        assert!(
            matches!(
                result,
                VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
            ),
            "h(0) = 99 should be provable from loaded axioms: got {:?}",
            result
        );

        // h(0) = 77 should NOT be provable (contradicts the axiom)
        let not_provable = eq(
            Expression::Operation {
                name: "h".to_string(),
                args: vec![Expression::Const("0".to_string())],
                span: None,
            },
            Expression::Const("77".to_string()),
        );
        let result2 = verifier.verify_axiom(&not_provable).unwrap();
        assert!(
            matches!(result2, VerificationResult::Invalid { .. }),
            "h(0) = 77 should be disprovable (h(0)=99 is asserted): got {:?}",
            result2
        );
    }

    /// Verify that a failed load does NOT leave the solver in an inconsistent
    /// state due to partially asserted axioms.
    ///
    /// Strategy: after a failed load, verify that a trivially true existential
    /// (∃x. x=x) is still satisfiable. If partial axioms were leaked and
    /// introduced a contradiction, the solver would be UNSAT and this would
    /// fail. Also verify that a trivially false universal (∀x. x=0) is
    /// correctly rejected — if the solver were UNSAT, it would vacuously
    /// accept this.
    ///
    /// We test both the existential and universal sides because they catch
    /// different failure modes.
    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_failed_load_does_not_leave_partial_axioms() {
        use crate::ast::Expression;

        let mut registry = StructureRegistry::new();

        // Structure with a good axiom followed by a type-conflicting axiom.
        // The good axiom (1=1) gets asserted before the bad one fails.
        // Without rollback, this partial assertion could corrupt solver state.
        let good_first = make_axiom(
            "trivially_true",
            eq(
                Expression::Const("1".to_string()),
                Expression::Const("1".to_string()),
            ),
        );
        let bad_second = make_axiom(
            "will_fail",
            eq(
                Expression::Operation {
                    name: "clash_op3".to_string(),
                    args: vec![Expression::Const("1".to_string())],
                    span: None,
                },
                Expression::Operation {
                    name: "clash_op3".to_string(),
                    args: vec![Expression::String("oops".to_string())],
                    span: None,
                },
            ),
        );
        registry
            .register(make_structure(
                "PartialStruct",
                vec![good_first, bad_second],
            ))
            .unwrap();

        let mut verifier = AxiomVerifier::new(&registry).unwrap();

        // Loading should fail
        let result = verifier.ensure_structure_loaded("PartialStruct");
        assert!(result.is_err(), "PartialStruct should fail to load");

        // Existential check: ∃(x:ℤ). x = x should be satisfiable.
        // If partial axioms corrupted the solver (UNSAT), this would fail.
        let trivial_exists = exists_int(
            "x",
            eq(
                Expression::Object("x".to_string()),
                Expression::Object("x".to_string()),
            ),
        );
        let result = verifier.verify_axiom(&trivial_exists).unwrap();
        assert!(
            matches!(
                result,
                VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
            ),
            "∃x. x=x should be satisfiable after failed load (solver must not be UNSAT): got {:?}",
            result
        );

        // Universal check: ∀(x:ℤ). x = 0 is obviously false.
        // If the solver were UNSAT, this would be vacuously "valid."
        let absurd_forall = forall_int(
            "x",
            eq(
                Expression::Object("x".to_string()),
                Expression::Const("0".to_string()),
            ),
        );
        let result = verifier.verify_axiom(&absurd_forall).unwrap();
        assert!(
            !matches!(result, VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }),
            "∀x. x=0 should be INVALID after failed load (solver must not be vacuously true): got {:?}", result
        );
    }

    /// Verify that an obviously false universal statement is correctly
    /// rejected (not vacuously true). This catches the UNSAT-solver scenario
    /// where everything becomes "valid" due to inconsistency.
    #[cfg(feature = "axiom-verification")]
    #[test]
    fn test_false_universal_rejected_after_failed_load() {
        use crate::ast::Expression;

        let mut registry = StructureRegistry::new();

        // Bad structure that will fail to load
        let bad = make_axiom(
            "will_fail",
            eq(
                Expression::Operation {
                    name: "clash_op4".to_string(),
                    args: vec![Expression::Const("1".to_string())],
                    span: None,
                },
                Expression::Operation {
                    name: "clash_op4".to_string(),
                    args: vec![Expression::String("oops".to_string())],
                    span: None,
                },
            ),
        );
        registry
            .register(make_structure("BadStruct2", vec![bad]))
            .unwrap();

        let mut verifier = AxiomVerifier::new(&registry).unwrap();

        // Attempt to load (will fail)
        let _ = verifier.ensure_structure_loaded("BadStruct2");

        // ∀(x:ℤ). x = 0 is obviously false — if the solver says it's valid,
        // the solver state is UNSAT (inconsistent) and everything is vacuously true.
        let absurd = forall_int(
            "x",
            eq(
                Expression::Object("x".to_string()),
                Expression::Const("0".to_string()),
            ),
        );
        let result = verifier.verify_axiom(&absurd).unwrap();
        assert!(
            !matches!(
                result,
                VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
            ),
            "∀x. x=0 should be REJECTED (not vacuously true from an UNSAT solver): got {:?}",
            result
        );
    }
}
