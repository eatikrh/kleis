//! Abstract Syntax Tree for Kleis mathematical expressions
//!
//! This module defines the core Expression type used throughout the system.
//! Both the parser and renderer use this shared representation.

use std::path::PathBuf;
use std::sync::Arc;

// ============================================================================
// Source Location Types (for debugging and error reporting)
// ============================================================================

/// Source span: complete location in source code including file path
///
/// Contains line/column positions and an Arc-wrapped file path for efficient
/// sharing across all expressions from the same file. The Arc means cloning
/// a SourceSpan is cheap (just increments reference count).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceSpan {
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)  
    pub column: u32,
    /// End line (for multi-line expressions)
    pub end_line: u32,
    /// End column
    pub end_column: u32,
    /// File path (Arc for cheap cloning - all expressions in same file share this)
    #[serde(skip)]
    pub file: Option<Arc<PathBuf>>,
}

impl SourceSpan {
    pub fn new(line: u32, column: u32) -> Self {
        Self {
            line,
            column,
            end_line: line,
            end_column: column,
            file: None,
        }
    }

    pub fn with_end(mut self, end_line: u32, end_column: u32) -> Self {
        self.end_line = end_line;
        self.end_column = end_column;
        self
    }

    pub fn with_file(mut self, file: Arc<PathBuf>) -> Self {
        self.file = Some(file);
        self
    }

    /// Get the file path as a string (for debugging/display)
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file.as_deref()
    }
}

/// Full source location: span + file path (for cross-file debugging)
///
/// This is the complete location information needed for debugging,
/// especially when stepping through imported files.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FullSourceLocation {
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// File path (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl FullSourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self {
            line,
            column,
            file: None,
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn from_span(span: &SourceSpan) -> Self {
        Self {
            line: span.line,
            column: span.column,
            file: None,
        }
    }

    pub fn from_span_with_file(span: &SourceSpan, file: impl Into<String>) -> Self {
        Self {
            line: span.line,
            column: span.column,
            file: Some(file.into()),
        }
    }
}

/// Source location with optional file path (for cross-file debugging)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceLocation {
    pub span: SourceSpan,
    pub file: Option<PathBuf>,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self {
            span: SourceSpan::new(line, column),
            file: None,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn line(&self) -> u32 {
        self.span.line
    }

    pub fn column(&self) -> u32 {
        self.span.column
    }
}

