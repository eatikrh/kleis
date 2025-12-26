//! Z3 exploration of Fermat's Last Theorem
//! Tests if Z3 can find counterexamples to FLT for small bounds

use kleis::ast::Expression;
use kleis::solvers::backend::{SatisfiabilityResult, SolverBackend};
use kleis::solvers::z3::backend::Z3Backend;
use kleis::structure_registry::StructureRegistry;

/// Build: a³ + b³ = c³ ∧ a > 0 ∧ b > 0 ∧ c > 0
fn build_flt_n3() -> Expression {
    let a_cubed = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Const("3".to_string()),
        ],
        span: None,
    };
    let b_cubed = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("b".to_string()),
            Expression::Const("3".to_string()),
        ],
        span: None,
    };
    let c_cubed = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("c".to_string()),
            Expression::Const("3".to_string()),
        ],
        span: None,
    };
    let sum = Expression::Operation {
        name: "add".to_string(),
        args: vec![a_cubed, b_cubed],
        span: None,
    };
    let eq = Expression::Operation {
        name: "equals".to_string(),
        args: vec![sum, c_cubed],
        span: None,
    };

    // Add positivity constraints: a > 0 ∧ b > 0 ∧ c > 0
    let zero = Expression::Const("0".to_string());
    let a_pos = Expression::Operation {
        name: "gt".to_string(),
        args: vec![Expression::Object("a".to_string()), zero.clone()],
        span: None,
    };
    let b_pos = Expression::Operation {
        name: "gt".to_string(),
        args: vec![Expression::Object("b".to_string()), zero.clone()],
        span: None,
    };
    let c_pos = Expression::Operation {
        name: "gt".to_string(),
        args: vec![Expression::Object("c".to_string()), zero],
        span: None,
    };

    // Combine: ((eq ∧ a_pos) ∧ b_pos) ∧ c_pos
    let and1 = Expression::Operation {
        name: "and".to_string(),
        args: vec![eq, a_pos],
        span: None,
    };
    let and2 = Expression::Operation {
        name: "and".to_string(),
        args: vec![and1, b_pos],
        span: None,
    };
    Expression::Operation {
        name: "and".to_string(),
        args: vec![and2, c_pos],
        span: None,
    }
}

/// Test: Can Z3 find integer solutions to a³ + b³ = c³?
#[test]
#[ignore] // Long-running Z3 satisfiability check
fn test_z3_flt_n3_satisfiability() {
    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).expect("Z3 backend creation failed");

    let expr = build_flt_n3();
    println!("Testing FLT n=3: a³ + b³ = c³");
    println!("AST: {:?}", expr);

    let result = backend.check_satisfiability(&expr);
    println!("Z3 result: {:?}", result);

    match result {
        Ok(SatisfiabilityResult::Satisfiable { example }) => {
            println!("Z3 found: {:?}", example);
            println!("Note: May include trivial solutions with zeros");
        }
        Ok(SatisfiabilityResult::Unsatisfiable) => {
            println!("✓ Z3 found no solutions");
        }
        Ok(SatisfiabilityResult::Unknown) => {
            println!("Z3 returned Unknown");
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Test n=4: a⁴ + b⁴ = c⁴
#[test]
#[ignore] // Long-running Z3 satisfiability check
fn test_z3_flt_n4_satisfiability() {
    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).expect("Z3 backend creation failed");

    let a4 = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Const("4".to_string()),
        ],
        span: None,
    };
    let b4 = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("b".to_string()),
            Expression::Const("4".to_string()),
        ],
        span: None,
    };
    let c4 = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("c".to_string()),
            Expression::Const("4".to_string()),
        ],
        span: None,
    };
    let sum = Expression::Operation {
        name: "add".to_string(),
        args: vec![a4, b4],
        span: None,
    };
    let expr = Expression::Operation {
        name: "equals".to_string(),
        args: vec![sum, c4],
        span: None,
    };

    println!("Testing FLT n=4: a⁴ + b⁴ = c⁴");
    let result = backend.check_satisfiability(&expr);
    println!("Z3 result: {:?}", result);
}
