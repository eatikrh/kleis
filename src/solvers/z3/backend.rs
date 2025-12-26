//! Z3 Backend Implementation
//!
//! Implements the SolverBackend trait for Z3 SMT solver.
//!
//! This is extracted and refactored from axiom_verifier.rs to fit the new
//! pluggable solver architecture.
//!
//! **Key Features:**
//! - Incremental solving (push/pop for efficiency)
//! - Smart axiom loading (on-demand, with dependency analysis)
//! - Mixed type handling (Int/Real conversions)
//! - Uninterpreted functions for unknown operations
//!
//! **Critical:** All public methods return Kleis Expression, not Z3 types!

use crate::ast::{Expression, QuantifiedVar, QuantifierKind};
use crate::evaluator::Evaluator;
use crate::solvers::backend::{
    SatisfiabilityResult, SolverBackend, SolverStats, VerificationResult,
};
use crate::solvers::capabilities::SolverCapabilities;
use crate::solvers::result_converter::ResultConverter;
use crate::solvers::z3::converter::Z3ResultConverter;
use crate::solvers::z3::translators::{arithmetic, boolean, comparison};
use crate::structure_registry::StructureRegistry;
use std::collections::{HashMap, HashSet};
use z3::ast::{Ast, Bool, Dynamic, Int, Real};
use z3::{DatatypeAccessor, DatatypeBuilder, DatatypeSort, FuncDecl, SatResult, Solver, Sort};

/// Z3 SMT Solver Backend
///
/// Wraps Z3's SMT solver to implement the SolverBackend trait.
/// Maintains long-lived solver state and loads axioms on-demand.
pub struct Z3Backend<'r> {
    /// Z3 solver instance (long-lived for incremental solving)
    solver: Solver,

    /// Structure registry (source of axioms and operations)
    /// Currently passed through from AxiomVerifier, will be used for
    /// advanced features (coverage analysis, operation lookup, etc.)
    #[allow(dead_code)]
    registry: &'r StructureRegistry,

    /// Capability manifest (loaded from capabilities.toml)
    capabilities: SolverCapabilities,

    /// Track which operations have been declared as uninterpreted functions
    declared_ops: HashSet<String>,

    /// Track which structures' axioms are currently loaded
    loaded_structures: HashSet<String>,

    /// Identity elements (zero, one, e, etc.) mapped to Z3 constants
    identity_elements: HashMap<String, Dynamic>,

    /// Free variables auto-created from undefined Object names
    free_variables: HashMap<String, Dynamic>,

    /// Result converter (Z3 Dynamic → Kleis Expression)
    converter: Z3ResultConverter,

    /// Complex number datatype for hybrid translation
    /// Enables concrete complex arithmetic: complex(1,2) + complex(3,4) = complex(4,6)
    complex_datatype: Option<ComplexDatatype>,
}

/// Complex number Z3 datatype
/// Stores the DatatypeSort which contains constructor and accessors
struct ComplexDatatype {
    /// The Complex sort (contains constructor and accessors)
    sort: DatatypeSort,
}

impl ComplexDatatype {
    /// Get the constructor: mk_complex(re: Real, im: Real) -> Complex
    fn constructor(&self) -> &FuncDecl {
        &self.sort.variants[0].constructor
    }

    /// Get the real part accessor
    fn accessor_re(&self) -> &FuncDecl {
        &self.sort.variants[0].accessors[0]
    }

    /// Get the imaginary part accessor
    fn accessor_im(&self) -> &FuncDecl {
        &self.sort.variants[0].accessors[1]
    }

    /// Get the Z3 Sort for Complex numbers
    #[allow(dead_code)]
    fn sort(&self) -> &Sort {
        &self.sort.sort
    }
}

impl<'r> Z3Backend<'r> {
    /// Create a new Z3 backend
    ///
    /// # Arguments
    /// * `registry` - Structure registry containing operations and axioms
    ///
    /// # Axiom Loading
    /// Axioms are loaded from stdlib/*.kleis files via assert_axioms_from_registry().
    /// Call this method after creating the backend to load all axioms before verification.
    pub fn new(registry: &'r StructureRegistry) -> Result<Self, String> {
        let solver = Solver::new();
        let capabilities = super::load_capabilities()?;

        // Create Complex number datatype: Complex = mk_complex(re: Real, im: Real)
        let complex_dt = DatatypeBuilder::new("Complex")
            .variant(
                "mk_complex",
                vec![
                    ("re", DatatypeAccessor::sort(Sort::real())),
                    ("im", DatatypeAccessor::sort(Sort::real())),
                ],
            )
            .finish();

        let complex_datatype = ComplexDatatype { sort: complex_dt };

        let mut backend = Self {
            solver,
            registry,
            capabilities,
            declared_ops: HashSet::new(),
            loaded_structures: HashSet::new(),
            identity_elements: HashMap::new(),
            free_variables: HashMap::new(),
            converter: Z3ResultConverter,
            complex_datatype: Some(complex_datatype),
        };

        // Initialize complex number constant 'i' as complex(0, 1)
        // This is now a concrete value, not an uninterpreted constant!
        backend.initialize_complex_i();

        Ok(backend)
    }

    /// Assert all axioms from the registry into Z3 solver
    ///
    /// This is the key method for making user-defined axioms available to Z3.
    /// Axioms are translated to Z3 assertions so they can be used for verification.
    ///
    /// # Example
    /// ```ignore
    /// let mut backend = Z3Backend::new(&registry)?;
    /// backend.assert_axioms_from_registry()?;  // Load all axioms
    /// backend.verify_axiom(&theorem)?;          // Now uses loaded axioms
    /// ```
    ///
    /// # Returns
    /// - Ok(count) - number of axioms successfully loaded
    /// - Err if any axiom fails to translate
    pub fn assert_axioms_from_registry(&mut self) -> Result<usize, String> {
        let mut count = 0;
        let empty_vars: HashMap<String, Dynamic> = HashMap::new();

        // Get all structures that have axioms
        let structures_with_axioms: Vec<String> = self
            .registry
            .structures_with_axioms()
            .iter()
            .map(|s| (*s).clone())
            .collect();

        for structure_name in structures_with_axioms {
            // Skip if already loaded
            if self.loaded_structures.contains(&structure_name) {
                continue;
            }

            let axioms = self.registry.get_axioms(&structure_name);

            for (axiom_name, axiom_expr) in axioms {
                match self.translate_and_assert_axiom(&axiom_name, axiom_expr, &empty_vars) {
                    Ok(()) => {
                        count += 1;
                        // Successfully asserted axiom
                    }
                    Err(_e) => {
                        // Continue with other axioms rather than failing entirely
                        // Axioms may fail if they use unsupported constructs
                    }
                }
            }

            self.loaded_structures.insert(structure_name);
        }

        Ok(count)
    }

    // =========================================================================
    // Beta Reduction Integration
    // =========================================================================

    /// Pre-reduce an expression using beta reduction before Z3 translation
    ///
    /// This applies beta reduction to any lambda applications in the expression,
    /// converting `(λ x . x + 1)(5)` to `5 + 1` before Z3 sees it.
    ///
    /// # Why This Matters
    /// Z3 can't directly apply lambda expressions. By reducing them first,
    /// we convert lambda applications into simpler expressions Z3 can verify.
    ///
    /// # Example
    /// ```ignore
    /// let expr = parse_expression("(λ x . x + 1)(5) = 6")?;
    /// let reduced = backend.beta_reduce_expression(&expr)?;
    /// // reduced = "5 + 1 = 6"
    /// backend.check_satisfiability(&reduced)?;
    /// ```
    pub fn beta_reduce_expression(&self, expr: &Expression) -> Result<Expression, String> {
        let evaluator = Evaluator::new();
        evaluator.reduce_to_normal_form(expr)
    }

    /// Check satisfiability with automatic beta reduction
    ///
    /// This is like `check_satisfiability` but first reduces any lambda expressions.
    pub fn check_satisfiability_with_reduction(
        &mut self,
        expr: &Expression,
    ) -> Result<SatisfiabilityResult, String> {
        let reduced = self.beta_reduce_expression(expr)?;
        self.check_satisfiability(&reduced)
    }

