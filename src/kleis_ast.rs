//! Extended AST for Kleis - Structure Definitions and Type Expressions
//!
//! This extends the basic Expression AST with support for:
//! - Structure definitions
//! - Operation declarations
//! - Type expressions
//! - Top-level program items
//!
//! Used for parsing complete Kleis programs with user-defined types.
use crate::ast::Expression;

/// Top-level program items
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    /// Import statement: import "path/to/file.kleis"
    Import(String),

    /// Structure definition: structure Name { members }
    StructureDef(StructureDef),

    /// Implements block: implements StructureName(Type) { ... }
    ImplementsDef(ImplementsDef),

    /// Data type definition: data Name = Variant1 | Variant2 | ...
    DataDef(DataDef),

    /// Operation declaration: operation name : Type (top-level utility)
    OperationDecl(OperationDecl),

    /// Function definition: define name(params) = expr
    FunctionDef(FunctionDef),

    /// Type alias: type Name = Type
    TypeAlias(TypeAlias),
}

/// Structure definition
/// Example: structure Monoid(M) extends Semigroup(M) { ... }
/// Example: structure VectorSpace(V) over Field(F) { ... }
#[derive(Debug, Clone, PartialEq)]
pub struct StructureDef {
    pub name: String,
    pub type_params: Vec<TypeParam>, // e.g., (m: Nat, n: Nat, T)
    pub members: Vec<StructureMember>,
    /// Optional parent structure (inheritance)
    /// Example: extends Semigroup(M)
    pub extends_clause: Option<TypeExpr>,
    /// Optional over clause (for structures parameterized over fields)
    /// Example: over Field(F) for vector spaces
    pub over_clause: Option<TypeExpr>,
}

/// Type parameter for structures
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub kind: Option<String>, // e.g., "Nat" for natural number parameters
}

/// Structure member (field, operation, or axiom)
#[derive(Debug, Clone, PartialEq)]
pub enum StructureMember {
    /// Field: fieldName : Type
    Field { name: String, type_expr: TypeExpr },

    /// Operation: operation name : TypeSignature
    Operation {
        name: String,
        type_signature: TypeExpr,
    },

    /// Axiom: axiom name : Proposition
    Axiom {
        name: String,
        proposition: Expression,
    },

    /// Nested structure: structure name : StructureType { members }
    /// Example: structure additive : AbelianGroup(R) { ... }
    NestedStructure {
        name: String,
        structure_type: TypeExpr,      // e.g., AbelianGroup(R)
        members: Vec<StructureMember>, // Recursive!
    },

    /// Function definition (v0.6): derived operations with default implementations
    /// Example: define (-)(x, y) = x + negate(y)
    FunctionDef(FunctionDef),
}

/// Operation declaration
#[derive(Debug, Clone, PartialEq)]
pub struct OperationDecl {
    pub name: String,
    pub type_signature: TypeExpr,
}

/// Function definition
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub type_annotation: Option<TypeExpr>,
    pub body: Expression,
}

/// Type alias
#[derive(Debug, Clone, PartialEq)]
/// Type alias: type Name = Type or type Name(params) = Type
///
/// Grammar v0.91: Parameterized type aliases enable user-defined generic types.
/// Examples:
///   type RealVector = Vector(ℝ)                              -- simple
///   type ComplexMatrix(m, n) = (Matrix(m,n,ℝ), Matrix(m,n,ℝ)) -- parameterized
pub struct TypeAlias {
    pub name: String,
    /// Optional type parameters (v0.91)
    /// Empty for simple aliases like `type RealVector = Vector(ℝ)`
    /// Non-empty for parameterized aliases like `type ComplexMatrix(m, n) = ...`
    pub params: Vec<TypeAliasParam>,
    pub type_expr: TypeExpr,
}

/// Type alias parameter: identifier with optional kind annotation
/// Examples: m, n, T, m: Nat, T: Type
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasParam {
    pub name: String,
    pub kind: Option<String>, // "Nat", "Type", etc.
}

/// Data type definition: data Name(T, U) = Variant1 | Variant2(T) | ...
///
/// Algebraic data types (ADR-021) for user-defined types.
/// Examples:
///   data Bool = True | False
///   data Option(T) = None | Some(T)
///   data Type = Scalar | Matrix(m: Nat, n: Nat)
#[derive(Debug, Clone, PartialEq)]
pub struct DataDef {
    /// Name of the data type (e.g., "Bool", "Option", "Type")
    pub name: String,

    /// Type parameters (e.g., T in Option(T), m and n in Matrix(m,n))
    pub type_params: Vec<TypeParam>,

    /// Data constructors/variants (e.g., True | False, None | Some(T))
    pub variants: Vec<DataVariant>,
}

/// Data constructor variant
///
/// A single variant of an algebraic data type.
/// Examples:
///   True (no fields)
///   Some(T) (one field)
///   Matrix(m: Nat, n: Nat) (two named fields)
#[derive(Debug, Clone, PartialEq)]
pub struct DataVariant {
    /// Constructor name (e.g., "True", "Some", "Matrix")
    pub name: String,

    /// Constructor fields/arguments
    pub fields: Vec<DataField>,
}

/// Field in a data constructor
///
/// Can be named or positional.
/// Examples:
///   T (positional field of type T)
///   value: T (named field)
///   m: Nat (named field with concrete type)
#[derive(Debug, Clone, PartialEq)]
pub struct DataField {
    /// Optional field name (None for positional fields)
    pub name: Option<String>,

