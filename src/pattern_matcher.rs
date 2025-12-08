//! Pattern Matching Evaluation for Kleis (ADR-021)
//!
//! This module provides runtime support for evaluating pattern matching expressions.
//! Since Kleis is a symbolic math system, "evaluation" means symbolic manipulation:
//! matching patterns against expressions and substituting bound variables.
//!
//! ## Key Operations
//!
//! 1. **Pattern Matching**: Try to match an expression against a pattern
//!    - Returns bindings if successful, None if pattern doesn't match
//!
//! 2. **Match Evaluation**: Evaluate a match expression
//!    - Try each case in order
//!    - Return body of first matching case with bindings substituted
//!
//! 3. **Binding Substitution**: Replace variables with their bound values
//!    - Recursively traverse expression tree
//!    - Replace Object(name) with bound value if exists
//!
//! ## Examples
//!
//! ```ignore
//! // match Some(5) { None => 0 | Some(x) => x }
//! let scrutinee = Expression::Operation {
//!     name: "Some".to_string(),
//!     args: vec![Expression::Const("5".to_string())],
//! };
//!
//! let cases = vec![
//!     MatchCase::new(
//!         Pattern::Constructor { name: "None".to_string(), args: vec![] },
//!         Expression::Const("0".to_string()),
//!     ),
//!     MatchCase::new(
//!         Pattern::Constructor {
//!             name: "Some".to_string(),
//!             args: vec![Pattern::Variable("x".to_string())],
//!         },
//!         Expression::Object("x".to_string()),
//!     ),
//! ];
//!
//! let matcher = PatternMatcher::new();
//! let result = matcher.eval_match(&scrutinee, &cases).unwrap();
//! // result = Expression::Const("5") (x was bound to 5 and substituted)
//! ```

use crate::ast::{Expression, MatchCase, Pattern};
use std::collections::HashMap;

/// Pattern matcher for evaluating pattern matching expressions
///
/// This performs symbolic pattern matching - matching expressions against
/// patterns and extracting variable bindings.
pub struct PatternMatcher;

impl PatternMatcher {
    /// Create a new pattern matcher
    pub fn new() -> Self {
        PatternMatcher
    }

    /// Try to match an expression against a pattern
    ///
    /// Returns Some(bindings) if the pattern matches, None otherwise.
    /// Bindings map variable names to their matched expressions.
    ///
    /// ## Examples
    ///
    /// ```ignore
    /// let value = Expression::Operation {
    ///     name: "Some".to_string(),
    ///     args: vec![Expression::Const("5".to_string())],
    /// };
    ///
    /// let pattern = Pattern::Constructor {
    ///     name: "Some".to_string(),
    ///     args: vec![Pattern::Variable("x".to_string())],
    /// };
    ///
    /// let bindings = matcher.match_pattern(&value, &pattern).unwrap();
    /// assert_eq!(bindings.get("x"), Some(&Expression::Const("5".to_string())));
    /// ```
    pub fn match_pattern(
        &self,
        value: &Expression,
        pattern: &Pattern,
    ) -> Option<HashMap<String, Expression>> {
        let mut bindings = HashMap::new();

        if self.match_pattern_internal(value, pattern, &mut bindings) {
            Some(bindings)
        } else {
            None
        }
    }

    /// Internal recursive pattern matching
    fn match_pattern_internal(
        &self,
        value: &Expression,
        pattern: &Pattern,
        bindings: &mut HashMap<String, Expression>,
    ) -> bool {
        match pattern {
            Pattern::Wildcard => {
                // Wildcard matches anything
                true
            }

            Pattern::Variable(name) => {
                // Variable matches anything and binds the value
                bindings.insert(name.clone(), value.clone());
                true
            }

            Pattern::Constructor { name, args } => {
                // Check if value is a matching constructor
                match value {
                    Expression::Operation {
                        name: value_name,
                        args: value_args,
                    } if value_name == name => {
                        // Check arity
                        if value_args.len() != args.len() {
                            return false;
                        }

                        // Recursively match arguments
                        for (val_arg, pat_arg) in value_args.iter().zip(args) {
                            if !self.match_pattern_internal(val_arg, pat_arg, bindings) {
                                return false;
                            }
                        }

                        true
                    }

                    Expression::Object(value_name) if value_name == name && args.is_empty() => {
                        // 0-arity constructor as object: True, False, None
                        true
                    }

                    _ => false,
                }
            }

            Pattern::Constant(pattern_value) => {
                // Constant must match exactly
                match value {
                    Expression::Const(value_str) => value_str == pattern_value,
                    _ => false,
                }
            }
        }
    }

