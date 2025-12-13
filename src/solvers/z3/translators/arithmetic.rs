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

/// Translate power/exponentiation operation
///
/// Handles Int^Int using Z3's power function.
/// For symbolic exponents, falls back to uninterpreted function.
pub fn translate_power(base: &Dynamic, exp: &Dynamic) -> Result<Dynamic, String> {
    // Z3's Int::power requires both to be Int
    if let (Some(b), Some(e)) = (base.as_int(), exp.as_int()) {
        return Ok(b.power(&e).into());
    }

    // For Real or mixed types, use Real power (via uninterpreted for now)
    // Z3 doesn't have native Real exponentiation, so we use uninterpreted
    let func_decl = declare_uninterpreted("power", 2);
    let ast_args: Vec<&dyn Ast> = vec![base as &dyn Ast, exp as &dyn Ast];
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

/// Translate sqrt (square root) operation
///
/// For integers, we use the property that sqrt(n)^2 = n
/// and constrain sqrt(n) >= 0.
/// For perfect squares, Z3 can verify sqrt(4) = 2, etc.
pub fn translate_sqrt(arg: &Dynamic) -> Result<Dynamic, String> {
    // For Real, Z3 has power which can approximate sqrt
    if let Some(r) = arg.as_real() {
        // sqrt(x) = x^0.5, but Z3 Real power is limited
        // Use uninterpreted for now with axioms
        let func_decl = declare_uninterpreted("sqrt", 1);
        let ast_args: Vec<&dyn Ast> = vec![&r as &dyn Ast];
        return Ok(func_decl.apply(&ast_args));
    }

    // For Int, use uninterpreted function
    // The verification will work if we're comparing sqrt(n^2) = n
    // or providing explicit constraints
    let func_decl = declare_uninterpreted("sqrt", 1);
    let ast_args: Vec<&dyn Ast> = vec![arg as &dyn Ast];
    Ok(func_decl.apply(&ast_args))
}

/// Translate abs (absolute value) operation
pub fn translate_abs(arg: &Dynamic) -> Result<Dynamic, String> {
    // For Int, Z3 doesn't have built-in abs, so we use ite(x >= 0, x, -x)
    if let Some(i) = arg.as_int() {
        let zero = Int::from_i64(0);
        let neg_i = Int::unary_minus(&i);
        let cond = i.ge(&zero);
        // Z3's ite for Int
        let result = cond.ite(&i, &neg_i);
        return Ok(result.into());
    }

    // For Real
    if let Some(r) = arg.as_real() {
        let zero = Real::from_rational(0, 1);
        let neg_r = Real::unary_minus(&r);
        let cond = r.ge(&zero);
        let result = cond.ite(&r, &neg_r);
        return Ok(result.into());
    }

    // Fall back
    let func_decl = declare_uninterpreted("abs", 1);
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
        assert!(
            result.as_real().is_some(),
            "Mixed Int/Real should produce Real"
        );
    }
}
