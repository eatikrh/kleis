//! Type Inference for Kleis - Proof of Concept
//!
//! Implements Hindley-Milner type inference for symbolic mathematical expressions.
//! This is a minimal PoC to demonstrate type inference on the existing AST.
//!
//! ## Future Vision (ADR-021: Algebraic Data Types)
//!
//! Currently, the Type enum is hardcoded in Rust. In Phase 3, this will move to Kleis:
//!
//! ```kleis
//! // stdlib/types.kleis
//! data Type =
//!   | Scalar
//!   | Vector(n: Nat)
//!   | Matrix(m: Nat, n: Nat)
//!   | Complex
//!   | Var(id: Nat)
//!   | Function(domain: Type, codomain: Type)
//!   | ForAll(var: Nat, body: Type)
//!   | UserDefined(name: String, params: List(Type))
//!
//! // Unification in Kleis (not Rust!)
//! operation unify : Type → Type → Option(Substitution)
//! define unify(t1, t2) = match (t1, t2) { ... }
//! ```
//!
//! This enables:
//! - Users can extend types without recompiling
//! - Type checking logic in Kleis (meta-circular!)
//! - True self-hosting (Kleis types defined in Kleis)
//!
//! See ADR-021 for complete proposal.

use crate::ast::Expression;
use std::collections::HashMap;

/// Type representation for Kleis expressions
///
/// **ADR-021: Algebraic Data Types - IMPLEMENTED!**
///
/// This Type enum now supports dynamic user-defined types via the `Data` variant.
/// Types are loaded from Kleis files (e.g., stdlib/types.kleis) rather than hardcoded.
///
/// ## Type Structure
///
/// **Bootstrap types** (needed for parsing Kleis itself):
/// - `Nat`: Natural numbers (for dimensions, indices)
/// - `String`: Text values
/// - `Bool`: Boolean values
///
/// **User-defined types** (loaded from Kleis):
/// - `Data`: Any type defined with `data` keyword
///   - Example: `data Type = Scalar | Matrix(m: Nat, n: Nat)`
///   - The Type system itself is a Data type!
///
/// **Meta-level types** (for type inference):
/// - `Var`: Type variables (α, β, γ) during inference
/// - `ForAll`: Polymorphic types (∀α. T) after generalization
///
/// ## Example Usage
///
/// ```kleis
/// // In stdlib/types.kleis:
/// data Type =
///   | Scalar
///   | Vector(n: Nat)
///   | Matrix(m: Nat, n: Nat)
///   | Complex
///
/// // Users can add their own:
/// data Currency = USD | EUR | GBP
/// ```
///
/// The registry maps variant names to types, enabling:
/// - `Scalar` → Data { type_name: "Type", constructor: "Scalar", args: [] }
/// - `Matrix(2, 3)` → Data { type_name: "Type", constructor: "Matrix", args: [Nat(2), Nat(3)] }
///
/// See ADR-021 for complete design.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ===== Bootstrap Types =====
    // These are needed to parse and represent Kleis itself
    /// Natural number type (for dimensions, indices)
    /// Used in: Matrix(m: Nat, n: Nat)
    Nat,

    /// Concrete natural number value (for dimension checking!)
    /// This is CRITICAL for distinguishing Matrix(2,3) from Matrix(2,2)
    /// Example: Matrix(2, 3) → Data { args: [NatValue(2), NatValue(3)] }
    NatValue(usize),

    /// Symbolic dimension expression (v0.92)
    /// Preserves type-level arithmetic: 2*n, n+1, n^2
    /// Used for proper dimension unification
    /// Example: Matrix(2*n, 2*n) → Data { args: [NatExpr(Mul(2, n)), NatExpr(Mul(2, n))] }
    NatExpr(crate::kleis_ast::DimExpr),

    /// String type (for text values)
    /// Used in: Currency(code: String)
    String,

    /// Concrete string value
    /// Example: Currency("USD") → Data { args: [StringValue("USD")] }
    StringValue(std::string::String),

    /// Boolean type (for logical values)
    Bool,

    /// Unit type (for expressions with no meaningful value)
    /// Represents () - the empty tuple / void / nothing
    /// Used in: Result(Unit, E), Option(Unit), side-effect functions
    Unit,

    // ===== User-Defined Types =====
    /// User-defined algebraic data type
    ///
    /// Loaded from `data` definitions in Kleis files.
    /// This is the KEY innovation - types are data, not hardcoded!
    ///
    /// Fields:
    /// - `type_name`: Which data type (e.g., "Type", "Option", "Currency")
    /// - `constructor`: Which variant (e.g., "Scalar", "Matrix", "Some")
    /// - `args`: Constructor arguments (e.g., [Nat(2), Nat(3)] for Matrix(2,3))
    ///
    /// Examples:
    /// - Scalar → Data { type_name: "Type", constructor: "Scalar", args: [] }
    /// - Matrix(2,3) → Data { type_name: "Type", constructor: "Matrix", args: [Nat, Nat] }
    /// - Some(x) → Data { type_name: "Option", constructor: "Some", args: [infer(x)] }
    Data {
        type_name: String,
        constructor: String,
        args: Vec<Type>,
    },

    // ===== Meta-Level Types =====
    // These exist at the type inference level, not user level
    /// Type variable (for inference)
    /// α, β, γ in type theory
    Var(TypeVar),

    /// Polymorphic type: ∀α. T
    /// For generalized types after inference
    ForAll(TypeVar, Box<Type>),
}

/// Type variable (α, β, γ, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

/// Type substitution: maps type variables to types
/// Example: {α → Scalar, β → Vector(3)}
#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<TypeVar, Type>,
}

impl Substitution {
    pub fn empty() -> Self {
        Substitution {
            map: HashMap::new(),
        }
    }

    pub fn singleton(var: TypeVar, ty: Type) -> Self {
        let mut map = HashMap::new();
        map.insert(var, ty);
        Substitution { map }
    }

    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if let Some(t) = self.map.get(v) {
                    // Recursively apply in case of chained substitutions
                    self.apply(t)
                } else {
                    ty.clone()
                }
            }
            Type::Data {
                type_name,
                constructor,
                args,
            } => {
                // Apply substitution to all type arguments
                let new_args: Vec<Type> = args.iter().map(|arg| self.apply(arg)).collect();
                Type::Data {
                    type_name: type_name.clone(),
                    constructor: constructor.clone(),
                    args: new_args,
                }
            }
            Type::ForAll(v, t) => Type::ForAll(v.clone(), Box::new(self.apply(t))),
            // Bootstrap types have no substructure (leaf types)
            // NatExpr is also a leaf - dimension variables are separate from type variables
            Type::Nat
            | Type::NatValue(_)
            | Type::NatExpr(_)
            | Type::String
            | Type::StringValue(_)
            | Type::Bool
            | Type::Unit => ty.clone(),
        }
    }

    /// Compose two substitutions
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut map = self.map.clone();
        for (var, ty) in &other.map {
            map.insert(var.clone(), self.apply(ty));
        }
        Substitution { map }
    }
}

/// Type constraint: represents equality between types
/// Example: α = Scalar
#[derive(Debug, Clone)]
pub struct Constraint {
    left: Type,
    right: Type,
}

/// Type inference context: maps variable names to types
#[derive(Debug, Clone)]
pub struct TypeContext {
    vars: HashMap<String, Type>,
    next_var: usize,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            vars: HashMap::new(),
            next_var: 0,
        }
    }

    /// Get type of a variable
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    /// Bind a variable to a type
    pub fn bind(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let var = TypeVar(self.next_var);
        self.next_var += 1;
        Type::Var(var)
    }

    /// Get all bound variables
    pub fn vars(&self) -> &HashMap<String, Type> {
        &self.vars
    }
}

