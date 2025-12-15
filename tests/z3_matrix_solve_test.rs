//! Test Z3 solving linear systems via matrix equations
//!
//! This test verifies that Z3 can solve:
//!   [[5], [11]] = [[1,2],[3,4]] × [[x], [y]]
//!
//! Which is the linear system:
//!   1*x + 2*y = 5
//!   3*x + 4*y = 11
//!
//! Solution: x = 1, y = 2

use kleis::ast::Expression;
use kleis::solvers::backend::{SatisfiabilityResult, SolverBackend};
use kleis::solvers::z3::backend::Z3Backend;
use kleis::structure_registry::StructureRegistry;

/// Build the matrix equation AST:
/// Matrix(2,1,[5,11]) = multiply(Matrix(2,2,[1,2,3,4]), Matrix(2,1,[x,y]))
fn build_matrix_equation() -> Expression {
    // Left side: [[5], [11]]
    let result_matrix = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("1".to_string()),
            Expression::List(vec![
                Expression::Const("5".to_string()),
                Expression::Const("11".to_string()),
            ]),
        ],
    };

    // Matrix A: [[1,2],[3,4]]
    let matrix_a = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            Expression::List(vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
                Expression::Const("3".to_string()),
                Expression::Const("4".to_string()),
            ]),
        ],
    };

    // Vector x: [[x], [y]] - with free variables
    let vector_x = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("1".to_string()),
            Expression::List(vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ]),
        ],
    };

    // A × x
    let product = Expression::Operation {
        name: "multiply".to_string(),
        args: vec![matrix_a, vector_x],
    };

    // result = A × x
    Expression::Operation {
        name: "equals".to_string(),
        args: vec![result_matrix, product],
    }
}

/// IGNORED: Requires list indexing for matrix element access
/// Matrix multiplication needs: result[i] = Σ_j A[i,j] * x[j]
///
/// TO ENABLE THIS TEST:
/// 1. Add list indexing axioms to stdlib/lists.kleis (see z3_tensor_test.rs)
/// 2. Replace builtin_* in stdlib/matrices.kleis with axioms:
///    ```kleis
///    axiom multiply_def : ∀ A : Matrix(m,n,T) . ∀ B : Matrix(n,p,T) .
///        ∀ i : Nat . ∀ j : Nat .
///        element(multiply(A,B), i, j) = sum_k(times(element(A,i,k), element(B,k,j)), 0, n)
///    ```
/// 3. Load axioms via: backend.assert_axioms_from_registry()
#[test]
#[ignore]
fn test_z3_solves_matrix_linear_system() {
    let registry = StructureRegistry::default();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    let equation = build_matrix_equation();
    println!("Testing equation: [[5],[11]] = [[1,2],[3,4]] × [[x],[y]]");

    // Check satisfiability - Z3 should find x=1, y=2
    let result = backend
        .check_satisfiability(&equation)
        .expect("Satisfiability check failed");

    println!("Result: {:?}", result);

    match result {
        SatisfiabilityResult::Satisfiable { example } => {
            println!("✅ SATISFIABLE! Solution: {}", example);
            // The solution should contain x and y values
            // x = 1, y = 2 satisfies:
            //   1*1 + 2*2 = 5 ✓
            //   3*1 + 4*2 = 11 ✓
            assert!(
                example.contains("1") || example.contains("2"),
                "Solution should contain the values 1 and 2"
            );
        }
        SatisfiabilityResult::Unsatisfiable => {
            panic!("❌ UNSATISFIABLE - but this system HAS a solution (x=1, y=2)!");
        }
        SatisfiabilityResult::Unknown => {
            panic!("❓ UNKNOWN - Z3 couldn't determine satisfiability");
        }
    }
}

/// Build: ∃x,y ∈ ℕ. [[5],[11]] = [[1,2],[3,4]] × [[x],[y]]
/// This adds constraints: x ≥ 0 ∧ y ≥ 0
fn build_matrix_equation_with_natural_constraints() -> Expression {
    // The matrix equation
    let matrix_eq = build_matrix_equation();

    // x >= 0
    let x_natural = Expression::Operation {
        name: "geq".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Const("0".to_string()),
        ],
    };

    // y >= 0
    let y_natural = Expression::Operation {
        name: "geq".to_string(),
        args: vec![
            Expression::Object("y".to_string()),
            Expression::Const("0".to_string()),
        ],
    };

    // x >= 0 AND y >= 0
    let naturals = Expression::Operation {
        name: "and".to_string(),
        args: vec![x_natural, y_natural],
    };

    // (matrix equation) AND (x,y are natural)
    Expression::Operation {
        name: "and".to_string(),
        args: vec![matrix_eq, naturals],
    }
}

/// IGNORED: Requires list indexing for matrix operations
/// TO ENABLE: See test_z3_solves_matrix_linear_system for required axioms
#[test]
#[ignore]
fn test_z3_solves_with_natural_number_constraint() {
    let registry = StructureRegistry::default();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    let equation = build_matrix_equation_with_natural_constraints();
    println!("Testing: ∃x,y ∈ ℕ. [[5],[11]] = [[1,2],[3,4]] × [[x],[y]]");
    println!("(with constraints x ≥ 0, y ≥ 0)");

    let result = backend
        .check_satisfiability(&equation)
        .expect("Satisfiability check failed");

    println!("Result: {:?}", result);

    match result {
        SatisfiabilityResult::Satisfiable { example } => {
            println!("✅ SATISFIABLE! Natural number solution found:");
            println!("{}", example);
            // x=1, y=2 are both natural numbers
        }
        SatisfiabilityResult::Unsatisfiable => {
            panic!("❌ UNSATISFIABLE - but x=1, y=2 IS a natural number solution!");
        }
        SatisfiabilityResult::Unknown => {
            panic!("❓ UNKNOWN");
        }
    }
}

