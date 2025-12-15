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
use z3::{FuncDecl, SatResult, Solver, Sort};

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
}

impl<'r> Z3Backend<'r> {
    /// Create a new Z3 backend
    ///
    /// # Arguments
    /// * `registry` - Structure registry containing operations and axioms
    ///
    /// # Axiom Loading
    /// Matrix multiplication axioms are loaded for testing Z3 integration.
    /// TODO: In the future, load axioms from stdlib/matrices.kleis instead of hardcoding.
    /// This requires:
    /// 1. Parsing .kleis axiom definitions
    /// 2. Translating them to Z3 assertions
    /// 3. Loading them on-demand when structures are used
    ///
    /// See: load_structure_axioms() for the intended API
    pub fn new(registry: &'r StructureRegistry) -> Result<Self, String> {
        let solver = Solver::new();
        let capabilities = super::load_capabilities()?;

        let mut backend = Self {
            solver,
            registry,
            capabilities,
            declared_ops: HashSet::new(),
            loaded_structures: HashSet::new(),
            identity_elements: HashMap::new(),
            free_variables: HashMap::new(),
            converter: Z3ResultConverter,
        };

        // Load matrix axioms for Z3 integration testing
        // TODO: Replace with axiom loading from stdlib/matrices.kleis
        backend.load_matrix_axioms();

        Ok(backend)
    }

    /// Matrix axiom loading placeholder
    ///
    /// **FOR Z3 INTEGRATION TESTING**
    /// Instead of universal quantifier axioms (which can cause Z3 to hang),
    /// we handle matrix multiplication by inline expansion in try_expand_matrix_multiply.
    ///
    /// TODO: Replace with proper axiom loading from stdlib/matrices.kleis
    fn load_matrix_axioms(&mut self) {
        // Mark matrix operations as declared (handled specially)
        self.declared_ops.insert("matrix_2x1".to_string());
        self.declared_ops.insert("matrix_2x2".to_string());
        self.declared_ops.insert("multiply".to_string());
    }