/// Type inference engine
pub struct TypeInference {
    context: TypeContext,
    constraints: Vec<Constraint>,
    data_registry: crate::data_registry::DataTypeRegistry,
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeInference {
    pub fn new() -> Self {
        TypeInference {
            context: TypeContext::new(),
            constraints: Vec::new(),
            data_registry: crate::data_registry::DataTypeRegistry::new(),
        }
    }

    /// Create with a pre-populated data registry
    pub fn with_data_registry(registry: crate::data_registry::DataTypeRegistry) -> Self {
        TypeInference {
            context: TypeContext::new(),
            constraints: Vec::new(),
            data_registry: registry,
        }
    }

    /// Get a reference to the data registry
    pub fn data_registry(&self) -> &crate::data_registry::DataTypeRegistry {
        &self.data_registry
    }

    /// Get a mutable reference to the data registry
    pub fn data_registry_mut(&mut self) -> &mut crate::data_registry::DataTypeRegistry {
        &mut self.data_registry
    }

    /// Add a constraint
    fn add_constraint(&mut self, left: Type, right: Type) {
        self.constraints.push(Constraint { left, right });
    }

    /// Clear all constraints
    /// This should be called after solving constraints for a function to prevent
    /// constraint leakage between function definitions
    pub fn clear_constraints(&mut self) {
        self.constraints.clear();
    }

    /// Bind a variable to a type
    pub fn bind(&mut self, name: String, ty: Type) {
        self.context.bind(name, ty);
    }

    /// Get the context
    pub fn context(&self) -> &TypeContext {
        &self.context
    }

    /// Get mutable access to the context
    /// Used for restoring saved contexts (e.g., after checking function bodies)
    pub fn context_mut(&mut self) -> &mut TypeContext {
        &mut self.context
    }

    /// Convert a type annotation string to a Type
    /// Used for binding quantified variables to their annotated types
    fn type_annotation_to_type(&mut self, annotation: &str) -> Type {
        match annotation {
            // Complex types
            "ℂ" | "Complex" | "C" => Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
            // Real/Scalar types
            "ℝ" | "Real" | "Scalar" | "R" => Type::scalar(),
            // Natural numbers
            "ℕ" | "Nat" | "N" => Type::Data {
                type_name: "Type".to_string(),
                constructor: "Nat".to_string(),
                args: vec![],
            },
            // Integer types
            "ℤ" | "Int" | "Integer" | "Z" => Type::Data {
                type_name: "Type".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            },
            // Rational types
            "ℚ" | "Rational" | "Q" => Type::Data {
                type_name: "Type".to_string(),
                constructor: "Rational".to_string(),
                args: vec![],
            },
            // Boolean
            "Bool" | "𝔹" => Type::Bool,
            // Unit
            "Unit" | "()" => Type::Unit,
            // String
            "String" => Type::String,
            // Unknown type annotation - create a fresh variable
            _ => self.context.fresh_var(),
        }
    }

    /// Get next type variable ID (for creating fresh vars)
    /// Used when adding function parameters to context
    pub fn next_var_id(&mut self) -> usize {
        let id = self.context.next_var;
        self.context.next_var += 1;
        id
    }

    /// Infer type of an expression
    pub fn infer(
        &mut self,
        expr: &Expression,
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        match expr {
            // Numeric constants: infer Int for integers, Scalar for reals
            // This enables proper type promotion (Int + Rational → Rational)
            Expression::Const(s) => {
                // Check if it's an integer (no decimal point, no exponent)
                if s.parse::<i64>().is_ok()
                    && !s.contains('.')
                    && !s.contains('e')
                    && !s.contains('E')
                {
                    Ok(Type::Data {
                        type_name: "Type".to_string(),
                        constructor: "Int".to_string(),
                        args: vec![],
                    })
                } else {
                    Ok(Type::scalar())
                }
            }

            // String literals are String type
            Expression::String(_) => Ok(Type::String),

            // Variables: look up in context or check if data constructor
            Expression::Object(name) => {
                // First check if it's a nullary data constructor (like None, True, False, Nil)
                if self.data_registry.has_variant(name) {
                    // It's a data constructor! Treat as constructor with zero args
                    return self.infer_data_constructor(name, &[], context_builder);
                }

                // Check if variable is bound in context FIRST
                // This allows quantified variables like ∀(i : ℝ) to override the imaginary unit
                if let Some(ty) = self.context.get(name) {
                    return Ok(ty.clone());
                }

                // OPERATOR OVERLOADING: The imaginary unit i is Complex (only if not bound)
                if name == "i" {
                    return Ok(Type::Data {
                        type_name: "Type".to_string(),
                        constructor: "Complex".to_string(),
                        args: vec![],
                    });
                }

                // Unknown variable: create fresh type variable
                let ty = self.context.fresh_var();
                self.context.bind(name.clone(), ty.clone());
                Ok(ty)
            }

            // Placeholders: unknown type (fresh variable)
            Expression::Placeholder { .. } => Ok(self.context.fresh_var()),

            // Operations: infer based on operation type
            Expression::Operation { name, args, .. } => {
                self.infer_operation(name, args, context_builder)
            }

            // Pattern matching: infer scrutinee and all branches
            Expression::Match { scrutinee, cases, .. } => {
                self.infer_match(scrutinee, cases, context_builder)
            }

            // List literal: infer element types and unify
            Expression::List(elements) => self.infer_list(elements, context_builder),

            // Quantifier: bind variables with their type annotations, then infer body
            Expression::Quantifier {
                variables, body, ..
            } => {
                // Bind quantified variables to their annotated types
                for var in variables {
                    if let Some(ref type_annotation) = var.type_annotation {
                        let ty = self.type_annotation_to_type(type_annotation);
                        self.context.bind(var.name.clone(), ty);
                    }
                }
                // Infer the body type (the proposition)
                self.infer(body, context_builder)
            }

            // Conditional (if-then-else): both branches must have same type
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Condition must be Bool
                let cond_ty = self.infer(condition, context_builder)?;
                self.add_constraint(cond_ty, Type::Bool);

                // Both branches must have the same type
                let then_ty = self.infer(then_branch, context_builder)?;
                let else_ty = self.infer(else_branch, context_builder)?;
                self.add_constraint(then_ty.clone(), else_ty);

                // Return the type of the branches
                Ok(then_ty)
            }

            // Let binding: infer value type, bind variable(s), infer body
            // Grammar v0.8: supports pattern destructuring
            // When type_annotation is present, USE IT for the binding type
            Expression::Let {
                pattern,
                type_annotation,
                value,
                body,
                ..
            } => {
                // Determine the type to bind:
                // 1. If type annotation is present, parse and use it
                // 2. Otherwise, infer from the value
                let binding_ty = if let Some(ref ann) = type_annotation {
                    // Parse the type annotation string into a Type
                    self.parse_type_annotation(ann)
                } else {
                    // No annotation - infer from value
                    self.infer(value, context_builder)?
                };

                // Bind all variables from the pattern
                self.bind_pattern_variables(pattern, &binding_ty);

                // Infer body type with the new bindings
                self.infer(body, context_builder)
            }

            // Type ascription: (expr) : Type
            // The type annotation specifies the expected type
            // TODO: Parse the type_annotation string into Type and verify it matches inferred type
            Expression::Ascription { expr, .. } => {
                // For now, just infer the inner expression's type
                // Full implementation would:
                // 1. Parse type_annotation string to Type
                // 2. Infer expr type
                // 3. Unify/check that they match
                self.infer(expr, context_builder)
            }

            // Lambda expression: λ params . body
            // Type is: param_types → body_type
            Expression::Lambda { params, body, .. } => {
                // Create fresh type variables for each parameter and bind them
                for param in params {
                    let param_type = self.context.fresh_var();
                    self.context.bind(param.name.clone(), param_type);
                }

                // Infer body type with the parameter bindings
                let body_type = self.infer(body, context_builder)?;

                // Return body type (full implementation would construct function type)
                Ok(body_type)
            }
        }
    }

    /// Infer type of an expression and return a typed AST
    ///
    /// Unlike `infer()` which only returns the type, this method returns a
    /// `TypedExpr` that includes both the expression and its type, along with
    /// typed children. This is needed for semantic lowering (operator overloading).
    ///
    /// ## Example
    ///
    /// For `3 + 4*i`:
    /// - Returns TypedExpr with ty=Complex
    /// - children[0] is TypedExpr for `3` with ty=Real
    /// - children[1] is TypedExpr for `4*i` with ty=Complex
    ///
    /// The lowering pass can then inspect operand types to rewrite:
    /// `plus(Real, Complex)` → `complex_add(lift(Real), Complex)`
    pub fn infer_typed(
        &mut self,
        expr: &Expression,
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<crate::typed_ast::TypedExpr, String> {
        use crate::typed_ast::TypedExpr;

        match expr {
            // Leaf nodes: no children
            Expression::Const(_) => {
                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::leaf(expr.clone(), ty))
            }

            Expression::String(_) => {
                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::leaf(expr.clone(), ty))
            }

            Expression::Object(_) => {
                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::leaf(expr.clone(), ty))
            }