/// Expression with source location attached
///
/// This wrapper allows tracking source locations without modifying the
/// core Expression enum. Use this in the parser for new code paths.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Option<SourceSpan>,
    /// File path for cross-file debugging
    pub file: Option<PathBuf>,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: SourceSpan) -> Self {
        Self {
            node,
            span: Some(span),
            file: None,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn new_with_file(node: T, span: SourceSpan, file: PathBuf) -> Self {
        Self {
            node,
            span: Some(span),
            file: Some(file),
        }
    }

    pub fn unspanned(node: T) -> Self {
        Self {
            node,
            span: None,
            file: None,
        }
    }

    /// Get full source location (span + file) for debugging
    pub fn location(&self) -> Option<SourceLocation> {
        self.span.clone().map(|s| SourceLocation {
            span: s,
            file: self.file.clone(),
        })
    }
}

/// Spanned expression - an expression with its source location
pub type SpannedExpr = Spanned<Expression>;

/// Core expression type representing mathematical structures
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Expression {
    /// Numeric constant (e.g., "1", "2", "3.14")
    Const(String),

    /// String literal (e.g., "hello", "world")
    /// Grammar v0.8: string ::= '"' { character } '"'
    /// Used for text values, labels, file paths, etc.
    String(String),

    /// Named object/variable (e.g., "x", "\\alpha", "\\pi")
    Object(String),

    /// Operation with a name and arguments
    /// Examples:
    /// - plus(a, b) for addition
    /// - sqrt(x) for square root
    /// - frac(num, den) for fractions
    Operation {
        name: String,
        args: Vec<Expression>,
        #[serde(skip)]
        span: Option<SourceSpan>,
    },

    /// Placeholder for structural editing
    /// Used to represent empty slots that need to be filled
    /// id: unique identifier for this placeholder
    /// hint: user-friendly description of what should go here (e.g., "numerator", "exponent")
    Placeholder { id: usize, hint: String },

    /// Pattern matching expression (ADR-021)
    /// Example: match myOption { None => 0 | Some(x) => x }
    Match {
        scrutinee: Box<Expression>,
        cases: Vec<MatchCase>,
        #[serde(skip)]
        span: Option<SourceSpan>,
    },

    /// List literal
    /// Example: [1, 2, 3] or [x, y, z]
    /// This enables Matrix(2, 2, [a, b, c, d]) instead of variable-arity Matrix(2, 2, a, b, c, d)
    List(Vec<Expression>),

    /// Universal quantifier (for axioms)
    /// Example: ∀(x : M). x • e = x
    /// With where clause: ∀(x : F) where x ≠ zero. inverse(x) × x = one
    /// Used in axiom propositions for theorem proving with Z3
    Quantifier {
        quantifier: QuantifierKind,
        variables: Vec<QuantifiedVar>,
        where_clause: Option<Box<Expression>>,
        body: Box<Expression>,
    },

    /// Conditional expression (if-then-else)
    /// Example: if x > 0 then x else -x
    /// Translates to Z3's ite (if-then-else) construct
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
        #[serde(skip)]
        span: Option<SourceSpan>,
    },

    /// Let binding expression with pattern support (Grammar v0.8)
    ///
    /// Simple: `let x = 5 in x + x`
    /// With type: `let x : ℝ = 5 in x^2`
    /// Destructuring: `let Point(x, y) = origin in x^2 + y^2` (v0.8)
    ///
    /// Introduces local variable binding(s) within an expression.
    /// Pure functional semantics: bound values are substituted into the body.
    ///
    /// Note: Simple `let x = ...` uses Pattern::Variable("x")
    Let {
        /// Pattern to match against (v0.8: was `name: String`)
        /// Simple names use Pattern::Variable
        pattern: Pattern,
        /// Optional type annotation (e.g., "ℝ", "ℤ", "Vector(3)")
        /// Only valid when pattern is a simple Variable
        type_annotation: Option<String>,
        value: Box<Expression>,
        body: Box<Expression>,
        #[serde(skip)]
        span: Option<SourceSpan>,
    },

    /// Type ascription expression
    /// Example: (a + b) : ℝ
    /// Annotates an expression with an explicit type.
    /// Used for disambiguation, documentation, or type checking.
    /// Follows Haskell convention: expr :: Type (we use single colon)
    Ascription {
        expr: Box<Expression>,
        /// The type annotation (e.g., "ℝ", "Vector(3)", "ℝ → ℝ")
        type_annotation: String,
    },

    /// Lambda expression (anonymous function)
    /// Example: λ x . x + 1 or lambda x y . x * y
    /// With type annotations: λ (x : ℝ) . x^2
    /// Grammar: lambda ::= "λ" params "." expression | "lambda" params "." expression
    Lambda {
        params: Vec<LambdaParam>,
        body: Box<Expression>,
        #[serde(skip)]
        span: Option<SourceSpan>,
    },
}

/// Kind of quantifier
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QuantifierKind {
    /// Universal quantifier: ∀ (for all)
    ForAll,
    /// Existential quantifier: ∃ (there exists)
    Exists,
}

/// A quantified variable with optional type annotation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QuantifiedVar {
    pub name: String,
    pub type_annotation: Option<String>, // e.g., "M", "R", "Nat"
}

impl QuantifiedVar {
    /// Create a quantified variable with type annotation
    pub fn new(name: impl Into<String>, type_annotation: Option<impl Into<String>>) -> Self {
        QuantifiedVar {
            name: name.into(),
            type_annotation: type_annotation.map(|t| t.into()),
        }
    }
}

/// A lambda parameter with optional type annotation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LambdaParam {
    pub name: String,
    pub type_annotation: Option<String>, // e.g., "ℝ", "ℤ", "Vector(3)"
}

impl LambdaParam {
    /// Create a lambda parameter without type annotation
    pub fn new(name: impl Into<String>) -> Self {
        LambdaParam {
            name: name.into(),
            type_annotation: None,
        }
    }

    /// Create a lambda parameter with type annotation
    pub fn typed(name: impl Into<String>, type_annotation: impl Into<String>) -> Self {
        LambdaParam {
            name: name.into(),
            type_annotation: Some(type_annotation.into()),
        }
    }
}

