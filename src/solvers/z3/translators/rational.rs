//! Rational Number Translation for Z3
//!
//! Z3 has native support for rationals via its Real sort (which is actually ℚ, not ℝ).
//! We use this directly, encoding rationals as Z3 Real values.
//!
//! Representation: r ∈ ℚ → Real (Z3's Real is actually rational)
//!
//! Operations are translated directly:
//! - r₁ + r₂ = Z3 Real addition
//! - r₁ × r₂ = Z3 Real multiplication
//! - r₁ / r₂ = Z3 Real division (requires r₂ ≠ 0)
//! - -r = Z3 Real negation
//!
//! Note: Z3's "Real" sort is actually the rationals ℚ, not the reals ℝ.
//! This is perfect for our rational number implementation.

use z3::ast::{Bool, Int, Real};

/// Represents a rational number as a Z3 Real expression
/// (Z3 Real is actually ℚ, the rationals)
#[derive(Clone)]
pub struct RationalZ3 {
    pub value: Real,
}

impl RationalZ3 {
    /// Create a new rational from a Z3 Real
    pub fn new(value: Real) -> Self {
        Self { value }
    }

    /// Create a rational from numerator and denominator integers
    pub fn from_fraction(numer: i64, denom: i64) -> Self {
        Self {
            value: Real::from_rational(numer, denom),
        }
    }

    /// Create a rational from an integer (denominator = 1)
    pub fn from_int(n: i64) -> Self {
        Self {
            value: Real::from_rational(n, 1),
        }
    }

    /// Create zero = 0/1
    pub fn zero() -> Self {
        Self::from_fraction(0, 1)
    }

    /// Create one = 1/1
    pub fn one() -> Self {
        Self::from_fraction(1, 1)
    }

    /// Create a fresh rational variable
    pub fn fresh(name: &str) -> Self {
        Self {
            value: Real::fresh_const(name),
        }
    }

    /// Create a named rational constant
    pub fn new_const(name: &str) -> Self {
        Self {
            value: Real::new_const(name),
        }
    }

    /// Rational addition: r₁ + r₂
    pub fn add(&self, other: &RationalZ3) -> RationalZ3 {
        RationalZ3 {
            value: Real::add(&[&self.value, &other.value]),
        }
    }

    /// Rational subtraction: r₁ - r₂
    pub fn sub(&self, other: &RationalZ3) -> RationalZ3 {
        RationalZ3 {
            value: Real::sub(&[&self.value, &other.value]),
        }
    }

    /// Rational multiplication: r₁ × r₂
    pub fn mul(&self, other: &RationalZ3) -> RationalZ3 {
        RationalZ3 {
            value: Real::mul(&[&self.value, &other.value]),
        }
    }

    /// Rational division: r₁ / r₂ (assumes r₂ ≠ 0)
    pub fn div(&self, other: &RationalZ3) -> RationalZ3 {
        RationalZ3 {
            value: Real::div(&self.value, &other.value),
        }
    }

    /// Rational negation: -r
    pub fn neg(&self) -> RationalZ3 {
        RationalZ3 {
            value: Real::unary_minus(&self.value),
        }
    }

    /// Absolute value: |r|
    pub fn abs(&self) -> RationalZ3 {
        let zero = RationalZ3::zero();
        let neg = self.neg();
        // if r >= 0 then r else -r
        RationalZ3 {
            value: self.value.ge(&zero.value).ite(&self.value, &neg.value),
        }
    }

    /// Reciprocal: 1/r (assumes r ≠ 0)
    pub fn inv(&self) -> RationalZ3 {
        let one = RationalZ3::one();
        one.div(self)
    }

    /// Less than: r₁ < r₂
    pub fn lt(&self, other: &RationalZ3) -> Bool {
        self.value.lt(&other.value)
    }

    /// Less than or equal: r₁ ≤ r₂
    pub fn le(&self, other: &RationalZ3) -> Bool {
        self.value.le(&other.value)
    }

    /// Greater than: r₁ > r₂
    pub fn gt(&self, other: &RationalZ3) -> Bool {
        self.value.gt(&other.value)
    }

    /// Greater than or equal: r₁ ≥ r₂
    pub fn ge(&self, other: &RationalZ3) -> Bool {
        self.value.ge(&other.value)
    }