            Expression::Placeholder { .. } => {
                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::leaf(expr.clone(), ty))
            }

            // Operations: recurse into arguments
            Expression::Operation { name: _, args, .. } => {
                // First infer types of all arguments
                let typed_args: Result<Vec<TypedExpr>, String> = args
                    .iter()
                    .map(|arg| self.infer_typed(arg, context_builder))
                    .collect();
                let children = typed_args?;

                // Then infer type of the whole operation
                let ty = self.infer(expr, context_builder)?;

                Ok(TypedExpr::node(expr.clone(), ty, children))
            }

            // List: recurse into elements
            Expression::List(elements) => {
                let typed_elements: Result<Vec<TypedExpr>, String> = elements
                    .iter()
                    .map(|elem| self.infer_typed(elem, context_builder))
                    .collect();
                let children = typed_elements?;

                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::node(expr.clone(), ty, children))
            }

            // Match: scrutinee + case bodies
            Expression::Match { scrutinee, cases, .. } => {
                let typed_scrutinee = self.infer_typed(scrutinee, context_builder)?;

                // For each case, type the body (patterns are not expressions)
                let mut children = vec![typed_scrutinee];
                for case in cases {
                    // Save context, bind pattern vars, type body, restore
                    let saved_context = self.context.clone();
                    let scrutinee_ty = children[0].ty.clone();
                    let _ = self.check_pattern(&case.pattern, &scrutinee_ty);
                    let typed_body = self.infer_typed(&case.body, context_builder)?;
                    children.push(typed_body);
                    self.context = saved_context;
                }

                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::node(expr.clone(), ty, children))
            }

            // Conditional: condition + branches
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let typed_cond = self.infer_typed(condition, context_builder)?;
                let typed_then = self.infer_typed(then_branch, context_builder)?;
                let typed_else = self.infer_typed(else_branch, context_builder)?;

                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::node(
                    expr.clone(),
                    ty,
                    vec![typed_cond, typed_then, typed_else],
                ))
            }

            // Let binding: value + body
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                let typed_value = self.infer_typed(value, context_builder)?;

                // Bind pattern variables before typing body
                self.bind_pattern_variables(pattern, &typed_value.ty);

                let typed_body = self.infer_typed(body, context_builder)?;

                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::node(
                    expr.clone(),
                    ty,
                    vec![typed_value, typed_body],
                ))
            }

            // Ascription: inner expression
            Expression::Ascription { expr: inner, .. } => {
                let typed_inner = self.infer_typed(inner, context_builder)?;
                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::node(expr.clone(), ty, vec![typed_inner]))
            }

            // Lambda: body
            Expression::Lambda { params, body, .. } => {
                // Bind parameters
                for param in params {
                    let param_type = self.context.fresh_var();
                    self.context.bind(param.name.clone(), param_type);
                }

                let typed_body = self.infer_typed(body, context_builder)?;
                let ty = self.infer(expr, context_builder)?;
                Ok(TypedExpr::node(expr.clone(), ty, vec![typed_body]))
            }

            // Quantifier: bind variables, then process body
            Expression::Quantifier {
                variables, body, ..
            } => {
                // Bind quantified variables to their annotated types
                for var in variables {
                    if let Some(ref type_annotation) = var.type_annotation {
                        let ty = self.type_annotation_to_type(type_annotation);
                        self.context.bind(var.name.clone(), ty);
                    }
                }
                let typed_body = self.infer_typed(body, context_builder)?;
                let ty = typed_body.ty.clone();
                Ok(TypedExpr::node(expr.clone(), ty, vec![typed_body]))
            }
        }
    }

    /// Infer type of a pattern matching expression
    ///
    /// Pattern matching type inference (ADR-021):
    /// 1. Infer type of scrutinee
    /// 2. For each case, check pattern matches scrutinee type
    /// 3. Infer body type in context extended with pattern bindings
    /// 4. Unify all branch types (all must have same result type)
    /// 5. Check exhaustiveness (warn if missing cases)
    ///
    /// Example:
    ///   match myOption {
    ///     None => 0        // Branch type: Scalar
    ///     Some(x) => x     // Branch type: Scalar (x bound to Scalar)
    ///   }
    ///   Result type: Scalar
    fn infer_match(
        &mut self,
        scrutinee: &Expression,
        cases: &[crate::ast::MatchCase],
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        if cases.is_empty() {
            return Err("Match expression must have at least one case".to_string());
        }

        // Step 1: Infer scrutinee type
        let scrutinee_ty = self.infer(scrutinee, context_builder)?;

        // Step 2: Infer each branch and collect result types
        let mut branch_types = Vec::new();

        for (i, case) in cases.iter().enumerate() {
            // Save context before this branch
            let saved_context = self.context.clone();

            // Check pattern matches scrutinee type and bind variables
            self.check_pattern(&case.pattern, &scrutinee_ty)
                .map_err(|e| format!("In branch {}: {}", i + 1, e))?;

            // Infer body type with pattern bindings in scope
            let body_ty = self.infer(&case.body, context_builder)?;
            branch_types.push(body_ty);

            // Restore context (pattern bindings don't escape branch)
            self.context = saved_context;
        }

        // Step 3: Unify all branch types (must all return same type)
        let result_ty = branch_types[0].clone();
        for branch_ty in branch_types.iter().skip(1) {
            self.add_constraint(result_ty.clone(), branch_ty.clone());
        }

        // Step 4: Check exhaustiveness (warn if missing cases)
        self.check_match_exhaustiveness(cases, &scrutinee_ty);

        Ok(result_ty)
    }

    /// Check match exhaustiveness and warn about issues
    ///
    /// This checks:
    /// 1. Are all constructors covered? (exhaustiveness)
    /// 2. Are any patterns unreachable? (dead code)
    ///
    /// Warnings are printed to stderr but don't fail type checking.
    fn check_match_exhaustiveness(&self, cases: &[crate::ast::MatchCase], scrutinee_ty: &Type) {
        use crate::pattern_matcher::ExhaustivenessChecker;

        let checker = ExhaustivenessChecker::new(self.data_registry.clone());
        let patterns: Vec<_> = cases.iter().map(|c| c.pattern.clone()).collect();

        // Check exhaustiveness
        match checker.check_exhaustive(&patterns, scrutinee_ty) {
            Ok(()) => {} // Exhaustive - good!
            Err(missing) => {
                eprintln!(
                    "Warning: Non-exhaustive match. Missing cases: {}",
                    missing.join(", ")
                );
            }
        }

        // Check for unreachable patterns
        let unreachable = checker.check_reachable(&patterns);
        if !unreachable.is_empty() {
            for idx in unreachable {
                eprintln!("Warning: Unreachable pattern at case {}", idx + 1);
            }
        }
    }

    /// Infer type of a list literal
    ///
    /// List literal type inference:
    /// 1. Infer type of each element
    /// 2. Unify all element types (all must be same type)
    /// 3. Return List(T) where T is the unified element type
    ///
    /// Examples:
    /// - [1, 2, 3] → List(Scalar)
    /// - [x, y, z] → List(T) where T is inferred from context
    /// - [] → List(α) where α is a fresh type variable
    fn infer_list(
        &mut self,
        elements: &[Expression],
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        if elements.is_empty() {
            // Empty list: List(α) where α is fresh type variable
            let elem_ty = self.context.fresh_var();
            return Ok(Type::Data {
                type_name: "List".to_string(),
                constructor: "List".to_string(),
                args: vec![elem_ty],
            });
        }

        // Infer type of first element
        let first_ty = self.infer(&elements[0], context_builder)?;

        // All other elements must have same type
        for elem in &elements[1..] {
            let elem_ty = self.infer(elem, context_builder)?;
            self.add_constraint(first_ty.clone(), elem_ty);
        }

        // Return List(T) where T is the element type
        Ok(Type::Data {
            type_name: "List".to_string(),
            constructor: "List".to_string(),
            args: vec![first_ty],
        })
    }

    /// Check that a pattern matches the expected type and bind pattern variables
    ///
    /// This validates:
    /// - Constructors belong to the expected data type
    /// - Constructor arity matches pattern arguments
    /// - Nested patterns match their expected types recursively
    ///
    /// Side effect: Binds pattern variables in the type context
    fn check_pattern(
        &mut self,
        pattern: &crate::ast::Pattern,
        expected_ty: &Type,
    ) -> Result<(), String> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => {
                // Wildcard matches anything
                Ok(())
            }

            Pattern::Variable(name) => {
                // Variable matches anything and gets bound to the type
                self.context.bind(name.clone(), expected_ty.clone());
                Ok(())
            }

            Pattern::Constructor { name, args } => {
                // Look up constructor in data registry
                let variant_info = self
                    .data_registry
                    .lookup_variant(name)
                    .ok_or_else(|| format!("Unknown constructor: {}", name))?
                    .clone(); // Clone to release borrow

                let (type_name, variant) = variant_info;

                // Extract type arguments from scrutinee for instantiating type parameters
                let type_args = match expected_ty {
                    Type::Data {
                        type_name: scrutinee_type,
                        args: scrutinee_args,
                        ..
                    } => {
                        if type_name != *scrutinee_type {
                            return Err(format!(
                                "Pattern mismatch: constructor {} belongs to type {}, \
                                 but scrutinee has type {}",
                                name, type_name, scrutinee_type
                            ));
                        }
                        // Use the type arguments from the scrutinee
                        scrutinee_args.clone()
                    }
                    Type::Var(_) => {
                        // Type variable - we'll constrain it through unification
                        // Create fresh type variables for each type parameter
                        let data_def = self
                            .data_registry
                            .get_type(&type_name)
                            .ok_or_else(|| format!("Type {} not found", type_name))?;
                        let fresh_args: Vec<Type> = data_def
                            .type_params
                            .iter()
                            .map(|_| self.context.fresh_var())
                            .collect();

                        let constructor_ty = Type::Data {
                            type_name: type_name.clone(),
                            constructor: name.clone(),
                            args: fresh_args.clone(),
                        };
                        self.add_constraint(expected_ty.clone(), constructor_ty);
                        fresh_args
                    }
                    _ => {
                        return Err(format!(
                            "Pattern mismatch: constructor {} expects data type, \
                             but scrutinee has type {:?}",
                            name, expected_ty
                        ));
                    }
                };

                // Check arity
                if variant.fields.len() != args.len() {
                    return Err(format!(
                        "Constructor {} expects {} arguments, got {}",
                        name,
                        variant.fields.len(),
                        args.len()
                    ));
                }

                // Get the data type definition to know type parameters
                // Clone to release borrow before recursive call
                let type_params = self
                    .data_registry
                    .get_type(&type_name)
                    .ok_or_else(|| format!("Type {} not found in registry", type_name))?
                    .type_params
                    .clone();

                // Recursively check nested patterns with type parameter substitution
                for (pattern_arg, field) in args.iter().zip(&variant.fields) {
                    // Convert TypeExpr to Type for the field, substituting type parameters
                    let field_ty = self.type_expr_to_type_with_params(
                        &field.type_expr,
                        &type_params,
                        &type_args,
                    )?;
                    self.check_pattern(pattern_arg, &field_ty)?;
                }

                Ok(())
            }

            Pattern::Constant(value) => {
                // Constant patterns must match primitive types
                // For now, assume numeric constants are Scalars
                // TODO: Add proper constant type checking
                if value.chars().all(|c| c.is_numeric() || c == '.') {
                    // Numeric constant - should be Scalar
                    let scalar_ty = Type::scalar();
                    self.add_constraint(expected_ty.clone(), scalar_ty);
                }
                Ok(())
            }

            // Grammar v0.8: As-pattern binds alias AND recurses into inner pattern
            Pattern::As { pattern, binding } => {
                // Bind the alias to the expected type
                self.context.bind(binding.clone(), expected_ty.clone());
                // Recursively check the inner pattern
                self.check_pattern(pattern, expected_ty)
            }
        }
    }

    /// Parse a type annotation string into a Type
    ///
    /// This handles annotations like "Matrix(3, 3, ℝ)" and converts them to Type.
    /// For parametric types with concrete dimensions, this creates a Data type
    /// with the dimensions captured.
    fn parse_type_annotation(&self, annotation: &str) -> Type {
        // Handle common base types
        match annotation.trim() {
            "ℝ" | "Real" => return Type::scalar(),
            "ℂ" | "Complex" => {
                return Type::Data {
                    type_name: "Type".to_string(),
                    constructor: "Complex".to_string(),
                    args: vec![],
                }
            }
            "ℤ" | "Int" | "Integer" => {
                return Type::Data {
                    type_name: "Type".to_string(),
                    constructor: "Integer".to_string(),
                    args: vec![],
                }
            }
            "ℕ" | "Nat" => return Type::Nat,
            "𝔹" | "Bool" => return Type::Bool,
            "String" => return Type::String,
            _ => {}
        }

        // Handle parametric types like Matrix(3, 3, ℝ)
        if let Some(paren_start) = annotation.find('(') {
            let type_name = annotation[..paren_start].trim();
            if let Some(paren_end) = annotation.rfind(')') {
                let params_str = &annotation[paren_start + 1..paren_end];

                match type_name {
                    "Matrix" => {
                        // Parse Matrix(m, n, T)
                        let parts: Vec<&str> = params_str.split(',').collect();
                        if parts.len() >= 2 {
                            if let (Ok(m), Ok(n)) = (
                                parts[0].trim().parse::<usize>(),
                                parts[1].trim().parse::<usize>(),
                            ) {
                                // Parse element type if present, default to ℝ
                                let elem_type = if parts.len() >= 3 {
                                    self.parse_type_annotation(parts[2].trim())
                                } else {
                                    Type::scalar()
                                };
                                return Type::matrix(m, n, elem_type);
                            }
                        }
                    }
                    "Vector" => {
                        // Parse Vector(n, T)
                        let parts: Vec<&str> = params_str.split(',').collect();
                        if let Ok(n) = parts[0].trim().parse::<usize>() {
                            // Parse element type if present, default to ℝ
                            let elem_type = if parts.len() >= 2 {
                                self.parse_type_annotation(parts[1].trim())
                            } else {
                                Type::scalar()
                            };
                            return Type::vector(n, elem_type);
                        }
                    }
                    _ => {}
                }
            }
        }

        // For anything else, return a Data type with the name
        Type::Data {
            type_name: annotation.trim().to_string(),
            constructor: annotation.trim().to_string(),
            args: vec![],
        }
    }

    /// Bind all variables in a pattern to a type (Grammar v0.8: for let destructuring)
    ///
    /// This is used for `let pattern = value in body` where we need to bind
    /// all variables in the pattern before inferring the body type.
    fn bind_pattern_variables(&mut self, pattern: &crate::ast::Pattern, ty: &Type) {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => {
                // Nothing to bind
            }
            Pattern::Variable(name) => {
                self.context.bind(name.clone(), ty.clone());
            }
            Pattern::Constructor { name, args } => {
                if args.is_empty() {
                    // Nullary constructor like `A` or `B` - treat as variable binding
                    // This handles `let A : Matrix(3,3,ℝ) = ...` where A is a variable name
                    self.context.bind(name.clone(), ty.clone());
                } else {
                    // For constructor patterns with args, we'd need to look up field types
                    // For now, bind each arg to a fresh type variable
                    for arg in args {
                        let fresh_ty = self.context.fresh_var();
                        self.bind_pattern_variables(arg, &fresh_ty);
                    }
                }
            }
            Pattern::Constant(_) => {
                // Nothing to bind
            }
            Pattern::As {
                pattern: inner,
                binding,
            } => {
                // Bind the alias
                self.context.bind(binding.clone(), ty.clone());
                // Recurse into inner pattern
                self.bind_pattern_variables(inner, ty);
            }
        }
    }

    /// Convert a TypeExpr (from AST) to a Type (for inference) with type parameter substitution
    ///
    /// This version handles parametric types by substituting type parameters with concrete types.
    ///
    /// Example: For `Some(value: T)` where T is bound to Scalar:
    ///   - type_params = ["T"]
    ///   - type_args = [Scalar]
    ///   - Result: value has type Scalar
    fn type_expr_to_type_with_params(
        &mut self,
        type_expr: &crate::kleis_ast::TypeExpr,
        type_params: &[crate::kleis_ast::TypeParam],
        type_args: &[Type],
    ) -> Result<Type, String> {
        use crate::kleis_ast::TypeExpr;

        match type_expr {
            TypeExpr::Var(param_name) => {
                // Look up in type parameters
                for (i, param) in type_params.iter().enumerate() {
                    if param.name == *param_name {
                        // Found! Return the corresponding type argument
                        return Ok(type_args
                            .get(i)
                            .ok_or_else(|| {
                                format!("Type parameter {} index out of bounds", param_name)
                            })?
                            .clone());
                    }
                }
                // Not a known type parameter - create fresh var
                Ok(self.context.fresh_var())
            }
            TypeExpr::Named(_name) => {
                // Delegate to regular conversion
                self.type_expr_to_type(type_expr)
            }
            TypeExpr::Parametric(name, params) => {
                // Parametric type like List(T) - recursively substitute
                let param_types: Result<Vec<Type>, String> = params
                    .iter()
                    .map(|p| self.type_expr_to_type_with_params(p, type_params, type_args))
                    .collect();
                let param_types = param_types?;

                Ok(Type::Data {
                    type_name: "Type".to_string(), // Meta-type
                    constructor: name.clone(),
                    args: param_types,
                })
            }
            _ => {
                // For other cases, delegate to regular conversion
                self.type_expr_to_type(type_expr)
            }
        }
    }

    /// Convert a TypeExpr (from AST) to a Type (for inference)
    ///
    /// This is needed to translate field types from data definitions into
    /// types we can use for pattern checking.
    ///
    /// Note: Now takes &mut self to handle type variables (creates fresh vars)
    fn type_expr_to_type(
        &mut self,
        type_expr: &crate::kleis_ast::TypeExpr,
    ) -> Result<Type, String> {
        use crate::kleis_ast::TypeExpr;

        match type_expr {
            TypeExpr::Named(name) if name == "ℝ" || name == "Scalar" => Ok(Type::scalar()),
            TypeExpr::Named(name) if name == "Nat" => Ok(Type::Nat),
            TypeExpr::Named(name) if name == "String" => Ok(Type::String),
            TypeExpr::Named(name) if name == "Bool" => Ok(Type::Bool),
            TypeExpr::Named(name) if name == "Unit" || name == "()" => Ok(Type::Unit),
            TypeExpr::Named(name) => {
                // Check if it's a user-defined data type
                if self.data_registry.has_type(name) {
                    Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(), // Use type name as constructor for now
                        args: vec![],
                    })
                } else {
                    // Unknown type - could be a type variable in the definition
                    // Treat single capital letters as type variables (Haskell convention)
                    if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                        // Type variable like T, U, V - create fresh type variable
                        Ok(self.context.fresh_var())
                    } else {
                        // Truly unknown type - error
                        Err(format!("Unknown type: {}", name))
                    }
                }
            }
            TypeExpr::Parametric(name, params) => {
                // Handle parametric types like Vector(n), Matrix(m, n)
                let param_types: Result<Vec<Type>, String> =
                    params.iter().map(|p| self.type_expr_to_type(p)).collect();
                let param_types = param_types?;

                Ok(Type::Data {
                    type_name: "Type".to_string(), // Meta-type
                    constructor: name.clone(),
                    args: param_types,
                })
            }
            TypeExpr::Var(_name) => {
                // Type variable in the definition (e.g., T in Option(T))
                // Create a fresh type variable for this type parameter
                // The HM algorithm will unify these correctly across the function
                //
                // Note: This creates a NEW type variable each time.
                // For proper polymorphism, we'd want to share the same type variable
                // for all occurrences of "T" in the same context, but for pattern
                // matching this works because unification will connect them.
                Ok(self.context.fresh_var())
            }
            TypeExpr::Function(_, _) => {
                // Function types in patterns not supported yet
                Err("Function types in patterns not yet supported".to_string())
            }
            TypeExpr::Product(_) => {
                // Product types in patterns not supported yet
                Err("Product types in patterns not yet supported".to_string())
            }
            TypeExpr::ForAll { vars: _, body } => {
                // Quantified type - strip the quantifier and convert the body
                // The HM type system handles polymorphism implicitly through
                // type variables and generalization, so we don't need the explicit quantifiers
                self.type_expr_to_type(body)
            }
            TypeExpr::DimExpr(dim) => {
                // v0.92: Dimension expressions preserve symbolic structure
                // for proper dimension unification
                Ok(Type::NatExpr(dim.clone()))
            }
        }
    }

    /// Infer type of an operation
    /// ADR-016 COMPLIANT: Delegates ALL operations to TypeContextBuilder.
    /// Only keeps Matrix constructors (they're literals, not operations).
    ///
    /// TODO(ADR-021): Matrix constructors are DATA CONSTRUCTORS
    /// With `data` keyword, these would be handled generically:
    /// ```kleis
    /// data Type = ... | Matrix(Nat, Nat)
    /// // Matrix(...) becomes a generic data constructor!
    /// // No special case needed!
    /// ```
    fn infer_operation(
        &mut self,
        name: &str,
        args: &[Expression],
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        // Unit type: () has type Unit
        if name == "Unit" && args.is_empty() {
            return Ok(Type::Unit);
        }

        // Matrix and Vector are now FIXED-ARITY data constructors using List literals!
        //
        // Matrix(2, 2, [a, b, c, d]) - 3 args (fixed!)
        // Vector(3, [x, y, z]) - 2 args (fixed!)
        //
        // They are handled by the generic data constructor path below.
        // No special case needed!

        // ADR-021: Check if this is a (fixed-arity) data constructor
        if self.data_registry.has_variant(name) {
            return self.infer_data_constructor(name, args, context_builder);
        }

        // OPERATOR OVERLOADING: Complex number constructor
        // complex(re, im) returns Type::Data { constructor: "Complex", ... }
        if name == "complex" && args.len() == 2 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: Complex accessors
        // re(z), im(z) return Scalar
        if matches!(name, "re" | "im") && args.len() == 1 {
            return Ok(Type::scalar());
        }

        // OPERATOR OVERLOADING: Conjugate and complex functions
        // conj(z), neg_complex(z), complex_inverse(z) return Complex
        if matches!(name, "conj" | "neg_complex" | "complex_inverse") && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: Complex binary operations
        // complex_add, complex_sub, complex_mul, complex_div return Complex
        if matches!(
            name,
            "complex_add" | "complex_sub" | "complex_mul" | "complex_div"
        ) && args.len() == 2
        {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: abs_squared returns Scalar
        if name == "abs_squared" && args.len() == 1 {
            return Ok(Type::scalar());
        }

        // OPERATOR OVERLOADING: Rational number constructor
        // rational(numer, denom) returns Rational
        if name == "rational" && args.len() == 2 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Rational".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: Rational accessors
        // numer(r), denom(r) return Int
        if matches!(name, "numer" | "denom") && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: Rational unary operations
        // neg_rational(r), abs_rational(r), rational_inv(r), canonical(r) return Rational
        if matches!(
            name,
            "neg_rational" | "abs_rational" | "rational_inv" | "canonical"
        ) && args.len() == 1
        {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Rational".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: Rational binary operations
        // rational_add, rational_sub, rational_mul, rational_div, min_rational, max_rational, midpoint return Rational
        if matches!(
            name,
            "rational_add"
                | "rational_sub"
                | "rational_mul"
                | "rational_div"
                | "min_rational"
                | "max_rational"
                | "midpoint"
        ) && args.len() == 2
        {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Rational".to_string(),
                args: vec![],
            });
        }

        // sign_rational(r : ℚ) : ℤ
        if name == "sign_rational" && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            });
        }

        // floor(r : ℚ) : ℤ, ceil(r : ℚ) : ℤ
        if matches!(name, "floor" | "ceil" | "ceiling") && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            });
        }

        // int_div, int_mod, int_rem : ℤ × ℤ → ℤ
        if matches!(
            name,
            "int_div" | "div" | "int_mod" | "mod" | "int_rem" | "rem"
        ) && args.len() == 2
        {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            });
        }

        // gcd : ℤ × ℤ → ℤ (or ℕ × ℕ → ℕ)
        if name == "gcd" && args.len() == 2 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            });
        }

        // ============================================
        // BIT-VECTOR OPERATIONS
        // ============================================

        // BitVec binary operations: bvand, bvor, bvxor, bvadd, bvsub, bvmul, bvshl, bvlshr, bvashr
        // These preserve the BitVec type
        if matches!(
            name,
            "bvand"
                | "bvor"
                | "bvxor"
                | "bvadd"
                | "bvsub"
                | "bvmul"
                | "bvudiv"
                | "bvsdiv"
                | "bvurem"
                | "bvshl"
                | "bvlshr"
                | "bvashr"
        ) && args.len() == 2
        {
            // Return BitVec type (width preserved from first argument)
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "BitVec".to_string(),
                args: vec![], // Width would be a type parameter in full dependent type system
            });
        }

        // BitVec unary operations: bvnot, bvneg
        if matches!(name, "bvnot" | "bvneg") && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "BitVec".to_string(),
                args: vec![],
            });
        }

        // BitVec comparison operations return Bool
        if matches!(
            name,
            "bvult" | "bvule" | "bvugt" | "bvuge" | "bvslt" | "bvsle" | "bvsgt" | "bvsge"
        ) && args.len() == 2
        {
            return Ok(Type::Bool);
        }

        // Bit extraction: bit(x, i) returns a single bit (0 or 1)
        if name == "bit" && args.len() == 2 {
            return Ok(Type::Nat);
        }

        // Width accessor
        if name == "width" && args.len() == 1 {
            return Ok(Type::Nat);
        }

        // Zero/ones constructors: bvzero(n), bvones(n), bvone(n) return BitVec(n)
        if matches!(name, "bvzero" | "bvones" | "bvone") && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "BitVec".to_string(),
                args: vec![],
            });
        }

        // OPERATOR OVERLOADING: Rational comparison operations return Bool
        if matches!(
            name,
            "rational_lt" | "rational_le" | "rational_gt" | "rational_ge"
        ) && args.len() == 2
        {
            return Ok(Type::Bool);
        }

        // OPERATOR OVERLOADING: Rational to Real conversion
        if name == "to_real" && args.len() == 1 {
            return Ok(Type::scalar());
        }

        // OPERATOR OVERLOADING: Integer/Natural to Rational conversion
        if matches!(name, "int_to_rational" | "nat_to_rational") && args.len() == 1 {
            return Ok(Type::Data {
                type_name: "Type".to_string(),
                constructor: "Rational".to_string(),
                args: vec![],
            });
        }

        // ADR-016: Arithmetic operations with type promotion
        // Uses registry-based inference with Promotes(From, To) for mixed types
        // Type hierarchy: ℕ → ℤ → ℚ → ℝ → ℂ
        if matches!(
            name,
            "plus" | "minus" | "times" | "divide" | "scalar_divide"
        ) && args.len() == 2
        {
            let t1 = self.infer(&args[0], context_builder)?;
            let t2 = self.infer(&args[1], context_builder)?;

            // Helper to extract constructor name from type
            let get_constructor = |t: &Type| -> Option<String> {
                match t {
                    Type::Data { constructor, .. } => Some(constructor.clone()),
                    Type::Var(_) => None, // Type variable - unknown
                    _ => None,
                }
            };

            let c1 = get_constructor(&t1);
            let c2 = get_constructor(&t2);

            // Case 1: Both types are concrete and same constructor
            if let (Some(ref con1), Some(ref con2)) = (&c1, &c2) {
                if con1 == con2 {
                    // Same constructor - for parametric types, verify Nat parameters match
                    // This is GENERIC: works for Matrix(m,n,T), Vector(n,T), Tensor(i,j,k,T),
                    // or any user-defined type with Nat parameters
                    if let (Type::Data { args: args1, .. }, Type::Data { args: args2, .. }) =
                        (&t1, &t2)
                    {
                        // Check that dimension parameters (NatValues) match
                        if !args1.is_empty() && !args2.is_empty() {
                            for (a1, a2) in args1.iter().zip(args2.iter()) {
                                match (a1, a2) {
                                    (Type::NatValue(n1), Type::NatValue(n2)) if n1 != n2 => {
                                        return Err(format!(
                                            "Dimension mismatch in {} operation: {} has parameter {} but {} has parameter {}",
                                            name, con1, n1, con2, n2
                                        ));
                                    }
                                    _ => {} // Type params, element types, etc. - OK
                                }
                            }
                        }
                    }
                    // All parameters match (or no parameters) - return t1
                    return Ok(t1.clone());
                }

                // Case 2: Different constructors for non-scalar types
                // Check if either type has Nat parameters (indicating a sized type)
                let has_nat_params = |t: &Type| -> bool {
                    if let Type::Data { args, .. } = t {
                        args.iter().any(|a| matches!(a, Type::NatValue(_)))
                    } else {
                        false
                    }
                };

                if has_nat_params(&t1) || has_nat_params(&t2) {
                    // Sized types (parametric with Nat) can't be combined with different types
                    return Err(format!(
                        "Type mismatch in {} operation: cannot combine {} and {}",
                        name, con1, con2
                    ));
                }

                // Case 3: Scalar types - use type promotion hierarchy
                // Query registry for common supertype
                if let Some(builder) = context_builder {
                    if let Some(common_type) = builder.find_common_supertype(con1, con2) {
                        return Ok(Type::Data {
                            type_name: "Type".to_string(),
                            constructor: common_type,
                            args: vec![],
                        });
                    }
                }

                // Fallback: return t1's type (arbitrary choice for unknown cases)
                return Ok(t1.clone());
            }

            // Case 4: One or both are type variables (placeholders)
            // If one is concrete, return that; otherwise return Scalar as default
            if let Some(con) = c1.or(c2) {
                return Ok(Type::Data {
                    type_name: "Type".to_string(),
                    constructor: con,
                    args: vec![],
                });
            }

            // Both are type variables - default to Scalar
            return Ok(Type::scalar());
        }

        // OPERATOR OVERLOADING: Unary negation
        if name == "negate" && args.len() == 1 {
            let t = self.infer(&args[0], context_builder)?;
            return Ok(t);
        }

        // OPERATOR OVERLOADING: Comparison operations
        // These return Bool, but we need to type-check operands
        if matches!(name, "equals" | "not_equals" | "neq") && args.len() == 2 {
            // Equality works on any type
            let _t1 = self.infer(&args[0], context_builder)?;
            let _t2 = self.infer(&args[1], context_builder)?;
            return Ok(Type::Bool);
        }

        // Ordering operations only work on orderable types (Scalar, Int, Nat, Real, Rational)
        // They do NOT work on Complex, Matrix, Bool, etc.
        if matches!(
            name,
            "less_than"
                | "greater_than"
                | "less_equal"
                | "greater_equal"
                | "leq"
                | "geq"
                | "lt"
                | "gt"
        ) && args.len() == 2
        {
            let t1 = self.infer(&args[0], context_builder)?;
            let t2 = self.infer(&args[1], context_builder)?;

            // Check if operands are orderable (numeric scalar types)
            let is_orderable = |t: &Type| {
                matches!(t, Type::Nat | Type::NatValue(_) | Type::Var(_))
                    || matches!(
                        t,
                        Type::Data { constructor, .. }
                            if constructor == "Int" || constructor == "Real" || constructor == "Scalar" || constructor == "Rational"
                    )
            };

            if is_orderable(&t1) && is_orderable(&t2) {
                return Ok(Type::Bool);
            }
            // If not orderable, fall through to context_builder for proper error
        }

        // Check if this is a defined function (from `define` statements)
        // Functions are stored in the type inference context
        if let Some(func_ty) = self.context.get(name) {
            // Found a function! For now, we return its type
            // TODO: Proper function application with currying (Wire 3)
            // For now, functions just return their body type regardless of args
            return Ok(func_ty.clone());
        }

        // Delegate to context_builder (ADR-016!)
        let arg_types: Vec<Type> = args
            .iter()
            .map(|arg| self.infer(arg, context_builder))
            .collect::<Result<Vec<_>, _>>()?;

        // If context_builder is available, query the registry
        if let Some(builder) = context_builder {
            builder.infer_operation_type(name, &arg_types, &self.data_registry)
        } else {
            // No context builder - return fresh variable (for backwards compatibility)
            // This allows tests without context_builder to still run
            Ok(self.context.fresh_var())
        }
    }

    /// Generic data constructor inference (ADR-021)
    ///
    /// This replaces hardcoded constructor logic (like infer_matrix_constructor)
    /// with generic lookup-based inference that works for ANY data constructor.
    ///
    /// Examples:
    /// - Scalar → Data { type_name: "Type", constructor: "Scalar", args: [] }
    /// - Matrix(2, 3, a, b, c, d, e, f) → Data { type_name: "Type", constructor: "Matrix", args: [Nat, Nat] }
    /// - Some(42) → Data { type_name: "Option", constructor: "Some", args: [infer(42)] }
    ///
    /// Algorithm:
    /// 1. Lookup variant definition in registry
    /// 2. Validate argument count
    /// 3. For each field:
    ///    - If type parameter (Nat, String): extract constant value
    ///    - If value field: infer type and add constraint
    /// 4. Construct Type::Data with extracted parameters
    fn infer_data_constructor(
        &mut self,
        constructor_name: &str,
        args: &[Expression],
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        // Lookup variant definition and clone to avoid borrow issues
        let (type_name, variant) = self
            .data_registry
            .lookup_variant(constructor_name)
            .ok_or_else(|| format!("Unknown data constructor: {}", constructor_name))?
            .clone(); // Clone to release borrow on self.data_registry

        // Validate argument count
        let expected_fields = variant.fields.len();
        if args.len() != expected_fields {
            return Err(format!(
                "Constructor {} expects {} arguments, got {}",
                constructor_name,
                expected_fields,
                args.len()
            ));
        }

        // Get the parent data type definition to know type parameters
        let data_def = self
            .data_registry
            .get_type(&type_name)
            .ok_or_else(|| format!("Data type {} not found", type_name))?
            .clone();

        // Build type arguments from constructor fields
        // If constructor has no fields but parent has type params, use fresh vars
        let mut constructor_args = Vec::new();

        if variant.fields.is_empty() && !data_def.type_params.is_empty() {
            // Constructor has no fields (like None), but parent has type params (like Option(T))
            // Create fresh type variables for each type parameter
            for _type_param in &data_def.type_params {
                constructor_args.push(self.context.fresh_var());
            }
        } else {
            // Constructor has fields - infer from actual arguments
            for (i, (arg_expr, field_def)) in args.iter().zip(&variant.fields).enumerate() {
                // Check if this is a type parameter (like dimensions) vs value field
                match &field_def.type_expr {
                    crate::kleis_ast::TypeExpr::Named(name) if name == "Nat" => {
                        // This is a dimension/index parameter - must be constant
                        match arg_expr {
                            Expression::Const(s) => {
                                // Extract actual numeric value
                                let value = s.parse::<usize>().map_err(|_| {
                                    format!(
                                        "Constructor parameter {} must be a valid number: {}",
                                        i, s
                                    )
                                })?;
                                constructor_args.push(Type::NatValue(value));
                            }
                            _ => {
                                return Err(format!(
                                    "Constructor parameter {} must be constant (Nat expected)",
                                    i
                                ));
                            }
                        }
                    }
                    crate::kleis_ast::TypeExpr::Named(name) if name == "String" => {
                        // String parameter - can be String literal or Const
                        match arg_expr {
                            Expression::String(s) => {
                                // Store actual string value (from "..." literal)
                                constructor_args.push(Type::StringValue(s.clone()));
                            }
                            Expression::Const(s) => {
                                // Also allow Const for backwards compatibility
                                constructor_args.push(Type::StringValue(s.clone()));
                            }
                            _ => {
                                return Err(format!(
                                    "Constructor parameter {} must be a string literal (String expected)",
                                    i
                                ));
                            }
                        }
                    }
                    _ => {
                        // This is a value field - infer its type and include it in result
                        let arg_type = self.infer(arg_expr, context_builder)?;
                        // TODO: Add constraint that arg_type matches field_def.type_expr
                        constructor_args.push(arg_type);
                    }
                }
            }
        }

        Ok(Type::Data {
            type_name,
            constructor: constructor_name.to_string(),
            args: constructor_args,
        })
    }

    /// Infer types of data constructor fields
    /// This is GENERIC! Works for any data constructor with typed fields.
    ///
    /// TODO(ADR-021): This logic is already generic and would work for all data constructors!
    /// Example: Cons(head: T, tail: List(T)) would validate head and tail fields.
    #[allow(dead_code)]
    fn infer_data_constructor_fields(
        &mut self,
        field_exprs: &[Expression],
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
        expected_type: Type,
    ) -> Result<(), String> {
        for field_expr in field_exprs {
            let field_type = self.infer(field_expr, context_builder)?;

            match field_type {
                Type::Var(_) => {
                    // Type variable (placeholder) - OK, will be unified later
                }
                _ => {
                    // Concrete type - add constraint that it matches expected type
                    self.add_constraint(field_type, expected_type.clone());
                }
            }
        }
        Ok(())
    }

    /// Solve all constraints using unification
    pub fn solve(&self) -> Result<Substitution, String> {
        let mut subst = Substitution::empty();

        for constraint in &self.constraints {
            let t1 = subst.apply(&constraint.left);
            let t2 = subst.apply(&constraint.right);

            let new_subst = unify(&t1, &t2)?;
            subst = subst.compose(&new_subst);
        }

        Ok(subst)
    }

    /// Infer and solve: complete type inference
    pub fn infer_and_solve(
        &mut self,
        expr: &Expression,
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        let ty = self.infer(expr, context_builder)?;
        let subst = self.solve()?;
        Ok(subst.apply(&ty))
    }
}

