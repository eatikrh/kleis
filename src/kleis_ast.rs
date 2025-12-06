///! Extended AST for Kleis - Structure Definitions and Type Expressions
///!
///! This extends the basic Expression AST with support for:
///! - Structure definitions
///! - Operation declarations
///! - Type expressions
///! - Top-level program items
///!
///! Used for parsing complete Kleis programs with user-defined types.
use crate::ast::Expression;

/// Top-level program items
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    /// Structure definition: structure Name { members }
    StructureDef(StructureDef),

    /// Implements block: implements StructureName(Type) { ... }
    ImplementsDef(ImplementsDef),

    /// Operation declaration: operation name : Type (top-level utility)
    OperationDecl(OperationDecl),

    /// Function definition: define name(params) = expr
    FunctionDef(FunctionDef),

    /// Type alias: type Name = Type
    TypeAlias(TypeAlias),
}

/// Structure definition
#[derive(Debug, Clone, PartialEq)]
pub struct StructureDef {
    pub name: String,
    pub members: Vec<StructureMember>,
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
pub struct TypeAlias {
    pub name: String,
    pub type_expr: TypeExpr,
}

/// Implements definition: implements StructureName(Type) { members }
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementsDef {
    pub structure_name: String,
    pub type_arg: TypeExpr,
    pub members: Vec<ImplMember>,
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
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