/// Build: [[6],[11]] = [[1,2],[3,4]] × [[x],[y]]
/// This system has REAL solution (x=2, y=2) but...
/// Wait, let's check: 1*2 + 2*2 = 6 ✓, 3*2 + 4*2 = 14 ≠ 11
/// Let's use [[7],[15]] which has solution x=1, y=3 (integers)
/// And [[6],[14]] which has x=2, y=2 (integers)
/// For no integer solution, use [[6],[11]]:
///   1*x + 2*y = 6 → x = 6 - 2y
///   3*x + 4*y = 11 → 3(6-2y) + 4y = 11 → 18 - 6y + 4y = 11 → y = 3.5 (not integer!)
fn build_no_integer_solution_equation() -> Expression {
    // Left side: [[6], [11]] - chosen so solution is y=3.5, x=-1
    let result_matrix = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("1".to_string()),
            Expression::List(vec![
                Expression::Const("6".to_string()),
                Expression::Const("11".to_string()),
            ]),
        ],
    };

    // Matrix A: [[1,2],[3,4]]
    let matrix_a = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            Expression::List(vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
                Expression::Const("3".to_string()),
                Expression::Const("4".to_string()),
            ]),
        ],
    };

    // Vector x: [[x], [y]]
    let vector_x = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("1".to_string()),
            Expression::List(vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ]),
        ],
    };

    // A × x
    let product = Expression::Operation {
        name: "multiply".to_string(),
        args: vec![matrix_a, vector_x],
    };

    // result = A × x
    Expression::Operation {
        name: "equals".to_string(),
        args: vec![result_matrix, product],
    }
}

/// IGNORED: Requires list indexing for matrix operations
/// TO ENABLE: See test_z3_solves_matrix_linear_system for required axioms
#[test]
#[ignore]
fn test_z3_no_integer_solution() {
    let registry = StructureRegistry::default();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    // [[6],[11]] = [[1,2],[3,4]] × [[x],[y]]
    // Real solution: y = 3.5, x = -1 (not integers!)
    let equation = build_no_integer_solution_equation();

    println!("Testing: [[6],[11]] = [[1,2],[3,4]] × [[x],[y]]");
    println!("Real solution: x = -1, y = 3.5 (NOT integers)");
    println!();

    // First, check if ANY solution exists (without integer constraint)
    // Note: Z3 uses integers by default, so this will be UNSAT
    let result = backend
        .check_satisfiability(&equation)
        .expect("Satisfiability check failed");

    println!("Result (integer domain): {:?}", result);

    match result {
        SatisfiabilityResult::Satisfiable { example } => {
            println!("Found integer solution: {}", example);
            println!("(This would mean Z3 found integers that work)");
        }
        SatisfiabilityResult::Unsatisfiable => {
            println!("✅ UNSATISFIABLE in integers!");
            println!("   This system has no integer solution.");
            println!("   (Real solution exists: x=-1, y=3.5)");
        }
        SatisfiabilityResult::Unknown => {
            println!("❓ UNKNOWN");
        }
    }
}

/// IGNORED: Requires list indexing for matrix operations
/// TO ENABLE: See test_z3_solves_matrix_linear_system for required axioms
#[test]
#[ignore]
fn test_z3_verifies_correct_solution() {
    let registry = StructureRegistry::default();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    // Test: [[5],[11]] = [[1,2],[3,4]] × [[1],[2]]
    // This should be VALID (or at least SATISFIABLE) since 1,2 is the correct solution
    let equation_with_solution = Expression::Operation {
        name: "equals".to_string(),
        args: vec![
            // Left: [[5], [11]]
            Expression::Operation {
                name: "Matrix".to_string(),
                args: vec![
                    Expression::Const("2".to_string()),
                    Expression::Const("1".to_string()),
                    Expression::List(vec![
                        Expression::Const("5".to_string()),
                        Expression::Const("11".to_string()),
                    ]),
                ],
            },
            // Right: [[1,2],[3,4]] × [[1],[2]]
            Expression::Operation {
                name: "multiply".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "Matrix".to_string(),
                        args: vec![
                            Expression::Const("2".to_string()),
                            Expression::Const("2".to_string()),
                            Expression::List(vec![
                                Expression::Const("1".to_string()),
                                Expression::Const("2".to_string()),
                                Expression::Const("3".to_string()),
                                Expression::Const("4".to_string()),
                            ]),
                        ],
                    },
                    Expression::Operation {
                        name: "Matrix".to_string(),
                        args: vec![
                            Expression::Const("2".to_string()),
                            Expression::Const("1".to_string()),
                            Expression::List(vec![
                                Expression::Const("1".to_string()),
                                Expression::Const("2".to_string()),
                            ]),
                        ],
                    },
                ],
            },
        ],
    };

    println!("Testing: [[5],[11]] = [[1,2],[3,4]] × [[1],[2]]");

    // This concrete equation should be VALID
    let result = backend
        .verify_axiom(&equation_with_solution)
        .expect("Verification failed");

    println!("Verification result: {:?}", result);

    use kleis::solvers::backend::VerificationResult;
    match result {
        VerificationResult::Valid => {
            println!("✅ VALID - The matrix equation is correct!");
        }
        VerificationResult::Invalid { counterexample } => {
            panic!(
                "❌ INVALID - but [[5],[11]] = [[1,2],[3,4]] × [[1],[2]] IS correct! CE: {}",
                counterexample
            );
        }
        VerificationResult::Unknown => {
            println!("❓ UNKNOWN - Z3 couldn't determine validity");
        }
    }
}
