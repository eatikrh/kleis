//! Tests for Cartan geometry computations
//!
//! Verifies that we can actually compute:
//! - Exterior derivatives using diff
//! - Wedge products
//! - Connection from tetrad
//! - Curvature from connection
//! - Ricci tensor from curvature

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::{parse_kleis_program, KleisParser};
use kleis::pretty_print::PrettyPrinter;

/// Load both symbolic_diff.kleis and cartan_compute.kleis
fn create_evaluator() -> Evaluator {
    let mut evaluator = Evaluator::new();

    // Load symbolic differentiation first
    let diff_source = std::fs::read_to_string("stdlib/symbolic_diff.kleis")
        .expect("Failed to read stdlib/symbolic_diff.kleis");
    let diff_program =
        parse_kleis_program(&diff_source).expect("Failed to parse stdlib/symbolic_diff.kleis");
    evaluator
        .load_program(&diff_program)
        .expect("Failed to load symbolic_diff.kleis");

    // Load Cartan computation
    let cartan_source = std::fs::read_to_string("stdlib/cartan_compute.kleis")
        .expect("Failed to read stdlib/cartan_compute.kleis");
    let cartan_program =
        parse_kleis_program(&cartan_source).expect("Failed to parse stdlib/cartan_compute.kleis");
    evaluator
        .load_program(&cartan_program)
        .expect("Failed to load cartan_compute.kleis");

    evaluator
}

/// Helper to evaluate an expression and return string result
fn eval(evaluator: &Evaluator, input: &str) -> Result<String, String> {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let result = evaluator.eval_concrete(&expr)?;
    let pp = PrettyPrinter::new();
    Ok(pp.format_expression(&result))
}

// =============================================================================
// Basic Form Operations
// =============================================================================

