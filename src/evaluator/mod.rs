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
use crate::axiom_verifier::{AxiomVerifier, VerificationResult};
use crate::debug::{DebugHook, SourceLocation};
use crate::kleis_ast::{ExampleBlock, ExampleStatement, FunctionDef, Program, TopLevel};
use crate::kleis_parser::SourceSpan;
use crate::pattern_matcher::PatternMatcher;
use crate::solvers::backend::Witness;
use crate::structure_registry::StructureRegistry;
use std::cell::RefCell;
use std::collections::HashMap;
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
    /// Assertion verified by Z3 (symbolic proof).
    /// For existential quantifiers, `witness` contains a satisfying assignment.
    Verified { witness: Option<Witness> },
    /// Assertion failed with concrete values (boxed to reduce enum size)
    Failed {
        expected: Box<Expression>,
        actual: Box<Expression>,
    },
    /// Assertion disproved by Z3 — the `witness` contains variable assignments
    /// (as Kleis expressions) that violate the property.
    Disproved { witness: Witness },
    /// Assertion couldn't be evaluated (symbolic)
    Unknown(String),
    /// Loaded axioms are mutually inconsistent — any assertion would be vacuously true
    InconsistentAxioms,
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

    /// Cached axiom consistency result (avoids re-checking per assertion).
    /// None = not yet checked, Some(true) = consistent, Some(false) = inconsistent.
    /// "Unknown" from Z3 is stored as None (re-check not attempted — too expensive).
    axiom_consistency_cache: RefCell<Option<Option<bool>>>,
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
                "pow" | "power" => vals.first().zip(vals.get(1)).map(|(a, b)| a.powf(*b)),
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
            axiom_consistency_cache: RefCell::new(None),
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
                        span: span.clone(),
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
                span,
            } => {
                let eval_cond = self.eval_internal(condition, depth + 1)?;

                if let Some(cond_bool) = self.as_boolean(&eval_cond) {
                    if cond_bool {
                        return self.eval_internal(then_branch, depth + 1);
                    }
                    return self.eval_internal(else_branch, depth + 1);
                }

                Ok(Expression::Conditional {
                    condition: Box::new(eval_cond),
                    then_branch: then_branch.clone(),
                    else_branch: else_branch.clone(),
                    span: span.clone(),
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

            // Atoms - return as-is (except Object which may need binding lookup)
            Expression::Const(_) | Expression::String(_) | Expression::Placeholder { .. } => {
                Ok(expr.clone())
            }

            Expression::Object(name) => {
                // Check bindings first (from example blocks / let statements)
                if let Some(bound) = self.bindings.get(name) {
                    // Recursively evaluate the bound value
                    self.eval_internal(bound, depth + 1)
                } else {
                    // Not bound, return as symbolic object
                    Ok(expr.clone())
                }
            }
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

    /// Verify a proposition (public wrapper around eval_assert).
    ///
    /// Tries concrete evaluation first; falls back to Z3 for quantified
    /// or symbolic expressions. This is the same pipeline that `assert()`
    /// uses inside example blocks.
    ///
    /// Returns the `AssertResult` which may be:
    /// - `Passed` — concrete evaluation confirmed truth
    /// - `Verified` — Z3 proved the proposition
    /// - `Failed` — concrete evaluation returned false
    /// - `Disproved { witness }` — Z3 found a counterexample (Kleis expressions)
    /// - `Unknown(msg)` — could not determine
    pub fn verify_proposition(&self, condition: &Expression) -> AssertResult {
        self.eval_assert(condition)
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
                    // Evaluate the value: first via eval (fires debug hooks for
                    // cross-file stepping), then eval_concrete to fully reduce.
                    // eval_internal returns substituted-but-unevaluated bodies for
                    // user-defined functions; without the second step, let bindings
                    // would hold intermediate expressions that break pattern matching.
                    match self.eval(value) {
                        Ok(partial) => {
                            let evaluated = self.eval_concrete(&partial).unwrap_or(partial);
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
                                AssertResult::Verified { witness } => {
                                    let msg = if let Some(w) = witness {
                                        format!("Verified by Z3 ✓ (witness: {})", w)
                                    } else {
                                        "Verified by Z3 ✓".to_string()
                                    };
                                    hook.on_assert_verified(condition, true, &msg, 0);
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
                                AssertResult::Disproved { witness } => {
                                    hook.on_assert_verified(
                                        condition,
                                        false,
                                        &format!("Disproved by Z3: {}", witness),
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
                                AssertResult::InconsistentAxioms => {
                                    hook.on_assert_verified(
                                        condition,
                                        false,
                                        "AXIOM INCONSISTENCY: loaded axioms are contradictory",
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
                        AssertResult::Verified { .. } => {
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
                        AssertResult::Disproved { witness } => {
                            // Z3 found a counterexample!
                            self.bindings = saved_bindings;
                            return ExampleResult {
                                name: example.name.clone(),
                                passed: false,
                                error: Some(format!(
                                    "Assertion disproved by Z3. Counterexample: {}",
                                    witness
                                )),
                                assertions_passed,
                                assertions_total,
                            };
                        }
                        AssertResult::InconsistentAxioms => {
                            self.bindings = saved_bindings;
                            return ExampleResult {
                                name: example.name.clone(),
                                passed: false,
                                error: Some(
                                    "AXIOM INCONSISTENCY DETECTED: The loaded axioms are \
                                     mutually unsatisfiable (Z3 proved them contradictory). \
                                     All assertions would be vacuously true. \
                                     This is a theory bug — check your axiom definitions \
                                     for contradictions."
                                        .to_string(),
                                ),
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
    pub fn verify_with_z3(&self, condition: &Expression) -> Option<AssertResult> {
        // Fast path: if we already know axioms are inconsistent, skip solver entirely
        if let Some(Some(false)) = *self.axiom_consistency_cache.borrow() {
            return Some(AssertResult::InconsistentAxioms);
        }

        let registry = self.build_registry_internal();

        match AxiomVerifier::new(&registry) {
            Ok(mut verifier) => {
                // Pass cached consistency result to avoid redundant checks.
                // If we already checked and got Sat or Unknown, tell the verifier
                // to skip the expensive re-check.
                if let Some(cached) = *self.axiom_consistency_cache.borrow() {
                    verifier.set_consistency_cache(cached);
                }

                verifier.load_adt_constructors(self.adt_constructors.iter());

                let result = verifier.verify_axiom(condition);

                // Capture the verifier's consistency result for future assertions
                if let Some(consistency) = verifier.get_consistency_result() {
                    *self.axiom_consistency_cache.borrow_mut() = Some(consistency);
                }

                match result {
                    Ok(result) => match result {
                        VerificationResult::Valid => Some(AssertResult::Verified { witness: None }),
                        VerificationResult::ValidWithWitness { witness } => {
                            Some(AssertResult::Verified {
                                witness: Some(witness),
                            })
                        }
                        VerificationResult::Invalid { witness } => {
                            Some(AssertResult::Disproved { witness })
                        }
                        VerificationResult::InconsistentAxioms => {
                            *self.axiom_consistency_cache.borrow_mut() = Some(Some(false));
                            Some(AssertResult::InconsistentAxioms)
                        }
                        VerificationResult::Unknown => None,
                        VerificationResult::Disabled => None,
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    /// Targeted Z3 verification for a structure operation with concrete arguments.
    ///
    /// Instead of loading quantified axioms (which cause timeouts with Z3's
    /// string theory), this method instantiates axioms with the concrete
    /// argument, producing ground (quantifier-free) assertions that Z3 can
    /// solve instantly.
    pub fn verify_structure_operation(
        &self,
        condition: &Expression,
        structure_name: &str,
    ) -> Option<AssertResult> {
        let concrete_arg = match condition {
            Expression::Operation { args, .. } if args.len() == 1 => &args[0],
            _ => return self.verify_with_z3(condition),
        };

        const LARGE_STRING_THRESHOLD: usize = 1_000_000;
        if let Expression::String(s) = concrete_arg {
            if s.len() > LARGE_STRING_THRESHOLD {
                eprintln!(
                    "[kleis-review] Warning: large string ({} bytes) passed to Z3 — \
                     temporary memory usage may be high",
                    s.len()
                );
            }
        }

        let structure = self.structures.iter().find(|s| s.name == structure_name)?;

        let mut ground_axioms = Vec::new();
        for member in &structure.members {
            if let crate::kleis_ast::StructureMember::Axiom {
                proposition:
                    Expression::Quantifier {
                        variables, body, ..
                    },
                ..
            } = member
            {
                if variables.len() == 1 {
                    let mut subst = HashMap::new();
                    subst.insert(variables[0].name.clone(), concrete_arg.clone());
                    ground_axioms.push(self.substitute(body, &subst));
                }
            }
        }

        if ground_axioms.is_empty() {
            return self.verify_with_z3(condition);
        }

        let mut ground_structure = structure.clone();
        ground_structure.members = ground_structure
            .members
            .iter()
            .filter(|m| !matches!(m, crate::kleis_ast::StructureMember::Axiom { .. }))
            .cloned()
            .collect();
        for (i, axiom) in ground_axioms.iter().enumerate() {
            ground_structure
                .members
                .push(crate::kleis_ast::StructureMember::Axiom {
                    name: format!("ground_{}", i),
                    proposition: axiom.clone(),
                });
        }

        let mut registry = StructureRegistry::new();
        let _ = registry.register(ground_structure);

        match AxiomVerifier::new(&registry) {
            Ok(mut verifier) => {
                verifier.set_consistency_cache(Some(true));
                verifier.load_adt_constructors(self.adt_constructors.iter());

                let result = verifier.verify_axiom(condition);
                match result {
                    Ok(result) => match result {
                        VerificationResult::Valid => Some(AssertResult::Verified { witness: None }),
                        VerificationResult::ValidWithWitness { witness } => {
                            Some(AssertResult::Verified {
                                witness: Some(witness),
                            })
                        }
                        VerificationResult::Invalid { witness } => {
                            Some(AssertResult::Disproved { witness })
                        }
                        VerificationResult::InconsistentAxioms => {
                            Some(AssertResult::InconsistentAxioms)
                        }
                        VerificationResult::Unknown => None,
                        VerificationResult::Disabled => None,
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
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
            "eq" | "=" | "==" | "equals" => {
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

            // === High-performance string operations ===
            // These are Rust-native implementations that replace recursive Kleis
            // functions to avoid O(n^2) string copying and stack overflow on large
            // inputs (e.g., parsing a 10K-line Rust file).
            "splitLines" => {
                // splitLines("line1\nline2\nline3") → Cons("line1", Cons("line2", Cons("line3", Nil)))
                // Auto-detects newline format: real \n (0x0A) or escaped two-char \n
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    let lines: Vec<&str> = if s.contains('\n') {
                        s.split('\n').collect()
                    } else if s.contains("\\n") {
                        s.split("\\n").collect()
                    } else {
                        vec![&s]
                    };
                    let mut result = Expression::Object("Nil".to_string());
                    for line in lines.into_iter().rev() {
                        result = Expression::Operation {
                            name: "Cons".to_string(),
                            args: vec![Expression::String(line.to_string()), result],
                            span: None,
                        };
                    }
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
            "countLines" => {
                // countLines("a\nb\nc") → 3
                // O(n) scan, no allocation
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    if s.is_empty() {
                        return Ok(Some(Expression::Const("0".to_string())));
                    }
                    let count = if s.contains('\n') {
                        s.split('\n').count()
                    } else if s.contains("\\n") {
                        s.split("\\n").count()
                    } else {
                        1
                    };
                    Ok(Some(Expression::Const(format!("{}", count))))
                } else {
                    Ok(None)
                }
            }
            "nthLine" => {
                // nthLine("a\nb\nc", 1) → "b" (0-indexed)
                if args.len() != 2 {
                    return Ok(None);
                }
                if let (Some(s), Some(n)) = (self.as_string(&args[0]), self.as_integer(&args[1])) {
                    let lines: Vec<&str> = if s.contains('\n') {
                        s.split('\n').collect()
                    } else if s.contains("\\n") {
                        s.split("\\n").collect()
                    } else {
                        vec![&s]
                    };
                    let idx = n as usize;
                    if idx < lines.len() {
                        Ok(Some(Expression::String(lines[idx].to_string())))
                    } else {
                        Ok(Some(Expression::String(String::new())))
                    }
                } else {
                    Ok(None)
                }
            }
            "readFile" => {
                // readFile("path/to/file.rs") → file contents as string
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(path) = self.as_string(&args[0]) {
                    match std::fs::read_to_string(&path) {
                        Ok(contents) => Ok(Some(Expression::String(contents))),
                        Err(e) => Err(format!("readFile: {}: {}", path, e)),
                    }
                } else {
                    Ok(None)
                }
            }
            "trimRight" => {
                // trimRight("hello  ") → "hello"
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    Ok(Some(Expression::String(s.trim_end().to_string())))
                } else {
                    Ok(None)
                }
            }
            "trimLeft" => {
                // trimLeft("  hello") → "hello"
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    Ok(Some(Expression::String(s.trim_start().to_string())))
                } else {
                    Ok(None)
                }
            }
            "trim" => {
                // trim("  hello  ") → "hello"
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    Ok(Some(Expression::String(s.trim().to_string())))
                } else {
                    Ok(None)
                }
            }
            "foldLines" => {
                // foldLines(f, init, source) → iteratively apply f(acc, line) over lines
                // Replaces recursive scan_lines: no stack overflow, no Cons list.
                // f can be a lambda or a named 2-arg function.
                if args.len() != 3 {
                    return Ok(None);
                }
                let func = &args[0];
                let init = &args[1];
                if let Some(source) = self.as_string(&args[2]) {
                    let lines: Vec<&str> = if source.contains('\n') {
                        source.split('\n').collect()
                    } else if source.contains("\\n") {
                        source.split("\\n").collect()
                    } else {
                        vec![&*source]
                    };
                    let mut acc = init.clone();
                    for line in &lines {
                        let line_expr = Expression::String(line.to_string());
                        // Try applying as lambda via beta reduction
                        match func {
                            Expression::Object(fname) => {
                                // Named function: call fname(acc, line)
                                let call = Expression::Operation {
                                    name: fname.clone(),
                                    args: vec![line_expr, acc.clone()],
                                    span: None,
                                };
                                acc = self.eval_concrete(&call)?;
                            }
                            _ => {
                                // Lambda or other expression: beta reduce
                                let reduced =
                                    self.beta_reduce_multi(func, &[line_expr, acc.clone()])?;
                                acc = self.eval_concrete(&reduced)?;
                            }
                        }
                    }
                    Ok(Some(acc))
                } else {
                    Ok(None)
                }
            }

            // === String predicates (concrete evaluation of Z3 regex primitives) ===
            // These mirror the Z3 regex-based predicates so that the evaluator
            // can reduce them without invoking Z3 (needed for check_action).
            "isAscii" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    // ASCII printable: every char in 0x20..=0x7E (space through tilde)
                    let ok = s.chars().all(|c| (' '..='~').contains(&c));
                    Ok(Some(Expression::Object(
                        if ok { "true" } else { "false" }.to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }
            "isDigits" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
                    Ok(Some(Expression::Object(
                        if ok { "true" } else { "false" }.to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }
            "isAlpha" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic());
                    Ok(Some(Expression::Object(
                        if ok { "true" } else { "false" }.to_string(),
                    )))
                } else {
                    Ok(None)
                }
            }
            "isAlphaNum" => {
                if args.len() != 1 {
                    return Ok(None);
                }
                if let Some(s) = self.as_string(&args[0]) {
                    let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric());
                    Ok(Some(Expression::Object(
                        if ok { "true" } else { "false" }.to_string(),
                    )))
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
            "null?" | "isEmpty" | "isNil" | "builtin_isEmpty" => {
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
            "nth" | "index" => {
                // nth(list, index) → element at index (alias: index)
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
            "apply_lambda" => {
                // apply_lambda(lambda, arg1, arg2, ...) → result of applying lambda to args
                // This is used internally when a let-bound lambda is called
                if args.is_empty() {
                    return Ok(None);
                }
                let lambda = &args[0];
                let call_args = &args[1..];

                // Apply the lambda using beta reduction
                if let Expression::Lambda { .. } = lambda {
                    let reduced = self.beta_reduce_multi(lambda, call_args)?;
                    // Evaluate the result
                    let result = self.eval_concrete(&reduced)?;
                    return Ok(Some(result));
                }
                Ok(None)
            }
            "list_map" | "map" | "builtin_map" => {
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
            "list_filter" | "filter" | "builtin_filter" => {
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
            "list_fold" | "foldl" | "builtin_foldl" => {
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
            "list_flatmap" | "flatmap" | "concat_map" | "builtin_flatmap" => {
                // list_flatmap(f, [a, b, c]) → flatten(map(f, [a, b, c]))
                // f should return a list, results are concatenated
                if args.len() != 2 {
                    return Ok(None);
                }
                let func = &args[0];

                // Evaluate the list argument first
                let evaluated_list = self.eval_concrete(&args[1])?;

                if let Expression::List(elements) = &evaluated_list {
                    let mut result = Vec::new();
                    for elem in elements {
                        // Apply function to element
                        let reduced = self.beta_reduce_multi(func, std::slice::from_ref(elem))?;
                        let mapped = self.eval_concrete(&reduced)?;
                        // Flatten: if result is a list, extend; otherwise push
                        if let Expression::List(inner) = mapped {
                            result.extend(inner);
                        } else {
                            result.push(mapped);
                        }
                    }
                    return Ok(Some(Expression::List(result)));
                }
                Ok(None)
            }
            "list_zip" | "zip" | "builtin_zip" => {
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
            "list_concat" | "list_append" | "append" | "builtin_append" => {
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

            // === Additional List Operations (for stdlib/lists.kleis) ===
            "reverse" | "builtin_reverse" => {
                // reverse([a, b, c]) → [c, b, a]
                if args.len() != 1 {
                    return Ok(None);
                }
                let evaluated_list = self.eval_concrete(&args[0])?;
                match &evaluated_list {
                    Expression::Object(s) if s == "Nil" => {
                        Ok(Some(Expression::Object("Nil".to_string())))
                    }
                    Expression::List(elements) => {
                        let mut reversed = elements.clone();
                        reversed.reverse();
                        Ok(Some(Expression::List(reversed)))
                    }
                    Expression::Operation {
                        name, args: inner, ..
                    } if name == "Cons" && inner.len() == 2 => {
                        // Convert Cons list to vec, reverse, return as List
                        let mut elements = vec![];
                        let mut current = evaluated_list.clone();
                        while let Expression::Operation {
                            name, args: inner, ..
                        } = &current
                        {
                            if name == "Cons" && inner.len() == 2 {
                                elements.push(inner[0].clone());
                                current = inner[1].clone();
                            } else {
                                break;
                            }
                        }
                        elements.reverse();
                        Ok(Some(Expression::List(elements)))
                    }
                    _ => Ok(None),
                }
            }

            "foldr" | "builtin_foldr" => {
                // foldr(f, z, [a, b, c]) → f(a, f(b, f(c, z)))
                if args.len() != 3 {
                    return Ok(None);
                }
                let func = &args[0];
                let z = &args[1];
                let evaluated_list = self.eval_concrete(&args[2])?;

                if let Expression::List(elements) = &evaluated_list {
                    let mut acc = z.clone();
                    for elem in elements.iter().rev() {
                        let reduced = self.beta_reduce_multi(func, &[elem.clone(), acc])?;
                        acc = self.eval_concrete(&reduced)?;
                    }
                    return Ok(Some(acc));
                }
                if let Expression::Object(s) = &evaluated_list {
                    if s == "Nil" {
                        return Ok(Some(z.clone()));
                    }
                }
                Ok(None)
            }

            "sum" | "builtin_sum" => {
                // sum([1, 2, 3]) → 6
                if args.len() != 1 {
                    return Ok(None);
                }
                let evaluated_list = self.eval_concrete(&args[0])?;
                if let Expression::List(elements) = &evaluated_list {
                    let mut total = 0.0;
                    for e in elements {
                        if let Some(n) = self.as_number(e) {
                            total += n;
                        } else {
                            return Ok(None);
                        }
                    }
                    return Ok(Some(Self::const_from_f64(total)));
                }
                Ok(None)
            }

            "product" | "builtin_product" => {
                // product([2, 3, 4]) → 24
                if args.len() != 1 {
                    return Ok(None);
                }
                let evaluated_list = self.eval_concrete(&args[0])?;
                if let Expression::List(elements) = &evaluated_list {
                    let mut total = 1.0;
                    for e in elements {
                        if let Some(n) = self.as_number(e) {
                            total *= n;
                        } else {
                            return Ok(None);
                        }
                    }
                    return Ok(Some(Self::const_from_f64(total)));
                }
                Ok(None)
            }

            "all" | "builtin_all" => {
                // all(p, [a, b, c]) → true if p(x) is true for all x
                if args.len() != 2 {
                    return Ok(None);
                }
                let pred = &args[0];
                let evaluated_list = self.eval_concrete(&args[1])?;

                if let Expression::List(elements) = &evaluated_list {
                    for elem in elements {
                        let reduced = self.beta_reduce(pred, elem)?;
                        let result = self.eval_concrete(&reduced)?;
                        if let Some(false) = self.as_bool(&result) {
                            return Ok(Some(Expression::Object("false".to_string())));
                        }
                    }
                    return Ok(Some(Expression::Object("true".to_string())));
                }
                if let Expression::Object(s) = &evaluated_list {
                    if s == "Nil" {
                        return Ok(Some(Expression::Object("true".to_string())));
                    }
                }
                Ok(None)
            }

            "any" | "builtin_any" => {
                // any(p, [a, b, c]) → true if p(x) is true for any x
                if args.len() != 2 {
                    return Ok(None);
                }
                let pred = &args[0];
                let evaluated_list = self.eval_concrete(&args[1])?;

                if let Expression::List(elements) = &evaluated_list {
                    for elem in elements {
                        let reduced = self.beta_reduce(pred, elem)?;
                        let result = self.eval_concrete(&reduced)?;
                        if let Some(true) = self.as_bool(&result) {
                            return Ok(Some(Expression::Object("true".to_string())));
                        }
                    }
                    return Ok(Some(Expression::Object("false".to_string())));
                }
                if let Expression::Object(s) = &evaluated_list {
                    if s == "Nil" {
                        return Ok(Some(Expression::Object("false".to_string())));
                    }
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
            "care" | "riccati" => self.lapack_care(args),

            #[cfg(feature = "numerical")]
            "lqr" => self.lapack_lqr(args),

            #[cfg(feature = "numerical")]
            "dare" | "riccati_discrete" => self.lapack_dare(args),

            #[cfg(feature = "numerical")]
            "dlqr" => self.lapack_dlqr(args),

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
            return Err("ode45 requires 3-4 arguments: f, y0, t_span, dt?\n\
                 Example: ode45((t, y) => [y[1], neg(y[0])], [1, 0], [0, 10])"
                .to_string());
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

        // We need the evaluator to properly evaluate complex lambda bodies.
        // Use a raw pointer to self - this is safe because:
        // 1. The closure is only called during integrate_dopri5
        // 2. self is valid for the entire duration of this function
        // 3. The closure doesn't escape builtin_ode45
        let eval_ptr = self as *const Evaluator;

        // Create dynamics function that calls the lambda
        let dynamics = move |t: f64, y: &[f64]| -> Vec<f64> {
            // Build: f(t, [y0, y1, ...])
            let t_expr = Expression::Const(format!("{}", t));
            let y_expr = Expression::List(
                y.iter()
                    .map(|&v| Expression::Const(format!("{}", v)))
                    .collect(),
            );

            // Apply lambda using the evaluator
            if let Expression::Lambda { params, .. } = &f_clone {
                if params.len() >= 2 {
                    // SAFETY: eval_ptr points to self which is valid for this function's duration
                    let evaluator = unsafe { &*eval_ptr };

                    // Use beta reduction to apply lambda: (λ t y . body)(t_val, y_val)
                    if let Ok(reduced) = evaluator.beta_reduce_multi(&f_clone, &[t_expr, y_expr]) {
                        // Evaluate the reduced expression
                        if let Ok(Expression::List(elems)) = evaluator.eval_concrete(&reduced) {
                            let nums: Option<Vec<f64>> = elems.iter().map(eval_numeric).collect();
                            if let Some(v) = nums {
                                return v;
                            }
                        }
                    }
                }
            }
            vec![0.0; dim]
        };

        // Integrate
        let result =
            crate::ode::integrate_dopri5(dynamics, &y0, (t0, t1), dt).map_err(|e| e.to_string())?;

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



}

mod helpers;
mod plotting;
mod substitute;

#[cfg(feature = "numerical")]
mod lapack;


impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::LambdaParam;
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

    #[test]
    fn test_object_resolves_from_bindings() {
        // Test that Expression::Object looks up values from self.bindings
        // This is critical for example blocks where let-bound variables
        // must be accessible when evaluating expressions containing Object references
        let mut eval = Evaluator::new();

        // Set a binding
        eval.set_binding("x".to_string(), Expression::Const("42".to_string()));

        // Evaluate an Object that references the binding
        let result = eval.eval(&Expression::Object("x".to_string())).unwrap();

        // Should resolve to the bound value
        assert_eq!(result, Expression::Const("42".to_string()));
    }

    #[test]
    fn test_bindings_used_in_operations() {
        // Test that bindings are resolved when used inside operations
        let mut eval = Evaluator::new();

        eval.set_binding("a".to_string(), Expression::Const("10".to_string()));
        eval.set_binding("b".to_string(), Expression::Const("5".to_string()));

        // Evaluate: a + b where a=10, b=5
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("a".to_string()),
                Expression::Object("b".to_string()),
            ],
            span: None,
        };

        // eval() resolves bindings, eval_concrete() also computes arithmetic
        let result = eval.eval_concrete(&expr).unwrap();

        // Should compute 10 + 5 = 15
        assert_eq!(result, Expression::Const("15".to_string()));
    }

    #[test]
    fn test_bindings_captured_in_lambda() {
        // Test that lambdas can access bindings from enclosing scope
        // This is the key fix for inverted_pendulum.kleis
        use crate::kleis_parser::parse_kleis;

        let mut eval = Evaluator::new();

        // Set up a binding
        eval.set_binding("k".to_string(), Expression::Const("2".to_string()));

        // Create and evaluate: let f = lambda x . k * x in f(5)
        // k should be captured from bindings
        let code = "let f = lambda x . k * x in f(5)";
        let expr = parse_kleis(code).unwrap();
        let result = eval.eval_concrete(&expr).unwrap();

        // k=2, x=5, so k*x = 10
        assert_eq!(result, Expression::Const("10".to_string()));
    }
}