/// A single case in a match expression
///
/// Grammar v0.8 adds optional guard:
///   matchCase ::= pattern ["if" expression] "=>" expression
///
/// Example: `x if x < 0 => "negative"`
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MatchCase {
    pub pattern: Pattern,
    /// Optional guard expression (Grammar v0.8)
    /// If present, pattern matches only when guard evaluates to true
    pub guard: Option<Expression>,
    pub body: Expression,
}

/// Pattern for matching against data constructors
///
/// Grammar v0.8 adds As-patterns for alias binding:
///   pattern ::= "_" | identifier | constructor | constant | pattern "as" identifier
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Pattern {
    /// Wildcard pattern: _
    Wildcard,

    /// Variable binding: x
    Variable(String),

    /// Constructor pattern: Some(x), None, True
    Constructor { name: String, args: Vec<Pattern> },

    /// Constant pattern: 0, 1, "hello"
    Constant(String),

    /// As-pattern (alias binding): Cons(h, t) as whole (Grammar v0.8)
    /// Binds both the destructured parts AND the whole value
    /// Like Haskell's `list@(x:xs)` or Rust's `list @ [head, ..]`
    As {
        pattern: Box<Pattern>,
        binding: String,
    },
}

impl Expression {
    /// Create a constant expression
    pub fn constant(s: impl Into<String>) -> Self {
        Expression::Const(s.into())
    }

    /// Create a string literal expression
    pub fn string(s: impl Into<String>) -> Self {
        Expression::String(s.into())
    }

    /// Create an object/variable expression
    pub fn object(s: impl Into<String>) -> Self {
        Expression::Object(s.into())
    }

    /// Create an operation expression
    pub fn operation(name: impl Into<String>, args: Vec<Expression>) -> Self {
        Expression::Operation {
            name: name.into(),
            args,
            span: None,
        }
    }

    /// Create a placeholder expression
    pub fn placeholder(id: usize, hint: impl Into<String>) -> Self {
        Expression::Placeholder {
            id,
            hint: hint.into(),
        }
    }

    /// Create a match expression
    pub fn match_expr(scrutinee: Expression, cases: Vec<MatchCase>) -> Self {
        Expression::Match {
            scrutinee: Box::new(scrutinee),
            cases,
            span: None,
        }
    }

    /// Create a universal quantifier expression
    pub fn forall(variables: Vec<QuantifiedVar>, body: Expression) -> Self {
        Expression::Quantifier {
            quantifier: QuantifierKind::ForAll,
            variables,
            where_clause: None,
            body: Box::new(body),
        }
    }

    /// Create an existential quantifier expression
    pub fn exists(variables: Vec<QuantifiedVar>, body: Expression) -> Self {
        Expression::Quantifier {
            quantifier: QuantifierKind::Exists,
            variables,
            where_clause: None,
            body: Box::new(body),
        }
    }

    /// Create a conditional (if-then-else) expression
    pub fn conditional(
        condition: Expression,
        then_branch: Expression,
        else_branch: Expression,
    ) -> Self {
        Expression::Conditional {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            span: None,
        }
    }

    /// Create a let binding expression without type annotation
    /// Create a let binding expression without type annotation
    /// Simple name binding: `let x = 5 in x + x`
    pub fn let_binding(name: impl Into<String>, value: Expression, body: Expression) -> Self {
        Expression::Let {
            pattern: Pattern::Variable(name.into()),
            type_annotation: None,
            value: Box::new(value),
            body: Box::new(body),
            span: None,
        }
    }

    /// Create a let binding expression with optional type annotation
    /// Typed binding: `let x : ℝ = 5 in x^2`
    pub fn let_binding_typed(
        name: impl Into<String>,
        type_annotation: Option<impl Into<String>>,
        value: Expression,
        body: Expression,
    ) -> Self {
        Expression::Let {
            pattern: Pattern::Variable(name.into()),
            type_annotation: type_annotation.map(|t| t.into()),
            value: Box::new(value),
            body: Box::new(body),
            span: None,
        }
    }

    /// Create a let binding with pattern destructuring (Grammar v0.8)
    /// Destructuring: `let Point(x, y) = origin in x^2 + y^2`
    pub fn let_pattern(pattern: Pattern, value: Expression, body: Expression) -> Self {
        Expression::Let {
            pattern,
            type_annotation: None,
            value: Box::new(value),
            body: Box::new(body),
            span: None,
        }
    }