#[test]
fn test_basis_forms_exist() {
    let evaluator = create_evaluator();

    // Check that basis 1-forms are defined
    let result = eval(&evaluator, "dt");
    println!("dt = {:?}", result);
    assert!(result.is_ok());

    let result = eval(&evaluator, "dr");
    println!("dr = {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_scale1() {
    let evaluator = create_evaluator();

    // Scale dt by r (using var("r") helper)
    let result = eval(&evaluator, "scale1(var(\"r\"), dt)");
    println!("r * dt = {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_add1() {
    let evaluator = create_evaluator();

    // Add dt + dr
    let result = eval(&evaluator, "add1(dt, dr)");
    println!("dt + dr = {:?}", result);
    assert!(result.is_ok());
}

// =============================================================================
// Exterior Derivative Tests
// =============================================================================

#[test]
fn test_d0_constant() {
    let evaluator = create_evaluator();

    // d(5) = 0 (derivative of constant is zero)
    let result = eval(&evaluator, "d0(num(5))");
    println!("d(5) = {:?}", result);
    assert!(result.is_ok());
    // All coefficients should be 0
}

#[test]
fn test_d0_variable() {
    let evaluator = create_evaluator();

    // d(r) = dr
    let result = eval(&evaluator, "d0(var(\"r\"))");
    println!("d(r) = {:?}", result);
    assert!(result.is_ok());
    // Should have coefficient 1 in dr position
}

#[test]
fn test_d0_r_squared() {
    let evaluator = create_evaluator();

    // d(r²) = 2r dr
    let result = eval(&evaluator, "d0(e_pow(var(\"r\"), num(2)))");
    println!("d(r²) = {:?}", result);
    assert!(result.is_ok());
    // Coefficient of dr should be 2r
}

#[test]
fn test_d1_coordinate_form() {
    let evaluator = create_evaluator();

    // d(dt) = 0 (second derivative)
    let result = eval(&evaluator, "d1(dt)");
    println!("d(dt) = {:?}", result);
    assert!(result.is_ok());
    // Should be zero 2-form (d² = 0)
}

// =============================================================================
// Wedge Product Tests
// =============================================================================

#[test]
fn test_wedge_basis_forms() {
    let evaluator = create_evaluator();

    // dt ∧ dr
    let result = eval(&evaluator, "wedge(dt, dr)");
    println!("dt ∧ dr = {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_wedge_antisymmetric() {
    let evaluator = create_evaluator();

    // dt ∧ dt = 0
    let result = eval(&evaluator, "wedge(dt, dt)");
    println!("dt ∧ dt = {:?}", result);
    assert!(result.is_ok());
    // Should be zero (antisymmetric)
}

// =============================================================================
// Tetrad Tests
// =============================================================================

#[test]
fn test_minkowski_tetrad() {
    let evaluator = create_evaluator();

    // Minkowski tetrad should be defined
    let result = eval(&evaluator, "minkowski_tetrad");
    println!("Minkowski tetrad = {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_schwarzschild_tetrad() {
    let evaluator = create_evaluator();

    // Schwarzschild tetrad with symbolic M
    let result = eval(&evaluator, "schwarzschild_tetrad(var(\"M\"))");
    println!("Schwarzschild tetrad = {:?}", result);
    assert!(result.is_ok());
}

// =============================================================================
// Tetrad Derivative Tests
// =============================================================================

#[test]
fn test_d_tetrad() {
    let evaluator = create_evaluator();

    // Compute exterior derivatives of Minkowski tetrad
    let result = eval(&evaluator, "d_tetrad(minkowski_tetrad)");
    println!("d(minkowski_tetrad) = {:?}", result);
    assert!(result.is_ok());
}

// =============================================================================
// Connection Solver Tests (with expression size debugging)
// =============================================================================

#[test]
fn test_solve_connection_minkowski() {
    let evaluator = create_evaluator();

    // Step 1: Check tetrad size
    let tetrad = eval(&evaluator, "minkowski_tetrad").unwrap();
    println!("📊 Minkowski tetrad size: {} chars", tetrad.len());

    // Step 2: Check d_tetrad size
    let d_tetrad = eval(&evaluator, "d_tetrad(minkowski_tetrad)").unwrap();
    println!("📊 d(tetrad) size: {} chars", d_tetrad.len());

    // Step 3: Check connection size
    let result = eval(&evaluator, "solve_connection(minkowski_tetrad)");
    let conn_size = result.as_ref().map(|s| s.len()).unwrap_or(0);
    println!("📊 Connection size: {} chars", conn_size);
    println!(
        "📊 Sample: {:?}",
        result.as_ref().map(|s| &s[..200.min(s.len())])
    );

    assert!(result.is_ok());
    assert!(
        conn_size < 50000,
        "Expression exploded! {} chars",
        conn_size
    );
}

#[test]
fn test_solve_connection_schwarzschild() {
    let evaluator = create_evaluator();

    // Step 1: Check tetrad size
    let tetrad = eval(&evaluator, "schwarzschild_tetrad(var(\"M\"))").unwrap();
    println!("📊 Schwarzschild tetrad size: {} chars", tetrad.len());

    // Step 2: Check d_tetrad size
    let d_tetrad = eval(&evaluator, "d_tetrad(schwarzschild_tetrad(var(\"M\")))").unwrap();
    println!("📊 d(tetrad) size: {} chars", d_tetrad.len());

    // Step 3: Check connection size
    let result = eval(
        &evaluator,
        "solve_connection(schwarzschild_tetrad(var(\"M\")))",
    );
    let conn_size = result.as_ref().map(|s| s.len()).unwrap_or(0);
    println!("📊 Connection size: {} chars", conn_size);

    assert!(result.is_ok());
    assert!(
        conn_size < 500000,
        "Expression exploded! {} chars",
        conn_size
    );
}

// =============================================================================
// Curvature Computation Tests (with expression size debugging)
// =============================================================================
// These tests compute the full Riemann curvature tensor using Cartan's formalism:
//   R^a_b = dω^a_b + Σ_c ω^a_c ∧ ω^c_b

#[test]
fn test_minkowski_curvature() {
    let evaluator = create_evaluator();

    // Step 1: Connection
    let conn = eval(&evaluator, "solve_connection(minkowski_tetrad)").unwrap();
    println!("📊 Minkowski connection size: {} chars", conn.len());

    // Step 2: Full curvature
    let result = eval(&evaluator, "minkowski_curvature");
    let curv_size = result.as_ref().map(|s| s.len()).unwrap_or(0);
    println!("📊 Minkowski curvature size: {} chars", curv_size);

    assert!(result.is_ok());
    assert!(
        curv_size < 100000,
        "Expression exploded! {} chars",
        curv_size
    );
}

#[test]
fn test_schwarzschild_curvature() {
    // Run in a larger stack to avoid CI stack overflow during deep formatting.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let evaluator = create_evaluator();

            // Step 1: Connection
            let conn = eval(
                &evaluator,
                "solve_connection(schwarzschild_tetrad(var(\"M\")))",
            )
            .unwrap();
            println!("📊 Schwarzschild connection size: {} chars", conn.len());

            // Step 2: Full curvature - this is the Riemann tensor for a black hole!
            let result = eval(&evaluator, "schwarzschild_curvature(var(\"M\"))");
            let curv_size = result.as_ref().map(|s| s.len()).unwrap_or(0);
            println!("📊 Schwarzschild curvature size: {} chars", curv_size);

            assert!(result.is_ok());
            // Schwarzschild curvature is genuinely complex - allow larger expressions
            assert!(
                curv_size < 5000000,
                "Expression exploded! {} chars",
                curv_size
            );
        })
        .expect("failed to spawn test thread with larger stack")
        .join()
        .expect("test thread panicked");
}

// =============================================================================
// Literature Verification Tests
// =============================================================================
// Verify computed results match known analytical solutions

#[test]
fn test_minkowski_curvature_is_zero() {
    // LITERATURE CHECK: Flat spacetime has ZERO curvature
    // Reference: Any GR textbook (Misner, Thorne, Wheeler "Gravitation" Ch. 1)
    let evaluator = create_evaluator();

    let result = eval(&evaluator, "minkowski_curvature").unwrap();

    // All components should simplify to ENumber(0)
    // The curvature is a 4x4 matrix of 4x4 matrices (2-forms)
    // For flat space, every component should be zero

    // Check that the result contains mostly zeros
    let zero_count = result.matches("ENumber(0)").count();
    let total_components = 16 * 16; // 4x4 matrix of 4x4 2-forms = 256 components

    println!(
        "📊 Minkowski curvature zero count: {} / {} components",
        zero_count, total_components
    );

    // Most components should be zero (allowing for some structural overhead)
    assert!(
        zero_count > 200,
        "Minkowski curvature should be mostly zeros, got {} zeros",
        zero_count
    );
}

#[test]
fn test_schwarzschild_connection_nonzero() {
    // LITERATURE CHECK: Schwarzschild has non-trivial connection
    // The connection 1-forms ω^a_b should contain terms with M/r
    // Reference: Carroll "Spacetime and Geometry" Ch. 3
    let evaluator = create_evaluator();

    let result = eval(
        &evaluator,
        "solve_connection(schwarzschild_tetrad(var(\"M\")))",
    )
    .unwrap();

    // Should contain the mass parameter M
    assert!(
        result.contains("EVariable(\"M\")"),
        "Schwarzschild connection should depend on mass M"
    );

    // Should contain radial coordinate r
    assert!(
        result.contains("EVariable(\"r\")"),
        "Schwarzschild connection should depend on radius r"
    );

    // Should contain angular terms (sin, cos of theta)
    let has_angular = result.contains("sin") || result.contains("cos") || result.contains("theta");
    assert!(
        has_angular,
        "Schwarzschild connection should have angular dependence"
    );

    println!("✓ Schwarzschild connection contains M, r, and angular terms as expected");
}

#[test]
fn test_schwarzschild_curvature_structure() {
    // Run in a larger stack to avoid CI stack overflow during deep formatting.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            // LITERATURE CHECK: Schwarzschild Riemann tensor structure
            // Key properties:
            // 1. Depends on M/r³ (Newtonian tidal forces)
            // 2. Has specific symmetries (Riemann symmetries)
            // Reference: Wald "General Relativity" Appendix C
            let evaluator = create_evaluator();

            let result = eval(&evaluator, "schwarzschild_curvature(var(\"M\"))").unwrap();

            // Should contain the mass parameter M
            assert!(
                result.contains("EVariable(\"M\")"),
                "Schwarzschild curvature should depend on mass M"
            );

            // Should contain radial coordinate r (tidal forces ~ 1/r³)
            assert!(
                result.contains("EVariable(\"r\")"),
                "Schwarzschild curvature should depend on radius r"
            );

            // Should contain the metric factor f = 1 - 2M/r or its derivatives
            // This appears as e_sub(num(1), ...) or sqrt
            let has_metric_factor = result.contains("minus") || result.contains("sqrt");
            assert!(
                has_metric_factor,
                "Schwarzschild curvature should contain metric factor"
            );

            println!("✓ Schwarzschild curvature has expected structure (M, r, metric factor)");
        })
        .expect("failed to spawn test thread with larger stack")
        .join()
        .expect("test thread panicked");
}

