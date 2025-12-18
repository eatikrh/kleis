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
    /// Solver backend (Z3, CVC5, etc.) - handles low-level Z3 operations
    backend: Z3Backend<'r>,

    #[cfg(feature = "axiom-verification")]
    /// Structure registry - source of all Kleis structure definitions (CRITICAL!)
    registry: &'r StructureRegistry,

    #[cfg(feature = "axiom-verification")]
    /// Track which structures' axioms are currently loaded
    loaded_structures: HashSet<String>,

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
        let backend = Z3Backend::new(registry)?;

        Ok(Self {
            backend,
            registry,
            loaded_structures: HashSet::new(),
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
                self.backend.declare_and_define_function(
                    &func_def.name,
                    &func_def.params,
                    &func_def.body,
                )?;
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
    #[cfg(feature = "axiom-verification")]
    pub fn load_adt_constructors<I, S>(&mut self, constructors: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for name in constructors {
            self.backend.load_identity_element(name.as_ref());
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
            } => {
                // Analyze all three parts of the conditional
                structures.extend(self.analyze_dependencies(condition));
                structures.extend(self.analyze_dependencies(then_branch));
                structures.extend(self.analyze_dependencies(else_branch));
            }

            Expression::Match { scrutinee, cases } => {
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
        println!("   Loading axioms for {}...", structure_name);
        if let Err(e) = self.load_axioms_recursive(&structure.members) {
            eprintln!("   ❌ ERROR loading axioms: {}", e);
            return Err(e);
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
                        // Delegate to backend to load identity element
                        self.backend.load_identity_element(name);
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
                    self.backend.assert_kleis_expression(proposition)?;
                }
                StructureMember::FunctionDef(func_def) => {
                    // Grammar v0.6: Load function definition via backend
                    self.backend.declare_and_define_function(
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
        eprintln!("DEBUG: Found dependencies: {:?}", dependencies);

        // Step 2: Ensure all required axioms are loaded
        for structure in &dependencies {
            eprintln!("DEBUG: Loading structure: {}", structure);
            self.ensure_structure_loaded(structure)?;
        }

        // Step 3: Delegate to backend for verification (uses solver abstraction layer!)
        use crate::solvers::backend::VerificationResult as BackendResult;

        let backend_result = self.backend.verify_axiom(expr)?;

        // Convert backend result to AxiomVerifier result
        Ok(match backend_result {
            BackendResult::Valid => VerificationResult::Valid,
            BackendResult::Invalid { counterexample } => {
                VerificationResult::Invalid { counterexample }
            }
            BackendResult::Unknown => VerificationResult::Unknown,
        })
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
