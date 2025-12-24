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
use crate::debug::{DebugAction, DebugHook, SourceLocation, StackFrame};
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

    /// Variable bindings from :let command
    /// Maps variable names to their evaluated values
    bindings: HashMap<String, Expression>,

    /// Last evaluation result (for `it` magic variable)
    /// Stores the result of the most recent :eval command
    last_result: Option<Expression>,

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
            last_result: None,
            matcher: PatternMatcher,
            adt_constructors: std::collections::HashSet::new(),
            all_constructors: std::collections::HashSet::new(),
            data_types: Vec::new(),
            structures: Vec::new(),
        }
    }

    // === REPL Value Bindings ===

    /// Set a variable binding (used by :let command)
    pub fn set_binding(&mut self, name: String, value: Expression) {
        self.bindings.insert(name, value);
    }

    /// Get a variable binding
    pub fn get_binding(&self, name: &str) -> Option<&Expression> {
        self.bindings.get(name)
    }

    /// Set the last evaluation result (for `it` magic variable)
    pub fn set_last_result(&mut self, value: Expression) {
        self.last_result = Some(value);
    }

    /// Get the last evaluation result
    pub fn get_last_result(&self) -> Option<&Expression> {
        self.last_result.as_ref()
    }

    /// List all bindings (for :env command)
    pub fn list_bindings(&self) -> Vec<(&String, &Expression)> {
        self.bindings.iter().collect()
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

    /// Evaluate an expression with debug hooks
    ///
    /// This is the same as `eval()` but calls the debug hook at key points,
    /// enabling step-through debugging.
    pub fn eval_with_debug(
        &self,
        expr: &Expression,
        hook: &mut dyn DebugHook,
        depth: usize,
    ) -> Result<Expression, String> {
        // Create a source location (TODO: get real location from AST)
        let location = SourceLocation::new(1, 1);

        // Call the hook before evaluating
        let action = hook.on_eval_start(expr, &location, depth);

        // If we should pause, wait for command
        if action != DebugAction::Continue {
            // The hook handles pausing internally via wait_for_command
        }

        // Now evaluate based on expression type
        let result = match expr {
            Expression::Operation { name, args } => {
                if self.functions.contains_key(name) {
                    // Notify hook about function entry
                    hook.on_function_enter(name, args, depth);
                    hook.push_frame(StackFrame::new(name, location.clone()));

                    // Evaluate arguments with debug
                    let eval_args: Result<Vec<_>, _> = args
                        .iter()
                        .map(|arg| self.eval_with_debug(arg, hook, depth + 1))
                        .collect();
                    let eval_args = eval_args?;

                    // Apply the function
                    let func_result = self.apply_function_with_debug(name, eval_args, hook, depth + 1);

                    // Notify hook about function exit
                    hook.on_function_exit(name, &func_result, depth);
                    hook.pop_frame();

                    func_result
                } else {
                    // Built-in operation - just evaluate arguments
                    let eval_args: Result<Vec<_>, _> = args
                        .iter()
                        .map(|arg| self.eval_with_debug(arg, hook, depth + 1))
                        .collect();
                    let eval_args = eval_args?;

                    Ok(Expression::Operation {
                        name: name.clone(),
                        args: eval_args,
                    })
                }
            }

            Expression::Match { scrutinee, cases } => {
                let eval_scrutinee = self.eval_with_debug(scrutinee, hook, depth + 1)?;
                let result = self.matcher.eval_match(&eval_scrutinee, cases)?;
                self.eval_with_debug(&result, hook, depth + 1)
            }

            Expression::List(elements) => {
                let eval_elements: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|elem| self.eval_with_debug(elem, hook, depth + 1))
                    .collect();
                Ok(Expression::List(eval_elements?))
            }

            Expression::Quantifier { .. } => Ok(expr.clone()),

            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let eval_cond = self.eval_with_debug(condition, hook, depth + 1)?;
                let eval_then = self.eval_with_debug(then_branch, hook, depth + 1)?;
                let eval_else = self.eval_with_debug(else_branch, hook, depth + 1)?;

                Ok(Expression::Conditional {
                    condition: Box::new(eval_cond),
                    then_branch: Box::new(eval_then),
                    else_branch: Box::new(eval_else),
                })
            }

            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                let eval_value = self.eval_with_debug(value, hook, depth + 1)?;

                // Collect bindings and notify hook
                let mut subst = std::collections::HashMap::new();
                self.match_pattern_to_bindings(pattern, &eval_value, &mut subst)?;

                // Notify hook about each binding
                for (name, value) in &subst {
                    hook.on_bind(name, value, depth);
                }

                let substituted_body = self.substitute(body, &subst);
                self.eval_with_debug(&substituted_body, hook, depth + 1)
            }

            Expression::Ascription { expr: inner, .. } => {
                self.eval_with_debug(inner, hook, depth + 1)
            }

            Expression::Lambda { .. } => Ok(expr.clone()),

            Expression::Const(_)
            | Expression::String(_)
            | Expression::Object(_)
            | Expression::Placeholder { .. } => Ok(expr.clone()),
        };

        // Notify hook about evaluation result
        hook.on_eval_end(expr, &result, depth);

        result
    }

    /// Apply a user-defined function with debug hooks
    fn apply_function_with_debug(
        &self,
        name: &str,
        args: Vec<Expression>,
        hook: &mut dyn DebugHook,
        depth: usize,
    ) -> Result<Expression, String> {
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

        // Build substitution map and notify hook about bindings
        let mut subst = HashMap::new();
        for (param, arg) in closure.params.iter().zip(args.iter()) {
            subst.insert(param.clone(), arg.clone());
            hook.on_bind(param, arg, depth);
        }

        // Substitute and evaluate with debug
        let substituted = self.substitute(&closure.body, &subst);
        self.eval_with_debug(&substituted, hook, depth)
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
    // Definition Removal (for :unload/:reload/:reset)
    // =========================================================================

    /// Remove a function by name
    /// Returns true if the function was found and removed
    pub fn remove_function(&mut self, name: &str) -> bool {
        self.functions.remove(name).is_some()
    }

    /// Remove a data type by name, including all its constructors
    /// Returns true if the data type was found and removed
    pub fn remove_data_type(&mut self, name: &str) -> bool {
        // Find the data type and get its constructors
        let mut found_idx = None;
        let mut constructors_to_remove = Vec::new();

        for (idx, data_type) in self.data_types.iter().enumerate() {
            if data_type.name == name {
                found_idx = Some(idx);
                for variant in &data_type.variants {
                    constructors_to_remove.push(variant.name.clone());
                }
                break;
            }
        }

        if let Some(idx) = found_idx {
            // Remove the data type
            self.data_types.remove(idx);

            // Remove its constructors from both sets
            for ctor in constructors_to_remove {
                self.adt_constructors.remove(&ctor);
                self.all_constructors.remove(&ctor);
            }
            true
        } else {
            false
        }
    }

    /// Remove a structure by name
    /// Returns true if the structure was found and removed
    pub fn remove_structure(&mut self, name: &str) -> bool {
        let initial_len = self.structures.len();
        self.structures.retain(|s| s.name != name);
        self.structures.len() < initial_len
    }

    /// Clear all definitions (for :reset command)
    /// Removes all functions, data types, structures, and bindings
    pub fn reset(&mut self) {
        self.functions.clear();
        self.bindings.clear();
        self.last_result = None;
        self.adt_constructors.clear();
        self.all_constructors.clear();
        self.data_types.clear();
        self.structures.clear();
    }

    /// Get counts for status display
    pub fn definition_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.functions.len(),
            self.data_types.len(),
            self.structures.len(),
            self.bindings.len(),
        )
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

            // Variables: check bindings, `it`, functions, ADT constructors
            Expression::Object(name) => {
                // 1. Check REPL bindings (from :let command)
                if let Some(value) = self.bindings.get(name) {
                    return Ok(value.clone());
                }

                // 2. Check `it` magic variable (last eval result)
                if name == "it" {
                    if let Some(value) = &self.last_result {
                        return Ok(value.clone());
                    }
                    // `it` not set yet - return as unbound
                    return Ok(expr.clone());
                }

                // 3. Check defined functions
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
            "negate" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(n) = self.as_number(&args[0]) {
                    Ok(Some(Self::const_from_f64(-n)))
                } else {
                    Ok(None)
                }
            }
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
                // Supports partial symbolic evaluation
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
                    let result: Vec<Expression> = elems1
                        .iter()
                        .zip(elems2.iter())
                        .map(|(a, b)| self.add_expressions(a, b))
                        .collect();
                    Ok(Some(self.make_matrix(m1, n1, result)))
                } else {
                    Ok(None)
                }
            }

            "matrix_sub" | "builtin_matrix_sub" => {
                // Matrix subtraction: element-wise subtraction
                // Supports partial symbolic evaluation
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
                    let result: Vec<Expression> = elems1
                        .iter()
                        .zip(elems2.iter())
                        .map(|(a, b)| self.sub_expressions(a, b))
                        .collect();
                    Ok(Some(self.make_matrix(m1, n1, result)))
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
                    // Check all elements are numeric (no symbolic variables)
                    let all_numeric = elems1
                        .iter()
                        .chain(elems2.iter())
                        .all(|e| self.as_number(e).is_some());
                    if !all_numeric {
                        // Contains symbolic elements - return unevaluated
                        return Ok(None);
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
                    // Check diagonal elements are numeric
                    let diag_numeric = (0..m).all(|i| self.as_number(&elems[i * n + i]).is_some());
                    if !diag_numeric {
                        // Contains symbolic diagonal elements - return unevaluated
                        return Ok(None);
                    }
                    let mut sum = 0.0;
                    for i in 0..m {
                        if let Some(val) = self.as_number(&elems[i * n + i]) {
                            sum += val;
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
                    // Check all elements are numeric (no symbolic variables)
                    if !elems.iter().all(|e| self.as_number(e).is_some()) {
                        // Contains symbolic elements - return unevaluated
                        return Ok(None);
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

            "size" | "shape" | "dims" => {
                // Get matrix dimensions as a tuple/list
                // size(M) → [m, n]
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, _)) = self.extract_matrix(&args[0]) {
                    Ok(Some(Expression::List(vec![
                        Expression::Const(m.to_string()),
                        Expression::Const(n.to_string()),
                    ])))
                } else {
                    Ok(None)
                }
            }

            "nrows" | "num_rows" => {
                // Get number of rows
                // nrows(M) → m
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, _, _)) = self.extract_matrix(&args[0]) {
                    Ok(Some(Expression::Const(m.to_string())))
                } else {
                    Ok(None)
                }
            }

            "ncols" | "num_cols" => {
                // Get number of columns
                // ncols(M) → n
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((_, n, _)) = self.extract_matrix(&args[0]) {
                    Ok(Some(Expression::Const(n.to_string())))
                } else {
                    Ok(None)
                }
            }

            "matrix_get" | "element" => {
                // Get element at (i, j) from matrix
                // matrix_get(M, i, j) → element
                if args.len() != 3 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    let i = self.as_integer(&args[1]);
                    let j = self.as_integer(&args[2]);
                    if let (Some(i), Some(j)) = (i, j) {
                        let i = i as usize;
                        let j = j as usize;
                        if i < m && j < n {
                            let idx = i * n + j;
                            Ok(Some(elems[idx].clone()))
                        } else {
                            Err(format!(
                                "matrix_get: index ({}, {}) out of bounds for {}x{} matrix",
                                i, j, m, n
                            ))
                        }
                    } else {
                        Ok(None) // Symbolic indices - return unevaluated
                    }
                } else {
                    Ok(None)
                }
            }

            "matrix_row" | "row" => {
                // Get row i from matrix as a list
                // matrix_row(M, i) → [elements]
                if args.len() != 2 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    if let Some(i) = self.as_integer(&args[1]) {
                        let i = i as usize;
                        if i < m {
                            let start = i * n;
                            let row: Vec<Expression> = elems[start..start + n].to_vec();
                            Ok(Some(Expression::List(row)))
                        } else {
                            Err(format!(
                                "matrix_row: row {} out of bounds for {}x{} matrix",
                                i, m, n
                            ))
                        }
                    } else {
                        Ok(None) // Symbolic index - return unevaluated
                    }
                } else {
                    Ok(None)
                }
            }

            "matrix_col" | "col" => {
                // Get column j from matrix as a list
                // matrix_col(M, j) → [elements]
                if args.len() != 2 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    if let Some(j) = self.as_integer(&args[1]) {
                        let j = j as usize;
                        if j < n {
                            let col: Vec<Expression> =
                                (0..m).map(|i| elems[i * n + j].clone()).collect();
                            Ok(Some(Expression::List(col)))
                        } else {
                            Err(format!(
                                "matrix_col: column {} out of bounds for {}x{} matrix",
                                j, m, n
                            ))
                        }
                    } else {
                        Ok(None) // Symbolic index - return unevaluated
                    }
                } else {
                    Ok(None)
                }
            }

            "matrix_diag" | "diag" => {
                // Get diagonal elements from square matrix as a list
                // matrix_diag(M) → [diagonal elements]
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    if m != n {
                        return Err(format!(
                            "matrix_diag: matrix must be square, got {}x{}",
                            m, n
                        ));
                    }
                    let diag: Vec<Expression> = (0..m).map(|i| elems[i * n + i].clone()).collect();
                    Ok(Some(Expression::List(diag)))
                } else {
                    Ok(None)
                }
            }

            // === Matrix Mutation (returns new matrix) ===
            "set_element" | "set" => {
                // Set element at (i, j) to a new value, return new matrix
                // set_element(M, i, j, value) → new Matrix
                if args.len() != 4 {
                    return Ok(None);
                }
                if let Some((m, n, mut elems)) = self.extract_matrix(&args[0]) {
                    let i = self.as_integer(&args[1]);
                    let j = self.as_integer(&args[2]);
                    if let (Some(i), Some(j)) = (i, j) {
                        let i = i as usize;
                        let j = j as usize;
                        if i < m && j < n {
                            let idx = i * n + j;
                            // Evaluate the new value
                            let new_val = match self.eval_concrete(&args[3]) {
                                Ok(v) => v,
                                Err(_) => args[3].clone(),
                            };
                            elems[idx] = new_val;
                            Ok(Some(self.make_matrix(m, n, elems)))
                        } else {
                            Err(format!(
                                "set_element: index ({}, {}) out of bounds for {}x{} matrix",
                                i, j, m, n
                            ))
                        }
                    } else {
                        Ok(None) // Symbolic indices - return unevaluated
                    }
                } else {
                    Ok(None)
                }
            }

            "set_row" => {
                // Set row i to new values, return new matrix
                // set_row(M, i, [values]) → new Matrix
                if args.len() != 3 {
                    return Ok(None);
                }
                if let Some((m, n, mut elems)) = self.extract_matrix(&args[0]) {
                    if let Some(i) = self.as_integer(&args[1]) {
                        let i = i as usize;
                        if i >= m {
                            return Err(format!(
                                "set_row: row {} out of bounds for {}x{} matrix",
                                i, m, n
                            ));
                        }
                        // Get the new row values
                        match &args[2] {
                            Expression::List(new_row) => {
                                if new_row.len() != n {
                                    return Err(format!(
                                        "set_row: row has {} elements but matrix has {} columns",
                                        new_row.len(),
                                        n
                                    ));
                                }
                                for (j, val) in new_row.iter().enumerate() {
                                    let new_val = match self.eval_concrete(val) {
                                        Ok(v) => v,
                                        Err(_) => val.clone(),
                                    };
                                    elems[i * n + j] = new_val;
                                }
                                Ok(Some(self.make_matrix(m, n, elems)))
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None) // Symbolic index - return unevaluated
                    }
                } else {
                    Ok(None)
                }
            }

            "set_col" => {
                // Set column j to new values, return new matrix
                // set_col(M, j, [values]) → new Matrix
                if args.len() != 3 {
                    return Ok(None);
                }
                if let Some((m, n, mut elems)) = self.extract_matrix(&args[0]) {
                    if let Some(j) = self.as_integer(&args[1]) {
                        let j = j as usize;
                        if j >= n {
                            return Err(format!(
                                "set_col: column {} out of bounds for {}x{} matrix",
                                j, m, n
                            ));
                        }
                        // Get the new column values
                        match &args[2] {
                            Expression::List(new_col) => {
                                if new_col.len() != m {
                                    return Err(format!(
                                        "set_col: column has {} elements but matrix has {} rows",
                                        new_col.len(),
                                        m
                                    ));
                                }
                                for (i, val) in new_col.iter().enumerate() {
                                    let new_val = match self.eval_concrete(val) {
                                        Ok(v) => v,
                                        Err(_) => val.clone(),
                                    };
                                    elems[i * n + j] = new_val;
                                }
                                Ok(Some(self.make_matrix(m, n, elems)))
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None) // Symbolic index - return unevaluated
                    }
                } else {
                    Ok(None)
                }
            }

            "set_diag" => {
                // Set diagonal elements to new values, return new matrix
                // set_diag(M, [values]) → new Matrix (square matrix only)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let Some((m, n, mut elems)) = self.extract_matrix(&args[0]) {
                    if m != n {
                        return Err(format!("set_diag: matrix must be square, got {}x{}", m, n));
                    }
                    match &args[1] {
                        Expression::List(new_diag) => {
                            if new_diag.len() != m {
                                return Err(format!(
                                    "set_diag: diagonal has {} elements but matrix is {}x{}",
                                    new_diag.len(),
                                    m,
                                    n
                                ));
                            }
                            for (i, val) in new_diag.iter().enumerate() {
                                let new_val = match self.eval_concrete(val) {
                                    Ok(v) => v,
                                    Err(_) => val.clone(),
                                };
                                elems[i * n + i] = new_val;
                            }
                            Ok(Some(self.make_matrix(m, n, elems)))
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }

            // === Matrix Constructors ===
            "eye" | "identity" => {
                // Create n×n identity matrix
                // eye(n) → Matrix(n, n, [1,0,0,...,0,1,0,...,0,0,1])
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(n) = self.as_integer(&args[0]) {
                    if n <= 0 {
                        return Err(format!("eye: size must be positive, got {}", n));
                    }
                    let n = n as usize;
                    let mut elems = Vec::with_capacity(n * n);
                    for i in 0..n {
                        for j in 0..n {
                            if i == j {
                                elems.push(Expression::Const("1".to_string()));
                            } else {
                                elems.push(Expression::Const("0".to_string()));
                            }
                        }
                    }
                    Ok(Some(self.make_matrix(n, n, elems)))
                } else {
                    Ok(None)
                }
            }

            "zeros" => {
                // Create m×n zero matrix
                // zeros(m, n) → Matrix(m, n, [0,0,...,0])
                // zeros(n) → Matrix(n, n, [0,0,...,0])
                if args.is_empty() || args.len() > 2 {
                    return Ok(None);
                }
                let (m, n) = if args.len() == 1 {
                    let size = self.as_integer(&args[0]).unwrap_or(0) as usize;
                    (size, size)
                } else {
                    let m = self.as_integer(&args[0]).unwrap_or(0) as usize;
                    let n = self.as_integer(&args[1]).unwrap_or(0) as usize;
                    (m, n)
                };
                if m == 0 || n == 0 {
                    return Ok(None);
                }
                let elems: Vec<Expression> = vec![Expression::Const("0".to_string()); m * n];
                Ok(Some(self.make_matrix(m, n, elems)))
            }

            "ones" => {
                // Create m×n matrix of ones
                // ones(m, n) → Matrix(m, n, [1,1,...,1])
                // ones(n) → Matrix(n, n, [1,1,...,1])
                if args.is_empty() || args.len() > 2 {
                    return Ok(None);
                }
                let (m, n) = if args.len() == 1 {
                    let size = self.as_integer(&args[0]).unwrap_or(0) as usize;
                    (size, size)
                } else {
                    let m = self.as_integer(&args[0]).unwrap_or(0) as usize;
                    let n = self.as_integer(&args[1]).unwrap_or(0) as usize;
                    (m, n)
                };
                if m == 0 || n == 0 {
                    return Ok(None);
                }
                let elems: Vec<Expression> = vec![Expression::Const("1".to_string()); m * n];
                Ok(Some(self.make_matrix(m, n, elems)))
            }

            "diag_matrix" | "diagonal" => {
                // Create diagonal matrix from list
                // diag_matrix([a, b, c]) → Matrix(3, 3, [a,0,0, 0,b,0, 0,0,c])
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Expression::List(values) = &args[0] {
                    let n = values.len();
                    if n == 0 {
                        return Ok(None);
                    }
                    let mut elems = Vec::with_capacity(n * n);
                    for (i, val) in values.iter().enumerate() {
                        for j in 0..n {
                            if i == j {
                                elems.push(val.clone());
                            } else {
                                elems.push(Expression::Const("0".to_string()));
                            }
                        }
                    }
                    Ok(Some(self.make_matrix(n, n, elems)))
                } else {
                    Ok(None)
                }
            }

            "matrix" => {
                // Create matrix from nested list (row-major)
                // matrix([[1, 2, 3], [4, 5, 6]]) → Matrix(2, 3, [1, 2, 3, 4, 5, 6])
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Expression::List(rows) = &args[0] {
                    if rows.is_empty() {
                        return Err("matrix: empty matrix".to_string());
                    }

                    // Extract first row to get number of columns
                    let first_row = match &rows[0] {
                        Expression::List(r) => r,
                        _ => return Err("matrix: expected list of rows".to_string()),
                    };
                    let n_cols = first_row.len();
                    if n_cols == 0 {
                        return Err("matrix: rows cannot be empty".to_string());
                    }
                    let n_rows = rows.len();

                    // Flatten all rows into elements
                    let mut elems = Vec::with_capacity(n_rows * n_cols);
                    for (i, row) in rows.iter().enumerate() {
                        match row {
                            Expression::List(r) => {
                                if r.len() != n_cols {
                                    return Err(format!(
                                        "matrix: row {} has {} elements, expected {}",
                                        i,
                                        r.len(),
                                        n_cols
                                    ));
                                }
                                for elem in r {
                                    // Evaluate each element to handle expressions like -1
                                    match self.eval_concrete(elem) {
                                        Ok(e) => elems.push(e),
                                        Err(_) => elems.push(elem.clone()),
                                    }
                                }
                            }
                            _ => {
                                return Err(format!("matrix: row {} is not a list", i));
                            }
                        }
                    }

                    Ok(Some(self.make_matrix(n_rows, n_cols, elems)))
                } else {
                    Ok(None)
                }
            }

            // === Matrix Row/Column Manipulation ===
            "vstack" | "append_rows" => {
                // Vertical stack: append rows from B to bottom of A
                // vstack(A, B) where A is m×n and B is k×n → (m+k)×n
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m1, n1, elems1)), Some((m2, n2, elems2))) =
                    (self.extract_matrix(&args[0]), self.extract_matrix(&args[1]))
                {
                    if n1 != n2 {
                        return Err(format!("vstack: column count mismatch: {} vs {}", n1, n2));
                    }
                    let mut result = elems1;
                    result.extend(elems2);
                    Ok(Some(self.make_matrix(m1 + m2, n1, result)))
                } else {
                    Ok(None)
                }
            }

            "hstack" | "append_cols" => {
                // Horizontal stack: append columns from B to right of A
                // hstack(A, B) where A is m×n and B is m×k → m×(n+k)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m1, n1, elems1)), Some((m2, n2, elems2))) =
                    (self.extract_matrix(&args[0]), self.extract_matrix(&args[1]))
                {
                    if m1 != m2 {
                        return Err(format!("hstack: row count mismatch: {} vs {}", m1, m2));
                    }
                    // Interleave columns: for each row, append B's columns after A's
                    let mut result = Vec::with_capacity(m1 * (n1 + n2));
                    for i in 0..m1 {
                        // Add row i from A
                        for j in 0..n1 {
                            result.push(elems1[i * n1 + j].clone());
                        }
                        // Add row i from B
                        for j in 0..n2 {
                            result.push(elems2[i * n2 + j].clone());
                        }
                    }
                    Ok(Some(self.make_matrix(m1, n1 + n2, result)))
                } else {
                    Ok(None)
                }
            }

            "prepend_row" => {
                // Add a row at the top of the matrix
                // prepend_row([a,b,c], M) where M is m×3 → (m+1)×3
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Expression::List(row), Some((m, n, elems))) =
                    (&args[0], self.extract_matrix(&args[1]))
                {
                    if row.len() != n {
                        return Err(format!(
                            "prepend_row: row has {} elements but matrix has {} columns",
                            row.len(),
                            n
                        ));
                    }
                    let mut result = row.clone();
                    result.extend(elems);
                    Ok(Some(self.make_matrix(m + 1, n, result)))
                } else {
                    Ok(None)
                }
            }

            "append_row" => {
                // Add a row at the bottom of the matrix
                // append_row(M, [a,b,c]) where M is m×3 → (m+1)×3
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m, n, elems)), Expression::List(row)) =
                    (self.extract_matrix(&args[0]), &args[1])
                {
                    if row.len() != n {
                        return Err(format!(
                            "append_row: row has {} elements but matrix has {} columns",
                            row.len(),
                            n
                        ));
                    }
                    let mut result = elems;
                    result.extend(row.clone());
                    Ok(Some(self.make_matrix(m + 1, n, result)))
                } else {
                    Ok(None)
                }
            }

            "prepend_col" => {
                // Add a column at the left of the matrix
                // prepend_col([a,b], M) where M is 2×n → 2×(n+1)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Expression::List(col), Some((m, n, elems))) =
                    (&args[0], self.extract_matrix(&args[1]))
                {
                    if col.len() != m {
                        return Err(format!(
                            "prepend_col: column has {} elements but matrix has {} rows",
                            col.len(),
                            m
                        ));
                    }
                    let mut result = Vec::with_capacity(m * (n + 1));
                    for i in 0..m {
                        result.push(col[i].clone());
                        for j in 0..n {
                            result.push(elems[i * n + j].clone());
                        }
                    }
                    Ok(Some(self.make_matrix(m, n + 1, result)))
                } else {
                    Ok(None)
                }
            }

            "append_col" => {
                // Add a column at the right of the matrix
                // append_col(M, [a,b]) where M is 2×n → 2×(n+1)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m, n, elems)), Expression::List(col)) =
                    (self.extract_matrix(&args[0]), &args[1])
                {
                    if col.len() != m {
                        return Err(format!(
                            "append_col: column has {} elements but matrix has {} rows",
                            col.len(),
                            m
                        ));
                    }
                    let mut result = Vec::with_capacity(m * (n + 1));
                    for i in 0..m {
                        for j in 0..n {
                            result.push(elems[i * n + j].clone());
                        }
                        result.push(col[i].clone());
                    }
                    Ok(Some(self.make_matrix(m, n + 1, result)))
                } else {
                    Ok(None)
                }
            }

            // ============================================
            // COMPLEX NUMBER OPERATIONS (Concrete Evaluation)
            // ============================================
            "complex_add" | "cadd" => {
                // Complex addition: (a+bi) + (c+di) = (a+c) + (b+d)i
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((re1, im1)), Some((re2, im2))) = (
                    self.extract_complex(&args[0]),
                    self.extract_complex(&args[1]),
                ) {
                    let re_sum = self.add_expressions(&re1, &re2);
                    let im_sum = self.add_expressions(&im1, &im2);
                    Ok(Some(self.make_complex(re_sum, im_sum)))
                } else {
                    Ok(None)
                }
            }

            "complex_sub" | "csub" => {
                // Complex subtraction: (a+bi) - (c+di) = (a-c) + (b-d)i
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((re1, im1)), Some((re2, im2))) = (
                    self.extract_complex(&args[0]),
                    self.extract_complex(&args[1]),
                ) {
                    let re_diff = self.sub_expressions(&re1, &re2);
                    let im_diff = self.sub_expressions(&im1, &im2);
                    Ok(Some(self.make_complex(re_diff, im_diff)))
                } else {
                    Ok(None)
                }
            }

            "complex_mul" | "cmul" => {
                // Complex multiplication: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((re1, im1)), Some((re2, im2))) = (
                    self.extract_complex(&args[0]),
                    self.extract_complex(&args[1]),
                ) {
                    // Real part: ac - bd
                    let ac = self.mul_expressions(&re1, &re2);
                    let bd = self.mul_expressions(&im1, &im2);
                    let re_result = self.sub_expressions(&ac, &bd);

                    // Imaginary part: ad + bc
                    let ad = self.mul_expressions(&re1, &im2);
                    let bc = self.mul_expressions(&im1, &re2);
                    let im_result = self.add_expressions(&ad, &bc);

                    Ok(Some(self.make_complex(re_result, im_result)))
                } else {
                    Ok(None)
                }
            }

            "complex_conj" | "conj" | "conjugate" => {
                // Complex conjugate: conj(a+bi) = a-bi
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((re, im)) = self.extract_complex(&args[0]) {
                    // Negate imaginary part
                    let neg_im = self.negate_expression(&im);
                    Ok(Some(self.make_complex(re, neg_im)))
                } else {
                    Ok(None)
                }
            }

            "complex_abs_squared" | "abs_sq" => {
                // |z|² = a² + b² (returns real)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((re, im)) = self.extract_complex(&args[0]) {
                    let re_sq = self.mul_expressions(&re, &re);
                    let im_sq = self.mul_expressions(&im, &im);
                    Ok(Some(self.add_expressions(&re_sq, &im_sq)))
                } else {
                    Ok(None)
                }
            }

            "Re" | "re" | "real_part" | "real" => {
                // Real part of complex number
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((re, _im)) = self.extract_complex(&args[0]) {
                    Ok(Some(re))
                } else {
                    Ok(None)
                }
            }

            "Im" | "im" | "imag_part" | "imag" => {
                // Imaginary part of complex number
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((_re, im)) = self.extract_complex(&args[0]) {
                    Ok(Some(im))
                } else {
                    Ok(None)
                }
            }

            // ============================================
            // COMPLEX MATRIX OPERATIONS (v0.91)
            // ============================================
            // ComplexMatrix(m, n) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))
            // Represented as a pair: (real_part, imag_part)
            "cmat_zero" | "builtin_cmat_zero" => {
                // Create zero complex matrix: (zeros(m,n), zeros(m,n))
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(m), Some(n)) = (self.as_nat(&args[0]), self.as_nat(&args[1])) {
                    let zeros =
                        self.make_matrix(m, n, vec![Expression::Const("0".to_string()); m * n]);
                    Ok(Some(Expression::List(vec![zeros.clone(), zeros])))
                } else {
                    Ok(None)
                }
            }

            "cmat_eye" | "builtin_cmat_eye" => {
                // Create complex identity matrix: (eye(n), zeros(n,n))
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(n) = self.as_nat(&args[0]) {
                    let mut eye_elems = vec![Expression::Const("0".to_string()); n * n];
                    for i in 0..n {
                        eye_elems[i * n + i] = Expression::Const("1".to_string());
                    }
                    let eye = self.make_matrix(n, n, eye_elems);
                    let zeros =
                        self.make_matrix(n, n, vec![Expression::Const("0".to_string()); n * n]);
                    Ok(Some(Expression::List(vec![eye, zeros])))
                } else {
                    Ok(None)
                }
            }

            "cmat_from_real" | "builtin_cmat_from_real" | "as_complex" => {
                // Promote real matrix to complex: A → (A, zeros)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    let real_part = self.make_matrix(m, n, elems);
                    let zeros =
                        self.make_matrix(m, n, vec![Expression::Const("0".to_string()); m * n]);
                    Ok(Some(Expression::List(vec![real_part, zeros])))
                } else {
                    Ok(None)
                }
            }

            "cmat_from_imag" | "builtin_cmat_from_imag" | "as_imaginary" => {
                // Create pure imaginary matrix: B → (zeros, B)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elems)) = self.extract_matrix(&args[0]) {
                    let imag_part = self.make_matrix(m, n, elems);
                    let zeros =
                        self.make_matrix(m, n, vec![Expression::Const("0".to_string()); m * n]);
                    Ok(Some(Expression::List(vec![zeros, imag_part])))
                } else {
                    Ok(None)
                }
            }

            "cmat_real" | "builtin_cmat_real" | "real_part_matrix" => {
                // Extract real part: (A, B) → A
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((real, _imag)) = self.extract_complex_matrix(&args[0]) {
                    Ok(Some(real))
                } else {
                    Ok(None)
                }
            }

            "cmat_imag" | "builtin_cmat_imag" | "imag_part_matrix" => {
                // Extract imaginary part: (A, B) → B
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((_real, imag)) = self.extract_complex_matrix(&args[0]) {
                    Ok(Some(imag))
                } else {
                    Ok(None)
                }
            }

            "cmat_add" | "builtin_cmat_add" => {
                // Complex matrix addition: (A₁,B₁) + (A₂,B₂) = (A₁+A₂, B₁+B₂)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((real1, imag1)), Some((real2, imag2))) = (
                    self.extract_complex_matrix(&args[0]),
                    self.extract_complex_matrix(&args[1]),
                ) {
                    let sum_real = self
                        .eval_concrete(&Expression::operation("matrix_add", vec![real1, real2]))?;
                    let sum_imag = self
                        .eval_concrete(&Expression::operation("matrix_add", vec![imag1, imag2]))?;
                    Ok(Some(Expression::List(vec![sum_real, sum_imag])))
                } else {
                    Ok(None)
                }
            }

            "cmat_sub" | "builtin_cmat_sub" => {
                // Complex matrix subtraction: (A₁,B₁) - (A₂,B₂) = (A₁-A₂, B₁-B₂)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((real1, imag1)), Some((real2, imag2))) = (
                    self.extract_complex_matrix(&args[0]),
                    self.extract_complex_matrix(&args[1]),
                ) {
                    let diff_real = self
                        .eval_concrete(&Expression::operation("matrix_sub", vec![real1, real2]))?;
                    let diff_imag = self
                        .eval_concrete(&Expression::operation("matrix_sub", vec![imag1, imag2]))?;
                    Ok(Some(Expression::List(vec![diff_real, diff_imag])))
                } else {
                    Ok(None)
                }
            }

            "cmat_mul" | "builtin_cmat_mul" => {
                // Complex matrix multiplication:
                // (A₁,B₁) · (A₂,B₂) = (A₁·A₂ - B₁·B₂, A₁·B₂ + B₁·A₂)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((a1, b1)), Some((a2, b2))) = (
                    self.extract_complex_matrix(&args[0]),
                    self.extract_complex_matrix(&args[1]),
                ) {
                    // Real part: A₁·A₂ - B₁·B₂
                    let a1a2 = self.eval_concrete(&Expression::operation(
                        "multiply",
                        vec![a1.clone(), a2.clone()],
                    ))?;
                    let b1b2 = self.eval_concrete(&Expression::operation(
                        "multiply",
                        vec![b1.clone(), b2.clone()],
                    ))?;
                    let real_part =
                        self.eval_concrete(&Expression::operation("matrix_sub", vec![a1a2, b1b2]))?;

                    // Imag part: A₁·B₂ + B₁·A₂
                    let a1b2 =
                        self.eval_concrete(&Expression::operation("multiply", vec![a1, b2]))?;
                    let b1a2 =
                        self.eval_concrete(&Expression::operation("multiply", vec![b1, a2]))?;
                    let imag_part =
                        self.eval_concrete(&Expression::operation("matrix_add", vec![a1b2, b1a2]))?;

                    Ok(Some(Expression::List(vec![real_part, imag_part])))
                } else {
                    Ok(None)
                }
            }

            "cmat_conj" | "builtin_cmat_conj" => {
                // Complex conjugate: conj((A,B)) = (A, -B)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((real, imag)) = self.extract_complex_matrix(&args[0]) {
                    // Negate imaginary part
                    let neg_imag = self.eval_concrete(&Expression::operation(
                        "scalar_matrix_mul",
                        vec![Expression::Const("-1".to_string()), imag],
                    ))?;
                    Ok(Some(Expression::List(vec![real, neg_imag])))
                } else {
                    Ok(None)
                }
            }

            "cmat_transpose" | "builtin_cmat_transpose" => {
                // Transpose: transpose((A,B)) = (transpose(A), transpose(B))
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((real, imag)) = self.extract_complex_matrix(&args[0]) {
                    let real_t =
                        self.eval_concrete(&Expression::operation("transpose", vec![real]))?;
                    let imag_t =
                        self.eval_concrete(&Expression::operation("transpose", vec![imag]))?;
                    Ok(Some(Expression::List(vec![real_t, imag_t])))
                } else {
                    Ok(None)
                }
            }

            "cmat_dagger" | "builtin_cmat_dagger" | "cmat_adjoint" => {
                // Conjugate transpose (Hermitian adjoint):
                // dagger((A,B)) = (transpose(A), -transpose(B))
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((real, imag)) = self.extract_complex_matrix(&args[0]) {
                    let real_t =
                        self.eval_concrete(&Expression::operation("transpose", vec![real]))?;
                    let imag_t =
                        self.eval_concrete(&Expression::operation("transpose", vec![imag]))?;
                    let neg_imag_t = self.eval_concrete(&Expression::operation(
                        "scalar_matrix_mul",
                        vec![Expression::Const("-1".to_string()), imag_t],
                    ))?;
                    Ok(Some(Expression::List(vec![real_t, neg_imag_t])))
                } else {
                    Ok(None)
                }
            }

            "cmat_trace" | "builtin_cmat_trace" => {
                // Trace: trace((A,B)) = (trace(A), trace(B))
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((real, imag)) = self.extract_complex_matrix(&args[0]) {
                    let trace_real =
                        self.eval_concrete(&Expression::operation("trace", vec![real]))?;
                    let trace_imag =
                        self.eval_concrete(&Expression::operation("trace", vec![imag]))?;
                    Ok(Some(Expression::List(vec![trace_real, trace_imag])))
                } else {
                    Ok(None)
                }
            }

            "cmat_scale_real" | "builtin_cmat_scale_real" => {
                // Scale by real scalar: r · (A,B) = (r·A, r·B)
                if args.len() != 2 {
                    return Ok(None);
                }
                let scalar = args[0].clone();
                if let Some((real, imag)) = self.extract_complex_matrix(&args[1]) {
                    let scaled_real = self.eval_concrete(&Expression::operation(
                        "scalar_matrix_mul",
                        vec![scalar.clone(), real],
                    ))?;
                    let scaled_imag = self.eval_concrete(&Expression::operation(
                        "scalar_matrix_mul",
                        vec![scalar, imag],
                    ))?;
                    Ok(Some(Expression::List(vec![scaled_real, scaled_imag])))
                } else {
                    Ok(None)
                }
            }

            "realify" | "builtin_realify" => {
                // Embed complex n×n matrix into real 2n×2n matrix:
                // realify((A, B)) = [[A, -B], [B, A]]
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((a_expr, b_expr)) = self.extract_complex_matrix(&args[0]) {
                    if let (Some((n, m, a_elems)), Some((n2, m2, b_elems))) =
                        (self.extract_matrix(&a_expr), self.extract_matrix(&b_expr))
                    {
                        if n != m || n2 != m2 || n != n2 {
                            return Err("realify: complex matrix must be square".to_string());
                        }
                        // Build 2n×2n block matrix [[A, -B], [B, A]]
                        let n2_size = 2 * n;
                        let mut result =
                            vec![Expression::Const("0".to_string()); n2_size * n2_size];

                        for i in 0..n {
                            for j in 0..n {
                                // Top-left: A
                                result[i * n2_size + j] = a_elems[i * n + j].clone();
                                // Top-right: -B
                                let b_val = &b_elems[i * n + j];
                                result[i * n2_size + (j + n)] =
                                    Expression::operation("negate", vec![b_val.clone()]);
                                // Bottom-left: B
                                result[(i + n) * n2_size + j] = b_elems[i * n + j].clone();
                                // Bottom-right: A
                                result[(i + n) * n2_size + (j + n)] = a_elems[i * n + j].clone();
                            }
                        }
                        // Evaluate to simplify negations
                        let mut simplified = Vec::with_capacity(result.len());
                        for elem in result {
                            simplified.push(self.eval_concrete(&elem)?);
                        }
                        Ok(Some(self.make_matrix(n2_size, n2_size, simplified)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }

            "complexify" | "builtin_complexify" => {
                // Extract complex n×n from real 2n×2n with block structure [[A, -B], [B, A]]
                // complexify(M) → (A, B)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m2, n2, elems)) = self.extract_matrix(&args[0]) {
                    if m2 != n2 || m2 % 2 != 0 {
                        return Err(
                            "complexify: matrix must be square with even dimension".to_string()
                        );
                    }
                    let n = m2 / 2;
                    // Extract A from top-left block and B from bottom-left block
                    let mut a_elems = Vec::with_capacity(n * n);
                    let mut b_elems = Vec::with_capacity(n * n);
                    for i in 0..n {
                        for j in 0..n {
                            a_elems.push(elems[i * m2 + j].clone());
                            b_elems.push(elems[(i + n) * m2 + j].clone());
                        }
                    }
                    let a = self.make_matrix(n, n, a_elems);
                    let b = self.make_matrix(n, n, b_elems);
                    Ok(Some(Expression::List(vec![a, b])))
                } else {
                    Ok(None)
                }
            }

            #[cfg(feature = "numerical")]
            "cmat_eigenvalues" | "cmat_eigvals" => {
                // Complex matrix eigenvalues via realification
                // eigenvalues of (A,B) come from eigenvalues of [[A,-B],[B,A]]
                // Real eigenvalues appear doubled; complex pairs appear as a ± bi
                if args.len() != 1 {
                    return Ok(None);
                }
                // First realify the complex matrix
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                // Get eigenvalues of the realified matrix
                let eigs =
                    self.eval_concrete(&Expression::operation("eigenvalues", vec![realified]))?;
                // The eigenvalues of realified matrix are: for each complex eigenvalue a+bi of M,
                // the realified matrix has eigenvalues a+bi and a-bi
                // Return the raw eigenvalues - user can interpret them
                Ok(Some(eigs))
            }

            #[cfg(feature = "numerical")]
            "cmat_schur" | "schur_complex" => {
                // Complex Schur decomposition via realification
                // schur_complex((A,B)) computes Schur of [[A,-B],[B,A]] then complexifies
                if args.len() != 1 {
                    return Ok(None);
                }
                // First realify
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                // Compute real Schur
                let schur_result =
                    self.eval_concrete(&Expression::operation("schur", vec![realified]))?;
                // Return the Schur result (Q, T, eigenvalues)
                // User can apply complexify to Q and T if needed
                Ok(Some(schur_result))
            }

            // === LAPACK Operations (feature-gated) ===
            #[cfg(feature = "numerical")]
            "eigenvalues" | "eigvals" => self.lapack_eigenvalues(args),

            #[cfg(feature = "numerical")]
            "eig" => self.lapack_eig(args),

            #[cfg(feature = "numerical")]
            "svd" => self.lapack_svd(args),

            #[cfg(feature = "numerical")]
            "singular_values" | "svdvals" => self.lapack_singular_values(args),

            #[cfg(feature = "numerical")]
            "solve" | "linsolve" => self.lapack_solve(args),

            #[cfg(feature = "numerical")]
            "inv" | "inverse" => self.lapack_inv(args),

            #[cfg(feature = "numerical")]
            "qr" => self.lapack_qr(args),

            #[cfg(feature = "numerical")]
            "cholesky" | "chol" => self.lapack_cholesky(args),

            #[cfg(feature = "numerical")]
            "rank" | "matrix_rank" => self.lapack_rank(args),

            #[cfg(feature = "numerical")]
            "cond" | "condition_number" => self.lapack_cond(args),

            #[cfg(feature = "numerical")]
            "norm" | "matrix_norm" => self.lapack_norm(args),

            #[cfg(feature = "numerical")]
            "det_lapack" => {
                // Use LAPACK determinant for large matrices (>3x3)
                self.lapack_det(args)
            }

            #[cfg(feature = "numerical")]
            "schur" | "schur_decomp" => self.lapack_schur(args),

            #[cfg(feature = "numerical")]
            "expm" | "matrix_exp" => {
                // Matrix exponential exp(A)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((m, n, elements)) = self.extract_matrix(&args[0]) {
                    if m != n {
                        return Err(format!("expm requires a square matrix, got {}×{}", m, n));
                    }
                    let data: Result<Vec<f64>, _> = elements
                        .iter()
                        .map(|e| {
                            self.as_number(e)
                                .ok_or_else(|| "Symbolic elements not supported".to_string())
                        })
                        .collect();
                    let data = data?;

                    let result = crate::numerical::expm(&data, n).map_err(|e| e.to_string())?;

                    let result_exprs: Vec<Expression> =
                        result.iter().map(|&v| Self::const_from_f64(v)).collect();
                    Ok(Some(self.make_matrix(n, n, result_exprs)))
                } else {
                    Ok(None)
                }
            }

            "mpow" | "matrix_pow" => {
                // Matrix power A^k for integer k
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some((m, n, _elements)), Some(k)) =
                    (self.extract_matrix(&args[0]), self.as_integer(&args[1]))
                {
                    if m != n {
                        return Err(format!("mpow requires a square matrix, got {}×{}", m, n));
                    }
                    #[cfg(feature = "numerical")]
                    if k < 0 {
                        // For negative powers, compute inv(A)^|k|
                        let inv_result = self
                            .eval_concrete(&Expression::operation("inv", vec![args[0].clone()]))?;
                        return self
                            .eval_concrete(&Expression::operation(
                                "mpow",
                                vec![inv_result, Expression::Const(format!("{}", -k))],
                            ))
                            .map(Some);
                    }
                    #[cfg(not(feature = "numerical"))]
                    if k < 0 {
                        return Err(
                            "mpow with negative exponent requires 'numerical' feature".to_string()
                        );
                    }
                    if k == 0 {
                        // A^0 = I
                        return self
                            .eval_concrete(&Expression::operation(
                                "eye",
                                vec![Expression::Const(format!("{}", n))],
                            ))
                            .map(Some);
                    }
                    if k == 1 {
                        return Ok(Some(args[0].clone()));
                    }

                    // Binary exponentiation
                    let mut result = self.eval_concrete(&Expression::operation(
                        "eye",
                        vec![Expression::Const(format!("{}", n))],
                    ))?;
                    let mut base = args[0].clone();
                    let mut exp = k as u64;

                    while exp > 0 {
                        if exp & 1 == 1 {
                            result = self
                                .apply_builtin("multiply", &[result.clone(), base.clone()])?
                                .unwrap_or(result);
                        }
                        base = self
                            .apply_builtin("multiply", &[base.clone(), base.clone()])?
                            .unwrap_or(base);
                        exp >>= 1;
                    }
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }

            // ============================================
            // COMPLEX MATRIX LAPACK OPERATIONS
            // ============================================
            // All use realification: compute on 2n×2n real matrix, interpret results
            #[cfg(feature = "numerical")]
            "cmat_svd" => {
                // Complex SVD via realification
                // For M = (A,B), compute SVD of realify(M)
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let svd_result =
                    self.eval_concrete(&Expression::operation("svd", vec![realified]))?;
                Ok(Some(svd_result))
            }

            #[cfg(feature = "numerical")]
            "cmat_singular_values" | "cmat_svdvals" => {
                // Complex singular values via realification
                // Singular values of complex matrix = singular values of realified / sqrt(2)
                // (Actually each singular value appears twice in realified)
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let svs =
                    self.eval_concrete(&Expression::operation("singular_values", vec![realified]))?;
                // Return the singular values (doubled due to realification)
                Ok(Some(svs))
            }

            #[cfg(feature = "numerical")]
            "cmat_solve" | "cmat_linsolve" => {
                // Solve complex linear system (A+Bi)x = (c+di)
                // Using realification: [[A,-B],[B,A]][xr,xi]^T = [c,d]^T
                if args.len() != 2 {
                    return Ok(None);
                }
                // Get complex matrix and RHS
                if let (Some((_a, _b)), Some((c, d))) = (
                    self.extract_complex_matrix(&args[0]),
                    self.extract_complex_matrix(&args[1]),
                ) {
                    // Realify the matrix
                    let realified_mat = self
                        .eval_concrete(&Expression::operation("realify", vec![args[0].clone()]))?;
                    // Stack RHS: [c; d]
                    let rhs_stacked =
                        self.eval_concrete(&Expression::operation("vstack", vec![c, d]))?;
                    // Solve the real system
                    let sol = self.eval_concrete(&Expression::operation(
                        "solve",
                        vec![realified_mat, rhs_stacked],
                    ))?;
                    // Split solution into real and imaginary parts
                    // solve returns a List, not a Matrix
                    let sol_elems: Vec<Expression> = if let Expression::List(items) = &sol {
                        items.clone()
                    } else if let Some((_n2, _, elems)) = self.extract_matrix(&sol) {
                        elems
                    } else {
                        return Ok(Some(sol));
                    };
                    let n2 = sol_elems.len();
                    let n = n2 / 2;
                    let xr: Vec<_> = sol_elems[..n].to_vec();
                    let xi: Vec<_> = sol_elems[n..].to_vec();
                    let real_part = self.make_matrix(n, 1, xr);
                    let imag_part = self.make_matrix(n, 1, xi);
                    Ok(Some(Expression::List(vec![real_part, imag_part])))
                } else {
                    Ok(None)
                }
            }

            #[cfg(feature = "numerical")]
            "cmat_inv" | "cmat_inverse" => {
                // Complex matrix inverse via realification
                // inv((A,B)) = complexify(inv(realify((A,B))))
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let inv_real =
                    self.eval_concrete(&Expression::operation("inv", vec![realified]))?;
                let result =
                    self.eval_concrete(&Expression::operation("complexify", vec![inv_real]))?;
                Ok(Some(result))
            }

            #[cfg(feature = "numerical")]
            "cmat_qr" => {
                // Complex QR via realification
                // The Q and R of realified matrix can be complexified
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let qr_result =
                    self.eval_concrete(&Expression::operation("qr", vec![realified]))?;
                // Return QR of realified (user can complexify if needed)
                Ok(Some(qr_result))
            }

            #[cfg(feature = "numerical")]
            "cmat_rank" | "cmat_matrix_rank" => {
                // Complex matrix rank via realification
                // rank((A,B)) = rank(realify((A,B))) / 2
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let rank_real =
                    self.eval_concrete(&Expression::operation("rank", vec![realified]))?;
                // Divide by 2 since realification doubles the dimension
                if let Expression::Const(s) = &rank_real {
                    if let Ok(r) = s.parse::<i64>() {
                        return Ok(Some(Expression::Const(format!("{}", r / 2))));
                    }
                }
                Ok(Some(rank_real))
            }

            #[cfg(feature = "numerical")]
            "cmat_cond" | "cmat_condition_number" => {
                // Complex condition number via realification
                // cond((A,B)) = cond(realify((A,B)))
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let cond = self.eval_concrete(&Expression::operation("cond", vec![realified]))?;
                Ok(Some(cond))
            }

            #[cfg(feature = "numerical")]
            "cmat_norm" | "cmat_matrix_norm" => {
                // Complex Frobenius norm: ||M||_F = sqrt(||A||_F^2 + ||B||_F^2)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some((a, b)) = self.extract_complex_matrix(&args[0]) {
                    let norm_a = self.eval_concrete(&Expression::operation("norm", vec![a]))?;
                    let norm_b = self.eval_concrete(&Expression::operation("norm", vec![b]))?;
                    // ||M||_F = sqrt(||A||^2 + ||B||^2)
                    if let (Some(na), Some(nb)) = (self.as_number(&norm_a), self.as_number(&norm_b))
                    {
                        let norm = (na * na + nb * nb).sqrt();
                        return Ok(Some(Expression::Const(format!("{}", norm))));
                    }
                }
                Ok(None)
            }

            #[cfg(feature = "numerical")]
            "cmat_det" | "cmat_determinant" => {
                // Complex determinant via realification
                // |det(M)|^2 = det(realify(M))
                // So det(M) = sqrt(det(realify(M))) * phase
                // For now, return the magnitude squared
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let det_real =
                    self.eval_concrete(&Expression::operation("det_lapack", vec![realified]))?;
                // det_real = |det(M)|^2, return as (det_real, 0) to indicate it's real
                if let Some(d) = self.as_number(&det_real) {
                    // Take square root for magnitude (sign handling is complex)
                    let mag = d.abs().sqrt();
                    return Ok(Some(Expression::List(vec![
                        Expression::Const(format!("{}", mag)),
                        Expression::Const("0".to_string()),
                    ])));
                }
                Ok(Some(det_real))
            }

            #[cfg(feature = "numerical")]
            "cmat_eig" => {
                // Full complex eigendecomposition via realification
                // Returns eigenvalues and eigenvectors
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let eig_result =
                    self.eval_concrete(&Expression::operation("eig", vec![realified]))?;
                Ok(Some(eig_result))
            }

            #[cfg(feature = "numerical")]
            "cmat_expm" | "cmat_matrix_exp" => {
                // Complex matrix exponential via realification
                // exp((A,B)) = complexify(exp(realify((A,B))))
                if args.len() != 1 {
                    return Ok(None);
                }
                let realified =
                    self.eval_concrete(&Expression::operation("realify", args.to_vec()))?;
                let exp_real =
                    self.eval_concrete(&Expression::operation("expm", vec![realified]))?;
                let result =
                    self.eval_concrete(&Expression::operation("complexify", vec![exp_real]))?;
                Ok(Some(result))
            }

            #[cfg(feature = "numerical")]
            "cmat_mpow" | "cmat_matrix_pow" => {
                // Complex matrix power (A+Bi)^k for integer k
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(_), Some(k)) = (
                    self.extract_complex_matrix(&args[0]),
                    self.as_integer(&args[1]),
                ) {
                    if k < 0 {
                        // For negative powers, compute inv(M)^|k|
                        let inv_result = self.eval_concrete(&Expression::operation(
                            "cmat_inv",
                            vec![args[0].clone()],
                        ))?;
                        return self
                            .eval_concrete(&Expression::operation(
                                "cmat_mpow",
                                vec![inv_result, Expression::Const(format!("{}", -k))],
                            ))
                            .map(Some);
                    }
                    if k == 0 {
                        // M^0 = I (complex identity)
                        // Need to get the dimension first
                        if let Some((a, _b)) = self.extract_complex_matrix(&args[0]) {
                            if let Some((n, _, _)) = self.extract_matrix(&a) {
                                return self.apply_builtin(
                                    "cmat_eye",
                                    &[Expression::Const(format!("{}", n))],
                                );
                            }
                        }
                        return Ok(None);
                    }
                    if k == 1 {
                        return Ok(Some(args[0].clone()));
                    }

                    // Binary exponentiation
                    let dim = if let Some((a, _)) = self.extract_complex_matrix(&args[0]) {
                        if let Some((n, _, _)) = self.extract_matrix(&a) {
                            n
                        } else {
                            return Ok(None);
                        }
                    } else {
                        return Ok(None);
                    };

                    let mut result = self
                        .apply_builtin("cmat_eye", &[Expression::Const(format!("{}", dim))])?
                        .unwrap_or(Expression::Const("error".to_string()));
                    let mut base = args[0].clone();
                    let mut exp = k as u64;

                    while exp > 0 {
                        if exp & 1 == 1 {
                            result = self
                                .apply_builtin("cmat_mul", &[result.clone(), base.clone()])?
                                .unwrap_or(result);
                        }
                        base = self
                            .apply_builtin("cmat_mul", &[base.clone(), base.clone()])?
                            .unwrap_or(base);
                        exp >>= 1;
                    }
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
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
            // Handle negate(x) -> -x
            Expression::Operation { name, args } if name == "negate" && args.len() == 1 => {
                self.as_number(&args[0]).map(|n| -n)
            }
            // Handle minus(a, b) -> a - b
            Expression::Operation { name, args } if name == "minus" && args.len() == 2 => {
                let a = self.as_number(&args[0])?;
                let b = self.as_number(&args[1])?;
                Some(a - b)
            }
            // Handle plus(a, b) -> a + b
            Expression::Operation { name, args } if name == "plus" && args.len() == 2 => {
                let a = self.as_number(&args[0])?;
                let b = self.as_number(&args[1])?;
                Some(a + b)
            }
            // Handle times(a, b) -> a * b
            Expression::Operation { name, args } if name == "times" && args.len() == 2 => {
                let a = self.as_number(&args[0])?;
                let b = self.as_number(&args[1])?;
                Some(a * b)
            }
            // Handle divide(a, b) -> a / b
            Expression::Operation { name, args } if name == "divide" && args.len() == 2 => {
                let a = self.as_number(&args[0])?;
                let b = self.as_number(&args[1])?;
                if b != 0.0 {
                    Some(a / b)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn as_integer(&self, expr: &Expression) -> Option<i64> {
        match expr {
            Expression::Const(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Tolerance for treating a floating-point number as zero
    const ZERO_TOLERANCE: f64 = 1e-15;

    /// Format a floating-point number, handling near-zero values as "0"
    fn format_number(v: f64) -> String {
        if v.abs() < Self::ZERO_TOLERANCE {
            "0".to_string()
        } else {
            format!("{}", v)
        }
    }

    /// Create a Const expression from a float, handling near-zero values
    fn const_from_f64(v: f64) -> Expression {
        Expression::Const(Self::format_number(v))
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

    // === Symbolic arithmetic helpers ===
    // These handle mixed concrete/symbolic expressions

    /// Add two expressions, computing concrete results when possible
    fn add_expressions(&self, a: &Expression, b: &Expression) -> Expression {
        match (self.as_number(a), self.as_number(b)) {
            (Some(x), Some(y)) => {
                let sum = x + y;
                if sum.fract() == 0.0 && sum.abs() < 1e15 {
                    Expression::Const(format!("{}", sum as i64))
                } else {
                    Expression::Const(format!("{}", sum))
                }
            }
            // 0 + x = x
            (Some(0.0), None) => b.clone(),
            // x + 0 = x
            (None, Some(0.0)) => a.clone(),
            // Symbolic: create plus operation
            _ => Expression::Operation {
                name: "plus".to_string(),
                args: vec![a.clone(), b.clone()],
            },
        }
    }

    /// Subtract two expressions, computing concrete results when possible
    fn sub_expressions(&self, a: &Expression, b: &Expression) -> Expression {
        match (self.as_number(a), self.as_number(b)) {
            (Some(x), Some(y)) => {
                let diff = x - y;
                if diff.fract() == 0.0 && diff.abs() < 1e15 {
                    Expression::Const(format!("{}", diff as i64))
                } else {
                    Expression::Const(format!("{}", diff))
                }
            }
            // x - 0 = x
            (None, Some(0.0)) => a.clone(),
            // Symbolic: create minus operation
            _ => Expression::Operation {
                name: "minus".to_string(),
                args: vec![a.clone(), b.clone()],
            },
        }
    }

    /// Multiply two expressions, computing concrete results when possible
    fn mul_expressions(&self, a: &Expression, b: &Expression) -> Expression {
        match (self.as_number(a), self.as_number(b)) {
            (Some(x), Some(y)) => {
                let prod = x * y;
                if prod.fract() == 0.0 && prod.abs() < 1e15 {
                    Expression::Const(format!("{}", prod as i64))
                } else {
                    Expression::Const(format!("{}", prod))
                }
            }
            // 0 * x = 0
            (Some(0.0), _) => Expression::Const("0".to_string()),
            // x * 0 = 0
            (_, Some(0.0)) => Expression::Const("0".to_string()),
            // 1 * x = x
            (Some(1.0), None) => b.clone(),
            // x * 1 = x
            (None, Some(1.0)) => a.clone(),
            // Symbolic: create times operation
            _ => Expression::Operation {
                name: "times".to_string(),
                args: vec![a.clone(), b.clone()],
            },
        }
    }

    /// Negate an expression
    fn negate_expression(&self, a: &Expression) -> Expression {
        match self.as_number(a) {
            Some(x) => {
                let neg = -x;
                if neg.fract() == 0.0 && neg.abs() < 1e15 {
                    Expression::Const(format!("{}", neg as i64))
                } else {
                    Expression::Const(format!("{}", neg))
                }
            }
            // 0 negated is still 0
            None if matches!(a, Expression::Const(s) if s == "0") => {
                Expression::Const("0".to_string())
            }
            // Symbolic: create negate operation
            None => Expression::Operation {
                name: "negate".to_string(),
                args: vec![a.clone()],
            },
        }
    }

    // === Complex number helpers ===

    /// Extract (real, imag) from a complex expression
    /// Handles: complex(re, im) or Complex(re, im)
    fn extract_complex(&self, expr: &Expression) -> Option<(Expression, Expression)> {
        match expr {
            Expression::Operation { name, args }
                if (name == "complex" || name == "Complex") && args.len() == 2 =>
            {
                Some((args[0].clone(), args[1].clone()))
            }
            _ => None,
        }
    }

    /// Create a complex expression from real and imaginary parts
    fn make_complex(&self, re: Expression, im: Expression) -> Expression {
        Expression::Operation {
            name: "complex".to_string(),
            args: vec![re, im],
        }
    }

    /// Extract (real_matrix, imag_matrix) from a complex matrix expression
    /// Complex matrices are represented as pairs: (A, B) or List([A, B])
    /// where A is the real part and B is the imaginary part
    fn extract_complex_matrix(&self, expr: &Expression) -> Option<(Expression, Expression)> {
        match expr {
            // List format: [A, B]
            Expression::List(parts) if parts.len() == 2 => {
                // Verify both parts are matrices
                if self.extract_matrix(&parts[0]).is_some()
                    && self.extract_matrix(&parts[1]).is_some()
                {
                    Some((parts[0].clone(), parts[1].clone()))
                } else {
                    None
                }
            }
            // Operation format: pair(A, B) or tuple(A, B)
            Expression::Operation { name, args }
                if (name == "pair" || name == "tuple" || name == "Pair" || name == "Tuple")
                    && args.len() == 2 =>
            {
                if self.extract_matrix(&args[0]).is_some()
                    && self.extract_matrix(&args[1]).is_some()
                {
                    Some((args[0].clone(), args[1].clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extract a natural number from an expression
    fn as_nat(&self, expr: &Expression) -> Option<usize> {
        match expr {
            Expression::Const(s) => s.parse::<usize>().ok(),
            _ => {
                // Try evaluating first
                if let Ok(Expression::Const(s)) = self.eval_concrete(expr) {
                    s.parse::<usize>().ok()
                } else {
                    None
                }
            }
        }
    }

    // === LAPACK Operations ===
    // These require the "numerical" feature flag

    #[cfg(feature = "numerical")]
    fn lapack_eigenvalues(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!(
                "eigenvalues requires a square matrix, got {}×{}",
                m, n
            ));
        }

        // Convert to f64 vec
        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported for LAPACK".to_string())
            })
            .collect();
        let data = data?;

        let eigvals = numerical::eigenvalues(&data, n).map_err(|e| e.to_string())?;

        // Return as list of complex numbers
        let result: Vec<Expression> = eigvals
            .iter()
            .map(|(re, im)| {
                if im.abs() < 1e-14 {
                    Expression::Const(format!("{}", re))
                } else {
                    self.make_complex(
                        Expression::Const(format!("{}", re)),
                        Expression::Const(format!("{}", im)),
                    )
                }
            })
            .collect();

        Ok(Some(Expression::List(result)))
    }

    #[cfg(feature = "numerical")]
    fn lapack_eig(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("eig requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let (eigvals, eigvecs) = numerical::eig(&data, n).map_err(|e| e.to_string())?;

        // Return as tuple (eigenvalues, eigenvectors)
        let vals: Vec<Expression> = eigvals
            .iter()
            .map(|(re, im)| {
                if im.abs() < 1e-14 {
                    Expression::Const(format!("{}", re))
                } else {
                    self.make_complex(
                        Expression::Const(format!("{}", re)),
                        Expression::Const(format!("{}", im)),
                    )
                }
            })
            .collect();

        // Each eigenvector is a column
        let vecs: Vec<Expression> = eigvecs
            .iter()
            .map(|v| {
                Expression::List(
                    v.iter()
                        .map(|(re, im)| {
                            if im.abs() < 1e-14 {
                                Expression::Const(format!("{}", re))
                            } else {
                                self.make_complex(
                                    Expression::Const(format!("{}", re)),
                                    Expression::Const(format!("{}", im)),
                                )
                            }
                        })
                        .collect(),
                )
            })
            .collect();

        // Return as a list of two elements: [eigenvalues, eigenvectors]
        Ok(Some(Expression::List(vec![
            Expression::List(vals),
            Expression::List(vecs),
        ])))
    }

    #[cfg(feature = "numerical")]
    fn lapack_svd(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let (u, s, vt) = numerical::svd(&data, m, n).map_err(|e| e.to_string())?;

        // Return (U, S, Vt) as matrices/vector
        let u_expr = self.make_matrix(m, m, u.iter().map(|&v| Self::const_from_f64(v)).collect());
        let s_expr = Expression::List(s.iter().map(|&v| Self::const_from_f64(v)).collect());
        let vt_expr = self.make_matrix(n, n, vt.iter().map(|&v| Self::const_from_f64(v)).collect());

        // Return as a list of three elements: [U, S, Vt]
        Ok(Some(Expression::List(vec![u_expr, s_expr, vt_expr])))
    }

    #[cfg(feature = "numerical")]
    fn lapack_singular_values(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let s = numerical::singular_values(&data, m, n).map_err(|e| e.to_string())?;

        Ok(Some(Expression::List(
            s.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    #[cfg(feature = "numerical")]
    fn lapack_solve(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 2 {
            return Ok(None);
        }

        let (m, n, a_elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("solve requires a square matrix A, got {}×{}", m, n));
        }

        // b can be a list, vector, or column matrix
        let b_elements = match &args[1] {
            Expression::List(items) => items.clone(),
            Expression::Operation {
                name,
                args: op_args,
            } if name == "Vector" => {
                // Vector(n, [elements])
                if op_args.len() >= 2 {
                    if let Expression::List(items) = &op_args[1] {
                        items.clone()
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            Expression::Operation {
                name,
                args: op_args,
            } if name == "Matrix" || name == "matrix" => {
                // Matrix(rows, cols, [elements]) - extract elements from column matrix
                if op_args.len() >= 3 {
                    if let Expression::List(items) = &op_args[2] {
                        items.clone()
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        if b_elements.len() != n {
            return Err(format!(
                "solve: b has {} elements but A is {}×{}",
                b_elements.len(),
                m,
                n
            ));
        }

        let a_data: Result<Vec<f64>, _> = a_elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let a_data = a_data?;

        let b_data: Result<Vec<f64>, _> = b_elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let b_data = b_data?;

        let x = numerical::solve(&a_data, &b_data, n).map_err(|e| e.to_string())?;

        Ok(Some(Expression::List(
            x.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    #[cfg(feature = "numerical")]
    fn lapack_inv(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("inv requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let inv_data = numerical::inv(&data, n).map_err(|e| e.to_string())?;

        Ok(Some(self.make_matrix(
            n,
            n,
            inv_data.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    #[cfg(feature = "numerical")]
    fn lapack_qr(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let (q, r) = numerical::qr(&data, m, n).map_err(|e| e.to_string())?;

        let k = m.min(n);
        let q_expr = self.make_matrix(m, k, q.iter().map(|&v| Self::const_from_f64(v)).collect());
        let r_expr = self.make_matrix(k, n, r.iter().map(|&v| Self::const_from_f64(v)).collect());

        // Return as a list of two elements: [Q, R]
        Ok(Some(Expression::List(vec![q_expr, r_expr])))
    }

    #[cfg(feature = "numerical")]
    fn lapack_cholesky(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!(
                "cholesky requires a square matrix, got {}×{}",
                m, n
            ));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let l = numerical::cholesky(&data, n).map_err(|e| e.to_string())?;

        Ok(Some(self.make_matrix(
            n,
            n,
            l.iter().map(|&v| Self::const_from_f64(v)).collect(),
        )))
    }

    #[cfg(feature = "numerical")]
    fn lapack_rank(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let r = numerical::rank(&data, m, n, None).map_err(|e| e.to_string())?;

        Ok(Some(Expression::Const(format!("{}", r))))
    }

    #[cfg(feature = "numerical")]
    fn lapack_cond(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let c = numerical::cond(&data, m, n).map_err(|e| e.to_string())?;

        if c.is_infinite() {
            Ok(Some(Expression::Object("Inf".to_string())))
        } else {
            Ok(Some(Expression::Const(format!("{}", c))))
        }
    }

    #[cfg(feature = "numerical")]
    fn lapack_norm(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        // norm(A) or norm(A, "fro")
        if args.is_empty() || args.len() > 2 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        let norm_type = if args.len() == 2 {
            match &args[1] {
                Expression::String(s) => s.as_str(),
                Expression::Object(s) => s.as_str(),
                _ => "fro",
            }
        } else {
            "fro"
        };

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let nval = numerical::norm(&data, m, n, norm_type).map_err(|e| e.to_string())?;

        Ok(Some(Expression::Const(format!("{}", nval))))
    }

    #[cfg(feature = "numerical")]
    fn lapack_det(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("det requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported".to_string())
            })
            .collect();
        let data = data?;

        let d = numerical::det(&data, n).map_err(|e| e.to_string())?;

        Ok(Some(Expression::Const(format!("{}", d))))
    }

    #[cfg(feature = "numerical")]
    fn lapack_schur(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::numerical;

        if args.len() != 1 {
            return Ok(None);
        }

        let (m, n, elements) = match self.extract_matrix(&args[0]) {
            Some(x) => x,
            None => return Ok(None),
        };

        if m != n {
            return Err(format!("schur requires a square matrix, got {}×{}", m, n));
        }

        let data: Result<Vec<f64>, _> = elements
            .iter()
            .map(|e| {
                self.as_number(e)
                    .ok_or_else(|| "Symbolic elements not supported for LAPACK".to_string())
            })
            .collect();
        let data = data?;

        // Use LAPACK Schur decomposition
        let result = numerical::schur_lapack(&data, n).map_err(|e| e.to_string())?;

        // Return [U, T, eigenvalues] as a list
        let u_matrix = self.make_matrix(
            n,
            n,
            result
                .u
                .iter()
                .map(|&x| Expression::Const(format!("{}", x)))
                .collect(),
        );

        let t_matrix = self.make_matrix(
            n,
            n,
            result
                .t
                .iter()
                .map(|&x| Expression::Const(format!("{}", x)))
                .collect(),
        );

        // Eigenvalues as complex pairs (or real if imaginary part is ~0)
        let eigenvalues: Vec<Expression> = result
            .wr
            .iter()
            .zip(result.wi.iter())
            .map(|(&re, &im)| {
                if im.abs() < 1e-14 {
                    Expression::Const(format!("{}", re))
                } else {
                    self.make_complex(
                        Expression::Const(format!("{}", re)),
                        Expression::Const(format!("{}", im)),
                    )
                }
            })
            .collect();

        Ok(Some(Expression::List(vec![
            u_matrix,
            t_matrix,
            Expression::List(eigenvalues),
        ])))
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

    // =========================================================================
    // Tests for remove/reset methods (REPL unload/reload support)
    // =========================================================================

    #[test]
    fn test_remove_function() {
        let mut eval = Evaluator::new();

        let code = "define foo(x) = x + 1\ndefine bar(x) = x * 2";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        assert!(eval.has_function("foo"));
        assert!(eval.has_function("bar"));

        // Remove foo
        assert!(eval.remove_function("foo"));
        assert!(!eval.has_function("foo"));
        assert!(eval.has_function("bar"));

        // Removing again returns false
        assert!(!eval.remove_function("foo"));
    }

    #[test]
    fn test_remove_data_type() {
        let mut eval = Evaluator::new();

        let code = r#"
            data Color = Red | Green | Blue
            data Option(T) = None | Some(value: T)
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Constructors should be registered
        assert!(eval.get_adt_constructors().contains("Red"));
        assert!(eval.get_adt_constructors().contains("Green"));
        assert!(eval.get_adt_constructors().contains("Blue"));

        // Remove Color data type
        assert!(eval.remove_data_type("Color"));

        // Constructors should be gone
        assert!(!eval.get_adt_constructors().contains("Red"));
        assert!(!eval.get_adt_constructors().contains("Green"));
        assert!(!eval.get_adt_constructors().contains("Blue"));

        // Option should still exist - verify by checking data type count
        let (_, data_count, _, _) = eval.definition_counts();
        assert_eq!(data_count, 1); // Only Option remains
    }

    #[test]
    fn test_reset() {
        let mut eval = Evaluator::new();

        let code = r#"
            define foo(x) = x + 1
            data Color = Red | Green | Blue
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Set some bindings
        eval.set_binding("x".to_string(), Expression::Const("42".to_string()));
        eval.set_last_result(Expression::Const("100".to_string()));

        assert!(eval.has_function("foo"));
        assert!(eval.get_adt_constructors().contains("Red"));
        assert!(eval.get_binding("x").is_some());
        assert!(eval.get_last_result().is_some());

        // Reset
        eval.reset();

        // Everything should be gone
        assert!(!eval.has_function("foo"));
        assert!(!eval.get_adt_constructors().contains("Red"));
        assert!(eval.get_binding("x").is_none());
        assert!(eval.get_last_result().is_none());

        let (funcs, data, structs, bindings) = eval.definition_counts();
        assert_eq!(funcs, 0);
        assert_eq!(data, 0);
        assert_eq!(structs, 0);
        assert_eq!(bindings, 0);
    }

    #[test]
    fn test_definition_counts() {
        let mut eval = Evaluator::new();

        let code = r#"
            define foo(x) = x + 1
            define bar(x) = x * 2
            data Color = Red | Green | Blue
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        eval.set_binding("x".to_string(), Expression::Const("1".to_string()));
        eval.set_binding("y".to_string(), Expression::Const("2".to_string()));

        let (funcs, data, structs, bindings) = eval.definition_counts();
        assert_eq!(funcs, 2);
        assert_eq!(data, 1);
        assert_eq!(structs, 0);
        assert_eq!(bindings, 2);
    }

    #[test]
    fn test_eval_with_debug_noop_hook() {
        use crate::debug::NoOpDebugHook;

        let eval = Evaluator::new();
        let mut hook = NoOpDebugHook;

        // Simple constant - should return as-is
        let expr = Expression::Const("42".to_string());
        let result = eval.eval_with_debug(&expr, &mut hook, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expression::Const("42".to_string()));
    }

    #[test]
    fn test_eval_with_debug_function_call() {
        use crate::debug::NoOpDebugHook;

        let mut eval = Evaluator::new();
        let mut hook = NoOpDebugHook;

        // Load a function
        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Call with debug hook
        let expr = Expression::Operation {
            name: "double".to_string(),
            args: vec![Expression::Const("5".to_string())],
        };
        let result = eval.eval_with_debug(&expr, &mut hook, 0);
        assert!(result.is_ok());

        // Result should be 5 + 5 (symbolic)
        match result.unwrap() {
            Expression::Operation { name, args } => {
                assert_eq!(name, "plus"); // Parser converts + to "plus"
                assert_eq!(args.len(), 2);
            }
            other => panic!("Expected Operation, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_with_debug_tracks_bindings() {
        use crate::debug::{DebugAction, DebugHook, DebugState, SourceLocation, StackFrame};
        use std::sync::{Arc, Mutex};

        // A hook that tracks bindings
        struct BindingTracker {
            bindings: Arc<Mutex<Vec<(String, String)>>>,
        }

        impl DebugHook for BindingTracker {
            fn on_eval_start(&mut self, _: &Expression, _: &SourceLocation, _: usize) -> DebugAction {
                DebugAction::Continue
            }
            fn on_eval_end(&mut self, _: &Expression, _: &Result<Expression, String>, _: usize) {}
            fn on_function_enter(&mut self, _: &str, _: &[Expression], _: usize) {}
            fn on_function_exit(&mut self, _: &str, _: &Result<Expression, String>, _: usize) {}
            fn on_bind(&mut self, name: &str, value: &Expression, _: usize) {
                self.bindings.lock().unwrap().push((name.to_string(), format!("{:?}", value)));
            }
            fn state(&self) -> &DebugState { &DebugState::Running }
            fn should_stop(&self, _: &SourceLocation, _: usize) -> bool { false }
            fn wait_for_command(&mut self) -> DebugAction { DebugAction::Continue }
            fn get_stack(&self) -> &[StackFrame] { &[] }
            fn push_frame(&mut self, _: StackFrame) {}
            fn pop_frame(&mut self) -> Option<StackFrame> { None }
        }

        let mut eval = Evaluator::new();
        let bindings = Arc::new(Mutex::new(Vec::new()));
        let mut hook = BindingTracker { bindings: bindings.clone() };

        // Load a function with parameters
        let code = "define add(a, b) = a + b";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Call the function
        let expr = Expression::Operation {
            name: "add".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        };
        let result = eval.eval_with_debug(&expr, &mut hook, 0);
        assert!(result.is_ok());

        // Check that bindings were tracked
        let tracked = bindings.lock().unwrap();
        assert!(tracked.len() >= 2, "Expected at least 2 bindings, got {}", tracked.len());
        assert!(tracked.iter().any(|(name, _)| name == "a"));
        assert!(tracked.iter().any(|(name, _)| name == "b"));
    }
}
