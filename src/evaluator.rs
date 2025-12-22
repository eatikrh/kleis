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
use crate::ast::{Expression, LambdaParam};
use crate::kleis_ast::{FunctionDef, Program, TopLevel};
use crate::pattern_matcher::PatternMatcher;
use std::collections::{HashMap, HashSet};

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

    /// All ADT constructor names (including those with fields like Some, Cons, Atom)
    /// Used by eval_concrete to recognize constructor calls as values
    all_constructors: std::collections::HashSet<String>,

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
            all_constructors: std::collections::HashSet::new(),
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
                // Track ALL constructors for eval_concrete
                self.all_constructors.insert(variant.name.clone());

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

                // Substitute in each case body and guard (patterns bind their own variables)
                let new_cases = cases
                    .iter()
                    .map(|case| crate::ast::MatchCase {
                        pattern: case.pattern.clone(),
                        guard: case.guard.as_ref().map(|g| self.substitute(g, subst)),
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
            // Note: the let-bound variable(s) shadow any outer binding
            Expression::Let {
                pattern,
                type_annotation,
                value,
                body,
            } => {
                let subst_value = self.substitute(value, subst);
                // Create new substitution map without the shadowed variables
                let mut inner_subst = subst.clone();
                // Remove all variables bound by the pattern
                self.remove_pattern_vars_from_subst(pattern, &mut inner_subst);
                let subst_body = self.substitute(body, &inner_subst);
                Expression::Let {
                    pattern: pattern.clone(),
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

            // Lambda - substitute in body, avoiding capture
            Expression::Lambda { params, body } => {
                // Filter out substitutions for variables that are shadowed by lambda params
                let shadowed: std::collections::HashSet<_> =
                    params.iter().map(|p| p.name.clone()).collect();
                let filtered_subst: std::collections::HashMap<_, _> = subst
                    .iter()
                    .filter(|(k, _)| !shadowed.contains(*k))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                Expression::Lambda {
                    params: params.clone(),
                    body: Box::new(self.substitute(body, &filtered_subst)),
                }
            }

            // Constants, strings, and placeholders don't change
            Expression::Const(_) | Expression::String(_) | Expression::Placeholder { .. } => {
                expr.clone()
            }
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
            // Grammar v0.8: supports pattern destructuring
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                // Evaluate the value
                let eval_value = self.eval(value)?;

                // Match pattern against value and collect bindings
                let mut subst = std::collections::HashMap::new();
                self.match_pattern_to_bindings(pattern, &eval_value, &mut subst)?;
                let substituted_body = self.substitute(body, &subst);
                self.eval(&substituted_body)
            }

            // Type ascription - evaluate inner expression, discard type annotation
            // (type checking happens at type-check time, not evaluation time)
            Expression::Ascription { expr: inner, .. } => self.eval(inner),

            // Lambda - return as a value (closures are values)
            Expression::Lambda { .. } => Ok(expr.clone()),

            // Atoms - return as-is
            Expression::Const(_)
            | Expression::String(_)
            | Expression::Object(_)
            | Expression::Placeholder { .. } => Ok(expr.clone()),
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

    // =========================================================================
    // Beta Reduction for Lambda Expressions
    // =========================================================================

    /// Default fuel limit for reduction (prevents infinite loops)
    const DEFAULT_REDUCTION_FUEL: usize = 1000;

    /// Perform beta reduction: (λ x . body)(arg) → body[x := arg]
    ///
    /// This is the core computational step in lambda calculus.
    /// It substitutes the argument for the bound variable in the lambda body.
    ///
    /// # Examples
    /// ```ignore
    /// // (λ x . x + 1)(5) → 5 + 1
    /// let lambda = Expression::Lambda { params: [x], body: x + 1 };
    /// let result = evaluator.beta_reduce(&lambda, &Expression::Const("5"))?;
    /// // result = Operation { name: "plus", args: [5, 1] }
    /// ```
    pub fn beta_reduce(&self, lambda: &Expression, arg: &Expression) -> Result<Expression, String> {
        match lambda {
            Expression::Lambda { params, body } => {
                if params.is_empty() {
                    // No params, return body as-is
                    return Ok((**body).clone());
                }

                let param = &params[0];

                // Check for potential variable capture and alpha-convert if needed
                let safe_body = self.alpha_convert_if_needed(body, &param.name, arg);

                // Build substitution map for first parameter
                let mut subst = HashMap::new();
                subst.insert(param.name.clone(), arg.clone());

                // Substitute param with arg in body
                let reduced_body = self.substitute(&safe_body, &subst);

                if params.len() == 1 {
                    // Fully applied single-param lambda
                    Ok(reduced_body)
                } else {
                    // Partial application - return new lambda with remaining params
                    Ok(Expression::Lambda {
                        params: params[1..].to_vec(),
                        body: Box::new(reduced_body),
                    })
                }
            }
            _ => Err(format!(
                "Cannot beta-reduce non-lambda expression: {:?}",
                lambda
            )),
        }
    }

    /// Beta reduce with multiple arguments (for multi-param lambdas or curried application)
    ///
    /// Applies arguments one at a time, handling partial application.
    pub fn beta_reduce_multi(
        &self,
        lambda: &Expression,
        args: &[Expression],
    ) -> Result<Expression, String> {
        let mut result = lambda.clone();

        for arg in args {
            result = self.beta_reduce(&result, arg)?;
        }

        Ok(result)
    }

    /// Reduce an expression to normal form with fuel limit
    ///
    /// This repeatedly applies beta reduction until no more redexes exist
    /// or the fuel runs out (preventing infinite loops).
    pub fn reduce_to_normal_form(&self, expr: &Expression) -> Result<Expression, String> {
        self.reduce_with_fuel(expr, Self::DEFAULT_REDUCTION_FUEL)
    }

    /// Reduce with explicit fuel limit
    pub fn reduce_with_fuel(&self, expr: &Expression, fuel: usize) -> Result<Expression, String> {
        if fuel == 0 {
            return Err(
                "Reduction limit exceeded (possible infinite loop or very complex expression)"
                    .to_string(),
            );
        }

        match self.reduction_step(expr)? {
            Some(reduced) => self.reduce_with_fuel(&reduced, fuel - 1),
            None => Ok(expr.clone()), // Normal form reached
        }
    }

    /// Perform a single reduction step (if possible)
    ///
    /// Returns Some(reduced) if a reduction was performed, None if in normal form.
    /// Uses normal order (leftmost-outermost) reduction strategy.
    fn reduction_step(&self, expr: &Expression) -> Result<Option<Expression>, String> {
        match expr {
            // Check for lambda application pattern in Operation
            // This handles: f(arg) where f resolves to a lambda
            Expression::Operation { name, args } => {
                // First, check if this is a named function that's actually a lambda
                if let Some(closure) = self.functions.get(name) {
                    // Check if the stored function body is a lambda
                    if matches!(closure.body, Expression::Lambda { .. })
                        && closure.params.is_empty()
                    {
                        // It's a lambda assigned to a name: define f = λ x . body
                        let lambda = &closure.body;
                        let result = self.beta_reduce_multi(lambda, args)?;
                        return Ok(Some(result));
                    }
                }

                // Try to reduce arguments (normal order: left to right)
                for (i, arg) in args.iter().enumerate() {
                    if let Some(reduced_arg) = self.reduction_step(arg)? {
                        let mut new_args = args.clone();
                        new_args[i] = reduced_arg;
                        return Ok(Some(Expression::Operation {
                            name: name.clone(),
                            args: new_args,
                        }));
                    }
                }

                Ok(None) // No reduction possible
            }

            // Lambda body reduction
            Expression::Lambda { params, body } => {
                if let Some(reduced_body) = self.reduction_step(body)? {
                    Ok(Some(Expression::Lambda {
                        params: params.clone(),
                        body: Box::new(reduced_body),
                    }))
                } else {
                    Ok(None)
                }
            }

            // Let bindings - reduce to substitution
            // Grammar v0.8: supports pattern destructuring
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                // Reduce value first
                if let Some(reduced_value) = self.reduction_step(value)? {
                    return Ok(Some(Expression::Let {
                        pattern: pattern.clone(),
                        type_annotation: None,
                        value: Box::new(reduced_value),
                        body: body.clone(),
                    }));
                }

                // Value is in normal form, perform pattern match and substitution
                let mut subst = HashMap::new();
                self.match_pattern_to_bindings(pattern, value, &mut subst)?;
                let result = self.substitute(body, &subst);
                Ok(Some(result))
            }

            // Conditionals - reduce condition, then branches
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                // Try to reduce condition first
                if let Some(reduced_cond) = self.reduction_step(condition)? {
                    return Ok(Some(Expression::Conditional {
                        condition: Box::new(reduced_cond),
                        then_branch: then_branch.clone(),
                        else_branch: else_branch.clone(),
                    }));
                }

                // Check if condition is a boolean constant
                match condition.as_ref() {
                    Expression::Object(s) if s == "True" || s == "true" => {
                        Ok(Some((**then_branch).clone()))
                    }
                    Expression::Object(s) if s == "False" || s == "false" => {
                        Ok(Some((**else_branch).clone()))
                    }
                    _ => {
                        // Reduce then branch
                        if let Some(reduced) = self.reduction_step(then_branch)? {
                            return Ok(Some(Expression::Conditional {
                                condition: condition.clone(),
                                then_branch: Box::new(reduced),
                                else_branch: else_branch.clone(),
                            }));
                        }
                        // Reduce else branch
                        if let Some(reduced) = self.reduction_step(else_branch)? {
                            return Ok(Some(Expression::Conditional {
                                condition: condition.clone(),
                                then_branch: then_branch.clone(),
                                else_branch: Box::new(reduced),
                            }));
                        }
                        Ok(None)
                    }
                }
            }

            // Ascription - reduce inner, discard type
            Expression::Ascription { expr: inner, .. } => {
                if let Some(reduced) = self.reduction_step(inner)? {
                    Ok(Some(reduced))
                } else {
                    // Already reduced, strip ascription
                    Ok(Some((**inner).clone()))
                }
            }

            // List - reduce elements
            Expression::List(elements) => {
                for (i, elem) in elements.iter().enumerate() {
                    if let Some(reduced) = self.reduction_step(elem)? {
                        let mut new_elements = elements.clone();
                        new_elements[i] = reduced;
                        return Ok(Some(Expression::List(new_elements)));
                    }
                }
                Ok(None)
            }

            // Atoms and quantifiers are already in normal form
            Expression::Const(_)
            | Expression::String(_)
            | Expression::Object(_)
            | Expression::Placeholder { .. }
            | Expression::Quantifier { .. }
            | Expression::Match { .. } => Ok(None),
        }
    }

    // =========================================================================
    // Alpha Conversion (Variable Capture Avoidance)
    // =========================================================================

    /// Check if substitution would cause variable capture and alpha-convert if needed
    ///
    /// Variable capture occurs when a free variable in the argument would become
    /// bound after substitution. For example:
    /// ```ignore
    /// (λ x . λ y . x + y)(y)
    /// // Naive substitution gives: λ y . y + y  (WRONG!)
    /// // The 'y' in the argument was captured by the inner λ y
    /// // Correct: α-convert first: λ z . y + z
    /// ```
    fn alpha_convert_if_needed(
        &self,
        body: &Expression,
        _param: &str,
        arg: &Expression,
    ) -> Expression {
        let free_in_arg = self.free_variables(arg);
        let bound_in_body = self.bound_variables(body);

        // Find variables that would be captured
        let captures: HashSet<_> = free_in_arg.intersection(&bound_in_body).cloned().collect();

        if captures.is_empty() {
            return body.clone();
        }

        // Alpha-convert: rename captured variables in body
        let mut result = body.clone();
        for captured in captures {
            let fresh = self.fresh_variable(&captured, &result, arg);
            result = self.alpha_convert(&result, &captured, &fresh);
        }

        result
    }

    /// Get all free variables in an expression
    fn free_variables(&self, expr: &Expression) -> HashSet<String> {
        let mut free = HashSet::new();
        self.collect_free_variables(expr, &mut HashSet::new(), &mut free);
        free
    }

    /// Helper to collect free variables, tracking bound variables
    fn collect_free_variables(
        &self,
        expr: &Expression,
        bound: &mut HashSet<String>,
        free: &mut HashSet<String>,
    ) {
        match expr {
            Expression::Object(name) => {
                if !bound.contains(name) {
                    free.insert(name.clone());
                }
            }
            Expression::Const(_) | Expression::String(_) | Expression::Placeholder { .. } => {}
            Expression::Operation { args, .. } => {
                for arg in args {
                    self.collect_free_variables(arg, bound, free);
                }
            }
            Expression::Lambda { params, body } => {
                let mut new_bound = bound.clone();
                for p in params {
                    new_bound.insert(p.name.clone());
                }
                self.collect_free_variables(body, &mut new_bound, free);
            }
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                self.collect_free_variables(value, bound, free);
                let mut new_bound = bound.clone();
                self.collect_pattern_vars(pattern, &mut new_bound);
                self.collect_free_variables(body, &mut new_bound, free);
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_free_variables(condition, bound, free);
                self.collect_free_variables(then_branch, bound, free);
                self.collect_free_variables(else_branch, bound, free);
            }
            Expression::Quantifier {
                variables,
                where_clause,
                body,
                ..
            } => {
                let mut new_bound = bound.clone();
                for v in variables {
                    new_bound.insert(v.name.clone()); // Extract name from QuantifiedVar
                }
                if let Some(w) = where_clause {
                    self.collect_free_variables(w, &mut new_bound, free);
                }
                self.collect_free_variables(body, &mut new_bound, free);
            }
            Expression::Match { scrutinee, cases } => {
                self.collect_free_variables(scrutinee, bound, free);
                for case in cases {
                    // Pattern variables are bound in the case body
                    let mut new_bound = bound.clone();
                    self.collect_pattern_vars_from_pattern(&case.pattern, &mut new_bound);
                    self.collect_free_variables(&case.body, &mut new_bound, free);
                }
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.collect_free_variables(elem, bound, free);
                }
            }
            Expression::Ascription { expr: inner, .. } => {
                self.collect_free_variables(inner, bound, free);
            }
        }
    }

    /// Collect variables bound by a Pattern
    #[allow(clippy::only_used_in_recursion)]
    fn collect_pattern_vars_from_pattern(
        &self,
        pattern: &crate::ast::Pattern,
        bound: &mut HashSet<String>,
    ) {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Variable(name) => {
                bound.insert(name.clone());
            }
            Pattern::Constructor { args, .. } => {
                for arg in args {
                    self.collect_pattern_vars_from_pattern(arg, bound);
                }
            }
            // Grammar v0.8: As-pattern binds the alias AND recurses into the pattern
            Pattern::As { pattern, binding } => {
                bound.insert(binding.clone());
                self.collect_pattern_vars_from_pattern(pattern, bound);
            }
            Pattern::Wildcard | Pattern::Constant(_) => {}
        }
    }

    /// Collect pattern variables into a HashSet (alias for collect_pattern_vars_from_pattern)
    fn collect_pattern_vars(&self, pattern: &crate::ast::Pattern, vars: &mut HashSet<String>) {
        self.collect_pattern_vars_from_pattern(pattern, vars);
    }

    /// Remove all variables bound by a pattern from a substitution map
    #[allow(clippy::only_used_in_recursion)]
    fn remove_pattern_vars_from_subst(
        &self,
        pattern: &crate::ast::Pattern,
        subst: &mut HashMap<String, Expression>,
    ) {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Variable(name) => {
                subst.remove(name);
            }
            Pattern::Constructor { args, .. } => {
                for arg in args {
                    self.remove_pattern_vars_from_subst(arg, subst);
                }
            }
            Pattern::As { pattern, binding } => {
                subst.remove(binding);
                self.remove_pattern_vars_from_subst(pattern, subst);
            }
            Pattern::Wildcard | Pattern::Constant(_) => {}
        }
    }

    /// Match a pattern against a value and collect variable bindings
    /// Grammar v0.8: Supports pattern destructuring in let bindings
    #[allow(clippy::only_used_in_recursion)]
    fn match_pattern_to_bindings(
        &self,
        pattern: &crate::ast::Pattern,
        value: &Expression,
        bindings: &mut HashMap<String, Expression>,
    ) -> Result<(), String> {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Variable(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            Pattern::Constant(c) => {
                if let Expression::Const(v) = value {
                    if c == v {
                        Ok(())
                    } else {
                        Err(format!("Pattern constant {} doesn't match value {}", c, v))
                    }
                } else {
                    Err(format!("Expected constant value for pattern {}", c))
                }
            }
            Pattern::Constructor { name, args } => {
                // Value should be a data constructor application
                if let Expression::Operation {
                    name: op_name,
                    args: op_args,
                } = value
                {
                    if name == op_name && args.len() == op_args.len() {
                        for (pat, val) in args.iter().zip(op_args.iter()) {
                            self.match_pattern_to_bindings(pat, val, bindings)?;
                        }
                        Ok(())
                    } else {
                        Err(format!(
                            "Constructor {} with {} args doesn't match {} with {} args",
                            name,
                            args.len(),
                            op_name,
                            op_args.len()
                        ))
                    }
                } else {
                    Err(format!(
                        "Expected constructor {} but got non-operation",
                        name
                    ))
                }
            }
            // Grammar v0.8: As-pattern binds the whole value AND destructures it
            Pattern::As {
                pattern: inner,
                binding,
            } => {
                // Bind the whole value to the alias
                bindings.insert(binding.clone(), value.clone());
                // Also destructure via the inner pattern
                self.match_pattern_to_bindings(inner, value, bindings)
            }
        }
    }

    /// Alpha-convert a pattern (rename variables)
    #[allow(clippy::only_used_in_recursion)]
    fn alpha_convert_pattern(
        &self,
        pattern: &crate::ast::Pattern,
        old_name: &str,
        new_name: &str,
    ) -> crate::ast::Pattern {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Variable(name) if name == old_name => Pattern::Variable(new_name.to_string()),
            Pattern::Variable(_) => pattern.clone(),
            Pattern::Constructor { name, args } => Pattern::Constructor {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|p| self.alpha_convert_pattern(p, old_name, new_name))
                    .collect(),
            },
            Pattern::Constant(_) | Pattern::Wildcard => pattern.clone(),
            // Grammar v0.8: As-pattern
            Pattern::As {
                pattern: inner,
                binding,
            } => Pattern::As {
                pattern: Box::new(self.alpha_convert_pattern(inner, old_name, new_name)),
                binding: if binding == old_name {
                    new_name.to_string()
                } else {
                    binding.clone()
                },
            },
        }
    }

    /// Get all bound variables in an expression
    fn bound_variables(&self, expr: &Expression) -> HashSet<String> {
        let mut bound = HashSet::new();
        self.collect_bound_variables(expr, &mut bound);
        bound
    }

    /// Helper to collect all bound variables
    fn collect_bound_variables(&self, expr: &Expression, bound: &mut HashSet<String>) {
        match expr {
            Expression::Lambda { params, body } => {
                for p in params {
                    bound.insert(p.name.clone());
                }
                self.collect_bound_variables(body, bound);
            }
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                self.collect_pattern_vars(pattern, bound);
                self.collect_bound_variables(value, bound);
                self.collect_bound_variables(body, bound);
            }
            Expression::Quantifier {
                variables,
                where_clause,
                body,
                ..
            } => {
                for v in variables {
                    bound.insert(v.name.clone()); // Extract name from QuantifiedVar
                }
                if let Some(w) = where_clause {
                    self.collect_bound_variables(w, bound);
                }
                self.collect_bound_variables(body, bound);
            }
            Expression::Operation { args, .. } => {
                for arg in args {
                    self.collect_bound_variables(arg, bound);
                }
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_bound_variables(condition, bound);
                self.collect_bound_variables(then_branch, bound);
                self.collect_bound_variables(else_branch, bound);
            }
            Expression::Match { scrutinee, cases } => {
                self.collect_bound_variables(scrutinee, bound);
                for case in cases {
                    self.collect_pattern_vars_from_pattern(&case.pattern, bound);
                    self.collect_bound_variables(&case.body, bound);
                }
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.collect_bound_variables(elem, bound);
                }
            }
            Expression::Ascription { expr: inner, .. } => {
                self.collect_bound_variables(inner, bound);
            }
            Expression::Const(_)
            | Expression::String(_)
            | Expression::Object(_)
            | Expression::Placeholder { .. } => {}
        }
    }

    /// Generate a fresh variable name that doesn't conflict
    fn fresh_variable(&self, base: &str, expr1: &Expression, expr2: &Expression) -> String {
        let mut all_vars = self.free_variables(expr1);
        all_vars.extend(self.free_variables(expr2));
        all_vars.extend(self.bound_variables(expr1));
        all_vars.extend(self.bound_variables(expr2));

        let mut candidate = format!("{}'", base);
        let mut counter = 1;
        while all_vars.contains(&candidate) {
            candidate = format!("{}'{}", base, counter);
            counter += 1;
        }
        candidate
    }

    /// Alpha-convert: rename all occurrences of a bound variable
    #[allow(clippy::only_used_in_recursion)]
    fn alpha_convert(&self, expr: &Expression, old_name: &str, new_name: &str) -> Expression {
        match expr {
            Expression::Lambda { params, body } => {
                // Check if this lambda binds the old name
                let binds_old = params.iter().any(|p| p.name == old_name);

                if binds_old {
                    // Rename the parameter and in the body
                    let new_params: Vec<LambdaParam> = params
                        .iter()
                        .map(|p| {
                            if p.name == old_name {
                                LambdaParam {
                                    name: new_name.to_string(),
                                    type_annotation: p.type_annotation.clone(),
                                }
                            } else {
                                p.clone()
                            }
                        })
                        .collect();
                    let new_body = self.alpha_convert(body, old_name, new_name);
                    Expression::Lambda {
                        params: new_params,
                        body: Box::new(new_body),
                    }
                } else {
                    // Just recurse into body
                    Expression::Lambda {
                        params: params.clone(),
                        body: Box::new(self.alpha_convert(body, old_name, new_name)),
                    }
                }
            }
            Expression::Object(name) if name == old_name => {
                Expression::Object(new_name.to_string())
            }
            Expression::Let {
                pattern,
                type_annotation,
                value,
                body,
            } => {
                let new_value = self.alpha_convert(value, old_name, new_name);
                // Alpha-convert variables in the pattern
                let new_pattern = self.alpha_convert_pattern(pattern, old_name, new_name);
                Expression::Let {
                    pattern: new_pattern,
                    type_annotation: type_annotation.clone(),
                    value: Box::new(new_value),
                    body: Box::new(self.alpha_convert(body, old_name, new_name)),
                }
            }
            Expression::Operation { name, args } => Expression::Operation {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| self.alpha_convert(a, old_name, new_name))
                    .collect(),
            },
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => Expression::Conditional {
                condition: Box::new(self.alpha_convert(condition, old_name, new_name)),
                then_branch: Box::new(self.alpha_convert(then_branch, old_name, new_name)),
                else_branch: Box::new(self.alpha_convert(else_branch, old_name, new_name)),
            },
            Expression::List(elements) => Expression::List(
                elements
                    .iter()
                    .map(|e| self.alpha_convert(e, old_name, new_name))
                    .collect(),
            ),
            Expression::Ascription {
                expr: inner,
                type_annotation,
            } => Expression::Ascription {
                expr: Box::new(self.alpha_convert(inner, old_name, new_name)),
                type_annotation: type_annotation.clone(),
            },
            // For other expressions, just clone (or handle similarly)
            _ => expr.clone(),
        }
    }

    // =========================================================================
    // Concrete Evaluation (for :eval command)
    // =========================================================================

    /// Evaluate an expression to a concrete value
    ///
    /// Unlike `eval` which is symbolic, this actually computes:
    /// - Arithmetic: 2 + 3 → 5
    /// - String operations: concat("a", "b") → "ab"
    /// - Conditionals: if true then x else y → x
    /// - Recursion: fib(5) → 5
    pub fn eval_concrete(&self, expr: &Expression) -> Result<Expression, String> {
        match expr {
            // Constants and strings are already values
            Expression::Const(s) => Ok(Expression::Const(s.clone())),
            Expression::String(s) => Ok(Expression::String(s.clone())),

            // Variables: check if they're defined functions (constants)
            Expression::Object(name) => {
                if let Some(closure) = self.functions.get(name) {
                    if closure.params.is_empty() {
                        // It's a constant (define pi = 3.14)
                        self.eval_concrete(&closure.body)
                    } else {
                        // It's a function, return as-is
                        Ok(expr.clone())
                    }
                } else if self.adt_constructors.contains(name) {
                    // It's a nullary constructor (True, False, None, etc.)
                    Ok(expr.clone())
                } else {
                    // Unbound variable
                    Ok(expr.clone())
                }
            }

            // Operations: evaluate args then apply built-in or user-defined function
            Expression::Operation { name, args } => {
                // First, evaluate all arguments
                // First, evaluate all arguments
                let eval_args: Result<Vec<_>, _> =
                    args.iter().map(|a| self.eval_concrete(a)).collect();
                let eval_args = eval_args?;

                // Check if this is a data constructor (e.g., Atom, List, Cons, Some)
                // Constructors are values - return them with evaluated args
                if self.all_constructors.contains(name) || self.is_constructor_name(name) {
                    return Ok(Expression::Operation {
                        name: name.clone(),
                        args: eval_args,
                    });
                }

                // Try built-in operations first
                if let Some(result) = self.apply_builtin(name, &eval_args)? {
                    return Ok(result);
                }

                // Try user-defined functions
                if self.functions.contains_key(name) {
                    let applied = self.apply_function(name, eval_args)?;
                    return self.eval_concrete(&applied);
                }

                // Unknown operation - return as-is with evaluated args
                Ok(Expression::Operation {
                    name: name.clone(),
                    args: eval_args,
                })
            }

            // Conditionals: evaluate condition and branch
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let eval_cond = self.eval_concrete(condition)?;
                match &eval_cond {
                    Expression::Object(s) if s == "true" || s == "True" => {
                        self.eval_concrete(then_branch)
                    }
                    Expression::Object(s) if s == "false" || s == "False" => {
                        self.eval_concrete(else_branch)
                    }
                    Expression::Const(s) if s == "true" || s == "True" => {
                        self.eval_concrete(then_branch)
                    }
                    Expression::Const(s) if s == "false" || s == "False" => {
                        self.eval_concrete(else_branch)
                    }
                    _ => {
                        // Condition didn't evaluate to a boolean - return symbolic
                        Ok(Expression::Conditional {
                            condition: Box::new(eval_cond),
                            then_branch: Box::new(self.eval_concrete(then_branch)?),
                            else_branch: Box::new(self.eval_concrete(else_branch)?),
                        })
                    }
                }
            }

            // Let bindings: evaluate value, bind, evaluate body
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                let eval_value = self.eval_concrete(value)?;
                let mut subst = HashMap::new();
                self.match_pattern_to_bindings(pattern, &eval_value, &mut subst)?;
                let substituted_body = self.substitute(body, &subst);
                self.eval_concrete(&substituted_body)
            }

            // Match expressions
            Expression::Match { scrutinee, cases } => {
                let eval_scrutinee = self.eval_concrete(scrutinee)?;
                let result = self.matcher.eval_match(&eval_scrutinee, cases)?;
                self.eval_concrete(&result)
            }

            // Lambda - return as value
            Expression::Lambda { .. } => Ok(expr.clone()),

            // Lists - evaluate elements
            Expression::List(elements) => {
                let eval_elements: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.eval_concrete(e)).collect();
                Ok(Expression::List(eval_elements?))
            }

            // Ascription - evaluate inner, discard type
            Expression::Ascription { expr: inner, .. } => self.eval_concrete(inner),

            // Quantifiers - not for concrete evaluation
            Expression::Quantifier { .. } => Ok(expr.clone()),

            // Placeholder - return as-is
            Expression::Placeholder { .. } => Ok(expr.clone()),
        }
    }

    /// Apply a built-in operation
    ///
    /// Returns Some(result) if the operation is built-in and all args are concrete,
    /// None if it should be handled by user-defined functions.
    fn apply_builtin(&self, name: &str, args: &[Expression]) -> Result<Option<Expression>, String> {
        match name {
            // === Arithmetic ===
            "plus" | "+" => self.builtin_arithmetic(args, |a, b| a + b),
            "minus" | "-" => self.builtin_arithmetic(args, |a, b| a - b),
            "times" | "*" | "mul" => self.builtin_arithmetic(args, |a, b| a * b),
            "divide" | "/" | "div" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(a), Some(b)) = (self.as_number(&args[0]), self.as_number(&args[1])) {
                    if b == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    Ok(Some(Expression::Const(format!("{}", a / b))))
                } else {
                    Ok(None)
                }
            }
            "mod" | "%" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(a), Some(b)) = (self.as_integer(&args[0]), self.as_integer(&args[1])) {
                    if b == 0 {
                        return Err("Modulo by zero".to_string());
                    }
                    Ok(Some(Expression::Const(format!("{}", a % b))))
                } else {
                    Ok(None)
                }
            }

            // === Comparison ===
            "eq" | "=" | "==" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                let result = self.values_equal(&args[0], &args[1]);
                Ok(Some(Expression::Object(
                    if result { "true" } else { "false" }.to_string(),
                )))
            }
            "neq" | "!=" | "≠" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                let result = !self.values_equal(&args[0], &args[1]);
                Ok(Some(Expression::Object(
                    if result { "true" } else { "false" }.to_string(),
                )))
            }
            "lt" | "<" => self.builtin_comparison(args, |a, b| a < b),
            "le" | "<=" | "≤" => self.builtin_comparison(args, |a, b| a <= b),
            "gt" | ">" => self.builtin_comparison(args, |a, b| a > b),
            "ge" | ">=" | "≥" => self.builtin_comparison(args, |a, b| a >= b),

            // === Boolean ===
            "and" | "∧" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                let a = self.as_bool(&args[0]);
                let b = self.as_bool(&args[1]);
                match (a, b) {
                    (Some(a), Some(b)) => Ok(Some(Expression::Object(
                        if a && b { "true" } else { "false" }.to_string(),
                    ))),
                    _ => Ok(None),
                }
            }
            "or" | "∨" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                let a = self.as_bool(&args[0]);
                let b = self.as_bool(&args[1]);
                match (a, b) {
                    (Some(a), Some(b)) => Ok(Some(Expression::Object(
                        if a || b { "true" } else { "false" }.to_string(),
                    ))),
                    _ => Ok(None),
                }
            }
            "not" | "¬" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(a) = self.as_bool(&args[0]) {
                    Ok(Some(Expression::Object(
                        if !a { "true" } else { "false" }.to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }

            // === String operations ===
            "concat" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(a), Some(b)) = (self.as_string(&args[0]), self.as_string(&args[1])) {
                    Ok(Some(Expression::String(format!("{}{}", a, b))))
                } else {
                    Ok(None)
                }
            }
            "strlen" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    Ok(Some(Expression::Const(format!("{}", s.len()))))
                } else {
                    Ok(None)
                }
            }
            "hasPrefix" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(s), Some(prefix)) =
                    (self.as_string(&args[0]), self.as_string(&args[1]))
                {
                    Ok(Some(Expression::Object(
                        if s.starts_with(&prefix) {
                            "true"
                        } else {
                            "false"
                        }
                        .to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }
            "hasSuffix" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(s), Some(suffix)) =
                    (self.as_string(&args[0]), self.as_string(&args[1]))
                {
                    Ok(Some(Expression::Object(
                        if s.ends_with(&suffix) {
                            "true"
                        } else {
                            "false"
                        }
                        .to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }
            "contains" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(s), Some(sub)) = (self.as_string(&args[0]), self.as_string(&args[1])) {
                    Ok(Some(Expression::Object(
                        if s.contains(&sub) { "true" } else { "false" }.to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }
            "indexOf" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(s), Some(sub)) = (self.as_string(&args[0]), self.as_string(&args[1])) {
                    let idx = s.find(&sub).map(|i| i as i64).unwrap_or(-1);
                    Ok(Some(Expression::Const(format!("{}", idx))))
                } else {
                    Ok(None)
                }
            }
            "substr" | "substring" => {
                if args.len() != 3 {
                    return Ok(None);
                }
                if let (Some(s), Some(start), Some(len)) = (
                    self.as_string(&args[0]),
                    self.as_integer(&args[1]),
                    self.as_integer(&args[2]),
                ) {
                    let start = start.max(0) as usize;
                    let len = len.max(0) as usize;
                    let result: String = s.chars().skip(start).take(len).collect();
                    Ok(Some(Expression::String(result)))
                } else {
                    Ok(None)
                }
            }
            "charAt" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(s), Some(idx)) = (self.as_string(&args[0]), self.as_integer(&args[1]))
                {
                    if idx >= 0 && (idx as usize) < s.len() {
                        let ch = s.chars().nth(idx as usize).unwrap();
                        Ok(Some(Expression::String(ch.to_string())))
                    } else {
                        Ok(Some(Expression::String(String::new())))
                    }
                } else {
                    Ok(None)
                }
            }
            "replace" => {
                if args.len() != 3 {
                    return Ok(None);
                }
                if let (Some(s), Some(from), Some(to)) = (
                    self.as_string(&args[0]),
                    self.as_string(&args[1]),
                    self.as_string(&args[2]),
                ) {
                    Ok(Some(Expression::String(s.replacen(&from, &to, 1))))
                } else {
                    Ok(None)
                }
            }
            "replaceAll" => {
                if args.len() != 3 {
                    return Ok(None);
                }
                if let (Some(s), Some(from), Some(to)) = (
                    self.as_string(&args[0]),
                    self.as_string(&args[1]),
                    self.as_string(&args[2]),
                ) {
                    Ok(Some(Expression::String(s.replace(&from, &to))))
                } else {
                    Ok(None)
                }
            }

            // === List operations ===
            "Cons" | "cons" => {
                // Cons(head, tail) - construct a list
                if args.len() != 2 {
                    return Ok(None);
                }
                Ok(Some(Expression::Operation {
                    name: "Cons".to_string(),
                    args: args.to_vec(),
                }))
            }
            "Nil" | "nil" => {
                // Nil - empty list
                Ok(Some(Expression::Object("Nil".to_string())))
            }
            "head" | "car" => {
                // head(Cons(h, t)) → h
                if args.len() != 1 {
                    return Ok(None);
                }
                match &args[0] {
                    Expression::Operation { name, args: inner }
                        if name == "Cons" && inner.len() == 2 =>
                    {
                        Ok(Some(inner[0].clone()))
                    }
                    Expression::List(elements) if !elements.is_empty() => {
                        Ok(Some(elements[0].clone()))
                    }
                    _ => Err("head: expected non-empty list".to_string()),
                }
            }
            "tail" | "cdr" => {
                // tail(Cons(h, t)) → t
                if args.len() != 1 {
                    return Ok(None);
                }
                match &args[0] {
                    Expression::Operation { name, args: inner }
                        if name == "Cons" && inner.len() == 2 =>
                    {
                        Ok(Some(inner[1].clone()))
                    }
                    Expression::List(elements) if !elements.is_empty() => {
                        Ok(Some(Expression::List(elements[1..].to_vec())))
                    }
                    _ => Err("tail: expected non-empty list".to_string()),
                }
            }
            "null?" | "isEmpty" | "isNil" => {
                // null?(list) → true if empty
                if args.len() != 1 {
                    return Ok(None);
                }
                let is_empty = match &args[0] {
                    Expression::Object(s) if s == "Nil" => true,
                    Expression::Operation { name, .. } if name == "Nil" => true,
                    Expression::List(elements) => elements.is_empty(),
                    Expression::Operation { name, .. } if name == "Cons" => false,
                    _ => return Ok(None),
                };
                Ok(Some(Expression::Object(
                    if is_empty { "true" } else { "false" }.to_string(),
                )))
            }
            "length" => {
                // length(list) → number of elements
                if args.len() != 1 {
                    return Ok(None);
                }
                match &args[0] {
                    Expression::List(elements) => {
                        Ok(Some(Expression::Const(format!("{}", elements.len()))))
                    }
                    Expression::Object(s) if s == "Nil" => {
                        Ok(Some(Expression::Const("0".to_string())))
                    }
                    Expression::Operation { name, args: inner } if name == "Cons" => {
                        // Count recursively: 1 + length(tail)
                        let tail_len = self.apply_builtin("length", &[inner[1].clone()])?;
                        if let Some(Expression::Const(n)) = tail_len {
                            let len: i64 = n.parse().unwrap_or(0);
                            Ok(Some(Expression::Const(format!("{}", len + 1))))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            }
            "nth" => {
                // nth(list, index) → element at index
                if args.len() != 2 {
                    return Ok(None);
                }
                let idx = self.as_integer(&args[1]);
                match (&args[0], idx) {
                    (Expression::List(elements), Some(i))
                        if i >= 0 && (i as usize) < elements.len() =>
                    {
                        Ok(Some(elements[i as usize].clone()))
                    }
                    (Expression::Operation { name, args: inner }, Some(0)) if name == "Cons" => {
                        Ok(Some(inner[0].clone()))
                    }
                    (Expression::Operation { name, args: inner }, Some(i))
                        if name == "Cons" && i > 0 =>
                    {
                        self.apply_builtin(
                            "nth",
                            &[inner[1].clone(), Expression::Const(format!("{}", i - 1))],
                        )
                    }
                    _ => Ok(None),
                }
            }

            // ============================================
            // MATRIX OPERATIONS (Concrete Evaluation)
            // ============================================
            "matrix_add" | "builtin_matrix_add" => {
                // Matrix addition: element-wise addition of two matrices
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m1, n1, elems1)), Some((m2, n2, elems2))) =
                    (self.extract_matrix(&args[0]), self.extract_matrix(&args[1]))
                {
                    if m1 != m2 || n1 != n2 {
                        return Err(format!(
                            "matrix_add: dimension mismatch: {}x{} vs {}x{}",
                            m1, n1, m2, n2
                        ));
                    }
                    let result: Result<Vec<Expression>, String> = elems1
                        .iter()
                        .zip(elems2.iter())
                        .map(|(a, b)| match (self.as_number(a), self.as_number(b)) {
                            (Some(x), Some(y)) => {
                                let sum = x + y;
                                if sum.fract() == 0.0 && sum.abs() < 1e15 {
                                    Ok(Expression::Const(format!("{}", sum as i64)))
                                } else {
                                    Ok(Expression::Const(format!("{}", sum)))
                                }
                            }
                            _ => Err("matrix_add: non-numeric element".to_string()),
                        })
                        .collect();
                    Ok(Some(self.make_matrix(m1, n1, result?)))
                } else {
                    Ok(None)
                }
            }

            "matrix_sub" | "builtin_matrix_sub" => {
                // Matrix subtraction: element-wise subtraction
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m1, n1, elems1)), Some((m2, n2, elems2))) =
                    (self.extract_matrix(&args[0]), self.extract_matrix(&args[1]))
                {
                    if m1 != m2 || n1 != n2 {
                        return Err(format!(
                            "matrix_sub: dimension mismatch: {}x{} vs {}x{}",
                            m1, n1, m2, n2
                        ));
                    }
                    let result: Result<Vec<Expression>, String> = elems1
                        .iter()
                        .zip(elems2.iter())
                        .map(|(a, b)| match (self.as_number(a), self.as_number(b)) {
                            (Some(x), Some(y)) => {
                                let diff = x - y;
                                if diff.fract() == 0.0 && diff.abs() < 1e15 {
                                    Ok(Expression::Const(format!("{}", diff as i64)))
                                } else {
                                    Ok(Expression::Const(format!("{}", diff)))
                                }
                            }
                            _ => Err("matrix_sub: non-numeric element".to_string()),
                        })
                        .collect();
                    Ok(Some(self.make_matrix(m1, n1, result?)))
                } else {
                    Ok(None)
                }
            }

            "multiply" | "builtin_matrix_mul" | "matmul" => {
                // Matrix multiplication: (m×n) · (n×p) → (m×p)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m1, n1, elems1)), Some((m2, n2, elems2))) =
                    (self.extract_matrix(&args[0]), self.extract_matrix(&args[1]))
                {
                    if n1 != m2 {
                        return Err(format!(
                            "matrix multiply: inner dimensions don't match: {}x{} vs {}x{}",
                            m1, n1, m2, n2
                        ));
                    }
                    // Compute C[i,j] = sum(A[i,k] * B[k,j] for k in 0..n1)
                    let mut result = Vec::with_capacity(m1 * n2);
                    for i in 0..m1 {
                        for j in 0..n2 {
                            let mut sum = 0.0;
                            for k in 0..n1 {
                                let a_val = self.as_number(&elems1[i * n1 + k]).unwrap_or(0.0);
                                let b_val = self.as_number(&elems2[k * n2 + j]).unwrap_or(0.0);
                                sum += a_val * b_val;
                            }
                            if sum.fract() == 0.0 && sum.abs() < 1e15 {
                                result.push(Expression::Const(format!("{}", sum as i64)));
                            } else {
                                result.push(Expression::Const(format!("{}", sum)));
                            }
                        }
                    }
                    Ok(Some(self.make_matrix(m1, n2, result)))
                } else {
                    Ok(None)
                }
            }

            "transpose" | "builtin_transpose" => {
                // Matrix transpose: (m×n) → (n×m)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    // Transpose: result[j,i] = original[i,j]
                    let mut result = Vec::with_capacity(m * n);
                    for j in 0..n {
                        for i in 0..m {
                            result.push(elems[i * n + j].clone());
                        }
                    }
                    Ok(Some(self.make_matrix(n, m, result)))
                } else {
                    Ok(None)
                }
            }

            "trace" | "builtin_trace" => {
                // Matrix trace: sum of diagonal elements (square matrices only)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    if m != n {
                        return Err(format!("trace: matrix must be square, got {}x{}", m, n));
                    }
                    let mut sum = 0.0;
                    for i in 0..m {
                        if let Some(val) = self.as_number(&elems[i * n + i]) {
                            sum += val;
                        } else {
                            return Err("trace: non-numeric diagonal element".to_string());
                        }
                    }
                    if sum.fract() == 0.0 && sum.abs() < 1e15 {
                        Ok(Some(Expression::Const(format!("{}", sum as i64))))
                    } else {
                        Ok(Some(Expression::Const(format!("{}", sum))))
                    }
                } else {
                    Ok(None)
                }
            }

            "det" | "builtin_determinant" => {
                // Matrix determinant (only 2x2 and 3x3 for now)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    if m != n {
                        return Err(format!("det: matrix must be square, got {}x{}", m, n));
                    }
                    let det = match m {
                        1 => self.as_number(&elems[0]).unwrap_or(0.0),
                        2 => {
                            // det([[a,b],[c,d]]) = ad - bc
                            let a = self.as_number(&elems[0]).unwrap_or(0.0);
                            let b = self.as_number(&elems[1]).unwrap_or(0.0);
                            let c = self.as_number(&elems[2]).unwrap_or(0.0);
                            let d = self.as_number(&elems[3]).unwrap_or(0.0);
                            a * d - b * c
                        }
                        3 => {
                            // Sarrus rule for 3x3
                            let a = |i: usize, j: usize| {
                                self.as_number(&elems[i * 3 + j]).unwrap_or(0.0)
                            };
                            a(0, 0) * (a(1, 1) * a(2, 2) - a(1, 2) * a(2, 1))
                                - a(0, 1) * (a(1, 0) * a(2, 2) - a(1, 2) * a(2, 0))
                                + a(0, 2) * (a(1, 0) * a(2, 1) - a(1, 1) * a(2, 0))
                        }
                        _ => {
                            return Err(format!(
                                "det: only 1x1, 2x2, 3x3 supported, got {}x{}",
                                m, n
                            ))
                        }
                    };
                    if det.fract() == 0.0 && det.abs() < 1e15 {
                        Ok(Some(Expression::Const(format!("{}", det as i64))))
                    } else {
                        Ok(Some(Expression::Const(format!("{}", det))))
                    }
                } else {
                    Ok(None)
                }
            }

            "scalar_matrix_mul" | "builtin_matrix_scalar_mul" => {
                // Scalar * Matrix: multiply all elements by scalar
                if args.len() != 2 {
                    return Ok(None);
                }
                // Try both orders: scalar * matrix or matrix * scalar
                let (scalar, matrix) = if let Some(s) = self.as_number(&args[0]) {
                    if let Some(mat) = self.extract_matrix(&args[1]) {
                        (s, mat)
                    } else {
                        return Ok(None);
                    }
                } else if let Some(s) = self.as_number(&args[1]) {
                    if let Some(mat) = self.extract_matrix(&args[0]) {
                        (s, mat)
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                };

                let (m, n, elems) = matrix;
                let result: Result<Vec<Expression>, String> = elems
                    .iter()
                    .map(|e| {
                        if let Some(val) = self.as_number(e) {
                            let product = scalar * val;
                            if product.fract() == 0.0 && product.abs() < 1e15 {
                                Ok(Expression::Const(format!("{}", product as i64)))
                            } else {
                                Ok(Expression::Const(format!("{}", product)))
                            }
                        } else {
                            Err("scalar_matrix_mul: non-numeric element".to_string())
                        }
                    })
                    .collect();
                Ok(Some(self.make_matrix(m, n, result?)))
            }

            // Not a built-in
            _ => Ok(None),
        }
    }

    /// Check if a name looks like a constructor (starts with uppercase)
    fn is_constructor_name(&self, name: &str) -> bool {
        name.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    // === Helper methods for built-in operations ===

    fn builtin_arithmetic<F>(
        &self,
        args: &[Expression],
        op: F,
    ) -> Result<Option<Expression>, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        if args.len() != 2 {
            return Ok(None);
        }
        if let (Some(a), Some(b)) = (self.as_number(&args[0]), self.as_number(&args[1])) {
            let result = op(a, b);
            // Format nicely: integers without decimal point
            if result.fract() == 0.0 && result.abs() < 1e15 {
                Ok(Some(Expression::Const(format!("{}", result as i64))))
            } else {
                Ok(Some(Expression::Const(format!("{}", result))))
            }
        } else {
            Ok(None)
        }
    }

    fn builtin_comparison<F>(
        &self,
        args: &[Expression],
        op: F,
    ) -> Result<Option<Expression>, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        if args.len() != 2 {
            return Ok(None);
        }
        if let (Some(a), Some(b)) = (self.as_number(&args[0]), self.as_number(&args[1])) {
            Ok(Some(Expression::Object(
                if op(a, b) { "true" } else { "false" }.to_string(),
            )))
        } else {
            Ok(None)
        }
    }

    fn as_number(&self, expr: &Expression) -> Option<f64> {
        match expr {
            Expression::Const(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn as_integer(&self, expr: &Expression) -> Option<i64> {
        match expr {
            Expression::Const(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn as_string(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::String(s) => Some(s.clone()),
            Expression::Const(s) => Some(s.clone()), // Also accept const as string
            _ => None,
        }
    }

    fn as_bool(&self, expr: &Expression) -> Option<bool> {
        match expr {
            Expression::Object(s) => match s.as_str() {
                "true" | "True" => Some(true),
                "false" | "False" => Some(false),
                _ => None,
            },
            Expression::Const(s) => match s.as_str() {
                "true" | "True" => Some(true),
                "false" | "False" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    fn values_equal(&self, a: &Expression, b: &Expression) -> bool {
        match (a, b) {
            (Expression::Const(x), Expression::Const(y)) => x == y,
            (Expression::String(x), Expression::String(y)) => x == y,
            (Expression::Object(x), Expression::Object(y)) => x == y,
            (Expression::Const(x), Expression::String(y)) => x == y,
            (Expression::String(x), Expression::Const(y)) => x == y,
            _ => false,
        }
    }

    // === Matrix helper methods ===

    /// Extract (rows, cols, elements) from a Matrix expression
    /// Handles: Matrix(m, n, [elements]) or Matrix(m, n, List([elements]))
    fn extract_matrix(&self, expr: &Expression) -> Option<(usize, usize, Vec<Expression>)> {
        match expr {
            Expression::Operation { name, args } if name == "Matrix" && args.len() >= 3 => {
                // Matrix(m, n, elements)
                let m = self.as_integer(&args[0])? as usize;
                let n = self.as_integer(&args[1])? as usize;

                // Elements can be a List or inline elements
                let elements = match &args[2] {
                    Expression::List(elems) => elems.clone(),
                    Expression::Operation {
                        name: list_name,
                        args: list_args,
                    } if list_name == "List" => list_args.clone(),
                    // If more than 3 args, elements are inline (old format)
                    _ if args.len() > 3 => args[2..].to_vec(),
                    // Single element matrix
                    other => vec![other.clone()],
                };

                if elements.len() == m * n {
                    Some((m, n, elements))
                } else {
                    None // Element count doesn't match dimensions
                }
            }
            _ => None,
        }
    }

    /// Create a Matrix expression from dimensions and elements
    fn make_matrix(&self, m: usize, n: usize, elements: Vec<Expression>) -> Expression {
        Expression::Operation {
            name: "Matrix".to_string(),
            args: vec![
                Expression::Const(format!("{}", m)),
                Expression::Const(format!("{}", n)),
                Expression::List(elements),
            ],
        }
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

    // =========================================================================
    // Beta Reduction Tests
    // =========================================================================

    #[test]
    fn test_beta_reduce_identity() {
        // (λ x . x)(5) → 5
        let eval = Evaluator::new();

        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x")],
            body: Box::new(Expression::Object("x".to_string())),
        };

        let result = eval
            .beta_reduce(&lambda, &Expression::Const("5".to_string()))
            .unwrap();

        assert!(matches!(result, Expression::Const(ref s) if s == "5"));
    }

    #[test]
    fn test_beta_reduce_simple_arithmetic() {
        // (λ x . x + 1)(5) → 5 + 1
        let eval = Evaluator::new();

        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x")],
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Const("1".to_string()),
                ],
            }),
        };

        let result = eval
            .beta_reduce(&lambda, &Expression::Const("5".to_string()))
            .unwrap();

        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert!(matches!(args[0], Expression::Const(ref s) if s == "5"));
                assert!(matches!(args[1], Expression::Const(ref s) if s == "1"));
            }
            _ => panic!("Expected Operation, got {:?}", result),
        }
    }

    #[test]
    fn test_beta_reduce_partial_application() {
        // (λ x y . x + y)(3) → λ y . 3 + y
        let eval = Evaluator::new();

        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x"), LambdaParam::new("y")],
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        };

        let result = eval
            .beta_reduce(&lambda, &Expression::Const("3".to_string()))
            .unwrap();

        // Should be λ y . 3 + y
        match result {
            Expression::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "y");

                match *body {
                    Expression::Operation { name, ref args } => {
                        assert_eq!(name, "plus");
                        assert!(matches!(args[0], Expression::Const(ref s) if s == "3"));
                        assert!(matches!(args[1], Expression::Object(ref s) if s == "y"));
                    }
                    _ => panic!("Expected Operation in body"),
                }
            }
            _ => panic!("Expected Lambda, got {:?}", result),
        }
    }

    #[test]
    fn test_beta_reduce_full_application() {
        // (λ x y . x + y)(3)(4) → 7 (symbolically: 3 + 4)
        let eval = Evaluator::new();

        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x"), LambdaParam::new("y")],
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        };

        // Apply first argument
        let partial = eval
            .beta_reduce(&lambda, &Expression::Const("3".to_string()))
            .unwrap();

        // Apply second argument
        let result = eval
            .beta_reduce(&partial, &Expression::Const("4".to_string()))
            .unwrap();

        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert!(matches!(args[0], Expression::Const(ref s) if s == "3"));
                assert!(matches!(args[1], Expression::Const(ref s) if s == "4"));
            }
            _ => panic!("Expected Operation, got {:?}", result),
        }
    }

    #[test]
    fn test_beta_reduce_multi() {
        // Apply multiple args at once: (λ x y . x * y)(2, 3) → 2 * 3
        let eval = Evaluator::new();

        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x"), LambdaParam::new("y")],
            body: Box::new(Expression::Operation {
                name: "times".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        };

        let result = eval
            .beta_reduce_multi(
                &lambda,
                &[
                    Expression::Const("2".to_string()),
                    Expression::Const("3".to_string()),
                ],
            )
            .unwrap();

        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "times");
                assert!(matches!(args[0], Expression::Const(ref s) if s == "2"));
                assert!(matches!(args[1], Expression::Const(ref s) if s == "3"));
            }
            _ => panic!("Expected Operation, got {:?}", result),
        }
    }

    #[test]
    fn test_free_variables() {
        let eval = Evaluator::new();

        // x + y has free variables x, y
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
        };

        let free = eval.free_variables(&expr);
        assert!(free.contains("x"));
        assert!(free.contains("y"));
        assert_eq!(free.len(), 2);
    }

    #[test]
    fn test_free_variables_in_lambda() {
        let eval = Evaluator::new();

        // λ x . x + y has only y free (x is bound)
        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x")],
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        };

        let free = eval.free_variables(&lambda);
        assert!(!free.contains("x")); // x is bound
        assert!(free.contains("y")); // y is free
        assert_eq!(free.len(), 1);
    }

    #[test]
    fn test_bound_variables() {
        let eval = Evaluator::new();

        // λ x . λ y . x + y has bound variables x, y
        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x")],
            body: Box::new(Expression::Lambda {
                params: vec![LambdaParam::new("y")],
                body: Box::new(Expression::Operation {
                    name: "plus".to_string(),
                    args: vec![
                        Expression::Object("x".to_string()),
                        Expression::Object("y".to_string()),
                    ],
                }),
            }),
        };

        let bound = eval.bound_variables(&lambda);
        assert!(bound.contains("x"));
        assert!(bound.contains("y"));
    }

    #[test]
    fn test_alpha_conversion() {
        let eval = Evaluator::new();

        // λ y . y → λ y' . y' (when we need to rename y)
        let lambda = Expression::Lambda {
            params: vec![LambdaParam::new("y")],
            body: Box::new(Expression::Object("y".to_string())),
        };

        let converted = eval.alpha_convert(&lambda, "y", "z");

        match converted {
            Expression::Lambda { params, body } => {
                assert_eq!(params[0].name, "z");
                assert!(matches!(*body, Expression::Object(ref s) if s == "z"));
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_variable_capture_avoidance() {
        let eval = Evaluator::new();

        // (λ x . λ y . x + y)(y) should NOT produce λ y . y + y
        // It should alpha-convert to avoid capture: λ y' . y + y'
        let outer_lambda = Expression::Lambda {
            params: vec![LambdaParam::new("x")],
            body: Box::new(Expression::Lambda {
                params: vec![LambdaParam::new("y")],
                body: Box::new(Expression::Operation {
                    name: "plus".to_string(),
                    args: vec![
                        Expression::Object("x".to_string()),
                        Expression::Object("y".to_string()),
                    ],
                }),
            }),
        };

        // Apply with 'y' as argument (potential capture)
        let result = eval
            .beta_reduce(&outer_lambda, &Expression::Object("y".to_string()))
            .unwrap();

        // Result should be λ y' . y + y' (or similar fresh name)
        match result {
            Expression::Lambda { params, body } => {
                let inner_param = &params[0].name;
                // The inner param should NOT be 'y' anymore (renamed to avoid capture)
                assert_ne!(inner_param, "y");

                match *body {
                    Expression::Operation { ref args, .. } => {
                        // First arg should be 'y' (the argument we passed)
                        assert!(matches!(args[0], Expression::Object(ref s) if s == "y"));
                        // Second arg should be the renamed parameter
                        assert!(matches!(args[1], Expression::Object(ref s) if s == inner_param));
                    }
                    _ => panic!("Expected Operation in body"),
                }
            }
            _ => panic!("Expected Lambda, got {:?}", result),
        }
    }

    #[test]
    fn test_reduce_named_lambda_function() {
        // define f = λ x . x + 1
        // f(5) → 5 + 1
        let mut eval = Evaluator::new();

        let code = "define f = λ x . x + 1";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Create expression f(5)
        let expr = Expression::Operation {
            name: "f".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };

        let result = eval.reduce_to_normal_form(&expr).unwrap();

        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus");
                assert!(matches!(args[0], Expression::Const(ref s) if s == "5"));
                assert!(matches!(args[1], Expression::Const(ref s) if s == "1"));
            }
            _ => panic!("Expected Operation, got {:?}", result),
        }
    }

    #[test]
    fn test_reduce_curried_function() {
        // define add = λ x y . x + y
        // add(3)(4) → 3 + 4
        let mut eval = Evaluator::new();

        let code = "define add = λ x y . x + y";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // For curried application, we need nested operations
        // First: add(3) → λ y . 3 + y
        let partial = Expression::Operation {
            name: "add".to_string(),
            args: vec![Expression::Const("3".to_string())],
        };

        let reduced_partial = eval.reduce_to_normal_form(&partial).unwrap();

        // Should be a lambda
        assert!(matches!(reduced_partial, Expression::Lambda { .. }));
    }

    #[test]
    fn test_reduction_fuel_limit() {
        let eval = Evaluator::new();

        // Create a simple expression that would take many steps
        let expr = Expression::Let {
            pattern: crate::ast::Pattern::Variable("x".to_string()),
            type_annotation: None,
            value: Box::new(Expression::Const("1".to_string())),
            body: Box::new(Expression::Object("x".to_string())),
        };

        // Should complete within fuel limit
        let result = eval.reduce_with_fuel(&expr, 10);
        assert!(result.is_ok());
    }

    // =========================================================================
    // Concrete Evaluation Tests (for :eval command)
    // =========================================================================

    #[test]
    fn test_eval_concrete_arithmetic() {
        let eval = Evaluator::new();

        // 2 + 3 → 5
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("2".to_string()),
                Expression::Const("3".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "5"));

        // 10 * 5 → 50
        let expr = Expression::Operation {
            name: "times".to_string(),
            args: vec![
                Expression::Const("10".to_string()),
                Expression::Const("5".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "50"));

        // 7 - 3 → 4
        let expr = Expression::Operation {
            name: "minus".to_string(),
            args: vec![
                Expression::Const("7".to_string()),
                Expression::Const("3".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "4"));
    }

    #[test]
    fn test_eval_concrete_string_ops() {
        let eval = Evaluator::new();

        // concat("hello", " world") → "hello world"
        let expr = Expression::Operation {
            name: "concat".to_string(),
            args: vec![
                Expression::String("hello".to_string()),
                Expression::String(" world".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "hello world"));

        // strlen("kleis") → 5
        let expr = Expression::Operation {
            name: "strlen".to_string(),
            args: vec![Expression::String("kleis".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "5"));

        // hasPrefix("(define fib)", "(define") → true
        let expr = Expression::Operation {
            name: "hasPrefix".to_string(),
            args: vec![
                Expression::String("(define fib)".to_string()),
                Expression::String("(define".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));

        // contains("hello world", "wor") → true
        let expr = Expression::Operation {
            name: "contains".to_string(),
            args: vec![
                Expression::String("hello world".to_string()),
                Expression::String("wor".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));
    }

    #[test]
    fn test_eval_concrete_comparison() {
        let eval = Evaluator::new();

        // gt(5, 3) → true
        let expr = Expression::Operation {
            name: "gt".to_string(),
            args: vec![
                Expression::Const("5".to_string()),
                Expression::Const("3".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));

        // lt(2, 10) → true
        let expr = Expression::Operation {
            name: "lt".to_string(),
            args: vec![
                Expression::Const("2".to_string()),
                Expression::Const("10".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));

        // eq(5, 5) → true
        let expr = Expression::Operation {
            name: "eq".to_string(),
            args: vec![
                Expression::Const("5".to_string()),
                Expression::Const("5".to_string()),
            ],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));
    }

    #[test]
    fn test_eval_concrete_conditional() {
        let eval = Evaluator::new();

        // if gt(5, 3) then "yes" else "no" → "yes"
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "gt".to_string(),
                args: vec![
                    Expression::Const("5".to_string()),
                    Expression::Const("3".to_string()),
                ],
            }),
            then_branch: Box::new(Expression::String("yes".to_string())),
            else_branch: Box::new(Expression::String("no".to_string())),
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "yes"));

        // if lt(5, 3) then "yes" else "no" → "no"
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "lt".to_string(),
                args: vec![
                    Expression::Const("5".to_string()),
                    Expression::Const("3".to_string()),
                ],
            }),
            then_branch: Box::new(Expression::String("yes".to_string())),
            else_branch: Box::new(Expression::String("no".to_string())),
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "no"));
    }

    #[test]
    fn test_eval_concrete_user_function() {
        let mut eval = Evaluator::new();

        // define double(x) = x + x
        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // double(5) → 10
        let expr = Expression::Operation {
            name: "double".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "10"));
    }

    #[test]
    fn test_eval_concrete_recursion() {
        let mut eval = Evaluator::new();

        // define fib(n) = if le(n, 1) then n else fib(n - 1) + fib(n - 2)
        let code = "define fib(n) = if le(n, 1) then n else fib(n - 1) + fib(n - 2)";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // fib(0) → 0
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("0".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "0"));

        // fib(1) → 1
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("1".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "1"));

        // fib(5) → 5
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "5"));

        // fib(10) → 55
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("10".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "55"));
    }

    #[test]
    fn test_eval_concrete_lisp_parsing() {
        let mut eval = Evaluator::new();

        // Define LISP parsing helpers
        let code = r#"
            define is_list_expr(s) = hasPrefix(s, "(")
            define strip_parens(s) = substr(s, 1, strlen(s) - 2)
            define get_op(s) = charAt(strip_parens(s), 0)
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // is_list_expr("(+ 2 3)") → true
        let expr = Expression::Operation {
            name: "is_list_expr".to_string(),
            args: vec![Expression::String("(+ 2 3)".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));

        // strip_parens("(+ 2 3)") → "+ 2 3"
        let expr = Expression::Operation {
            name: "strip_parens".to_string(),
            args: vec![Expression::String("(+ 2 3)".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "+ 2 3"));

        // get_op("(+ 2 3)") → "+"
        let expr = Expression::Operation {
            name: "get_op".to_string(),
            args: vec![Expression::String("(+ 2 3)".to_string())],
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "+"));
    }
}
