//! Abstract Syntax Tree for Kleis mathematical expressions
//!
//! This module defines the core Expression type used throughout the system.
//! Both the parser and renderer use this shared representation.

/// Core expression type representing mathematical structures
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Expression {
    /// Numeric constant (e.g., "1", "2", "3.14")
    Const(String),

    /// Named object/variable (e.g., "x", "\\alpha", "\\pi")
    Object(String),

    /// Operation with a name and arguments
    /// Examples:
    /// - plus(a, b) for addition
    /// - sqrt(x) for square root
    /// - frac(num, den) for fractions
    Operation { name: String, args: Vec<Expression> },

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

    /// Create an object/variable expression
    pub fn object(s: impl Into<String>) -> Self {
        Expression::Object(s.into())
    }

    /// Create an operation expression
    pub fn operation(name: impl Into<String>, args: Vec<Expression>) -> Self {
        Expression::Operation {
            name: name.into(),
            args,
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
            Expression::Match { scrutinee, cases } => {
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
            _ => {}
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
