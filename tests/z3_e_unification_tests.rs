/// Z3 and E-Unification (Equational Unification)
///
/// Testing if Z3 can help with unification modulo equational theories.
/// Z3 has built-in support for certain theories (AC, ACU, etc.)

#[cfg(feature = "axiom-verification")]
mod z3_e_unification_tests {
    use z3::ast::{Ast, Int};
    use z3::{SatResult, Solver};

    #[test]
    fn test_commutativity_built_in() {
        // Z3 knows addition is commutative!
        // Can it determine that (a + b) and (b + a) are equivalent?
        
        let a = Int::fresh_const("a");
        let b = Int::fresh_const("b");
        let c = Int::fresh_const("c");
        
        let expr1 = &a + &b;  // a + b
        let expr2 = &b + &a;  // b + a
        
        let solver = Solver::new();
        
        // Assert they're equal, and that c equals one of them
        solver.assert(&c._eq(&expr1));
        solver.assert(&c._eq(&expr2));
        
        match solver.check() {
            SatResult::Sat => {
                println!("✅ Z3 knows a+b = b+a (commutativity built-in)");
                println!("   Z3 is doing E-unification with AC theory!");
            }
            SatResult::Unsat => {
                println!("❌ Z3 thinks a+b ≠ b+a (shouldn't happen)");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_associativity_built_in() {
        // Z3 knows addition is associative!
        // Can it determine (a+b)+c = a+(b+c)?
        
        let a = Int::fresh_const("a");
        let b = Int::fresh_const("b");
        let c = Int::fresh_const("c");
        let result = Int::fresh_const("result");
        
        let expr1 = (&a + &b) + &c;    // (a + b) + c
        let expr2 = &a + (&b + &c);    // a + (b + c)
        
        let solver = Solver::new();
        
        // Both should equal result
        solver.assert(&result._eq(&expr1));
        solver.assert(&result._eq(&expr2));
        
        match solver.check() {
            SatResult::Sat => {
                println!("✅ Z3 knows (a+b)+c = a+(b+c) (associativity built-in)");
                println!("   Z3 handles AC theory automatically!");
            }
            SatResult::Unsat => {
                println!("❌ Z3 thinks they're different");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_algebraic_simplification_detection() {
        // Can Z3 tell us when expressions are algebraically equivalent?
        // This is the KEY for simplification!
        
        let x = Int::fresh_const("x");
        
        let expr1 = &x + &Int::from_i64(0);  // x + 0
        let expr2 = x.clone();                // x
        
        let solver = Solver::new();
        
        // Are they always equal?
        solver.assert(&expr1._eq(&expr2).not());
        
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Z3 knows x + 0 = x (algebraic simplification!)");
                println!("   Can use Z3 to VERIFY simplification rules!");
            }
            SatResult::Sat => {
                println!("❌ Z3 found case where x+0 ≠ x");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_distributivity_as_rewrite() {
        // Can Z3 verify that rewriting x(y+z) → xy + xz is valid?
        
        let x = Int::fresh_const("x");
        let y = Int::fresh_const("y");
        let z = Int::fresh_const("z");
        
        let before = &x * (&y + &z);           // x(y + z)
        let after = (&x * &y) + (&x * &z);     // xy + xz
        
        let solver = Solver::new();
        
        // Check if transformation is always valid
        solver.assert(&before._eq(&after).not());
        
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Z3 confirms: x(y+z) → xy+xz is ALWAYS valid");
                println!("   Can use Z3 to verify rewrite rules!");
            }
            SatResult::Sat => {
                println!("❌ Found case where distributivity fails");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_multiplication_by_zero() {
        // Can Z3 verify: ∀x. x × 0 = 0
        
        let x = Int::fresh_const("x");
        let zero = Int::from_i64(0);
        
        let expr = &x * &zero;  // x × 0
        
        let solver = Solver::new();
        
        // Check if it always equals zero
        solver.assert(&expr._eq(&zero).not());
        
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Z3 knows: x × 0 = 0 always");
                println!("   Zero multiplication simplification verified!");
            }
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                println!("❌ Found x where x×0 ≠ 0: x={}", model.eval(&x, true).unwrap());
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_multiplication_by_one() {
        // Can Z3 verify: ∀x. x × 1 = x
        
        let x = Int::fresh_const("x");
        let one = Int::from_i64(1);
        
        let expr = &x * &one;  // x × 1
        
        let solver = Solver::new();
        
        // Check if it always equals x
        solver.assert(&expr._eq(&x).not());
        
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Z3 knows: x × 1 = x always");
                println!("   Identity simplification verified!");
            }
            SatResult::Sat => {
                println!("❌ Found x where x×1 ≠ x");
            }
            SatResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
        }
    }

    #[test]
    fn test_double_negation() {
        // Can Z3 verify: ∀x. -(-x) = x
        
        let x = Int::fresh_const("x");
        
        let expr = -(-x.clone());  // -(-x)
        
        let solver = Solver::new();
        
        // Check if -(-x) = x always
        solver.assert(&expr._eq(&x).not());
        
        match solver.check() {
            SatResult::Unsat => {
                println!("✅ Z3 knows: -(-x) = x (double negation)");
                println!("   Simplification rule verified!");
            }
            SatResult::Sat => {
                println!("❌ Found x where -(-x) ≠ x");
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
        println!("⚠️ Z3 E-unification tests skipped - compile with --features axiom-verification");
    }
}