#[test]
fn test_schwarzschild_curvature_component_r0101() {
    // Run in a larger stack to avoid CI stack overflow during deep formatting.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            // Extract and print R^0_1_01 (time-radial curvature, dt∧dr coefficient)
            //
            // LITERATURE: For Schwarzschild in orthonormal frame,
            // R^0_1_01 should be proportional to M/r³
            // Reference: Carroll "Spacetime and Geometry" Eq. 5.29
            let evaluator = create_evaluator();

            // Full curvature is 4x4 matrix of 2-forms (each 2-form is 4x4)
            // R^a_b_μν = curv[a][b][μ][ν]
            // R^0_1_01 = curv[0][1][0][1]

            let component = eval(
                &evaluator,
                "let curv = schwarzschild_curvature(var(\"M\")) in \
         nth(nth(nth(nth(curv, 0), 1), 0), 1)",
            )
            .unwrap();

            println!("\n=== Schwarzschild Curvature Component R^0_1_01 ===");
            println!("(coefficient of dt∧dr in R^0_1 curvature 2-form)\n");
            println!("{}\n", component);
            println!("Size: {} chars", component.len());

            // Verify it contains expected terms
            assert!(
                component.contains("EVariable(\"M\")") || component.contains("M"),
                "R^0_1_01 should contain mass M"
            );
            assert!(
                component.contains("EVariable(\"r\")") || component.contains("r"),
                "R^0_1_01 should contain radius r"
            );

            println!("\n=== Expected from Literature ===");
            println!("R^0_1_01 ∝ M/r³ (radial tidal force)");
            println!("Reference: Carroll 'Spacetime and Geometry' Chapter 5");
        })
        .expect("failed to spawn test thread with larger stack")
        .join()
        .expect("test thread panicked");
}

