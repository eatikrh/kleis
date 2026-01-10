//! Z3 verification tests for Navier-Stokes and fluid dynamics equations
//!
//! Navier-Stokes equations:
//! 1. Continuity: ∂ρ/∂t + ∇·(ρv) = 0
//! 2. Momentum: ρ(∂v/∂t + v·∇v) = -∇p + μ∇²v + ρf
//!
//! Also includes: Stokes flow, Euler equations, Bernoulli equation

#![allow(unused_imports)]

use kleis::axiom_verifier::AxiomVerifier;
use kleis::structure_registry::StructureRegistry;

// ============================================================================
// Viscous Stress Tensor Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_viscous_stress_tensor_structure_loaded() {
    println!("\n=== Test: ViscousStressTensor structure loaded ===");

    let mut registry = StructureRegistry::new();
    let load_result = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    assert!(load_result.is_ok(), "Should load fluid_dynamics.kleis");
    println!("   Loaded {} structures", load_result.unwrap());

    let axioms = registry.get_axioms("ViscousStressTensor");
    println!("   ViscousStressTensor has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have tau_symmetric axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_viscous_stress_symmetric_axiom() {
    println!("\n=== Test: Viscous stress tensor symmetry (τ_ij = τ_ji) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("ViscousStressTensor");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Stress tensor symmetry should verify");
    }
}

// ============================================================================
// Newtonian Viscosity Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_newtonian_viscosity_structure_loaded() {
    println!("\n=== Test: NewtonianViscosity structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("NewtonianViscosity");
    println!("   NewtonianViscosity has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have newtonian_viscosity axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_newtonian_viscosity_axiom() {
    println!("\n=== Test: Newtonian viscosity (div(τ)_i = μ∇²u_i) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("NewtonianViscosity");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Newtonian viscosity axiom should verify");
    }
}

// ============================================================================
// Continuity Equation Tests (Mass Conservation)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_continuity_equation_structure_loaded() {
    println!("\n=== Test: ContinuityEquation structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("ContinuityEquation");
    println!("   ContinuityEquation has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have continuity axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_continuity_equation_axiom() {
    println!("\n=== Test: Continuity equation (∂ρ/∂t + ∇·(ρv) = 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("ContinuityEquation");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Continuity equation should verify");
    }
}

// ============================================================================
// Momentum Equation Tests (Navier-Stokes Core)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_momentum_equation_structure_loaded() {
    println!("\n=== Test: MomentumEquation structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("MomentumEquation");
    println!("   MomentumEquation has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have momentum axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_navier_stokes_momentum_axiom() {
    println!("\n=== Test: Navier-Stokes momentum equation ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("MomentumEquation");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "NS momentum equation should verify");
    }
}

// ============================================================================
// Incompressible Flow Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_incompressible_flow_structure_loaded() {
    println!("\n=== Test: IncompressibleFlow structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("IncompressibleFlow");
    println!("   IncompressibleFlow has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have incompressible axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_incompressibility_axiom() {
    println!("\n=== Test: Incompressibility constraint (∇·v = 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("IncompressibleFlow");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Incompressibility should verify");
    }
}

// ============================================================================
// Stokes Flow Tests (Creeping Flow, Re << 1)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_stokes_flow_structure_loaded() {
    println!("\n=== Test: StokesFlow structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("StokesFlow");
    println!("   StokesFlow has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have stokes axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_stokes_equation_axiom() {
    println!("\n=== Test: Stokes equation (∇p = μ∇²v) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("StokesFlow");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Stokes equation should verify");
    }
}

// ============================================================================
// Euler Equations Tests (Inviscid Flow)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_euler_equations_structure_loaded() {
    println!("\n=== Test: EulerEquations structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("EulerEquations");
    println!("   EulerEquations has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have euler axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_euler_inviscid_axiom() {
    println!("\n=== Test: Euler equation (inviscid Navier-Stokes) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("EulerEquations");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Euler equation should verify");
    }
}

// ============================================================================
// Bernoulli Equation Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_bernoulli_equation_structure_loaded() {
    println!("\n=== Test: BernoulliEquation structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("BernoulliEquation");
    println!("   BernoulliEquation has {} axioms", axioms.len());
    assert!(
        axioms.len() >= 3,
        "Should have B1_def, B2_def, bernoulli_conservation axioms"
    );
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_bernoulli_conservation_axiom() {
    println!("\n=== Test: Bernoulli's principle (p + ½ρv² + ρgh = const) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("BernoulliEquation");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Bernoulli axiom should verify");
    }
}

// ============================================================================
// Compressible Flow + Energy Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_compressible_energy_structure_loaded() {
    println!("\n=== Test: CompressibleEnergy structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("CompressibleEnergy");
    println!("   CompressibleEnergy has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have energy axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_energy_equation_axiom() {
    println!("\n=== Test: Energy equation (∂E/∂t + ∇·[(E+p)v] = ...) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("CompressibleEnergy");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Energy equation should verify");
    }
}

// ============================================================================
// Ideal Gas Equation of State Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_ideal_gas_eos_structure_loaded() {
    println!("\n=== Test: IdealGasEOS structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("IdealGasEOS");
    println!("   IdealGasEOS has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have ideal_gas axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_ideal_gas_axiom() {
    println!("\n=== Test: Ideal gas equation of state (p = ρRT) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/fluid_dynamics.kleis");

    let axioms = registry.get_axioms("IdealGasEOS");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Ideal gas EOS should verify");
    }
}
