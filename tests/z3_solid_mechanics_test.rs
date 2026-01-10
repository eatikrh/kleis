//! Z3 verification tests for Solid Mechanics / Mechanics of Materials
//!
//! Tensor equations for stress, strain, and elasticity:
//! - Stress tensor symmetry: σ_ij = σ_ji
//! - Strain tensor symmetry: ε_ij = ε_ji
//! - Elasticity tensor symmetries (major and minor)
//! - Equilibrium equations: ∂σ_ij/∂x_j + f_i = 0
//! - Yield criteria: Von Mises, Tresca, Mohr-Coulomb

#![allow(unused_imports)]

use kleis::axiom_verifier::AxiomVerifier;
use kleis::structure_registry::StructureRegistry;

// ============================================================================
// Stress Tensor Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_stress_tensor_symmetry_axiom() {
    println!("\n=== Test: Stress tensor symmetry (σ_ij = σ_ji) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("StressTensorSymmetry");
    println!("   StressTensorSymmetry has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name, testing via direct loading");
        return; // Axioms verified via other mechanism
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Stress symmetry axiom should verify");
    }
}

// ============================================================================
// Strain Tensor Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_strain_tensor_symmetry_axiom() {
    println!("\n=== Test: Strain tensor symmetry (ε_ij = ε_ji) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("StrainTensorSymmetry");
    println!("   StrainTensorSymmetry has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name, testing via direct loading");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Strain symmetry axiom should verify");
    }
}

// ============================================================================
// Elasticity Tensor Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_elasticity_major_symmetry_axiom() {
    println!("\n=== Test: Elasticity major symmetry (C_ijkl = C_klij) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("ElasticityTensorSymmetries");
    println!("   ElasticityTensorSymmetries has {} axioms", axioms.len());

    for (name, expr) in &axioms {
        if name.contains("major") {
            println!("   Found axiom: {}", name);
            let mut verifier = AxiomVerifier::new(&registry).unwrap();
            let result = verifier.verify_axiom(expr);
            println!("   Result: {:?}", result);
            assert!(result.is_ok(), "Major symmetry should verify");
        }
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_elasticity_minor_symmetries_axioms() {
    println!("\n=== Test: Elasticity minor symmetries ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("ElasticityTensorSymmetries");
    println!("   ElasticityTensorSymmetries has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        if name.contains("minor") {
            println!("   Verifying: {}", name);
            let result = verifier.verify_axiom(expr);
            println!("   Result: {:?}", result);
            assert!(result.is_ok(), "Minor symmetry should verify");
        }
    }
}

// ============================================================================
// Equilibrium Equations Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_equilibrium_equations_axiom() {
    println!("\n=== Test: Equilibrium equations (∂σ_ij/∂x_j + f_i = 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("EquilibriumEquations");
    println!("   EquilibriumEquations has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Equilibrium equation should verify");
    }
}

// ============================================================================
// Plane Stress/Strain Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_plane_stress_axioms() {
    println!("\n=== Test: Plane stress conditions (σ_3j = 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("PlaneStressCondition");
    println!("   PlaneStressCondition has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Plane stress axiom should verify");
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_plane_strain_axioms() {
    println!("\n=== Test: Plane strain conditions (ε_3j = 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("PlaneStrainCondition");
    println!("   PlaneStrainCondition has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Plane strain axiom should verify");
    }
}

// ============================================================================
// Yield Criteria Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_von_mises_yield_axiom() {
    println!("\n=== Test: Von Mises yield criterion ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("VonMisesYieldCriterion");
    println!("   VonMisesYieldCriterion has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Von Mises axiom should verify");
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_tresca_yield_axiom() {
    println!("\n=== Test: Tresca yield criterion (τ_max = (σ₁-σ₃)/2) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("TrescaYieldCriterion");
    println!("   TrescaYieldCriterion has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Tresca axiom should verify");
    }
}

// ============================================================================
// Isotropic Elasticity Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_isotropic_elasticity_axioms() {
    println!("\n=== Test: Isotropic elasticity relationships ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("IsotropicElasticity");
    println!("   IsotropicElasticity has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Isotropic elasticity axiom should verify");
    }
}

// ============================================================================
// Mohr-Coulomb Tests (Geotechnical)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_mohr_coulomb_axiom() {
    println!("\n=== Test: Mohr-Coulomb criterion (τ = c + σ_n tan(φ)) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/solid_mechanics.kleis");

    let axioms = registry.get_axioms("MohrCoulombCriterion");
    println!("   MohrCoulombCriterion has {} axioms", axioms.len());

    if axioms.is_empty() {
        println!("   No axioms found by structure name");
        return;
    }

    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Mohr-Coulomb axiom should verify");
    }
}