/// Unification: make two types equal
///
/// TODO(ADR-021): This pattern matching should be in Kleis, not Rust!
///
/// Future (Phase 3): Once Type is defined with `data`, unification becomes:
/// ```kleis
/// operation unify : Type → Type → Option(Substitution)
///
/// define unify(t1, t2) = match (t1, t2) {
///   (Scalar, Scalar) => Some(empty)
///   (Vector(n), Vector(m)) if n == m => Some(empty)
///   (Matrix(r1,c1), Matrix(r2,c2)) if r1==r2 && c1==c2 => Some(empty)
///   (Var(id), t) => Some(bind(id, t))
///   (t, Var(id)) => Some(bind(id, t))
///   (Function(a1,b1), Function(a2,b2)) =>
///     unify(a1,a2).and_then(s1 =>
///       unify(s1(b1), s1(b2)).map(s2 => compose(s1,s2)))
///   _ => None
/// }
/// ```
///
/// Benefits:
/// - Unification logic in Kleis, not Rust
/// - Users can see/modify unification rules
/// - Extensible to user-defined types
/// - Meta-circular type system!
fn unify(t1: &Type, t2: &Type) -> Result<Substitution, String> {
    match (t1, t2) {
        // Bootstrap types unify with themselves
        (Type::Nat, Type::Nat) => Ok(Substitution::empty()),
        (Type::NatValue(n1), Type::NatValue(n2)) if n1 == n2 => Ok(Substitution::empty()),
        (Type::NatValue(n1), Type::NatValue(n2)) => Err(format!(
            "Cannot unify different dimensions: {} vs {}",
            n1, n2
        )),

        // Symbolic dimension expressions: delegate to dimension solver
        (Type::NatExpr(e1), Type::NatExpr(e2)) => {
            use crate::dimension_solver::{unify_dims, DimUnifyResult};
            match unify_dims(e1, e2) {
                DimUnifyResult::Equal(_) => Ok(Substitution::empty()),
                DimUnifyResult::Unequal(msg) => Err(msg),
            }
        }

        // NatExpr vs NatValue: check if expression evaluates to value
        (Type::NatExpr(e), Type::NatValue(n)) | (Type::NatValue(n), Type::NatExpr(e)) => {
            use crate::dimension_solver::{unify_dims, DimUnifyResult};
            use crate::kleis_ast::DimExpr;
            match unify_dims(e, &DimExpr::Lit(*n)) {
                DimUnifyResult::Equal(_) => Ok(Substitution::empty()),
                DimUnifyResult::Unequal(msg) => Err(msg),
            }
        }

        // Nat (kind) unifies with any dimension
        (Type::Nat, Type::NatValue(_))
        | (Type::NatValue(_), Type::Nat)
        | (Type::Nat, Type::NatExpr(_))
        | (Type::NatExpr(_), Type::Nat) => Ok(Substitution::empty()),

        (Type::String, Type::String) => Ok(Substitution::empty()),
        (Type::StringValue(s1), Type::StringValue(s2)) if s1 == s2 => Ok(Substitution::empty()),
        (Type::StringValue(s1), Type::StringValue(s2)) => Err(format!(
            "Cannot unify different strings: {:?} vs {:?}",
            s1, s2
        )),
        (Type::Bool, Type::Bool) => Ok(Substitution::empty()),
        (Type::Unit, Type::Unit) => Ok(Substitution::empty()),

        // Data types: must have same type and constructor, then unify args
        (
            Type::Data {
                type_name: t1,
                constructor: c1,
                args: a1,
            },
            Type::Data {
                type_name: t2,
                constructor: c2,
                args: a2,
            },
        ) => {
            // Must be from the same parent ADT type
            // All constructors of the same ADT produce values of that type
            // (True and False both produce Bool values, not different types)
            if t1 != t2 {
                return Err(format!(
                    "Cannot unify types from different data types: {} vs {}",
                    t1, t2
                ));
            }

            // Constructor names are VALUE-level distinctions (True vs False),
            // not TYPE-level distinctions. We unify based on the parent ADT.
            // This is how Haskell/ML handle algebraic data types.
            //
            // REMOVED: Constructor name check (was incorrectly preventing
            // True and False from unifying, even though both are type Bool)

            // Must have same number of type arguments
            if a1.len() != a2.len() {
                return Err(format!(
                    "Cannot unify {}({} type args) with {}({} type args)",
                    c1,
                    a1.len(),
                    c2,
                    a2.len()
                ));
            }

            // Unify all arguments recursively
            let mut subst = Substitution::empty();
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                let s = unify(&subst.apply(arg1), &subst.apply(arg2))?;
                subst = subst.compose(&s);
            }
            Ok(subst)
        }

        // Type variable unifies with anything (if not occurs)
        (Type::Var(v1), Type::Var(v2)) if v1 == v2 => {
            // Same type variable - trivially unifies with itself
            Ok(Substitution::empty())
        }
        (Type::Var(v), t) | (t, Type::Var(v)) => {
            if occurs(v, t) {
                Err(format!("Occurs check failed: {:?} occurs in {:?}", v, t))
            } else {
                Ok(Substitution::singleton(v.clone(), t.clone()))
            }
        }

        // Otherwise: cannot unify
        _ => Err(format!("Cannot unify {} with {}", t1, t2)),
    }
}

