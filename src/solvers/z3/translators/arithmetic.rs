//! Arithmetic Operation Translators
//!
//! Translates Kleis arithmetic operations to Z3's arithmetic theories.
//!
//! **Supported Operations:**
//! - `plus`, `add`: Addition (Int + Int → Int, Real + Real → Real, mixed → Real)
//! - `minus`, `subtract`: Subtraction (same type rules as plus)
//! - `times`, `multiply`: Multiplication (same type rules as plus)
//! - `neg`, `negate`: Unary negation
//!
//! **Mixed Type Handling:**
//! Z3 has separate Int and Real types. When mixing:
//! 1. Try Int + Int → Int (fastest)
//! 2. Try Real + Real → Real
//! 3. Convert Int to Real and do Real arithmetic
//! 4. Fall back to uninterpreted function if types incompatible
//!
//! **Example Translations:**
//! ```ignore
//! Kleis: plus(2, 3)        → Z3: Int::add([2, 3])      → 5
//! Kleis: plus(2.5, 3.0)    → Z3: Real::add([2.5, 3.0]) → 5.5
//! Kleis: plus(2, 3.0)      → Z3: Real::add([2.0, 3.0]) → 5.0 (Int converted)
//! ```

use z3::ast::{Ast, Dynamic, Int, Real};
use z3::{FuncDecl, Sort};

/// Translate plus/add operation
///
/// Handles Int + Int, Real + Real, and mixed Int/Real arithmetic.
/// Falls back to uninterpreted function if types are incompatible.
pub fn translate_plus(left: &Dynamic, right: &Dynamic) -> Result<Dynamic, String> {
    // Try Int + Int (most common case)
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(Int::add(&[&l, &r]).into());
    }

    // Try Real + Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(Real::add(&[&l, &r]).into());
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(Real::add(&[&l, &r]).into());
    }

    // Fall back to uninterpreted function
    let func_decl = declare_uninterpreted("plus", 2);
    let ast_args: Vec<&dyn Ast> = vec![left as &dyn Ast, right as &dyn Ast];
    Ok(func_decl.apply(&ast_args))
}

/// Translate minus/subtract operation
///
/// Same type handling as plus.
pub fn translate_minus(left: &Dynamic, right: &Dynamic) -> Result<Dynamic, String> {
    // Try Int - Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(Int::sub(&[&l, &r]).into());
    }

    // Try Real - Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(Real::sub(&[&l, &r]).into());
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(Real::sub(&[&l, &r]).into());
    }

    // Fall back to uninterpreted function
    let func_decl = declare_uninterpreted("minus", 2);
    let ast_args: Vec<&dyn Ast> = vec![left as &dyn Ast, right as &dyn Ast];
    Ok(func_decl.apply(&ast_args))
}

/// Translate times/multiply operation
///
/// Same type handling as plus.
pub fn translate_times(left: &Dynamic, right: &Dynamic) -> Result<Dynamic, String> {
    // Try Int * Int
    if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
        return Ok(Int::mul(&[&l, &r]).into());
    }

    // Try Real * Real
    if let (Some(l), Some(r)) = (left.as_real(), right.as_real()) {
        return Ok(Real::mul(&[&l, &r]).into());
    }

    // Handle mixed Int/Real - convert to Real
    let l_real = left
        .as_real()
        .or_else(|| left.as_int().map(|i| i.to_real()));
    let r_real = right
        .as_real()
        .or_else(|| right.as_int().map(|i| i.to_real()));

    if let (Some(l), Some(r)) = (l_real, r_real) {
        return Ok(Real::mul(&[&l, &r]).into());
    }

    // Fall back to uninterpreted function
    let func_decl = declare_uninterpreted("times", 2);
    let ast_args: Vec<&dyn Ast> = vec![left as &dyn Ast, right as &dyn Ast];
    Ok(func_decl.apply(&ast_args))
}

/// Translate neg/negate operation (unary negation)
///
/// Handles both Int and Real negation.
pub fn translate_negate(arg: &Dynamic) -> Result<Dynamic, String> {
    // Try Int negation
    if let Some(i) = arg.as_int() {
        return Ok(Int::unary_minus(&i).into());
    }

    // Try Real negation
    if let Some(r) = arg.as_real() {
        return Ok(Real::unary_minus(&r).into());
    }

    // Fall back to uninterpreted function
    let func_decl = declare_uninterpreted("negate", 1);
    let ast_args: Vec<&dyn Ast> = vec![arg as &dyn Ast];
    Ok(func_decl.apply(&ast_args))
}

/// Helper: Declare an uninterpreted function
///
/// Used as fallback when native Z3 operations can't handle the types.
/// Creates a function symbol that Z3 will reason about abstractly.
fn declare_uninterpreted(name: &str, arity: usize) -> FuncDecl {
    let domain: Vec<_> = (0..arity).map(|_| Sort::int()).collect();
    let domain_refs: Vec<_> = domain.iter().collect();
    FuncDecl::new(name, &domain_refs, &Sort::int())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_plus_int() {
        let left: Dynamic = Int::from_i64(2).into();
        let right: Dynamic = Int::from_i64(3).into();

        let result = translate_plus(&left, &right).unwrap();
        assert!(result.as_int().is_some(), "Result should be Int");
    }

    #[test]
    fn test_translate_minus_int() {
        let left: Dynamic = Int::from_i64(5).into();
        let right: Dynamic = Int::from_i64(3).into();

        let result = translate_minus(&left, &right).unwrap();
        assert!(result.as_int().is_some(), "Result should be Int");
    }

    #[test]
    fn test_translate_times_int() {
        let left: Dynamic = Int::from_i64(4).into();
        let right: Dynamic = Int::from_i64(7).into();

        let result = translate_times(&left, &right).unwrap();
        assert!(result.as_int().is_some(), "Result should be Int");
    }

    #[test]
    fn test_translate_negate() {
        let arg: Dynamic = Int::from_i64(42).into();

        let result = translate_negate(&arg).unwrap();
        assert!(result.as_int().is_some(), "Result should be Int");
    }

    #[test]
    fn test_mixed_int_real_addition() {
        let left: Dynamic = Int::from_i64(2).into();
        let right: Dynamic = Real::from_real(3, 1).into(); // 3.0

        let result = translate_plus(&left, &right).unwrap();
        assert!(result.as_real().is_some(), "Mixed Int/Real should produce Real");
    }
}

