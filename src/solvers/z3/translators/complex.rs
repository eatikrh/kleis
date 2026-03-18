//! Complex Number Translation for Z3
//!
//! Z3 doesn't have native complex number support, so we encode ℂ as pairs of reals.
//!
//! Representation: z ∈ ℂ  →  (z_re : Real, z_im : Real)
//!
//! Operations are translated algebraically:
//! - z₁ + z₂ = (re₁ + re₂, im₁ + im₂)
//! - z₁ × z₂ = (re₁·re₂ - im₁·im₂, re₁·im₂ + im₁·re₂)
//! - conj(z) = (re, -im)
//! - |z|² = re² + im²
//! - i = (0, 1)

use std::collections::HashMap;
use z3::ast::{Bool, Dynamic, Real};

/// Represents a complex number as a pair of Z3 Real expressions
#[derive(Clone)]
pub struct ComplexZ3 {
    pub re: Real,
    pub im: Real,
}

impl ComplexZ3 {
    /// Create a new complex number from real and imaginary parts
    pub fn new(re: Real, im: Real) -> Self {
        Self { re, im }
    }

    /// Create a complex number from a real (imaginary part = 0)
    pub fn from_real(re: Real) -> Self {
        Self {
            re,
            im: Real::from_rational(0, 1),
        }
    }

    /// Create the imaginary unit i = (0, 1)
    pub fn i() -> Self {
        Self {
            re: Real::from_rational(0, 1),
            im: Real::from_rational(1, 1),
        }
    }

    /// Create zero = (0, 0)
    pub fn zero() -> Self {
        Self {
            re: Real::from_rational(0, 1),
            im: Real::from_rational(0, 1),
        }
    }

    /// Create one = (1, 0)
    pub fn one() -> Self {
        Self {
            re: Real::from_rational(1, 1),
            im: Real::from_rational(0, 1),
        }
    }

    /// Create a complex number from integer real and imaginary parts
    pub fn from_integers(re: i64, im: i64) -> Self {
        Self {
            re: Real::from_rational(re, 1),
            im: Real::from_rational(im, 1),
        }
    }

    /// Create a fresh complex variable (creates two real variables: name_re, name_im)
    pub fn fresh(name: &str) -> Self {
        Self {
            re: Real::fresh_const(&format!("{}_re", name)),
            im: Real::fresh_const(&format!("{}_im", name)),
        }
    }

    /// Create a named complex variable
    pub fn new_const(name: &str) -> Self {
        Self {
            re: Real::new_const(format!("{}_re", name)),
            im: Real::new_const(format!("{}_im", name)),
        }
    }

    /// Complex addition: z₁ + z₂ = (re₁ + re₂, im₁ + im₂)
    pub fn add(&self, other: &ComplexZ3) -> ComplexZ3 {
        ComplexZ3 {
            re: Real::add(&[&self.re, &other.re]),
            im: Real::add(&[&self.im, &other.im]),
        }
    }

    /// Complex subtraction: z₁ - z₂ = (re₁ - re₂, im₁ - im₂)
    pub fn sub(&self, other: &ComplexZ3) -> ComplexZ3 {
        ComplexZ3 {
            re: Real::sub(&[&self.re, &other.re]),
            im: Real::sub(&[&self.im, &other.im]),
        }
    }

    /// Complex multiplication: z₁ × z₂ = (re₁·re₂ - im₁·im₂, re₁·im₂ + im₁·re₂)
    pub fn mul(&self, other: &ComplexZ3) -> ComplexZ3 {
        // Real part: re₁·re₂ - im₁·im₂
        let re_re = Real::mul(&[&self.re, &other.re]);
        let im_im = Real::mul(&[&self.im, &other.im]);
        let real_part = Real::sub(&[&re_re, &im_im]);

        // Imaginary part: re₁·im₂ + im₁·re₂
        let re_im = Real::mul(&[&self.re, &other.im]);
        let im_re = Real::mul(&[&self.im, &other.re]);
        let imag_part = Real::add(&[&re_im, &im_re]);

        ComplexZ3 {
            re: real_part,
            im: imag_part,
        }
    }

    /// Complex division: z₁ / z₂
    /// = (re₁·re₂ + im₁·im₂) / (re₂² + im₂²) + i·(im₁·re₂ - re₁·im₂) / (re₂² + im₂²)
    pub fn div(&self, other: &ComplexZ3) -> ComplexZ3 {
        // Denominator: |z₂|² = re₂² + im₂²
        let denom = other.abs_squared();

        // Real part numerator: re₁·re₂ + im₁·im₂
        let re_re = Real::mul(&[&self.re, &other.re]);
        let im_im = Real::mul(&[&self.im, &other.im]);
        let real_num = Real::add(&[&re_re, &im_im]);

        // Imaginary part numerator: im₁·re₂ - re₁·im₂
        let im_re = Real::mul(&[&self.im, &other.re]);
        let re_im = Real::mul(&[&self.re, &other.im]);
        let imag_num = Real::sub(&[&im_re, &re_im]);

        ComplexZ3 {
            re: real_num.div(&denom),
            im: imag_num.div(&denom),
        }
    }

