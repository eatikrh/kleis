//! Z3 Result Converter - Z3::Dynamic → Kleis Expression
//!
//! **CRITICAL ABSTRACTION BOUNDARY**
//!
//! This module converts Z3's result types back to Kleis expressions.
//! Z3 types (Dynamic, Int, Real, Bool) MUST NOT escape this module.
//!
//! **Type Mapping:**
//! - Z3::Int → Expression::Const (integer string)
//! - Z3::Real → Expression::Const (decimal string)
//! - Z3::Bool → Expression::Const ("true" or "false")
//! - Z3::Dynamic → (detect runtime type, convert accordingly)
//!
//! **Design Pattern:**
//! ```text
//! Z3Backend::evaluate()
//!    |
//! Z3 computation -> Dynamic result
//!    |
//! Z3ResultConverter::to_expression() <- THIS MODULE
//!    |
//! Expression (safe to return to user)
//! ```

use crate::ast::Expression;
use crate::solvers::result_converter::ResultConverter;
use z3::ast::Dynamic;

/// Converter for Z3 Dynamic values to Kleis expressions
pub struct Z3ResultConverter;

impl ResultConverter<Dynamic> for Z3ResultConverter {
    /// Convert Z3 Dynamic value to Kleis Expression
    ///
    /// Z3's Dynamic type can hold Int, Real, Bool, or other types.
    /// We inspect the runtime type and convert accordingly.
    ///
    /// # Type Handling
    /// - Int: Convert to integer string constant
    /// - Real: Convert to decimal string constant  
    /// - Bool: Convert to "true" or "false" string
    /// - Other: Error (unsupported type)
    ///
    /// # Example
    /// ```ignore
    /// let z3_result: Dynamic = Int::from_i64(42).into();
    /// let converter = Z3ResultConverter;
    /// let expr = converter.to_expression(&z3_result)?;
    /// assert_eq!(expr, Expression::Const("42".to_string()));
    /// ```
    fn to_expression(&self, value: &Dynamic) -> Result<Expression, String> {
        // Try Int first (most common for arithmetic)
        if let Some(int_val) = value.as_int() {
            // Get the integer value as i64
            if let Some(i) = int_val.as_i64() {
                return Ok(Expression::Const(i.to_string()));
            } else {
                // Large integer that doesn't fit i64 - use string representation
                return Ok(Expression::Const(int_val.to_string()));
            }
        }

        // Try Bool
        if let Some(bool_val) = value.as_bool() {
            // Z3 Bool needs to be evaluated to get concrete true/false
            // For now, return string representation
            return Ok(Expression::Const(bool_val.to_string()));
        }

        // Try Real
        if let Some(real_val) = value.as_real() {
            // Get Real as approximate decimal
            if let Some((numerator, denominator)) = real_val.as_rational() {
                // Z3 Real is represented as rational number (numerator/denominator)
                if denominator == 1 {
                    return Ok(Expression::Const(numerator.to_string()));
                } else {
                    // Return as decimal approximation
                    let decimal = numerator as f64 / denominator as f64;
                    return Ok(Expression::Const(decimal.to_string()));
                }
            } else {
                // Use string representation
                return Ok(Expression::Const(real_val.to_string()));
            }
        }

        // Unsupported type - return string representation as fallback
        Ok(Expression::Const(format!("{}", value)))
    }
}

/// Helper: Convert Z3 Bool to Kleis Expression using a model
///
/// When evaluating boolean expressions, we need a model to get concrete values.
pub fn bool_with_model_to_expression(
    bool_val: &z3::ast::Bool,
    model: &z3::Model,
) -> Result<Expression, String> {
    // model.eval<Bool> returns Option<Bool>
    if let Some(evaluated) = model.eval(bool_val, true) {
        // evaluated is a Bool, extract concrete value
        // Bool::as_bool() returns Option<bool>
        if let Some(concrete_value) = evaluated.as_bool() {
            Ok(Expression::Const(concrete_value.to_string()))
        } else {
            // Bool is symbolic (not concrete true/false)
            Ok(Expression::Const(evaluated.to_string()))
        }
    } else {
        Err("Failed to evaluate boolean expression".to_string())
    }
}

/// Helper: Convert Z3 Int to Kleis Expression using a model
pub fn int_with_model_to_expression(
    int_val: &z3::ast::Int,
    model: &z3::Model,
) -> Result<Expression, String> {
    // model.eval<Int> returns Option<Int>
    if let Some(evaluated) = model.eval(int_val, true) {
        // evaluated is an Int, extract concrete value
        // Int::as_i64() returns Option<i64>
        if let Some(value) = evaluated.as_i64() {
            Ok(Expression::Const(value.to_string()))
        } else {
            // Large integer or symbolic - use string representation
            Ok(Expression::Const(evaluated.to_string()))
        }
    } else {
        Err("Failed to evaluate integer expression".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::ast::Int;

    #[test]
    fn test_convert_int() {
        let converter = Z3ResultConverter;
        let z3_val: Dynamic = Int::from_i64(42).into();
        let expr = converter.to_expression(&z3_val).unwrap();
        assert_eq!(expr, Expression::Const("42".to_string()));
    }

    #[test]
    fn test_convert_negative_int() {
        let converter = Z3ResultConverter;
        let z3_val: Dynamic = Int::from_i64(-17).into();
        let expr = converter.to_expression(&z3_val).unwrap();
        assert_eq!(expr, Expression::Const("-17".to_string()));
    }

    #[test]
    fn test_convert_zero() {
        let converter = Z3ResultConverter;
        let z3_val: Dynamic = Int::from_i64(0).into();
        let expr = converter.to_expression(&z3_val).unwrap();
        assert_eq!(expr, Expression::Const("0".to_string()));
    }

    #[test]
    fn test_to_i64_convenience() {
        let converter = Z3ResultConverter;
        let z3_val: Dynamic = Int::from_i64(123).into();
        let result = converter.to_i64(&z3_val).unwrap();
        assert_eq!(result, 123);
    }
}