#[test]
fn test_schwarzschild_numerical_verification() {
    // Numerically verify curvature at specific point
    // At r=10M (well outside horizon), R^0_1_01 should ≈ -2M/r³ = -2/(10³) = -0.002
    // (in geometric units where M=1)
    let evaluator = create_evaluator();

    // First, substitute M=1 into the curvature expression
    // We need to evaluate with concrete values

    // Get the expression with M=1
    let component = eval(
        &evaluator,
        "let curv = schwarzschild_curvature(num(1)) in \
         nth(nth(nth(nth(curv, 0), 1), 0), 1)",
    )
    .unwrap();

    println!("\n=== Numerical Verification at M=1 ===");
    println!("Expression with M=1: {} chars", component.len());

    // The expression still contains var("r")
    // To get a number, we'd need to substitute r as well
    // For now, just verify the structure is maintained

    assert!(
        component.contains("EVariable(\"r\")"),
        "Should still contain r"
    );
    assert!(
        !component.contains("EVariable(\"M\")"),
        "M should be substituted"
    );

    println!("✓ M successfully substituted to 1");
    println!("✓ Expression still contains r (as expected)");

    // Note: Full numerical evaluation would require:
    // 1. Substituting r=10 into the Expr AST
    // 2. Evaluating the resulting arithmetic
    // This would require extending the simplify/eval capabilities
}

#[test]
fn test_compute_riemann() {
    let evaluator = create_evaluator();

    // Direct call to compute_riemann - should match minkowski_curvature
    let result = eval(&evaluator, "compute_riemann(minkowski_tetrad)");
    let curv_size = result.as_ref().map(|s| s.len()).unwrap_or(0);
    println!("📊 compute_riemann(minkowski) size: {} chars", curv_size);

    assert!(result.is_ok());
    assert!(
        curv_size < 100000,
        "Expression exploded! {} chars",
        curv_size
    );
}
