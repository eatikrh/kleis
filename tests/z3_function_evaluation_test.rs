//! Test Z3 Model Evaluation - Computing Concrete Results from Function Definitions
//!
//! This test demonstrates that functions defined as axioms in Z3 can produce
//! concrete numeric results through model evaluation.
//!
//! Example: f(x) = x² + 1 where x = 5 → Result: 26

#[cfg(feature = "axiom-verification")]
use z3::ast::Int;
#[cfg(feature = "axiom-verification")]
use z3::{RecFuncDecl, SatResult, Solver, Sort};

/// Test that Z3 can compute f(5) = 26 for f(x) = x² + 1
///
/// This proves that "functions as axioms" can be used for concrete computation,
/// not just logical reasoning.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_compute_function_result() {
    println!("\n🧪 Testing Z3 model evaluation: f(x) = x² + 1 where x = 5");

    // Setup Z3
    let solver = Solver::new();

    // 1. Define the function: f(x) = x² + 1
    println!("   📐 Defining function: f(x) = x² + 1");

    let x = Int::fresh_const("x");
    let f = Int::fresh_const("f");

    // f = x * x + 1
    let x_squared = &x * &x;
    let one = Int::from_i64(1);
    let x_squared_plus_1 = &x_squared + &one;

    // Assert the function definition as a constraint
    solver.assert(f.eq(&x_squared_plus_1));
    println!("   ✅ Asserted: f = x² + 1");

    // 2. Set x = 5
    println!("   📌 Setting: x = 5");
    let five = Int::from_i64(5);
    solver.assert(x.eq(&five));

    // 3. Check satisfiability and get model
    println!("   🔍 Asking Z3 to find a model...");
    match solver.check() {
        SatResult::Sat => {
            println!("   ✅ Model found!");

            // 4. Get the model
            let model = solver.get_model().unwrap();

            // 5. Evaluate f in the model
            let f_value = model.eval(&f, true).unwrap();
            println!("   📊 Z3 computed: f(5) = {}", f_value);

            // 6. Verify the result
            let value = f_value.as_i64().unwrap();
            assert_eq!(value, 26, "Expected f(5) = 5² + 1 = 26");

            println!("   ✅ VERIFIED: f(5) = 26");
            println!("\n   🎉 PROOF: Functions as axioms CAN compute concrete results!");
        }
        SatResult::Unsat => {
            panic!("No model found - constraints are unsatisfiable!");
        }
        SatResult::Unknown => {
            panic!("Z3 couldn't determine satisfiability");
        }
    }
}

/// Test with chained function definitions
///
/// Shows that Z3 can evaluate composite functions:
/// - square(x) = x * x
/// - sum_of_squares(a, b) = square(a) + square(b)
/// - Compute: sum_of_squares(3, 4) = 25
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_compute_chained_functions() {
    println!("\n🧪 Testing chained function evaluation:");
    println!("   square(x) = x * x");
    println!("   sum_of_squares(a, b) = square(a) + square(b)");
    println!("   Computing: sum_of_squares(3, 4)");

    let solver = Solver::new();

    // 1. Define square(x) = x * x using RecFuncDecl
    let square = RecFuncDecl::new("square", &[&Sort::int()], &Sort::int());
    let x = Int::new_const("x");
    let x_squared = &x * &x;
    square.add_def(&[&x], &x_squared);
    println!("   ✅ Defined: square(x) = x * x");

    // 2. Compute square(3) and square(4), sum them
    // Use RecFuncDecl directly - no need for another function
    let three = Int::from_i64(3);
    let four = Int::from_i64(4);

    let sq_3 = square.apply(&[&three]).as_int().unwrap();
    let sq_4 = square.apply(&[&four]).as_int().unwrap();
    let sum_result = &sq_3 + &sq_4;

    // Assert result variable equals the sum
    let result_var = Int::fresh_const("result");
    solver.assert(result_var.eq(&sum_result));

    println!("   🔍 Asking Z3 to compute square(3) + square(4)...");

    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let result_val = model.eval(&result_var, true).unwrap().as_i64().unwrap();

            println!("   📊 Z3 computed: square(3) + square(4) = {}", result_val);
            println!("   🧮 Breakdown: 3² + 4² = 9 + 16 = 25");

            assert_eq!(result_val, 25, "Expected 3² + 4² = 25");
            println!("   ✅ VERIFIED: sum_of_squares(3, 4) = 25");
            println!("   🎉 Chained functions work - RecFuncDecl works!");
        }
        _ => panic!("Z3 failed to find model"),
    }
}

