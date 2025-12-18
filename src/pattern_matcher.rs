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
use crate::data_registry::DataTypeRegistry;
use crate::type_inference::Type;
use std::collections::{HashMap, HashSet};

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
    #[allow(clippy::only_used_in_recursion)]
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

            // Grammar v0.8: As-pattern binds alias AND matches inner pattern
            Pattern::As {
                pattern: inner,
                binding,
            } => {
                // Bind the whole value to the alias
                bindings.insert(binding.clone(), value.clone());
                // Also match the inner pattern
                self.match_pattern_internal(value, inner, bindings)
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
                // Pattern matched! Now check guard if present (Grammar v0.8)
                if let Some(guard) = &case.guard {
                    // Substitute bindings into guard expression
                    let guard_with_bindings = self.substitute_bindings(guard, &bindings);
                    // Check if guard evaluates to true
                    // Guards must evaluate to boolean-like values
                    if !self.evaluate_guard(&guard_with_bindings) {
                        // Guard failed, try next case
                        continue;
                    }
                }
                // Pattern matched and guard (if any) passed!
                return Ok(self.substitute_bindings(&case.body, &bindings));
            }
        }

        // No case matched - non-exhaustive match at runtime
        Err("Non-exhaustive match: no pattern matched the scrutinee".to_string())
    }

    /// Evaluate a guard expression (Grammar v0.8)
    ///
    /// Guards are boolean expressions. This function checks if the guard is "truthy".
    /// Note: For full evaluation, this would need access to the Evaluator.
    /// Currently handles simple cases like True/False constructors.
    fn evaluate_guard(&self, guard: &Expression) -> bool {
        match guard {
            // Constructor True
            Expression::Operation { name, args } if name == "True" && args.is_empty() => true,
            Expression::Object(name) if name == "True" => true,
            // Constructor False
            Expression::Operation { name, args } if name == "False" && args.is_empty() => false,
            Expression::Object(name) if name == "False" => false,
            // Comparison results (from Z3 or evaluator)
            Expression::Const(s) if s == "true" || s == "True" => true,
            Expression::Const(s) if s == "false" || s == "False" => false,
            // For complex guards, we'd need to evaluate them
            // For now, assume they pass (conservative approach)
            // TODO: Full guard evaluation requires Evaluator integration
            _ => true,
        }
    }

    /// Substitute variable bindings into an expression
    ///
    /// Recursively traverse the expression and replace Object(name) with
    /// the bound value if it exists in bindings.
    #[allow(clippy::only_used_in_recursion)]
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
                // Substitute in scrutinee, guards, and case bodies
                let subst_scrutinee = Box::new(self.substitute_bindings(scrutinee, bindings));
                let subst_cases = cases
                    .iter()
                    .map(|case| MatchCase {
                        pattern: case.pattern.clone(), // Patterns don't substitute
                        guard: case
                            .guard
                            .as_ref()
                            .map(|g| self.substitute_bindings(g, bindings)),
                        body: self.substitute_bindings(&case.body, bindings),
                    })
                    .collect();
                Expression::Match {
                    scrutinee: subst_scrutinee,
                    cases: subst_cases,
                }
            }

            Expression::List(elements) => {
                // Substitute in all list elements
                let substituted_elements = elements
                    .iter()
                    .map(|elem| self.substitute_bindings(elem, bindings))
                    .collect();
                Expression::List(substituted_elements)
            }

            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
            } => {
                // Substitute in quantifier body and where clause
                Expression::Quantifier {
                    quantifier: quantifier.clone(),
                    variables: variables.clone(),
                    where_clause: where_clause
                        .as_ref()
                        .map(|w| Box::new(self.substitute_bindings(w, bindings))),
                    body: Box::new(self.substitute_bindings(body, bindings)),
                }
            }

            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => Expression::Conditional {
                condition: Box::new(self.substitute_bindings(condition, bindings)),
                then_branch: Box::new(self.substitute_bindings(then_branch, bindings)),
                else_branch: Box::new(self.substitute_bindings(else_branch, bindings)),
            },

            Expression::Let {
                pattern,
                type_annotation,
                value,
                body,
            } => {
                let subst_value = self.substitute_bindings(value, bindings);
                // Create new bindings without the shadowed variables
                let mut inner_bindings = bindings.clone();
                self.remove_pattern_vars(pattern, &mut inner_bindings);
                let subst_body = self.substitute_bindings(body, &inner_bindings);
                Expression::Let {
                    pattern: pattern.clone(),
                    type_annotation: type_annotation.clone(),
                    value: Box::new(subst_value),
                    body: Box::new(subst_body),
                }
            }

            Expression::Ascription {
                expr: inner,
                type_annotation,
            } => Expression::Ascription {
                expr: Box::new(self.substitute_bindings(inner, bindings)),
                type_annotation: type_annotation.clone(),
            },

            Expression::Lambda { params, body } => {
                // Filter out bindings that are shadowed by lambda params
                let shadowed: std::collections::HashSet<_> =
                    params.iter().map(|p| p.name.as_str()).collect();
                let filtered_bindings: std::collections::HashMap<_, _> = bindings
                    .iter()
                    .filter(|(k, _)| !shadowed.contains(k.as_str()))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                Expression::Lambda {
                    params: params.clone(),
                    body: Box::new(self.substitute_bindings(body, &filtered_bindings)),
                }
            }

            Expression::Placeholder { .. } | Expression::Const(_) => {
                // Leaves don't contain variables
                expr.clone()
            }
        }
    }

    /// Remove all variables bound by a pattern from a bindings map
    #[allow(clippy::only_used_in_recursion)]
    fn remove_pattern_vars(
        &self,
        pattern: &Pattern,
        bindings: &mut std::collections::HashMap<String, Expression>,
    ) {
        match pattern {
            Pattern::Variable(name) => {
                bindings.remove(name);
            }
            Pattern::Constructor { args, .. } => {
                for arg in args {
                    self.remove_pattern_vars(arg, bindings);
                }
            }
            // Grammar v0.8: As-pattern
            Pattern::As {
                pattern: inner,
                binding,
            } => {
                bindings.remove(binding);
                self.remove_pattern_vars(inner, bindings);
            }
            Pattern::Wildcard | Pattern::Constant(_) => {}
        }
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Exhaustiveness checker for pattern matching
///
/// Checks if patterns cover all possible cases for a data type and
/// detects unreachable patterns (patterns that can never match because
/// earlier patterns are more general).
///
/// ## Examples
///
/// ```ignore
/// // Exhaustive match - all Bool constructors covered
/// match x { True => 1 | False => 0 }
/// // ✅ OK
///
/// // Non-exhaustive - False missing
/// match x { True => 1 }
/// // ⚠️ Warning: Missing case: False
///
/// // Unreachable pattern - wildcard makes False unreachable
/// match x { True => 1 | _ => 0 | False => 2 }
/// //                              ^^^^^^^^ unreachable!
/// ```
pub struct ExhaustivenessChecker {
    data_registry: DataTypeRegistry,
}

impl ExhaustivenessChecker {
    /// Create a new exhaustiveness checker with the given data registry
    pub fn new(data_registry: DataTypeRegistry) -> Self {
        ExhaustivenessChecker { data_registry }
    }

    /// Check if patterns are exhaustive for a given type
    ///
    /// Returns Ok(()) if exhaustive, Err(missing_constructors) if not.
    ///
    /// A match is exhaustive if:
    /// - All constructors of the data type are covered, OR
    /// - There's a wildcard or variable pattern (catches all)
    ///
    /// ## Example
    ///
    /// ```ignore
    /// // Bool has constructors: True, False
    /// let patterns = vec![
    ///     Pattern::Constructor { name: "True".to_string(), args: vec![] },
    ///     Pattern::Constructor { name: "False".to_string(), args: vec![] },
    /// ];
    ///
    /// let result = checker.check_exhaustive(&patterns, &bool_type);
    /// assert!(result.is_ok()); // All cases covered
    /// ```
    pub fn check_exhaustive(
        &self,
        patterns: &[Pattern],
        scrutinee_ty: &Type,
    ) -> Result<(), Vec<String>> {
        match scrutinee_ty {
            Type::Data { type_name, .. } => {
                // Get all constructors for this type
                if let Some(data_def) = self.data_registry.get_type(type_name) {
                    let all_constructors: HashSet<_> =
                        data_def.variants.iter().map(|v| &v.name).collect();

                    // Get covered constructors from patterns
                    let mut covered = HashSet::new();
                    let mut has_wildcard = false;

                    for pattern in patterns {
                        self.collect_pattern_coverage(pattern, &mut covered, &mut has_wildcard);
                    }

                    // If wildcard exists, automatically exhaustive
                    if has_wildcard {
                        return Ok(());
                    }

                    // Check if all constructors are covered
                    let missing: Vec<_> = all_constructors
                        .difference(&covered)
                        .map(|s| s.to_string())
                        .collect();

                    if missing.is_empty() {
                        Ok(())
                    } else {
                        Err(missing)
                    }
                } else {
                    // Unknown type - can't check exhaustiveness
                    Ok(())
                }
            }

            // Other types (Scalar, Var, etc.) - can't enumerate cases
            _ => Ok(()),
        }
    }

    /// Helper to collect pattern coverage info (Grammar v0.8: handles As-patterns)
    #[allow(clippy::only_used_in_recursion)]
    fn collect_pattern_coverage<'a>(
        &self,
        pattern: &'a Pattern,
        covered: &mut HashSet<&'a String>,
        has_wildcard: &mut bool,
    ) {
        match pattern {
            Pattern::Wildcard | Pattern::Variable(_) => {
                *has_wildcard = true;
            }
            Pattern::Constructor { name, .. } => {
                covered.insert(name);
            }
            Pattern::Constant(_) => {
                // Constants don't contribute to constructor coverage
            }
            // Grammar v0.8: As-pattern - recurse into inner pattern
            Pattern::As { pattern: inner, .. } => {
                self.collect_pattern_coverage(inner, covered, has_wildcard);
            }
        }
    }

    /// Check for unreachable patterns
    ///
    /// Returns indices of patterns that can never match because they're
    /// subsumed by earlier patterns.
    ///
    /// A pattern is unreachable if an earlier pattern always matches when
    /// this pattern would match.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let patterns = vec![
    ///     Pattern::Constructor { name: "True".to_string(), args: vec![] },
    ///     Pattern::Wildcard,  // Catches everything else
    ///     Pattern::Constructor { name: "False".to_string(), args: vec![] },  // UNREACHABLE!
    /// ];
    ///
    /// let unreachable = checker.check_reachable(&patterns);
    /// assert_eq!(unreachable, vec![2]); // Pattern at index 2 is unreachable
    /// ```
    pub fn check_reachable(&self, patterns: &[Pattern]) -> Vec<usize> {
        let mut unreachable = Vec::new();

        for (i, pattern) in patterns.iter().enumerate() {
            // Check if this pattern is shadowed by earlier patterns
            if i > 0 && self.is_subsumed(pattern, &patterns[..i]) {
                unreachable.push(i);
            }
        }

        unreachable
    }

    /// Check if a pattern is subsumed by any earlier pattern
    fn is_subsumed(&self, pattern: &Pattern, earlier: &[Pattern]) -> bool {
        for earlier_pattern in earlier {
            if self.pattern_subsumes(earlier_pattern, pattern) {
                return true;
            }
        }
        false
    }

    /// Check if pattern1 subsumes pattern2
    ///
    /// Pattern p1 subsumes p2 if whenever p2 matches, p1 also matches.
    /// This makes p2 unreachable if p1 comes before it.
    ///
    /// Examples:
    /// - Wildcard subsumes everything
    /// - Variable subsumes everything
    /// - Some(x) subsumes Some(5)
    /// - Some(_) subsumes Some(x)
    #[allow(clippy::only_used_in_recursion)]
    fn pattern_subsumes(&self, p1: &Pattern, p2: &Pattern) -> bool {
        match (p1, p2) {
            // Wildcard/variable subsumes everything
            (Pattern::Wildcard, _) => true,
            (Pattern::Variable(_), _) => true,

            // Same constructor - check if sub-patterns are subsumed
            (
                Pattern::Constructor { name: n1, args: a1 },
                Pattern::Constructor { name: n2, args: a2 },
            ) if n1 == n2 => {
                // All sub-patterns must be subsumed
                if a1.len() != a2.len() {
                    return false;
                }
                a1.iter()
                    .zip(a2)
                    .all(|(p1, p2)| self.pattern_subsumes(p1, p2))
            }

            // Same constant
            (Pattern::Constant(c1), Pattern::Constant(c2)) => c1 == c2,

            _ => false,
        }
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

    // ===== Exhaustiveness Checking Tests =====

    #[test]
    fn test_exhaustiveness_all_covered() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};
        use crate::type_inference::Type;

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Bool".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "True".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "False".to_string(),
                        fields: vec![],
                    },
                ],
            })
            .unwrap();

        let checker = ExhaustivenessChecker::new(registry);
        let patterns = vec![
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
            Pattern::Constructor {
                name: "False".to_string(),
                args: vec![],
            },
        ];
        let ty = Type::Data {
            type_name: "Bool".to_string(),
            constructor: "Bool".to_string(),
            args: vec![],
        };

        let result = checker.check_exhaustive(&patterns, &ty);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exhaustiveness_wildcard() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};
        use crate::type_inference::Type;

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Bool".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "True".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "False".to_string(),
                        fields: vec![],
                    },
                ],
            })
            .unwrap();

        let checker = ExhaustivenessChecker::new(registry);
        let patterns = vec![
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
            Pattern::Wildcard,
        ];
        let ty = Type::Data {
            type_name: "Bool".to_string(),
            constructor: "Bool".to_string(),
            args: vec![],
        };

        let result = checker.check_exhaustive(&patterns, &ty);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exhaustiveness_variable() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};
        use crate::type_inference::Type;

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Bool".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "True".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "False".to_string(),
                        fields: vec![],
                    },
                ],
            })
            .unwrap();

        let checker = ExhaustivenessChecker::new(registry);
        let patterns = vec![Pattern::Variable("x".to_string())];
        let ty = Type::Data {
            type_name: "Bool".to_string(),
            constructor: "Bool".to_string(),
            args: vec![],
        };

        // Variable pattern catches everything
        let result = checker.check_exhaustive(&patterns, &ty);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exhaustiveness_missing_case() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};
        use crate::type_inference::Type;

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Bool".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "True".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "False".to_string(),
                        fields: vec![],
                    },
                ],
            })
            .unwrap();

        let checker = ExhaustivenessChecker::new(registry);
        let patterns = vec![Pattern::Constructor {
            name: "True".to_string(),
            args: vec![],
        }];
        let ty = Type::Data {
            type_name: "Bool".to_string(),
            constructor: "Bool".to_string(),
            args: vec![],
        };

        let result = checker.check_exhaustive(&patterns, &ty);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"False".to_string()));
    }

    #[test]
    fn test_exhaustiveness_multiple_missing() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};
        use crate::type_inference::Type;

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Status".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "Running".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Idle".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Paused".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Completed".to_string(),
                        fields: vec![],
                    },
                ],
            })
            .unwrap();

        let checker = ExhaustivenessChecker::new(registry);
        let patterns = vec![Pattern::Constructor {
            name: "Running".to_string(),
            args: vec![],
        }];
        let ty = Type::Data {
            type_name: "Status".to_string(),
            constructor: "Status".to_string(),
            args: vec![],
        };

        let result = checker.check_exhaustive(&patterns, &ty);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 3);
    }

    #[test]
    fn test_unreachable_after_wildcard() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let patterns = vec![
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
            Pattern::Wildcard,
            Pattern::Constructor {
                name: "False".to_string(),
                args: vec![],
            },
        ];

        let unreachable = checker.check_reachable(&patterns);
        assert_eq!(unreachable, vec![2]);
    }

    #[test]
    fn test_unreachable_after_variable() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let patterns = vec![
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
            Pattern::Variable("x".to_string()),
            Pattern::Constructor {
                name: "False".to_string(),
                args: vec![],
            },
        ];

        let unreachable = checker.check_reachable(&patterns);
        assert_eq!(unreachable, vec![2]);
    }

    #[test]
    fn test_unreachable_duplicate_constructor() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let patterns = vec![
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
        ];

        let unreachable = checker.check_reachable(&patterns);
        assert_eq!(unreachable, vec![1]);
    }

    #[test]
    fn test_reachable_different_constructors() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let patterns = vec![
            Pattern::Constructor {
                name: "True".to_string(),
                args: vec![],
            },
            Pattern::Constructor {
                name: "False".to_string(),
                args: vec![],
            },
        ];

        let unreachable = checker.check_reachable(&patterns);
        assert!(unreachable.is_empty());
    }

    #[test]
    fn test_pattern_subsumes_wildcard() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let p1 = Pattern::Wildcard;
        let p2 = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![],
        };

        assert!(checker.pattern_subsumes(&p1, &p2));
    }

    #[test]
    fn test_pattern_subsumes_variable() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let p1 = Pattern::Variable("x".to_string());
        let p2 = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![],
        };

        assert!(checker.pattern_subsumes(&p1, &p2));
    }

    #[test]
    fn test_pattern_subsumes_nested() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let p1 = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![Pattern::Wildcard],
        };
        let p2 = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![Pattern::Variable("x".to_string())],
        };

        assert!(checker.pattern_subsumes(&p1, &p2));
    }

    #[test]
    fn test_pattern_not_subsumes_different_constructor() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let p1 = Pattern::Constructor {
            name: "Some".to_string(),
            args: vec![],
        };
        let p2 = Pattern::Constructor {
            name: "None".to_string(),
            args: vec![],
        };

        assert!(!checker.pattern_subsumes(&p1, &p2));
    }

    #[test]
    fn test_exhaustiveness_non_data_type() {
        let checker = ExhaustivenessChecker::new(DataTypeRegistry::new());
        let patterns = vec![Pattern::Constant("0".to_string())];
        let ty = Type::scalar();

        // Can't check exhaustiveness for non-data types
        let result = checker.check_exhaustive(&patterns, &ty);
        assert!(result.is_ok());
    }

    // ==========================================================================
    // Grammar v0.8: Guard tests
    // ==========================================================================

    #[test]
    fn test_eval_match_with_guard_true() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Const("5".to_string());
        let cases = vec![
            MatchCase {
                pattern: Pattern::variable("n".to_string()),
                // Guard evaluates to true
                guard: Some(Expression::Object("True".to_string())),
                body: Expression::Const("matched".to_string()),
            },
            MatchCase::new(Pattern::Wildcard, Expression::Const("fallback".to_string())),
        ];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        assert_eq!(result, Expression::Const("matched".to_string()));
    }

    #[test]
    fn test_eval_match_with_guard_false() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Const("5".to_string());
        let cases = vec![
            MatchCase {
                pattern: Pattern::variable("n".to_string()),
                // Guard evaluates to false
                guard: Some(Expression::Object("False".to_string())),
                body: Expression::Const("matched".to_string()),
            },
            MatchCase::new(Pattern::Wildcard, Expression::Const("fallback".to_string())),
        ];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        // Should skip to fallback because guard was false
        assert_eq!(result, Expression::Const("fallback".to_string()));
    }

    #[test]
    fn test_eval_match_with_guard_const_true() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Const("5".to_string());
        let cases = vec![MatchCase {
            pattern: Pattern::variable("n".to_string()),
            // Guard as Const("true")
            guard: Some(Expression::Const("true".to_string())),
            body: Expression::Const("matched".to_string()),
        }];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        assert_eq!(result, Expression::Const("matched".to_string()));
    }

    #[test]
    fn test_eval_match_with_guard_const_false() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Const("5".to_string());
        let cases = vec![
            MatchCase {
                pattern: Pattern::variable("n".to_string()),
                guard: Some(Expression::Const("false".to_string())),
                body: Expression::Const("matched".to_string()),
            },
            MatchCase::new(Pattern::Wildcard, Expression::Const("fallback".to_string())),
        ];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        // Should skip to fallback because guard was false
        assert_eq!(result, Expression::Const("fallback".to_string()));
    }

    // ==========================================================================
    // Grammar v0.8: As-pattern tests
    // ==========================================================================

    #[test]
    fn test_match_as_pattern() {
        let matcher = PatternMatcher::new();
        // Cons(1, Nil) as whole
        let value = Expression::Operation {
            name: "Cons".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Operation {
                    name: "Nil".to_string(),
                    args: vec![],
                },
            ],
        };

        let pattern = Pattern::As {
            pattern: Box::new(Pattern::Constructor {
                name: "Cons".to_string(),
                args: vec![
                    Pattern::variable("h".to_string()),
                    Pattern::variable("t".to_string()),
                ],
            }),
            binding: "whole".to_string(),
        };

        let bindings = matcher.match_pattern(&value, &pattern).unwrap();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings.get("h"), Some(&Expression::Const("1".to_string())));
        assert!(bindings.get("t").is_some());
        // whole should be the entire value
        assert_eq!(bindings.get("whole"), Some(&value));
    }

    #[test]
    fn test_eval_match_with_as_pattern() {
        let matcher = PatternMatcher::new();
        let scrutinee = Expression::Operation {
            name: "Cons".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Operation {
                    name: "Nil".to_string(),
                    args: vec![],
                },
            ],
        };

        let cases = vec![MatchCase::new(
            Pattern::As {
                pattern: Box::new(Pattern::Constructor {
                    name: "Cons".to_string(),
                    args: vec![Pattern::variable("h".to_string()), Pattern::Wildcard],
                }),
                binding: "whole".to_string(),
            },
            // Body uses 'whole'
            Expression::Object("whole".to_string()),
        )];

        let result = matcher.eval_match(&scrutinee, &cases).unwrap();
        // Result should be the original scrutinee since body is 'whole'
        assert_eq!(result, scrutinee);
    }
}
