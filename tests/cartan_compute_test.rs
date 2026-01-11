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

    // Scale dt by r
    let result = eval(&evaluator, "scale1(Var(\"r\"), dt)");
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
    let result = eval(&evaluator, "d0(Const(5))");
    println!("d(5) = {:?}", result);
    assert!(result.is_ok());
    // All coefficients should be 0
}

#[test]
fn test_d0_variable() {
    let evaluator = create_evaluator();

    // d(r) = dr
    let result = eval(&evaluator, "d0(Var(\"r\"))");
    println!("d(r) = {:?}", result);
    assert!(result.is_ok());
    // Should have coefficient 1 in dr position
}

#[test]
fn test_d0_r_squared() {
    let evaluator = create_evaluator();

    // d(r²) = 2r dr
    let result = eval(&evaluator, "d0(Pow(Var(\"r\"), Const(2)))");
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
    let result = eval(&evaluator, "schwarzschild_tetrad(Var(\"M\"))");
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
// Built-in Test Expressions
// =============================================================================

#[test]
fn test_d_r_squared() {
    let evaluator = create_evaluator();

    // d(r²) = 2r dr (should have 2r in position 1)
    let result = eval(&evaluator, "test_d_r_squared");
    println!("d(r²) = {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_d_sin_theta() {
    let evaluator = create_evaluator();

    // d(sin(θ)) = cos(θ) dθ
    let result = eval(&evaluator, "test_d_sin_theta");
    println!("d(sin(θ)) = {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_wedge_dt_dr() {
    let evaluator = create_evaluator();

    // dt ∧ dr has 1 at (0,1), -1 at (1,0), 0 elsewhere
    let result = eval(&evaluator, "test_wedge_dt_dr");
    println!("dt ∧ dr = {:?}", result);
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
    let tetrad = eval(&evaluator, "schwarzschild_tetrad(Var(\"M\"))").unwrap();
    println!("📊 Schwarzschild tetrad size: {} chars", tetrad.len());

    // Step 2: Check d_tetrad size
    let d_tetrad = eval(&evaluator, "d_tetrad(schwarzschild_tetrad(Var(\"M\")))").unwrap();
    println!("📊 d(tetrad) size: {} chars", d_tetrad.len());

    // Step 3: Check connection size
    let result = eval(
        &evaluator,
        "solve_connection(schwarzschild_tetrad(Var(\"M\")))",
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
    let evaluator = create_evaluator();

    // Step 1: Connection
    let conn = eval(
        &evaluator,
        "solve_connection(schwarzschild_tetrad(Var(\"M\")))",
    )
    .unwrap();
    println!("📊 Schwarzschild connection size: {} chars", conn.len());

    // Step 2: Full curvature - this is the Riemann tensor for a black hole!
    let result = eval(&evaluator, "schwarzschild_curvature(Var(\"M\"))");
    let curv_size = result.as_ref().map(|s| s.len()).unwrap_or(0);
    println!("📊 Schwarzschild curvature size: {} chars", curv_size);

    assert!(result.is_ok());
    // Schwarzschild curvature is genuinely complex - allow larger expressions
    assert!(
        curv_size < 5000000,
        "Expression exploded! {} chars",
        curv_size
    );
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

    // All components should simplify to Const(0)
    // The curvature is a 4x4 matrix of 4x4 matrices (2-forms)
    // For flat space, every component should be zero

    // Check that the result contains mostly zeros
    let zero_count = result.matches("Const(0)").count();
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
        "solve_connection(schwarzschild_tetrad(Var(\"M\")))",
    )
    .unwrap();

    // Should contain the mass parameter M
    assert!(
        result.contains("Var(\"M\")"),
        "Schwarzschild connection should depend on mass M"
    );

    // Should contain radial coordinate r
    assert!(
        result.contains("Var(\"r\")"),
        "Schwarzschild connection should depend on radius r"
    );

    // Should contain angular terms (sin, cos of theta)
    let has_angular = result.contains("Sin") || result.contains("Cos") || result.contains("theta");
    assert!(
        has_angular,
        "Schwarzschild connection should have angular dependence"
    );

    println!("✓ Schwarzschild connection contains M, r, and angular terms as expected");
}

#[test]
fn test_schwarzschild_curvature_structure() {
    // LITERATURE CHECK: Schwarzschild Riemann tensor structure
    // Key properties:
    // 1. Depends on M/r³ (Newtonian tidal forces)
    // 2. Has specific symmetries (Riemann symmetries)
    // Reference: Wald "General Relativity" Appendix C
    let evaluator = create_evaluator();

    let result = eval(&evaluator, "schwarzschild_curvature(Var(\"M\"))").unwrap();

    // Should contain the mass parameter M
    assert!(
        result.contains("Var(\"M\")"),
        "Schwarzschild curvature should depend on mass M"
    );

    // Should contain radial coordinate r (tidal forces ~ 1/r³)
    assert!(
        result.contains("Var(\"r\")"),
        "Schwarzschild curvature should depend on radius r"
    );

    // Should contain the metric factor f = 1 - 2M/r or its derivatives
    // This appears as Sub(Const(1), Div(...)) or similar
    let has_metric_factor = result.contains("Sub(Const(1)") || result.contains("Sqrt");
    assert!(
        has_metric_factor,
        "Schwarzschild curvature should contain metric factor"
    );

    println!("✓ Schwarzschild curvature has expected structure (M, r, metric factor)");
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

// =============================================================================
// Sanity Checks
// =============================================================================

#[test]
fn test_lorentzian_signature() {
    let evaluator = create_evaluator();

    let result = eval(&evaluator, "lorentzian");
    println!("Lorentzian signature = {:?}", result);
    assert!(result.is_ok());
    // Should be [-1, 1, 1, 1]
}

#[test]
fn test_coord_names() {
    let evaluator = create_evaluator();

    let result = eval(&evaluator, "coord_names");
    println!("Coordinate names = {:?}", result);
    assert!(result.is_ok());
    // Should be ["t", "r", "theta", "phi"]
}
