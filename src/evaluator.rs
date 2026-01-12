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
use crate::axiom_verifier::{AxiomVerifier, VerificationResult};
use crate::debug::{DebugHook, SourceLocation};
use crate::kleis_ast::{ExampleBlock, ExampleStatement, FunctionDef, Program, TopLevel};
use crate::kleis_parser::SourceSpan;
use crate::pattern_matcher::PatternMatcher;
use crate::structure_registry::StructureRegistry;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Result of evaluating an example block
#[derive(Debug, Clone)]
pub struct ExampleResult {
    /// Name of the example
    pub name: String,
    /// Whether the example passed (all assertions succeeded)
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Number of assertions that passed
    pub assertions_passed: usize,
    /// Total number of assertions
    pub assertions_total: usize,
}

/// Result of a single assertion
#[derive(Debug, Clone)]
pub enum AssertResult {
    /// Assertion passed (concrete equality)
    Passed,
    /// Assertion verified by Z3 (symbolic proof)
    Verified,
    /// Assertion failed with concrete values (boxed to reduce enum size)
    Failed {
        expected: Box<Expression>,
        actual: Box<Expression>,
    },
    /// Assertion disproved by Z3 with counterexample
    Disproved { counterexample: String },
    /// Assertion couldn't be evaluated (symbolic)
    Unknown(String),
}

/// Represents a user-defined function as a closure
#[derive(Debug, Clone)]
pub struct Closure {
    /// Parameter names
    pub params: Vec<String>,

    /// Function body (expression to evaluate)
    pub body: Expression,

    /// Captured environment (for closures - not used yet in Wire 3)
    pub env: HashMap<String, Expression>,

    /// Source location where this function is defined
    pub span: Option<SourceSpan>,

    /// File path where this function is defined (for cross-file debugging)
    pub file: Option<PathBuf>,
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

    /// Top-level operation declarations (for Z3 type signatures)
    toplevel_operations: HashMap<String, crate::kleis_ast::TypeExpr>,

    /// Implements blocks (for registry - where constraints and concrete bindings)
    implements_blocks: Vec<crate::kleis_ast::ImplementsDef>,

    /// Type aliases (for registry - type abbreviations)
    type_aliases: Vec<crate::kleis_ast::TypeAlias>,

    /// Optional debug hook for step-through debugging
    /// When set, eval() calls hook methods at key points
    /// Uses RefCell for interior mutability (hook needs &mut self)
    debug_hook: RefCell<Option<Box<dyn DebugHook + Send>>>,
}

// =============================================================================
// Free functions for numeric evaluation (used by ODE solver and as_number)
// =============================================================================