    /// Try to expand matrix multiplication inline
    ///
    /// If both arguments are Matrix expressions with known dimensions,
    /// we compute the product symbolically and return the result.
    ///
    /// Returns None if this isn't a matrix multiplication we can expand.
    fn try_expand_matrix_multiply(
        &mut self,
        left: &Expression,
        right: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Dynamic>, String> {
        // Extract matrix info: (rows, cols, elements)
        let left_info = self.extract_matrix_info(left);
        let right_info = self.extract_matrix_info(right);

        match (left_info, right_info) {
            // 2x2 × 2x1 -> 2x1 (returns a MatrixResult for component-wise comparison)
            (Some((2, 2, left_elems)), Some((2, 1, right_elems)))
                if left_elems.len() == 4 && right_elems.len() == 2 =>
            {
                // Translate all elements to Z3
                let a = self.kleis_to_z3(&left_elems[0], vars)?;
                let b = self.kleis_to_z3(&left_elems[1], vars)?;
                let c = self.kleis_to_z3(&left_elems[2], vars)?;
                let d = self.kleis_to_z3(&left_elems[3], vars)?;
                let x = self.kleis_to_z3(&right_elems[0], vars)?;
                let y = self.kleis_to_z3(&right_elems[1], vars)?;

                // Compute: result[0] = a*x + b*y, result[1] = c*x + d*y
                let a_int = a.as_int().ok_or("Matrix element must be integer")?;
                let b_int = b.as_int().ok_or("Matrix element must be integer")?;
                let c_int = c.as_int().ok_or("Matrix element must be integer")?;
                let d_int = d.as_int().ok_or("Matrix element must be integer")?;
                let x_int = x.as_int().ok_or("Matrix element must be integer")?;
                let y_int = y.as_int().ok_or("Matrix element must be integer")?;

                let r0 = Int::add(&[&Int::mul(&[&a_int, &x_int]), &Int::mul(&[&b_int, &y_int])]);
                let r1 = Int::add(&[&Int::mul(&[&c_int, &x_int]), &Int::mul(&[&d_int, &y_int])]);

                // Use a unique function that encodes BOTH components
                // This prevents Z3 from "cheating" by making all matrices equal
                // We encode as: r0 * LARGE_PRIME + r1 (bijective for reasonable values)
                let large_prime = Int::from_i64(1000003);
                let encoded = Int::add(&[&Int::mul(&[&r0, &large_prime]), &r1]);
                Ok(Some(encoded.into()))
            }

            // Not a matrix multiplication pattern we recognize
            _ => Ok(None),
        }
    }

    /// Extract matrix dimensions and elements from a Matrix expression
    fn extract_matrix_info(&self, expr: &Expression) -> Option<(usize, usize, Vec<Expression>)> {
        if let Expression::Operation { name, args } = expr {
            if name == "Matrix" && args.len() == 3 {
                let rows = match &args[0] {
                    Expression::Const(s) => s.parse::<usize>().ok()?,
                    _ => return None,
                };
                let cols = match &args[1] {
                    Expression::Const(s) => s.parse::<usize>().ok()?,
                    _ => return None,
                };
                if let Expression::List(elements) = &args[2] {
                    return Some((rows, cols, elements.clone()));
                }
            }
        }
        None
    }

    // =========================================
    // Tensor Expansion Methods (Concrete Tensors)
    // =========================================

    /// Extract Tensor2 info: (dimension, components)
    /// Tensor2(dim, [component_list])
    fn extract_tensor2_info(&self, expr: &Expression) -> Option<(usize, Vec<Expression>)> {
        if let Expression::Operation { name, args } = expr {
            if name == "Tensor2" && args.len() == 2 {
                let dim = match &args[0] {
                    Expression::Const(s) => s.parse::<usize>().ok()?,
                    _ => return None,
                };
                if let Expression::List(elements) = &args[1] {
                    // For a dim×dim tensor, we expect dim² elements
                    if elements.len() == dim * dim {
                        return Some((dim, elements.clone()));
                    }
                }
            }
        }
        None
    }

    /// Extract Vector info: (dimension, components)
    /// Vector(dim, [component_list])
    fn extract_vector_info(&self, expr: &Expression) -> Option<(usize, Vec<Expression>)> {
        if let Expression::Operation { name, args } = expr {
            if name == "Vector" && args.len() == 2 {
                let dim = match &args[0] {
                    Expression::Const(s) => s.parse::<usize>().ok()?,
                    _ => return None,
                };
                if let Expression::List(elements) = &args[1] {
                    if elements.len() == dim {
                        return Some((dim, elements.clone()));
                    }
                }
            }
        }
        None
    }

    /// Expand tensor contraction: A_μρ B^ρ_ν = Σ_ρ A_μρ * B_ρν
    /// Returns encoded result matrix for Z3
    fn try_expand_tensor_contraction(
        &mut self,
        left: &Expression,
        right: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Dynamic>, String> {
        let left_info = self.extract_tensor2_info(left);
        let right_info = self.extract_tensor2_info(right);

        match (left_info, right_info) {
            (Some((dim, left_elems)), Some((dim2, right_elems))) if dim == dim2 => {
                // Contract: C_ij = Σ_k A_ik * B_kj
                // For small dimensions (2, 3, 4), expand explicitly
                if dim > 4 {
                    return Ok(None); // Fall back to uninterpreted for large tensors
                }

                let mut result_terms: Vec<Int> = Vec::with_capacity(dim * dim);

                for i in 0..dim {
                    for j in 0..dim {
                        // C_ij = Σ_k A_ik * B_kj
                        let mut sum_terms: Vec<Int> = Vec::new();
                        for k in 0..dim {
                            let a_idx = i * dim + k;
                            let b_idx = k * dim + j;

                            let a_z3 = self.kleis_to_z3(&left_elems[a_idx], vars)?;
                            let b_z3 = self.kleis_to_z3(&right_elems[b_idx], vars)?;

                            let a_int =
                                a_z3.as_int().ok_or("Tensor element must be integer/real")?;
                            let b_int =
                                b_z3.as_int().ok_or("Tensor element must be integer/real")?;

                            sum_terms.push(Int::mul(&[&a_int, &b_int]));
                        }

                        // Sum all terms for this component
                        let refs: Vec<&Int> = sum_terms.iter().collect();
                        result_terms.push(Int::add(&refs));
                    }
                }

                // Encode result using large primes (similar to matrix)
                // This encodes all components into a single Z3 integer
                let prime = Int::from_i64(1000003);
                let mut encoded = Int::from_i64(0);
                for term in result_terms.iter().rev() {
                    encoded = Int::add(&[&Int::mul(&[&encoded, &prime]), term]);
                }

                Ok(Some(encoded.into()))
            }
            _ => Ok(None),
        }
    }

    /// Expand tensor trace: Tr(T) = Σ_μ T_μμ
    fn try_expand_tensor_trace(
        &mut self,
        tensor: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Dynamic>, String> {
        if let Some((dim, elems)) = self.extract_tensor2_info(tensor) {
            // Trace = sum of diagonal elements
            let mut diag_terms: Vec<Int> = Vec::new();
            for i in 0..dim {
                let diag_idx = i * dim + i; // Element T_ii
                let elem_z3 = self.kleis_to_z3(&elems[diag_idx], vars)?;
                let elem_int = elem_z3
                    .as_int()
                    .ok_or("Tensor element must be integer/real")?;
                diag_terms.push(elem_int);
            }

            let refs: Vec<&Int> = diag_terms.iter().collect();
            Ok(Some(Int::add(&refs).into()))
        } else {
            Ok(None)
        }
    }

    /// Expand index lowering: V_μ = g_μν V^ν
    fn try_expand_index_lower(
        &mut self,
        metric: &Expression,
        vector: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Dynamic>, String> {
        let metric_info = self.extract_tensor2_info(metric);
        let vector_info = self.extract_vector_info(vector);

        match (metric_info, vector_info) {
            (Some((dim, g_elems)), Some((vdim, v_elems))) if dim == vdim => {
                // W_μ = g_μν V^ν = Σ_ν g_μν * V_ν
                let mut result_terms: Vec<Int> = Vec::with_capacity(dim);

                for mu in 0..dim {
                    let mut sum_terms: Vec<Int> = Vec::new();
                    #[allow(clippy::needless_range_loop)]
                    for nu in 0..dim {
                        let g_idx = mu * dim + nu;
                        let g_z3 = self.kleis_to_z3(&g_elems[g_idx], vars)?;
                        let v_z3 = self.kleis_to_z3(&v_elems[nu], vars)?;

                        let g_int = g_z3.as_int().ok_or("Metric element must be integer")?;
                        let v_int = v_z3.as_int().ok_or("Vector element must be integer")?;

                        sum_terms.push(Int::mul(&[&g_int, &v_int]));
                    }
                    let refs: Vec<&Int> = sum_terms.iter().collect();
                    result_terms.push(Int::add(&refs));
                }

                // Encode result vector
                let prime = Int::from_i64(1000003);
                let mut encoded = Int::from_i64(0);
                for term in result_terms.iter().rev() {
                    encoded = Int::add(&[&Int::mul(&[&encoded, &prime]), term]);
                }

                Ok(Some(encoded.into()))
            }
            _ => Ok(None),
        }
    }

    /// Expand index raising: V^μ = g^μν V_ν
    fn try_expand_index_raise(
        &mut self,
        inv_metric: &Expression,
        covector: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Dynamic>, String> {
        // Same logic as lower_index, just uses inverse metric
        self.try_expand_index_lower(inv_metric, covector, vars)
    }

    /// Expand tensor component access: component(T, i, j)
    fn try_expand_tensor_component(
        &mut self,
        tensor: &Expression,
        idx_i: &Expression,
        idx_j: &Expression,
        vars: &HashMap<String, Dynamic>,
    ) -> Result<Option<Dynamic>, String> {
        if let Some((dim, elems)) = self.extract_tensor2_info(tensor) {
            // Only expand if indices are concrete constants
            let i = match idx_i {
                Expression::Const(s) => s.parse::<usize>().ok(),
                _ => None,
            };
            let j = match idx_j {
                Expression::Const(s) => s.parse::<usize>().ok(),
                _ => None,
            };

            if let (Some(i_val), Some(j_val)) = (i, j) {
                if i_val < dim && j_val < dim {
                    let elem_idx = i_val * dim + j_val;
                    return self.kleis_to_z3(&elems[elem_idx], vars).map(Some);
                }
            }
        }
        Ok(None)
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
                // 1. Check quantified variables
                if let Some(var) = vars.get(name) {
                    return Ok(var.clone());
                }

                // 2. Check identity elements
                if let Some(identity) = self.identity_elements.get(name) {
                    return Ok(identity.clone());
                }

                // 3. Check already-created free variables
                if let Some(free_var) = self.free_variables.get(name) {
                    return Ok(free_var.clone());
                }

                // 4. Create fresh constant for this free variable
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

            Expression::Operation { name, args } => {
                // Special handling for Matrix - encode as integer for component-wise comparison
                if name == "Matrix" && args.len() == 3 {
                    if let Expression::List(items) = &args[2] {
                        let rows: usize = match &args[0] {
                            Expression::Const(s) => s.parse().unwrap_or(0),
                            _ => 0,
                        };
                        let cols: usize = match &args[1] {
                            Expression::Const(s) => s.parse().unwrap_or(0),
                            _ => 0,
                        };

                        // For 2x1 matrices, encode as: e0 * LARGE_PRIME + e1
                        // This enables component-wise equality comparison
                        if rows == 2 && cols == 1 && items.len() == 2 {
                            let e0 = self.kleis_to_z3(&items[0], vars)?;
                            let e1 = self.kleis_to_z3(&items[1], vars)?;
                            let e0_int = e0.as_int().ok_or("Matrix element must be integer")?;
                            let e1_int = e1.as_int().ok_or("Matrix element must be integer")?;

                            let large_prime = Int::from_i64(1000003);
                            let encoded = Int::add(&[&Int::mul(&[&e0_int, &large_prime]), &e1_int]);
                            return Ok(encoded.into());
                        }

                        // For other matrices, use uninterpreted function (structural equality only)
                        let matrix_name = format!("matrix_{}x{}", rows, cols);
                        let z3_elements: Result<Vec<_>, _> = items
                            .iter()
                            .map(|item| self.kleis_to_z3(item, vars))
                            .collect();
                        let z3_elements = z3_elements?;
                        let func_decl = self.declare_uninterpreted(&matrix_name, z3_elements.len());
                        let ast_args: Vec<&dyn Ast> =
                            z3_elements.iter().map(|d| d as &dyn Ast).collect();
                        return Ok(func_decl.apply(&ast_args));
                    }
                }

                // Special handling for matrix multiplication (inline expansion)
                // multiply(Matrix(2,2,[a,b,c,d]), Matrix(2,1,[x,y]))
                //   = Matrix(2,1,[a*x+b*y, c*x+d*y])
                if name == "multiply" && args.len() == 2 {
                    if let Some(result) =
                        self.try_expand_matrix_multiply(&args[0], &args[1], vars)?
                    {
                        return Ok(result);
                    }
                }

                // Tensor contraction expansion
                // contract(Tensor2(dim, components), Tensor2(dim, components))
                //   = Sum over contracted index
                if name == "contract" && args.len() == 2 {
                    if let Some(result) =
                        self.try_expand_tensor_contraction(&args[0], &args[1], vars)?
                    {
                        return Ok(result);
                    }
                }

                // Tensor trace expansion
                // trace(Tensor2(dim, components)) = sum of diagonal
                if name == "trace" && args.len() == 1 {
                    if let Some(result) = self.try_expand_tensor_trace(&args[0], vars)? {
                        return Ok(result);
                    }
                }

                // Index lowering with metric
                // lower_index(g, V) = g_μν V^ν (contraction)
                if name == "lower_index" && args.len() == 2 {
                    if let Some(result) = self.try_expand_index_lower(&args[0], &args[1], vars)? {
                        return Ok(result);
                    }
                }

                // Index raising with inverse metric
                // raise_index(g_inv, W) = g^μν W_ν (contraction)
                if name == "raise_index" && args.len() == 2 {
                    if let Some(result) = self.try_expand_index_raise(&args[0], &args[1], vars)? {
                        return Ok(result);
                    }
                }

                // Tensor component access
                // component(T, i, j) -> T_ij value
                if name == "component" && args.len() == 3 {
                    if let Some(result) =
                        self.try_expand_tensor_component(&args[0], &args[1], &args[2], vars)?
                    {
                        return Ok(result);
                    }
                }

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

            Expression::Let { name, value, body } => {
                // 1. Translate the value expression
                let z3_value = self.kleis_to_z3(value, vars)?;

                // 2. Extend vars with the new binding
                let mut extended_vars = vars.clone();
                extended_vars.insert(name.clone(), z3_value);

                // 3. Translate body with the extended context
                self.kleis_to_z3(body, &extended_vars)
            }

            Expression::Match { scrutinee, cases } => {
                // Translate match expression to nested ite
                self.translate_match(scrutinee, cases, vars)
            }

            Expression::List(items) => {
                // For now, lists are not directly translatable to Z3
                // They're typically used in matrix constructors
                Err(format!(
                    "List expressions not directly supported in Z3 (use in constructors): {:?}",
                    items.len()
                ))
            }

            _ => Err(format!("Unsupported expression type for Z3: {:?}", expr)),
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

            // Unknown operation - use uninterpreted function
            _ => {
                let func_decl = self.declare_uninterpreted(name, args.len());
                let ast_args: Vec<&dyn Ast> = args.iter().map(|d| d as &dyn Ast).collect();
                Ok(func_decl.apply(&ast_args))
            }
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
        // Use a temporary solver to check satisfiability and get a model
        let temp_solver = Solver::new();

        // For constant expressions, we can try to extract the value directly
        // For symbolic expressions, we need a model

        // Try to get concrete value directly
        if let Some(int_val) = z3_expr.as_int() {
            if let Some(value) = int_val.as_i64() {
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
        temp_solver.push();

        // Create a fresh variable and assert it equals our expression
        let result_var = Int::fresh_const("eval_result");

        // Try to cast z3_expr to Int and assert equality
        if let Some(int_expr) = z3_expr.as_int() {
            temp_solver.assert(result_var.eq(&int_expr));

            match temp_solver.check() {
                SatResult::Sat => {
                    if let Some(model) = temp_solver.get_model() {
                        if let Some(evaluated) = model.eval(&result_var, true) {
                            // Convert Z3 result to Kleis Expression using converter
                            let z3_dynamic: Dynamic = evaluated.into();
                            return self.converter.to_expression(&z3_dynamic);
                        }
                    }
                }
                _ => {
                    return Err("Cannot evaluate expression - no satisfying assignment".to_string())
                }
            }
        }

        temp_solver.pop(1);

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
            }),
            then_branch: Box::new(Expression::Const("42".to_string())),
            else_branch: Box::new(Expression::Const("0".to_string())),
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
            }),
            then_branch: Box::new(Expression::Const("42".to_string())),
            else_branch: Box::new(Expression::Const("0".to_string())),
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
            }),
            then_branch: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("10".to_string()),
                    Expression::Const("1".to_string()),
                ],
            }),
            else_branch: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("20".to_string()),
                    Expression::Const("1".to_string()),
                ],
            }),
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
            }),
            then_branch: Box::new(Expression::Const("100".to_string())),
            else_branch: Box::new(Expression::Conditional {
                condition: Box::new(Expression::Operation {
                    name: "greater_than".to_string(),
                    args: vec![
                        Expression::Const("2".to_string()),
                        Expression::Const("1".to_string()),
                    ],
                }),
                then_branch: Box::new(Expression::Const("200".to_string())),
                else_branch: Box::new(Expression::Const("300".to_string())),
            }),
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
            }),
            then_branch: Box::new(Expression::Const("5".to_string())),
            else_branch: Box::new(Expression::Const("10".to_string())),
        };

        let result = backend.simplify(&expr).unwrap();
        assert_eq!(result, Expression::Const("5".to_string()));
    }
}