    /// Type of the field
    pub type_expr: TypeExpr,
}

/// Implements definition: implements StructureName(Type, ...) over Field(F) where Constraint { members }
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementsDef {
    pub structure_name: String,
    pub type_args: Vec<TypeExpr>, // Changed from single type_arg to multiple
    pub members: Vec<ImplMember>,
    /// Optional over clause (for vector spaces over fields)
    /// Example: over Field(ℝ)
    pub over_clause: Option<TypeExpr>,
    /// Optional where clause with structure constraints
    /// Example: where Semiring(T), Ord(T)
    pub where_clause: Option<Vec<WhereConstraint>>,
}

/// A constraint in a where clause
#[derive(Debug, Clone, PartialEq)]
pub struct WhereConstraint {
    pub structure_name: String,
    pub type_args: Vec<TypeExpr>,
}

/// Implementation member (element or operation)
#[derive(Debug, Clone, PartialEq)]
pub enum ImplMember {
    /// Element binding: element zero = 0
    Element { name: String, value: Expression },

    /// Operation implementation: operation abs = builtin_abs
    Operation {
        name: String,
        implementation: Implementation,
    },
}

/// Operation implementation
#[derive(Debug, Clone, PartialEq)]
pub enum Implementation {
    /// Builtin function name: builtin_abs
    Builtin(String),

    /// Inline definition: operation abs(x) = x^2
    Inline {
        params: Vec<String>,
        body: Expression,
    },
}

/// Type expression
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// Named type: ℝ, Money, Vector
    Named(String),

    /// Parametric type: Vector(3), Set(ℤ), Matrix(m, n)
    Parametric(String, Vec<TypeExpr>),

    /// Function type: ℝ → ℝ, Vector(n) → Scalar
    Function(Box<TypeExpr>, Box<TypeExpr>),

    /// Product type: A × B (for multi-argument functions)
    Product(Vec<TypeExpr>),

    /// Polymorphic type variable: T, α, n
    Var(String),

    /// Quantified type: ∀(n : ℕ). Vector(n) → ℝ
    /// or ∀(m n p : ℕ, T). Matrix(m,n,T) × Matrix(n,p,T) → Matrix(m,p,T)
    ForAll {
        /// Quantified variables with their kinds/types
        /// e.g., [("n", Named("ℕ")), ("T", Named("Type"))]
        vars: Vec<(String, TypeExpr)>,
        /// The body type
        body: Box<TypeExpr>,
    },
}

impl TypeExpr {
    /// Create a named type
    pub fn named(name: impl Into<String>) -> Self {
        TypeExpr::Named(name.into())
    }

    /// Create a parametric type
    pub fn parametric(name: impl Into<String>, params: Vec<TypeExpr>) -> Self {
        TypeExpr::Parametric(name.into(), params)
    }

    /// Create a function type
    pub fn function(from: TypeExpr, to: TypeExpr) -> Self {
        TypeExpr::Function(Box::new(from), Box::new(to))
    }

    /// Create a product type
    pub fn product(types: Vec<TypeExpr>) -> Self {
        TypeExpr::Product(types)
    }
}

impl std::fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpr::Named(name) => write!(f, "{}", name),
            TypeExpr::Parametric(name, params) => {
                let params_str: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "{}({})", name, params_str.join(", "))
            }
            TypeExpr::Function(from, to) => write!(f, "{} → {}", from, to),
            TypeExpr::Product(types) => {
                let types_str: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "{}", types_str.join(" × "))
            }
            TypeExpr::Var(name) => write!(f, "{}", name),
            TypeExpr::ForAll { vars, body } => {
                let vars_str: Vec<String> = vars
                    .iter()
                    .map(|(name, ty)| format!("{} : {}", name, ty))
                    .collect();
                write!(f, "∀({}). {}", vars_str.join(", "), body)
            }
        }
    }
}

/// Complete program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

impl Program {
    pub fn new() -> Self {
        Program { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: TopLevel) {
        self.items.push(item);
    }

    /// Get all structure definitions
    pub fn structures(&self) -> Vec<&StructureDef> {
        self.items
            .iter()
            .filter_map(|item| {
                if let TopLevel::StructureDef(s) = item {
                    Some(s)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all operation declarations
    pub fn operations(&self) -> Vec<&OperationDecl> {
        self.items
            .iter()
            .filter_map(|item| {
                if let TopLevel::OperationDecl(op) = item {
                    Some(op)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all function definitions
    pub fn functions(&self) -> Vec<&FunctionDef> {
        self.items
            .iter()
            .filter_map(|item| {
                if let TopLevel::FunctionDef(f) = item {
                    Some(f)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all implements blocks
    pub fn implements(&self) -> Vec<&ImplementsDef> {
        self.items
            .iter()
            .filter_map(|item| {
                if let TopLevel::ImplementsDef(impl_def) = item {
                    Some(impl_def)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all data type definitions
    pub fn data_types(&self) -> Vec<&DataDef> {
        self.items
            .iter()
            .filter_map(|item| {
                if let TopLevel::DataDef(data_def) = item {
                    Some(data_def)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn type_aliases(&self) -> Vec<&TypeAlias> {
        self.items
            .iter()
            .filter_map(|item| {
                if let TopLevel::TypeAlias(alias) = item {
                    Some(alias)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