    /// Create a type ascription expression
    /// Example: ascription(expr, "ℝ") for (expr) : ℝ
    pub fn ascription(expr: Expression, type_annotation: impl Into<String>) -> Self {
        Expression::Ascription {
            expr: Box::new(expr),
            type_annotation: type_annotation.into(),
        }
    }

    /// Create a lambda expression
    /// Example: lambda(vec![param("x")], body) for λ x . body
    pub fn lambda(params: Vec<LambdaParam>, body: Expression) -> Self {
        Expression::Lambda {
            params,
            body: Box::new(body),
            span: None,
        }
    }

    /// Traverse the expression tree to find all placeholders
    pub fn find_placeholders(&self) -> Vec<(usize, String)> {
        let mut placeholders = Vec::new();
        self.collect_placeholders(&mut placeholders);
        placeholders
    }

    fn collect_placeholders(&self, acc: &mut Vec<(usize, String)>) {
        match self {
            Expression::Placeholder { id, hint } => {
                acc.push((*id, hint.clone()));
            }
            Expression::Operation { args, .. } => {
                for arg in args {
                    arg.collect_placeholders(acc);
                }
            }
            Expression::Match {
                scrutinee, cases, ..
            } => {
                scrutinee.collect_placeholders(acc);
                for case in cases {
                    case.body.collect_placeholders(acc);
                }
            }
            Expression::Quantifier { body, .. } => {
                body.collect_placeholders(acc);
            }
            Expression::List(items) => {
                for item in items {
                    item.collect_placeholders(acc);
                }
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                condition.collect_placeholders(acc);
                then_branch.collect_placeholders(acc);
                else_branch.collect_placeholders(acc);
            }
            Expression::Let { value, body, .. } => {
                value.collect_placeholders(acc);
                body.collect_placeholders(acc);
            }
            Expression::Ascription { expr, .. } => {
                expr.collect_placeholders(acc);
            }
            Expression::Lambda { body, .. } => {
                body.collect_placeholders(acc);
            }
            // Const, String, and Object are leaf nodes with no placeholders
            Expression::Const(_) | Expression::String(_) | Expression::Object(_) => {}
        }
    }

    /// Get the next placeholder ID after the given one
    pub fn next_placeholder(&self, current_id: usize) -> Option<usize> {
        let placeholders = self.find_placeholders();
        placeholders
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| *id > current_id)
            .min()
    }

    /// Get the previous placeholder ID before the given one
    pub fn prev_placeholder(&self, current_id: usize) -> Option<usize> {
        let placeholders = self.find_placeholders();
        placeholders
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| *id < current_id)
            .max()
    }
}

impl MatchCase {
    /// Create a match case without guard
    pub fn new(pattern: Pattern, body: Expression) -> Self {
        MatchCase {
            pattern,
            guard: None,
            body,
        }
    }

    /// Create a match case with guard (Grammar v0.8)
    /// Example: `x if x < 0 => "negative"`
    pub fn with_guard(pattern: Pattern, guard: Expression, body: Expression) -> Self {
        MatchCase {
            pattern,
            guard: Some(guard),
            body,
        }
    }
}

impl Pattern {
    /// Create a wildcard pattern
    pub fn wildcard() -> Self {
        Pattern::Wildcard
    }

    /// Create a variable pattern
    pub fn variable(name: impl Into<String>) -> Self {
        Pattern::Variable(name.into())
    }

    /// Create a constructor pattern
    pub fn constructor(name: impl Into<String>, args: Vec<Pattern>) -> Self {
        Pattern::Constructor {
            name: name.into(),
            args,
        }
    }

    /// Create a constant pattern
    pub fn constant(value: impl Into<String>) -> Self {
        Pattern::Constant(value.into())
    }

    /// Create an as-pattern (alias binding) (Grammar v0.8)
    /// Example: `Cons(h, t) as whole` binds whole to the entire matched value
    pub fn as_pattern(pattern: Pattern, binding: impl Into<String>) -> Self {
        Pattern::As {
            pattern: Box::new(pattern),
            binding: binding.into(),
        }
    }
}
