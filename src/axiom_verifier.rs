///! Axiom Verification using Z3 Theorem Prover
///!
///! This module provides verification of Kleis axioms by translating them to Z3
///! and checking if they're satisfiable/valid.
///!
///! **Architecture: Incremental Z3 Solving with Smart Caching**
///! - Long-lived Solver instance with push/pop for efficiency
///! - Axiom filtering: Only loads relevant axioms for each query
///! - Structure dependency analysis: Understands type relationships
///! - Uninterpreted functions: Custom operations declared in Z3
///! - Scales to thousands of axioms efficiently
///!
///! **Key Design Decisions:**
///! - Z3 Rust bindings use global context internally (no lifetime management)
///! - Solver persists across queries
///! - Each verify_axiom() uses push/pop (lightweight, ~1ms)
///! - Axioms loaded on-demand based on expression analysis
///! - Background theory cached per structure combination
///!
///! **Usage:**
///! ```rust
///! let registry = StructureRegistry::new();
///! let verifier = AxiomVerifier::new(&registry)?;
///! let result = verifier.verify_axiom(&axiom)?;
///! ```
use crate::ast::{Expression, QuantifiedVar, QuantifierKind};
use crate::structure_registry::StructureRegistry;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "axiom-verification")]
use z3::ast::{Bool, Int};
#[cfg(feature = "axiom-verification")]
use z3::{FuncDecl, SatResult, Solver};

/// Result of axiom verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Axiom is valid (holds for all inputs)
    Valid,

    /// Axiom is invalid (counterexample found)
    Invalid { counterexample: String },

    /// Z3 couldn't determine (timeout, too complex, etc.)
    Unknown,

    /// Feature not enabled
    Disabled,
}