    /// Complex conjugate: conj(z) = (re, -im)
    pub fn conj(&self) -> ComplexZ3 {
        ComplexZ3 {
            re: self.re.clone(),
            im: self.im.unary_minus(),
        }
    }

    /// Squared magnitude: |z|² = re² + im²
    pub fn abs_squared(&self) -> Real {
        let re_sq = Real::mul(&[&self.re, &self.re]);
        let im_sq = Real::mul(&[&self.im, &self.im]);
        Real::add(&[&re_sq, &im_sq])
    }

    /// Negation: -z = (-re, -im)
    pub fn neg(&self) -> ComplexZ3 {
        ComplexZ3 {
            re: self.re.unary_minus(),
            im: self.im.unary_minus(),
        }
    }

    /// Multiplicative inverse: 1/z = conj(z) / |z|²
    /// = (re, -im) / (re² + im²)
    pub fn inverse(&self) -> ComplexZ3 {
        let abs_sq = self.abs_squared();
        ComplexZ3 {
            re: self.re.div(&abs_sq),
            im: self.im.unary_minus().div(&abs_sq),
        }
    }

    /// Check equality: z₁ = z₂ iff (re₁ = re₂ ∧ im₁ = im₂)
    pub fn eq_complex(&self, other: &ComplexZ3) -> Bool {
        let re_eq = self.re.eq(&other.re);
        let im_eq = self.im.eq(&other.im);
        Bool::and(&[&re_eq, &im_eq])
    }

    /// Convert to a pair of Dynamic values (re, im)
    pub fn to_dynamics(&self) -> (Dynamic, Dynamic) {
        (self.re.clone().into(), self.im.clone().into())
    }
}

/// Storage for complex variables during Z3 translation
/// Maps variable names to their ComplexZ3 representation
pub type ComplexVarMap = HashMap<String, ComplexZ3>;

/// Check if a type annotation indicates a complex number
pub fn is_complex_type(type_annotation: &str) -> bool {
    matches!(type_annotation, "ℂ" | "Complex" | "C")
}

/// Extract real part operation
pub fn translate_re(z: &ComplexZ3) -> Dynamic {
    z.re.clone().into()
}

/// Extract imaginary part operation
pub fn translate_im(z: &ComplexZ3) -> Dynamic {
    z.im.clone().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::SatResult;

    #[test]
    fn test_complex_i_squared() {
        let solver = z3::Solver::new();

        // i² = -1
        let i = ComplexZ3::i();
        let i_squared = i.mul(&i);

        // i² should have re = -1, im = 0
        let minus_one = Real::from_rational(-1, 1);
        let zero = Real::from_rational(0, 1);

        solver.assert(&i_squared.re._eq(&minus_one));
        solver.assert(&i_squared.im._eq(&zero));

        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn test_complex_conjugate_product() {
        let solver = z3::Solver::new();

        // z * conj(z) = |z|² (which is real, so im = 0)
        let z = ComplexZ3::fresh("z");
        let z_conj = z.conj();
        let product = z.mul(&z_conj);

        // Check that imaginary part is 0
        let zero = Real::from_rational(0, 1);
        solver.assert(&product.im._eq(&zero));

        // Check that real part equals |z|²
        let abs_sq = z.abs_squared();
        solver.assert(&product.re._eq(&abs_sq));

        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn test_complex_addition() {
        let solver = z3::Solver::new();

        // (1 + 2i) + (3 + 4i) = (4 + 6i)
        let z1 = ComplexZ3::new(Real::from_rational(1, 1), Real::from_rational(2, 1));
        let z2 = ComplexZ3::new(Real::from_rational(3, 1), Real::from_rational(4, 1));
        let sum = z1.add(&z2);

        solver.assert(&sum.re._eq(&Real::from_rational(4, 1)));
        solver.assert(&sum.im._eq(&Real::from_rational(6, 1)));

        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn test_complex_multiplication() {
        let solver = z3::Solver::new();

        // (1 + 2i) * (3 + 4i) = (1*3 - 2*4) + (1*4 + 2*3)i = -5 + 10i
        let z1 = ComplexZ3::new(Real::from_rational(1, 1), Real::from_rational(2, 1));
        let z2 = ComplexZ3::new(Real::from_rational(3, 1), Real::from_rational(4, 1));
        let product = z1.mul(&z2);

        solver.assert(&product.re._eq(&Real::from_rational(-5, 1)));
        solver.assert(&product.im._eq(&Real::from_rational(10, 1)));

        assert_eq!(solver.check(), SatResult::Sat);
    }
}