/// Evaluate an expression to a numeric value (no evaluator state needed).
///
/// Handles: constants, arithmetic (+, -, *, /), power, trig (sin, cos),
/// exp, sqrt, negation. Returns None for symbolic/unevaluable expressions.
///
/// This is a pure function - it doesn't need Evaluator state, so it can be
/// called from closures (like the ODE solver dynamics function).
pub fn eval_numeric(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::Const(s) => s.parse().ok(),
        Expression::Operation { name, args, .. } => {
            let vals: Option<Vec<f64>> = args.iter().map(eval_numeric).collect();
            let vals = vals?;
            match name.as_str() {
                "plus" | "add" => Some(vals.iter().sum()),
                "minus" | "sub" => match vals.len() {
                    1 => Some(-vals[0]),
                    2 => Some(vals[0] - vals[1]),
                    _ => None,
                },
                "times" | "mul" => Some(vals.iter().product()),
                "div" | "divide" => {
                    if vals.len() == 2 && vals[1] != 0.0 {
                        Some(vals[0] / vals[1])
                    } else {
                        None
                    }
                }
                "pow" | "power" => vals.get(0).zip(vals.get(1)).map(|(a, b)| a.powf(*b)),
                "sin" => vals.first().map(|v| v.sin()),
                "cos" => vals.first().map(|v| v.cos()),
                "tan" => vals.first().map(|v| v.tan()),
                "exp" => vals.first().map(|v| v.exp()),
                "ln" | "log" => vals.first().map(|v| v.ln()),
                "sqrt" => vals.first().map(|v| v.sqrt()),
                "abs" => vals.first().map(|v| v.abs()),
                "neg" | "negate" => vals.first().map(|v| -v),
                _ => None,
            }
        }
        _ => None,
    }
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
            toplevel_operations: HashMap::new(),
            implements_blocks: Vec::new(),
            type_aliases: Vec::new(),
            debug_hook: RefCell::new(None),
        }
    }

    /// Basic unescape for common sequences (\n, \t, \", \\)
    fn unescape_basic(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        out
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

    /// Get all variable bindings (for debugger variable inspection)
    pub fn get_all_bindings(&self) -> impl Iterator<Item = (&String, &Expression)> {
        self.bindings.iter()
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
        self.load_program_with_file(program, None)
    }

    /// Load a program with file path for cross-file debugging
    ///
    /// This is the preferred method when loading files, as it enables
    /// the debugger to track source locations across file boundaries.
    pub fn load_program_with_file(
        &mut self,
        program: &Program,
        file: Option<PathBuf>,
    ) -> Result<(), String> {
        for item in &program.items {
            if let TopLevel::FunctionDef(func_def) = item {
                self.load_function_def_with_file(func_def, file.clone())?;
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

        // Store implements blocks for registry
        for item in &program.items {
            if let TopLevel::ImplementsDef(impl_def) = item {
                self.implements_blocks.push(impl_def.clone());
            }
        }

        // Store type aliases for registry
        for item in &program.items {
            if let TopLevel::TypeAlias(type_alias) = item {
                self.type_aliases.push(type_alias.clone());
            }
        }

        // Store top-level operation declarations for Z3 type lookup
        for item in &program.items {
            if let TopLevel::OperationDecl(op_decl) = item {
                self.toplevel_operations
                    .insert(op_decl.name.clone(), op_decl.type_signature.clone());
            }
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
        self.load_function_def_with_file(func_def, None)
    }

    /// Load a single function definition with file path for cross-file debugging
    pub fn load_function_def_with_file(
        &mut self,
        func_def: &FunctionDef,
        file: Option<PathBuf>,
    ) -> Result<(), String> {
        let closure = Closure {
            params: func_def.params.clone(),
            body: func_def.body.clone(),
            env: HashMap::new(), // Empty environment for now
            span: func_def.span.clone(),
            file,
        };

        self.functions.insert(func_def.name.clone(), closure);
        Ok(())
    }

    /// Get the full source location of a function (line, column, file)
    pub fn get_function_location(&self, name: &str) -> Option<SourceLocation> {
        self.functions.get(name).and_then(|c| {
            c.span.clone().map(|span| {
                // Use SourceLocation::from_span which extracts file from span.file
                let mut loc = SourceLocation::from_span(&span);
                // Fall back to Closure.file if span.file is None (legacy support)
                if loc.file.is_none() {
                    if let Some(ref file) = c.file {
                        loc = loc.with_file(file.clone());
                    }
                }
                loc
            })
        })
    }

    /// Get the source location of a function
    pub fn get_function_span(&self, name: &str) -> Option<SourceSpan> {
        self.functions.get(name).and_then(|c| c.span.clone())
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
        self.apply_function_internal(name, args, 0)
    }

    /// Internal apply_function with depth tracking for debugging
    fn apply_function_internal(
        &self,
        name: &str,
        args: Vec<Expression>,
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

        // Build substitution map: param_name -> argument_value
        let mut subst = HashMap::new();
        for (param, arg) in closure.params.iter().zip(args.iter()) {
            subst.insert(param.clone(), arg.clone());
        }

        // Notify debug hook about parameter bindings
        {
            let mut hook_ref = self.debug_hook.borrow_mut();
            if let Some(ref mut hook) = *hook_ref {
                for (param, arg) in &subst {
                    hook.on_bind(param, arg, depth);
                }
            }
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

            Expression::Operation { name, args, span } => {
                // Recursively substitute in arguments
                let substituted_args: Vec<Expression> =
                    args.iter().map(|arg| self.substitute(arg, subst)).collect();

                // Check if the operation name is a bound variable (higher-order function)
                // If f is bound to "my_func", then f(x) becomes my_func(x)
                let resolved_name = if let Some(bound_value) = subst.get(name) {
                    match bound_value {
                        // If bound to an Object, use that name as the function
                        Expression::Object(func_name) => func_name.clone(),
                        // Otherwise keep original name (can't call a non-function)
                        _ => name.clone(),
                    }
                } else {
                    name.clone()
                };

                Expression::Operation {
                    name: resolved_name,
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
                    .map(|case| crate::ast::MatchCase {
                        pattern: case.pattern.clone(),
                        guard: case.guard.as_ref().map(|g| self.substitute(g, subst)),
                        body: self.substitute(&case.body, subst),
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

    /// Evaluate an expression (symbolic evaluation)
    ///
    /// This resolves function applications and match expressions symbolically.
    /// It does NOT perform arithmetic computation.
    ///
    /// If a debug hook is set (via `set_debug_hook`), this method will call
    /// hook methods at key evaluation points, enabling step-through debugging.
    pub fn eval(&self, expr: &Expression) -> Result<Expression, String> {
        self.eval_internal(expr, 0)
    }

    /// Internal evaluation with depth tracking for debugging
    fn eval_internal(&self, expr: &Expression, depth: usize) -> Result<Expression, String> {
        // Call debug hook ONLY if expression has a valid span
        // Leaf expressions (Const, Object) without spans should NOT trigger stops
        if let Some(span) = expr.get_span() {
            let location = SourceLocation::from_span(span);
            let mut hook_ref = self.debug_hook.borrow_mut();
            if let Some(ref mut hook) = *hook_ref {
                let _action = hook.on_eval_start(expr, &location, depth);
                // Hook handles pausing internally if needed
            }
        }

        // Evaluate based on expression type
        let result = match expr {
            // Check if this is a function application
            Expression::Operation { name, args, span } => {
                if self.functions.contains_key(name) {
                    // Get the function's full source location (span + file) for debugging
                    // Fallback to expression span if function location not found
                    let expr_location = span
                        .as_ref()
                        .map(SourceLocation::from_span)
                        .unwrap_or_else(|| SourceLocation::new(1, 1));
                    let func_location = self.get_function_location(name).unwrap_or(expr_location);

                    crate::logging::log(
                        "DEBUG",
                        "eval",
                        &format!(
                            "Entering function '{}' with location: line={}, file={:?}",
                            name, func_location.line, func_location.file
                        ),
                    );

                    // Call debug hook for function entry with correct location
                    {
                        let mut hook_ref = self.debug_hook.borrow_mut();
                        if let Some(ref mut hook) = *hook_ref {
                            hook.on_function_enter(name, args, &func_location, depth);
                        }
                    }

                    // It's a user-defined function - apply it
                    let eval_args: Result<Vec<_>, _> = args
                        .iter()
                        .map(|arg| self.eval_internal(arg, depth + 1))
                        .collect();
                    let eval_args = eval_args?;

                    let func_result = self.apply_function_internal(name, eval_args, depth + 1);

                    // Report the result's location (span has line/column/file from source)
                    if let Ok(ref result_expr) = func_result {
                        if let Some(span) = result_expr.get_span() {
                            let result_location = SourceLocation::from_span(span);
                            let mut hook_ref = self.debug_hook.borrow_mut();
                            if let Some(ref mut hook) = *hook_ref {
                                let _action =
                                    hook.on_eval_start(result_expr, &result_location, depth);
                            }
                        }
                    }

                    // Call debug hook for function exit
                    // Note: on_function_exit already calls pop_frame internally
                    {
                        let mut hook_ref = self.debug_hook.borrow_mut();
                        if let Some(ref mut hook) = *hook_ref {
                            hook.on_function_exit(name, &func_result, depth);
                        }
                    }

                    func_result
                } else if name == "eval" || name == "reduce" {
                    // Special built-in: evaluate ground terms via Z3 simplify
                    // eval(expr) → concrete value if expr has no free variables
                    if args.len() != 1 {
                        return Err("eval() takes exactly 1 argument".to_string());
                    }

                    // First evaluate the argument
                    let arg_eval = self.eval_internal(&args[0], depth + 1)?;

                    // Check if the expression is symbolic (has free variables)
                    if self.is_symbolic(&arg_eval) {
                        return Err("eval() cannot evaluate expressions with free variables. \
                             Use assert() with Z3 for symbolic verification instead."
                            .to_string());
                    }

                    // Use Z3 simplify to reduce the ground term
                    self.eval_ground_term(&arg_eval)
                } else {
                    // Built-in operation - just evaluate arguments
                    let eval_args: Result<Vec<_>, _> = args
                        .iter()
                        .map(|arg| self.eval_internal(arg, depth + 1))
                        .collect();
                    let eval_args = eval_args?;

                    Ok(Expression::Operation {
                        name: name.clone(),
                        args: eval_args,
                        span: None,
                    })
                }
            }

            // Match expressions - delegate to PatternMatcher
            Expression::Match {
                scrutinee, cases, ..
            } => {
                let eval_scrutinee = self.eval_internal(scrutinee, depth + 1)?;
                let result = self.matcher.eval_match(&eval_scrutinee, cases)?;
                self.eval_internal(&result, depth + 1)
            }

            // Lists - evaluate elements
            Expression::List(elements) => {
                let eval_elements: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|elem| self.eval_internal(elem, depth + 1))
                    .collect();
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
                ..
            } => {
                let eval_cond = self.eval_internal(condition, depth + 1)?;
                let eval_then = self.eval_internal(then_branch, depth + 1)?;
                let eval_else = self.eval_internal(else_branch, depth + 1)?;

                // Return as conditional (we don't evaluate the condition itself)
                // The actual branching is handled by Z3 or pattern matching
                Ok(Expression::Conditional {
                    condition: Box::new(eval_cond),
                    then_branch: Box::new(eval_then),
                    else_branch: Box::new(eval_else),
                    span: None,
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
                let eval_value = self.eval_internal(value, depth + 1)?;

                // Match pattern against value and collect bindings
                let mut subst = std::collections::HashMap::new();
                self.match_pattern_to_bindings(pattern, &eval_value, &mut subst)?;

                // Call debug hook for each binding
                {
                    let mut hook_ref = self.debug_hook.borrow_mut();
                    if let Some(ref mut hook) = *hook_ref {
                        for (name, val) in &subst {
                            hook.on_bind(name, val, depth);
                        }
                    }
                }

                let substituted_body = self.substitute(body, &subst);
                self.eval_internal(&substituted_body, depth + 1)
            }

            // Type ascription - evaluate inner expression, discard type annotation
            // (type checking happens at type-check time, not evaluation time)
            Expression::Ascription { expr: inner, .. } => self.eval_internal(inner, depth),

            // Lambda - return as a value (closures are values)
            Expression::Lambda { .. } => Ok(expr.clone()),

            // Atoms - return as-is
            Expression::Const(_)
            | Expression::String(_)
            | Expression::Object(_)
            | Expression::Placeholder { .. } => Ok(expr.clone()),
        };

        // Call debug hook after evaluation
        {
            let mut hook_ref = self.debug_hook.borrow_mut();
            if let Some(ref mut hook) = *hook_ref {
                hook.on_eval_end(expr, &result, depth);
            }
        }

        result
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
    // Debug Hook Management
    // =========================================================================

    /// Set a debug hook for step-through debugging
    /// When set, eval() will call hook methods at key evaluation points
    pub fn set_debug_hook(&self, hook: Box<dyn DebugHook + Send>) {
        *self.debug_hook.borrow_mut() = Some(hook);
    }

    /// Remove the debug hook (return to normal evaluation)
    pub fn clear_debug_hook(&self) {
        *self.debug_hook.borrow_mut() = None;
    }

    /// Check if debugging is active
    pub fn is_debugging(&self) -> bool {
        self.debug_hook.borrow().is_some()
    }

    // =========================================================================
    // Example Block Evaluation (v0.93)
    // =========================================================================

    /// Helper: Get the source location of a statement (if available)
    fn get_statement_location(stmt: &ExampleStatement) -> Option<crate::ast::FullSourceLocation> {
        match stmt {
            ExampleStatement::Let { location, .. } => location.clone(),
            ExampleStatement::Assert { location, .. } => location.clone(),
            ExampleStatement::Expr { location, .. } => location.clone(),
        }
    }

    /// Helper: Convert a statement to an expression for debug hook
    fn statement_to_expr(stmt: &ExampleStatement) -> Expression {
        match stmt {
            ExampleStatement::Let { value, .. } => value.clone(),
            ExampleStatement::Assert { condition, .. } => condition.clone(),
            ExampleStatement::Expr { expr, .. } => expr.clone(),
        }
    }

    /// Evaluate an example block, returning the result
    ///
    /// Example blocks execute statements sequentially:
    /// - `let` bindings add to local scope
    /// - `assert` statements check conditions
    /// - Expression statements are evaluated for side effects
    ///
    /// # Arguments
    /// * `example` - The example block to evaluate
    ///
    /// # Returns
    /// * `ExampleResult` - Summary of the example execution
    pub fn eval_example_block(&mut self, example: &ExampleBlock) -> ExampleResult {
        let mut assertions_passed = 0;
        let mut assertions_total = 0;

        // Create a snapshot of current bindings to restore later
        let saved_bindings = self.bindings.clone();

        for stmt in &example.statements {
            // Call debug hook with statement location (includes file path)
            if let Some(full_loc) = Self::get_statement_location(stmt) {
                // Convert FullSourceLocation to debug::SourceLocation
                let loc = SourceLocation::new(full_loc.line, full_loc.column);
                let loc = if let Some(ref file) = full_loc.file {
                    loc.with_file(std::path::PathBuf::from(file))
                } else {
                    loc
                };

                if let Some(ref mut hook) = *self.debug_hook.borrow_mut() {
                    let action = hook.on_eval_start(
                        &Self::statement_to_expr(stmt),
                        &loc,
                        0, // top-level depth
                    );
                    // Handle step/continue actions
                    match action {
                        crate::debug::DebugAction::Continue => {}
                        crate::debug::DebugAction::StepInto
                        | crate::debug::DebugAction::StepOver
                        | crate::debug::DebugAction::StepOut => {
                            // These will be handled by the hook's wait_for_command
                        }
                    }
                }
            }

            match stmt {
                ExampleStatement::Let {
                    name,
                    type_annotation: _,
                    value,
                    location: _,
                } => {
                    // Evaluate the value and bind it
                    match self.eval(value) {
                        Ok(evaluated) => {
                            // Notify debug hook about the binding
                            {
                                let mut hook_ref = self.debug_hook.borrow_mut();
                                if let Some(ref mut hook) = *hook_ref {
                                    hook.on_bind(name, &evaluated, 0);
                                }
                            }
                            self.bindings.insert(name.clone(), evaluated);
                        }
                        Err(e) => {
                            // Restore bindings and return error
                            self.bindings = saved_bindings;
                            return ExampleResult {
                                name: example.name.clone(),
                                passed: false,
                                error: Some(format!("Error evaluating let {}: {}", name, e)),
                                assertions_passed,
                                assertions_total,
                            };
                        }
                    }
                }
                ExampleStatement::Assert {
                    condition,
                    location: _,
                } => {
                    assertions_total += 1;
                    let result = self.eval_assert(condition);

                    // Notify debug hook about assertion verification
                    {
                        let mut hook_ref = self.debug_hook.borrow_mut();
                        if let Some(ref mut hook) = *hook_ref {
                            match &result {
                                AssertResult::Passed => {
                                    hook.on_assert_verified(
                                        condition,
                                        true,
                                        "Passed (concrete)",
                                        0,
                                    );
                                }
                                AssertResult::Verified => {
                                    hook.on_assert_verified(condition, true, "Verified by Z3 ✓", 0);
                                }
                                AssertResult::Failed { expected, actual } => {
                                    hook.on_assert_verified(
                                        condition,
                                        false,
                                        &format!(
                                            "Failed: expected {:?}, got {:?}",
                                            expected, actual
                                        ),
                                        0,
                                    );
                                }
                                AssertResult::Disproved { counterexample } => {
                                    hook.on_assert_verified(
                                        condition,
                                        false,
                                        &format!("Disproved by Z3: {}", counterexample),
                                        0,
                                    );
                                }
                                AssertResult::Unknown(reason) => {
                                    hook.on_assert_verified(
                                        condition,
                                        false,
                                        &format!("Unknown: {}", reason),
                                        0,
                                    );
                                }
                            }
                        }
                    }

                    match result {
                        AssertResult::Passed => {
                            assertions_passed += 1;
                        }
                        AssertResult::Verified => {
                            // Z3 verified the symbolic assertion!
                            assertions_passed += 1;
                        }
                        AssertResult::Failed { expected, actual } => {
                            // Restore bindings and return error
                            self.bindings = saved_bindings;
                            return ExampleResult {
                                name: example.name.clone(),
                                passed: false,
                                error: Some(format!(
                                    "Assertion failed: expected {:?}, got {:?}",
                                    expected, actual
                                )),
                                assertions_passed,
                                assertions_total,
                            };
                        }
                        AssertResult::Disproved { counterexample } => {
                            // Z3 found a counterexample!
                            self.bindings = saved_bindings;
                            return ExampleResult {
                                name: example.name.clone(),
                                passed: false,
                                error: Some(format!(
                                    "Assertion disproved by Z3. Counterexample: {}",
                                    counterexample
                                )),
                                assertions_passed,
                                assertions_total,
                            };
                        }
                        AssertResult::Unknown(reason) => {
                            // Unknown means we couldn't verify - fail with explanation
                            // (could be Z3 timeout, feature disabled, or symbolic limitation)
                            self.bindings = saved_bindings;
                            return ExampleResult {
                                name: example.name.clone(),
                                passed: false,
                                error: Some(format!("Assertion unknown: {}", reason)),
                                assertions_passed,
                                assertions_total,
                            };
                        }
                    }
                }
                ExampleStatement::Expr { expr, location: _ } => {
                    // Evaluate expression concretely for side effects (e.g., out())
                    if let Err(e) = self.eval_concrete(expr) {
                        // Restore bindings and return error
                        self.bindings = saved_bindings;
                        return ExampleResult {
                            name: example.name.clone(),
                            passed: false,
                            error: Some(format!("Error evaluating expression: {}", e)),
                            assertions_passed,
                            assertions_total,
                        };
                    }
                }
            }
        }

        // Restore original bindings (example blocks don't leak bindings)
        self.bindings = saved_bindings;

        ExampleResult {
            name: example.name.clone(),
            passed: true,
            error: None,
            assertions_passed,
            assertions_total,
        }
    }

    /// Evaluate an assert condition
    ///
    /// Handles different forms of assertions:
    /// - `a = b` - Equality check
    /// - `predicate(x)` - Predicate check (must evaluate to true-like)
    /// - Concrete values - Directly check if true
    /// - Symbolic values - Return Unknown (for future Z3 integration)
    fn eval_assert(&self, condition: &Expression) -> AssertResult {
        // Check if this is an equality assertion: a = b
        if let Expression::Operation { name, args, .. } = condition {
            if (name == "eq" || name == "equals" || name == "=") && args.len() == 2 {
                return self.eval_equality_assert(&args[0], &args[1]);
            }
        }

        // For quantified assertions, try Z3 first (they can't be evaluated concretely)
        if matches!(condition, Expression::Quantifier { .. }) {
            if let Some(result) = self.verify_with_z3(condition) {
                return result;
            }
            // Z3 couldn't help - return unknown
            return AssertResult::Unknown(
                "Quantified assertion could not be verified (Z3 unavailable or inconclusive)"
                    .to_string(),
            );
        }

        // Otherwise, evaluate the condition and check if it's "true"
        match self.eval(condition) {
            Ok(result) => {
                if self.is_truthy(&result) {
                    AssertResult::Passed
                } else {
                    // If evaluation returned a symbolic result, try Z3
                    if self.is_symbolic(&result) {
                        if let Some(z3_result) = self.verify_with_z3(condition) {
                            return z3_result;
                        }
                    }
                    AssertResult::Failed {
                        expected: Box::new(Expression::Object("true".to_string())),
                        actual: Box::new(result),
                    }
                }
            }
            Err(e) => AssertResult::Unknown(format!("Could not evaluate: {}", e)),
        }
    }

    /// Build a StructureRegistry from the evaluator's loaded structures
    pub fn build_registry(&self, registry: &mut StructureRegistry) {
        for structure in &self.structures {
            let _ = registry.register(structure.clone());
        }
        // Add implements blocks (for where constraints)
        for impl_def in &self.implements_blocks {
            registry.register_implements(impl_def.clone());
        }
        // Add data types (for ADT constructor recognition)
        for data_def in &self.data_types {
            registry.register_data_type(data_def.clone());
        }
        // Add type aliases
        for type_alias in &self.type_aliases {
            registry.register_type_alias(
                type_alias.name.clone(),
                type_alias.params.clone(),
                type_alias.type_expr.clone(),
            );
        }
        // Add top-level operations
        for (name, type_sig) in &self.toplevel_operations {
            registry.register_toplevel_operation(name.clone(), type_sig.clone());
        }
        // Add function definitions (convert Closure to FunctionDef for Z3)
        for (name, closure) in &self.functions {
            let func_def = FunctionDef {
                name: name.clone(),
                params: closure.params.clone(),
                type_annotation: None, // Closures don't preserve type annotations
                body: closure.body.clone(),
                span: closure.span.clone(),
            };
            registry.register_function(func_def);
        }
    }

    /// Build a new StructureRegistry from the evaluator's loaded structures (internal use)
    fn build_registry_internal(&self) -> StructureRegistry {
        let mut registry = StructureRegistry::new();
        self.build_registry(&mut registry);
        registry
    }

    /// Try to verify an assertion using Z3 (for symbolic claims)
    fn verify_with_z3(&self, condition: &Expression) -> Option<AssertResult> {
        let registry = self.build_registry_internal();

        match AxiomVerifier::new(&registry) {
            Ok(mut verifier) => {
                // Load ADT constructors
                verifier.load_adt_constructors(self.adt_constructors.iter());

                match verifier.verify_axiom(condition) {
                    Ok(result) => match result {
                        VerificationResult::Valid => Some(AssertResult::Verified),
                        VerificationResult::Invalid { counterexample } => {
                            Some(AssertResult::Disproved { counterexample })
                        }
                        VerificationResult::Unknown => None, // Fall back to simple eval
                        VerificationResult::Disabled => None, // Feature not enabled
                    },
                    Err(_) => None, // Verification error, fall back
                }
            }
            Err(_) => None, // Couldn't create verifier, fall back
        }
    }

    /// Evaluate a ground term (no free variables) using Z3 simplify.
    ///
    /// This provides concrete evaluation for expressions like:
    /// - `eval(1 + 2 * 3)` → `7`
    /// - `eval(if 5 ≤ 3 then 1 else 2)` → `2`
    ///
    /// # Errors
    /// Returns an error if:
    /// - Z3 is not available (axiom-verification feature disabled)
    /// - Z3 fails to simplify the expression
    fn eval_ground_term(&self, expr: &Expression) -> Result<Expression, String> {
        let registry = self.build_registry_internal();

        match AxiomVerifier::new(&registry) {
            Ok(mut verifier) => {
                // Load ADT constructors for proper type handling
                verifier.load_adt_constructors(self.adt_constructors.iter());

                // Use Z3 simplify to evaluate the ground term
                match verifier.simplify(expr) {
                    Ok(simplified) => Ok(simplified),
                    Err(e) => Err(format!("eval() failed to simplify expression: {}", e)),
                }
            }
            Err(e) => Err(format!(
                "eval() requires Z3 (axiom-verification feature). Error: {}",
                e
            )),
        }
    }

    /// Evaluate an equality assertion: assert(a = b)
    fn eval_equality_assert(&self, left: &Expression, right: &Expression) -> AssertResult {
        // First, resolve any variables in the expressions
        let left_resolved = self.resolve_expression(left);
        let right_resolved = self.resolve_expression(right);

        // Try to FULLY evaluate both sides using eval_concrete()
        // This ensures sin(0) becomes 0, not Operation{sin, [0]}
        let left_result = self.eval_concrete(&left_resolved);
        let right_result = self.eval_concrete(&right_resolved);

        match (left_result, right_result) {
            (Ok(left_val), Ok(right_val)) => {
                // Both sides evaluated - check structural equality
                if self.expressions_equal(&left_val, &right_val) {
                    return AssertResult::Passed;
                }

                // For numeric comparisons, also try floating-point equality
                // (handles cases like 1.0 vs 1 or floating point rounding)
                if let (Some(left_num), Some(right_num)) =
                    (self.as_number(&left_val), self.as_number(&right_val))
                {
                    // Use relative epsilon for floating point comparison
                    let diff = (left_num - right_num).abs();
                    let max_val = left_num.abs().max(right_num.abs()).max(1.0);
                    if diff < max_val * 1e-10 {
                        return AssertResult::Passed;
                    }
                }

                // Structural equality failed - check if either side is symbolic
                // If so, try Z3 verification
                if self.is_symbolic(&left_val) || self.is_symbolic(&right_val) {
                    let equality_expr = Expression::Operation {
                        name: "equals".to_string(),
                        args: vec![left_val.clone(), right_val.clone()],
                        span: None,
                    };

                    if let Some(result) = self.verify_with_z3(&equality_expr) {
                        return result;
                    }

                    // Z3 couldn't help - return unknown (optimistic)
                    return AssertResult::Unknown(format!(
                        "Symbolic assertion: cannot verify {} = {}",
                        self.expr_summary(&left_val),
                        self.expr_summary(&right_val)
                    ));
                }

                // Both sides are concrete but not equal - fail
                AssertResult::Failed {
                    expected: Box::new(right_val),
                    actual: Box::new(left_val),
                }
            }
            (Err(_), _) | (_, Err(_)) => {
                // At least one side couldn't be evaluated - try Z3 verification
                let equality_expr = Expression::Operation {
                    name: "equals".to_string(),
                    args: vec![left_resolved.clone(), right_resolved.clone()],
                    span: None,
                };

                if let Some(result) = self.verify_with_z3(&equality_expr) {
                    return result;
                }

                // Z3 couldn't help - return unknown
                AssertResult::Unknown(format!(
                    "Symbolic assertion: {} = {}",
                    self.expr_summary(&left_resolved),
                    self.expr_summary(&right_resolved)
                ))
            }
        }
    }

    /// Check if an expression is symbolic (contains unbound variables)
    fn is_symbolic(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Object(name) => {
                // It's symbolic if not bound and not an ADT constructor
                !self.bindings.contains_key(name) && !self.adt_constructors.contains(name)
            }
            Expression::Const(_) | Expression::String(_) | Expression::Placeholder { .. } => false,
            Expression::Operation { args, .. } => args.iter().any(|arg| self.is_symbolic(arg)),
            Expression::List(elements) => elements.iter().any(|e| self.is_symbolic(e)),
            Expression::Match {
                scrutinee, cases, ..
            } => {
                self.is_symbolic(scrutinee) || cases.iter().any(|case| self.is_symbolic(&case.body))
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.is_symbolic(condition)
                    || self.is_symbolic(then_branch)
                    || self.is_symbolic(else_branch)
            }
            Expression::Let { value, body, .. } => {
                self.is_symbolic(value) || self.is_symbolic(body)
            }
            Expression::Lambda { body, .. } => self.is_symbolic(body),
            Expression::Ascription { expr, .. } => self.is_symbolic(expr),
            Expression::Quantifier { body, .. } => self.is_symbolic(body),
        }
    }

    /// Get a short summary of an expression for error messages
    fn expr_summary(&self, expr: &Expression) -> String {
        format!("{:?}", expr).chars().take(40).collect()
    }

    /// Resolve variables in an expression using current bindings
    fn resolve_expression(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Object(name) => {
                // Check if this variable is bound
                if let Some(value) = self.bindings.get(name) {
                    value.clone()
                } else {
                    expr.clone()
                }
            }
            Expression::Operation { name, args, .. } => Expression::Operation {
                name: name.clone(),
                args: args.iter().map(|a| self.resolve_expression(a)).collect(),
                span: None,
            },
            _ => expr.clone(),
        }
    }

    /// Check if two expressions are structurally equal
    fn expressions_equal(&self, left: &Expression, right: &Expression) -> bool {
        // First, normalize boolean representations
        let left_bool = self.as_boolean(left);
        let right_bool = self.as_boolean(right);
        if let (Some(l), Some(r)) = (left_bool, right_bool) {
            return l == r;
        }

        match (left, right) {
            (Expression::Const(a), Expression::Const(b)) => {
                // Try numeric comparison for constants
                match (a.parse::<f64>(), b.parse::<f64>()) {
                    (Ok(a_num), Ok(b_num)) => (a_num - b_num).abs() < 1e-10,
                    _ => a == b,
                }
            }
            (Expression::String(a), Expression::String(b)) => a == b,
            (Expression::Object(a), Expression::Object(b)) => a == b,
            // Handle Const/Object cross-comparison for identical strings
            (Expression::Const(a), Expression::Object(b))
            | (Expression::Object(a), Expression::Const(b)) => a == b,
            (
                Expression::Operation {
                    name: n1, args: a1, ..
                },
                Expression::Operation {
                    name: n2, args: a2, ..
                },
            ) => {
                n1 == n2
                    && a1.len() == a2.len()
                    && a1
                        .iter()
                        .zip(a2.iter())
                        .all(|(x, y)| self.expressions_equal(x, y))
            }
            _ => left == right, // Fall back to Eq trait
        }
    }

    /// Try to interpret an expression as a boolean value
    fn as_boolean(&self, expr: &Expression) -> Option<bool> {
        match expr {
            Expression::Const(s) | Expression::Object(s) => match s.to_lowercase().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    /// Check if an expression represents a "truthy" value
    fn is_truthy(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Object(name) => {
                // Common truth values
                name == "true" || name == "True" || name == "⊤"
            }
            Expression::Const(s) => {
                // Non-zero numbers are truthy
                s.parse::<f64>().map(|n| n != 0.0).unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Run all example blocks in a program
    ///
    /// Returns a vector of results for each example block
    pub fn run_all_examples(&mut self, program: &Program) -> Vec<ExampleResult> {
        let mut results = Vec::new();

        for item in &program.items {
            if let TopLevel::ExampleBlock(example) = item {
                let result = self.eval_example_block(example);
                results.push(result);
            }
        }

        results
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
    fn reduction_step(&self, expr: &Expression) -> Result<Option<Expression>, String> {
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
            Expression::Operation { name, args, .. } => {
                // First, evaluate all arguments
                // First, evaluate all arguments
                let eval_args: Result<Vec<_>, _> =
                    args.iter().map(|a| self.eval_concrete(a)).collect();
                let eval_args = eval_args?;

                // Fast-path for raw Typst helpers to ensure they always collapse
                if name == "typst_raw" {
                    return self.builtin_typst_raw(&eval_args).map(|res| {
                        res.unwrap_or(Expression::Operation {
                            name: name.clone(),
                            args: eval_args.clone(),
                            span: None,
                        })
                    });
                }
                if name == "concat" {
                    return self.builtin_concat(&eval_args).map(|res| {
                        res.unwrap_or(Expression::Operation {
                            name: name.clone(),
                            args: eval_args.clone(),
                            span: None,
                        })
                    });
                }

                // Check if this is a data constructor (e.g., Atom, List, Cons, Some)
                // Constructors are values - return them with evaluated args
                if self.all_constructors.contains(name) || self.is_constructor_name(name) {
                    return Ok(Expression::Operation {
                        name: name.clone(),
                        args: eval_args,
                        span: None,
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
                    span: None,
                })
            }

            // Conditionals: evaluate condition and branch
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
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
                            span: None,
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
            Expression::Match {
                scrutinee, cases, ..
            } => {
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
            // === Output (for exploration) ===
            "out" | "show" | "print" => {
                // out(expr) - pretty-prints the expression and returns it
                // Useful for exploring computed values in example blocks and Jupyter
                if args.len() != 1 {
                    return Err("out() takes exactly 1 argument".to_string());
                }
                // Evaluate the argument first to get the concrete value
                let value = self.eval_concrete(&args[0])?;
                // Pretty-print the value
                let formatted = self.pretty_print_value(&value);
                println!("{}", formatted);
                // Return the value (so it can be used in further expressions)
                Ok(Some(value))
            }

            // === Plotting (Compositional API - matches Lilaq 1:1) ===
            //
            // diagram(options, plot(...), bar(...), scatter(...))
            //   → Combines elements and renders to SVG
            //
            // plot(xs, ys, options) → PlotElement
            // bar(xs, heights, options) → PlotElement
            // scatter(xs, ys, options) → PlotElement
            // etc.
            //
            "diagram" => self.builtin_diagram(args),
            "plot" => self.builtin_plot_element(args, crate::plotting::PlotType::Line),
            "scatter" => self.builtin_plot_element(args, crate::plotting::PlotType::Scatter),
            "bar" => self.builtin_plot_element(args, crate::plotting::PlotType::Bar),
            "hbar" => self.builtin_plot_element(args, crate::plotting::PlotType::HBar),
            "stem" => self.builtin_plot_element(args, crate::plotting::PlotType::Stem),
            "hstem" => self.builtin_plot_element(args, crate::plotting::PlotType::HStem),
            "fill_between" => self.builtin_fill_between_element(args),
            "stacked_area" => self.builtin_stacked_area(args),
            "boxplot" => self.builtin_boxplot_element(args, false),
            "hboxplot" => self.builtin_boxplot_element(args, true),
            "heatmap" | "colormesh" => {
                self.builtin_matrix_element(args, crate::plotting::PlotType::Colormesh)
            }
            "contour" => self.builtin_matrix_element(args, crate::plotting::PlotType::Contour),
            "quiver" => self.builtin_quiver_element(args),
            "place" => self.builtin_place_element(args),
            "yaxis" | "secondary_yaxis" => self.builtin_yaxis_element(args),
            "xaxis" | "secondary_xaxis" => self.builtin_xaxis_element(args),
            "path" => self.builtin_path_element(args),

            // Export Typst code (for embedding in documents)
            "export_typst" => self.builtin_export_typst(args),
            "export_typst_fragment" => self.builtin_export_typst_fragment(args),

            // Generate Typst table from Kleis data
            "table_typst" => self.builtin_table_typst(args),
            "table_typst_raw" => self.builtin_table_typst_raw(args),
            "typst_raw" => self.builtin_typst_raw(args),
            "concat" => self.builtin_concat(args),
            "str_eq" => self.builtin_str_eq(args),

            // Render EditorNode AST to Typst (for equations in documents)
            "render_to_typst" => self.builtin_render_to_typst(args),

            "lighten" => {
                // lighten(color, amount) → "color.lighten(amount)"
                // For Typst color manipulation
                if args.len() != 2 {
                    return Ok(None);
                }
                let color = self.extract_string(&args[0])?;
                let amount = self.extract_string(&args[1])?;
                Ok(Some(Expression::String(format!(
                    "{}.lighten({})",
                    color, amount
                ))))
            }

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
            "lt" | "<" | "less_than" => self.builtin_comparison(args, |a, b| a < b),
            "le" | "<=" | "≤" | "leq" | "less_or_equal" => {
                self.builtin_comparison(args, |a, b| a <= b)
            }
            "gt" | ">" | "greater_than" => self.builtin_comparison(args, |a, b| a > b),
            "ge" | ">=" | "≥" | "geq" | "greater_or_equal" => {
                self.builtin_comparison(args, |a, b| a >= b)
            }

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
            "not" | "¬" | "logical_not" => {
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
            "intToStr" | "int_to_str" | "fromInt" | "intToString" | "builtin_intToStr" => {
                // intToStr(42) → "42"
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(n) = self.as_integer(&args[0]) {
                    Ok(Some(Expression::String(format!("{}", n))))
                } else if let Some(f) = self.as_number(&args[0]) {
                    // Handle floats by converting to integer first
                    Ok(Some(Expression::String(format!("{}", f as i64))))
                } else {
                    Ok(None)
                }
            }
            "strToInt" | "str_to_int" | "toInt" | "builtin_strToInt" => {
                // strToInt("42") → 42
                // strToInt("abc") → -1 (invalid)
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    match s.trim().parse::<i64>() {
                        Ok(n) => Ok(Some(Expression::Const(format!("{}", n)))),
                        Err(_) => Ok(Some(Expression::Const("-1".to_string()))),
                    }
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
                    span: None,
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
                    Expression::Operation {
                        name, args: inner, ..
                    } if name == "Cons" && inner.len() == 2 => Ok(Some(inner[0].clone())),
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
                    Expression::Operation {
                        name, args: inner, ..
                    } if name == "Cons" && inner.len() == 2 => Ok(Some(inner[1].clone())),
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
                    Expression::Operation {
                        name, args: inner, ..
                    } if name == "Cons" => {
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
                    (
                        Expression::Operation {
                            name, args: inner, ..
                        },
                        Some(0),
                    ) if name == "Cons" => Ok(Some(inner[0].clone())),
                    (
                        Expression::Operation {
                            name, args: inner, ..
                        },
                        Some(i),
                    ) if name == "Cons" && i > 0 => self.apply_builtin(
                        "nth",
                        &[inner[1].clone(), Expression::Const(format!("{}", i - 1))],
                    ),
                    _ => Ok(None),
                }
            }
            "list_map" => {
                // list_map(f, [a, b, c]) → [f(a), f(b), f(c)]
                // Works with Expression::List (bracket lists)
                if args.len() != 2 {
                    return Ok(None);
                }
                let func = &args[0];

                // Evaluate the list argument first (e.g., linspace(0, 10, 5) → [0, 2.5, ...])
                let evaluated_list = self.eval_concrete(&args[1])?;

                // Handle Expression::List
                if let Expression::List(elements) = &evaluated_list {
                    let mut results = Vec::with_capacity(elements.len());
                    for elem in elements {
                        // Apply function using beta reduction
                        let reduced = self.beta_reduce(func, elem)?;
                        let result = self.eval_concrete(&reduced)?;
                        results.push(result);
                    }
                    return Ok(Some(Expression::List(results)));
                }

                // Also handle Cons/Nil lists for compatibility
                if let Expression::Object(s) = &evaluated_list {
                    if s == "Nil" {
                        return Ok(Some(Expression::List(vec![])));
                    }
                }
                if let Expression::Operation {
                    name, args: inner, ..
                } = &evaluated_list
                {
                    if name == "Nil" {
                        return Ok(Some(Expression::List(vec![])));
                    }
                    if name == "Cons" && inner.len() == 2 {
                        // Recursively map over Cons list
                        let head = &inner[0];
                        let tail = &inner[1];

                        // Apply function to head using beta reduction
                        let reduced = self.beta_reduce(func, head)?;
                        let new_head = self.eval_concrete(&reduced)?;

                        // Recursively map over tail
                        let mapped_tail =
                            self.apply_builtin("list_map", &[func.clone(), tail.clone()])?;
                        if let Some(Expression::List(mut tail_elems)) = mapped_tail {
                            let mut result = vec![new_head];
                            result.append(&mut tail_elems);
                            return Ok(Some(Expression::List(result)));
                        }
                    }
                }

                Ok(None)
            }
            "list_filter" => {
                // list_filter(predicate, [a, b, c]) → elements where predicate(x) is true
                if args.len() != 2 {
                    return Ok(None);
                }
                let pred = &args[0];

                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[1])?;

                if let Expression::List(elements) = &evaluated_list {
                    let mut results = Vec::new();
                    for elem in elements {
                        // Apply predicate using beta reduction
                        let reduced = self.beta_reduce(pred, elem)?;
                        let result = self.eval_concrete(&reduced)?;
                        // Check if result is truthy
                        if let Expression::Object(s) = &result {
                            if s == "true" || s == "True" {
                                results.push(elem.clone());
                            }
                        } else if let Expression::Const(s) = &result {
                            if s == "true" || s == "True" {
                                results.push(elem.clone());
                            }
                        }
                    }
                    return Ok(Some(Expression::List(results)));
                }
                Ok(None)
            }
            "list_fold" => {
                // list_fold(f, init, [a, b, c]) → f(f(f(init, a), b), c)
                if args.len() != 3 {
                    return Ok(None);
                }
                let func = &args[0];
                let init = &args[1];

                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[2])?;

                if let Expression::List(elements) = &evaluated_list {
                    let mut acc = init.clone();
                    for elem in elements {
                        // Apply function: acc = f(acc, elem) using beta reduction
                        let reduced = self.beta_reduce_multi(func, &[acc, elem.clone()])?;
                        acc = self.eval_concrete(&reduced)?;
                    }
                    return Ok(Some(acc));
                }
                Ok(None)
            }
            "list_zip" => {
                // list_zip([a, b, c], [1, 2, 3]) → [(a, 1), (b, 2), (c, 3)]
                // Returns pairs (tuples) of corresponding elements
                if args.len() != 2 {
                    return Ok(None);
                }

                // Evaluate both list arguments first
                let evaluated_xs = self.eval_concrete(&args[0])?;
                let evaluated_ys = self.eval_concrete(&args[1])?;

                if let (Expression::List(xs), Expression::List(ys)) = (&evaluated_xs, &evaluated_ys)
                {
                    let pairs: Vec<Expression> = xs
                        .iter()
                        .zip(ys.iter())
                        .map(|(x, y)| Expression::operation("Pair", vec![x.clone(), y.clone()]))
                        .collect();
                    return Ok(Some(Expression::List(pairs)));
                }
                Ok(None)
            }
            "fst" | "first" => {
                // fst(Pair(a, b)) → a
                if args.len() != 1 {
                    return Ok(None);
                }
                // Evaluate the argument first
                let evaluated = self.eval_concrete(&args[0])?;

                if let Expression::Operation {
                    name,
                    args: pair_args,
                    ..
                } = &evaluated
                {
                    if name == "Pair" && pair_args.len() == 2 {
                        return Ok(Some(pair_args[0].clone()));
                    }
                }
                Ok(None)
            }
            "snd" | "second" => {
                // snd(Pair(a, b)) → b
                if args.len() != 1 {
                    return Ok(None);
                }
                // Evaluate the argument first
                let evaluated = self.eval_concrete(&args[0])?;

                if let Expression::Operation {
                    name,
                    args: pair_args,
                    ..
                } = &evaluated
                {
                    if name == "Pair" && pair_args.len() == 2 {
                        return Ok(Some(pair_args[1].clone()));
                    }
                }
                Ok(None)
            }
            "list_nth" => {
                // list_nth([a, b, c], 1) → b
                // Index into a list (0-based)
                if args.len() != 2 {
                    return Ok(None);
                }

                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[0])?;

                if let Expression::List(elements) = &evaluated_list {
                    if let Some(idx) = self.as_number(&args[1]) {
                        let idx = idx as usize;
                        if idx < elements.len() {
                            return Ok(Some(elements[idx].clone()));
                        }
                    }
                }
                Ok(None)
            }
            "list_length" => {
                // list_length([a, b, c]) → 3
                if args.len() != 1 {
                    return Ok(None);
                }

                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[0])?;

                if let Expression::List(elements) = &evaluated_list {
                    return Ok(Some(Expression::Const(elements.len().to_string())));
                }
                Ok(None)
            }
            "list_concat" | "list_append" => {
                // list_concat([a, b], [c, d]) → [a, b, c, d]
                if args.len() != 2 {
                    return Ok(None);
                }
                // Evaluate both list arguments first
                let evaluated_xs = self.eval_concrete(&args[0])?;
                let evaluated_ys = self.eval_concrete(&args[1])?;

                if let (Expression::List(xs), Expression::List(ys)) = (&evaluated_xs, &evaluated_ys)
                {
                    let mut result = xs.clone();
                    result.extend(ys.clone());
                    return Ok(Some(Expression::List(result)));
                }
                Ok(None)
            }
            "list_flatten" | "list_join" => {
                // list_flatten([[a, b], [c, d]]) → [a, b, c, d]
                if args.len() != 1 {
                    return Ok(None);
                }
                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[0])?;

                if let Expression::List(outer) = &evaluated_list {
                    let mut result = Vec::new();
                    for item in outer {
                        // Also evaluate each inner item in case it's unevaluated
                        let evaluated_item = self.eval_concrete(item)?;
                        if let Expression::List(inner) = evaluated_item {
                            result.extend(inner);
                        } else {
                            result.push(evaluated_item);
                        }
                    }
                    return Ok(Some(Expression::List(result)));
                }
                Ok(None)
            }
            "list_slice" => {
                // list_slice([a, b, c, d], 1, 3) → [b, c] (from index 1 up to but not including 3)
                if args.len() < 2 {
                    return Ok(None);
                }
                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[0])?;

                if let Expression::List(elements) = &evaluated_list {
                    let start = if args.len() >= 2 {
                        self.extract_f64(&args[1]).unwrap_or(0.0) as usize
                    } else {
                        0
                    };
                    let end = if args.len() >= 3 {
                        self.extract_f64(&args[2]).unwrap_or(elements.len() as f64) as usize
                    } else {
                        elements.len()
                    };
                    let end = end.min(elements.len());
                    let start = start.min(end);
                    return Ok(Some(Expression::List(elements[start..end].to_vec())));
                }
                Ok(None)
            }
            "list_rotate" => {
                // list_rotate([a, b, c], 1) → [b, c, a] (rotate left by 1)
                if args.len() != 2 {
                    return Ok(None);
                }
                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[0])?;

                if let Expression::List(elements) = &evaluated_list {
                    let n = self.extract_f64(&args[1]).unwrap_or(0.0) as usize;
                    if elements.is_empty() {
                        return Ok(Some(Expression::List(vec![])));
                    }
                    let n = n % elements.len();
                    let mut result = elements[n..].to_vec();
                    result.extend(elements[..n].to_vec());
                    return Ok(Some(Expression::List(result)));
                }
                Ok(None)
            }

            // ============================================
            // MATH FUNCTIONS
            // ============================================
            "range" => {
                // range(n) → [0, 1, 2, ..., n-1]
                // range(start, end) → [start, start+1, ..., end-1]
                if args.is_empty() {
                    return Ok(None);
                }
                let (start, end) = if args.len() == 1 {
                    (0, self.extract_f64(&args[0])? as i64)
                } else {
                    (
                        self.extract_f64(&args[0])? as i64,
                        self.extract_f64(&args[1])? as i64,
                    )
                };
                let result: Vec<Expression> = (start..end)
                    .map(|i| Expression::Const(i.to_string()))
                    .collect();
                Ok(Some(Expression::List(result)))
            }
            "linspace" => {
                // linspace(start, end) → 50 evenly spaced values (default)
                // linspace(start, end, count) → count evenly spaced values
                if args.len() < 2 {
                    return Err("linspace requires at least start and end".to_string());
                }
                let start = self.extract_f64(&args[0])?;
                let end = self.extract_f64(&args[1])?;
                let count = if args.len() >= 3 {
                    self.extract_f64(&args[2])? as usize
                } else {
                    50 // Default like numpy/Lilaq
                };
                if count == 0 {
                    return Ok(Some(Expression::List(vec![])));
                }
                if count == 1 {
                    return Ok(Some(Expression::List(vec![Expression::Const(
                        start.to_string(),
                    )])));
                }
                let step = (end - start) / (count - 1) as f64;
                let result: Vec<Expression> = (0..count)
                    .map(|i| Expression::Const((start + i as f64 * step).to_string()))
                    .collect();
                Ok(Some(Expression::List(result)))
            }
            "random" | "random_uniform" => {
                // random(count) → list of pseudo-random values in [0, 1]
                // random(count, seed) → reproducible random values
                // Uses a simple LCG for reproducibility
                if args.is_empty() {
                    return Err("random requires count".to_string());
                }
                let count = self.extract_f64(&args[0])? as usize;
                let seed = if args.len() >= 2 {
                    self.extract_f64(&args[1])? as u64
                } else {
                    42 // Default seed
                };
                // Simple LCG: x_{n+1} = (a * x_n + c) mod m
                let a: u64 = 1664525;
                let c: u64 = 1013904223;
                let m: u64 = 1 << 32;
                let mut x = seed;
                let result: Vec<Expression> = (0..count)
                    .map(|_| {
                        x = (a.wrapping_mul(x).wrapping_add(c)) % m;
                        Expression::Const((x as f64 / m as f64).to_string())
                    })
                    .collect();
                Ok(Some(Expression::List(result)))
            }
            "random_normal" => {
                // random_normal(count) → list of pseudo-random values from N(0, 1)
                // random_normal(count, seed) → reproducible
                // random_normal(count, seed, scale) → N(0, scale)
                // Uses Box-Muller transform
                if args.is_empty() {
                    return Err("random_normal requires count".to_string());
                }
                let count = self.extract_f64(&args[0])? as usize;
                let seed = if args.len() >= 2 {
                    self.extract_f64(&args[1])? as u64
                } else {
                    42
                };
                let scale = if args.len() >= 3 {
                    self.extract_f64(&args[2])?
                } else {
                    1.0
                };
                // Simple LCG
                let a: u64 = 1664525;
                let c: u64 = 1013904223;
                let m: u64 = 1 << 32;
                let mut x = seed;
                let mut uniform = || {
                    x = (a.wrapping_mul(x).wrapping_add(c)) % m;
                    (x as f64 / m as f64).max(1e-10) // Avoid log(0)
                };
                // Box-Muller transform
                let mut result: Vec<Expression> = Vec::with_capacity(count);
                while result.len() < count {
                    let u1 = uniform();
                    let u2 = uniform();
                    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                    let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
                    result.push(Expression::Const((z0 * scale).to_string()));
                    if result.len() < count {
                        result.push(Expression::Const((z1 * scale).to_string()));
                    }
                }
                Ok(Some(Expression::List(result)))
            }
            "vec_add" => {
                // Element-wise vector addition: vec_add([a, b], [c, d]) = [a+c, b+d]
                if args.len() != 2 {
                    return Err("vec_add requires two lists".to_string());
                }
                let list1 = self.extract_number_list_v2(&args[0])?;
                let list2 = self.extract_number_list_v2(&args[1])?;
                if list1.len() != list2.len() {
                    return Err("vec_add: lists must have same length".to_string());
                }
                let result: Vec<Expression> = list1
                    .iter()
                    .zip(list2.iter())
                    .map(|(a, b)| Expression::Const((a + b).to_string()))
                    .collect();
                Ok(Some(Expression::List(result)))
            }
            "cos" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.cos().to_string())))
            }
            "sin" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.sin().to_string())))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.sqrt().to_string())))
            }
            "pi" => Ok(Some(Expression::Const(std::f64::consts::PI.to_string()))),
            "deg_to_rad" | "radians" => {
                // Convert degrees to radians
                if args.len() != 1 {
                    return Ok(None);
                }
                let deg = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(deg.to_radians().to_string())))
            }

            // ============================================
            // TRANSCENDENTAL FUNCTIONS (Concrete Evaluation)
            // ============================================
            // Using Rust's std::f64 - IEEE 754 compliant, < 1-2 ULP accuracy
            // Same accuracy as NumPy, MATLAB, Julia

            // Trigonometric functions (radians)
            "tan" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.tan().to_string())))
            }
            "asin" | "arcsin" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.asin().to_string())))
            }
            "acos" | "arccos" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.acos().to_string())))
            }
            "atan" | "arctan" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.atan().to_string())))
            }
            "atan2" | "arctan2" => {
                // atan2(y, x) - 2-argument arctangent
                if args.len() != 2 {
                    return Ok(None);
                }
                let y = self.extract_f64(&args[0])?;
                let x = self.extract_f64(&args[1])?;
                Ok(Some(Expression::Const(y.atan2(x).to_string())))
            }

            // Hyperbolic functions
            "sinh" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.sinh().to_string())))
            }
            "cosh" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.cosh().to_string())))
            }
            "tanh" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.tanh().to_string())))
            }
            "asinh" | "arcsinh" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.asinh().to_string())))
            }
            "acosh" | "arccosh" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.acosh().to_string())))
            }
            "atanh" | "arctanh" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.atanh().to_string())))
            }

            // Exponential and logarithmic functions
            "exp" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.exp().to_string())))
            }
            "exp2" => {
                // 2^x
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.exp2().to_string())))
            }
            "ln" | "log" => {
                // Natural logarithm (base e)
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.ln().to_string())))
            }
            "log10" => {
                // Base-10 logarithm
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.log10().to_string())))
            }
            "log2" => {
                // Base-2 logarithm
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.log2().to_string())))
            }
            "pow" | "power" => {
                // x^y
                if args.len() != 2 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                let y = self.extract_f64(&args[1])?;
                Ok(Some(Expression::Const(x.powf(y).to_string())))
            }

            // Utility functions
            "abs" | "fabs" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.abs().to_string())))
            }
            "floor" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.floor().to_string())))
            }
            "ceil" | "ceiling" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.ceil().to_string())))
            }
            "round" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.round().to_string())))
            }
            "trunc" | "truncate" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.trunc().to_string())))
            }
            "frac" | "fract" => {
                // Fractional part of x (x - floor(x))
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.fract().to_string())))
            }
            "sign" | "signum" => {
                // Sign of x: -1.0, 0.0, or 1.0
                if args.len() != 1 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                Ok(Some(Expression::Const(x.signum().to_string())))
            }
            "fmod" | "remainder" => {
                // Floating-point remainder (x mod y)
                // Note: "mod" is already handled earlier in this match
                if args.len() != 2 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                let y = self.extract_f64(&args[1])?;
                Ok(Some(Expression::Const((x % y).to_string())))
            }
            "min" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                let a = self.extract_f64(&args[0])?;
                let b = self.extract_f64(&args[1])?;
                Ok(Some(Expression::Const(a.min(b).to_string())))
            }
            "max" => {
                if args.len() != 2 {
                    return Ok(None);
                }
                let a = self.extract_f64(&args[0])?;
                let b = self.extract_f64(&args[1])?;
                Ok(Some(Expression::Const(a.max(b).to_string())))
            }
            "hypot" => {
                // sqrt(x² + y²) computed stably
                if args.len() != 2 {
                    return Ok(None);
                }
                let x = self.extract_f64(&args[0])?;
                let y = self.extract_f64(&args[1])?;
                Ok(Some(Expression::Const(x.hypot(y).to_string())))
            }
            "e" => {
                // Euler's number
                Ok(Some(Expression::Const(std::f64::consts::E.to_string())))
            }
            "tau" => {
                // τ = 2π
                Ok(Some(Expression::Const(std::f64::consts::TAU.to_string())))
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

            // === ODE Solver ===
            // For now, ode45 takes explicit derivative expressions
            // Example: ode45([v, -x], [x, v], [1, 0], [0, 10], 0.1)
            //          dynamics,    vars,   y0,     t_span, dt
            "ode45" | "integrate" => self.builtin_ode45(args),

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

    // === Plotting helpers ===

    // =========================================================================
    // COMPOSITIONAL PLOTTING API (matches Lilaq 1:1)
    // =========================================================================

    /// diagram(options, element1, element2, ...) - Compose plot elements and render to SVG
    fn builtin_diagram(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::plotting::{compile_diagram, DiagramOptions, PlotElement};

        if args.is_empty() {
            return Err("diagram() requires at least one plot element".to_string());
        }

        let mut options = DiagramOptions::default();
        let mut elements: Vec<PlotElement> = Vec::new();

        // v0.96: Named arguments produce a trailing record
        // Check both first arg (legacy) and last arg (v0.96 style)
        let mut start_idx = 0;
        let mut end_idx = args.len();

        // Check first arg for options record (legacy style)
        if let Some(Expression::Operation {
            name, args: opts, ..
        }) = args.first()
        {
            if name == "record" {
                self.parse_diagram_options(opts, &mut options)?;
                start_idx = 1;
            }
        }

        // Check last arg for options record (v0.96 named arguments style)
        if end_idx > start_idx {
            if let Some(Expression::Operation {
                name, args: opts, ..
            }) = args.last()
            {
                if name == "record" {
                    self.parse_diagram_options(opts, &mut options)?;
                    end_idx = args.len() - 1;
                }
            }
        }

        // Collect plot elements from middle args
        for arg in &args[start_idx..end_idx] {
            let evaluated = self.eval_concrete(arg)?;

            // Handle lists of PlotElements (for dynamic generation with list_map)
            if let Expression::List(list_elements) = &evaluated {
                for list_elem in list_elements {
                    if let Expression::Operation { name, .. } = list_elem {
                        if name == "PlotElement" {
                            let element = self.decode_plot_element(list_elem)?;
                            elements.push(element);
                        }
                    }
                }
                continue;
            }

            if let Expression::Operation {
                name,
                args: _elem_args,
                ..
            } = &evaluated
            {
                if name == "PlotElement" {
                    // Decode PlotElement from expression
                    let element = self.decode_plot_element(&evaluated)?;
                    elements.push(element);
                } else if name == "record" {
                    // Skip stray records (already processed)
                    continue;
                } else {
                    return Err(format!(
                        "diagram() expects PlotElement, got: {}(). Use plot(), bar(), scatter(), etc.",
                        name
            ));
                }
            } else {
                return Err(format!(
                    "diagram() expects PlotElement, got: {:?}",
                    evaluated
                ));
            }
        }

        if elements.is_empty() {
            return Err("diagram() requires at least one plot element".to_string());
        }

        // Compile to SVG (do not print raw SVG to stdout to avoid polluting Typst output)
        match compile_diagram(&elements, &options) {
            Ok(output) => Ok(Some(Expression::operation(
                "PlotSVG",
                vec![
                    Expression::Const(format!("{:.0}", output.width)),
                    Expression::Const(format!("{:.0}", output.height)),
                ],
            ))),
            Err(e) => Err(format!("diagram() failed: {}", e)),
        }
    }

    /// Export a diagram as Typst code (without compiling to SVG)
    ///
    /// Usage: export_typst(plot(...), bar(...), title = "My Plot")
    /// Returns: String containing complete Typst code
    fn builtin_export_typst(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::plotting::{export_diagram_typst, DiagramOptions, PlotElement};

        if args.is_empty() {
            return Err("export_typst() requires at least one plot element".to_string());
        }

        let mut options = DiagramOptions::default();
        let mut elements: Vec<PlotElement> = Vec::new();

        // Parse arguments (same logic as builtin_diagram)
        let mut start_idx = 0;
        let mut end_idx = args.len();

        // Check first arg for options record
        if let Some(Expression::Operation {
            name, args: opts, ..
        }) = args.first()
        {
            if name == "record" {
                self.parse_diagram_options(opts, &mut options)?;
                start_idx = 1;
            }
        }

        // Check last arg for options record (v0.96 named arguments)
        if end_idx > start_idx {
            if let Some(Expression::Operation {
                name, args: opts, ..
            }) = args.last()
            {
                if name == "record" {
                    self.parse_diagram_options(opts, &mut options)?;
                    end_idx = args.len() - 1;
                }
            }
        }

        // Collect plot elements
        for arg in &args[start_idx..end_idx] {
            let evaluated = self.eval_concrete(arg)?;

            if let Expression::List(list_elements) = &evaluated {
                for list_elem in list_elements {
                    if let Expression::Operation { name, .. } = list_elem {
                        if name == "PlotElement" {
                            let element = self.decode_plot_element(list_elem)?;
                            elements.push(element);
                        }
                    }
                }
                continue;
            }

            if let Expression::Operation { name, .. } = &evaluated {
                if name == "PlotElement" {
                    let element = self.decode_plot_element(&evaluated)?;
                    elements.push(element);
                } else if name == "record" {
                    continue;
                } else {
                    return Err(format!(
                        "export_typst() expects PlotElement, got: {}()",
                        name
                    ));
                }
            }
        }

        if elements.is_empty() {
            return Err("export_typst() requires at least one plot element".to_string());
        }

        // Generate Typst code (without compiling)
        let typst_code = export_diagram_typst(&elements, &options);

        // Return as string
        Ok(Some(Expression::String(typst_code)))
    }

    /// Export just the lq.diagram(...) fragment (without preamble)
    ///
    /// Useful for embedding in existing Typst documents
    fn builtin_export_typst_fragment(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        use crate::plotting::{export_diagram_typst_fragment, DiagramOptions, PlotElement};

        if args.is_empty() {
            return Err("export_typst_fragment() requires at least one plot element".to_string());
        }

        let mut options = DiagramOptions::default();
        let mut elements: Vec<PlotElement> = Vec::new();

        // Parse arguments (same logic as builtin_diagram)
        let mut start_idx = 0;
        let mut end_idx = args.len();

        if let Some(Expression::Operation {
            name, args: opts, ..
        }) = args.first()
        {
            if name == "record" {
                self.parse_diagram_options(opts, &mut options)?;
                start_idx = 1;
            }
        }

        if end_idx > start_idx {
            if let Some(Expression::Operation {
                name, args: opts, ..
            }) = args.last()
            {
                if name == "record" {
                    self.parse_diagram_options(opts, &mut options)?;
                    end_idx = args.len() - 1;
                }
            }
        }

        for arg in &args[start_idx..end_idx] {
            let evaluated = self.eval_concrete(arg)?;

            if let Expression::List(list_elements) = &evaluated {
                for list_elem in list_elements {
                    if let Expression::Operation { name, .. } = list_elem {
                        if name == "PlotElement" {
                            let element = self.decode_plot_element(list_elem)?;
                            elements.push(element);
                        }
                    }
                }
                continue;
            }

            if let Expression::Operation { name, .. } = &evaluated {
                if name == "PlotElement" {
                    let element = self.decode_plot_element(&evaluated)?;
                    elements.push(element);
                } else if name == "record" {
                    continue;
                }
            }
        }

        if elements.is_empty() {
            return Err("export_typst_fragment() requires at least one plot element".to_string());
        }

        // Generate Typst fragment (without preamble)
        let typst_code = export_diagram_typst_fragment(&elements, &options);

        Ok(Some(Expression::String(typst_code)))
    }

    /// Generate Typst table code from Kleis data
    ///
    /// Usage: table_typst(headers, rows)
    /// - headers: List of column header strings ["Name", "Age", "Score"]
    /// - rows: List of rows, each row is a list [[a, b, c], [d, e, f]]
    ///
    /// Returns: String containing Typst table code
    fn builtin_table_typst(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err(
                "table_typst() requires 2 arguments: headers (list), rows (list of lists)"
                    .to_string(),
            );
        }

        // Extract headers
        let headers_expr = self.eval_concrete(&args[0])?;
        let headers: Vec<String> = match headers_expr {
            Expression::List(items) => items
                .iter()
                .map(|item| match item {
                    Expression::String(s) => s.clone(),
                    Expression::Const(s) => s.clone(),
                    other => format!("{:?}", other),
                })
                .collect(),
            _ => return Err("table_typst(): first argument must be a list of headers".to_string()),
        };

        // Extract rows
        let rows_expr = self.eval_concrete(&args[1])?;
        let rows: Vec<Vec<String>> =
            match rows_expr {
                Expression::List(row_items) => row_items
                    .iter()
                    .map(|row| match row {
                        Expression::List(cells) => cells
                            .iter()
                            .map(|cell| match cell {
                                Expression::String(s) => s.clone(),
                                Expression::Const(s) => s.clone(),
                                other => format!("{:?}", other),
                            })
                            .collect(),
                        _ => vec![format!("{:?}", row)],
                    })
                    .collect(),
                _ => return Err(
                    "table_typst(): second argument must be a list of rows (each row is a list)"
                        .to_string(),
                ),
            };

        // Build Typst table code (no # prefix - for embedding in figures)
        let num_cols = headers.len();
        let mut code = format!("table(\n  columns: {},\n", num_cols);

        // Add headers
        for (i, header) in headers.iter().enumerate() {
            code.push_str(&format!("  [{}]", header));
            if i < num_cols - 1 {
                code.push_str(", ");
            }
        }
        code.push_str(",\n");

        // Add rows
        for row in &rows {
            code.push_str("  ");
            for (i, cell) in row.iter().enumerate() {
                code.push_str(&format!("[{}]", cell));
                if i < row.len() - 1 {
                    code.push_str(", ");
                }
            }
            code.push_str(",\n");
        }

        code.push(')');

        Ok(Some(Expression::String(code)))
    }

    /// Generate Typst table code (raw Object) from Kleis data (no quotes, no '#')
    ///
    /// Usage: table_typst_raw(headers, rows)
    /// - headers: List of column header strings ["Name", "Age", "Score"]
    /// - rows: List of rows, each row is a list [[a, b, c], [d, e, f]]
    ///
    /// Returns: Object containing Typst table code (no string quotes, no #)
    fn builtin_table_typst_raw(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err(
                "table_typst_raw() requires 2 arguments: headers (list), rows (list of lists)"
                    .to_string(),
            );
        }

        // Extract headers as strict strings
        let headers_expr = self.eval_concrete(&args[0])?;
        let headers: Vec<String> = match headers_expr {
            Expression::List(items) => items
                .iter()
                .map(|item| self.extract_string(item))
                .collect::<Result<_, _>>()
                .map_err(|e| format!("table_typst_raw headers: {}", e))?,
            _ => {
                return Err(
                    "table_typst_raw(): first argument must be a list of headers".to_string(),
                )
            }
        };

        // Extract rows as list of list of strings
        let rows_expr = self.eval_concrete(&args[1])?;
        let rows: Vec<Vec<String>> = match rows_expr {
            Expression::List(row_items) => row_items
                .iter()
                .map(|row| match row {
                    Expression::List(cells) => cells
                        .iter()
                        .map(|cell| self.extract_string(cell))
                        .collect::<Result<_, _>>()
                        .map_err(|e| format!("table_typst_raw row: {}", e)),
                    _ => Err("Each row must be a list".to_string()),
                })
                .collect::<Result<_, _>>()?,
            _ => {
                return Err("table_typst_raw(): second argument must be list of rows (list)".into())
            }
        };

        let num_cols = headers.len();
        let mut code = format!("table(\n  columns: {},\n", num_cols);

        // Headers
        for (i, header) in headers.iter().enumerate() {
            code.push_str(&format!("  [{}]", header));
            if i < num_cols - 1 {
                code.push_str(", ");
            } else {
                code.push_str(",\n");
            }
        }

        // Rows
        for row in &rows {
            code.push_str("  [");
            for (i, cell) in row.iter().enumerate() {
                code.push_str(&format!("[{}]", cell));
                if i < num_cols - 1 {
                    code.push_str(", ");
                }
            }
            code.push_str("],\n");
        }

        code.push(')');

        Ok(Some(Expression::Object(code)))
    }

    /// Convert a Typst string to a raw object (no quotes/escapes)
    ///
    /// Usage: typst_raw(text_string)
    /// Returns: Object containing the text verbatim
    fn builtin_typst_raw(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }
        let v = self.eval_concrete(&args[0])?;
        match v {
            Expression::String(s) | Expression::Const(s) | Expression::Object(s) => {
                Ok(Some(Expression::Object(self.unescape_basic(&s))))
            }
            other => Err(format!(
                "typst_raw(): expected string/object, got {:?}",
                other
            )),
        }
    }

    /// Concatenate strings/objects into a single string/object
    ///
    /// Usage: concat(a, b, c, ...)
    /// - Accepts String, Const, or Object
    /// - If any arg is Object, result is Object; otherwise String
    fn builtin_concat(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.is_empty() {
            return Ok(None);
        }

        let mut parts = Vec::with_capacity(args.len());
        let mut has_object = false;

        for a in args {
            let v = self.eval_concrete(a)?;
            match v {
                Expression::String(s) | Expression::Const(s) => {
                    parts.push(self.unescape_basic(&s));
                }
                Expression::Object(s) => {
                    has_object = true;
                    parts.push(self.unescape_basic(&s));
                }
                other => {
                    return Err(format!(
                        "concat(): unsupported argument type {:?}, expected string/object",
                        other
                    ))
                }
            }
        }

        let joined = parts.join("");
        if has_object {
            Ok(Some(Expression::Object(joined)))
        } else {
            Ok(Some(Expression::String(joined)))
        }
    }

    /// String equality comparison
    ///
    /// Usage: str_eq(a, b)
    /// Returns: true if a == b (as strings), false otherwise
    fn builtin_str_eq(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() != 2 {
            return Err("str_eq() requires exactly 2 arguments".to_string());
        }

        let a = self.eval_concrete(&args[0])?;
        let b = self.eval_concrete(&args[1])?;

        let a_str = match &a {
            Expression::String(s) | Expression::Const(s) | Expression::Object(s) => s.clone(),
            _ => return Ok(None), // Can't compare non-strings
        };

        let b_str = match &b {
            Expression::String(s) | Expression::Const(s) | Expression::Object(s) => s.clone(),
            _ => return Ok(None), // Can't compare non-strings
        };

        if a_str == b_str {
            Ok(Some(Expression::Object("true".to_string())))
        } else {
            Ok(Some(Expression::Object("false".to_string())))
        }
    }

    /// ODE solver using Dormand-Prince 5(4) method
    ///
    /// Usage: ode45(f, y0, t_span, dt?)
    ///   f: dynamics function (t, y) -> [dy/dt...]
    ///   y0: initial state, e.g., [1, 0]
    ///   t_span: [t0, t1]
    ///   dt: initial step (optional, default 0.1)
    ///
    /// Example:
    ///   // Harmonic oscillator: x'' = -x
    ///   let f = (t, y) => [y[1], neg(y[0])]
    ///   ode45(f, [1, 0], [0, 10], 0.1)
    ///
    /// Returns: list of [t, [y0, y1, ...]] pairs
    fn builtin_ode45(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() < 3 || args.len() > 4 {
            return Err(
                "ode45 requires 3-4 arguments: f, y0, t_span, dt?\n\
                 Example: ode45((t, y) => [y[1], neg(y[0])], [1, 0], [0, 10])"
                    .to_string(),
            );
        }

        // args[0] should be a lambda: (t, y) => ...
        let f_lambda = &args[0];

        // Extract initial state y0
        let y0: Vec<f64> = if let Expression::List(elems) = &args[1] {
            elems
                .iter()
                .map(|e| {
                    self.as_number(e)
                        .ok_or_else(|| "y0 must be numeric".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            return Err("y0 must be a list".to_string());
        };

        // Extract time span [t0, t1]
        let (t0, t1) = if let Expression::List(elems) = &args[2] {
            if elems.len() != 2 {
                return Err("t_span must be [t0, t1]".to_string());
            }
            let t0 = self
                .as_number(&elems[0])
                .ok_or_else(|| "t0 must be numeric".to_string())?;
            let t1 = self
                .as_number(&elems[1])
                .ok_or_else(|| "t1 must be numeric".to_string())?;
            (t0, t1)
        } else {
            return Err("t_span must be [t0, t1]".to_string());
        };

        // Extract dt (optional)
        let dt = if args.len() == 4 {
            self.as_number(&args[3])
                .ok_or_else(|| "dt must be numeric".to_string())?
        } else {
            0.1
        };

        let dim = y0.len();
        let f_clone = f_lambda.clone();

        // Create dynamics function that calls the lambda
        let dynamics = |t: f64, y: &[f64]| -> Vec<f64> {
            // Build: f(t, [y0, y1, ...])
            let t_expr = Expression::Const(format!("{}", t));
            let y_expr = Expression::List(
                y.iter()
                    .map(|&v| Expression::Const(format!("{}", v)))
                    .collect(),
            );

            // Apply lambda: substitute params with args and evaluate body
            if let Expression::Lambda { params, body, .. } = &f_clone {
                if params.len() >= 2 {
                    let mut subst = std::collections::HashMap::new();
                    subst.insert(params[0].name.clone(), t_expr);
                    subst.insert(params[1].name.clone(), y_expr);

                    // Substitute and evaluate
                    let substituted = Self::substitute_simple(body, &subst);
                    if let Some(result) = Self::eval_numeric_expr(&substituted) {
                        return result;
                    }
                }
            }
            vec![0.0; dim]
        };

        // Integrate
        let result = crate::ode::integrate_dopri5(dynamics, &y0, (t0, t1), dt)
            .map_err(|e| e.to_string())?;

        // Convert to Kleis list of [t, [y...]]
        let trajectory: Vec<Expression> = result
            .into_iter()
            .map(|(t, y)| {
                Expression::List(vec![
                    Expression::Const(format!("{}", t)),
                    Expression::List(
                        y.into_iter()
                            .map(|v| Expression::Const(format!("{}", v)))
                            .collect(),
                    ),
                ])
            })
            .collect();

        Ok(Some(Expression::List(trajectory)))
    }

    /// Simple substitution for ODE evaluation (doesn't need full evaluator)
    fn substitute_simple(
        expr: &Expression,
        subst: &std::collections::HashMap<String, Expression>,
    ) -> Expression {
        match expr {
            Expression::Object(name) => subst.get(name).cloned().unwrap_or_else(|| expr.clone()),
            Expression::Operation { name, args, span } => {
                // Check if operation name is a variable (like indexing)
                if name == "index" && args.len() == 2 {
                    // Handle y[i] => get element from list
                    let arr = Self::substitute_simple(&args[0], subst);
                    let idx_expr = Self::substitute_simple(&args[1], subst);
                    if let (Expression::List(elems), Expression::Const(idx_str)) = (&arr, &idx_expr)
                    {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            if idx < elems.len() {
                                return elems[idx].clone();
                            }
                        }
                    }
                }
                Expression::Operation {
                    name: name.clone(),
                    args: args
                        .iter()
                        .map(|a| Self::substitute_simple(a, subst))
                        .collect(),
                    span: span.clone(),
                }
            }
            Expression::List(elems) => Expression::List(
                elems
                    .iter()
                    .map(|e| Self::substitute_simple(e, subst))
                    .collect(),
            ),
            _ => expr.clone(),
        }
    }

    /// Evaluate expression to numeric vector (for ODE)
    fn eval_numeric_expr(expr: &Expression) -> Option<Vec<f64>> {
        match expr {
            Expression::List(elems) => elems.iter().map(eval_numeric).collect(),
            _ => None,
        }
    }

    /// Render an EditorNode AST to Typst code
    ///
    /// Usage: render_to_typst(ast)
    /// Usage: render_to_typst(ast, "typst")  // or "latex", "unicode"
    /// Returns: String containing Typst (or other format) code
    ///
    /// The ast should be a Kleis EditorNode expression, e.g.:
    ///   binop("equals", sym("E"), binop("times", sym("m"), sup(sym("c"), num("2"))))
    fn builtin_render_to_typst(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        use crate::render::RenderTarget;
        use crate::render_editor::render_editor_node;

        if args.is_empty() {
            return Err("render_to_typst() requires an EditorNode argument".to_string());
        }

        // First argument is the EditorNode AST
        let ast_expr = self.eval_concrete(&args[0])?;

        // Optional second argument: render target (default: Typst)
        let target = if args.len() > 1 {
            let target_str = self.extract_string(&args[1])?;
            match target_str.to_lowercase().as_str() {
                "typst" => RenderTarget::Typst,
                "latex" => RenderTarget::LaTeX,
                "unicode" => RenderTarget::Unicode,
                "html" => RenderTarget::HTML,
                "kleis" => RenderTarget::Kleis,
                _ => {
                    return Err(format!(
                        "Unknown render target: '{}'. Use 'typst', 'latex', 'unicode', 'html', or 'kleis'",
                        target_str
                    ))
                }
            }
        } else {
            RenderTarget::Typst
        };

        // Convert Kleis Expression to Rust EditorNode
        let editor_node = self.expression_to_editor_node(&ast_expr)?;

        // Render to the target format
        let output = render_editor_node(&editor_node, &target);

        Ok(Some(Expression::String(output)))
    }

    /// Convert a Kleis EditorNode expression to a Rust EditorNode struct
    ///
    /// Mapping:
    ///   EObject(symbol)           -> EditorNode::Object { object: symbol }
    ///   EConst(value)             -> EditorNode::Const { value }
    ///   EOp(name, args, kind, _)  -> EditorNode::Operation { ... }
    ///   EList(nodes)              -> EditorNode::List { list }
    ///   EPlaceholder(data)        -> EditorNode::Placeholder { ... }
    fn expression_to_editor_node(
        &self,
        expr: &Expression,
    ) -> Result<crate::editor_ast::EditorNode, String> {
        use crate::editor_ast::{EditorNode, OperationData, PlaceholderData};

        match expr {
            // EObject(symbol) -> Object { object: symbol }
            Expression::Operation { name, args, .. } if name == "EObject" => {
                if args.len() != 1 {
                    return Err("EObject expects 1 argument".to_string());
                }
                let symbol = self.extract_string(&args[0])?;
                Ok(EditorNode::Object { object: symbol })
            }

            // EConst(value) -> Const { value }
            Expression::Operation { name, args, .. } if name == "EConst" => {
                if args.len() != 1 {
                    return Err("EConst expects 1 argument".to_string());
                }
                let value = self.extract_string(&args[0])?;
                Ok(EditorNode::Const { value })
            }

            // EPlaceholder(data) -> Placeholder { placeholder }
            Expression::Operation { name, args, .. } if name == "EPlaceholder" => {
                // For now, create a simple placeholder with id 0
                let id = if !args.is_empty() {
                    if let Expression::Const(s) = &args[0] {
                        s.parse::<usize>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };
                Ok(EditorNode::Placeholder {
                    placeholder: PlaceholderData { id, hint: None },
                })
            }

            // EOp(name, args, kind, meta) -> Operation { operation: OperationData }
            Expression::Operation { name, args, .. } if name == "EOp" => {
                if args.len() < 2 {
                    return Err("EOp expects at least 2 arguments (name, args)".to_string());
                }

                let op_name = self.extract_string(&args[0])?;

                // Convert args list
                let op_args = self.extract_editor_node_list(&args[1])?;

                // Optional kind (3rd arg)
                let kind = if args.len() > 2 {
                    let k = self.extract_string(&args[2]).unwrap_or_default();
                    if k.is_empty() || k == "NoMeta" {
                        None
                    } else {
                        Some(k)
                    }
                } else {
                    None
                };

                // Metadata (4th arg) - for now, ignore complex metadata
                // TODO: Parse TensorMeta, MatrixMeta if needed

                Ok(EditorNode::Operation {
                    operation: OperationData {
                        name: op_name,
                        args: op_args,
                        kind,
                        metadata: None,
                    },
                })
            }

            // EList(nodes) -> List { list }
            Expression::Operation { name, args, .. } if name == "EList" => {
                if args.len() != 1 {
                    return Err("EList expects 1 argument (list of nodes)".to_string());
                }
                let nodes = self.extract_editor_node_list(&args[0])?;
                Ok(EditorNode::List { list: nodes })
            }

            // Handle raw Object as a symbol (for convenience)
            Expression::Object(s) => Ok(EditorNode::Object { object: s.clone() }),

            // Handle raw Const as a constant
            Expression::Const(s) => Ok(EditorNode::Const { value: s.clone() }),

            // Handle raw String as a constant
            Expression::String(s) => Ok(EditorNode::Const { value: s.clone() }),

            _ => Err(format!(
                "Cannot convert expression to EditorNode: {:?}",
                expr
            )),
        }
    }

    /// Extract a list of EditorNodes from a Kleis List expression
    fn extract_editor_node_list(
        &self,
        expr: &Expression,
    ) -> Result<Vec<crate::editor_ast::EditorNode>, String> {
        match expr {
            Expression::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(self.expression_to_editor_node(item)?);
                }
                Ok(result)
            }

            // Handle Cons/Nil list representation
            Expression::Operation { name, args, .. } if name == "Cons" => {
                if args.len() != 2 {
                    return Err("Cons expects 2 arguments".to_string());
                }
                let head = self.expression_to_editor_node(&args[0])?;
                let mut tail = self.extract_editor_node_list(&args[1])?;
                let mut result = vec![head];
                result.append(&mut tail);
                Ok(result)
            }

            Expression::Operation { name, .. } if name == "Nil" => Ok(vec![]),

            Expression::Object(s) if s == "Nil" => Ok(vec![]),

            _ => Err(format!("Expected list of EditorNodes, got: {:?}", expr)),
        }
    }

    /// Parse diagram options from a record expression
    fn parse_diagram_options(
        &self,
        opts: &[Expression],
        options: &mut crate::plotting::DiagramOptions,
    ) -> Result<(), String> {
        for opt in opts {
            if let Expression::Operation { name, args, .. } = opt {
                if name == "field" && args.len() == 2 {
                    let key = self.extract_string(&args[0])?;
                    match key.as_str() {
                        "width" => options.width = Some(self.extract_f64(&args[1])?),
                        "height" => options.height = Some(self.extract_f64(&args[1])?),
                        "title" => options.title = Some(self.extract_string(&args[1])?),
                        "xlabel" => options.xlabel = Some(self.extract_string(&args[1])?),
                        "ylabel" => options.ylabel = Some(self.extract_string(&args[1])?),
                        "xscale" => options.xscale = Some(self.extract_string(&args[1])?),
                        "yscale" => options.yscale = Some(self.extract_string(&args[1])?),
                        "legend" | "legend_position" => {
                            options.legend_position = Some(self.extract_string(&args[1])?)
                        }
                        "grid" => options.grid = Some(self.extract_bool(&args[1])?),
                        "fill" => options.fill = Some(self.extract_string(&args[1])?),
                        "aspect_ratio" => options.aspect_ratio = Some(self.extract_f64(&args[1])?),
                        "xaxis_subticks" | "x_subticks" => {
                            options.xaxis_subticks = Some(self.extract_string(&args[1])?)
                        }
                        "yaxis_subticks" | "y_subticks" => {
                            options.yaxis_subticks = Some(self.extract_string(&args[1])?)
                        }
                        "yaxis_mirror" | "y_mirror" => {
                            options.yaxis_mirror = Some(self.extract_bool(&args[1])?)
                        }
                        "margin_top" => options.margin_top = Some(self.extract_string(&args[1])?),
                        "margin_bottom" => {
                            options.margin_bottom = Some(self.extract_string(&args[1])?)
                        }
                        "margin_left" => options.margin_left = Some(self.extract_string(&args[1])?),
                        "margin_right" => {
                            options.margin_right = Some(self.extract_string(&args[1])?)
                        }
                        "xaxis_ticks" | "x_ticks" => {
                            options.xaxis_ticks = Some(self.extract_string_list(&args[1])?)
                        }
                        "xaxis_tick_rotate" | "x_tick_rotate" => {
                            options.xaxis_tick_rotate = Some(self.extract_f64(&args[1])?)
                        }
                        "xaxis_ticks_none" | "hide_xaxis_ticks" => {
                            options.xaxis_ticks_none = Some(self.extract_bool(&args[1])?)
                        }
                        "yaxis_ticks_none" | "hide_yaxis_ticks" => {
                            options.yaxis_ticks_none = Some(self.extract_bool(&args[1])?)
                        }
                        "xaxis_tick_unit" | "x_tick_unit" => {
                            options.xaxis_tick_unit = Some(self.extract_f64(&args[1])?)
                        }
                        "xaxis_tick_suffix" | "x_tick_suffix" => {
                            options.xaxis_tick_suffix = Some(self.extract_string(&args[1])?)
                        }
                        "yaxis_tick_unit" | "y_tick_unit" => {
                            options.yaxis_tick_unit = Some(self.extract_f64(&args[1])?)
                        }
                        "yaxis_tick_suffix" | "y_tick_suffix" => {
                            options.yaxis_tick_suffix = Some(self.extract_string(&args[1])?)
                        }
                        "xlim" | "x_lim" => {
                            let limits = self.extract_f64_list_from_diagram_option(&args[1])?;
                            if limits.len() >= 2 {
                                options.xlim = Some((limits[0], limits[1]));
                            }
                        }
                        "ylim" | "y_lim" => {
                            let limits = self.extract_f64_list_from_diagram_option(&args[1])?;
                            if limits.len() >= 2 {
                                options.ylim = Some((limits[0], limits[1]));
                            }
                        }
                        "theme" => options.theme = Some(self.extract_string(&args[1])?),
                        _ => {} // Ignore unknown options
                    }
                }
            }
        }
        Ok(())
    }

    /// plot(xs, ys), bar(xs, heights), etc. - Create a PlotElement (not rendered yet)
    fn builtin_plot_element(
        &self,
        args: &[Expression],
        plot_type: crate::plotting::PlotType,
    ) -> Result<Option<Expression>, String> {
        use crate::plotting::PlotType;

        if args.len() < 2 {
            return Err(format!(
                "{}() requires at least 2 arguments: x_data, y_data",
                match plot_type {
                    PlotType::Line => "plot",
                    PlotType::Scatter => "scatter",
                    PlotType::Bar => "bar",
                    PlotType::HBar => "hbar",
                    PlotType::Stem => "stem",
                    PlotType::HStem => "hstem",
                    PlotType::FillBetween => "fill_between",
                    _ => "plot",
                }
            ));
        }

        let x_data = self.extract_number_list_v2(&args[0])?;
        let y_data = self.extract_number_list_v2(&args[1])?;

        if x_data.len() != y_data.len() {
            return Err(format!(
                "x_data and y_data must have same length (got {} and {})",
                x_data.len(),
                y_data.len()
            ));
        }

        // Build PlotElement expression with encoded data
        let mut element_args = vec![
            Expression::Const(format!("{:?}", plot_type)),
            self.encode_f64_list(&x_data),
            self.encode_f64_list(&y_data),
        ];

        // Parse options if present
        if args.len() >= 3 {
            element_args.push(args[2].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// boxplot(data...) - Create a boxplot PlotElement
    fn builtin_boxplot_element(
        &self,
        args: &[Expression],
        horizontal: bool,
    ) -> Result<Option<Expression>, String> {
        if args.is_empty() {
            return Err("boxplot() requires at least one dataset".to_string());
        }

        // Extract datasets
        let mut datasets = Vec::new();
        for arg in args {
            let data = self.extract_number_list_v2(arg)?;
            datasets.push(data);
        }

        let plot_type = if horizontal {
            crate::plotting::PlotType::HBoxplot
        } else {
            crate::plotting::PlotType::Boxplot
        };

        // Encode datasets as nested list
        let datasets_expr =
            Expression::List(datasets.iter().map(|d| self.encode_f64_list(d)).collect());

        Ok(Some(Expression::operation(
            "PlotElement",
            vec![Expression::Const(format!("{:?}", plot_type)), datasets_expr],
        )))
    }

    /// heatmap(matrix) or contour(matrix) - Create matrix-based PlotElement
    fn builtin_matrix_element(
        &self,
        args: &[Expression],
        plot_type: crate::plotting::PlotType,
    ) -> Result<Option<Expression>, String> {
        if args.is_empty() {
            return Err("heatmap/contour() requires a matrix".to_string());
        }

        let matrix = self.extract_f64_matrix(&args[0])?;

        // Encode matrix
        let matrix_expr =
            Expression::List(matrix.iter().map(|row| self.encode_f64_list(row)).collect());

        let mut element_args = vec![Expression::Const(format!("{:?}", plot_type)), matrix_expr];

        // Parse options if present
        if args.len() >= 2 {
            element_args.push(args[1].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// quiver(xs, ys, directions) - Create vector field PlotElement
    fn builtin_quiver_element(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() < 3 {
            return Err("quiver() requires: x_coords, y_coords, directions".to_string());
        }

        let x_coords = self.extract_number_list_v2(&args[0])?;
        let y_coords = self.extract_number_list_v2(&args[1])?;
        let dir_matrix = self.extract_f64_matrix(&args[2])?;

        // Encode data
        let x_expr = self.encode_f64_list(&x_coords);
        let y_expr = self.encode_f64_list(&y_coords);
        let dir_expr = Expression::List(
            dir_matrix
                .iter()
                .map(|row| self.encode_f64_list(row))
                .collect(),
        );

        let mut element_args = vec![
            Expression::Const("Quiver".to_string()),
            x_expr,
            y_expr,
            dir_expr,
        ];

        // Parse options if present
        if args.len() >= 4 {
            element_args.push(args[3].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// place(x, y, text, align = "top") - Text annotation at coordinates
    fn builtin_place_element(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() < 3 {
            return Err("place() requires: x, y, text".to_string());
        }

        let x = self.extract_f64(&args[0])?;
        let y = self.extract_f64(&args[1])?;
        let text = self.extract_string(&args[2])?;

        let mut element_args = vec![
            Expression::Const("Place".to_string()),
            self.encode_f64_list(&[x]),
            self.encode_f64_list(&[y]),
            Expression::String(text),
        ];

        // Parse options if present (trailing record from named arguments)
        if args.len() >= 4 {
            element_args.push(args[3].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// yaxis(position = "right", label = "...", child_elements...) - Secondary y-axis
    fn builtin_yaxis_element(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        // yaxis can contain child plot elements and options
        // yaxis(bar(...), plot(...), position = "right", label = "...")

        let mut child_elements: Vec<Expression> = Vec::new();
        let mut options_record: Option<Expression> = None;

        for arg in args {
            let evaluated = self.eval_concrete(arg)?;

            // Check for options record
            if let Expression::Operation { name, .. } = &evaluated {
                if name == "record" {
                    options_record = Some(evaluated.clone());
                    continue;
                }
                if name == "PlotElement" {
                    child_elements.push(evaluated);
                    continue;
                }
            }

            // Try to evaluate as a plot element
            if let Expression::Operation { name, .. } = &arg {
                if name == "PlotElement" {
                    child_elements.push(arg.clone());
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("SecondaryYAxis".to_string()),
            Expression::List(child_elements),
        ];

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// xaxis(position = "top", label = "...", functions = ("x => k/x", "x => k/x")) - Secondary x-axis
    fn builtin_xaxis_element(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        // xaxis can contain child plot elements and options
        // xaxis(plot(...), position = "top", label = "Energy (eV)", functions = ...)

        let mut child_elements: Vec<Expression> = Vec::new();
        let mut options_record: Option<Expression> = None;

        for arg in args {
            let evaluated = self.eval_concrete(arg)?;

            // Check for options record
            if let Expression::Operation { name, .. } = &evaluated {
                if name == "record" {
                    options_record = Some(evaluated.clone());
                    continue;
                }
                if name == "PlotElement" {
                    child_elements.push(evaluated);
                    continue;
                }
            }

            // Try to evaluate as a plot element
            if let Expression::Operation { name, .. } = &arg {
                if name == "PlotElement" {
                    child_elements.push(arg.clone());
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("SecondaryXAxis".to_string()),
            Expression::List(child_elements),
        ];

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// path(points, fill = "blue", closed = true) - Arbitrary path for polygons/fractals
    fn builtin_path_element(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        // path can take:
        // - A list of (x, y) pairs: path([(0,0), (1,1), (2,0)])
        // - Separate x and y lists: path(xs, ys)
        // - Options: fill, stroke, closed

        let mut x_data: Vec<f64> = Vec::new();
        let mut y_data: Vec<f64> = Vec::new();
        let mut options_record: Option<Expression> = None;

        for arg in args {
            let evaluated = self.eval_concrete(arg)?;

            // Check for options record
            if let Expression::Operation { name, .. } = &evaluated {
                if name == "record" {
                    options_record = Some(evaluated.clone());
                    continue;
                }
            }

            // Check for list of pairs
            if let Expression::List(items) = &evaluated {
                if !items.is_empty() {
                    // Check if first item is a Pair
                    if let Expression::Operation {
                        name,
                        args: pair_args,
                        ..
                    } = self.eval_concrete(&items[0])?
                    {
                        if name == "Pair" && pair_args.len() == 2 {
                            // It's a list of pairs
                            for item in items {
                                if let Expression::Operation {
                                    name: n,
                                    args: p_args,
                                    ..
                                } = self.eval_concrete(item)?
                                {
                                    if n == "Pair" && p_args.len() == 2 {
                                        x_data.push(self.extract_f64(&p_args[0])?);
                                        y_data.push(self.extract_f64(&p_args[1])?);
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    // Otherwise it might be a list of numbers (x or y data)
                    let nums: Result<Vec<f64>, _> =
                        items.iter().map(|e| self.extract_f64(e)).collect();
                    if let Ok(nums) = nums {
                        if x_data.is_empty() {
                            x_data = nums;
                        } else {
                            y_data = nums;
                        }
                    }
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("Path".to_string()),
            self.encode_f64_list(&x_data),
            self.encode_f64_list(&y_data),
        ];

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// fill_between(xs, y1, y2 = ..., fill = ...) - Shaded area between curves
    fn builtin_fill_between_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err("fill_between() requires at least x and y data".to_string());
        }

        let x_data = self.extract_number_list_v2(&args[0])?;
        let y_data = self.extract_number_list_v2(&args[1])?;

        // Check for y2 parameter in options or as third positional arg
        let mut y2_data: Option<Vec<f64>> = None;
        let mut options_record: Option<Expression> = None;

        for (i, arg) in args.iter().enumerate().skip(2) {
            let evaluated = self.eval_concrete(arg)?;
            if let Expression::Operation {
                ref name, ref args, ..
            } = evaluated
            {
                if name == "record" {
                    // Check for y2 in the record
                    for field_arg in args {
                        if let Expression::Operation {
                            name: fname,
                            args: fargs,
                            ..
                        } = field_arg
                        {
                            if fname == "field" && fargs.len() >= 2 {
                                if let Expression::Const(key) = &fargs[0] {
                                    if key == "y2" {
                                        y2_data = Some(self.extract_number_list_v2(&fargs[1])?);
                                    }
                                }
                            }
                        }
                    }
                    options_record = Some(evaluated);
                }
            } else if i == 2 && y2_data.is_none() {
                // Third positional arg might be y2 array
                if let Ok(y2) = self.extract_number_list_v2(&evaluated) {
                    y2_data = Some(y2);
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("FillBetween".to_string()),
            self.encode_f64_list(&x_data),
            self.encode_f64_list(&y_data),
        ];

        // Add y2 if present
        if let Some(ref y2) = y2_data {
            element_args.push(self.encode_f64_list(y2));
        } else {
            element_args.push(Expression::List(vec![])); // Empty placeholder
        }

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// stacked_area(xs, ys1, ys2, ys3, ...) - Create stacked area chart
    fn builtin_stacked_area(&self, args: &[Expression]) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err("stacked_area() requires x data and at least one y series".to_string());
        }

        let x_data = self.extract_number_list_v2(&args[0])?;
        let n = x_data.len();

        // Collect all y series
        let mut y_series: Vec<Vec<f64>> = Vec::new();
        for arg in args.iter().skip(1) {
            let evaluated = self.eval_concrete(arg)?;
            // Skip options records
            if let Expression::Operation { ref name, .. } = evaluated {
                if name == "record" {
                    continue;
                }
            }
            if let Ok(ys) = self.extract_number_list_v2(&evaluated) {
                if ys.len() == n {
                    y_series.push(ys);
                }
            }
        }

        if y_series.is_empty() {
            return Err("stacked_area() requires at least one y series".to_string());
        }

        // Compute cumulative sums (stacked values)
        let mut stacked: Vec<Vec<f64>> = vec![vec![0.0; n]]; // Start with zeros
        for ys in &y_series {
            let prev = stacked.last().unwrap();
            let new_stack: Vec<f64> = prev.iter().zip(ys.iter()).map(|(a, b)| a + b).collect();
            stacked.push(new_stack);
        }

        // Create fill-between elements for each layer
        // stacked[i] to stacked[i+1] for i in 0..y_series.len()
        let mut fill_elements: Vec<Expression> = Vec::new();

        // Default colors for stacked areas
        let colors = [
            "#5B8FB9", "#E19F8F", "#B5651D", "#7CB342", "#9C27B0", "#FF9800",
        ];

        for i in 0..y_series.len() {
            let y1 = &stacked[i];
            let y2 = &stacked[i + 1];
            let color = colors[i % colors.len()];

            let element_args = vec![
                Expression::Const("FillBetween".to_string()),
                self.encode_f64_list(&x_data),
                self.encode_f64_list(y1),
                self.encode_f64_list(y2),
                // Options record with fill color
                Expression::operation(
                    "record",
                    vec![Expression::operation(
                        "field",
                        vec![
                            Expression::Const("fill".to_string()),
                            Expression::Const(format!("rgb(\"{}\")", color)),
                        ],
                    )],
                ),
            ];
            fill_elements.push(Expression::operation("PlotElement", element_args));
        }

        // Return as a list of PlotElements
        Ok(Some(Expression::List(fill_elements)))
    }

    /// Encode a list of f64 as an Expression::List
    fn encode_f64_list(&self, data: &[f64]) -> Expression {
        Expression::List(
            data.iter()
                .map(|&v| Expression::Const(v.to_string()))
                .collect(),
        )
    }

    /// Decode a PlotElement expression back to a PlotElement struct
    fn decode_plot_element(
        &self,
        expr: &Expression,
    ) -> Result<crate::plotting::PlotElement, String> {
        use crate::plotting::{PlotElement, PlotElementOptions, PlotType};

        if let Expression::Operation { name, args, .. } = expr {
            if name != "PlotElement" {
                return Err(format!("Expected PlotElement, got {}", name));
            }

            if args.is_empty() {
                return Err("PlotElement has no arguments".to_string());
            }

            // First arg is the type
            let type_str = self.extract_string(&args[0])?;
            let element_type = match type_str.as_str() {
                "Line" => PlotType::Line,
                "Scatter" => PlotType::Scatter,
                "Bar" => PlotType::Bar,
                "HBar" => PlotType::HBar,
                "Stem" => PlotType::Stem,
                "HStem" => PlotType::HStem,
                "FillBetween" => PlotType::FillBetween,
                "Boxplot" => PlotType::Boxplot,
                "HBoxplot" => PlotType::HBoxplot,
                "Colormesh" => PlotType::Colormesh,
                "Contour" => PlotType::Contour,
                "Quiver" => PlotType::Quiver,
                "Place" => PlotType::Place,
                "SecondaryYAxis" => PlotType::SecondaryYAxis,
                "SecondaryXAxis" => PlotType::SecondaryXAxis,
                "Path" => PlotType::Path,
                _ => return Err(format!("Unknown PlotElement type: {}", type_str)),
            };

            let mut element = PlotElement {
                element_type: element_type.clone(),
                x_data: None,
                y_data: None,
                y2_data: None,
                matrix_data: None,
                direction_data: None,
                datasets: None,
                options: PlotElementOptions::default(),
            };

            // Decode based on type
            match element_type {
                PlotType::Line
                | PlotType::Scatter
                | PlotType::Bar
                | PlotType::HBar
                | PlotType::Stem
                | PlotType::HStem => {
                    if args.len() >= 3 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                    }
                    if args.len() >= 4 {
                        self.parse_element_options(&args[3], &mut element.options)?;
                    }
                }
                PlotType::FillBetween => {
                    // fill_between(x, y1, y2, options)
                    if args.len() >= 3 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                    }
                    if args.len() >= 4 {
                        // arg[3] could be y2 data or options
                        if let Ok(y2) = self.decode_f64_list(&args[3]) {
                            if !y2.is_empty() {
                                element.y2_data = Some(y2);
                            }
                        }
                    }
                    if args.len() >= 5 {
                        self.parse_element_options(&args[4], &mut element.options)?;
                    }
                }
                PlotType::Boxplot | PlotType::HBoxplot => {
                    if args.len() >= 2 {
                        element.datasets = Some(self.decode_f64_matrix(&args[1])?);
                    }
                    if args.len() >= 3 {
                        self.parse_element_options(&args[2], &mut element.options)?;
                    }
                }
                PlotType::Colormesh | PlotType::Contour => {
                    if args.len() >= 2 {
                        element.matrix_data = Some(self.decode_f64_matrix(&args[1])?);
                    }
                    if args.len() >= 3 {
                        self.parse_element_options(&args[2], &mut element.options)?;
                    }
                }
                PlotType::Quiver => {
                    if args.len() >= 4 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                        // Decode directions as matrix, then convert to tuples
                        let dir_matrix = self.decode_f64_matrix(&args[3])?;
                        let directions: Vec<Vec<(f64, f64)>> = dir_matrix
                            .iter()
                            .map(|row| {
                                row.chunks(2)
                                    .map(|chunk| {
                                        if chunk.len() == 2 {
                                            (chunk[0], chunk[1])
                                        } else {
                                            (chunk[0], 0.0)
                                        }
                                    })
                                    .collect()
                            })
                            .collect();
                        element.direction_data = Some(directions);
                    }
                    if args.len() >= 5 {
                        self.parse_element_options(&args[4], &mut element.options)?;
                    }
                }
                PlotType::Place => {
                    if args.len() >= 4 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                        // Text is the 4th argument
                        if let Expression::String(text) = &args[3] {
                            element.options.text = Some(text.clone());
                        } else if let Expression::Const(text) = &args[3] {
                            element.options.text = Some(text.clone());
                        }
                    }
                    if args.len() >= 5 {
                        self.parse_element_options(&args[4], &mut element.options)?;
                    }
                }
                PlotType::SecondaryYAxis | PlotType::SecondaryXAxis => {
                    // args[1] is the list of child elements
                    if args.len() >= 2 {
                        if let Expression::List(children) = &args[1] {
                            let mut decoded_children = Vec::new();
                            for child in children {
                                let decoded = self.decode_plot_element(child)?;
                                decoded_children.push(Box::new(decoded));
                            }
                            element.options.children = Some(decoded_children);
                        }
                    }
                    if args.len() >= 3 {
                        self.parse_element_options(&args[2], &mut element.options)?;
                    }
                }
                PlotType::GroupedBars => {}
                PlotType::Path => {
                    // args[1] is x_data, args[2] is y_data
                    if args.len() >= 2 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                    }
                    if args.len() >= 3 {
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                    }
                    if args.len() >= 4 {
                        self.parse_element_options(&args[3], &mut element.options)?;
                    }
                }
            }

            Ok(element)
        } else {
            Err(format!("Expected PlotElement expression, got: {:?}", expr))
        }
    }

    /// Decode a list of f64 from an Expression::List
    fn decode_f64_list(&self, expr: &Expression) -> Result<Vec<f64>, String> {
        if let Expression::List(items) = expr {
            let mut result = Vec::new();
            for item in items {
                let n = self
                    .as_number(item)
                    .ok_or_else(|| format!("Expected number in list, got: {:?}", item))?;
                result.push(n);
            }
            Ok(result)
        } else {
            Err(format!("Expected list, got: {:?}", expr))
        }
    }

    /// Decode a 2D matrix from nested Expression::List
    fn decode_f64_matrix(&self, expr: &Expression) -> Result<Vec<Vec<f64>>, String> {
        if let Expression::List(rows) = expr {
            let mut result = Vec::new();
            for row in rows {
                result.push(self.decode_f64_list(row)?);
            }
            Ok(result)
        } else {
            Err(format!("Expected matrix (list of lists), got: {:?}", expr))
        }
    }

    /// Parse element options from a record expression
    fn parse_element_options(
        &self,
        expr: &Expression,
        options: &mut crate::plotting::PlotElementOptions,
    ) -> Result<(), String> {
        if let Expression::Operation { name, args, .. } = expr {
            if name == "record" {
                for opt in args {
                    if let Expression::Operation {
                        name: field_name,
                        args: field_args,
                        ..
                    } = opt
                    {
                        if field_name == "field" && field_args.len() == 2 {
                            let key = self.extract_string(&field_args[0])?;
                            match key.as_str() {
                                "label" => {
                                    options.label = Some(self.extract_string(&field_args[1])?)
                                }
                                "color" => {
                                    options.color = Some(self.extract_string(&field_args[1])?)
                                }
                                "stroke" => {
                                    options.stroke = Some(self.extract_string(&field_args[1])?)
                                }
                                "mark" => options.mark = Some(self.extract_string(&field_args[1])?),
                                "mark_size" => {
                                    options.mark_size = Some(self.extract_f64(&field_args[1])?)
                                }
                                "xerr" => {
                                    options.xerr =
                                        Some(self.extract_number_list_v2(&field_args[1])?)
                                }
                                "yerr" => {
                                    options.yerr =
                                        Some(self.extract_number_list_v2(&field_args[1])?)
                                }
                                "step" => options.step = Some(self.extract_string(&field_args[1])?),
                                "smooth" => {
                                    options.smooth = Some(self.extract_bool(&field_args[1])?)
                                }
                                "every" => {
                                    options.every = Some(self.extract_f64(&field_args[1])? as usize)
                                }
                                "offset" => {
                                    options.offset = Some(self.extract_f64(&field_args[1])?)
                                }
                                "width" => options.width = Some(self.extract_f64(&field_args[1])?),
                                "fill" => options.fill = Some(self.extract_string(&field_args[1])?),
                                "base" => options.base = Some(self.extract_f64(&field_args[1])?),
                                "colormap" | "map" => {
                                    options.colormap = Some(self.extract_string(&field_args[1])?)
                                }
                                "colors" | "color_values" => {
                                    // Per-point color values for scatter plots (floats 0-1)
                                    options.colors =
                                        Some(self.extract_number_list_v2(&field_args[1])?)
                                }
                                "scale" => options.scale = Some(self.extract_f64(&field_args[1])?),
                                "pivot" => {
                                    options.pivot = Some(self.extract_string(&field_args[1])?)
                                }
                                "clip" => options.clip = Some(self.extract_bool(&field_args[1])?),
                                "z_index" => {
                                    options.z_index = Some(self.extract_f64(&field_args[1])? as i32)
                                }
                                // place() options
                                "text" => options.text = Some(self.extract_string(&field_args[1])?),
                                "align" => {
                                    options.align = Some(self.extract_string(&field_args[1])?)
                                }
                                "padding" | "pad" => {
                                    options.padding = Some(self.extract_string(&field_args[1])?)
                                }
                                // yaxis() and xaxis() options
                                "position" => {
                                    options.position = Some(self.extract_string(&field_args[1])?)
                                }
                                "axis_label" => {
                                    options.axis_label = Some(self.extract_string(&field_args[1])?)
                                }
                                // xaxis() specific options
                                "tick_distance" => {
                                    options.tick_distance = Some(self.extract_f64(&field_args[1])?)
                                }
                                "exponent" => {
                                    options.exponent =
                                        Some(self.extract_f64(&field_args[1])? as i32)
                                }
                                "axis_offset" => {
                                    options.axis_offset = Some(self.extract_f64(&field_args[1])?)
                                }
                                // path() options
                                "closed" => {
                                    options.closed = Some(self.extract_bool(&field_args[1])?)
                                }
                                "functions" => {
                                    // functions = ("x => k/x", "x => k/x")
                                    // Expect a pair of strings
                                    if let Expression::Operation {
                                        name,
                                        args: fn_args,
                                        ..
                                    } = self.eval_concrete(&field_args[1])?
                                    {
                                        if name == "Pair" && fn_args.len() == 2 {
                                            options.transform_forward =
                                                Some(self.extract_string(&fn_args[0])?);
                                            options.transform_inverse =
                                                Some(self.extract_string(&fn_args[1])?);
                                        }
                                    }
                                }
                                _ => {} // Ignore unknown options
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper to extract boolean from Expression
    fn extract_bool(&self, expr: &Expression) -> Result<bool, String> {
        let evaluated = self.eval_concrete(expr)?;
        let s = match &evaluated {
            Expression::Const(s) | Expression::Object(s) => s.to_lowercase(),
            _ => return Err(format!("Expected boolean, got: {:?}", evaluated)),
        };
        match s.as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(format!("Expected boolean, got: {}", s)),
        }
    }

    /// Extract a 2D matrix of f64 values from an expression (list of lists)
    fn extract_f64_matrix(&self, expr: &Expression) -> Result<Vec<Vec<f64>>, String> {
        // First evaluate the expression
        let evaluated = self.eval_concrete(expr)?;

        // Helper to extract row from various list representations
        let extract_row = |row: &Expression| -> Result<Vec<f64>, String> {
            // Try Expression::List first
            if let Expression::List(elems) = row {
                let mut row_data = Vec::new();
                for elem in elems {
                    if let Some(n) = self.as_number(elem) {
                        row_data.push(n);
                    } else {
                        return Err(format!("Expected number in matrix, got: {:?}", elem));
                    }
                }
                return Ok(row_data);
            }
            // Try extract_flat_list
            if let Some(elems) = self.extract_flat_list(row) {
                let mut row_data = Vec::new();
                for elem in elems {
                    if let Some(n) = self.as_number(&elem) {
                        row_data.push(n);
                    } else {
                        return Err(format!("Expected number in matrix, got: {:?}", elem));
                    }
                }
                return Ok(row_data);
            }
            Err(format!("Expected row to be a list, got: {:?}", row))
        };

        // Try Expression::List for outer list
        if let Expression::List(rows) = &evaluated {
            let mut matrix = Vec::new();
            for row in rows {
                matrix.push(extract_row(row)?);
            }
            return Ok(matrix);
        }

        // Try extract_flat_list for outer list
        if let Some(rows) = self.extract_flat_list(&evaluated) {
            let mut matrix = Vec::new();
            for row in rows {
                matrix.push(extract_row(&row)?);
            }
            return Ok(matrix);
        }

        Err(format!(
            "Expected matrix (list of lists), got: {:?}",
            evaluated
        ))
    }

    /// Extract a string from a Const expression
    fn extract_string(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Const(s) => {
                // Remove quotes if present
                let s = s.trim_matches('"');
                Ok(s.to_string())
            }
            Expression::String(s) => Ok(s.clone()),
            Expression::Object(s) => Ok(s.clone()),
            _ => Err(format!("Expected string, got: {:?}", expr)),
        }
    }

    /// Extract a list of strings from an expression
    fn extract_string_list(&self, expr: &Expression) -> Result<Vec<String>, String> {
        let evaluated = self.eval_concrete(expr)?;
        if let Expression::List(elements) = evaluated {
            elements.iter().map(|e| self.extract_string(e)).collect()
        } else {
            Err(format!("Expected list of strings, got: {:?}", expr))
        }
    }

    /// Extract a list of f64 numbers from an expression
    fn extract_f64_list_from_diagram_option(&self, expr: &Expression) -> Result<Vec<f64>, String> {
        let evaluated = self.eval_concrete(expr)?;
        if let Expression::List(elements) = evaluated {
            elements.iter().map(|e| self.extract_f64(e)).collect()
        } else {
            Err(format!("Expected list of numbers, got: {:?}", expr))
        }
    }

    /// Extract a single number (f64) from an expression
    fn extract_number(&self, expr: &Expression) -> Result<f64, String> {
        let evaluated = self.eval_concrete(expr)?;
        if let Some(n) = self.as_number(&evaluated) {
            Ok(n)
        } else if let Expression::Const(s) = &evaluated {
            s.parse::<f64>()
                .map_err(|_| format!("Expected number, got: {}", s))
        } else {
            Err(format!("Expected number, got: {:?}", evaluated))
        }
    }

    /// Alias for extract_number
    fn extract_f64(&self, expr: &Expression) -> Result<f64, String> {
        self.extract_number(expr)
    }

    /// Extract elements from a flat list (like [1, 2, 3])
    fn extract_flat_list(&self, expr: &Expression) -> Option<Vec<Expression>> {
        match expr {
            Expression::Operation { name, args, .. } if name == "list" || name == "List" => {
                Some(args.clone())
            }
            Expression::Operation { name, args, .. } if name == "Cons" => {
                // Recursively extract from Cons structure
                if args.len() == 2 {
                    let head = args[0].clone();
                    if let Some(mut tail) = self.extract_flat_list(&args[1]) {
                        let mut result = vec![head];
                        result.append(&mut tail);
                        return Some(result);
                    }
                }
                None
            }
            Expression::Object(s) if s == "Nil" => Some(vec![]),
            _ => None,
        }
    }

    /// Extract a list of numbers, handling various list representations
    fn extract_number_list_v2(&self, expr: &Expression) -> Result<Vec<f64>, String> {
        let evaluated = self.eval_concrete(expr)?;

        // Handle Expression::List variant (e.g., [1, 2, 3])
        if let Expression::List(elements) = &evaluated {
            let mut result = Vec::new();
            for elem in elements {
                if let Some(n) = self.as_number(elem) {
                    result.push(n);
                } else {
                    return Err(format!("Expected number in list, got: {:?}", elem));
                }
            }
            return Ok(result);
        }

        // Handle List Operation (less common but possible)
        if let Expression::Operation { name, args, .. } = &evaluated {
            if name == "List" || name == "list" {
                let mut result = Vec::new();
                for arg in args {
                    if let Some(n) = self.as_number(arg) {
                        result.push(n);
                    } else {
                        return Err(format!("Expected number in list, got: {:?}", arg));
                    }
                }
                return Ok(result);
            }
        }

        // Try flat list extraction (for Cons structures)
        if let Some(elems) = self.extract_flat_list(&evaluated) {
            let mut result = Vec::new();
            for elem in elems {
                if let Some(n) = self.as_number(&elem) {
                    result.push(n);
                } else {
                    return Err(format!("Expected number in list, got: {:?}", elem));
                }
            }
            return Ok(result);
        }

        Err(format!("Expected list of numbers, got: {:?}", evaluated))
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
        // Delegate to the free function - DRY principle
        eval_numeric(expr)
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

    /// Pretty-print a value for `out()` function
    /// Formats lists/matrices nicely for readability
    fn pretty_print_value(&self, expr: &Expression) -> String {
        match expr {
            Expression::Const(s) => s.clone(),
            Expression::String(s) => format!("\"{}\"", s),
            Expression::Object(s) => s.clone(),
            Expression::List(elements) => {
                // Check if this is a matrix (list of lists)
                let is_matrix = elements.iter().all(|e| matches!(e, Expression::List(_)));
                if is_matrix && !elements.is_empty() {
                    // Pretty-print as matrix with alignment
                    self.pretty_print_matrix(elements)
                } else {
                    // Simple list
                    let items: Vec<String> = elements
                        .iter()
                        .map(|e| self.pretty_print_value(e))
                        .collect();
                    format!("[{}]", items.join(", "))
                }
            }
            Expression::Operation { name, args, .. } => {
                // Special-case raw Typst helpers so we don't emit wrapper syntax
                if name == "typst_raw" && args.len() == 1 {
                    if let Ok(
                        Expression::String(s) | Expression::Const(s) | Expression::Object(s),
                    ) = self.eval_concrete(&args[0])
                    {
                        return s;
                    }
                    if let Ok(s) = self.extract_string(&args[0]) {
                        return s;
                    }
                }
                if name == "concat" {
                    let mut parts = Vec::with_capacity(args.len());
                    let mut all_ok = true;
                    for a in args {
                        match self.eval_concrete(a) {
                            Ok(Expression::String(s))
                            | Ok(Expression::Const(s))
                            | Ok(Expression::Object(s)) => parts.push(s),
                            _ => {
                                all_ok = false;
                                break;
                            }
                        }
                    }
                    if all_ok {
                        return parts.join("");
                    }
                }
                // Handle Matrix(rows, cols, [elements]) format
                if (name == "Matrix" || name == "matrix") && args.len() == 3 {
                    if let (Some(rows), Some(cols)) =
                        (self.as_integer(&args[0]), self.as_integer(&args[1]))
                    {
                        if let Expression::List(elements) = &args[2] {
                            return self.pretty_print_flat_matrix(
                                rows as usize,
                                cols as usize,
                                elements,
                            );
                        }
                    }
                }
                if args.is_empty() {
                    name.clone()
                } else {
                    let args_str: Vec<String> =
                        args.iter().map(|a| self.pretty_print_value(a)).collect();
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            _ => format!("{:?}", expr),
        }
    }

    /// Pretty-print a matrix (list of lists) with alignment
    fn pretty_print_matrix(&self, rows: &[Expression]) -> String {
        // Extract all elements as strings
        let string_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                if let Expression::List(cols) = row {
                    cols.iter().map(|e| self.pretty_print_value(e)).collect()
                } else {
                    vec![self.pretty_print_value(row)]
                }
            })
            .collect();

        if string_rows.is_empty() {
            return "[]".to_string();
        }

        // Find max width for each column
        let num_cols = string_rows[0].len();
        let col_widths: Vec<usize> = (0..num_cols)
            .map(|c| {
                string_rows
                    .iter()
                    .map(|row| row.get(c).map(|s| s.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        // Build output with box drawing
        let mut lines = Vec::new();
        lines.push(
            "┌".to_string() + &" ".repeat(col_widths.iter().sum::<usize>() + num_cols * 2) + "┐",
        );

        for row in &string_rows {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{:>width$}", s, width = col_widths[i]))
                .collect();
            lines.push(format!("│ {} │", cells.join("  ")));
        }

        lines.push(
            "└".to_string() + &" ".repeat(col_widths.iter().sum::<usize>() + num_cols * 2) + "┘",
        );
        lines.join("\n")
    }

    /// Pretty-print a flat matrix (Matrix(rows, cols, [flat_elements]))
    fn pretty_print_flat_matrix(
        &self,
        rows: usize,
        cols: usize,
        elements: &[Expression],
    ) -> String {
        if elements.len() != rows * cols {
            // Fallback if dimensions don't match
            return format!("Matrix({}, {}, {:?})", rows, cols, elements);
        }

        // Convert flat elements to 2D
        let string_rows: Vec<Vec<String>> = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| self.pretty_print_value(&elements[r * cols + c]))
                    .collect()
            })
            .collect();

        // Find max width for each column
        let col_widths: Vec<usize> = (0..cols)
            .map(|c| {
                string_rows
                    .iter()
                    .map(|row| row.get(c).map(|s| s.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        // Build output with box drawing
        let inner_width = col_widths.iter().sum::<usize>() + (cols - 1) * 2;
        let mut lines = Vec::new();
        lines.push(format!("┌{}┐", " ".repeat(inner_width + 2)));

        for row in &string_rows {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{:>width$}", s, width = col_widths[i]))
                .collect();
            lines.push(format!("│ {} │", cells.join("  ")));
        }

        lines.push(format!("└{}┘", " ".repeat(inner_width + 2)));
        lines.join("\n")
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
    /// Handles:
    /// - Matrix(m, n, [elements])
    /// - Matrix(m, n, List([elements]))
    /// - [[a, b], [c, d]] - nested list syntax (row-major)
    fn extract_matrix(&self, expr: &Expression) -> Option<(usize, usize, Vec<Expression>)> {
        match expr {
            // Nested list syntax: [[a, b], [c, d]] → 2×2 matrix
            Expression::List(rows) if !rows.is_empty() => {
                // Check if this is a list of lists (matrix rows)
                let first_row = match &rows[0] {
                    Expression::List(cols) => cols,
                    _ => return None, // Not a nested list
                };

                let m = rows.len(); // Number of rows
                let n = first_row.len(); // Number of columns

                if n == 0 {
                    return None; // Empty columns
                }

                // Flatten all rows into elements (row-major order)
                let mut elements = Vec::with_capacity(m * n);
                for row in rows {
                    match row {
                        Expression::List(cols) => {
                            if cols.len() != n {
                                return None; // Inconsistent column count
                            }
                            elements.extend(cols.clone());
                        }
                        _ => return None, // Not a list row
                    }
                }

                Some((m, n, elements))
            }

            // Explicit Matrix(m, n, elements) format
            Expression::Operation { name, args, .. } if name == "Matrix" && args.len() >= 3 => {
                // Matrix(m, n, elements)
                let m = self.as_integer(&args[0])? as usize;
                let n = self.as_integer(&args[1])? as usize;

                // Elements can be a List or inline elements
                let elements = match &args[2] {
                    Expression::List(elems) => elems.clone(),
                    Expression::Operation {
                        name: list_name,
                        args: list_args,
                        ..
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
            span: None,
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
                span: None,
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
                span: None,
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
                span: None,
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
                span: None,
            },
        }
    }

    // === Complex number helpers ===

    /// Extract (real, imag) from a complex expression
    /// Handles: complex(re, im) or Complex(re, im)
    fn extract_complex(&self, expr: &Expression) -> Option<(Expression, Expression)> {
        match expr {
            Expression::Operation { name, args, .. }
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
            span: None,
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
            Expression::Operation { name, args, .. }
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
                span: None,
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
                span: None,
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
            Expression::Operation { name, args, .. } => {
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
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "plus");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_higher_order_function_basic() {
        // Test that functions can be passed as arguments and called
        let mut eval = Evaluator::new();

        let code = r#"
            define double(x) = x + x
            define apply_fn(f, x) = f(x)
            define test = apply_fn(double, 5)
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        let result = eval.eval_concrete(&Expression::Object("test".to_string()));
        assert!(result.is_ok(), "HOF should work: {:?}", result);

        // Result should be double(5) = 5 + 5 = 10
        let expr = result.unwrap();
        assert_eq!(expr, Expression::Const("10".to_string()));
    }

    #[test]
    fn test_higher_order_function_apply_twice() {
        // Test nested HOF calls: apply_twice(f, x) = f(f(x))
        let mut eval = Evaluator::new();

        let code = r#"
            define inc(x) = x + 1
            define apply_twice(f, x) = f(f(x))
            define test = apply_twice(inc, 10)
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        let result = eval.eval_concrete(&Expression::Object("test".to_string()));
        assert!(result.is_ok(), "Nested HOF should work: {:?}", result);
    }

    #[test]
    fn test_higher_order_function_with_pattern_match() {
        // Test HOF with pattern matching (the original use case for is_t/is_r)
        let mut eval = Evaluator::new();

        let code = r#"
            define is_one(x) = match x { 1 => 1 | _ => 0 }
            define check(f, x) = f(x)
            define test1 = check(is_one, 1)
            define test2 = check(is_one, 2)
        "#;
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        let result1 = eval.eval_concrete(&Expression::Object("test1".to_string()));
        assert!(result1.is_ok());
        assert_eq!(
            result1.unwrap(),
            Expression::Const("1".to_string()),
            "is_one(1) should be 1"
        );

        let result2 = eval.eval_concrete(&Expression::Object("test2".to_string()));
        assert!(result2.is_ok());
        assert_eq!(
            result2.unwrap(),
            Expression::Const("0".to_string()),
            "is_one(2) should be 0"
        );
    }

    #[test]
    fn test_expression_span_is_source_of_truth_for_debugger() {
        use std::path::PathBuf;
        use std::sync::Arc;

        // The span on an Expression IS the source location for the debugger.
        // No lookup, no searching - just read expression.span.

        // Create a file path wrapped in Arc (cheap to clone)
        let file = Arc::new(PathBuf::from("test_file.kleis"));

        // Create an expression with a known span (line 42, column 10, file)
        let span = SourceSpan::new(42, 10).with_file(Arc::clone(&file));
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
            span: Some(span),
        };

        // This is what the debugger does: read the span from the current expression
        if let Expression::Operation { span, .. } = &expr {
            // Verify we get the correct location including file
            assert!(span.is_some(), "Expression should have a span");
            let loc = span.as_ref().unwrap();
            assert_eq!(loc.line, 42, "Debugger should report line 42");
            assert_eq!(loc.column, 10, "Debugger should report column 10");
            assert!(loc.file.is_some(), "Debugger should have file path");
            assert_eq!(
                loc.file.as_ref().unwrap().to_string_lossy(),
                "test_file.kleis"
            );
        } else {
            panic!("Expected Operation expression");
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
            span: None,
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
            span: None,
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
                span: None,
            }),
            span: None,
        };

        let result = eval
            .beta_reduce(&lambda, &Expression::Const("5".to_string()))
            .unwrap();

        match result {
            Expression::Operation { name, args, .. } => {
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
                span: None,
            }),
            span: None,
        };

        let result = eval
            .beta_reduce(&lambda, &Expression::Const("3".to_string()))
            .unwrap();

        // Should be λ y . 3 + y
        match result {
            Expression::Lambda { params, body, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "y");

                match *body {
                    Expression::Operation { name, ref args, .. } => {
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
                span: None,
            }),
            span: None,
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
            Expression::Operation { name, args, .. } => {
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
                span: None,
            }),
            span: None,
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
            Expression::Operation { name, args, .. } => {
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
            span: None,
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
                span: None,
            }),
            span: None,
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
                    span: None,
                }),
                span: None,
            }),
            span: None,
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
            span: None,
        };

        let converted = eval.alpha_convert(&lambda, "y", "z");

        match converted {
            Expression::Lambda { params, body, .. } => {
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
                    span: None,
                }),
                span: None,
            }),
            span: None,
        };

        // Apply with 'y' as argument (potential capture)
        let result = eval
            .beta_reduce(&outer_lambda, &Expression::Object("y".to_string()))
            .unwrap();

        // Result should be λ y' . y + y' (or similar fresh name)
        match result {
            Expression::Lambda { params, body, .. } => {
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
            span: None,
        };

        let result = eval.reduce_to_normal_form(&expr).unwrap();

        match result {
            Expression::Operation { name, args, .. } => {
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
            span: None,
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
            span: None,
        };

        // Should complete within fuel limit
        let result = eval.reduce_with_fuel(&expr, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_typst_raw_unescape_concat() {
        let mut eval = Evaluator::new();
        let expr = Expression::Operation {
            name: "typst_raw".to_string(),
            args: vec![Expression::Operation {
                name: "concat".to_string(),
                args: vec![
                    Expression::String("a\\n".to_string()),
                    Expression::String("b\\\"c".to_string()),
                ],
                span: None,
            }],
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        match result {
            Expression::Object(s) => assert_eq!(s, "a\nb\"c"),
            other => panic!("Expected Object, got {:?}", other),
        }
    }

    #[test]
    fn test_typst_raw_accepts_const_and_object() {
        let mut eval = Evaluator::new();

        // Const input
        let const_expr = Expression::Operation {
            name: "typst_raw".to_string(),
            args: vec![Expression::Const("foo".to_string())],
            span: None,
        };
        let const_result = eval.eval_concrete(&const_expr).unwrap();
        assert!(matches!(const_result, Expression::Object(ref s) if s == "foo"));

        // Object input
        let obj_expr = Expression::Operation {
            name: "typst_raw".to_string(),
            args: vec![Expression::Object("bar".to_string())],
            span: None,
        };
        let obj_result = eval.eval_concrete(&obj_expr).unwrap();
        assert!(matches!(obj_result, Expression::Object(ref s) if s == "bar"));
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
            span: None,
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
            span: None,
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
            span: None,
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
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "hello world"));

        // strlen("kleis") → 5
        let expr = Expression::Operation {
            name: "strlen".to_string(),
            args: vec![Expression::String("kleis".to_string())],
            span: None,
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
            span: None,
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
            span: None,
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
            span: None,
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
            span: None,
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
            span: None,
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
                span: None,
            }),
            then_branch: Box::new(Expression::String("yes".to_string())),
            else_branch: Box::new(Expression::String("no".to_string())),
            span: None,
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
                span: None,
            }),
            then_branch: Box::new(Expression::String("yes".to_string())),
            else_branch: Box::new(Expression::String("no".to_string())),
            span: None,
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
            span: None,
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
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "0"));

        // fib(1) → 1
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("1".to_string())],
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "1"));

        // fib(5) → 5
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("5".to_string())],
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "5"));

        // fib(10) → 55
        let expr = Expression::Operation {
            name: "fib".to_string(),
            args: vec![Expression::Const("10".to_string())],
            span: None,
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
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "true"));

        // strip_parens("(+ 2 3)") → "+ 2 3"
        let expr = Expression::Operation {
            name: "strip_parens".to_string(),
            args: vec![Expression::String("(+ 2 3)".to_string())],
            span: None,
        };
        let result = eval.eval_concrete(&expr).unwrap();
        assert!(matches!(result, Expression::String(ref s) if s == "+ 2 3"));

        // get_op("(+ 2 3)") → "+"
        let expr = Expression::Operation {
            name: "get_op".to_string(),
            args: vec![Expression::String("(+ 2 3)".to_string())],
            span: None,
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
    fn test_eval_without_debug_hook() {
        // Default behavior: no debug hook set, eval works normally
        let eval = Evaluator::new();

        // Simple constant - should return as-is
        let expr = Expression::Const("42".to_string());
        let result = eval.eval(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expression::Const("42".to_string()));
    }

    #[test]
    fn test_eval_function_call_no_hook() {
        let mut eval = Evaluator::new();

        // Load a function
        let code = "define double(x) = x + x";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Call without debug hook
        let expr = Expression::Operation {
            name: "double".to_string(),
            args: vec![Expression::Const("5".to_string())],
            span: None,
        };
        let result = eval.eval(&expr);
        assert!(result.is_ok());

        // Result should be 5 + 5 (symbolic)
        match result.unwrap() {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "plus"); // Parser converts + to "plus"
                assert_eq!(args.len(), 2);
            }
            other => panic!("Expected Operation, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_with_debug_hook_tracks_bindings() {
        use crate::debug::{DebugAction, DebugHook, DebugState, SourceLocation, StackFrame};
        use std::sync::{Arc, Mutex};

        // A hook that tracks bindings (uses Arc<Mutex> for shared state)
        struct BindingTracker {
            bindings: Arc<Mutex<Vec<(String, String)>>>,
        }

        impl DebugHook for BindingTracker {
            fn on_eval_start(
                &mut self,
                _: &Expression,
                _: &SourceLocation,
                _: usize,
            ) -> DebugAction {
                DebugAction::Continue
            }
            fn on_eval_end(&mut self, _: &Expression, _: &Result<Expression, String>, _: usize) {}
            fn on_function_enter(
                &mut self,
                _: &str,
                _: &[Expression],
                _: &SourceLocation,
                _: usize,
            ) {
            }
            fn on_function_exit(&mut self, _: &str, _: &Result<Expression, String>, _: usize) {}
            fn on_bind(&mut self, name: &str, value: &Expression, _: usize) {
                self.bindings
                    .lock()
                    .unwrap()
                    .push((name.to_string(), format!("{:?}", value)));
            }
            fn state(&self) -> &DebugState {
                &DebugState::Running
            }
            fn should_stop(&self, _: &SourceLocation, _: usize) -> bool {
                false
            }
            fn wait_for_command(&mut self) -> DebugAction {
                DebugAction::Continue
            }
            fn get_stack(&self) -> &[StackFrame] {
                &[]
            }
            fn push_frame(&mut self, _: StackFrame) {}
            fn pop_frame(&mut self) -> Option<StackFrame> {
                None
            }
        }

        let mut eval = Evaluator::new();

        // Shared state to collect bindings
        let bindings = Arc::new(Mutex::new(Vec::new()));
        let hook = BindingTracker {
            bindings: bindings.clone(),
        };

        // Set the debug hook
        eval.set_debug_hook(Box::new(hook));

        // Load a function with parameters
        let code = "define add(a, b) = a + b";
        let program = parse_kleis_program(code).unwrap();
        eval.load_program(&program).unwrap();

        // Call the function - debug hook will be called automatically
        let expr = Expression::Operation {
            name: "add".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
            span: None,
        };
        let result = eval.eval(&expr);
        assert!(result.is_ok());

        // Check that bindings were tracked
        let tracked = bindings.lock().unwrap();
        assert!(
            tracked.len() >= 2,
            "Expected at least 2 bindings, got {}",
            tracked.len()
        );
        assert!(tracked.iter().any(|(name, _)| name == "a"));
        assert!(tracked.iter().any(|(name, _)| name == "b"));
    }

    #[test]
    fn test_set_and_clear_debug_hook() {
        let eval = Evaluator::new();

        // Initially, no hook is set
        assert!(!eval.is_debugging());

        // Set a hook
        use crate::debug::NoOpDebugHook;
        eval.set_debug_hook(Box::new(NoOpDebugHook));
        assert!(eval.is_debugging());

        // Clear the hook
        eval.clear_debug_hook();
        assert!(!eval.is_debugging());
    }

    // =========================================
    // Example Block Evaluation Tests (v0.93)
    // =========================================

    #[test]
    fn test_eval_example_block_simple() {
        use crate::kleis_ast::{ExampleBlock, ExampleStatement};

        let mut eval = Evaluator::new();

        // Create a simple example block
        let example = ExampleBlock {
            name: "simple test".to_string(),
            statements: vec![
                ExampleStatement::Let {
                    name: "x".to_string(),
                    type_annotation: None,
                    value: Expression::Const("5".to_string()),
                    location: None,
                },
                ExampleStatement::Let {
                    name: "y".to_string(),
                    type_annotation: None,
                    value: Expression::Const("5".to_string()),
                    location: None,
                },
                ExampleStatement::Assert {
                    condition: Expression::Operation {
                        name: "eq".to_string(),
                        args: vec![
                            Expression::Object("x".to_string()),
                            Expression::Object("y".to_string()),
                        ],
                        span: None,
                    },
                    location: None,
                },
            ],
        };

        let result = eval.eval_example_block(&example);
        assert!(
            result.passed,
            "Expected example to pass: {:?}",
            result.error
        );
        assert_eq!(result.assertions_passed, 1);
        assert_eq!(result.assertions_total, 1);
    }

    #[test]
    fn test_eval_example_block_failing_assert() {
        use crate::kleis_ast::{ExampleBlock, ExampleStatement};

        let mut eval = Evaluator::new();

        // Create an example block with a failing assertion
        let example = ExampleBlock {
            name: "failing test".to_string(),
            statements: vec![
                ExampleStatement::Let {
                    name: "x".to_string(),
                    type_annotation: None,
                    value: Expression::Const("5".to_string()),
                    location: None,
                },
                ExampleStatement::Let {
                    name: "y".to_string(),
                    type_annotation: None,
                    value: Expression::Const("10".to_string()),
                    location: None,
                },
                ExampleStatement::Assert {
                    condition: Expression::Operation {
                        name: "eq".to_string(),
                        args: vec![
                            Expression::Object("x".to_string()),
                            Expression::Object("y".to_string()),
                        ],
                        span: None,
                    },
                    location: None,
                },
            ],
        };

        let result = eval.eval_example_block(&example);
        assert!(!result.passed, "Expected example to fail");
        assert!(result.error.is_some());
        assert_eq!(result.assertions_passed, 0);
        assert_eq!(result.assertions_total, 1);
    }

    #[test]
    fn test_eval_example_block_bindings_dont_leak() {
        use crate::kleis_ast::{ExampleBlock, ExampleStatement};

        let mut eval = Evaluator::new();

        // Set a binding before the example
        eval.set_binding("outer".to_string(), Expression::Const("1".to_string()));

        let example = ExampleBlock {
            name: "scope test".to_string(),
            statements: vec![ExampleStatement::Let {
                name: "inner".to_string(),
                type_annotation: None,
                value: Expression::Const("2".to_string()),
                location: None,
            }],
        };

        let result = eval.eval_example_block(&example);
        assert!(result.passed);

        // The inner binding should NOT leak out
        assert!(eval.get_binding("inner").is_none());
        // The outer binding should still exist
        assert!(eval.get_binding("outer").is_some());
    }

    #[test]
    fn test_run_all_examples() {
        use crate::kleis_parser::parse_kleis_program;

        let code = r#"
            example "test one" {
                let x = 1
            }
            
            example "test two" {
                let y = 2
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let mut eval = Evaluator::new();
        let results = eval.run_all_examples(&program);

        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(results[1].passed);
        assert_eq!(results[0].name, "test one");
        assert_eq!(results[1].name, "test two");
    }
}
