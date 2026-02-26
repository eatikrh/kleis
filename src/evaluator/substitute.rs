use std::collections::{HashMap, HashSet};

use crate::ast::{Expression, LambdaParam};

use super::Evaluator;

impl Evaluator {
    /// Substitute variables in an expression
    ///
    /// Recursively traverses the expression tree and replaces Object(name)
    /// with the bound value from the substitution map.
    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn substitute(&self, expr: &Expression, subst: &HashMap<String, Expression>) -> Expression {
        match expr {
            Expression::Object(name) => {
                // Replace with bound value if exists, otherwise keep as-is
                subst.get(name).cloned().unwrap_or_else(|| expr.clone())
            }

            Expression::Operation { name, args, span } => {
                // Recursively substitute in arguments
                let substituted_args: Vec<Expression> =
                    args.iter().map(|arg| self.substitute(arg, subst)).collect();

                // Check if the operation name is a bound variable (higher-order function)
                if let Some(bound_value) = subst.get(name) {
                    match bound_value {
                        // If bound to an Object, use that name as the function
                        Expression::Object(func_name) => {
                            return Expression::Operation {
                                name: func_name.clone(),
                                args: substituted_args,
                                span: span.clone(),
                            };
                        }
                        // If bound to a Lambda, create a let-binding that applies it
                        // f(x, y) where f = lambda a b . body becomes:
                        // let __f = lambda a b . body in __f(x, y)
                        // This defers evaluation to the evaluator's let handling
                        Expression::Lambda { .. } => {
                            // Create: (let __anon = lambda in __anon(args))
                            // Actually simpler: just inline the lambda application
                            // using a special "apply_lambda" operation
                            return Expression::Operation {
                                name: "apply_lambda".to_string(),
                                args: std::iter::once(bound_value.clone())
                                    .chain(substituted_args)
                                    .collect(),
                                span: span.clone(),
                            };
                        }
                        // Otherwise keep original name
                        _ => {}
                    }
                }

                Expression::Operation {
                    name: name.clone(),
                    args: substituted_args,
                    span: span.clone(),
                }
            }

            Expression::Match {
                scrutinee,
                cases,
                span,
            } => {
                // Substitute in scrutinee
                let new_scrutinee = Box::new(self.substitute(scrutinee, subst));

                // Substitute in each case body and guard (patterns bind their own variables)
                let new_cases = cases
                    .iter()
                    .map(|case| {
                        let mut case_subst = subst.clone();
                        self.remove_pattern_vars_from_subst(&case.pattern, &mut case_subst);
                        crate::ast::MatchCase {
                            pattern: case.pattern.clone(),
                            guard: case.guard.as_ref().map(|g| self.substitute(g, &case_subst)),
                            body: self.substitute(&case.body, &case_subst),
                        }
                    })
                    .collect();

                Expression::Match {
                    scrutinee: new_scrutinee,
                    cases: new_cases,
                    span: span.clone(),
                }
            }

            // Quantifiers - substitute in body
            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
            } => {
                let mut filtered_subst = subst.clone();
                for var in variables {
                    filtered_subst.remove(&var.name);
                }
                Expression::Quantifier {
                    quantifier: quantifier.clone(),
                    variables: variables.clone(),
                    where_clause: where_clause
                        .as_ref()
                        .map(|w| Box::new(self.substitute(w, &filtered_subst))),
                    body: Box::new(self.substitute(body, &filtered_subst)),
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

            // Conditionals - substitute in all branches
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                span,
            } => Expression::Conditional {
                condition: Box::new(self.substitute(condition, subst)),
                then_branch: Box::new(self.substitute(then_branch, subst)),
                else_branch: Box::new(self.substitute(else_branch, subst)),
                span: span.clone(),
            },

            // Let bindings - substitute in value and body
            // Note: the let-bound variable(s) shadow any outer binding
            Expression::Let {
                pattern,
                type_annotation,
                value,
                body,
                span,
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
                    span: span.clone(),
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
            Expression::Lambda { params, body, span } => {
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
                    span: span.clone(),
                }
            }

            // Constants, strings, and placeholders don't change
            Expression::Const(_) | Expression::String(_) | Expression::Placeholder { .. } => {
                expr.clone()
            }
        }
    }


    // =========================================================================
    // Beta Reduction for Lambda Expressions
    // =========================================================================