    /// Translate a single axiom and assert it into Z3
    fn translate_and_assert_axiom(
        &mut self,
        name: &str,
        expr: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<(), String> {
        // Handle quantified axioms (∀ x : T . body)
        if let Expression::Quantifier {
            quantifier,
            variables,
            where_clause,
            body,
            ..
        } = expr
        {
            // Create Z3 forall
            let z3_axiom =
                self.translate_quantifier_as_forall(quantifier, variables, where_clause, body)?;
            self.solver.assert(&z3_axiom);
            return Ok(());
        }

        // Non-quantified axiom: translate directly
        let z3_expr = self.kleis_to_z3(expr, vars)?;

        // Must be boolean
        let z3_bool = z3_expr
            .as_bool()
            .ok_or_else(|| format!("Axiom '{}' must be a boolean expression", name))?;

        self.solver.assert(&z3_bool);
        Ok(())
    }

    /// Translate a quantified expression to a proper Z3 forall
    ///
    /// This creates an actual Z3 forall constraint, not just the body.
    fn translate_quantifier_as_forall(
        &mut self,
        quantifier: &QuantifierKind,
        variables: &[QuantifiedVar],
        where_clause: &Option<Box<Expression>>,
        body: &Expression,
    ) -> Result<Bool, String> {
        // Create Z3 bound variables
        let mut bound_vars: Vec<Dynamic> = Vec::new();
        let mut var_map: HashMap<String, Dynamic> = HashMap::new();

        for var in variables {
            let z3_var: Dynamic = if let Some(type_annotation) = &var.type_annotation {
                match type_annotation.as_str() {
                    "Bool" | "Boolean" => Bool::fresh_const(&var.name).into(),
                    "ℝ" | "Real" | "R" => Real::fresh_const(&var.name).into(),
                    "ℤ" | "Int" | "Z" | "Nat" => Int::fresh_const(&var.name).into(),
                    "ℂ" | "Complex" | "C" => {
                        // Create fresh Complex constant for quantified complex variables
                        self.fresh_complex_const(&var.name)
                            .unwrap_or_else(|| Int::fresh_const(&var.name).into())
                    }
                    _ => Int::fresh_const(&var.name).into(),
                }
            } else {
                Int::fresh_const(&var.name).into()
            };
            bound_vars.push(z3_var.clone());
            var_map.insert(var.name.clone(), z3_var);
        }

        // Translate body
        let body_z3 = self.kleis_to_z3(body, &var_map)?;
        let body_bool = body_z3
            .as_bool()
            .ok_or_else(|| "Quantifier body must be boolean".to_string())?;

        // Handle where clause: where_clause ⟹ body
        let formula = if let Some(condition) = where_clause {
            let condition_z3 = self.kleis_to_z3(condition, &var_map)?;
            let condition_bool = condition_z3
                .as_bool()
                .ok_or_else(|| "Where clause must be boolean".to_string())?;
            condition_bool.implies(&body_bool)
        } else {
            body_bool
        };

        // Create Z3 forall/exists
        let bound_refs: Vec<&dyn Ast> = bound_vars.iter().map(|v| v as &dyn Ast).collect();

        let result = match quantifier {
            QuantifierKind::ForAll => z3::ast::forall_const(&bound_refs, &[], &formula),
            QuantifierKind::Exists => z3::ast::exists_const(&bound_refs, &[], &formula),
        };

        // Convert back to Bool (forall_const returns Bool)
        Ok(result)
    }

    /// Translate a Kleis List to a cons-chain
    ///
    /// [a, b, c] -> cons(a, cons(b, cons(c, nil)))
    ///
    /// This enables axioms from stdlib/lists.kleis to work:
    /// - nth(cons(x, xs), 0) = x
    /// - nth(cons(x, xs), n+1) = nth(xs, n)
    fn translate_list_to_cons(
        &mut self,
        items: &[Expression],
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Dynamic, String> {
        // nil is represented as an uninterpreted constant
        let nil_func = self.declare_uninterpreted("nil", 0);
        let mut result = nil_func.apply(&[]);

        // Build cons chain from right to left
        for item in items.iter().rev() {
            let item_z3 = self.kleis_to_z3(item, vars)?;
            let cons_func = self.declare_uninterpreted("cons", 2);
            result = cons_func.apply(&[&item_z3 as &dyn Ast, &result as &dyn Ast]);
        }

        Ok(result)
    }

    /// Translate Kleis expression to Z3 Dynamic
    ///
    /// This is the core translation function. It recursively converts
    /// Kleis expressions to Z3's internal representation.
    ///
    /// **Internal only** - results stay within Z3Backend.
    fn kleis_to_z3(
        &mut self,
        expr: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Dynamic, String> {
        match expr {
            Expression::Object(name) => {
                // 1. Check quantified variables (highest priority)
                if let Some(var) = vars.get(name) {
                    return Ok(var.clone());
                }

                // 2. Check identity elements
                if let Some(identity) = self.identity_elements.get(name) {
                    return Ok(identity.clone());
                }

                // 3. Special case: 'i' as the complex imaginary unit
                // Only use complex i if NOT already in free_variables (which means
                // it was used as a loop variable first)
                if name == "i" && !self.free_variables.contains_key("i") {
                    if let Some(i_value) = self.get_complex_i() {
                        return Ok(i_value);
                    }
                }

                // 4. Check already-created free variables
                if let Some(free_var) = self.free_variables.get(name) {
                    return Ok(free_var.clone());
                }

                // 5. Create fresh constant for this free variable
                // This allows equations like "A = Matrix(...)" to be verified
                let fresh = Int::fresh_const(name);
                let dynamic: Dynamic = fresh.into();
                self.free_variables.insert(name.clone(), dynamic.clone());
                Ok(dynamic)
            }

            Expression::Const(s) => {
                // Try to parse as number
                if let Ok(n) = s.parse::<i64>() {
                    Ok(Int::from_i64(n).into())
                } else {
                    Err(format!("Cannot convert constant to Z3: {}", s))
                }
            }

            Expression::String(s) => {
                // String literals are converted to Z3 String sort
                // Note: Z3's String sort requires z3::ast::String which can represent string constants
                Ok(z3::ast::String::from(s.clone()).into())
            }

            Expression::Operation { name, args, .. } => {
                // Matrix and tensor operations are handled via axioms from stdlib/*.kleis
                // Use assert_axioms_from_registry() to load them before verification

                // Standard path: translate arguments first
                let z3_args: Result<Vec<_>, _> =
                    args.iter().map(|arg| self.kleis_to_z3(arg, vars)).collect();
                let z3_args = z3_args?;

                // Use modular translators
                self.translate_operation(name, &z3_args)
            }

            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
                ..
            } => {
                let bool_result = self.translate_quantifier(
                    quantifier,
                    variables,
                    where_clause.as_ref().map(|b| &**b),
                    body,
                    vars,
                )?;
                Ok(bool_result.into())
            }

            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Translate all three parts
                let cond_z3 = self.kleis_to_z3(condition, vars)?;
                let then_z3 = self.kleis_to_z3(then_branch, vars)?;
                let else_z3 = self.kleis_to_z3(else_branch, vars)?;

                // Convert condition to Bool
                let cond_bool = cond_z3.as_bool().ok_or_else(|| {
                    "Conditional condition must be a boolean expression".to_string()
                })?;

                // Use Z3's ite (if-then-else)
                Ok(boolean::translate_ite(&cond_bool, &then_z3, &else_z3))
            }

            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                // 1. Translate the value expression
                let z3_value = self.kleis_to_z3(value, vars)?;

                // 2. Extend vars with bindings from pattern match
                // Grammar v0.8: Support pattern destructuring
                let mut extended_vars = vars.clone();
                self.bind_pattern_to_z3(pattern, &z3_value, value, &mut extended_vars)?;

                // 3. Translate body with the extended context
                self.kleis_to_z3(body, &extended_vars)
            }

            Expression::Match {
                scrutinee, cases, ..
            } => {
                // Translate match expression to nested ite
                self.translate_match(scrutinee, cases, vars)
            }

            Expression::List(items) => {
                // Convert list to cons-chain: [a, b, c] -> cons(a, cons(b, cons(c, nil)))
                // This allows axioms from stdlib/lists.kleis to work
                self.translate_list_to_cons(items, vars)
            }

            Expression::Ascription { expr, .. } => {
                // Type annotations don't affect Z3 semantics - just translate the inner expression
                self.kleis_to_z3(expr, vars)
            }

            Expression::Lambda { params, body, .. } => {
                // Lambda expressions in Z3 context:
                // Translate the lambda body with parameters bound as fresh Int variables.
                // This allows Z3 to reason about the body symbolically.
                //
                // NOTE: For lambda applications like (λ x . x + 1)(5), use
                // check_satisfiability_with_reduction() which performs beta reduction
                // before Z3 translation, converting it to 5 + 1.
                let mut new_vars = vars.clone();
                for param in params {
                    // Create fresh variable for each lambda parameter
                    // Use type annotation if available, default to Int
                    let z3_var: Dynamic = if let Some(ref ty) = param.type_annotation {
                        match ty.as_str() {
                            "Bool" | "Boolean" => Bool::fresh_const(&param.name).into(),
                            "ℝ" | "Real" | "R" => Real::fresh_const(&param.name).into(),
                            "ℂ" | "Complex" | "C" => self
                                .fresh_complex_const(&param.name)
                                .unwrap_or_else(|| Int::fresh_const(&param.name).into()),
                            _ => Int::fresh_const(&param.name).into(),
                        }
                    } else {
                        Int::fresh_const(&param.name).into()
                    };
                    new_vars.insert(param.name.clone(), z3_var);
                }
                self.kleis_to_z3(body, &new_vars)
            }

            Expression::Placeholder { .. } => {
                // Placeholders shouldn't reach Z3 - they're for the editor
                Err(
                    "Placeholder expressions cannot be verified - fill in all slots first"
                        .to_string(),
                )
            }
        }
    }

