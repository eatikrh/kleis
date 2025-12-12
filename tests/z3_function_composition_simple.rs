//! Simple Z3 Function Composition Test
//!
//! Demonstrates that Z3 can handle functions using other functions.
//! This test uses a simplified approach that avoids quantifier complexities.

#![allow(clippy::needless_borrows_for_generic_args)]

#[cfg(feature = "axiom-verification")]
use z3::ast::Int;
#[cfg(feature = "axiom-verification")]
use z3::{SatResult, Solver};

/// Test: f(x) = x² + 1, compute f(5) = 26
/// Then: g(y) = 2 * f_result, where f_result = 26
/// Compute: g should be 52
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_sequential_function_computation() {
    println!("\n🧪 Testing: Sequential function computation");
    println!("   Step 1: f(x) = x² + 1, compute f(5)");
    println!("   Step 2: g = 2 * f_result, compute g");

    let solver = Solver::new();

    // Step 1: Compute f(5) where f(x) = x² + 1
    println!("\n   Step 1: Computing f(5)");
    let x = Int::fresh_const("x");
    let f = Int::fresh_const("f");

    let x_squared = &x * &x;
    let one = Int::from_i64(1);
    let five = Int::from_i64(5);

    // f = x² + 1 AND x = 5
    solver.assert(&f.eq(&(&x_squared + &one)));
    solver.assert(&x.eq(&five));

    let f_value = if solver.check() == SatResult::Sat {
        let model = solver.get_model().unwrap();
        let result = model.eval(&f, true).unwrap();
        result.as_i64().unwrap()
    } else {
        panic!("Failed to compute f(5)");
    };

    println!("   📊 Computed: f(5) = {}", f_value);
    assert_eq!(f_value, 26);
    println!("   ✅ f(5) = 26");

    // Step 2: Compute g = 2 * f_result
    println!("\n   Step 2: Computing g = 2 * f_result");
    let solver2 = Solver::new();

    let g = Int::fresh_const("g");
    let two = Int::from_i64(2);
    let f_const = Int::from_i64(f_value); // Use the computed value

    // g = 2 * 26
    solver2.assert(&g.eq(&(&two * &f_const)));

    let g_value = if solver2.check() == SatResult::Sat {
        let model = solver2.get_model().unwrap();
        let result = model.eval(&g, true).unwrap();
        result.as_i64().unwrap()
    } else {
        panic!("Failed to compute g");
    };

    println!("   📊 Computed: g = 2 * {} = {}", f_value, g_value);
    assert_eq!(g_value, 52);
    println!("   ✅ g = 52");

    println!("\n   🎉 SUCCESS: Used f's result (26) in g's computation!");
    println!("      f(5) = 26 ✅");
    println!("      g = 2 * f(5) = 52 ✅");
}

/// Test: Can Z3 reason about function composition in a single model?
/// f(x) = x + 10, compute f(5) and f(7) in same model
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_multiple_function_evaluations() {
    println!("\n🧪 Testing: Multiple function evaluations in one model");
    println!("   f(x) = x + 10");
    println!("   Computing: f(5) and f(7)");

    let solver = Solver::new();

    // Create separate variables for each evaluation
    let x1 = Int::fresh_const("x1");
    let f1 = Int::fresh_const("f1");
    let x2 = Int::fresh_const("x2");
    let f2 = Int::fresh_const("f2");

    let ten = Int::from_i64(10);

    // f1 = x1 + 10, x1 = 5
    solver.assert(&f1.eq(&(&x1 + &ten)));
    solver.assert(&x1.eq(&Int::from_i64(5)));

    // f2 = x2 + 10, x2 = 7
    solver.assert(&f2.eq(&(&x2 + &ten)));
    solver.assert(&x2.eq(&Int::from_i64(7)));

    println!("   🔍 Computing f(5) and f(7) in same model...");

    if solver.check() == SatResult::Sat {
        let model = solver.get_model().unwrap();

        let f1_val = model.eval(&f1, true).unwrap().as_i64().unwrap();
        let f2_val = model.eval(&f2, true).unwrap().as_i64().unwrap();

        println!("   📊 Z3 computed:");
        println!("      f(5) = {}", f1_val);
        println!("      f(7) = {}", f2_val);

        assert_eq!(f1_val, 15, "Expected f(5) = 15");
        assert_eq!(f2_val, 17, "Expected f(7) = 17");

        println!("   ✅ VERIFIED: Multiple evaluations work!");
        println!("   🎯 Same function, different inputs, same model!");
    } else {
        panic!("Z3 failed");
    }
}

/// Test: Pythagorean theorem - c² = a² + b²
/// square(x) = x², sum(a, b) = a + b
/// For a=3, b=4, prove c²=25, so c=5
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_pythagorean_with_functions() {
    println!("\n🧪 Testing: Pythagorean theorem with functions");
    println!("   square(x) = x²");
    println!("   For triangle: a=3, b=4");
    println!("   Find c where c² = a² + b²");

    let solver = Solver::new();

    // Define values
    let a = Int::fresh_const("a");
    let b = Int::fresh_const("b");
    let c = Int::fresh_const("c");

    let a_squared = Int::fresh_const("a_squared");
    let b_squared = Int::fresh_const("b_squared");
    let c_squared = Int::fresh_const("c_squared");

    // Set a=3, b=4
    solver.assert(&a.eq(&Int::from_i64(3)));
    solver.assert(&b.eq(&Int::from_i64(4)));

    // Define squares
    solver.assert(&a_squared.eq(&(&a * &a)));
    solver.assert(&b_squared.eq(&(&b * &b)));
    solver.assert(&c_squared.eq(&(&c * &c)));

    // Pythagorean: c² = a² + b²
    solver.assert(&c_squared.eq(&(&a_squared + &b_squared)));

    println!("   ✅ Defined: c² = a² + b²");
    println!("   🔍 Finding c where c² = 3² + 4²...");

    if solver.check() == SatResult::Sat {
        let model = solver.get_model().unwrap();

        let a_sq_val = model.eval(&a_squared, true).unwrap().as_i64().unwrap();
        let b_sq_val = model.eval(&b_squared, true).unwrap().as_i64().unwrap();
        let c_sq_val = model.eval(&c_squared, true).unwrap().as_i64().unwrap();
        let c_val = model.eval(&c, true).unwrap().as_i64().unwrap();

        println!("   📊 Z3 computed:");
        println!("      a² = {}", a_sq_val);
        println!("      b² = {}", b_sq_val);
        println!(
            "      c² = a² + b² = {} + {} = {}",
            a_sq_val, b_sq_val, c_sq_val
        );
        println!("      c = {}", c_val);

        assert_eq!(a_sq_val, 9, "Expected 3² = 9");
        assert_eq!(b_sq_val, 16, "Expected 4² = 16");
        assert_eq!(c_sq_val, 25, "Expected c² = 25");
        assert_eq!(c_val.abs(), 5, "Expected c = 5 (or -5)");

        println!("   ✅ VERIFIED: Pythagorean theorem!");
        println!("   🎯 Functions compose correctly!");
    } else {
        panic!("Z3 failed");
    }
}

#[test]
#[cfg(not(feature = "axiom-verification"))]
fn test_z3_not_enabled() {
    println!("⚠️  Z3 tests skipped - feature 'axiom-verification' not enabled");
}
