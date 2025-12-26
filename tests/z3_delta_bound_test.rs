//! Test Z3 on the δ-bound approach to FLT
//!
//! Key inequality: For 0 < a < b, prove b³ - a³ > (b-1)³
//! This is equivalent to: δ = (b³-a³)^(1/3) - b > -1

use kleis::ast::Expression;
use kleis::solvers::backend::{SatisfiabilityResult, SolverBackend};
use kleis::solvers::z3::backend::Z3Backend;
use kleis::structure_registry::StructureRegistry;

/// Test: Is b³ - a³ > (b-1)³ always true for 0 < a < b?
/// We ask Z3 to find a counterexample where b³ - a³ ≤ (b-1)³
#[test]
fn test_delta_lower_bound_n3() {
    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).expect("Z3 backend");

    // Build: 0 < a < b ∧ b³ - a³ ≤ (b-1)³
    // If UNSAT, the bound holds!

    // a > 0
    let a_pos = Expression::Operation {
        name: "gt".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Const("0".to_string()),
        ],
        span: None,
    };

    // a < b
    let a_lt_b = Expression::Operation {
        name: "lt".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Object("b".to_string()),
        ],
        span: None,
    };

    // b³ - a³ ≤ (b-1)³  (negation of what we want to prove)
    // Expand: b³ - a³ ≤ b³ - 3b² + 3b - 1
    //         -a³ ≤ -3b² + 3b - 1
    //         a³ ≥ 3b² - 3b + 1

    let a_cubed = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Const("3".to_string()),
        ],
        span: None,
    };

    // 3b² - 3b + 1
    let b_sq = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("b".to_string()),
            Expression::Const("2".to_string()),
        ],
        span: None,
    };
    let three = Expression::Const("3".to_string());
    let one = Expression::Const("1".to_string());
    let three_b_sq = Expression::Operation {
        name: "multiply".to_string(),
        args: vec![three.clone(), b_sq],
        span: None,
    };
    let three_b = Expression::Operation {
        name: "multiply".to_string(),
        args: vec![three, Expression::Object("b".to_string())],
        span: None,
    };
    let rhs = Expression::Operation {
        name: "subtract".to_string(),
        args: vec![
            Expression::Operation {
                name: "add".to_string(),
                args: vec![three_b_sq, one],
                span: None,
            },
            three_b,
        ],
        span: None,
    };

    // a³ ≥ 3b² - 3b + 1
    let counterex = Expression::Operation {
        name: "geq".to_string(),
        args: vec![a_cubed, rhs],
        span: None,
    };

    // Combine: a_pos ∧ a_lt_b ∧ counterex
    let and1 = Expression::Operation {
        name: "and".to_string(),
        args: vec![a_pos, a_lt_b],
        span: None,
    };
    let expr = Expression::Operation {
        name: "and".to_string(),
        args: vec![and1, counterex],
        span: None,
    };

    println!("Testing δ > -1 bound for n=3");
    println!("Query: ∃ a,b : 0 < a < b ∧ a³ ≥ 3b² - 3b + 1");

    let result = backend.check_satisfiability(&expr);

    match &result {
        Ok(SatisfiabilityResult::Unsatisfiable) => {
            println!("✓ UNSAT: δ > -1 holds for all 0 < a < b!");
            println!("  This proves: (b³-a³)^(1/3) > b-1 always");
        }
        Ok(SatisfiabilityResult::Satisfiable { example }) => {
            println!("✗ SAT: Found counterexample!");
            println!("  {}", example);
            println!("  The bound δ > -1 does NOT hold universally");
        }
        Ok(SatisfiabilityResult::Unknown) => {
            println!("? Unknown - Z3 couldn't decide");
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