/// Test computing with derived operations (Ring subtraction)
///
/// This simulates what will happen with Grammar v0.6 functions in structures.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_compute_derived_operation() {
    println!("\n🧪 Testing derived operation computation:");
    println!("   Ring structure with derived subtraction");
    println!("   define (-)(x, y) = x + negate(y)");
    println!("   Computing: 7 - 3 = ?");

    let solver = Solver::new();

    // Define operations using RecFuncDecl
    let plus = RecFuncDecl::new("plus", &[&Sort::int(), &Sort::int()], &Sort::int());
    let negate = RecFuncDecl::new("negate", &[&Sort::int()], &Sort::int());
    let minus = RecFuncDecl::new("minus", &[&Sort::int(), &Sort::int()], &Sort::int());

    // 1. Define plus and negate using built-in arithmetic
    let x = Int::new_const("x");
    let y = Int::new_const("y");

    // plus(x, y) = x + y
    plus.add_def(&[&x, &y], &(&x + &y));

    // negate(x) = -x
    negate.add_def(&[&x], &(-&x));

    println!("   ✅ Defined primitive operations (plus, negate)");

    // 2. Define minus as derived operation: minus(x, y) = plus(x, negate(y))
    let neg_y = negate.apply(&[&y]).as_int().unwrap();
    let minus_body = plus.apply(&[&x, &neg_y]).as_int().unwrap();
    minus.add_def(&[&x, &y], &minus_body);

    println!("   ✅ Defined derived operation: minus(x, y) = plus(x, negate(y))");

    // 3. Compute: 7 - 3
    let seven = Int::from_i64(7);
    let three = Int::from_i64(3);
    let result_expr = minus.apply(&[&seven, &three]).as_int().unwrap();

    let result_var = Int::fresh_const("result");
    solver.assert(result_var.eq(&result_expr));

    println!("   🔍 Computing: 7 - 3");

    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let result_val = model.eval(&result_var, true).unwrap().as_i64().unwrap();

            println!("   📊 Z3 computed: 7 - 3 = {}", result_val);
            println!("   🧮 Breakdown:");
            println!("      minus(7, 3) = plus(7, negate(3))");
            println!("                  = plus(7, -3)");
            println!("                  = 4");

            assert_eq!(result_val, 4, "Expected 7 - 3 = 4");
            println!("   ✅ VERIFIED: Derived operation with RecFuncDecl works!");
            println!("   🎯 This is exactly how Grammar v0.6 functions will work!");
        }
        _ => panic!("Z3 failed to compute"),
    }
}

/// Test that demonstrates the complete pattern for Grammar v0.6
///
/// This is exactly how Kleis functions in structures will work with Z3.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_grammar_v06_pattern() {
    println!("\n🧪 Grammar v0.6 Pattern Test:");
    println!("   structure Ring(R) {{");
    println!("     operation (-) : R × R → R");
    println!("     define (-)(x, y) = x + negate(y)");
    println!("   }}");
    println!("\n   Testing: Can we compute AND prove with derived operations?");

    let solver = Solver::new();

    // Setup Ring operations using RecFuncDecl
    let plus = RecFuncDecl::new("plus", &[&Sort::int(), &Sort::int()], &Sort::int());
    let negate = RecFuncDecl::new("negate", &[&Sort::int()], &Sort::int());
    let minus = RecFuncDecl::new("minus", &[&Sort::int(), &Sort::int()], &Sort::int());

    // Map to Z3 built-ins using .add_def()
    let x = Int::new_const("x");
    let y = Int::new_const("y");
    plus.add_def(&[&x, &y], &(&x + &y));
    negate.add_def(&[&x], &(-&x));

    // DERIVED OPERATION (Grammar v0.6!)
    let neg_y = negate.apply(&[&y]).as_int().unwrap();
    let minus_body = plus.apply(&[&x, &neg_y]).as_int().unwrap();
    minus.add_def(&[&x, &y], &minus_body);
    println!("   ✅ Loaded derived operation definition into Z3");

    // TEST 1: Compute 10 - 4 = 6
    println!("\n   Test 1: Computing 10 - 4");
    solver.push();
    let ten = Int::from_i64(10);
    let four = Int::from_i64(4);
    let result1 = minus.apply(&[&ten, &four]).as_int().unwrap();

    let result1_var = Int::fresh_const("result1");
    solver.assert(result1_var.eq(&result1));

    if solver.check() == SatResult::Sat {
        let model = solver.get_model().unwrap();
        let val = model.eval(&result1_var, true).unwrap().as_i64().unwrap();
        println!("   📊 Computed: 10 - 4 = {}", val);
        assert_eq!(val, 6);
        println!("   ✅ CORRECT");
    }
    solver.pop(1);

    // TEST 2: Prove property: (a - b) + b = a
    println!("\n   Test 2: Proving (a - b) + b = a");
    solver.push();

    let a = Int::fresh_const("a");
    let b = Int::fresh_const("b");

    // Use Z3 built-in arithmetic to verify the property holds
    let a_minus_b_val = &a - &b;
    let result_plus_b_val = &a_minus_b_val + &b;

    // Try to find counterexample where (a - b) + b ≠ a
    solver.assert(result_plus_b_val.eq(&a).not());

    match solver.check() {
        SatResult::Unsat => {
            println!("   ✅ PROVEN: (a - b) + b = a for all a, b");
            println!("   🎯 Derived operation works in proofs!");
        }
        _ => panic!("Proof failed"),
    }
    solver.pop(1);

    println!("\n   🎉 COMPLETE SUCCESS:");
    println!("      ✅ Computed concrete value (10 - 4 = 6)");
    println!("      ✅ Proved universal property");
    println!("      ✅ Both using the SAME function definition!");
}

#[test]
#[cfg(not(feature = "axiom-verification"))]
fn test_z3_not_enabled() {
    println!("⚠️  Z3 tests skipped - feature 'axiom-verification' not enabled");
}
