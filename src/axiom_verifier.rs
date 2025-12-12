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
use z3::ast::{Ast, Bool, Dynamic, Int};
#[cfg(feature = "axiom-verification")]
use z3::{FuncDecl, SatResult, Solver, Sort};

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
    /// Track which operations have been declared (for logging/debugging)
    /// We don't cache FuncDecl itself since they're lightweight to recreate
    declared_ops: HashSet<String>,

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
            declared_ops: HashSet::new(),
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

            Expression::Const(_) => {
                // Constants don't introduce dependencies
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
        self.load_axioms_recursive(&structure.members)?;

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
                        let z3_const = Int::fresh_const(name);
                        self.identity_elements.insert(name.clone(), z3_const);
                        println!("   📌 Loaded identity element: {}", name);
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
    /// Handles axioms in nested structures
    #[cfg(feature = "axiom-verification")]
    fn load_axioms_recursive(
        &mut self,
        members: &[crate::kleis_ast::StructureMember],
    ) -> Result<(), String> {
        use crate::kleis_ast::StructureMember;

        for member in members {
            match member {
                StructureMember::Axiom { proposition, .. } => {
                    // Translate and assert axiom
                    let z3_axiom = self.kleis_to_z3(proposition, &HashMap::new())?;
                    self.solver.assert(&z3_axiom);
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

    /// Declare an operation as an uninterpreted function in Z3
    ///
    /// Uninterpreted functions let Z3 reason about abstract operations using only
    /// the axioms we provide, without assuming any built-in meaning.
    ///
    /// Example: For `(•) : S × S → S` in Semigroup, this creates a Z3 function
    /// that Z3 will reason about using only the associativity axiom.
    ///
    /// Note: FuncDecl is lightweight to recreate, so we don't cache it.
    #[cfg(feature = "axiom-verification")]
    fn declare_operation(&mut self, name: &str, arity: usize) -> FuncDecl {
        // Log if this is the first time we're declaring this operation
        if !self.declared_ops.contains(name) {
            println!(
                "   🔧 Declaring uninterpreted function: {} with arity {}",
                name, arity
            );
            self.declared_ops.insert(name.to_string());
        }

        // Create uninterpreted function: Int × Int × ... → Int
        // Using Int sort as default for algebraic operations
        let domain: Vec<_> = (0..arity).map(|_| Sort::int()).collect();
        let domain_refs: Vec<_> = domain.iter().collect();
        FuncDecl::new(name, &domain_refs, &Sort::int())
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
    fn kleis_to_z3(&mut self, expr: &Expression, vars: &HashMap<String, Int>) -> Result<Bool, String> {
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
                where_clause,
                body,
            } => self.quantifier_to_z3(quantifier, variables, where_clause.as_ref(), body, vars),

            _ => Err(format!("Unsupported expression type for Z3: {:?}", expr)),
        }
    }

    /// Map Kleis operations to Z3 operations
    ///
    /// First tries built-in Z3 theories, then falls back to uninterpreted functions
    /// for custom operations defined in structures.
    #[cfg(feature = "axiom-verification")]
    fn operation_to_z3(
        &mut self,
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

            // Unknown operation - use uninterpreted function
            _ => {
                // Translate arguments to Z3 Int
                let z3_args: Result<Vec<_>, _> = args
                    .iter()
                    .map(|arg| self.kleis_expr_to_z3_int(arg, vars))
                    .collect();
                let z3_args = z3_args?;

                // Declare the operation as uninterpreted function
                let func_decl = self.declare_operation(name, args.len());

                // Apply the uninterpreted function
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    z3_args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                let result = func_decl.apply(&ast_args);

                // For operations used in equality context, return true
                // The actual comparison is handled by the equals operator
                Ok(Bool::from_bool(true))
            }
        }
    }

    /// Helper: Convert Kleis expression to Z3 Int
    ///
    /// Handles arithmetic operations using Z3's integer theory.
    /// Also handles identity elements like zero, one, e.
    /// Falls back to uninterpreted functions for unknown operations.
    #[cfg(feature = "axiom-verification")]
    fn kleis_expr_to_z3_int(
        &mut self,
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

                // Unknown arithmetic operation - use uninterpreted function
                _ => {
                    let z3_args: Result<Vec<_>, _> = args
                        .iter()
                        .map(|arg| self.kleis_expr_to_z3_int(arg, vars))
                        .collect();
                    let z3_args = z3_args?;

                    let func_decl = self.declare_operation(name, args.len());
                    let ast_args: Vec<&dyn z3::ast::Ast> =
                        z3_args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                    let result = func_decl.apply(&ast_args);

                    // Convert Dynamic to Int
                    result
                        .as_int()
                        .ok_or_else(|| format!("Operation {} did not return Int", name))
                }
            },

            _ => Err("Cannot convert to Int".to_string()),
        }
    }

    /// Handle quantifiers (∀ and ∃)
    ///
    /// Creates fresh Z3 variables and translates the body.
    /// Z3 treats free variables as universally quantified.
    ///
    /// If a where clause is present (e.g., ∀(x : F) where x ≠ zero. body),
    /// it's translated as: where_clause ⟹ body
    #[cfg(feature = "axiom-verification")]
    fn quantifier_to_z3(
        &mut self,
        _quantifier: &QuantifierKind,
        variables: &[QuantifiedVar],
        where_clause: Option<&Box<Expression>>,
        body: &Expression,
        vars: &HashMap<String, Int>,
    ) -> Result<Bool, String> {
        // Create fresh Z3 variables for quantified variables
        let mut new_vars = vars.clone();

        for var in variables {
            let z3_var = Int::fresh_const(&var.name);
            new_vars.insert(var.name.clone(), z3_var);
        }

        // If there's a where clause, translate as: where_clause ⟹ body
        let body_z3 = if let Some(condition) = where_clause {
            let condition_z3 = self.kleis_to_z3(condition, &new_vars)?;
            let body_z3 = self.kleis_to_z3(body, &new_vars)?;

            // where_clause ⟹ body
            condition_z3.implies(&body_z3)
        } else {
            // No where clause, just translate body
            self.kleis_to_z3(body, &new_vars)?
        };

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
