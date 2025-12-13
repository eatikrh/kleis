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
pub fn translate_and(left: &Dynamic, right: &Dynamic) -> Result<Bool, String> {
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
pub fn translate_or(left: &Dynamic, right: &Dynamic) -> Result<Bool, String> {
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
pub fn translate_implies(left: &Dynamic, right: &Dynamic) -> Result<Bool, String> {
    let left_bool = left
        .as_bool()
        .ok_or_else(|| "implies requires Bool arguments".to_string())?;
    let right_bool = right
        .as_bool()
        .ok_or_else(|| "implies requires Bool arguments".to_string())?;

    Ok(left_bool.implies(&right_bool))
}

/// Translate if-then-else (ite) expression
///
/// Takes a boolean condition and two branches of the same type.
/// Returns the appropriate branch based on the condition value.
///
/// This is the core translation for Kleis's `if cond then a else b` syntax.
/// Z3's ite is polymorphic over the branch types.
///
/// # Example
/// ```ignore
/// Kleis: if x > 0 then x else -x
/// Z3: ite(x > 0, x, -x)
/// ```
pub fn translate_ite(condition: &Bool, then_branch: &Dynamic, else_branch: &Dynamic) -> Dynamic {
    condition.ite(then_branch, else_branch)
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

    #[test]
    fn test_translate_ite_bool_branches() {
        let cond = Z3Bool::from_bool(true);
        let then_branch: Dynamic = Z3Bool::from_bool(true).into();
        let else_branch: Dynamic = Z3Bool::from_bool(false).into();

        let result = translate_ite(&cond, &then_branch, &else_branch);
        // Result should be a Bool (since branches are Bool)
        assert!(result.as_bool().is_some());
    }

    #[test]
    fn test_translate_ite_int_branches() {
        use z3::ast::Int;

        let cond = Z3Bool::from_bool(true);
        let then_branch: Dynamic = Int::from_i64(42).into();
        let else_branch: Dynamic = Int::from_i64(17).into();

        let result = translate_ite(&cond, &then_branch, &else_branch);
        // Result should be an Int (since branches are Int)
        assert!(result.as_int().is_some());
    }

    #[test]
    fn test_translate_ite_preserves_type() {
        use z3::ast::Real;

        let cond = Z3Bool::from_bool(false);
        let then_branch: Dynamic = Real::from_real(1, 2).into(); // 0.5
        let else_branch: Dynamic = Real::from_real(3, 4).into(); // 0.75

        let result = translate_ite(&cond, &then_branch, &else_branch);
        // Result should be a Real
        assert!(result.as_real().is_some());
    }
}