/// Occurs check: does variable v occur in type t?
fn occurs(v: &TypeVar, t: &Type) -> bool {
    match t {
        Type::Var(v2) => v == v2,
        Type::Data { args, .. } => args.iter().any(|arg| occurs(v, arg)),
        Type::ForAll(_, t) => occurs(v, t),
        // Leaf types (no type variables can occur in them)
        // NatExpr has dimension variables, not type variables
        Type::Nat
        | Type::NatValue(_)
        | Type::NatExpr(_)
        | Type::String
        | Type::StringValue(_)
        | Type::Bool
        | Type::Unit => false,
    }
}

impl Type {
    /// Create a Scalar type (backward compatibility)
    ///
    /// This is a convenience constructor to ease the transition from
    /// the old hardcoded Type::Scalar to the new Data-based system.
    #[allow(clippy::doc_lazy_continuation)]
    pub fn scalar() -> Type {
        Type::Data {
            type_name: "Type".to_string(),
            constructor: "Scalar".to_string(),
            args: vec![],
        }
    }

    /// Create a Vector type (backward compatibility)
    ///
    /// The dimension is stored as a concrete value, enabling:
    /// - Vector(3) ≠ Vector(4) (different types!)
    /// - Dimension checking in operations
    ///
    /// Create a Vector type
    ///
    /// Vector(n, T) where:
    /// - n is dimension (Nat value)
    /// - T is element type (Type)
    #[allow(clippy::doc_lazy_continuation)]
    pub fn vector(n: usize, elem_type: Type) -> Type {
        Type::Data {
            type_name: "Vector".to_string(),
            constructor: "Vector".to_string(),
            args: vec![Type::NatValue(n), elem_type],
        }
    }