    /// Equality: r₁ = r₂
    pub fn eq(&self, other: &RationalZ3) -> Bool {
        self.value.eq(&other.value)
    }

    /// Convert to Z3 Dynamic
    pub fn to_dynamic(&self) -> z3::ast::Dynamic {
        z3::ast::Dynamic::from_ast(&self.value)
    }

    /// Get the underlying Z3 Real
    pub fn as_real(&self) -> &Real {
        &self.value
    }
}

/// Translator for rational number operations
pub struct RationalTranslator;

impl RationalTranslator {
    /// Translate rational(numer, denom) constructor
    pub fn translate_constructor(
        numer: &z3::ast::Dynamic,
        denom: &z3::ast::Dynamic,
    ) -> Result<z3::ast::Dynamic, String> {
        // Convert arguments to Real (they might be Int)
        let numer_real = if let Some(int_val) = numer.as_int() {
            Int::to_real(&int_val)
        } else if let Some(real_val) = numer.as_real() {
            real_val
        } else {
            return Err("rational: numerator must be numeric".to_string());
        };

        let denom_real = if let Some(int_val) = denom.as_int() {
            Int::to_real(&int_val)
        } else if let Some(real_val) = denom.as_real() {
            real_val
        } else {
            return Err("rational: denominator must be numeric".to_string());
        };

        // rational(p, q) = p / q
        let result = Real::div(&numer_real, &denom_real);
        Ok(z3::ast::Dynamic::from_ast(&result))
    }

    /// Translate numer(r) - get numerator
    /// Note: Z3 doesn't expose numerator/denominator directly for Real,
    /// so we use uninterpreted functions for symbolic reasoning
    pub fn translate_numer(r: &z3::ast::Dynamic) -> Result<z3::ast::Dynamic, String> {
        // For concrete rationals, we could extract it
        // For symbolic, we use an uninterpreted function
        let numer_func = z3::FuncDecl::new("numer", &[&z3::Sort::real()], &z3::Sort::int());
        let result = numer_func.apply(&[r]);
        Ok(result)
    }

    /// Translate denom(r) - get denominator
    pub fn translate_denom(r: &z3::ast::Dynamic) -> Result<z3::ast::Dynamic, String> {
        let denom_func = z3::FuncDecl::new("denom", &[&z3::Sort::real()], &z3::Sort::int());
        let result = denom_func.apply(&[r]);
        Ok(result)
    }

    /// Translate int_to_rational(n)
    pub fn translate_int_to_rational(n: &z3::ast::Dynamic) -> Result<z3::ast::Dynamic, String> {
        if let Some(int_val) = n.as_int() {
            let real_val = Int::to_real(&int_val);
            Ok(z3::ast::Dynamic::from_ast(&real_val))
        } else if let Some(real_val) = n.as_real() {
            // Already a real
            Ok(z3::ast::Dynamic::from_ast(&real_val))
        } else {
            Err("int_to_rational: argument must be integer".to_string())
        }
    }

    /// Translate to_real(r) - rational to real (identity in Z3)
    pub fn translate_to_real(r: &z3::ast::Dynamic) -> Result<z3::ast::Dynamic, String> {
        // In Z3, Real IS Rational, so this is identity
        Ok(r.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::ast::Ast;

    #[test]
    fn test_rational_operations() {
        let half = RationalZ3::from_fraction(1, 2);
        let third = RationalZ3::from_fraction(1, 3);

        // 1/2 + 1/3 should work
        let sum = half.add(&third);

        // 1/2 * 1/3 = 1/6
        let product = half.mul(&third);

        // 1/2 / 1/3 = 3/2
        let quotient = half.div(&third);

        // These are valid Z3 expressions
        assert!(sum.value.get_sort() == z3::Sort::real());
        assert!(product.value.get_sort() == z3::Sort::real());
        assert!(quotient.value.get_sort() == z3::Sort::real());
    }

    #[test]
    fn test_rational_comparison() {
        let half = RationalZ3::from_fraction(1, 2);
        let third = RationalZ3::from_fraction(1, 3);

        // 1/3 < 1/2
        let lt = third.lt(&half);

        // Create a solver to check
        let solver = z3::Solver::new();
        solver.assert(&lt);

        assert_eq!(solver.check(), z3::SatResult::Sat);
    }
}
