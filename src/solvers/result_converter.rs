//! Result Converter - Solver Results to Kleis AST
//!
//! **CRITICAL ABSTRACTION BOUNDARY**
//!
//! This module maintains solver independence by converting solver-specific
//! result types back to Kleis expressions.
//!
//! **Why this matters:**
//! - Z3 returns `Dynamic`, `Int`, `Real`, `Bool` types
//! - CVC5 has its own type system
//! - Kleis code should NEVER see these types
//! - All public APIs work with `Expression` only
//!
//! **Design Pattern:**
//! ```
//! User Code
//!    ↓ Expression
//! SolverBackend::evaluate()
//!    ↓ (internal translation)
//! Solver-specific types (Z3::Dynamic, etc.)
//!    ↓ (result conversion) ← THIS MODULE
//! Expression
//!    ↑
//! User Code
//! ```
//!
//! Each solver implementation provides its own ResultConverter.

use crate::ast::Expression;

/// Convert solver-specific result values to Kleis expressions
///
/// This trait is implemented by each solver backend to convert their
/// internal result types back to Kleis AST.
///
/// # Example (Z3 Implementation)
/// ```ignore
/// impl ResultConverter for Z3ResultConverter {
///     fn to_expression(&self, value: &Dynamic) -> Result<Expression, String> {
///         if let Some(i) = value.as_int() {
///             Ok(Expression::Const(i.to_string()))
///         } else if let Some(b) = value.as_bool() {
///             Ok(Expression::Const(b.to_string()))
///         } else {
///             Err("Unsupported Z3 type".to_string())
///         }
///     }
/// }
/// ```
pub trait ResultConverter<SolverValue> {
    /// Convert solver result to Kleis expression
    ///
    /// This is the primary method that maintains the abstraction boundary.
    ///
    /// # Arguments
    /// * `value` - Solver-specific result value
    ///
    /// # Returns
    /// Kleis Expression representing the result
    fn to_expression(&self, value: &SolverValue) -> Result<Expression, String>;

    /// Convert to integer constant (convenience method)
    ///
    /// Converts solver value to Expression, then extracts integer.
    ///
    /// # Returns
    /// Integer value if result is a constant integer
    fn to_i64(&self, value: &SolverValue) -> Result<i64, String> {
        let expr = self.to_expression(value)?;
        match expr {
            Expression::Const(s) => s
                .parse()
                .map_err(|_| format!("Not an integer: {}", s)),
            _ => Err(format!("Not a constant: {:?}", expr)),
        }
    }

    /// Convert to boolean constant (convenience method)
    ///
    /// Converts solver value to Expression, then extracts boolean.
    ///
    /// # Returns
    /// Boolean value if result is a constant boolean
    fn to_bool(&self, value: &SolverValue) -> Result<bool, String> {
        let expr = self.to_expression(value)?;
        match expr {
            Expression::Const(s) => match s.as_str() {
                "true" | "True" | "TRUE" => Ok(true),
                "false" | "False" | "FALSE" => Ok(false),
                _ => Err(format!("Not a boolean: {}", s)),
            },
            _ => Err(format!("Not a constant: {:?}", expr)),
        }
    }

    /// Convert to floating point constant (convenience method)
    ///
    /// Converts solver value to Expression, then extracts float.
    ///
    /// # Returns
    /// Float value if result is a constant real number
    fn to_f64(&self, value: &SolverValue) -> Result<f64, String> {
        let expr = self.to_expression(value)?;
        match expr {
            Expression::Const(s) => s
                .parse()
                .map_err(|_| format!("Not a float: {}", s)),
            _ => Err(format!("Not a constant: {:?}", expr)),
        }
    }

    /// Convert to string representation (convenience method)
    ///
    /// Useful for debugging and error messages.
    ///
    /// # Returns
    /// String representation of the value
    fn to_string(&self, value: &SolverValue) -> String {
        self.to_expression(value)
            .map(|expr| format!("{:?}", expr))
            .unwrap_or_else(|e| format!("<error: {}>", e))
    }
}

/// Helper: Convert Kleis Expression to specific primitive type
///
/// This is the inverse operation - extracting primitives from Kleis expressions.
/// Useful for test assertions and validation.
pub mod expression_extractors {
    use crate::ast::Expression;

    /// Extract integer from Expression::Const
    pub fn extract_i64(expr: &Expression) -> Result<i64, String> {
        match expr {
            Expression::Const(s) => s
                .parse()
                .map_err(|_| format!("Not an integer: {}", s)),
            _ => Err(format!("Not a constant: {:?}", expr)),
        }
    }

    /// Extract boolean from Expression::Const
    pub fn extract_bool(expr: &Expression) -> Result<bool, String> {
        match expr {
            Expression::Const(s) => match s.as_str() {
                "true" | "True" | "TRUE" => Ok(true),
                "false" | "False" | "FALSE" => Ok(false),
                _ => Err(format!("Not a boolean: {}", s)),
            },
            _ => Err(format!("Not a constant: {:?}", expr)),
        }
    }

    /// Extract float from Expression::Const
    pub fn extract_f64(expr: &Expression) -> Result<f64, String> {
        match expr {
            Expression::Const(s) => s
                .parse()
                .map_err(|_| format!("Not a float: {}", s)),
            _ => Err(format!("Not a constant: {:?}", expr)),
        }
    }

    /// Extract variable name from Expression::Object
    pub fn extract_var_name(expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Object(name) => Ok(name.clone()),
            _ => Err(format!("Not a variable: {:?}", expr)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock solver value for testing
    #[derive(Debug)]
    enum MockValue {
        Int(i64),
        Bool(bool),
        Real(f64),
    }

    // Mock converter implementation
    struct MockConverter;

    impl ResultConverter<MockValue> for MockConverter {
        fn to_expression(&self, value: &MockValue) -> Result<Expression, String> {
            match value {
                MockValue::Int(i) => Ok(Expression::Const(i.to_string())),
                MockValue::Bool(b) => Ok(Expression::Const(b.to_string())),
                MockValue::Real(r) => Ok(Expression::Const(r.to_string())),
            }
        }
    }

    #[test]
    fn test_convert_int() {
        let converter = MockConverter;
        let value = MockValue::Int(42);
        let expr = converter.to_expression(&value).unwrap();
        assert_eq!(expr, Expression::Const("42".to_string()));
    }

    #[test]
    fn test_convert_bool() {
        let converter = MockConverter;
        let value = MockValue::Bool(true);
        let expr = converter.to_expression(&value).unwrap();
        assert_eq!(expr, Expression::Const("true".to_string()));
    }

    #[test]
    fn test_to_i64_convenience() {
        let converter = MockConverter;
        let value = MockValue::Int(42);
        let result = converter.to_i64(&value).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_to_bool_convenience() {
        let converter = MockConverter;
        let value = MockValue::Bool(false);
        let result = converter.to_bool(&value).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_expression_extractors() {
        use expression_extractors::*;

        let int_expr = Expression::Const("123".to_string());
        assert_eq!(extract_i64(&int_expr).unwrap(), 123);

        let bool_expr = Expression::Const("true".to_string());
        assert!(extract_bool(&bool_expr).unwrap());

        let float_expr = Expression::Const("3.14".to_string());
        assert!((extract_f64(&float_expr).unwrap() - 3.14).abs() < 0.001);

        let var_expr = Expression::Object("x".to_string());
        assert_eq!(extract_var_name(&var_expr).unwrap(), "x");
    }
}