    /// Create a Matrix type
    ///
    /// Matrix(m, n, T) where:
    /// - m, n are dimensions (Nat values)
    /// - T is element type (Type)
    ///
    /// Dimensions are stored as concrete values, enabling:
    /// - Matrix(2,3,ℝ) ≠ Matrix(2,2,ℝ) (different types!)
    /// - Matrix(2,3,ℝ) × Matrix(3,4,ℝ) → Matrix(2,4,ℝ) dimension checking
    pub fn matrix(m: usize, n: usize, elem_type: Type) -> Type {
        Type::Data {
            type_name: "Matrix".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![Type::NatValue(m), Type::NatValue(n), elem_type],
        }
    }

    /// Create a PMatrix type (matrix with parentheses)
    pub fn pmatrix(m: usize, n: usize, elem_type: Type) -> Type {
        Type::Data {
            type_name: "PMatrix".to_string(),
            constructor: "PMatrix".to_string(),
            args: vec![Type::NatValue(m), Type::NatValue(n), elem_type],
        }
    }

    /// Create a VMatrix type (matrix with vertical bars, for determinants)
    pub fn vmatrix(m: usize, n: usize, elem_type: Type) -> Type {
        Type::Data {
            type_name: "VMatrix".to_string(),
            constructor: "VMatrix".to_string(),
            args: vec![Type::NatValue(m), Type::NatValue(n), elem_type],
        }
    }

    /// Create a BMatrix type (matrix with braces)
    pub fn bmatrix(m: usize, n: usize, elem_type: Type) -> Type {
        Type::Data {
            type_name: "BMatrix".to_string(),
            constructor: "BMatrix".to_string(),
            args: vec![Type::NatValue(m), Type::NatValue(n), elem_type],
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Bootstrap types
            Type::Nat => write!(f, "Nat"),
            Type::NatValue(n) => write!(f, "{}", n),
            Type::String => write!(f, "String"),
            Type::StringValue(s) => write!(f, "\"{}\"", s),
            Type::Bool => write!(f, "Bool"),
            Type::Unit => write!(f, "Unit"),

            // User-defined data types
            Type::Data {
                constructor, args, ..
            } => {
                if args.is_empty() {
                    // Simple constructor: Scalar, True, None
                    write!(f, "{}", constructor)
                } else {
                    // Parameterized constructor: Matrix(2, 3), Some(T)
                    write!(f, "{}(", constructor)?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ")")
                }
            }

            // Symbolic dimension expression (v0.92)
            Type::NatExpr(dim) => write!(f, "{}", format_dim_expr(dim)),

            // Meta-level types
            Type::Var(TypeVar(n)) => write!(f, "α{}", n),
            Type::ForAll(TypeVar(n), t) => write!(f, "∀α{}. {}", n, t),
        }
    }
}

