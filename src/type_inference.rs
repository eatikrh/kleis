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
/// TODO(ADR-021): This enum should be defined in Kleis, not hardcoded in Rust!
///
/// Future (Phase 2.5): Replace with dynamic type system
/// ```kleis
/// // In stdlib/types.kleis:
/// data Type =
///   | Scalar
///   | Vector(n: Nat)
///   | Matrix(m: Nat, n: Nat)
///   | Complex
///   | Var(Nat)
///   | Function(Type, Type)
///   | ForAll(Nat, Type)
///   | UserDefined(String, List(Type))  // ← Users can extend!
/// ```
///
/// Benefits:
/// - Users can add new types without recompiling
/// - Type system becomes self-hosting
/// - Enables true meta-circularity (Kleis types defined in Kleis)
/// - Pattern matching on types moves to Kleis code
///
/// See ADR-021 for full proposal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Scalar (ℝ)
    Scalar,

    /// Vector of dimension n
    Vector(usize),

    /// Matrix of dimensions m×n
    Matrix(usize, usize),

    /// Type variable (for inference)
    /// α, β, γ in type theory
    Var(TypeVar),

    /// Function type: input → output
    Function(Box<Type>, Box<Type>),

    /// Polymorphic type: ∀α. T
    /// For now, we'll represent this after generalization
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
            Type::Function(t1, t2) => {
                Type::Function(Box::new(self.apply(t1)), Box::new(self.apply(t2)))
            }
            Type::ForAll(v, t) => Type::ForAll(v.clone(), Box::new(self.apply(t))),
            _ => ty.clone(),
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
}

impl TypeInference {
    pub fn new() -> Self {
        TypeInference {
            context: TypeContext::new(),
            constraints: Vec::new(),
        }
    }

    /// Add a constraint
    fn add_constraint(&mut self, left: Type, right: Type) {
        self.constraints.push(Constraint { left, right });
    }

    /// Bind a variable to a type
    pub fn bind(&mut self, name: String, ty: Type) {
        self.context.bind(name, ty);
    }

    /// Get the context
    pub fn context(&self) -> &TypeContext {
        &self.context
    }

