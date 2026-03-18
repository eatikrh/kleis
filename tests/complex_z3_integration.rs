//! Integration tests for Complex Number Z3 Integration
//!
//! Tests that complex number operations are correctly wired into the Z3 backend.

use kleis::solvers::z3::translators::complex::ComplexZ3;
use z3::ast::Real;
use z3::SatResult;

/// Test that i² = -1 is verified by Z3
#[test]
fn test_i_squared_equals_minus_one() {
    // Create i = complex(0, 1)
    let i = ComplexZ3::i();

    // Compute i² = i * i
    let i_squared = i.mul(&i);

    // Check that i² = -1 (i.e., complex(-1, 0))
    // This should give: real part = -1, imaginary part = 0
    let solver = z3::Solver::new();

    // Assert that the real part of i² is -1
    let minus_one = Real::from_rational(-1, 1);
    let re_constraint = i_squared.re.eq(&minus_one).not();
    solver.assert(re_constraint);

    // If UNSAT, then i²'s real part equals -1 (which is what we want)
    let result = solver.check();
    assert_eq!(
        result,
        SatResult::Unsat,
        "i² should have real part equal to -1"
    );

    // Also verify imaginary part is 0
    let solver2 = z3::Solver::new();
    let zero = Real::from_rational(0, 1);
    let im_constraint = i_squared.im.eq(&zero).not();
    solver2.assert(im_constraint);

    let result2 = solver2.check();
    assert_eq!(
        result2,
        SatResult::Unsat,
        "i² should have imaginary part equal to 0"
    );
}

/// Test complex addition: (1+2i) + (3+4i) = (4+6i)
#[test]
fn test_complex_addition_concrete() {
    let z1 = ComplexZ3::from_integers(1, 2);
    let z2 = ComplexZ3::from_integers(3, 4);
    let sum = z1.add(&z2);

    let solver = z3::Solver::new();

    // Check real part = 4
    let four = Real::from_rational(4, 1);
    solver.assert(sum.re.eq(&four).not());
    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Sum real part should be 4"
    );

    // Check imaginary part = 6
    let solver2 = z3::Solver::new();
    let six = Real::from_rational(6, 1);
    solver2.assert(sum.im.eq(&six).not());
    assert_eq!(
        solver2.check(),
        SatResult::Unsat,
        "Sum imaginary part should be 6"
    );
}

/// Test complex multiplication: (1+2i) * (3+4i) = (-5+10i)
/// Re: 1*3 - 2*4 = 3 - 8 = -5
/// Im: 1*4 + 2*3 = 4 + 6 = 10
#[test]
fn test_complex_multiplication_concrete() {
    let z1 = ComplexZ3::from_integers(1, 2);
    let z2 = ComplexZ3::from_integers(3, 4);
    let product = z1.mul(&z2);

    let solver = z3::Solver::new();

    // Check real part = -5
    let minus_five = Real::from_rational(-5, 1);
    solver.assert(product.re.eq(&minus_five).not());
    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Product real part should be -5"
    );

    // Check imaginary part = 10
    let solver2 = z3::Solver::new();
    let ten = Real::from_rational(10, 1);
    solver2.assert(product.im.eq(&ten).not());
    assert_eq!(
        solver2.check(),
        SatResult::Unsat,
        "Product imaginary part should be 10"
    );
}

/// Test complex conjugate: conj(3+4i) = 3-4i
#[test]
fn test_complex_conjugate() {
    let z = ComplexZ3::from_integers(3, 4);
    let z_conj = z.conj();

    let solver = z3::Solver::new();

    // Real part unchanged: 3
    let three = Real::from_rational(3, 1);
    solver.assert(z_conj.re.eq(&three).not());
    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Conjugate real part should be 3"
    );

    // Imaginary part negated: -4
    let solver2 = z3::Solver::new();
    let minus_four = Real::from_rational(-4, 1);
    solver2.assert(z_conj.im.eq(&minus_four).not());
    assert_eq!(
        solver2.check(),
        SatResult::Unsat,
        "Conjugate imaginary part should be -4"
    );
}

/// Test |z|² = z * conj(z) gives real result
/// For z = 3+4i: |z|² = 9 + 16 = 25
#[test]
fn test_magnitude_squared() {
    let z = ComplexZ3::from_integers(3, 4);
    let abs_sq = z.abs_squared();

    let solver = z3::Solver::new();
    let twentyfive = Real::from_rational(25, 1);
    solver.assert(abs_sq.eq(&twentyfive).not());

    assert_eq!(solver.check(), SatResult::Unsat, "|3+4i|² should equal 25");
}

/// Test double conjugate: conj(conj(z)) = z
#[test]
fn test_double_conjugate() {
    let z = ComplexZ3::from_integers(3, 4);
    let z_double_conj = z.conj().conj();

    let solver = z3::Solver::new();

    // Check equality via both parts
    solver.assert(z.eq_complex(&z_double_conj).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "conj(conj(z)) should equal z"
    );
}
