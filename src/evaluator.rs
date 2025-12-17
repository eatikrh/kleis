//! Symbolic Evaluator for Kleis (Wire 3: Self-Hosting)
//!
//! This module provides symbolic evaluation of Kleis expressions, including
//! user-defined functions (`define` statements).
//!
//! **Key Concept**: Kleis is a symbolic math system, not a computational interpreter.
//! "Evaluation" means symbolic manipulation: substituting variables and simplifying expressions.
//!
//! ## Capabilities
//!
//! 1. **Function Storage**: Store function definitions as closures
//! 2. **Function Application**: Apply functions via symbolic substitution
//! 3. **Pattern Matching**: Delegate to PatternMatcher for match expressions
//!
//! ## Examples
//!
//! ```ignore
//! let mut eval = Evaluator::new();
//!
//! // Load function: define double(x) = x + x
//! eval.load_function("double", vec!["x"],
//!     Expression::Operation {
//!         name: "plus",
//!         args: vec![Object("x"), Object("x")]
//!     });
//!
//! // Apply: double(5)
//! let result = eval.apply_function("double", vec![Const("5")])?;
//! // result = plus(5, 5) (symbolic, not computed to 10)
//! ```
use crate::ast::Expression;
use crate::kleis_ast::{FunctionDef, Program, TopLevel};
use crate::pattern_matcher::PatternMatcher;
use std::collections::HashMap;

/// Represents a user-defined function as a closure
#[derive(Debug, Clone)]
pub struct Closure {
    /// Parameter names
    pub params: Vec<String>,

    /// Function body (expression to evaluate)
    pub body: Expression,

    /// Captured environment (for closures - not used yet in Wire 3)
    pub env: HashMap<String, Expression>,
}

/// Symbolic evaluator for Kleis expressions
pub struct Evaluator {
    /// Loaded function definitions
    functions: HashMap<String, Closure>,

    /// Variable bindings (for evaluation context)
    /// Reserved for future use in evaluation context
    #[allow(dead_code)]
    bindings: HashMap<String, Expression>,

    /// Pattern matcher for match expressions
    matcher: PatternMatcher,

    /// ADT constructor names (nullary constructors like TCP, UDP, ICMP)
    /// These are values that should be recognized as constants, not variables
    adt_constructors: std::collections::HashSet<String>,

    /// Loaded data type definitions (for export)
    data_types: Vec<crate::kleis_ast::DataDef>,