    /// Evaluate a match expression
    ///
    /// Tries each case in order until one matches, then returns the body
    /// with variable bindings substituted.
    ///
    /// Returns an error if no case matches (non-exhaustive match).
    ///
    /// ## Example
    ///
    /// ```ignore
    /// // match True { True => 1 | False => 0 }
    /// let scrutinee = Expression::Object("True".to_string());
    /// let cases = vec![
    ///     MatchCase::new(
    ///         Pattern::Constructor { name: "True".to_string(), args: vec![] },
    ///         Expression::Const("1".to_string()),
    ///     ),
    ///     MatchCase::new(
    ///         Pattern::Constructor { name: "False".to_string(), args: vec![] },
    ///         Expression::Const("0".to_string()),
    ///     ),
    /// ];
    ///
    /// let result = matcher.eval_match(&scrutinee, &cases).unwrap();
    /// assert_eq!(result, Expression::Const("1".to_string()));
    /// ```
    pub fn eval_match(
        &self,
        scrutinee: &Expression,
        cases: &[MatchCase],
    ) -> Result<Expression, String> {
        // Try each case in order
        for case in cases {
            if let Some(bindings) = self.match_pattern(scrutinee, &case.pattern) {
                // Found a match! Substitute bindings into body
                return Ok(self.substitute_bindings(&case.body, &bindings));
            }
        }

        // No case matched - non-exhaustive match at runtime
        Err("Non-exhaustive match: no pattern matched the scrutinee".to_string())
    }

