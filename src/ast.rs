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

    /// Let binding expression
    /// Example: let x = 5 in x + x
    /// Introduces a local variable binding within a function definition.
    /// Pure functional semantics: the bound value is substituted into the body.
    Let {
        name: String,
        value: Box<Expression>,
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

/// A single case in a match expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Expression,
}

/// Pattern for matching against data constructors
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

    /// Create a let binding expression
    pub fn let_binding(name: impl Into<String>, value: Expression, body: Expression) -> Self {
        Expression::Let {
            name: name.into(),
            value: Box::new(value),
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
    /// Create a match case
    pub fn new(pattern: Pattern, body: Expression) -> Self {
        MatchCase { pattern, body }
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
}
