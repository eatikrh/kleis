//! Comparison Operation Translators
//!
//! Translates Kleis comparison operations to Z3's comparison operations.
//!
//! **Supported Operations:**
//! - `equals`, `eq`: Equality (works with any type)
//! - `less_than`, `lt`: Less than (Int and Real)
//! - `greater_than`, `gt`: Greater than (Int and Real)
//! - `leq`: Less than or equal (Int and Real)
//! - `geq`: Greater than or equal (Int and Real)
//!
//! **Type Handling:**
//! - Equality works with any Z3 type (Int, Real, Bool, uninterpreted)
//! - Comparisons (<, >, ≤, ≥) require Int or Real
//! - Mixed Int/Real comparisons convert to Real
//! - Returns Z3 Bool (not Dynamic)
//!
//! **Example Translations:**
//! ```ignore
//! Kleis: lt(2, 3)         → Z3: Int::lt(2, 3)       → true
//! Kleis: equals(x, y)     → Z3: x._eq(&y)           → Bool
//! Kleis: geq(5.0, 3.0)    → Z3: Real::ge(5.0, 3.0)  → true
//! ```

use z3::ast::{Bool, Dynamic};

/// Translate equals/eq operation
///
/// Z3 equality works with any type. If types match, use direct equality.
/// If types differ (Int vs Real), convert both to Real.
pub fn translate_equals(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    // If types match, use direct equality
    if left.sort_kind() == right.sort_kind() {
        return Ok(left.eq(right));
    }

    // Handle mixed Int/Real - convert both to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(l.eq(&r));
    }

    // Fall back to Dynamic equality (may fail if sorts differ)
    Ok(left.eq(right))
}

/// Translate less_than/lt operation
///
/// Requires Int or Real types. Converts to Real if mixed.
pub fn translate_less_than(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    // Try Int < Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(l.lt(&r));
    }

    // Try Real < Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(l.lt(&r));
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(l.lt(&r));
    }

    Err("less_than requires Int or Real types".to_string())
}

/// Translate greater_than/gt operation
///
/// Requires Int or Real types. Converts to Real if mixed.
pub fn translate_greater_than(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    // Try Int > Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(l.gt(&r));
    }

    // Try Real > Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(l.gt(&r));
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(l.gt(&r));
    }

    Err("greater_than requires Int or Real types".to_string())
}

/// Translate leq (≤) operation
///
/// Requires Int or Real types. Converts to Real if mixed.
pub fn translate_leq(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    // Try Int ≤ Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(l.le(&r));
    }

    // Try Real ≤ Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(l.le(&r));
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(l.le(&r));
    }

    Err("leq requires Int or Real types".to_string())
}

/// Translate geq (≥) operation
///
/// Requires Int or Real types. Converts to Real if mixed.
pub fn translate_geq(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    // Try Int ≥ Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(l.ge(&r));
    }

    // Try Real ≥ Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(l.ge(&r));
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(l.ge(&r));
    }

    Err("geq requires Int or Real types".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::ast::Int;

    #[test]
    fn test_translate_equals_int() {
        let left: Dynamic = Int::from_i64(5).into();
        let right: Dynamic = Int::from_i64(5).into();

        let result = translate_equals(&left, &right);
        assert!(result.is_ok(), "translate_equals should succeed");
    }

    #[test]
    fn test_translate_less_than() {
        let left: Dynamic = Int::from_i64(2).into();
        let right: Dynamic = Int::from_i64(3).into();

        let result = translate_less_than(&left, &right);
        assert!(result.is_ok(), "translate_less_than should succeed");
    }

    #[test]
    fn test_translate_greater_than() {
        let left: Dynamic = Int::from_i64(5).into();
        let right: Dynamic = Int::from_i64(2).into();

        let result = translate_greater_than(&left, &right);
        assert!(result.is_ok(), "translate_greater_than should succeed");
    }

    #[test]
    fn test_translate_leq() {
        let left: Dynamic = Int::from_i64(3).into();
        let right: Dynamic = Int::from_i64(3).into();

        let result = translate_leq(&left, &right);
        assert!(result.is_ok(), "translate_leq should succeed");
    }

    #[test]
    fn test_translate_geq() {
        let left: Dynamic = Int::from_i64(7).into();
        let right: Dynamic = Int::from_i64(3).into();

        let result = translate_geq(&left, &right);
        assert!(result.is_ok(), "translate_geq should succeed");
    }
}