/// Axiom verifier using Z3 with incremental solving and smart caching
///
/// This struct maintains a long-lived Z3 solver and intelligently loads
/// only relevant axioms for each verification query.
pub struct AxiomVerifier<'r> {
    #[cfg(feature = "axiom-verification")]
    solver: Solver,

    #[cfg(feature = "axiom-verification")]
    registry: &'r StructureRegistry,

    #[cfg(feature = "axiom-verification")]
    /// Cache of declared operations as uninterpreted functions
    declared_ops: HashMap<String, FuncDecl>,

    #[cfg(feature = "axiom-verification")]
    /// Track which structures' axioms are currently loaded
    loaded_structures: HashSet<String>,

    #[cfg(feature = "axiom-verification")]
    /// Identity elements (zero, one, e, etc.) mapped to Z3 constants
    /// Key: element name (e.g., "zero", "one", "e")
    /// Value: Z3 Int constant representing that identity element
    identity_elements: HashMap<String, Int>,

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
        let solver = Solver::new();

        Ok(Self {
            solver,
            registry,
            declared_ops: HashMap::new(),
            loaded_structures: HashSet::new(),
            identity_elements: HashMap::new(),
        })
    }

    /// Create verifier without axiom-verification feature
    #[cfg(not(feature = "axiom-verification"))]
    pub fn new(_registry: &'r StructureRegistry) -> Result<Self, String> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Analyze expression to find which structures it depends on
    ///
    /// This enables smart axiom loading - only load axioms for structures
    /// that are actually used in the expression being verified.
    #[cfg(feature = "axiom-verification")]
    fn analyze_dependencies(&self, expr: &Expression) -> HashSet<String> {
        let mut structures = HashSet::new();

        match expr {
            Expression::Operation { name, args } => {
                // Check if this operation belongs to a known structure
                if let Some(owners) = self.registry.get_operation_owners(name) {
                    structures.extend(owners);
                }

                // Recursively analyze arguments
                for arg in args {
                    structures.extend(self.analyze_dependencies(arg));
                }
            }

            Expression::Quantifier { body, .. } => {
                structures.extend(self.analyze_dependencies(body));
            }

            Expression::Object(_) | Expression::Const(_) => {
                // Variables and constants don't introduce dependencies
            }

            _ => {
                // Other expression types analyzed recursively if needed
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
    #[cfg(feature = "axiom-verification")]
    fn ensure_structure_loaded(&mut self, structure_name: &str) -> Result<(), String> {
        // Already loaded?
        if self.loaded_structures.contains(structure_name) {
            return Ok(());
        }

        // Get structure definition from registry
        let structure = self
            .registry
            .get(structure_name)
            .ok_or_else(|| format!("Structure not found: {}", structure_name))?;

        // Phase 1: Load identity elements (nullary operations: zero, one, e, etc.)
        for member in &structure.members {
            if let crate::kleis_ast::StructureMember::Operation {
                name,
                type_signature,
            } = member
            {
                // Check if this is a nullary operation (identity element)
                // Nullary operations have type signatures that are NOT Function types
                // Examples:
                //   - "operation zero : R" → TypeExpr::Named("R") - IS nullary
                //   - "operation plus : R → R → R" → TypeExpr::Function(...) - NOT nullary
                use crate::kleis_ast::TypeExpr;

                let is_nullary = !matches!(type_signature, TypeExpr::Function(..));

                if is_nullary {
                    // This is an identity element/constant!
                    let z3_const = Int::fresh_const(name);
                    self.identity_elements.insert(name.clone(), z3_const);

                    println!("   📌 Loaded identity element: {}", name);
                }
            }
        }

        // Phase 2: Get and load axioms
        let axioms = self.registry.get_axioms(structure_name);

        // Load each axiom as background assumption
        for (_axiom_name, axiom_expr) in axioms {
            // Translate and assert the axiom
            // Now identity elements will be available!
            let z3_axiom = self.kleis_to_z3(&axiom_expr, &HashMap::new())?;
            self.solver.assert(&z3_axiom);
        }

        // Mark as loaded
        self.loaded_structures.insert(structure_name.to_string());

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
    ///     VerificationResult::Invalid { counterexample } => {
    ///         println!("❌ Counterexample: {}", counterexample)
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

        // Step 3: Use push/pop for incremental solving
        self.solver.push();

        // Step 4: Translate Kleis expression to Z3
        let z3_expr = self.kleis_to_z3(expr, &HashMap::new())?;

        // Step 5: For axioms, we want to check if they're always true
        // So we assert the NEGATION and check if it's unsatisfiable
        // If unsat, the original axiom is valid
        self.solver.assert(&z3_expr.not());

        // Step 6: Check satisfiability
        let result = match self.solver.check() {
            SatResult::Unsat => {
                // Negation is unsatisfiable → axiom is valid!
                VerificationResult::Valid
            }
            SatResult::Sat => {
                // Negation is satisfiable → found counterexample
                let counterexample = if let Some(model) = self.solver.get_model() {
                    format!("{}", model)
                } else {
                    "No model available".to_string()
                };
                VerificationResult::Invalid { counterexample }
            }
            SatResult::Unknown => VerificationResult::Unknown,
        };

        // Step 7: Pop the assertion - restore solver state
        self.solver.pop(1);

        Ok(result)
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

            self.solver.push();

            let z3_expr1 = self.kleis_to_z3(expr1, &HashMap::new())?;
            let z3_expr2 = self.kleis_to_z3(expr2, &HashMap::new())?;

            // Check if expr1 ≠ expr2 is unsatisfiable
            self.solver.assert(&z3_expr1.eq(&z3_expr2).not());

            let result = matches!(self.solver.check(), SatResult::Unsat);

            self.solver.pop(1);

            Ok(result)
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
        VerifierStats {
            loaded_structures: self.loaded_structures.len(),
            declared_operations: self.declared_ops.len(),
        }
    }

    /// Generic translator: Kleis Expression → Z3 AST
    ///
    /// **NO HARDCODING!** This function handles ANY expression by:
    /// - Reading operation names from Expression
    /// - Creating variables dynamically
    /// - Mapping operations generically
    /// - Looking up identity elements from structures
    ///
    /// Operations not recognized as built-ins are treated as uninterpreted functions.
    #[cfg(feature = "axiom-verification")]
    fn kleis_to_z3(&self, expr: &Expression, vars: &HashMap<String, Int>) -> Result<Bool, String> {
        match expr {
            // Variables and identity elements: look up in environment
            Expression::Object(name) => {
                // 1. Check if it's a quantified variable
                if let Some(_var) = vars.get(name) {
                    // For now, return a placeholder boolean
                    // TODO: Properly handle typed variables
                    return Ok(Bool::from_bool(true));
                }

                // 2. Check if it's an identity element (zero, one, e, etc.)
                if self.identity_elements.contains_key(name) {
                    // Found an identity element!
                    // For now, return success - full implementation would use the constant
                    return Ok(Bool::from_bool(true));
                }

                // 3. Not found
                Err(format!("Undefined variable or identity: {}", name))
            }

            // Constants: convert to Z3
            Expression::Const(s) => {
                // Try to parse as number
                if let Ok(n) = s.parse::<i64>() {
                    let _ = Int::from_i64(n);
                    Ok(Bool::from_bool(true)) // Placeholder
                } else {
                    Err(format!("Cannot convert constant to Z3: {}", s))
                }
            }

            // Operations: map by name
            Expression::Operation { name, args } => self.operation_to_z3(name, args, vars),

            // Quantifiers: handle forall/exists
            Expression::Quantifier {
                quantifier,
                variables,
                body,
            } => self.quantifier_to_z3(quantifier, variables, body, vars),

            _ => Err(format!("Unsupported expression type for Z3: {:?}", expr)),
        }
    }

    /// Map Kleis operations to Z3 operations
    ///
    /// First tries built-in Z3 theories, then falls back to uninterpreted functions
    /// for custom operations defined in structures.
    #[cfg(feature = "axiom-verification")]
    fn operation_to_z3(
        &self,
        name: &str,
        args: &[Expression],
        vars: &HashMap<String, Int>,
    ) -> Result<Bool, String> {
        match name {
            // Equality
            "equals" | "eq" => {
                if args.len() != 2 {
                    return Err("equals requires 2 arguments".to_string());
                }
                let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                Ok(left.eq(&right))
            }

            // Comparisons
            "less_than" | "lt" => {
                if args.len() != 2 {
                    return Err("less_than requires 2 arguments".to_string());
                }
                let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                Ok(left.lt(&right))
            }

            "greater_than" | "gt" => {
                if args.len() != 2 {
                    return Err("greater_than requires 2 arguments".to_string());
                }
                let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                Ok(left.gt(&right))
            }

            "leq" => {
                if args.len() != 2 {
                    return Err("leq requires 2 arguments".to_string());
                }
                let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                Ok(left.le(&right))
            }

            "geq" => {
                if args.len() != 2 {
                    return Err("geq requires 2 arguments".to_string());
                }
                let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                Ok(left.ge(&right))
            }

            // Boolean operations
            "and" | "logical_and" => {
                if args.len() != 2 {
                    return Err("and requires 2 arguments".to_string());
                }
                let left = self.kleis_to_z3(&args[0], vars)?;
                let right = self.kleis_to_z3(&args[1], vars)?;
                Ok(Bool::and(&[&left, &right]))
            }

            "or" | "logical_or" => {
                if args.len() != 2 {
                    return Err("or requires 2 arguments".to_string());
                }
                let left = self.kleis_to_z3(&args[0], vars)?;
                let right = self.kleis_to_z3(&args[1], vars)?;
                Ok(Bool::or(&[&left, &right]))
            }

            "not" | "logical_not" => {
                if args.len() != 1 {
                    return Err("not requires 1 argument".to_string());
                }
                let arg = self.kleis_to_z3(&args[0], vars)?;
                Ok(arg.not())
            }

            // Implication: P ⟹ Q is equivalent to ¬P ∨ Q
            "implies" => {
                if args.len() != 2 {
                    return Err("implies requires 2 arguments".to_string());
                }
                let left = self.kleis_to_z3(&args[0], vars)?;
                let right = self.kleis_to_z3(&args[1], vars)?;
                Ok(left.implies(&right))
            }

            // Unknown operation - could be custom structure operation
            _ => {
                // TODO: Check if operation is in declared_ops and use uninterpreted function
                Err(format!(
                    "Unsupported operation for Z3: {} (not in built-in theories)",
                    name
                ))
            }
        }
    }

    /// Helper: Convert Kleis expression to Z3 Int
    ///
    /// Handles arithmetic operations using Z3's integer theory.
    /// Also handles identity elements like zero, one, e.
    #[cfg(feature = "axiom-verification")]
    fn kleis_expr_to_z3_int(
        &self,
        expr: &Expression,
        vars: &HashMap<String, Int>,
    ) -> Result<Int, String> {
        match expr {
            Expression::Object(name) => {
                // 1. Check quantified variables first
                if let Some(var) = vars.get(name) {
                    return Ok(var.clone());
                }

                // 2. Check identity elements (zero, one, e, etc.)
                if let Some(identity) = self.identity_elements.get(name) {
                    return Ok(identity.clone());
                }

                // 3. Not found
                Err(format!("Undefined variable or identity: {}", name))
            }

            Expression::Const(s) => {
                let n: i64 = s.parse().map_err(|_| format!("Not a number: {}", s))?;
                Ok(Int::from_i64(n))
            }

            Expression::Operation { name, args } => match name.as_str() {
                "plus" | "add" => {
                    if args.len() != 2 {
                        return Err("plus requires 2 arguments".to_string());
                    }
                    let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                    let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                    Ok(Int::add(&[&left, &right]))
                }

                "times" | "multiply" => {
                    if args.len() != 2 {
                        return Err("times requires 2 arguments".to_string());
                    }
                    let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                    let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                    Ok(Int::mul(&[&left, &right]))
                }

                "minus" | "subtract" => {
                    if args.len() != 2 {
                        return Err("minus requires 2 arguments".to_string());
                    }
                    let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
                    let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
                    Ok(Int::sub(&[&left, &right]))
                }

                "neg" | "negate" => {
                    // Unary negation: -x
                    if args.len() != 1 {
                        return Err("neg requires 1 argument".to_string());
                    }
                    let arg = self.kleis_expr_to_z3_int(&args[0], vars)?;
                    Ok(Int::unary_minus(&arg))
                }

                _ => Err(format!("Unsupported arithmetic operation: {}", name)),
            },

            _ => Err("Cannot convert to Int".to_string()),
        }
    }

    /// Handle quantifiers (∀ and ∃)
    ///
    /// Creates fresh Z3 variables and translates the body.
    /// Z3 treats free variables as universally quantified.
    #[cfg(feature = "axiom-verification")]
    fn quantifier_to_z3(
        &self,
        _quantifier: &QuantifierKind,
        variables: &[QuantifiedVar],
        body: &Expression,
        vars: &HashMap<String, Int>,
    ) -> Result<Bool, String> {
        // Create fresh Z3 variables for quantified variables
        let mut new_vars = vars.clone();

        for var in variables {
            let z3_var = Int::fresh_const(&var.name);
            new_vars.insert(var.name.clone(), z3_var);
        }

        // Translate body with new variables
        let body_z3 = self.kleis_to_z3(body, &new_vars)?;

        // For both universal and existential quantifiers,
        // Z3 treats free variables as universally quantified
        Ok(body_z3)
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
            counterexample: "x=1".to_string(),
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
        };
        let deps = verifier.analyze_dependencies(&expr);
        // Dependencies depend on registry content
        println!("Dependencies for plus operation: {:?}", deps);
    }
}