    /// Default fuel limit for reduction (prevents infinite loops)
    pub(crate) const DEFAULT_REDUCTION_FUEL: usize = 1000;

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
            Expression::Lambda { params, body, .. } => {
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
                        span: None,
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
    pub(crate) fn reduction_step(&self, expr: &Expression) -> Result<Option<Expression>, String> {
        match expr {
            // Check for lambda application pattern in Operation
            // This handles: f(arg) where f resolves to a lambda
            Expression::Operation { name, args, .. } => {
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
                            span: None,
                        }));
                    }
                }

                Ok(None) // No reduction possible
            }

            // Lambda body reduction
            Expression::Lambda { params, body, .. } => {
                if let Some(reduced_body) = self.reduction_step(body)? {
                    Ok(Some(Expression::Lambda {
                        params: params.clone(),
                        body: Box::new(reduced_body),
                        span: None,
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
                        span: None,
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
                ..
            } => {
                // Try to reduce condition first
                if let Some(reduced_cond) = self.reduction_step(condition)? {
                    return Ok(Some(Expression::Conditional {
                        condition: Box::new(reduced_cond),
                        then_branch: then_branch.clone(),
                        else_branch: else_branch.clone(),
                        span: None,
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
                                span: None,
                            }));
                        }
                        // Reduce else branch
                        if let Some(reduced) = self.reduction_step(else_branch)? {
                            return Ok(Some(Expression::Conditional {
                                condition: condition.clone(),
                                then_branch: then_branch.clone(),
                                else_branch: Box::new(reduced),
                                span: None,
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
    pub(crate) fn alpha_convert_if_needed(
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
    pub(crate) fn free_variables(&self, expr: &Expression) -> HashSet<String> {
        let mut free = HashSet::new();
        self.collect_free_variables(expr, &mut HashSet::new(), &mut free);
        free
    }

    /// Helper to collect free variables, tracking bound variables
    pub(crate) fn collect_free_variables(
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
            Expression::Lambda { params, body, .. } => {
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
                ..
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
            Expression::Match {
                scrutinee, cases, ..
            } => {
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
    pub(crate) fn collect_pattern_vars_from_pattern(
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
    pub(crate) fn collect_pattern_vars(&self, pattern: &crate::ast::Pattern, vars: &mut HashSet<String>) {
        self.collect_pattern_vars_from_pattern(pattern, vars);
    }

    /// Remove all variables bound by a pattern from a substitution map
    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn remove_pattern_vars_from_subst(
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
    pub(crate) fn match_pattern_to_bindings(
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
                    ..
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
    pub(crate) fn alpha_convert_pattern(
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
    pub(crate) fn bound_variables(&self, expr: &Expression) -> HashSet<String> {
        let mut bound = HashSet::new();
        self.collect_bound_variables(expr, &mut bound);
        bound
    }

    /// Helper to collect all bound variables
    pub(crate) fn collect_bound_variables(&self, expr: &Expression, bound: &mut HashSet<String>) {
        match expr {
            Expression::Lambda { params, body, .. } => {
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
                ..
            } => {
                self.collect_bound_variables(condition, bound);
                self.collect_bound_variables(then_branch, bound);
                self.collect_bound_variables(else_branch, bound);
            }
            Expression::Match {
                scrutinee, cases, ..
            } => {
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
    pub(crate) fn fresh_variable(&self, base: &str, expr1: &Expression, expr2: &Expression) -> String {
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
    pub(crate) fn alpha_convert(&self, expr: &Expression, old_name: &str, new_name: &str) -> Expression {
        match expr {
            Expression::Lambda { params, body, .. } => {
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
                        span: None,
                    }
                } else {
                    // Just recurse into body
                    Expression::Lambda {
                        params: params.clone(),
                        body: Box::new(self.alpha_convert(body, old_name, new_name)),
                        span: None,
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
                ..
            } => {
                let new_value = self.alpha_convert(value, old_name, new_name);
                // Alpha-convert variables in the pattern
                let new_pattern = self.alpha_convert_pattern(pattern, old_name, new_name);
                Expression::Let {
                    pattern: new_pattern,
                    type_annotation: type_annotation.clone(),
                    value: Box::new(new_value),
                    body: Box::new(self.alpha_convert(body, old_name, new_name)),
                    span: None,
                }
            }
            Expression::Operation { name, args, .. } => Expression::Operation {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| self.alpha_convert(a, old_name, new_name))
                    .collect(),
                span: None,
            },
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => Expression::Conditional {
                condition: Box::new(self.alpha_convert(condition, old_name, new_name)),
                then_branch: Box::new(self.alpha_convert(then_branch, old_name, new_name)),
                else_branch: Box::new(self.alpha_convert(else_branch, old_name, new_name)),
                span: None,
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

}