    /// Infer type of an expression
    pub fn infer(
        &mut self,
        expr: &Expression,
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        match expr {
            // Constants are scalars
            Expression::Const(_) => Ok(Type::Scalar),

            // Variables: look up in context or create fresh var
            Expression::Object(name) => {
                if let Some(ty) = self.context.get(name) {
                    Ok(ty.clone())
                } else {
                    // Unknown variable: create fresh type variable
                    let ty = self.context.fresh_var();
                    self.context.bind(name.clone(), ty.clone());
                    Ok(ty)
                }
            }

            // Placeholders: unknown type (fresh variable)
            Expression::Placeholder { .. } => Ok(self.context.fresh_var()),

            // Operations: infer based on operation type
            Expression::Operation { name, args } => {
                self.infer_operation(name, args, context_builder)
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
        match name {
            // Matrix constructors are LITERALS (data constructors, not operations)
            // Format: Matrix(rows, cols, ...elements)
            //
            // TODO(ADR-021): Replace with generic data constructor handling
            // Once we have `data Type = ... | Matrix(Nat, Nat)`, this special case
            // can be replaced with generic logic that works for ALL data constructors.
            "Matrix" | "PMatrix" | "VMatrix" | "BMatrix" => {
                self.infer_matrix_constructor(name, args, context_builder)
            }

            // EVERYTHING ELSE: Delegate to context_builder (ADR-016!)
            _ => {
                // Infer argument types first
                let arg_types: Vec<Type> = args
                    .iter()
                    .map(|arg| self.infer(arg, context_builder))
                    .collect::<Result<Vec<_>, _>>()?;

                // If context_builder is available, query the registry
                if let Some(builder) = context_builder {
                    builder.infer_operation_type(name, &arg_types)
                } else {
                    // No context builder - return fresh variable (for backwards compatibility)
                    // This allows tests without context_builder to still run
                    Ok(self.context.fresh_var())
                }
            }
        }
    }

    /// Infer type of matrix constructor (Matrix, PMatrix, VMatrix, BMatrix)
    /// These are special because they're LITERALS that construct values,
    /// not operations that transform values.
    ///
    /// TODO(ADR-021): This should be generic data constructor inference!
    ///
    /// With `data` keyword:
    /// ```kleis
    /// data Type = ... | Matrix(m: Nat, n: Nat)
    ///
    /// // Generic inference for ALL data constructors:
    /// fn infer_data_constructor(name: &str, args: &[Expr]) -> Type {
    ///     let variant = lookup_variant(name)?;
    ///     validate_args(variant, args)?;
    ///     construct_type(variant, args)
    /// }
    /// ```
    ///
    /// This function would disappear - replaced by generic data constructor handling!
    ///
    /// REFACTORING NOTES:
    /// - Separated dimension extraction (lines 254-263) - could be generic
    /// - Element type inference (lines 265-278) - already generic
    /// - Type construction (line 280) - would use generic constructor registry
    fn infer_matrix_constructor(
        &mut self,
        _name: &str,
        args: &[Expression],
        context_builder: Option<&crate::type_context::TypeContextBuilder>,
    ) -> Result<Type, String> {
        // Step 1: Extract constructor parameters (dimensions)
        // TODO(ADR-021): Generic data constructors would get params from variant definition
        let (rows, cols) = self.extract_matrix_dimensions(args)?;

        // Step 2: Infer field types (matrix elements)
        // NOTE: This is already generic! Could work for any data constructor.
        self.infer_data_constructor_fields(&args[2..], context_builder, Type::Scalar)?;

        // Step 3: Construct result type
        // TODO(ADR-021): Would lookup variant definition and construct generically
        Ok(Type::Matrix(rows, cols))
    }

    /// Extract matrix dimensions from first two arguments
    /// TODO(ADR-021): Generic version would extract params based on variant definition
    fn extract_matrix_dimensions(&self, args: &[Expression]) -> Result<(usize, usize), String> {
        if args.len() < 2 {
            return Err(
                "Matrix constructor requires at least 2 arguments (rows, cols)".to_string(),
            );
        }

        let rows = match &args[0] {
            Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
            _ => 2, // Default if not a constant
        };
        let cols = match &args[1] {
            Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
            _ => 2, // Default if not a constant
        };

        Ok((rows, cols))
    }

    /// Infer types of data constructor fields
    /// This is GENERIC! Works for any data constructor with typed fields.
    ///
    /// TODO(ADR-021): This logic is already generic and would work for all data constructors!
    /// Example: Cons(head: T, tail: List(T)) would validate head and tail fields.
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
        // Same concrete types unify trivially
        (Type::Scalar, Type::Scalar) => Ok(Substitution::empty()),
        (Type::Vector(n1), Type::Vector(n2)) if n1 == n2 => Ok(Substitution::empty()),
        (Type::Matrix(m1, n1), Type::Matrix(m2, n2)) if m1 == m2 && n1 == n2 => {
            Ok(Substitution::empty())
        }

        // Type variable unifies with anything (if not occurs)
        (Type::Var(v), t) | (t, Type::Var(v)) => {
            if occurs(v, t) {
                Err(format!("Occurs check failed: {:?} occurs in {:?}", v, t))
            } else {
                Ok(Substitution::singleton(v.clone(), t.clone()))
            }
        }

        // Function types: unify components
        (Type::Function(a1, b1), Type::Function(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&s1.apply(b1), &s1.apply(b2))?;
            Ok(s1.compose(&s2))
        }

        // Otherwise: cannot unify
        _ => Err(format!("Cannot unify {:?} with {:?}", t1, t2)),
    }
}

/// Occurs check: does variable v occur in type t?
fn occurs(v: &TypeVar, t: &Type) -> bool {
    match t {
        Type::Var(v2) => v == v2,
        Type::Function(t1, t2) => occurs(v, t1) || occurs(v, t2),
        Type::ForAll(_, t) => occurs(v, t),
        _ => false,
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Scalar => write!(f, "ℝ"),
            Type::Vector(n) => write!(f, "Vector(ℝ{})", n),
            Type::Matrix(m, n) => write!(f, "Matrix({}, {})", m, n),
            Type::Var(TypeVar(n)) => write!(f, "α{}", n),
            Type::Function(t1, t2) => write!(f, "{} → {}", t1, t2),
            Type::ForAll(TypeVar(n), t) => write!(f, "∀α{}. {}", n, t),
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
        let expr = Expression::Const("42".to_string());
        let ty = infer.infer_and_solve(&expr, None).unwrap();
        assert_eq!(ty, Type::Scalar);
    }

    #[test]
    fn test_addition_type() {
        let mut infer = TypeInference::new();
        let context = create_test_context();

        // 1 + 2
        let expr = Expression::operation(
            "plus",
            vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        );

        let ty = infer.infer_and_solve(&expr, Some(&context)).unwrap();
        assert_eq!(ty, Type::Scalar);
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
        // Should infer x : Scalar
        assert_eq!(ty, Type::Scalar);
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
        // Should infer x : Scalar, result: Scalar
        println!("Inferred type: {}", ty);
        assert_eq!(ty, Type::Scalar);
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
}