    /// Translate operation using modular translators
    fn translate_operation(&mut self, name: &str, args: &[Dynamic]) -> Result<Dynamic, String> {
        match name {
            // Equality
            "equals" | "eq" => {
                if args.len() != 2 {
                    return Err("equals requires 2 arguments".to_string());
                }
                Ok(comparison::translate_equals(&args[0], &args[1])?.into())
            }

            "neq" | "not_equals" => {
                if args.len() != 2 {
                    return Err("neq requires 2 arguments".to_string());
                }
                Ok(comparison::translate_not_equals(&args[0], &args[1])?.into())
            }

            // Comparisons
            "less_than" | "lt" => {
                if args.len() != 2 {
                    return Err("less_than requires 2 arguments".to_string());
                }
                Ok(comparison::translate_less_than(&args[0], &args[1])?.into())
            }

            "greater_than" | "gt" => {
                if args.len() != 2 {
                    return Err("greater_than requires 2 arguments".to_string());
                }
                Ok(comparison::translate_greater_than(&args[0], &args[1])?.into())
            }

            "leq" => {
                if args.len() != 2 {
                    return Err("leq requires 2 arguments".to_string());
                }
                Ok(comparison::translate_leq(&args[0], &args[1])?.into())
            }

            "geq" => {
                if args.len() != 2 {
                    return Err("geq requires 2 arguments".to_string());
                }
                Ok(comparison::translate_geq(&args[0], &args[1])?.into())
            }

            // Boolean operations
            "and" | "logical_and" => {
                if args.len() != 2 {
                    return Err("and requires 2 arguments".to_string());
                }
                Ok(boolean::translate_and(&args[0], &args[1])?.into())
            }

            "or" | "logical_or" => {
                if args.len() != 2 {
                    return Err("or requires 2 arguments".to_string());
                }
                Ok(boolean::translate_or(&args[0], &args[1])?.into())
            }

            "not" | "logical_not" => {
                if args.len() != 1 {
                    return Err("not requires 1 argument".to_string());
                }
                Ok(boolean::translate_not(&args[0])?.into())
            }

            "implies" => {
                if args.len() != 2 {
                    return Err("implies requires 2 arguments".to_string());
                }
                Ok(boolean::translate_implies(&args[0], &args[1])?.into())
            }

            // Arithmetic operations
            "plus" | "add" => {
                if args.len() != 2 {
                    return Err("plus requires 2 arguments".to_string());
                }
                arithmetic::translate_plus(&args[0], &args[1])
            }

            "minus" | "subtract" => {
                if args.len() != 2 {
                    return Err("minus requires 2 arguments".to_string());
                }
                arithmetic::translate_minus(&args[0], &args[1])
            }

            "times" | "multiply" => {
                if args.len() != 2 {
                    return Err("times requires 2 arguments".to_string());
                }
                arithmetic::translate_times(&args[0], &args[1])
            }

            "power" | "pow" | "^" => {
                if args.len() != 2 {
                    return Err("power requires 2 arguments".to_string());
                }
                arithmetic::translate_power(&args[0], &args[1])
            }

            "sqrt" => {
                if args.len() != 1 {
                    return Err("sqrt requires 1 argument".to_string());
                }
                arithmetic::translate_sqrt(&args[0])
            }

            // Derivative operators (Mathematica-style)
            // D(f, x) - partial derivative ∂f/∂x
            // Dt(f, x) - total derivative df/dx
            "D" | "partial" => {
                // D(f, x) or D(f, x, y) for mixed partials
                if args.is_empty() {
                    return Err("D requires at least 1 argument".to_string());
                }
                // Use uninterpreted function - axioms define behavior
                let func_decl = self.declare_uninterpreted("D", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            "Dt" | "total" => {
                // Dt(f, x) - total derivative
                if args.len() < 2 {
                    return Err("Dt requires at least 2 arguments".to_string());
                }
                // Use uninterpreted function - axioms define behavior
                let func_decl = self.declare_uninterpreted("Dt", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Integral operators (Mathematica-style)
            // Integrate(f, x) - indefinite integral ∫f dx
            // Integrate(f, {x, a, b}) - definite integral ∫[a,b] f dx
            "Integrate" | "integral" => {
                if args.is_empty() {
                    return Err("Integrate requires at least 1 argument".to_string());
                }
                let func_decl = self.declare_uninterpreted("Integrate", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Double integral ∬
            "DoubleIntegral" | "integral2" => {
                if args.is_empty() {
                    return Err("DoubleIntegral requires at least 1 argument".to_string());
                }
                let func_decl = self.declare_uninterpreted("DoubleIntegral", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Triple integral ∭
            "TripleIntegral" | "integral3" => {
                if args.is_empty() {
                    return Err("TripleIntegral requires at least 1 argument".to_string());
                }
                let func_decl = self.declare_uninterpreted("TripleIntegral", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Line integral ∮
            "LineIntegral" | "contour" => {
                if args.is_empty() {
                    return Err("LineIntegral requires at least 1 argument".to_string());
                }
                let func_decl = self.declare_uninterpreted("LineIntegral", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Surface integral ∯
            "SurfaceIntegral" | "surface" => {
                if args.is_empty() {
                    return Err("SurfaceIntegral requires at least 1 argument".to_string());
                }
                let func_decl = self.declare_uninterpreted("SurfaceIntegral", args.len());
                let ast_args: Vec<&dyn z3::ast::Ast> =
                    args.iter().map(|a| a as &dyn z3::ast::Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            "abs" | "absolute" => {
                if args.len() != 1 {
                    return Err("abs requires 1 argument".to_string());
                }
                arithmetic::translate_abs(&args[0])
            }

            "neg" | "negate" => {
                if args.len() != 1 {
                    return Err("negate requires 1 argument".to_string());
                }
                arithmetic::translate_negate(&args[0])
            }

            // Nth root: nth_root(n, x) - uninterpreted for integers
            // (sqrt is already handled above via arithmetic::translate_sqrt)
            "nth_root" => {
                if args.len() != 2 {
                    return Err("nth_root requires 2 arguments (index, radicand)".to_string());
                }
                let func_decl = self.declare_uninterpreted("nth_root", 2);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // ============================================
            // STRING OPERATIONS (Grammar v0.8)
            // These use Z3's native string theory (QF_SLIA)
            // ============================================

            // String concatenation: concat("hello", " world") = "hello world"
            "concat" | "str_concat" | "++" => {
                if args.len() < 2 {
                    return Err("concat requires at least 2 arguments".to_string());
                }
                // Convert all args to Z3 strings and concatenate
                let strings: Result<Vec<z3::ast::String>, String> = args
                    .iter()
                    .map(|a| {
                        a.as_string()
                            .ok_or_else(|| "concat arguments must be strings".to_string())
                    })
                    .collect();
                let strings = strings?;
                // Use Z3's concat (variadic)
                let refs: Vec<&z3::ast::String> = strings.iter().collect();
                Ok(z3::ast::String::concat(&refs).into())
            }

            // String length: strlen("hello") = 5
            "strlen" | "str_len" | "length" => {
                if args.len() != 1 {
                    return Err("strlen requires 1 argument".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "strlen argument must be a string".to_string())?;
                Ok(s.length().into())
            }

            // String contains: contains("hello", "ell") = True
            "contains" | "str_contains" => {
                if args.len() != 2 {
                    return Err("contains requires 2 arguments".to_string());
                }
                let haystack = args[0]
                    .as_string()
                    .ok_or_else(|| "contains first argument must be a string".to_string())?;
                let needle = args[1]
                    .as_string()
                    .ok_or_else(|| "contains second argument must be a string".to_string())?;
                Ok(haystack.contains(&needle).into())
            }

            // String prefix: hasPrefix("hello", "he") = True
            "hasPrefix" | "str_prefix" | "prefix" => {
                if args.len() != 2 {
                    return Err("hasPrefix requires 2 arguments".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "hasPrefix first argument must be a string".to_string())?;
                let pre = args[1]
                    .as_string()
                    .ok_or_else(|| "hasPrefix second argument must be a string".to_string())?;
                Ok(pre.prefix(&s).into())
            }

            // String suffix: hasSuffix("hello", "lo") = True
            "hasSuffix" | "str_suffix" | "suffix" => {
                if args.len() != 2 {
                    return Err("hasSuffix requires 2 arguments".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "hasSuffix first argument must be a string".to_string())?;
                let suf = args[1]
                    .as_string()
                    .ok_or_else(|| "hasSuffix second argument must be a string".to_string())?;
                Ok(suf.suffix(&s).into())
            }

            // ============================================
            // SUBSTRING OPERATIONS
            // ============================================

            // Substring extraction: substr("hello", 1, 3) = "ell"
            "substr" | "substring" => {
                if args.len() != 3 {
                    return Err("substr requires 3 arguments (string, start, length)".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "substr first argument must be a string".to_string())?;
                let start = args[1].as_int().ok_or_else(|| {
                    "substr second argument (start) must be an integer".to_string()
                })?;
                let len = args[2].as_int().ok_or_else(|| {
                    "substr third argument (length) must be an integer".to_string()
                })?;
                Ok(s.substr(start, len).into())
            }

            // Find index of substring: indexOf("hello", "ll", 0) = 2
            "indexOf" | "str_indexof" | "indexof" => {
                if args.len() != 3 {
                    return Err(
                        "indexOf requires 3 arguments (haystack, needle, start)".to_string()
                    );
                }
                let haystack = args[0]
                    .as_string()
                    .ok_or_else(|| "indexOf first argument must be a string".to_string())?;
                let needle = args[1]
                    .as_string()
                    .ok_or_else(|| "indexOf second argument must be a string".to_string())?;
                let start = args[2]
                    .as_int()
                    .ok_or_else(|| "indexOf third argument must be an integer".to_string())?;
                Ok(haystack.index_of(&needle, start).into())
            }

            // Replace first occurrence: replace("hello", "l", "L") = "heLlo"
            "replace" | "str_replace" => {
                if args.len() != 3 {
                    return Err("replace requires 3 arguments (string, old, new)".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "replace first argument must be a string".to_string())?;
                let old = args[1]
                    .as_string()
                    .ok_or_else(|| "replace second argument must be a string".to_string())?;
                let new_str = args[2]
                    .as_string()
                    .ok_or_else(|| "replace third argument must be a string".to_string())?;
                Ok(s.replace(&old, &new_str).into())
            }

            // Get character at index: charAt("hello", 0) = "h"
            // Uses at() which returns the character at the given index as a string
            "charAt" | "str_at" => {
                if args.len() != 2 {
                    return Err("charAt requires 2 arguments (string, index)".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "charAt first argument must be a string".to_string())?;
                let idx = args[1]
                    .as_int()
                    .ok_or_else(|| "charAt second argument must be an integer".to_string())?;
                Ok(s.at(idx).into())
            }

            // ============================================
            // STRING-INTEGER CONVERSION
            // ============================================

            // String to integer: strToInt("42") = 42
            "strToInt" | "str_to_int" | "toInt" => {
                if args.len() != 1 {
                    return Err("strToInt requires 1 argument".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "strToInt argument must be a string".to_string())?;
                Ok(s.to_int().into())
            }

            // Integer to string: intToStr(42) = "42"
            "intToStr" | "int_to_str" | "fromInt" | "intToString" => {
                if args.len() != 1 {
                    return Err("intToStr requires 1 argument".to_string());
                }
                let n = args[0]
                    .as_int()
                    .ok_or_else(|| "intToStr argument must be an integer".to_string())?;
                Ok(z3::ast::String::from_int(&n).into())
            }

            // ============================================
            // REGULAR EXPRESSION OPERATIONS
            // ============================================

            // Check if string matches regex: matchesRegex("hello", "hello")
            // Note: The pattern is a literal string that must match exactly
            // For more complex patterns, use the isDigits/isAlpha helpers
            "matchesRegex" | "matches" | "str_in_re" => {
                if args.len() != 2 {
                    return Err("matchesRegex requires 2 arguments (string, pattern)".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "matchesRegex first argument must be a string".to_string())?;
                let pattern = args[1]
                    .as_string()
                    .ok_or_else(|| "matchesRegex second argument must be a string".to_string())?;
                // Get the pattern as a Rust string and create a literal regex
                // Note: This treats the pattern as a literal string match
                if let Some(pattern_str) = pattern.as_string() {
                    let re = z3::ast::Regexp::literal(&pattern_str);
                    Ok(s.regex_matches(&re).into())
                } else {
                    // Pattern is symbolic - use uninterpreted function
                    let func_decl = self.declare_uninterpreted("matchesRegex", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Check if string contains only digits: isDigits("123") = True
            "isDigits" | "is_digits" => {
                if args.len() != 1 {
                    return Err("isDigits requires 1 argument".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "isDigits argument must be a string".to_string())?;
                // Regex for zero or more digits: [0-9]*
                let digit = z3::ast::Regexp::range(&'0', &'9');
                let digits_re = z3::ast::Regexp::star(&digit);
                Ok(s.regex_matches(&digits_re).into())
            }

            // Check if string contains only letters: isAlpha("abc") = True
            "isAlpha" | "is_alpha" => {
                if args.len() != 1 {
                    return Err("isAlpha requires 1 argument".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "isAlpha argument must be a string".to_string())?;
                // Regex for letters: [a-zA-Z]*
                let lower = z3::ast::Regexp::range(&'a', &'z');
                let upper = z3::ast::Regexp::range(&'A', &'Z');
                let letter = z3::ast::Regexp::union(&[&lower, &upper]);
                let letters_re = z3::ast::Regexp::star(&letter);
                Ok(s.regex_matches(&letters_re).into())
            }

            // Check if string is alphanumeric: isAlphaNum("abc123") = True
            "isAlphaNum" | "is_alphanum" => {
                if args.len() != 1 {
                    return Err("isAlphaNum requires 1 argument".to_string());
                }
                let s = args[0]
                    .as_string()
                    .ok_or_else(|| "isAlphaNum argument must be a string".to_string())?;
                // Regex for alphanumeric: [a-zA-Z0-9]*
                let lower = z3::ast::Regexp::range(&'a', &'z');
                let upper = z3::ast::Regexp::range(&'A', &'Z');
                let digit = z3::ast::Regexp::range(&'0', &'9');
                let alphanum = z3::ast::Regexp::union(&[&lower, &upper, &digit]);
                let alphanum_re = z3::ast::Regexp::star(&alphanum);
                Ok(s.regex_matches(&alphanum_re).into())
            }

            // ============================================
            // SET OPERATIONS
            // Uses Z3's native set theory
            // ============================================

            // Empty set: empty_set or builtin_set_empty
            "empty_set" | "builtin_set_empty" | "set_empty" => {
                // For empty set, we need to know the element type
                // Default to Int sort for now
                let int_sort = z3::Sort::int();
                Ok(z3::ast::Set::empty(&int_sort).into())
            }

            // Set membership: in_set(x, S) or builtin_set_member
            "in_set" | "builtin_set_member" | "set_member" | "member" => {
                if args.len() != 2 {
                    return Err("in_set requires 2 arguments (element, set)".to_string());
                }
                let set = args[1]
                    .as_set()
                    .ok_or_else(|| "in_set second argument must be a set".to_string())?;
                Ok(set.member(&args[0]).into())
            }

            // Set union: union(A, B) or builtin_set_union
            "union" | "builtin_set_union" | "set_union" => {
                if args.len() < 2 {
                    return Err("union requires at least 2 arguments".to_string());
                }
                let sets: Result<Vec<z3::ast::Set>, String> = args
                    .iter()
                    .map(|a| {
                        a.as_set()
                            .ok_or_else(|| "union arguments must be sets".to_string())
                    })
                    .collect();
                let sets = sets?;
                let refs: Vec<&z3::ast::Set> = sets.iter().collect();
                Ok(z3::ast::Set::set_union(&refs).into())
            }

            // Set intersection: intersect(A, B) or builtin_set_intersect
            "intersect" | "builtin_set_intersect" | "set_intersect" | "intersection" => {
                if args.len() < 2 {
                    return Err("intersect requires at least 2 arguments".to_string());
                }
                let sets: Result<Vec<z3::ast::Set>, String> = args
                    .iter()
                    .map(|a| {
                        a.as_set()
                            .ok_or_else(|| "intersect arguments must be sets".to_string())
                    })
                    .collect();
                let sets = sets?;
                let refs: Vec<&z3::ast::Set> = sets.iter().collect();
                Ok(z3::ast::Set::intersect(&refs).into())
            }

            // Set difference: difference(A, B) or builtin_set_difference
            "difference" | "builtin_set_difference" | "set_difference" | "set_diff" => {
                if args.len() != 2 {
                    return Err("difference requires 2 arguments".to_string());
                }
                let set_a = args[0]
                    .as_set()
                    .ok_or_else(|| "difference first argument must be a set".to_string())?;
                let set_b = args[1]
                    .as_set()
                    .ok_or_else(|| "difference second argument must be a set".to_string())?;
                Ok(set_a.difference(&set_b).into())
            }

            // Set complement: complement(A) or builtin_set_complement
            "complement" | "builtin_set_complement" | "set_complement" => {
                if args.len() != 1 {
                    return Err("complement requires 1 argument".to_string());
                }
                let set = args[0]
                    .as_set()
                    .ok_or_else(|| "complement argument must be a set".to_string())?;
                Ok(set.complement().into())
            }

            // Subset check: subset(A, B) or builtin_set_subset
            "subset" | "builtin_set_subset" | "set_subset" => {
                if args.len() != 2 {
                    return Err("subset requires 2 arguments".to_string());
                }
                let set_a = args[0]
                    .as_set()
                    .ok_or_else(|| "subset first argument must be a set".to_string())?;
                let set_b = args[1]
                    .as_set()
                    .ok_or_else(|| "subset second argument must be a set".to_string())?;
                Ok(set_a.set_subset(&set_b).into())
            }

            // Singleton set: singleton(x) or builtin_set_singleton
            "singleton" | "builtin_set_singleton" | "set_singleton" => {
                if args.len() != 1 {
                    return Err("singleton requires 1 argument".to_string());
                }
                // Create empty set and add the element
                let int_sort = z3::Sort::int();
                let empty = z3::ast::Set::empty(&int_sort);
                Ok(empty.add(&args[0]).into())
            }

            // Add element to set: insert(x, S) or builtin_set_add
            "insert" | "builtin_set_add" | "set_add" => {
                if args.len() != 2 {
                    return Err("insert requires 2 arguments (element, set)".to_string());
                }
                let set = args[1]
                    .as_set()
                    .ok_or_else(|| "insert second argument must be a set".to_string())?;
                Ok(set.add(&args[0]).into())
            }

            // Remove element from set: remove(x, S) or builtin_set_del
            "remove" | "builtin_set_del" | "set_del" => {
                if args.len() != 2 {
                    return Err("remove requires 2 arguments (element, set)".to_string());
                }
                let set = args[1]
                    .as_set()
                    .ok_or_else(|| "remove second argument must be a set".to_string())?;
                Ok(set.del(&args[0]).into())
            }

            // ============================================
            // COMPLEX NUMBER OPERATIONS (Hybrid Translation)
            // Uses Z3 Datatype for concrete arithmetic!
            // Complex = mk_complex(re: Real, im: Real)
            // ============================================

            // Imaginary unit: i = complex(0, 1)
            // This is a nullary operation (0 arguments)
            "i" => {
                if !args.is_empty() {
                    return Err("i takes no arguments".to_string());
                }
                if let Some(i_value) = self.get_complex_i() {
                    return Ok(i_value);
                }
                // Fallback to uninterpreted constant
                let func_decl = self.declare_uninterpreted("i", 0);
                Ok(func_decl.apply(&[]))
            }

            // Complex constructor: complex(re, im) creates re + im*i
            "complex" => {
                if args.len() != 2 {
                    return Err("complex requires 2 arguments (re, im)".to_string());
                }
                // Use datatype constructor for algebraic operations
                if let Some(ref cdt) = self.complex_datatype {
                    // Convert args to Real if they're Int
                    let re = args[0]
                        .as_real()
                        .or_else(|| args[0].as_int().map(|i| i.to_real()))
                        .ok_or("complex re argument must be numeric")?;
                    let im = args[1]
                        .as_real()
                        .or_else(|| args[1].as_int().map(|i| i.to_real()))
                        .ok_or("complex im argument must be numeric")?;
                    Ok(cdt.constructor().apply(&[&re as &dyn Ast, &im as &dyn Ast]))
                } else {
                    // Fallback to uninterpreted
                    let func_decl = self.declare_uninterpreted("complex", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Extract real part: re(z)
            "re" | "real_part" => {
                if args.len() != 1 {
                    return Err("re requires 1 argument".to_string());
                }
                // Use datatype accessor
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) {
                        return Ok(cdt.accessor_re().apply(&[&args[0] as &dyn Ast]));
                    }
                }
                // Fallback for symbolic complex
                let func_decl = self.declare_uninterpreted("re", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Extract imaginary part: im(z)
            "im" | "imag_part" => {
                if args.len() != 1 {
                    return Err("im requires 1 argument".to_string());
                }
                // Use datatype accessor
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) {
                        return Ok(cdt.accessor_im().apply(&[&args[0] as &dyn Ast]));
                    }
                }
                // Fallback for symbolic complex
                let func_decl = self.declare_uninterpreted("im", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex conjugate: conj(z) = complex(re(z), -im(z))
            "conj" | "conjugate" => {
                if args.len() != 1 {
                    return Err("conj requires 1 argument".to_string());
                }
                // Use algebraic translation
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) {
                        let re = cdt.accessor_re().apply(&[&args[0] as &dyn Ast]);
                        let im = cdt.accessor_im().apply(&[&args[0] as &dyn Ast]);
                        let neg_im = im.as_real().map(|r| r.unary_minus()).ok_or("im not Real")?;
                        let re_real = re.as_real().ok_or("re not Real")?;
                        return Ok(cdt
                            .constructor()
                            .apply(&[&re_real as &dyn Ast, &neg_im as &dyn Ast]));
                    }
                }
                // Fallback
                let func_decl = self.declare_uninterpreted("conj", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex addition: (a+bi) + (c+di) = (a+c) + (b+d)i
            "complex_add" => {
                if args.len() != 2 {
                    return Err("complex_add requires 2 arguments".to_string());
                }
                // Algebraic translation
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) && self.is_complex_sort(&args[1]) {
                        let re1 = cdt.accessor_re().apply(&[&args[0] as &dyn Ast]);
                        let im1 = cdt.accessor_im().apply(&[&args[0] as &dyn Ast]);
                        let re2 = cdt.accessor_re().apply(&[&args[1] as &dyn Ast]);
                        let im2 = cdt.accessor_im().apply(&[&args[1] as &dyn Ast]);

                        let re_sum = Real::add(&[
                            &re1.as_real().ok_or("re1 not Real")?,
                            &re2.as_real().ok_or("re2 not Real")?,
                        ]);
                        let im_sum = Real::add(&[
                            &im1.as_real().ok_or("im1 not Real")?,
                            &im2.as_real().ok_or("im2 not Real")?,
                        ]);
                        return Ok(cdt
                            .constructor()
                            .apply(&[&re_sum as &dyn Ast, &im_sum as &dyn Ast]));
                    }
                }
                // Fallback
                let func_decl = self.declare_uninterpreted("complex_add", 2);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex multiplication: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
            "complex_mul" => {
                if args.len() != 2 {
                    return Err("complex_mul requires 2 arguments".to_string());
                }
                // Algebraic translation
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) && self.is_complex_sort(&args[1]) {
                        let a = cdt
                            .accessor_re()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("a not Real")?;
                        let b = cdt
                            .accessor_im()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("b not Real")?;
                        let c = cdt
                            .accessor_re()
                            .apply(&[&args[1] as &dyn Ast])
                            .as_real()
                            .ok_or("c not Real")?;
                        let d = cdt
                            .accessor_im()
                            .apply(&[&args[1] as &dyn Ast])
                            .as_real()
                            .ok_or("d not Real")?;

                        // Real part: ac - bd
                        let ac = Real::mul(&[&a, &c]);
                        let bd = Real::mul(&[&b, &d]);
                        let re_result = Real::sub(&[&ac, &bd]);

                        // Imaginary part: ad + bc
                        let ad = Real::mul(&[&a, &d]);
                        let bc = Real::mul(&[&b, &c]);
                        let im_result = Real::add(&[&ad, &bc]);

                        return Ok(cdt
                            .constructor()
                            .apply(&[&re_result as &dyn Ast, &im_result as &dyn Ast]));
                    }
                }
                // Fallback
                let func_decl = self.declare_uninterpreted("complex_mul", 2);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex inverse: 1/z = conj(z) / |z|²
            "complex_inverse" => {
                if args.len() != 1 {
                    return Err("complex_inverse requires 1 argument".to_string());
                }
                // Algebraic: 1/z = (a - bi) / (a² + b²)
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) {
                        let a = cdt
                            .accessor_re()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("a not Real")?;
                        let b = cdt
                            .accessor_im()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("b not Real")?;

                        // |z|² = a² + b²
                        let a_sq = Real::mul(&[&a, &a]);
                        let b_sq = Real::mul(&[&b, &b]);
                        let abs_sq = Real::add(&[&a_sq, &b_sq]);

                        // 1/z = (a / |z|², -b / |z|²)
                        let re_result = a.div(&abs_sq);
                        let neg_b = b.unary_minus();
                        let im_result = neg_b.div(&abs_sq);

                        return Ok(cdt
                            .constructor()
                            .apply(&[&re_result as &dyn Ast, &im_result as &dyn Ast]));
                    }
                }
                let func_decl = self.declare_uninterpreted("complex_inverse", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex subtraction: (a+bi) - (c+di) = (a-c) + (b-d)i
            "complex_sub" => {
                if args.len() != 2 {
                    return Err("complex_sub requires 2 arguments".to_string());
                }
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) && self.is_complex_sort(&args[1]) {
                        let re1 = cdt
                            .accessor_re()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("re1")?;
                        let im1 = cdt
                            .accessor_im()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("im1")?;
                        let re2 = cdt
                            .accessor_re()
                            .apply(&[&args[1] as &dyn Ast])
                            .as_real()
                            .ok_or("re2")?;
                        let im2 = cdt
                            .accessor_im()
                            .apply(&[&args[1] as &dyn Ast])
                            .as_real()
                            .ok_or("im2")?;

                        let re_diff = Real::sub(&[&re1, &re2]);
                        let im_diff = Real::sub(&[&im1, &im2]);
                        return Ok(cdt
                            .constructor()
                            .apply(&[&re_diff as &dyn Ast, &im_diff as &dyn Ast]));
                    }
                }
                let func_decl = self.declare_uninterpreted("complex_sub", 2);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex division: z1/z2 = z1 * (1/z2)
            "complex_div" => {
                if args.len() != 2 {
                    return Err("complex_div requires 2 arguments".to_string());
                }
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) && self.is_complex_sort(&args[1]) {
                        let a = cdt
                            .accessor_re()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("a")?;
                        let b = cdt
                            .accessor_im()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("b")?;
                        let c = cdt
                            .accessor_re()
                            .apply(&[&args[1] as &dyn Ast])
                            .as_real()
                            .ok_or("c")?;
                        let d = cdt
                            .accessor_im()
                            .apply(&[&args[1] as &dyn Ast])
                            .as_real()
                            .ok_or("d")?;

                        // z1/z2 = (ac + bd)/(c² + d²) + i(bc - ad)/(c² + d²)
                        let c_sq = Real::mul(&[&c, &c]);
                        let d_sq = Real::mul(&[&d, &d]);
                        let denom = Real::add(&[&c_sq, &d_sq]);

                        let ac = Real::mul(&[&a, &c]);
                        let bd = Real::mul(&[&b, &d]);
                        let bc = Real::mul(&[&b, &c]);
                        let ad = Real::mul(&[&a, &d]);

                        let re_num = Real::add(&[&ac, &bd]);
                        let im_num = Real::sub(&[&bc, &ad]);

                        let re_result = re_num.div(&denom);
                        let im_result = im_num.div(&denom);

                        return Ok(cdt
                            .constructor()
                            .apply(&[&re_result as &dyn Ast, &im_result as &dyn Ast]));
                    }
                }
                let func_decl = self.declare_uninterpreted("complex_div", 2);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex negation: -z = (-re, -im)
            "neg_complex" => {
                if args.len() != 1 {
                    return Err("neg_complex requires 1 argument".to_string());
                }
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) {
                        let re = cdt
                            .accessor_re()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("re")?;
                        let im = cdt
                            .accessor_im()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("im")?;
                        let neg_re = re.unary_minus();
                        let neg_im = im.unary_minus();
                        return Ok(cdt
                            .constructor()
                            .apply(&[&neg_re as &dyn Ast, &neg_im as &dyn Ast]));
                    }
                }
                let func_decl = self.declare_uninterpreted("neg_complex", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Complex magnitude squared: |z|² = re² + im²
            "abs_squared" => {
                if args.len() != 1 {
                    return Err("abs_squared requires 1 argument".to_string());
                }
                if let Some(ref cdt) = self.complex_datatype {
                    if self.is_complex_sort(&args[0]) {
                        let re = cdt
                            .accessor_re()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("re")?;
                        let im = cdt
                            .accessor_im()
                            .apply(&[&args[0] as &dyn Ast])
                            .as_real()
                            .ok_or("im")?;
                        let re_sq = Real::mul(&[&re, &re]);
                        let im_sq = Real::mul(&[&im, &im]);
                        return Ok(Real::add(&[&re_sq, &im_sq]).into());
                    }
                }
                let func_decl = self.declare_uninterpreted("abs_squared", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // ============================================
            // RATIONAL NUMBER OPERATIONS
            // Z3 Real sort is actually ℚ (rationals), so we use it directly
            // ============================================

            // Rational constructor: rational(p, q) = p / q
            "rational" => {
                if args.len() != 2 {
                    return Err("rational requires 2 arguments".to_string());
                }
                // Convert to Real and divide
                let numer = self.to_real(&args[0])?;
                let denom = self.to_real(&args[1])?;
                Ok(Real::div(&numer, &denom).into())
            }

            // Rational addition
            "rational_add" => {
                if args.len() != 2 {
                    return Err("rational_add requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(Real::add(&[&r1, &r2]).into())
            }

            // Rational subtraction
            "rational_sub" => {
                if args.len() != 2 {
                    return Err("rational_sub requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(Real::sub(&[&r1, &r2]).into())
            }

            // Rational multiplication
            "rational_mul" => {
                if args.len() != 2 {
                    return Err("rational_mul requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(Real::mul(&[&r1, &r2]).into())
            }

            // Rational division
            "rational_div" => {
                if args.len() != 2 {
                    return Err("rational_div requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(Real::div(&r1, &r2).into())
            }

            // Rational negation
            "neg_rational" => {
                if args.len() != 1 {
                    return Err("neg_rational requires 1 argument".to_string());
                }
                let r = self.to_real(&args[0])?;
                Ok(r.unary_minus().into())
            }

            // Rational inverse (reciprocal)
            "rational_inv" => {
                if args.len() != 1 {
                    return Err("rational_inv requires 1 argument".to_string());
                }
                let r = self.to_real(&args[0])?;
                let one = Real::from_rational(1, 1);
                Ok(Real::div(&one, &r).into())
            }

            // Rational comparisons - return Bool
            "rational_lt" => {
                if args.len() != 2 {
                    return Err("rational_lt requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(r1.lt(&r2).into())
            }

            "rational_le" => {
                if args.len() != 2 {
                    return Err("rational_le requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(r1.le(&r2).into())
            }

            "rational_gt" => {
                if args.len() != 2 {
                    return Err("rational_gt requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(r1.gt(&r2).into())
            }

            "rational_ge" => {
                if args.len() != 2 {
                    return Err("rational_ge requires 2 arguments".to_string());
                }
                let r1 = self.to_real(&args[0])?;
                let r2 = self.to_real(&args[1])?;
                Ok(r1.ge(&r2).into())
            }

            // Integer to rational conversion
            "int_to_rational" | "nat_to_rational" => {
                if args.len() != 1 {
                    return Err(format!("{} requires 1 argument", name));
                }
                // Convert Int to Real (ℤ → ℚ)
                Ok(self.to_real(&args[0])?.into())
            }

            // Rational to real (identity in Z3, since Real = ℚ)
            "to_real" => {
                if args.len() != 1 {
                    return Err("to_real requires 1 argument".to_string());
                }
                Ok(self.to_real(&args[0])?.into())
            }

            // Numerator accessor (uninterpreted - Z3 doesn't expose this)
            "numer" => {
                let func_decl = self.declare_uninterpreted("numer", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // Denominator accessor (uninterpreted - Z3 doesn't expose this)
            "denom" => {
                let func_decl = self.declare_uninterpreted("denom", 1);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // ============================================
            // INTEGER DIVISION AND MODULO OPERATIONS
            // ============================================

            // Integer division: a div b (floor division)
            "int_div" | "div" => {
                if args.len() != 2 {
                    return Err("int_div requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_int(), args[1].as_int()) {
                    Ok(a.div(&b).into())
                } else {
                    Err("int_div requires integer arguments".to_string())
                }
            }

            // Integer modulo: a mod b (always non-negative result)
            "int_mod" | "mod" => {
                if args.len() != 2 {
                    return Err("int_mod requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_int(), args[1].as_int()) {
                    Ok(a.modulo(&b).into())
                } else {
                    Err("int_mod requires integer arguments".to_string())
                }
            }

            // Integer remainder: a rem b (sign follows dividend)
            "int_rem" | "rem" => {
                if args.len() != 2 {
                    return Err("int_rem requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_int(), args[1].as_int()) {
                    Ok(a.rem(&b).into())
                } else {
                    Err("int_rem requires integer arguments".to_string())
                }
            }

            // ============================================
            // FLOOR AND CEILING (ℚ → ℤ)
            // ============================================

            // Floor: largest integer ≤ r
            "floor" => {
                if args.len() != 1 {
                    return Err("floor requires 1 argument".to_string());
                }
                let r = self.to_real(&args[0])?;
                // Z3's Real::to_int() computes floor
                Ok(r.to_int().into())
            }

            // Ceiling: smallest integer ≥ r
            // ceil(r) = -floor(-r)
            "ceil" | "ceiling" => {
                if args.len() != 1 {
                    return Err("ceil requires 1 argument".to_string());
                }
                let r = self.to_real(&args[0])?;
                let neg_r = r.unary_minus();
                let floor_neg_r = neg_r.to_int();
                Ok(Int::unary_minus(&floor_neg_r).into())
            }

            // ============================================
            // GCD (Greatest Common Divisor)
            // Defined axiomatically: gcd(a,b) is the largest d such that d|a and d|b
            // ============================================
            "gcd" => {
                if args.len() != 2 {
                    return Err("gcd requires 2 arguments".to_string());
                }
                // Use uninterpreted function with axioms
                // The actual GCD computation is done via axioms in stdlib/rational.kleis
                let func_decl = self.declare_uninterpreted("gcd", 2);
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }

            // ============================================
            // ABSOLUTE VALUE
            // ============================================

            // Absolute value for rationals (abs is handled above, this catches abs_rational)
            "abs_rational" => {
                if args.len() != 1 {
                    return Err("abs requires 1 argument".to_string());
                }
                let r = self.to_real(&args[0])?;
                let zero = Real::from_rational(0, 1);
                let neg_r = r.unary_minus();
                // abs(r) = if r >= 0 then r else -r
                Ok(r.ge(&zero).ite(&r, &neg_r).into())
            }

            // ============================================
            // BIT-VECTOR OPERATIONS (native Z3 BitVec theory)
            // ============================================

            // Bitwise AND
            "bvand" => {
                if args.len() != 2 {
                    return Err("bvand requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvand(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvand", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bitwise OR
            "bvor" => {
                if args.len() != 2 {
                    return Err("bvor requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvor(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvor", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bitwise XOR
            "bvxor" => {
                if args.len() != 2 {
                    return Err("bvxor requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvxor(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvxor", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bitwise NOT
            "bvnot" => {
                if args.len() != 1 {
                    return Err("bvnot requires 1 argument".to_string());
                }
                if let Some(a) = args[0].as_bv() {
                    Ok(a.bvnot().into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvnot", 1);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bit-vector addition (modular)
            "bvadd" => {
                if args.len() != 2 {
                    return Err("bvadd requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvadd(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvadd", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bit-vector subtraction
            "bvsub" => {
                if args.len() != 2 {
                    return Err("bvsub requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvsub(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvsub", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bit-vector multiplication
            "bvmul" => {
                if args.len() != 2 {
                    return Err("bvmul requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvmul(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvmul", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Bit-vector negation (two's complement)
            "bvneg" => {
                if args.len() != 1 {
                    return Err("bvneg requires 1 argument".to_string());
                }
                if let Some(a) = args[0].as_bv() {
                    Ok(a.bvneg().into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvneg", 1);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Unsigned division
            "bvudiv" => {
                if args.len() != 2 {
                    return Err("bvudiv requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvudiv(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvudiv", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Signed division
            "bvsdiv" => {
                if args.len() != 2 {
                    return Err("bvsdiv requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvsdiv(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvsdiv", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Unsigned remainder
            "bvurem" => {
                if args.len() != 2 {
                    return Err("bvurem requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvurem(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvurem", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Left shift
            "bvshl" => {
                if args.len() != 2 {
                    return Err("bvshl requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvshl(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvshl", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Logical right shift
            "bvlshr" => {
                if args.len() != 2 {
                    return Err("bvlshr requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvlshr(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvlshr", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Arithmetic right shift
            "bvashr" => {
                if args.len() != 2 {
                    return Err("bvashr requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvashr(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvashr", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Unsigned less-than
            "bvult" => {
                if args.len() != 2 {
                    return Err("bvult requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvult(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvult", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Unsigned less-or-equal
            "bvule" => {
                if args.len() != 2 {
                    return Err("bvule requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvule(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvule", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Signed less-than
            "bvslt" => {
                if args.len() != 2 {
                    return Err("bvslt requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvslt(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvslt", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Signed less-or-equal
            "bvsle" => {
                if args.len() != 2 {
                    return Err("bvsle requires 2 arguments".to_string());
                }
                if let (Some(a), Some(b)) = (args[0].as_bv(), args[1].as_bv()) {
                    Ok(a.bvsle(&b).into())
                } else {
                    let func_decl = self.declare_uninterpreted("bvsle", 2);
                    let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                    Ok(func_decl.apply(&ast_args))
                }
            }

            // Unknown operation - use uninterpreted function
            _ => {
                let func_decl = self.declare_uninterpreted(name, args.len());
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }
        }
    }

    /// Convert a Dynamic to a Real (for rational operations)
    fn to_real(&self, d: &Dynamic) -> Result<Real, String> {
        if let Some(r) = d.as_real() {
            Ok(r)
        } else if let Some(i) = d.as_int() {
            Ok(Int::to_real(&i))
        } else {
            // Try to use it as-is and hope it works
            Err(format!("Cannot convert {:?} to Real", d))
        }
    }

    /// Declare an uninterpreted function in Z3
    fn declare_uninterpreted(&mut self, name: &str, arity: usize) -> FuncDecl {
        if !self.declared_ops.contains(name) {
            println!(
                "   🔧 Declaring uninterpreted function: {} with arity {}",
                name, arity
            );
            self.declared_ops.insert(name.to_string());
        }

        let domain: Vec<_> = (0..arity).map(|_| Sort::int()).collect();
        let domain_refs: Vec<_> = domain.iter().collect();
        FuncDecl::new(name, &domain_refs, &Sort::int())
    }

    /// Translate quantifier to Z3
    fn translate_quantifier(
        &mut self,
        _quantifier: &QuantifierKind,
        variables: &[QuantifiedVar],
        where_clause: Option<&Expression>,
        body: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Bool, String> {
        // Create fresh Z3 variables
        let mut new_vars = vars.clone();

        for var in variables {
            let z3_var: Dynamic = if let Some(type_annotation) = &var.type_annotation {
                match type_annotation.as_str() {
                    "Bool" | "Boolean" => Bool::fresh_const(&var.name).into(),
                    "ℝ" | "Real" | "R" => Real::fresh_const(&var.name).into(),
                    "ℤ" | "Int" | "Z" => Int::fresh_const(&var.name).into(),
                    "String" | "Str" => z3::ast::String::fresh_const(&var.name).into(),
                    "ℂ" | "Complex" | "C" => {
                        // Create fresh Complex constant for quantified complex variables
                        self.fresh_complex_const(&var.name)
                            .unwrap_or_else(|| Int::fresh_const(&var.name).into())
                    }
                    "ℚ" | "Rational" | "Q" => {
                        // Rationals are represented as Z3 Real (which is actually ℚ)
                        Real::fresh_const(&var.name).into()
                    }
                    _ => Int::fresh_const(&var.name).into(),
                }
            } else {
                Int::fresh_const(&var.name).into()
            };
            new_vars.insert(var.name.clone(), z3_var);
        }

        // Translate body (with optional where clause)
        let body_z3 = if let Some(condition) = where_clause {
            let condition_z3 = self.kleis_to_z3(condition, &new_vars)?;
            let condition_bool = condition_z3
                .as_bool()
                .ok_or_else(|| "Where clause must be boolean".to_string())?;

            let body_dyn = self.kleis_to_z3(body, &new_vars)?;
            let body_bool = body_dyn
                .as_bool()
                .ok_or_else(|| "Quantifier body must be boolean".to_string())?;

            // where_clause ⟹ body
            condition_bool.implies(&body_bool)
        } else {
            let body_dyn = self.kleis_to_z3(body, &new_vars)?;
            body_dyn
                .as_bool()
                .ok_or_else(|| "Quantifier body must be boolean".to_string())?
        };

        Ok(body_z3)
    }

    /// Translate match expression to nested Z3 ite
    fn translate_match(
        &mut self,
        scrutinee: &Expression,
        cases: &[crate::ast::MatchCase],
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Dynamic, String> {
        if cases.is_empty() {
            return Err("Match expression must have at least one case".to_string());
        }

        // Translate scrutinee
        let scrutinee_z3 = self.kleis_to_z3(scrutinee, vars)?;

        // Build nested ite from cases (last case is the default)
        // We process cases in reverse to build nested ite
        let mut result: Option<Dynamic> = None;

        for case in cases.iter().rev() {
            // Try to translate this case
            let case_result = self.translate_match_case(
                &scrutinee_z3,
                scrutinee,
                &case.pattern,
                &case.body,
                vars,
            )?;

            match (&result, case_result) {
                (None, body_z3) => {
                    // Last case (or only case) - becomes the else branch
                    result = Some(body_z3);
                }
                (Some(else_branch), body_z3) => {
                    // Build condition for this pattern
                    if let Some(condition) =
                        self.pattern_to_condition(&scrutinee_z3, scrutinee, &case.pattern, vars)?
                    {
                        // ite(condition, body, else_branch)
                        result = Some(boolean::translate_ite(&condition, &body_z3, else_branch));
                    } else {
                        // Wildcard or variable - always matches, replaces else
                        result = Some(body_z3);
                    }
                }
            }
        }

        result.ok_or_else(|| "Failed to translate match expression".to_string())
    }

    /// Translate a single match case
    fn translate_match_case(
        &mut self,
        _scrutinee_z3: &Dynamic,
        scrutinee_expr: &Expression,
        pattern: &crate::ast::Pattern,
        body: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Dynamic, String> {
        // Extend vars with pattern bindings
        let mut extended_vars = vars.clone();
        self.bind_pattern_vars(&mut extended_vars, scrutinee_expr, pattern)?;

        // Translate body with extended bindings
        self.kleis_to_z3(body, &extended_vars)
    }

    /// Bind pattern variables to corresponding parts of scrutinee
    fn bind_pattern_vars(
        &mut self,
        vars: &mut HashMap<String, Dynamic>,
        scrutinee: &Expression,
        pattern: &crate::ast::Pattern,
    ) -> Result<(), String> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Variable(name) => {
                // Bind the variable to the scrutinee value
                let scrutinee_z3 = self.kleis_to_z3(scrutinee, vars)?;
                vars.insert(name.clone(), scrutinee_z3);
                Ok(())
            }
            Pattern::Constructor { name: _, args } => {
                // For constructor patterns, we need to extract fields
                // This works when scrutinee is also a constructor application
                if let Expression::Operation {
                    name: _,
                    args: scrutinee_args,
                    ..
                } = scrutinee
                {
                    if args.len() == scrutinee_args.len() {
                        for (pat, arg) in args.iter().zip(scrutinee_args.iter()) {
                            self.bind_pattern_vars(vars, arg, pat)?;
                        }
                    }
                }
                Ok(())
            }
            Pattern::Constant(_) => {
                // Constants don't bind variables
                Ok(())
            }
            // Grammar v0.8: As-pattern binds alias AND recurses
            Pattern::As {
                pattern: inner,
                binding,
            } => {
                // First bind the whole scrutinee to the alias
                let scrutinee_z3 = self.kleis_to_z3(scrutinee, vars)?;
                vars.insert(binding.clone(), scrutinee_z3);
                // Then recurse into the inner pattern
                self.bind_pattern_vars(vars, scrutinee, inner)
            }
        }
    }

    /// Bind pattern variables from a Z3 value (Grammar v0.8: for let destructuring)
    ///
    /// This function extracts bindings from patterns for use in let expressions.
    /// For constructor patterns like `Point(x, y)`, it destructures the expression
    /// and binds pattern variables to corresponding Z3 values.
    fn bind_pattern_to_z3(
        &mut self,
        pattern: &crate::ast::Pattern,
        z3_value: &Dynamic,
        original_expr: &Expression,
        vars: &mut HashMap<String, Dynamic>,
    ) -> Result<(), String> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Variable(name) => {
                vars.insert(name.clone(), z3_value.clone());
                Ok(())
            }
            Pattern::Constructor { name, args } => {
                // Grammar v0.8: Constructor destructuring for let bindings
                // Check if the original expression is an Operation with matching constructor
                match original_expr {
                    Expression::Operation {
                        name: op_name,
                        args: op_args,
                        ..
                    } if op_name == name && op_args.len() == args.len() => {
                        // Recursively bind each pattern argument to the corresponding operation argument
                        for (pat, arg_expr) in args.iter().zip(op_args.iter()) {
                            let arg_z3 = self.kleis_to_z3(arg_expr, vars)?;
                            self.bind_pattern_to_z3(pat, &arg_z3, arg_expr, vars)?;
                        }
                        Ok(())
                    }
                    Expression::Object(var_name) => {
                        // The value is a variable - we can't destructure it at Z3 level
                        // This would require Z3 datatypes with accessors
                        // For now, create symbolic field accessors as fresh variables
                        // This allows verification to proceed with symbolic field values
                        for (i, pat) in args.iter().enumerate() {
                            let field_var_name = format!("{}_{}_field{}", var_name, name, i);
                            let field_z3: Dynamic = Int::fresh_const(&field_var_name).into();
                            // Create a placeholder expression for recursion
                            let placeholder = Expression::Object(field_var_name.clone());
                            self.bind_pattern_to_z3(pat, &field_z3, &placeholder, vars)?;
                        }
                        Ok(())
                    }
                    _ => {
                        // For other expressions, we can try to evaluate/translate and extract
                        // For now, return a more helpful error
                        Err(format!(
                            "Cannot destructure pattern '{}({})' from expression type {:?}. \
                             Constructor destructuring requires a matching Operation or Object.",
                            name,
                            args.len(),
                            std::mem::discriminant(original_expr)
                        ))
                    }
                }
            }
            Pattern::Constant(_) => {
                // Constants don't bind variables
                Ok(())
            }
            Pattern::As {
                pattern: inner,
                binding,
            } => {
                // Bind whole value to alias
                vars.insert(binding.clone(), z3_value.clone());
                // Recurse into inner pattern
                self.bind_pattern_to_z3(inner, z3_value, original_expr, vars)
            }
        }
    }

    /// Convert a pattern to a Z3 boolean condition (None for wildcard/variable)
    fn pattern_to_condition(
        &mut self,
        scrutinee_z3: &Dynamic,
        scrutinee_expr: &Expression,
        pattern: &crate::ast::Pattern,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Bool>, String> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => Ok(None),    // Always matches
            Pattern::Variable(_) => Ok(None), // Always matches (binds)
            Pattern::Constant(val) => {
                // Check if scrutinee equals the constant
                if let Some(scrutinee_int) = scrutinee_z3.as_int() {
                    if let Ok(n) = val.parse::<i64>() {
                        let const_z3 = Int::from_i64(n);
                        Ok(Some(scrutinee_int.eq(&const_z3)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Pattern::Constructor { name, args } => {
                // Check if scrutinee is a constructor with matching name
                if let Expression::Operation {
                    name: scrutinee_name,
                    args: scrutinee_args,
                    ..
                } = scrutinee_expr
                {
                    if scrutinee_name == name && args.len() == scrutinee_args.len() {
                        // Match constructor name - check nested patterns
                        let mut conditions = Vec::new();

                        for (pat, arg) in args.iter().zip(scrutinee_args.iter()) {
                            let arg_z3 = self.kleis_to_z3(arg, vars)?;
                            if let Some(cond) =
                                self.pattern_to_condition(&arg_z3, arg, pat, vars)?
                            {
                                conditions.push(cond);
                            }
                        }

                        if conditions.is_empty() {
                            // All sub-patterns are wildcards/variables
                            Ok(Some(Bool::from_bool(true)))
                        } else {
                            // Combine conditions with AND
                            let mut result = conditions[0].clone();
                            for cond in &conditions[1..] {
                                result = Bool::and(&[&result, cond]);
                            }
                            Ok(Some(result))
                        }
                    } else {
                        // Different constructor - doesn't match
                        Ok(Some(Bool::from_bool(false)))
                    }
                } else if let Expression::Const(val) = scrutinee_expr {
                    // Scrutinee is a literal constant
                    if name == val {
                        Ok(Some(Bool::from_bool(true)))
                    } else {
                        Ok(Some(Bool::from_bool(false)))
                    }
                } else if args.is_empty() {
                    // NULLARY CONSTRUCTOR PATTERN with symbolic scrutinee
                    // This is the key fix for symbolic ADT matching!
                    // Example: match p { Owner => 4 | ... } where p is a variable
                    //
                    // Check if this constructor is a known identity element
                    // If so, compare scrutinee_z3 == identity_element[name]
                    if let Some(constructor_z3) = self.identity_elements.get(name) {
                        // Use Z3 equality to compare the symbolic scrutinee
                        // with the constructor identity element
                        let eq = comparison::translate_equals(scrutinee_z3, constructor_z3)?;
                        Ok(Some(eq))
                    } else {
                        // Constructor not registered as identity element
                        // This shouldn't happen if ADT was properly loaded
                        eprintln!(
                            "   ⚠️ Warning: Constructor '{}' not found in identity elements",
                            name
                        );
                        Ok(None)
                    }
                } else {
                    // Constructor with args on symbolic scrutinee - can't handle yet
                    // Would need Z3 ADT sorts for proper handling
                    Ok(None)
                }
            }
            // Grammar v0.8: As-pattern - just recurse into inner pattern for condition
            Pattern::As { pattern: inner, .. } => {
                self.pattern_to_condition(scrutinee_z3, scrutinee_expr, inner, vars)
            }
        }
    }

    /// Get solver statistics
    pub fn stats(&self) -> SolverStats {
        SolverStats {
            loaded_structures: self.loaded_structures.len(),
            declared_operations: self.declared_ops.len(),
            assertion_count: 0, // TODO: Track assertions
        }
    }

    // =========================================================================
    // Complex Number Support (Hybrid Translation)
    // =========================================================================

    /// Initialize the complex constant 'i' = complex(0, 1)
    /// NOTE: We don't put 'i' in identity_elements because it conflicts with
    /// 'i' used as a loop variable in Sum/Product tests. Instead, we handle
    /// 'i' specially in translate_object_i() below.
    fn initialize_complex_i(&mut self) {
        // Complex numbers initialized - 'i' is handled specially in translate_object_i()
    }

    /// Get the complex constant i = complex(0, 1)
    fn get_complex_i(&self) -> Option<Dynamic> {
        self.complex_datatype.as_ref().map(|cdt| {
            let zero = Real::from_rational(0, 1);
            let one = Real::from_rational(1, 1);
            cdt.constructor()
                .apply(&[&zero as &dyn Ast, &one as &dyn Ast])
        })
    }

    /// Create a concrete complex number from two Real values
    #[allow(dead_code)]
    fn make_complex(&self, re: &Real, im: &Real) -> Option<Dynamic> {
        self.complex_datatype
            .as_ref()
            .map(|cdt| cdt.constructor().apply(&[re as &dyn Ast, im as &dyn Ast]))
    }

    /// Extract real part from a complex Dynamic
    #[allow(dead_code)]
    fn extract_re(&self, z: &Dynamic) -> Option<Dynamic> {
        self.complex_datatype
            .as_ref()
            .map(|cdt| cdt.accessor_re().apply(&[z as &dyn Ast]))
    }

    /// Extract imaginary part from a complex Dynamic
    #[allow(dead_code)]
    fn extract_im(&self, z: &Dynamic) -> Option<Dynamic> {
        self.complex_datatype
            .as_ref()
            .map(|cdt| cdt.accessor_im().apply(&[z as &dyn Ast]))
    }

    /// Check if a Dynamic is of Complex sort
    fn is_complex_sort(&self, d: &Dynamic) -> bool {
        if let Some(ref cdt) = self.complex_datatype {
            d.sort_kind() == z3::SortKind::Datatype
                && d.get_sort().to_string() == cdt.sort.sort.to_string()
        } else {
            false
        }
    }

    /// Create a fresh Complex constant for quantified variables
    /// Returns complex(re_fresh, im_fresh) where re and im are fresh Real constants
    fn fresh_complex_const(&self, name: &str) -> Option<Dynamic> {
        self.complex_datatype.as_ref().map(|cdt| {
            // Create fresh Real constants for the real and imaginary parts
            let re = Real::fresh_const(&format!("{}_re", name));
            let im = Real::fresh_const(&format!("{}_im", name));
            // Construct the complex number
            cdt.constructor().apply(&[&re as &dyn Ast, &im as &dyn Ast])
        })
    }

    // TODO: These methods are temporary to support AxiomVerifier's axiom loading
    // Should be refactored when axiom loading is moved to backend properly

    /// Load an identity element (nullary operation like zero, one, e)
    pub fn load_identity_element(&mut self, name: &str) {
        if !self.identity_elements.contains_key(name) {
            let z3_const: Dynamic = Int::fresh_const(name).into();

            // Assert this new constant is distinct from all existing identity elements
            // This is critical for symbolic ADT matching to work correctly!
            for (existing_name, existing_z3) in &self.identity_elements {
                if let (Some(new_int), Some(existing_int)) =
                    (z3_const.as_int(), existing_z3.as_int())
                {
                    // Assert: new_const ≠ existing_const
                    let distinct = new_int.eq(&existing_int).not();
                    self.solver.assert(&distinct);
                    println!("   🔒 Asserted distinct: {} ≠ {}", name, existing_name);
                }
            }

            self.identity_elements.insert(name.to_string(), z3_const);
            println!("   📌 Loaded identity element: {}", name);
        }
    }

    /// Translate Kleis expression to Z3 and assert it (for axiom loading)
    pub fn assert_kleis_expression(&mut self, expr: &Expression) -> Result<(), String> {
        let z3_expr = self.kleis_to_z3(expr, &HashMap::new())?;
        let z3_bool = z3_expr
            .as_bool()
            .ok_or_else(|| "Expression must be boolean for assertion".to_string())?;
        self.solver.assert(&z3_bool);
        Ok(())
    }

    /// Declare a function and assert its definition (for function loading)
    pub fn declare_and_define_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &Expression,
    ) -> Result<(), String> {
        // Create fresh Z3 variables for parameters
        let mut z3_vars = HashMap::new();
        let mut param_ints = Vec::new();

        for param in params {
            let z3_var = Int::fresh_const(param);
            param_ints.push(z3_var.clone());
            z3_vars.insert(param.clone(), z3_var.into());
        }

        // Translate function body
        let body_z3 = self.kleis_to_z3(body, &z3_vars)?;

        // Declare function
        let func_decl = self.declare_uninterpreted(name, params.len());

        // Create application and assert definition
        let ast_args: Vec<&dyn Ast> = param_ints.iter().map(|p| p as &dyn Ast).collect();
        let func_app = func_decl.apply(&ast_args);
        let definition = func_app.eq(&body_z3);
        self.solver.assert(&definition);

        Ok(())
    }
}

impl<'r> SolverBackend for Z3Backend<'r> {
    fn name(&self) -> &str {
        "Z3"
    }

    fn capabilities(&self) -> &SolverCapabilities {
        &self.capabilities
    }

    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String> {
        // Use push/pop for incremental solving
        self.solver.push();

        // Translate to Z3
        let z3_expr = self.kleis_to_z3(axiom, &HashMap::new())?;
        let z3_bool = z3_expr
            .as_bool()
            .ok_or_else(|| "Axiom must be a boolean expression".to_string())?;

        // Assert negation
        self.solver.assert(z3_bool.not());

        // Check satisfiability
        let result = match self.solver.check() {
            SatResult::Unsat => VerificationResult::Valid,
            SatResult::Sat => {
                let counterexample = if let Some(model) = self.solver.get_model() {
                    format!("{}", model)
                } else {
                    "No model available".to_string()
                };
                VerificationResult::Invalid { counterexample }
            }
            SatResult::Unknown => VerificationResult::Unknown,
        };

        // Pop the assertion
        self.solver.pop(1);

        Ok(result)
    }

    fn check_satisfiability(&mut self, expr: &Expression) -> Result<SatisfiabilityResult, String> {
        // Use push/pop for incremental solving
        self.solver.push();

        // Translate to Z3
        let z3_expr = self.kleis_to_z3(expr, &HashMap::new())?;
        let z3_bool = z3_expr
            .as_bool()
            .ok_or_else(|| "Expression must be a boolean proposition".to_string())?;

        // Assert the expression directly (not negated)
        self.solver.assert(&z3_bool);

        // Check satisfiability
        let result = match self.solver.check() {
            SatResult::Sat => {
                let example = if let Some(model) = self.solver.get_model() {
                    format!("{}", model)
                } else {
                    "Satisfiable (no model details)".to_string()
                };
                SatisfiabilityResult::Satisfiable { example }
            }
            SatResult::Unsat => SatisfiabilityResult::Unsatisfiable,
            SatResult::Unknown => SatisfiabilityResult::Unknown,
        };

        // Pop the assertion
        self.solver.pop(1);

        Ok(result)
    }

    fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String> {
        // Translate Kleis expression to Z3
        let z3_expr = self.kleis_to_z3(expr, &HashMap::new())?;

        // For evaluation, we need a concrete value
        // Use self.solver which has axioms already asserted
        // Push a scope so we can pop after evaluation
        self.solver.push();

        // For constant expressions, we can try to extract the value directly
        // For symbolic expressions, we need a model

        // Try to get concrete value directly
        if let Some(int_val) = z3_expr.as_int() {
            if let Some(value) = int_val.as_i64() {
                self.solver.pop(1);
                return Ok(Expression::Const(value.to_string()));
            }
        }

        if let Some(bool_val) = z3_expr.as_bool() {
            if let Some(value) = bool_val.as_bool() {
                return Ok(Expression::Const(value.to_string()));
            }
        }

        if let Some(real_val) = z3_expr.as_real() {
            if let Some((num, den)) = real_val.as_rational() {
                if den == 1 {
                    return Ok(Expression::Const(num.to_string()));
                } else {
                    let decimal = num as f64 / den as f64;
                    return Ok(Expression::Const(decimal.to_string()));
                }
            }
        }

        // For symbolic expressions, try to get a satisfying model
        // NOTE: This can hang with quantified axioms! Use with caution.

        // Create a fresh variable and assert it equals our expression
        let result_var = Int::fresh_const("eval_result");

        // Try to cast z3_expr to Int and assert equality
        if let Some(int_expr) = z3_expr.as_int() {
            self.solver.assert(result_var.eq(&int_expr));

            match self.solver.check() {
                SatResult::Sat => {
                    if let Some(model) = self.solver.get_model() {
                        if let Some(evaluated) = model.eval(&result_var, true) {
                            let z3_dynamic: Dynamic = evaluated.into();
                            self.solver.pop(1);
                            return self.converter.to_expression(&z3_dynamic);
                        }
                    }
                }
                SatResult::Unsat => {
                    self.solver.pop(1);
                    return Err("Cannot evaluate expression - unsatisfiable".to_string());
                }
                SatResult::Unknown => {
                    self.solver.pop(1);
                    return Err("Cannot evaluate expression - unknown".to_string());
                }
            }
        }

        self.solver.pop(1);

        // Fallback: return string representation
        Ok(Expression::Const(z3_expr.to_string()))
    }

    fn simplify(&mut self, expr: &Expression) -> Result<Expression, String> {
        // Translate Kleis expression to Z3
        let z3_expr = self.kleis_to_z3(expr, &HashMap::new())?;

        // Use Z3's simplify method
        let simplified = z3_expr.simplify();

        // Convert simplified Z3 expression back to Kleis Expression
        // CRITICAL: This maintains the abstraction boundary!

        // Check if it's a concrete value we can extract
        if let Some(int_val) = simplified.as_int() {
            if let Some(value) = int_val.as_i64() {
                return Ok(Expression::Const(value.to_string()));
            }
            // Large integer or symbolic
            return Ok(Expression::Const(int_val.to_string()));
        }

        if let Some(bool_val) = simplified.as_bool() {
            if let Some(value) = bool_val.as_bool() {
                return Ok(Expression::Const(value.to_string()));
            }
            // Symbolic boolean
            return Ok(Expression::Const(bool_val.to_string()));
        }

        if let Some(real_val) = simplified.as_real() {
            if let Some((num, den)) = real_val.as_rational() {
                if den == 1 {
                    return Ok(Expression::Const(num.to_string()));
                } else {
                    let decimal = num as f64 / den as f64;
                    return Ok(Expression::Const(decimal.to_string()));
                }
            }
            return Ok(Expression::Const(real_val.to_string()));
        }

        // For complex expressions that can't be simplified to constants,
        // we need to reconstruct the Kleis AST from Z3's simplified form
        // For now, return string representation (TODO: proper AST reconstruction)
        Ok(Expression::Const(simplified.to_string()))
    }

    fn are_equivalent(&mut self, expr1: &Expression, expr2: &Expression) -> Result<bool, String> {
        self.solver.push();

        let z3_expr1 = self.kleis_to_z3(expr1, &HashMap::new())?;
        let z3_expr2 = self.kleis_to_z3(expr2, &HashMap::new())?;

        // Check if expr1 ≠ expr2 is unsatisfiable
        let equality = if z3_expr1.sort_kind() == z3_expr2.sort_kind() {
            z3_expr1.eq(&z3_expr2)
        } else {
            // Mixed types - try converting to Real
            let e1_real = z3_expr1
                .as_real()
                .or_else(|| z3_expr1.as_int().map(|i| i.to_real()));
            let e2_real = z3_expr2
                .as_real()
                .or_else(|| z3_expr2.as_int().map(|i| i.to_real()));

            if let (Some(r1), Some(r2)) = (e1_real, e2_real) {
                r1.eq(&r2)
            } else {
                return Err("Cannot compare expressions of incompatible types".to_string());
            }
        };

        self.solver.assert(equality.not());
        let result = matches!(self.solver.check(), SatResult::Unsat);
        self.solver.pop(1);

        Ok(result)
    }

    fn load_structure_axioms(
        &mut self,
        _structure_name: &str,
        _axioms: &[Expression],
    ) -> Result<(), String> {
        // TODO: Implement axiom loading
        // This would translate axioms to Z3 and assert them
        Ok(())
    }

    fn push(&mut self) {
        self.solver.push();
    }

    fn pop(&mut self, levels: u32) {
        self.solver.pop(levels);
    }

    fn reset(&mut self) {
        // Create a new solver instance
        self.solver = Solver::new();
        self.declared_ops.clear();
        self.loaded_structures.clear();
        self.identity_elements.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z3_backend_creation() {
        let registry = StructureRegistry::new();
        let backend = Z3Backend::new(&registry);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_backend_name() {
        let registry = StructureRegistry::new();
        let backend = Z3Backend::new(&registry).unwrap();
        assert_eq!(backend.name(), "Z3");
    }

    #[test]
    fn test_capabilities_loaded() {
        let registry = StructureRegistry::new();
        let backend = Z3Backend::new(&registry).unwrap();

        assert!(backend.capabilities().has_operation("plus"));
        assert!(backend.capabilities().has_operation("equals"));
        assert!(backend.capabilities().has_theory("arithmetic"));
    }

    #[test]
    fn test_push_pop() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // Should not panic
        backend.push();
        backend.pop(1);
    }

    #[test]
    fn test_evaluate_returns_kleis_ast() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // Simple arithmetic: 2 + 3
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("2".to_string()),
                Expression::Const("3".to_string()),
            ],
            span: None,
        };

        let result = backend.evaluate(&expr).unwrap();

        // Result MUST be Kleis Expression, not Z3 type!
        match result {
            Expression::Const(s) => {
                assert_eq!(s, "5", "2 + 3 should evaluate to 5");
            }
            _ => panic!("Expected Expression::Const, got {:?}", result),
        }
    }

    #[test]
    fn test_simplify_returns_kleis_ast() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // Expression: x + 0 (should simplify to x in ideal case, but at minimum returns Expression)
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("42".to_string()),
                Expression::Const("0".to_string()),
            ],
            span: None,
        };

        let result = backend.simplify(&expr).unwrap();

        // Result MUST be Kleis Expression, not Z3 type!
        match result {
            Expression::Const(s) => {
                assert_eq!(s, "42", "42 + 0 should simplify to 42");
            }
            _ => panic!("Expected Expression::Const, got {:?}", result),
        }
    }

    #[test]
    fn test_evaluate_concrete_constant() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // Already a constant
        let expr = Expression::Const("123".to_string());
        let result = backend.evaluate(&expr).unwrap();

        assert_eq!(result, Expression::Const("123".to_string()));
    }

    #[test]
    fn test_conditional_true_branch() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // if true then 42 else 0
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Const("1".to_string()),
                ],
                span: None,
            }),
            then_branch: Box::new(Expression::Const("42".to_string())),
            else_branch: Box::new(Expression::Const("0".to_string())),
            span: None,
        };

        let result = backend.evaluate(&expr).unwrap();
        assert_eq!(result, Expression::Const("42".to_string()));
    }

    #[test]
    fn test_conditional_false_branch() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // if false then 42 else 0
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Const("2".to_string()),
                ],
                span: None,
            }),
            then_branch: Box::new(Expression::Const("42".to_string())),
            else_branch: Box::new(Expression::Const("0".to_string())),
            span: None,
        };

        let result = backend.evaluate(&expr).unwrap();
        assert_eq!(result, Expression::Const("0".to_string()));
    }

    #[test]
    fn test_conditional_with_arithmetic() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // if 5 > 3 then 10 + 1 else 20 + 1
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "greater_than".to_string(),
                args: vec![
                    Expression::Const("5".to_string()),
                    Expression::Const("3".to_string()),
                ],
                span: None,
            }),
            then_branch: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("10".to_string()),
                    Expression::Const("1".to_string()),
                ],
                span: None,
            }),
            else_branch: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("20".to_string()),
                    Expression::Const("1".to_string()),
                ],
                span: None,
            }),
            span: None,
        };

        let result = backend.evaluate(&expr).unwrap();
        assert_eq!(result, Expression::Const("11".to_string()));
    }

    #[test]
    fn test_conditional_nested() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // if 1 > 2 then 100 else (if 2 > 1 then 200 else 300)
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "greater_than".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Const("2".to_string()),
                ],
                span: None,
            }),
            then_branch: Box::new(Expression::Const("100".to_string())),
            else_branch: Box::new(Expression::Conditional {
                condition: Box::new(Expression::Operation {
                    name: "greater_than".to_string(),
                    args: vec![
                        Expression::Const("2".to_string()),
                        Expression::Const("1".to_string()),
                    ],
                    span: None,
                }),
                then_branch: Box::new(Expression::Const("200".to_string())),
                else_branch: Box::new(Expression::Const("300".to_string())),
                span: None,
            }),
            span: None,
        };

        let result = backend.evaluate(&expr).unwrap();
        assert_eq!(result, Expression::Const("200".to_string()));
    }

    #[test]
    fn test_simplify_conditional() {
        let registry = StructureRegistry::new();
        let mut backend = Z3Backend::new(&registry).unwrap();

        // if true then 5 else 10 should simplify to 5
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Const("1".to_string()),
                ],
                span: None,
            }),
            then_branch: Box::new(Expression::Const("5".to_string())),
            else_branch: Box::new(Expression::Const("10".to_string())),
            span: None,
        };

        let result = backend.simplify(&expr).unwrap();
        assert_eq!(result, Expression::Const("5".to_string()));
    }
}