    /// Substitute variable bindings into an expression
    ///
    /// Recursively traverse the expression and replace Object(name) with
    /// the bound value if it exists in bindings.
    fn substitute_bindings(
        &self,
        expr: &Expression,
        bindings: &HashMap<String, Expression>,
    ) -> Expression {
        match expr {
            Expression::Object(name) => {
                // Replace with bound value if it exists
                if let Some(bound_value) = bindings.get(name) {
                    bound_value.clone()
                } else {
                    expr.clone()
                }
            }

            Expression::Operation { name, args } => {
                // Recursively substitute in arguments
                let substituted_args = args
                    .iter()
                    .map(|arg| self.substitute_bindings(arg, bindings))
                    .collect();
                Expression::operation(name.clone(), substituted_args)
            }

            Expression::Match { scrutinee, cases } => {
                // Substitute in scrutinee and case bodies
                let subst_scrutinee = Box::new(self.substitute_bindings(scrutinee, bindings));
                let subst_cases = cases
                    .iter()
                    .map(|case| MatchCase {
                        pattern: case.pattern.clone(), // Patterns don't substitute
                        body: self.substitute_bindings(&case.body, bindings),
                    })
                    .collect();
                Expression::Match {
                    scrutinee: subst_scrutinee,
                    cases: subst_cases,
                }
            }

            Expression::Placeholder { .. } | Expression::Const(_) => {
                // Leaves don't contain variables
                expr.clone()
            }
        }
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_wildcard() {
        let matcher = PatternMatcher::new();
        let value = Expression::Const("5".to_string());
        let pattern = Pattern::Wildcard;

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_match_variable() {
        let matcher = PatternMatcher::new();
        let value = Expression::Const("5".to_string());
        let pattern = Pattern::Variable("x".to_string());

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings.get("x"), Some(&Expression::Const("5".to_string())));
    }

    #[test]
    fn test_match_constant_success() {
        let matcher = PatternMatcher::new();
        let value = Expression::Const("5".to_string());
        let pattern = Pattern::Constant("5".to_string());

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_match_constant_failure() {
        let matcher = PatternMatcher::new();
        let value = Expression::Const("5".to_string());
        let pattern = Pattern::Constant("6".to_string());

        let bindings = matcher.match_pattern(&value, &pattern);
        assert!(bindings.is_none());
    }

    #[test]
    fn test_match_constructor_simple() {
        let matcher = PatternMatcher::new();
        let value = Expression::Object("True".to_string());
        let pattern = Pattern::Constructor {
            name: "True".to_string(),
            args: vec![],
        };

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_match_constructor_with_args() {
        let matcher = PatternMatcher::new();
        let value = Expression::Operation {
            name: "Some".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };
        let pattern = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![Pattern::Variable("x".to_string())],
        };

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings.get("x"), Some(&Expression::Const("5".to_string())));
    }

    #[test]
    fn test_match_constructor_wrong_name() {
        let matcher = PatternMatcher::new();
        let value = Expression::Operation {
            name: "Some".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };
        let pattern = Pattern::Constructor {
            name: "None".to_string(),
            args: vec![],
        };

        let bindings = matcher.match_pattern(&value, &pattern);
        assert!(bindings.is_none());
    }

    #[test]
    fn test_match_nested_pattern() {
        let matcher = PatternMatcher::new();
        let value = Expression::Operation {
            name: "Some".to_string(),
            args: vec![Expression::Operation {
                name: "Pair".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Const("2".to_string()),
                ],
            }],
        };
        let pattern = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![Pattern::Constructor {
                name: "Pair".to_string(),
                args: vec![
                    Pattern::Variable("a".to_string()),
                    Pattern::Variable("b".to_string()),
                ],
            }],
        };

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings.get("a"), Some(&Expression::Const("1".to_string())));
        assert_eq!(bindings.get("b"), Some(&Expression::Const("2".to_string())));
    }

    #[test]
    fn test_eval_match_simple() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Object("True".to_string());
        let cases = vec![
            MatchCase::new(
                Pattern::Constructor {
                    name: "True".to_string(),
                    args: vec![],
                },
                Expression::Const("1".to_string()),
            ),
            MatchCase::new(
                Pattern::Constructor {
                    name: "False".to_string(),
                    args: vec![],
                },
                Expression::Const("0".to_string()),
            ),
        ];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        assert_eq!(result, Expression::Const("1".to_string()));
    }

    #[test]
    fn test_eval_match_with_binding() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Operation {
            name: "Some".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };
        let cases = vec![
            MatchCase::new(
                Pattern::Constructor {
                    name: "None".to_string(),
                    args: vec![],
                },
                Expression::Const("0".to_string()),
            ),
            MatchCase::new(
                Pattern::Constructor {
                    name: "Some".to_string(),
                    args: vec![Pattern::Variable("x".to_string())],
                },
                Expression::Object("x".to_string()),
            ),
        ];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        assert_eq!(result, Expression::Const("5".to_string()));
    }

    #[test]
    fn test_eval_match_wildcard() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Object("Unknown".to_string());
        let cases = vec![
            MatchCase::new(
                Pattern::Constructor {
                    name: "Known".to_string(),
                    args: vec![],
                },
                Expression::Const("1".to_string()),
            ),
            MatchCase::new(Pattern::Wildcard, Expression::Const("0".to_string())),
        ];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        assert_eq!(result, Expression::Const("0".to_string()));
    }

    #[test]
    fn test_eval_match_non_exhaustive() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Object("Unknown".to_string());
        let cases = vec![MatchCase::new(
            Pattern::Constructor {
                name: "Known".to_string(),
                args: vec![],
            },
            Expression::Const("1".to_string()),
        )];

        let result = matcher.eval_match(&scrutinee, &cases);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Non-exhaustive"));
    }

    #[test]
    fn test_substitute_simple() {
        let matcher = PatternMatcher::new();
        let expr = Expression::Object("x".to_string());
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), Expression::Const("5".to_string()));

        let result = matcher.substitute_bindings(&expr, &bindings);
        assert_eq!(result, Expression::Const("5".to_string()));
    }

    #[test]
    fn test_substitute_in_operation() {
        let matcher = PatternMatcher::new();
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
        };
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), Expression::Const("3".to_string()));
        bindings.insert("y".to_string(), Expression::Const("4".to_string()));

        let result = matcher.substitute_bindings(&expr, &bindings);
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Expression::Const("3".to_string()));
                assert_eq!(args[1], Expression::Const("4".to_string()));
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_eval_match_multiple_variables() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Operation {
            name: "Pair".to_string(),
            args: vec![
                Expression::Const("3".to_string()),
                Expression::Const("4".to_string()),
            ],
        };
        let cases = vec![MatchCase::new(
            Pattern::Constructor {
                name: "Pair".to_string(),
                args: vec![
                    Pattern::Variable("a".to_string()),
                    Pattern::Variable("b".to_string()),
                ],
            },
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("a".to_string()),
                    Expression::Object("b".to_string()),
                ],
            },
        )];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert_eq!(args[0], Expression::Const("3".to_string()));
                assert_eq!(args[1], Expression::Const("4".to_string()));
            }
            _ => panic!("Expected Operation"),
        }
    }
}
