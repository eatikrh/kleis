///! Symbolic Evaluator for Kleis (Wire 3: Self-Hosting)
///!
///! This module provides symbolic evaluation of Kleis expressions, including
///! user-defined functions (`define` statements).
///!
///! **Key Concept**: Kleis is a symbolic math system, not a computational interpreter.
///! "Evaluation" means symbolic manipulation: substituting variables and simplifying expressions.
///!
///! ## Capabilities
///!
///! 1. **Function Storage**: Store function definitions as closures
///! 2. **Function Application**: Apply functions via symbolic substitution
///! 3. **Pattern Matching**: Delegate to PatternMatcher for match expressions
///!
///! ## Examples
///!
///! ```ignore
///! let mut eval = Evaluator::new();
///!
///! // Load function: define double(x) = x + x
///! eval.load_function("double", vec!["x"],
///!     Expression::Operation {
///!         name: "plus",
///!         args: vec![Object("x"), Object("x")]
///!     });
///!
///! // Apply: double(5)
///! let result = eval.apply_function("double", vec![Const("5")])?;
///! // result = plus(5, 5) (symbolic, not computed to 10)
///! ```
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
    bindings: HashMap<String, Expression>,

    /// Pattern matcher for match expressions
    matcher: PatternMatcher,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Evaluator {
            functions: HashMap::new(),
            bindings: HashMap::new(),
            matcher: PatternMatcher,
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

        // Substitute parameters in body and evaluate
        // This enables pattern matching execution in function bodies!
        let substituted = self.substitute(&closure.body, &subst);
        self.eval(&substituted)
    }

    /// Substitute variables in an expression
    ///
    /// Recursively traverses the expression tree and replaces Object(name)
    /// with the bound value from the substitution map.
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

            Expression::List(elements) => {
                // Substitute in list elements
                Expression::List(
                    elements
                        .iter()
                        .map(|elem| self.substitute(elem, subst))
                        .collect(),
                )
            }

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
                    // Built-in operation - evaluate arguments then simplify
                    let eval_args: Result<Vec<_>, _> =
                        args.iter().map(|arg| self.eval(arg)).collect();
                    let eval_args = eval_args?;

                    let expr = Expression::Operation {
                        name: name.clone(),
                        args: eval_args,
                    };
                    
                    // Apply simplification rules (ADR-002)
                    Ok(self.simplify(&expr))
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

            // Atoms - return as-is
            Expression::Const(_) | Expression::Object(_) | Expression::Placeholder { .. } => {
                Ok(expr.clone())
            }
        }
    }

    /// Simplify an expression using algebraic rules (ADR-002: Symbolic Simplification)
    ///
    /// This implements symbolic rewriting for matrix operations and other algebraic structures.
    pub fn simplify(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Operation { name, args } => {
                // First, recursively simplify arguments
                let simplified_args: Vec<Expression> =
                    args.iter().map(|arg| self.simplify(arg)).collect();

                // Then apply simplification rules
                match name.as_str() {
                    "plus" if simplified_args.len() == 2 => {
                        self.simplify_plus(&simplified_args[0], &simplified_args[1])
                    }
                    "times" if simplified_args.len() == 2 => {
                        self.simplify_times(&simplified_args[0], &simplified_args[1])
                    }
                    _ => Expression::Operation {
                        name: name.clone(),
                        args: simplified_args,
                    },
                }
            }
            Expression::List(elements) => {
                Expression::List(elements.iter().map(|e| self.simplify(e)).collect())
            }
            Expression::Match { scrutinee, cases } => {
                Expression::Match {
                    scrutinee: Box::new(self.simplify(scrutinee)),
                    cases: cases
                        .iter()
                        .map(|case| crate::ast::MatchCase {
                            pattern: case.pattern.clone(),
                            body: self.simplify(&case.body),
                        })
                        .collect(),
                }
            }
            _ => expr.clone(),
        }
    }

    /// Simplify matrix/vector addition: plus(Matrix(...), Matrix(...)) → Matrix with element-wise plus
    fn simplify_plus(&self, left: &Expression, right: &Expression) -> Expression {
        match (left, right) {
            (
                Expression::Operation {
                    name: left_name,
                    args: left_args,
                },
                Expression::Operation {
                    name: right_name,
                    args: right_args,
                },
            ) if (left_name == "Matrix" || left_name == "Vector")
                && left_name == right_name
                && self.check_matrix_dimensions_match(left_args, right_args, left_name) =>
            {
                self.elementwise_binary_op("plus", left_name, left_args, right_args)
            }
            _ => Expression::Operation {
                name: "plus".to_string(),
                args: vec![left.clone(), right.clone()],
            },
        }
    }

    /// Simplify multiplication: scalar * Matrix or Matrix * Matrix
    fn simplify_times(&self, left: &Expression, right: &Expression) -> Expression {
        // TODO: Implement scalar-matrix and matrix-matrix multiplication
        Expression::Operation {
            name: "times".to_string(),
            args: vec![left.clone(), right.clone()],
        }
    }

    /// Check if two matrices/vectors have matching dimensions
    fn check_matrix_dimensions_match(
        &self,
        left_args: &[Expression],
        right_args: &[Expression],
        type_name: &str,
    ) -> bool {
        match type_name {
            "Matrix" if left_args.len() == 3 && right_args.len() == 3 => {
                matches!(
                    (&left_args[0], &right_args[0], &left_args[1], &right_args[1]),
                    (Expression::Const(m1), Expression::Const(m2),
                     Expression::Const(n1), Expression::Const(n2))
                    if m1 == m2 && n1 == n2
                )
            }
            "Vector" if left_args.len() == 2 && right_args.len() == 2 => {
                matches!(
                    (&left_args[0], &right_args[0]),
                    (Expression::Const(n1), Expression::Const(n2)) if n1 == n2
                )
            }
            _ => false,
        }
    }

    /// Perform element-wise binary operation on matrices/vectors
    fn elementwise_binary_op(
        &self,
        op_name: &str,
        type_name: &str,
        left_args: &[Expression],
        right_args: &[Expression],
    ) -> Expression {
        let (left_elems, right_elems) = match type_name {
            "Matrix" => {
                if let (Expression::List(l), Expression::List(r)) = (&left_args[2], &right_args[2])
                {
                    (l, r)
                } else {
                    // No list format - can't simplify
                    return Expression::Operation {
                        name: op_name.to_string(),
                        args: vec![
                            Expression::Operation {
                                name: type_name.to_string(),
                                args: left_args.to_vec(),
                            },
                            Expression::Operation {
                                name: type_name.to_string(),
                                args: right_args.to_vec(),
                            },
                        ],
                    };
                }
            }
            "Vector" => {
                if let (Expression::List(l), Expression::List(r)) = (&left_args[1], &right_args[1])
                {
                    (l, r)
                } else {
                    return Expression::Operation {
                        name: op_name.to_string(),
                        args: vec![
                            Expression::Operation {
                                name: type_name.to_string(),
                                args: left_args.to_vec(),
                            },
                            Expression::Operation {
                                name: type_name.to_string(),
                                args: right_args.to_vec(),
                            },
                        ],
                    };
                }
            }
            _ => {
                return Expression::Operation {
                    name: op_name.to_string(),
                    args: vec![
                        Expression::Operation {
                            name: type_name.to_string(),
                            args: left_args.to_vec(),
                        },
                        Expression::Operation {
                            name: type_name.to_string(),
                            args: right_args.to_vec(),
                        },
                    ],
                }
            }
        };

        // Create element-wise operations
        let new_elements: Vec<Expression> = left_elems
            .iter()
            .zip(right_elems.iter())
            .map(|(l, r)| Expression::Operation {
                name: op_name.to_string(),
                args: vec![l.clone(), r.clone()],
            })
            .collect();

        // Build result Matrix/Vector
        if type_name == "Matrix" {
            Expression::Operation {
                name: "Matrix".to_string(),
                args: vec![
                    left_args[0].clone(),
                    left_args[1].clone(),
                    Expression::List(new_elements),
                ],
            }
        } else {
            Expression::Operation {
                name: "Vector".to_string(),
                args: vec![left_args[0].clone(), Expression::List(new_elements)],
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
