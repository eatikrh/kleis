//! Z3 verification tests for Maxwell's equations
//!
//! Maxwell's equations in covariant form:
//! 1. ∂_μ F^μν = μ₀ J^ν  (inhomogeneous - Gauss + Ampère)
//! 2. ∂_[λ F_μν] = 0     (homogeneous - Faraday + no monopoles)
//!
//! The electromagnetic field tensor F_μν is antisymmetric.

#![allow(unused_imports)]

use kleis::axiom_verifier::AxiomVerifier;
use kleis::structure_registry::StructureRegistry;

// ============================================================================
// Field Tensor Properties Tests
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_field_tensor_structure_loaded() {
    println!("\n=== Test: FieldTensorProperties structure loaded ===");

    let mut registry = StructureRegistry::new();
    let load_result = registry.load_from_file("stdlib/maxwell.kleis");

    assert!(load_result.is_ok(), "Should load maxwell.kleis");
    println!("   Loaded {} structures", load_result.unwrap());

    let axioms = registry.get_axioms("FieldTensorProperties");
    println!("   FieldTensorProperties has {} axioms", axioms.len());
    assert!(
        axioms.len() >= 2,
        "Should have F_antisymmetric and F_diagonal_zero axioms"
    );
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_f_antisymmetric_axiom() {
    println!("\n=== Test: F antisymmetry axiom ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("FieldTensorProperties");

    for (name, expr) in &axioms {
        if name.contains("antisymmetric") {
            println!("   Found axiom: {}", name);
            let mut verifier = AxiomVerifier::new(&registry).unwrap();
            let result = verifier.verify_axiom(expr);
            println!("   Verification result: {:?}", result);
            assert!(
                result.is_ok(),
                "F antisymmetry axiom verification should not error"
            );
        }
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_f_diagonal_zero_axiom() {
    println!("\n=== Test: F diagonal zero axiom ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("FieldTensorProperties");

    for (name, expr) in &axioms {
        if name.contains("diagonal") {
            println!("   Found axiom: {}", name);
            let mut verifier = AxiomVerifier::new(&registry).unwrap();
            let result = verifier.verify_axiom(expr);
            println!("   Verification result: {:?}", result);
            assert!(
                result.is_ok(),
                "F diagonal zero axiom verification should not error"
            );
        }
    }
}

// ============================================================================
// Inhomogeneous Maxwell Equation Tests (Gauss + Ampère)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_inhomogeneous_structure_loaded() {
    println!("\n=== Test: MaxwellInhomogeneous structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("MaxwellInhomogeneous");
    println!("   MaxwellInhomogeneous has {} axioms", axioms.len());
    assert!(
        !axioms.is_empty(),
        "Should have maxwell_inhomogeneous axiom"
    );
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_gauss_ampere_axiom() {
    println!("\n=== Test: Maxwell inhomogeneous equation (Gauss + Ampère) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("MaxwellInhomogeneous");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(
            result.is_ok(),
            "Maxwell inhomogeneous axiom should not error"
        );
    }
}

// ============================================================================
// Homogeneous Maxwell Equation Tests (Faraday + No Monopoles)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_homogeneous_structure_loaded() {
    println!("\n=== Test: MaxwellHomogeneous structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("MaxwellHomogeneous");
    println!("   MaxwellHomogeneous has {} axioms", axioms.len());
    assert!(!axioms.is_empty(), "Should have maxwell_homogeneous axiom");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_maxwell_faraday_no_monopoles_axiom() {
    println!("\n=== Test: Maxwell homogeneous equation (Faraday + No Monopoles) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("MaxwellHomogeneous");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Maxwell homogeneous axiom should not error");
    }
}

// ============================================================================
// Einstein-Maxwell Tests (Electromagnetism in Curved Spacetime)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_einstein_maxwell_structure_loaded() {
    println!("\n=== Test: EinsteinMaxwell structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("EinsteinMaxwell");
    println!("   EinsteinMaxwell has {} axioms", axioms.len());
    assert!(
        axioms.len() >= 5,
        "Should have field equations and symmetry axioms"
    );
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_einstein_maxwell_field_equations() {
    println!("\n=== Test: Einstein-Maxwell field equations ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("EinsteinMaxwell");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(
            result.is_ok(),
            "Einstein-Maxwell axiom verification should not error"
        );
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_einstein_maxwell_em_stress_symmetric() {
    println!("\n=== Test: EM stress-energy tensor symmetry ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/maxwell.kleis");

    let axioms = registry.get_axioms("EinsteinMaxwell");

    for (name, expr) in &axioms {
        if name.contains("T_EM_symmetric") {
            println!("   Found axiom: {}", name);
            let mut verifier = AxiomVerifier::new(&registry).unwrap();
            let result = verifier.verify_axiom(expr);
            println!("   Result: {:?}", result);
            assert!(result.is_ok(), "T_EM symmetry should verify");
        }
    }
}
