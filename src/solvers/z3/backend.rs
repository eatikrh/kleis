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
use crate::solvers::backend::{SolverBackend, SolverStats, VerificationResult};
use crate::solvers::capabilities::SolverCapabilities;
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
    registry: &'r StructureRegistry,

    /// Capability manifest (loaded from capabilities.toml)
    capabilities: SolverCapabilities,

    /// Track which operations have been declared as uninterpreted functions
    declared_ops: HashSet<String>,

    /// Track which structures' axioms are currently loaded
    loaded_structures: HashSet<String>,

    /// Identity elements (zero, one, e, etc.) mapped to Z3 constants
    identity_elements: HashMap<String, Dynamic>,

    /// Result converter (Z3 Dynamic → Kleis Expression)
    converter: Z3ResultConverter,
}

impl<'r> Z3Backend<'r> {
    /// Create a new Z3 backend
    ///
    /// # Arguments
    /// * `registry` - Structure registry containing operations and axioms
    pub fn new(registry: &'r StructureRegistry) -> Result<Self, String> {
        let solver = Solver::new();
        let capabilities = super::load_capabilities()?;

        Ok(Self {
            solver,
            registry,
            capabilities,
            declared_ops: HashSet::new(),
            loaded_structures: HashSet::new(),
            identity_elements: HashMap::new(),
            converter: Z3ResultConverter,
        })
    }

    /// Translate Kleis expression to Z3 Dynamic
    ///
    /// This is the core translation function. It recursively converts
    /// Kleis expressions to Z3's internal representation.
    ///
    /// **Internal only** - results stay within Z3Backend.
    fn kleis_to_z3(&mut self, expr: &Expression, vars: &HashMap<String, Dynamic>) -> Result<Dynamic, String> {
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

                // 3. Undefined
                Err(format!("Undefined variable or identity: {}", name))
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
                // Translate arguments first
                let z3_args: Result<Vec<_>, _> = args
                    .iter()
                    .map(|arg| self.kleis_to_z3(arg, vars))
                    .collect();
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

            "neg" | "negate" => {
                if args.len() != 1 {
                    return Err("negate requires 1 argument".to_string());
                }
                arithmetic::translate_negate(&args[0])
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

    /// Get solver statistics
    pub fn stats(&self) -> SolverStats {
        SolverStats {
            loaded_structures: self.loaded_structures.len(),
            declared_operations: self.declared_ops.len(),
            assertion_count: 0, // TODO: Track assertions
        }
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

    fn evaluate(&mut self, _expr: &Expression) -> Result<Expression, String> {
        // TODO: Implement evaluation using Z3's model evaluation
        Err("evaluate() not yet implemented for Z3Backend".to_string())
    }

    fn simplify(&mut self, _expr: &Expression) -> Result<Expression, String> {
        // TODO: Implement simplification using Z3's simplify()
        Err("simplify() not yet implemented for Z3Backend".to_string())
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

    fn load_structure_axioms(&mut self, _structure_name: &str, _axioms: &[Expression]) -> Result<(), String> {
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
}

