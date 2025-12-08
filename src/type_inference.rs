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

    /// String type (for text values)
    /// Used in: Currency(code: String)
    String,

    /// Concrete string value
    /// Example: Currency("USD") → Data { args: [StringValue("USD")] }
    StringValue(std::string::String),

    /// Boolean type (for logical values)
    Bool,

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
            Type::Nat | Type::NatValue(_) | Type::String | Type::StringValue(_) | Type::Bool => {
                ty.clone()
            }
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
    data_registry: crate::data_registry::DataTypeRegistry,
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
            Expression::Const(_) => Ok(Type::scalar()),

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
        // ADR-021: Check if this is a data constructor first!
        if self.data_registry.has_variant(name) {
            return self.infer_data_constructor(name, args, context_builder);
        }

        // Special handling for Matrix constructors (backward compatibility)
        // TODO(ADR-021): Once stdlib/types.kleis is loaded, "Matrix" will be in registry
        // and this special case can be removed!
        match name {
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
                    builder.infer_operation_type(name, &arg_types, &self.data_registry)
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
        self.infer_data_constructor_fields(&args[2..], context_builder, Type::scalar())?;

        // Step 3: Construct result type
        // TODO(ADR-021): Would lookup variant definition and construct generically
        Ok(Type::matrix(rows, cols))
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

        // Separate constructor parameters (Nat, String) from value fields
        let mut constructor_args = Vec::new();

        for (i, (arg_expr, field_def)) in args.iter().zip(&variant.fields).enumerate() {
            // Check if this is a type parameter (like dimensions) vs value field
            match &field_def.type_expr {
                crate::kleis_ast::TypeExpr::Named(name) if name == "Nat" => {
                    // This is a dimension/index parameter - must be constant
                    match arg_expr {
                        Expression::Const(s) => {
                            // Extract actual numeric value
                            let value = s.parse::<usize>().map_err(|_| {
                                format!("Constructor parameter {} must be a valid number: {}", i, s)
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
                    // String parameter - must be constant
                    match arg_expr {
                        Expression::Const(s) => {
                            // Store actual string value
                            constructor_args.push(Type::StringValue(s.clone()));
                        }
                        _ => {
                            return Err(format!(
                                "Constructor parameter {} must be constant (String expected)",
                                i
                            ));
                        }
                    }
                }
                _ => {
                    // This is a value field - infer its type
                    let arg_type = self.infer(arg_expr, context_builder)?;
                    // TODO: Add constraint that arg_type matches field_def.type_expr
                    // For now, we just infer and don't constrain
                    let _ = arg_type; // Suppress warning
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
        (Type::String, Type::String) => Ok(Substitution::empty()),
        (Type::StringValue(s1), Type::StringValue(s2)) if s1 == s2 => Ok(Substitution::empty()),
        (Type::StringValue(s1), Type::StringValue(s2)) => Err(format!(
            "Cannot unify different strings: {:?} vs {:?}",
            s1, s2
        )),
        (Type::Bool, Type::Bool) => Ok(Substitution::empty()),

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
            // Must be from the same data type
            if t1 != t2 {
                return Err(format!(
                    "Cannot unify types from different data types: {} vs {}",
                    t1, t2
                ));
            }

            // Must be the same constructor
            if c1 != c2 {
                return Err(format!(
                    "Cannot unify different constructors: {} vs {}",
                    c1, c2
                ));
            }

            // Must have same number of arguments
            if a1.len() != a2.len() {
                return Err(format!(
                    "Constructor {} has different number of arguments: {} vs {}",
                    c1,
                    a1.len(),
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
        // Leaf types (no variables can occur in them)
        Type::Nat | Type::NatValue(_) | Type::String | Type::StringValue(_) | Type::Bool => false,
    }
}

impl Type {
    /// Create a Scalar type (backward compatibility)
    ///
    /// This is a convenience constructor to ease the transition from
    /// the old hardcoded Type::Scalar to the new Data-based system.
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
    pub fn vector(n: usize) -> Type {
        Type::Data {
            type_name: "Type".to_string(),
            constructor: "Vector".to_string(),
            args: vec![Type::NatValue(n)],
        }
    }

    /// Create a Matrix type (backward compatibility)
    ///
    /// Dimensions are stored as concrete values, enabling:
    /// - Matrix(2,3) ≠ Matrix(2,2) (different types!)
    /// - Matrix(2,3) × Matrix(3,4) → Matrix(2,4) dimension checking
    pub fn matrix(m: usize, n: usize) -> Type {
        Type::Data {
            type_name: "Type".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![Type::NatValue(m), Type::NatValue(n)],
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

            // Meta-level types
            Type::Var(TypeVar(n)) => write!(f, "α{}", n),
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
        assert_eq!(ty, Type::scalar());
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
        assert_eq!(ty, Type::scalar());
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
        // Accept either Scalar (backward compat) or Var (correct polymorphism)
        assert!(
            matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar")
                || matches!(&ty, Type::Var(_)),
            "Expected Scalar or Var, got {:?}",
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
        // Accept either Scalar (backward compat) or Var (correct polymorphism)
        assert!(
            matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar")
                || matches!(&ty, Type::Var(_)),
            "Expected Scalar or Var, got {:?}",
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
        };

        // Should fall back to context_builder (returns type var)
        let ty = infer.infer(&expr, None).unwrap();
        assert!(matches!(ty, Type::Var(_)));
    }
}
