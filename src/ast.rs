//! Abstract Syntax Tree for Kleis mathematical expressions
//!
//! This module defines the core Expression type used throughout the system.
//! Both the parser and renderer use this shared representation.

/// Core expression type representing mathematical structures
#[derive(Debug, Clone, PartialEq)]
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
    Operation {
        name: String,
        args: Vec<Expression>,
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
}