/// Format a dimension expression for display
fn format_dim_expr(dim: &crate::kleis_ast::DimExpr) -> String {
    use crate::kleis_ast::DimExpr;
    match dim {
        DimExpr::Lit(n) => n.to_string(),
        DimExpr::Var(name) => name.clone(),
        DimExpr::Add(l, r) => format!("({}+{})", format_dim_expr(l), format_dim_expr(r)),
        DimExpr::Sub(l, r) => format!("({}-{})", format_dim_expr(l), format_dim_expr(r)),
        DimExpr::Mul(l, r) => format!("({}*{})", format_dim_expr(l), format_dim_expr(r)),
        DimExpr::Div(l, r) => format!("({}/{})", format_dim_expr(l), format_dim_expr(r)),
        DimExpr::Pow(l, r) => format!("({}^{})", format_dim_expr(l), format_dim_expr(r)),
        DimExpr::Call(name, args) => {
            let arg_strs: Vec<_> = args.iter().map(format_dim_expr).collect();
            format!("{}({})", name, arg_strs.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_context::TypeContextBuilder;

    /// Helper to create a context builder with minimal stdlib
    fn create_test_context() -> TypeContextBuilder {
        use crate::kleis_parser::parse_kleis_program;

        let minimal_prelude = include_str!("../stdlib/minimal_prelude.kleis");
        let program =
            parse_kleis_program(minimal_prelude).expect("Failed to parse minimal_prelude.kleis");

        TypeContextBuilder::from_program(program)
            .expect("Failed to build context from minimal_prelude")
    }

    #[test]
    fn test_const_type() {
        let mut infer = TypeInference::new();
        // Integer literals are now typed as Int (not Scalar)
        // This enables proper type promotion: Int + Rational → Rational
        let expr = Expression::Const("42".to_string());
        let ty = infer.infer_and_solve(&expr, None).unwrap();
        assert!(
            matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
            "Integer literal should be Int, got {:?}",
            ty
        );

        // Real literals remain Scalar
        let expr_real = Expression::Const("3.14".to_string());
        let ty_real = infer.infer_and_solve(&expr_real, None).unwrap();
        assert_eq!(ty_real, Type::scalar());
    }

    #[test]
    fn test_addition_type() {
        let mut infer = TypeInference::new();
        let context = create_test_context();

        // 1 + 2 (integer literals) → Int
        let expr = Expression::operation(
            "plus",
            vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        );

        let ty = infer.infer_and_solve(&expr, Some(&context)).unwrap();
        // Int + Int → Int (same type, no promotion needed)
        assert!(
            matches!(&ty, Type::Data { constructor, .. } if constructor == "Int" || constructor == "Scalar"),
            "Int + Int should be Int or Scalar, got {:?}",
            ty
        );
    }

    #[test]
    fn test_variable_inference() {
        let mut infer = TypeInference::new();
        let context = create_test_context();

        // x + 1 (where x is unknown)
        let expr = Expression::operation(
            "plus",
            vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        );

        let ty = infer.infer_and_solve(&expr, Some(&context)).unwrap();
        // With proper polymorphism, x is unbound so remains a type variable
        // The operation plus : T → T → T preserves polymorphism
        // Accept Scalar, Int (integer literals now type as Int), or Var
        assert!(
            matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
                || matches!(&ty, Type::Var(_)),
            "Expected Scalar, Int, or Var, got {:?}",
            ty
        );
    }

    #[test]
    fn test_division_type() {
        let mut infer = TypeInference::new();
        let context = create_test_context();

        // x / 2
        let expr = Expression::operation(
            "divide",
            vec![
                Expression::Object("x".to_string()),
                Expression::Const("2".to_string()),
            ],
        );

        let ty = infer.infer_and_solve(&expr, Some(&context)).unwrap();
        // With proper polymorphism, x is unbound so remains a type variable
        // The operation divide : T → T → T preserves polymorphism
        println!("Inferred type: {}", ty);
        // Accept Scalar, Int (integer literals now type as Int), or Var
        assert!(
            matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
                || matches!(&ty, Type::Var(_)),
            "Expected Scalar, Int, or Var, got {:?}",
            ty
        );
    }

    #[test]
    fn test_without_context_returns_type_var() {
        let mut infer = TypeInference::new();

        // Without context_builder, operations return fresh variables
        let expr = Expression::operation("unknown_op", vec![Expression::Const("1".to_string())]);

        let ty = infer.infer_and_solve(&expr, None).unwrap();
        // Should return a type variable (not an error)
        assert!(matches!(ty, Type::Var(_)));
    }

    // ===== Generic Data Constructor Tests (ADR-021) =====

    #[test]
    fn test_data_constructor_simple() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};

        // Create registry with Bool type
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

        let mut infer = TypeInference::with_data_registry(registry);

        // Infer True constructor
        let true_expr = Expression::Operation {
            name: "True".to_string(),
            args: vec![],
            span: None,
        };

        let ty = infer.infer(&true_expr, None).unwrap();

        // Should be Data { type_name: "Bool", constructor: "True", args: [] }
        match ty {
            Type::Data {
                type_name,
                constructor,
                args,
            } => {
                assert_eq!(type_name, "Bool");
                assert_eq!(constructor, "True");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Data type, got {:?}", ty),
        }
    }

    #[test]
    fn test_data_constructor_with_nat_params() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        // Create registry with Type data type
        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Type".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "Scalar".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Vector".to_string(),
                        fields: vec![DataField {
                            name: Some("n".to_string()),
                            type_expr: TypeExpr::Named("Nat".to_string()),
                        }],
                    },
                ],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        // Infer Vector(3) constructor
        let vector_expr = Expression::Operation {
            name: "Vector".to_string(),
            args: vec![Expression::Const("3".to_string())],
            span: None,
        };

        let ty = infer.infer(&vector_expr, None).unwrap();

        // Should be Data { type_name: "Type", constructor: "Vector", args: [Nat] }
        match ty {
            Type::Data {
                type_name,
                constructor,
                args,
            } => {
                assert_eq!(type_name, "Type");
                assert_eq!(constructor, "Vector");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::NatValue(3));
            }
            _ => panic!("Expected Data type, got {:?}", ty),
        }
    }

    #[test]
    fn test_data_constructor_parametric() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr, TypeParam};

        // Create registry with Option type
        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Option".to_string(),
                type_params: vec![TypeParam {
                    name: "T".to_string(),
                    kind: None,
                }],
                variants: vec![
                    DataVariant {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Some".to_string(),
                        fields: vec![DataField {
                            name: None,
                            type_expr: TypeExpr::Named("T".to_string()),
                        }],
                    },
                ],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        // Infer None constructor
        let none_expr = Expression::Operation {
            name: "None".to_string(),
            args: vec![],
            span: None,
        };

        let ty = infer.infer(&none_expr, None).unwrap();

        match ty {
            Type::Data {
                type_name,
                constructor,
                ..
            } => {
                assert_eq!(type_name, "Option");
                assert_eq!(constructor, "None");
            }
            _ => panic!("Expected Data type, got {:?}", ty),
        }
    }

    #[test]
    fn test_data_constructor_error_wrong_arity() {
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Type".to_string(),
                type_params: vec![],
                variants: vec![DataVariant {
                    name: "Vector".to_string(),
                    fields: vec![DataField {
                        name: Some("n".to_string()),
                        type_expr: TypeExpr::Named("Nat".to_string()),
                    }],
                }],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        // Try to call Vector with wrong number of args
        let bad_expr = Expression::Operation {
            name: "Vector".to_string(),
            args: vec![], // Should have 1 arg!
            span: None,
        };

        let result = infer.infer(&bad_expr, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 1 arguments, got 0"));
    }

    #[test]
    fn test_data_constructor_unknown_variant() {
        let mut infer = TypeInference::new(); // Empty registry

        let expr = Expression::Operation {
            name: "UnknownConstructor".to_string(),
            args: vec![],
            span: None,
        };

        // Should fall back to context_builder (returns type var)
        let ty = infer.infer(&expr, None).unwrap();
        assert!(matches!(ty, Type::Var(_)));
    }

    // ===== Pattern Matching Type Inference Tests =====

    #[test]
    fn test_match_simple_bool() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};

        // Create registry with Bool type
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

        let mut infer = TypeInference::with_data_registry(registry);

        // Bind a boolean variable
        infer.bind(
            "x".to_string(),
            Type::Data {
                type_name: "Bool".to_string(),
                constructor: "Bool".to_string(),
                args: vec![],
            },
        );

        // match x { True => 1 | False => 0 }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("x".to_string())),
            cases: vec![
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
            ],
            span: None,
        };

        let ty = infer.infer(&match_expr, None).unwrap();
        // Both branches return integer literals (now typed as Int)
        assert!(matches!(
            ty,
            Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int"
        ));
    }

    #[test]
    fn test_match_with_variable_binding() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        // Create registry with Option type
        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Option".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Some".to_string(),
                        fields: vec![DataField {
                            name: Some("value".to_string()),
                            type_expr: TypeExpr::Named("ℝ".to_string()),
                        }],
                    },
                ],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        // Bind an Option variable
        infer.bind(
            "opt".to_string(),
            Type::Data {
                type_name: "Option".to_string(),
                constructor: "Option".to_string(),
                args: vec![],
            },
        );

        // match opt { None => 0 | Some(x) => x }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("opt".to_string())),
            cases: vec![
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
            ],
            span: None,
        };

        let ty = infer.infer(&match_expr, None).unwrap();
        // First branch returns Int (integer literal), second returns Scalar (from Option field)
        // Common supertype is Scalar (Int → Scalar promotion)
        assert!(matches!(
            ty,
            Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int"
        ));
    }

    #[test]
    fn test_match_with_wildcard() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};

        // Create registry with Status type
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
                ],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        infer.bind(
            "status".to_string(),
            Type::Data {
                type_name: "Status".to_string(),
                constructor: "Status".to_string(),
                args: vec![],
            },
        );

        // match status { Running => 1 | _ => 0 }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("status".to_string())),
            cases: vec![
                MatchCase::new(
                    Pattern::Constructor {
                        name: "Running".to_string(),
                        args: vec![],
                    },
                    Expression::Const("1".to_string()),
                ),
                MatchCase::new(Pattern::Wildcard, Expression::Const("0".to_string())),
            ],
            span: None,
        };

        let ty = infer.infer(&match_expr, None).unwrap();
        // Both branches return integer literals (now typed as Int)
        assert!(matches!(
            ty,
            Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int"
        ));
    }

    #[test]
    fn test_match_nested_patterns() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        // Create registry with Result and Option types
        let mut registry = DataTypeRegistry::new();

        registry
            .register(DataDef {
                name: "Option".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Some".to_string(),
                        fields: vec![DataField {
                            name: Some("value".to_string()),
                            type_expr: TypeExpr::Named("ℝ".to_string()),
                        }],
                    },
                ],
            })
            .unwrap();

        registry
            .register(DataDef {
                name: "Result".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "Ok".to_string(),
                        fields: vec![DataField {
                            name: Some("value".to_string()),
                            type_expr: TypeExpr::Named("Option".to_string()),
                        }],
                    },
                    DataVariant {
                        name: "Err".to_string(),
                        fields: vec![],
                    },
                ],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        infer.bind(
            "result".to_string(),
            Type::Data {
                type_name: "Result".to_string(),
                constructor: "Result".to_string(),
                args: vec![],
            },
        );

        // match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => 0 }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("result".to_string())),
            cases: vec![
                MatchCase::new(
                    Pattern::Constructor {
                        name: "Ok".to_string(),
                        args: vec![Pattern::Constructor {
                            name: "Some".to_string(),
                            args: vec![Pattern::Variable("x".to_string())],
                        }],
                    },
                    Expression::Object("x".to_string()),
                ),
                MatchCase::new(
                    Pattern::Constructor {
                        name: "Ok".to_string(),
                        args: vec![Pattern::Constructor {
                            name: "None".to_string(),
                            args: vec![],
                        }],
                    },
                    Expression::Const("0".to_string()),
                ),
                MatchCase::new(
                    Pattern::Constructor {
                        name: "Err".to_string(),
                        args: vec![],
                    },
                    Expression::Const("0".to_string()),
                ),
            ],
            span: None,
        };

        let ty = infer.infer(&match_expr, None).unwrap();
        assert!(matches!(
            ty,
            Type::Data { constructor, .. } if constructor == "Scalar"
        ));
    }

    #[test]
    fn test_match_error_wrong_constructor() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};

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

        registry
            .register(DataDef {
                name: "Option".to_string(),
                type_params: vec![],
                variants: vec![DataVariant {
                    name: "None".to_string(),
                    fields: vec![],
                }],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        infer.bind(
            "x".to_string(),
            Type::Data {
                type_name: "Bool".to_string(),
                constructor: "Bool".to_string(),
                args: vec![],
            },
        );

        // match x { None => 0 }  // ERROR: None is not a Bool constructor!
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("x".to_string())),
            cases: vec![MatchCase::new(
                Pattern::Constructor {
                    name: "None".to_string(),
                    args: vec![],
                },
                Expression::Const("0".to_string()),
            )],
            span: None,
        };

        let result = infer.infer(&match_expr, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pattern mismatch"));
    }

    #[test]
    fn test_match_error_wrong_arity() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Option".to_string(),
                type_params: vec![],
                variants: vec![DataVariant {
                    name: "Some".to_string(),
                    fields: vec![DataField {
                        name: Some("value".to_string()),
                        type_expr: TypeExpr::Named("ℝ".to_string()),
                    }],
                }],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        infer.bind(
            "opt".to_string(),
            Type::Data {
                type_name: "Option".to_string(),
                constructor: "Option".to_string(),
                args: vec![],
            },
        );

        // match opt { Some(x, y) => 0 }  // ERROR: Some takes 1 arg, not 2!
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("opt".to_string())),
            cases: vec![MatchCase::new(
                Pattern::Constructor {
                    name: "Some".to_string(),
                    args: vec![
                        Pattern::Variable("x".to_string()),
                        Pattern::Variable("y".to_string()),
                    ],
                },
                Expression::Const("0".to_string()),
            )],
            span: None,
        };

        let result = infer.infer(&match_expr, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 1 arguments, got 2"));
    }

    #[test]
    fn test_match_error_no_cases() {
        use crate::ast::Expression;

        let mut infer = TypeInference::new();

        // match x { }  // ERROR: No cases!
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("x".to_string())),
            cases: vec![],
            span: None,
        };

        let result = infer.infer(&match_expr, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have at least one case"));
    }

    #[test]
    fn test_match_with_constant_pattern() {
        use crate::ast::{Expression, MatchCase, Pattern};

        let mut infer = TypeInference::new();

        infer.bind("n".to_string(), Type::scalar());

        // match n { 0 => "zero" | 1 => "one" | _ => "other" }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("n".to_string())),
            cases: vec![
                MatchCase::new(
                    Pattern::Constant("0".to_string()),
                    Expression::Object("zero".to_string()),
                ),
                MatchCase::new(
                    Pattern::Constant("1".to_string()),
                    Expression::Object("one".to_string()),
                ),
                MatchCase::new(Pattern::Wildcard, Expression::Object("other".to_string())),
            ],
            span: None,
        };

        // Should infer successfully (all branches return type variables that unify)
        let ty = infer.infer(&match_expr, None).unwrap();
        assert!(matches!(ty, Type::Var(_)));
    }

    // ===== Constructor Unification Tests (Fix for Bool pattern matching) =====

    #[test]
    fn test_unify_same_enum_constructors() {
        // True and False should unify (both are Bool)
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataVariant};

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

        let mut infer = TypeInference::with_data_registry(registry);

        // Infer True and False
        let true_ty = infer
            .infer(
                &Expression::Operation {
                    name: "True".to_string(),
                    args: vec![],
                    span: None,
                },
                None,
            )
            .unwrap();

        let false_ty = infer
            .infer(
                &Expression::Operation {
                    name: "False".to_string(),
                    args: vec![],
                    span: None,
                },
                None,
            )
            .unwrap();

        // They should both be Data{Bool, ..., []}
        assert!(matches!(true_ty, Type::Data { .. }));
        assert!(matches!(false_ty, Type::Data { .. }));

        // Now unify them - should succeed!
        infer.add_constraint(true_ty, false_ty);
        let result = infer.solve();
        assert!(
            result.is_ok(),
            "True and False should unify (both are Bool)"
        );
    }

    #[test]
    fn test_unify_different_matrix_dimensions() {
        // Matrix(2,3) and Matrix(3,2) should NOT unify
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Type".to_string(),
                type_params: vec![],
                variants: vec![DataVariant {
                    name: "Matrix".to_string(),
                    fields: vec![
                        DataField {
                            name: Some("m".to_string()),
                            type_expr: TypeExpr::Named("Nat".to_string()),
                        },
                        DataField {
                            name: Some("n".to_string()),
                            type_expr: TypeExpr::Named("Nat".to_string()),
                        },
                    ],
                }],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        // Matrix(2, 3)
        let mat23_ty = infer
            .infer(
                &Expression::Operation {
                    name: "Matrix".to_string(),
                    args: vec![
                        Expression::Const("2".to_string()),
                        Expression::Const("3".to_string()),
                    ],
                    span: None,
                },
                None,
            )
            .unwrap();

        // Matrix(3, 2)
        let mat32_ty = infer
            .infer(
                &Expression::Operation {
                    name: "Matrix".to_string(),
                    args: vec![
                        Expression::Const("3".to_string()),
                        Expression::Const("2".to_string()),
                    ],
                    span: None,
                },
                None,
            )
            .unwrap();

        // Try to unify - should FAIL (different dimensions)
        infer.add_constraint(mat23_ty, mat32_ty);
        let result = infer.solve();
        assert!(
            result.is_err(),
            "Matrix(2,3) and Matrix(3,2) should NOT unify"
        );
    }

    #[test]
    fn test_match_variable_binding_scope() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Option".to_string(),
                type_params: vec![],
                variants: vec![
                    DataVariant {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                    DataVariant {
                        name: "Some".to_string(),
                        fields: vec![DataField {
                            name: Some("value".to_string()),
                            type_expr: TypeExpr::Named("ℝ".to_string()),
                        }],
                    },
                ],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        infer.bind(
            "opt".to_string(),
            Type::Data {
                type_name: "Option".to_string(),
                constructor: "Option".to_string(),
                args: vec![],
            },
        );

        // match opt { Some(x) => x | None => 0 }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("opt".to_string())),
            cases: vec![
                MatchCase::new(
                    Pattern::Constructor {
                        name: "Some".to_string(),
                        args: vec![Pattern::Variable("x".to_string())],
                    },
                    Expression::Object("x".to_string()),
                ),
                MatchCase::new(
                    Pattern::Constructor {
                        name: "None".to_string(),
                        args: vec![],
                    },
                    Expression::Const("0".to_string()),
                ),
            ],
            span: None,
        };

        // Infer the match
        let _ty = infer.infer(&match_expr, None).unwrap();

        // After match, 'x' should NOT be in scope (bindings are local to branches)
        assert!(infer.context().get("x").is_none());
    }

    #[test]
    fn test_match_multiple_variables() {
        use crate::ast::{Expression, MatchCase, Pattern};
        use crate::data_registry::DataTypeRegistry;
        use crate::kleis_ast::{DataDef, DataField, DataVariant, TypeExpr};

        let mut registry = DataTypeRegistry::new();
        registry
            .register(DataDef {
                name: "Pair".to_string(),
                type_params: vec![],
                variants: vec![DataVariant {
                    name: "Pair".to_string(),
                    fields: vec![
                        DataField {
                            name: Some("first".to_string()),
                            type_expr: TypeExpr::Named("ℝ".to_string()),
                        },
                        DataField {
                            name: Some("second".to_string()),
                            type_expr: TypeExpr::Named("ℝ".to_string()),
                        },
                    ],
                }],
            })
            .unwrap();

        let mut infer = TypeInference::with_data_registry(registry);

        infer.bind(
            "pair".to_string(),
            Type::Data {
                type_name: "Pair".to_string(),
                constructor: "Pair".to_string(),
                args: vec![],
            },
        );

        // match pair { Pair(a, b) => plus(a, b) }
        let match_expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("pair".to_string())),
            cases: vec![MatchCase::new(
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
                    span: None,
                },
            )],
            span: None,
        };

        // Should infer successfully
        let ty = infer.infer(&match_expr, None).unwrap();
        // After operator overloading changes, plus(a, b) with non-Complex args returns Scalar
        // (Previously returned type variable when no context_builder)
        assert!(matches!(ty, Type::Data { constructor, .. } if constructor == "Scalar"));
    }
}