    /// Loaded structure definitions (for export)
    structures: Vec<crate::kleis_ast::StructureDef>,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Evaluator {
            functions: HashMap::new(),
            bindings: HashMap::new(),
            matcher: PatternMatcher,
            adt_constructors: std::collections::HashSet::new(),
            data_types: Vec::new(),
            structures: Vec::new(),
        }
    }

    /// Load function definitions from a parsed program (Wire 3: Self-hosting)
    ///
    /// Processes `define` statements and stores them as closures.
    ///
    /// Example:
    /// ```ignore
    /// let code = "define double(x) = x + x";
    /// let program = parse_kleis_program(code)?;
    /// evaluator.load_program(&program)?;
    /// // Now 'double' is available for application
    /// ```
    pub fn load_program(&mut self, program: &Program) -> Result<(), String> {
        for item in &program.items {
            if let TopLevel::FunctionDef(func_def) = item {
                self.load_function_def(func_def)?;
            }
        }

        // Extract ADT constructor names and store data definitions
        for data_type in program.data_types() {
            // Store the full data definition for export
            self.data_types.push(data_type.clone());

            for variant in &data_type.variants {
                // Nullary constructors (no fields) are values/constants
                if variant.fields.is_empty() {
                    self.adt_constructors.insert(variant.name.clone());
                }
            }
        }

        // Store structure definitions for export
        for structure in program.structures() {
            self.structures.push(structure.clone());
        }

        Ok(())
    }

    /// Get the set of ADT constructor names (nullary constructors)
    pub fn get_adt_constructors(&self) -> &std::collections::HashSet<String> {
        &self.adt_constructors
    }

    /// Get all loaded data type definitions
    pub fn get_data_types(&self) -> &[crate::kleis_ast::DataDef] {
        &self.data_types
    }

    /// Get all loaded structure definitions
    pub fn get_structures(&self) -> &[crate::kleis_ast::StructureDef] {
        &self.structures
    }

    /// Load function definitions from structure members (Grammar v0.6)
    ///
    /// Processes `define` statements inside structures and makes them available
    /// for symbolic expansion.
    ///
    /// Example:
    /// ```ignore
    /// structure Ring(R) {
    ///   operation (-) : R × R → R
    ///   define (-)(x, y) = x + negate(y)
    /// }
    /// // Now (-) is available for expansion: a - b → a + negate(b)
    /// ```
    pub fn load_structure_functions(
        &mut self,
        structure: &crate::kleis_ast::StructureDef,
    ) -> Result<(), String> {
        self.load_structure_functions_recursive(&structure.members)
    }

    /// Recursively load functions from structure members
    fn load_structure_functions_recursive(
        &mut self,
        members: &[crate::kleis_ast::StructureMember],
    ) -> Result<(), String> {
        use crate::kleis_ast::StructureMember;

        for member in members {
            match member {
                StructureMember::FunctionDef(func_def) => {
                    // Load function for symbolic expansion
                    self.load_function_def(func_def)?;
                }
                StructureMember::NestedStructure { members, .. } => {
                    // Recursively load from nested structures
                    self.load_structure_functions_recursive(members)?;
                }
                _ => {
                    // Operation, Field, Axiom - not functions
                }
            }
        }
        Ok(())
    }

    /// Load a single function definition
    pub fn load_function_def(&mut self, func_def: &FunctionDef) -> Result<(), String> {
        let closure = Closure {
            params: func_def.params.clone(),
            body: func_def.body.clone(),
            env: HashMap::new(), // Empty environment for now
        };

        self.functions.insert(func_def.name.clone(), closure);
        Ok(())
    }

    /// Apply a user-defined function to arguments (symbolic substitution)
    ///
    /// This performs symbolic substitution: replaces parameters with arguments in the body.
    ///
    /// Example:
    /// ```ignore
    /// // Given: define double(x) = x + x
    /// // Call: double(5)
    /// // Result: 5 + 5 (symbolic)
    /// ```
    pub fn apply_function(&self, name: &str, args: Vec<Expression>) -> Result<Expression, String> {
        let closure = self
            .functions
            .get(name)
            .ok_or_else(|| format!("Function '{}' not defined", name))?;

        if args.len() != closure.params.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, got {}",
                name,
                closure.params.len(),
                args.len()
            ));
        }

        // Build substitution map: param_name -> argument_value
        let mut subst = HashMap::new();
        for (param, arg) in closure.params.iter().zip(args.iter()) {
            subst.insert(param.clone(), arg.clone());
        }

        // Substitute parameters in body
        Ok(self.substitute(&closure.body, &subst))
    }

    /// Substitute variables in an expression
    ///
    /// Recursively traverses the expression tree and replaces Object(name)
    /// with the bound value from the substitution map.
    #[allow(clippy::only_used_in_recursion)]
    fn substitute(&self, expr: &Expression, subst: &HashMap<String, Expression>) -> Expression {
        match expr {
            Expression::Object(name) => {
                // Replace with bound value if exists, otherwise keep as-is
                subst.get(name).cloned().unwrap_or_else(|| expr.clone())
            }

            Expression::Operation { name, args } => {
                // Recursively substitute in arguments
                Expression::Operation {
                    name: name.clone(),
                    args: args.iter().map(|arg| self.substitute(arg, subst)).collect(),
                }
            }

            Expression::Match { scrutinee, cases } => {
                // Substitute in scrutinee
                let new_scrutinee = Box::new(self.substitute(scrutinee, subst));

                // Substitute in each case body (patterns bind their own variables)
                let new_cases = cases
                    .iter()
                    .map(|case| crate::ast::MatchCase {
                        pattern: case.pattern.clone(),
                        body: self.substitute(&case.body, subst),
                    })
                    .collect();

                Expression::Match {
                    scrutinee: new_scrutinee,
                    cases: new_cases,
                }
            }

            // Quantifiers - substitute in body
            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
            } => Expression::Quantifier {
                quantifier: quantifier.clone(),
                variables: variables.clone(),
                where_clause: where_clause
                    .as_ref()
                    .map(|w| Box::new(self.substitute(w, subst))),
                body: Box::new(self.substitute(body, subst)),
            },

            Expression::List(elements) => {
                // Substitute in list elements
                Expression::List(
                    elements
                        .iter()
                        .map(|elem| self.substitute(elem, subst))
                        .collect(),
                )
            }

            // Conditionals - substitute in all branches
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => Expression::Conditional {
                condition: Box::new(self.substitute(condition, subst)),
                then_branch: Box::new(self.substitute(then_branch, subst)),
                else_branch: Box::new(self.substitute(else_branch, subst)),
            },

            // Let bindings - substitute in value and body
            // Note: the let-bound variable shadows any outer binding
            Expression::Let {
                name,
                type_annotation,
                value,
                body,
            } => {
                let subst_value = self.substitute(value, subst);
                // Create new substitution map without the shadowed variable
                let mut inner_subst = subst.clone();
                inner_subst.remove(name);
                let subst_body = self.substitute(body, &inner_subst);
                Expression::Let {
                    name: name.clone(),
                    type_annotation: type_annotation.clone(),
                    value: Box::new(subst_value),
                    body: Box::new(subst_body),
                }
            }

            // Type ascription - substitute in inner expression
            Expression::Ascription {
                expr: inner,
                type_annotation,
            } => Expression::Ascription {
                expr: Box::new(self.substitute(inner, subst)),
                type_annotation: type_annotation.clone(),
            },

            // Constants and placeholders don't change
            Expression::Const(_) | Expression::Placeholder { .. } => expr.clone(),
        }
    }

    /// Evaluate an expression (symbolic evaluation)
    ///
    /// This resolves function applications and match expressions symbolically.
    /// It does NOT perform arithmetic computation.
    pub fn eval(&self, expr: &Expression) -> Result<Expression, String> {
        match expr {
            // Check if this is a function application
            Expression::Operation { name, args } => {
                if self.functions.contains_key(name) {
                    // It's a user-defined function - apply it
                    let eval_args: Result<Vec<_>, _> =
                        args.iter().map(|arg| self.eval(arg)).collect();
                    let eval_args = eval_args?;

                    self.apply_function(name, eval_args)
                } else {
                    // Built-in operation - just evaluate arguments
                    let eval_args: Result<Vec<_>, _> =
                        args.iter().map(|arg| self.eval(arg)).collect();
                    let eval_args = eval_args?;

                    Ok(Expression::Operation {
                        name: name.clone(),
                        args: eval_args,
                    })
                }
            }

            // Match expressions - delegate to PatternMatcher
            Expression::Match { scrutinee, cases } => {
                let eval_scrutinee = self.eval(scrutinee)?;
                let result = self.matcher.eval_match(&eval_scrutinee, cases)?;
                self.eval(&result)
            }

            // Lists - evaluate elements
            Expression::List(elements) => {
                let eval_elements: Result<Vec<_>, _> =
                    elements.iter().map(|elem| self.eval(elem)).collect();
                Ok(Expression::List(eval_elements?))
            }

            // Quantifiers - not evaluated (used in axioms)
            Expression::Quantifier { .. } => {
                // Quantifiers are for axioms, not runtime evaluation
                Ok(expr.clone())
            }

            // Conditionals - evaluate condition and select branch
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let eval_cond = self.eval(condition)?;
                let eval_then = self.eval(then_branch)?;
                let eval_else = self.eval(else_branch)?;

                // Return as conditional (we don't evaluate the condition itself)
                // The actual branching is handled by Z3 or pattern matching
                Ok(Expression::Conditional {
                    condition: Box::new(eval_cond),
                    then_branch: Box::new(eval_then),
                    else_branch: Box::new(eval_else),
                })
            }

            // Let bindings - evaluate value and substitute into body
            Expression::Let {
                name, value, body, ..
            } => {
                // Evaluate the value
                let eval_value = self.eval(value)?;

                // Substitute value for name in body, then evaluate
                let mut subst = std::collections::HashMap::new();
                subst.insert(name.clone(), eval_value);
                let substituted_body = self.substitute(body, &subst);
                self.eval(&substituted_body)
            }

            // Type ascription - evaluate inner expression, discard type annotation
            // (type checking happens at type-check time, not evaluation time)
            Expression::Ascription { expr: inner, .. } => self.eval(inner),

            // Atoms - return as-is
            Expression::Const(_) | Expression::Object(_) | Expression::Placeholder { .. } => {
                Ok(expr.clone())
            }
        }
    }

    /// Get a function definition (for inspection/testing)
    pub fn get_function(&self, name: &str) -> Option<&Closure> {
        self.functions.get(name)
    }

    /// Check if a function is defined
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// List all defined function names
    pub fn list_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_parser::parse_kleis_program;

    #[test]
    fn test_load_simple_function() {
        let mut eval = Evaluator::new();

        let code = "define pi = 3.14159";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        assert!(eval.has_function("pi"));
    }

    #[test]
    fn test_load_function_with_params() {
        let mut eval = Evaluator::new();

        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        assert!(eval.has_function("double"));
        let closure = eval.get_function("double").unwrap();
        assert_eq!(closure.params.len(), 1);
        assert_eq!(closure.params[0], "x");
    }

    #[test]
    fn test_apply_function() {
        let mut eval = Evaluator::new();

        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Apply: double(5)
        let result = eval
            .apply_function("double", vec![Expression::Const("5".to_string())])
            .unwrap();

        // Should get: 5 + 5 (symbolic)
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expression::Const(ref s) if s == "5"));
                assert!(matches!(args[1], Expression::Const(ref s) if s == "5"));
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_apply_function_two_params() {
        let mut eval = Evaluator::new();

        let code = "define add(x, y) = x + y";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Apply: add(3, 7)
        let result = eval
            .apply_function(
                "add",
                vec![
                    Expression::Const("3".to_string()),
                    Expression::Const("7".to_string()),
                ],
            )
            .unwrap();

        // Should get: 3 + 7 (symbolic)
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_eval_function_application() {
        let mut eval = Evaluator::new();

        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Evaluate: double(5)
        let expr = Expression::Operation {
            name: "double".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };

        let result = eval.eval(&expr).unwrap();

        // Should get: 5 + 5
        match result {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "plus");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_load_multiple_functions() {
        let mut eval = Evaluator::new();

        let code = r#"
            define pi = 3.14159
            define double(x) = x + x
            define add(x, y) = x + y
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        assert!(eval.has_function("pi"));
        assert!(eval.has_function("double"));
        assert!(eval.has_function("add"));
    }

    #[test]
    fn test_function_not_found() {
        let eval = Evaluator::new();

        let result = eval.apply_function("undefined", vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not defined"));
    }

    #[test]
    fn test_function_wrong_arity() {
        let mut eval = Evaluator::new();

        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Try to call with wrong number of arguments
        let result = eval.apply_function(
            "double",
            vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 1 arguments, got 2"));
    }
}
