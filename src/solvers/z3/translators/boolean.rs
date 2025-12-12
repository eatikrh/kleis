//! Boolean Operation Translators
//!
//! Translates Kleis boolean/logical operations to Z3's boolean theory.
//!
//! **Supported Operations:**
//! - `and`, `logical_and`: Conjunction (∧)
//! - `or`, `logical_or`: Disjunction (∨)
//! - `not`, `logical_not`: Negation (¬)
//! - `implies`: Implication (→)
//!
//! **Type Requirements:**
//! All operations require Bool type. Attempting to apply boolean operations
//! to non-boolean types will return an error.
//!
//! **Example Translations:**
//! ```ignore
//! Kleis: and(p, q)        → Z3: Bool::and([p, q])     → Bool
//! Kleis: or(p, q)         → Z3: Bool::or([p, q])      → Bool
//! Kleis: not(p)           → Z3: p.not()               → Bool
//! Kleis: implies(p, q)    → Z3: p.implies(&q)         → Bool
//! ```
//!
//! **Design Note:**
//! These are the core operations for propositional logic and quantified formulas.
//! Z3's boolean theory is decidable and highly optimized.

use z3::ast::{Bool, Dynamic};

/// Translate and/logical_and operation
///
/// Requires both arguments to be Bool type.
/// Returns Bool representing logical conjunction.
pub fn translate_and(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    let left_bool = left
        .as_bool()
        .ok_or_else(|| "and requires Bool arguments".to_string())?;
    let right_bool = right
        .as_bool()
        .ok_or_else(|| "and requires Bool arguments".to_string())?;

    Ok(Bool::and(&[left_bool, right_bool]))
}

/// Translate or/logical_or operation
///
/// Requires both arguments to be Bool type.
/// Returns Bool representing logical disjunction.
pub fn translate_or(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    let left_bool = left
        .as_bool()
        .ok_or_else(|| "or requires Bool arguments".to_string())?;
    let right_bool = right
        .as_bool()
        .ok_or_else(|| "or requires Bool arguments".to_string())?;

    Ok(Bool::or(&[left_bool, right_bool]))
}

/// Translate not/logical_not operation
///
/// Requires argument to be Bool type.
/// Returns Bool representing logical negation.
pub fn translate_not(arg: &Dynamic) -> Result<Bool, String> {
    let arg_bool = arg
        .as_bool()
        .ok_or_else(|| "not requires Bool argument".to_string())?;

    Ok(arg_bool.not())
}

/// Translate implies operation
///
/// Requires both arguments to be Bool type.
/// Returns Bool representing logical implication (p → q ≡ ¬p ∨ q).
pub fn translate_implies(
    left: &Dynamic,
    right: &Dynamic,
) -> Result<Bool, String> {
    let left_bool = left
        .as_bool()
        .ok_or_else(|| "implies requires Bool arguments".to_string())?;
    let right_bool = right
        .as_bool()
        .ok_or_else(|| "implies requires Bool arguments".to_string())?;

    Ok(left_bool.implies(&right_bool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::ast::Bool as Z3Bool;

    #[test]
    fn test_translate_and() {
        let left: Dynamic = Z3Bool::from_bool(true).into();
        let right: Dynamic = Z3Bool::from_bool(false).into();

        let result = translate_and(&left, &right);
        assert!(result.is_ok(), "translate_and should succeed");
        // Result is Bool, not Dynamic, so it's always valid
    }

    #[test]
    fn test_translate_or() {
        let left: Dynamic = Z3Bool::from_bool(true).into();
        let right: Dynamic = Z3Bool::from_bool(false).into();

        let result = translate_or(&left, &right);
        assert!(result.is_ok(), "translate_or should succeed");
    }

    #[test]
    fn test_translate_not() {
        let arg: Dynamic = Z3Bool::from_bool(true).into();

        let result = translate_not(&arg);
        assert!(result.is_ok(), "translate_not should succeed");
    }

    #[test]
    fn test_translate_implies() {
        let left: Dynamic = Z3Bool::from_bool(true).into();
        let right: Dynamic = Z3Bool::from_bool(false).into();

        let result = translate_implies(&left, &right);
        assert!(result.is_ok(), "translate_implies should succeed");
    }

    #[test]
    fn test_non_bool_error() {
        use z3::ast::Int;

        let left: Dynamic = Int::from_i64(42).into();
        let right: Dynamic = Int::from_i64(17).into();

        // Should error because Int is not Bool
        let result = translate_and(&left, &right);
        assert!(result.is_err());
    }
}

