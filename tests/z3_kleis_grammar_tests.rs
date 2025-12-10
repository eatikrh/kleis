/// Z3 Tests for Kleis Grammar Features
///
/// Exploring how Z3 can help verify Kleis language constructs:
/// - Matrix dimension rules
/// - Piecewise function consistency
/// - Type compatibility
/// - Logical operators
/// - Comparison operations

#[cfg(feature = "axiom-verification")]
mod z3_grammar_tests {
    use z3::ast::{Ast, Bool, Int};
    use z3::{SatResult, Solver};

    #[test]
    fn test_matrix_multiplication_dimensions() {
        // Matrix multiplication rule: (m×n) · (n×p) → (m×p)
        // Can Z3 verify dimension compatibility?

        let m = Int::fresh_const("m");
        let n1 = Int::fresh_const("n1");
        let n2 = Int::fresh_const("n2");
        let p = Int::fresh_const("p");

        // All dimensions must be positive
        let solver = Solver::new();
        solver.assert(&m.gt(&Int::from_i64(0)));
        solver.assert(&n1.gt(&Int::from_i64(0)));
        solver.assert(&n2.gt(&Int::from_i64(0)));
        solver.assert(&p.gt(&Int::from_i64(0)));

        // Matrix multiplication requires inner dimensions to match
        // Let's check: Can n1 ≠ n2 while still being valid?
        solver.assert(&n1._eq(&n2).not());

        match solver.check() {
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                println!("Found dimensions where n1 ≠ n2:");
                println!(
                    "  m={}, n1={}, n2={}, p={}",
                    model.eval(&m, true).unwrap(),
                    model.eval(&n1, true).unwrap(),
                    model.eval(&n2, true).unwrap(),
                    model.eval(&p, true).unwrap()
                );
                println!("✅ Z3 can find dimension mismatches!");
            }
            SatResult::Unsat => {
                println!("❌ Cannot have n1 ≠ n2 (weird)");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_piecewise_type_consistency() {
        // Piecewise: all cases must return same type
        // Example: f(x) = { expr1 if cond1, expr2 if cond2 }
        // Type checking: expr1 and expr2 must have same type

        // Simulate with integer ranges
        let expr1 = Int::fresh_const("expr1");
        let expr2 = Int::fresh_const("expr2");

        // Can we have a piecewise where expressions are "incompatible"?
        // Let's say expr1 is always even, expr2 is always odd
        let solver = Solver::new();
        solver.assert(&(&expr1 % &Int::from_i64(2))._eq(&Int::from_i64(0))); // expr1 even
        solver.assert(&(&expr2 % &Int::from_i64(2))._eq(&Int::from_i64(1))); // expr2 odd

        // But they should be "same type" (both integers) - this should be SAT
        match solver.check() {
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                println!("✅ Piecewise can have different values (same type):");
                println!(
                    "  expr1={} (even), expr2={} (odd)",
                    model.eval(&expr1, true).unwrap(),
                    model.eval(&expr2, true).unwrap()
                );
            }
            SatResult::Unsat => {
                println!("❌ Cannot have even and odd integers?");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_comparison_operators() {
        // Test the comparison operators we added today
        // less_than, greater_than, leq, geq

        let x = Int::fresh_const("x");
        let y = Int::fresh_const("y");

        let solver = Solver::new();

        // Claim: If x < y then NOT (x ≥ y)
        // This should always hold
        let x_lt_y = x.lt(&y);
        let x_ge_y = x.ge(&y);

        solver.assert(&Bool::and(&[&x_lt_y, &x_ge_y]));

        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Verified: x < y implies NOT (x ≥ y)");
            }
            SatResult::Sat => {
                println!("❌ Found case where x < y AND x ≥ y (impossible!)");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_logical_operators() {
        // Test logical_and, logical_or, logical_not we added today

        let p = Bool::fresh_const("p");
        let q = Bool::fresh_const("q");

        let solver = Solver::new();

        // Test De Morgan's law: ¬(p ∧ q) = (¬p) ∨ (¬q)
        let lhs = Bool::and(&[&p, &q]).not();
        let rhs = Bool::or(&[&p.not(), &q.not()]);

        // Check if they're equivalent by asserting inequality
        solver.assert(&lhs._eq(&rhs).not());

        match solver.check() {
            SatResult::Unsat => {
                println!("✅ De Morgan's law verified: ¬(p∧q) = (¬p)∨(¬q)");
            }
            SatResult::Sat => {
                println!("❌ De Morgan's law violated!");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_piecewise_condition_logic() {
        // Test that piecewise conditions can be verified
        // f(x) = { 0 if x < 0, 1 if x ≥ 0 }
        // These conditions should be exhaustive and non-overlapping

        let x = Int::fresh_const("x");
        let zero = Int::from_i64(0);

        let cond1 = x.lt(&zero); // x < 0
        let cond2 = x.ge(&zero); // x ≥ 0

        let solver = Solver::new();

        // Test 1: Conditions are exhaustive (one must be true)
        solver.push();
        solver.assert(&Bool::and(&[&cond1.not(), &cond2.not()]));
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Conditions are exhaustive (always one is true)");
            }
            SatResult::Sat => {
                println!("❌ Found case where neither condition holds");
            }
            _ => {}
        }
        solver.pop(1);

        // Test 2: Conditions don't overlap (can't both be true)
        solver.push();
        solver.assert(&Bool::and(&[&cond1, &cond2]));
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Conditions don't overlap (mutually exclusive)");
            }
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                println!("⚠️ Found overlap at x={}", model.eval(&x, true).unwrap());
            }
            _ => {}
        }
        solver.pop(1);
    }

    #[test]
    fn test_type_unification_simulation() {
        // Simulate type unification with Z3
        // Can we encode "Matrix(2,3) and Matrix(m,n) unify if m=2, n=3"?

        let m = Int::fresh_const("m");
        let n = Int::fresh_const("n");

        let solver = Solver::new();

        // Concrete matrix: Matrix(2, 3)
        // Type variable matrix: Matrix(m, n)
        // Unification should set m=2, n=3

        solver.assert(&m._eq(&Int::from_i64(2)));
        solver.assert(&n._eq(&Int::from_i64(3)));

        // Can these be satisfied?
        match solver.check() {
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                println!("✅ Type unification successful:");
                println!(
                    "  Matrix(2,3) unifies with Matrix({},{})",
                    model.eval(&m, true).unwrap(),
                    model.eval(&n, true).unwrap()
                );
            }
            SatResult::Unsat => {
                println!("❌ Cannot unify (shouldn't happen for 2=2, 3=3)");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_dimension_mismatch_detection() {
        // Can Z3 detect dimension mismatches we catch in type checker?
        // Example: Matrix(2,2) vs Matrix(3,3) - cannot unify!

        let m1 = Int::from_i64(2);
        let n1 = Int::from_i64(2);
        let m2 = Int::from_i64(3);
        let n2 = Int::from_i64(3);

        let solver = Solver::new();

        // Try to unify Matrix(2,2) with Matrix(3,3)
        // This requires m1=m2 AND n1=n2
        solver.assert(&m1._eq(&m2));
        solver.assert(&n1._eq(&n2));

        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Z3 correctly detects: Matrix(2,2) ≠ Matrix(3,3)");
                println!("   Cannot unify different dimensions!");
            }
            SatResult::Sat => {
                println!("❌ Z3 thinks 2=3 (impossible!)");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }
}

#[cfg(not(feature = "axiom-verification"))]
mod placeholder {
    #[test]
    fn test_z3_feature_disabled() {
        println!("⚠️ Z3 grammar tests skipped - compile with --features axiom-verification");
    }
}
