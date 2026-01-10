//! Z3 verification tests for cosmological spacetimes
//!
//! Solutions to Einstein's field equations:
//! G_μν + Λg_μν = κT_μν
//!
//! Includes: Minkowski, de Sitter, Anti-de Sitter, FLRW, Schwarzschild

#![allow(unused_imports)]

use kleis::axiom_verifier::AxiomVerifier;
use kleis::structure_registry::StructureRegistry;

// ============================================================================
// Minkowski Spacetime Tests (Flat, Special Relativity)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_minkowski_structure_loaded() {
    println!("\n=== Test: MinkowskiSpacetime structure loaded ===");

    let mut registry = StructureRegistry::new();
    let load_result = registry.load_from_file("stdlib/cosmology.kleis");

    assert!(load_result.is_ok(), "Should load cosmology.kleis");
    println!("   Loaded {} structures", load_result.unwrap());

    let axioms = registry.get_axioms("MinkowskiSpacetime");
    println!("   MinkowskiSpacetime has {} axioms", axioms.len());
    assert!(axioms.len() >= 3, "Should have curvature vanishing and symmetry axioms");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_minkowski_flat_spacetime_axioms() {
    println!("\n=== Test: Minkowski spacetime (flat, Ric=0, G=0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("MinkowskiSpacetime");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Minkowski axiom should verify");
    }
}

// ============================================================================
// de Sitter Spacetime Tests (Positive Λ, Accelerating Expansion)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_desitter_structure_loaded() {
    println!("\n=== Test: DeSitterSpacetime structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("DeSitterSpacetime");
    println!("   DeSitterSpacetime has {} axioms", axioms.len());
    assert!(axioms.len() >= 3, "Should have positive_lambda, desitter_einstein, symmetry axioms");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_desitter_vacuum_solution_axioms() {
    println!("\n=== Test: de Sitter vacuum solution (G = -Λg, Λ > 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("DeSitterSpacetime");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "de Sitter axiom should verify");
    }
}

// ============================================================================
// Anti-de Sitter Spacetime Tests (Negative Λ, AdS/CFT)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_anti_desitter_structure_loaded() {
    println!("\n=== Test: AntiDeSitterSpacetime structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("AntiDeSitterSpacetime");
    println!("   AntiDeSitterSpacetime has {} axioms", axioms.len());
    assert!(axioms.len() >= 4, "Should have negative_lambda, ads_einstein, ads_ricci, ads_ricci_scalar axioms");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_anti_desitter_negative_curvature_axioms() {
    println!("\n=== Test: Anti-de Sitter (negative Λ, negative curvature) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("AntiDeSitterSpacetime");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Anti-de Sitter axiom should verify");
    }
}

// ============================================================================
// FLRW Cosmology Tests (Homogeneous, Isotropic Universe)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_flrw_structure_loaded() {
    println!("\n=== Test: FLRWCosmology structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("FLRWCosmology");
    println!("   FLRWCosmology has {} axioms", axioms.len());
    assert!(axioms.len() >= 3, "Should have symmetry and field equation axioms");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_flrw_cosmology_axioms() {
    println!("\n=== Test: FLRW cosmology (homogeneous, isotropic) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("FLRWCosmology");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "FLRW axiom should verify");
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_flrw_perfect_fluid() {
    println!("\n=== Test: FLRW perfect fluid stress-energy ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("FLRWCosmology");

    for (name, expr) in &axioms {
        if name.contains("perfect_fluid") {
            println!("   Found axiom: {}", name);
            let mut verifier = AxiomVerifier::new(&registry).unwrap();
            let result = verifier.verify_axiom(expr);
            println!("   Result: {:?}", result);
            assert!(result.is_ok(), "Perfect fluid axiom should verify");
        }
    }
}

// ============================================================================
// Schwarzschild Spacetime Tests (Non-rotating Black Hole)
// ============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_schwarzschild_structure_loaded() {
    println!("\n=== Test: SchwarzschildSpacetime structure loaded ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("SchwarzschildSpacetime");
    println!("   SchwarzschildSpacetime has {} axioms", axioms.len());
    assert!(axioms.len() >= 3, "Should have vacuum and symmetry axioms");
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_schwarzschild_vacuum_axioms() {
    println!("\n=== Test: Schwarzschild vacuum solution (G = 0) ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    let axioms = registry.get_axioms("SchwarzschildSpacetime");
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    for (name, expr) in &axioms {
        println!("   Verifying: {}", name);
        let result = verifier.verify_axiom(expr);
        println!("   Result: {:?}", result);
        assert!(result.is_ok(), "Schwarzschild axiom should verify");
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_schwarzschild_vs_minkowski() {
    println!("\n=== Test: Schwarzschild and Minkowski both have G=0 ===");

    let mut registry = StructureRegistry::new();
    let _ = registry.load_from_file("stdlib/cosmology.kleis");

    // Both are vacuum solutions with Λ=0
    let minkowski_axioms = registry.get_axioms("MinkowskiSpacetime");
    let schwarzschild_axioms = registry.get_axioms("SchwarzschildSpacetime");

    // Count vacuum axioms
    let mink_vacuum = minkowski_axioms
        .iter()
        .filter(|(n, _)| n.contains("vanish") || n.contains("lambda"))
        .count();
    let schw_vacuum = schwarzschild_axioms
        .iter()
        .filter(|(n, _)| n.contains("vacuum") || n.contains("lambda"))
        .count();

    println!("   Minkowski vacuum axioms: {}", mink_vacuum);
    println!("   Schwarzschild vacuum axioms: {}", schw_vacuum);

    assert!(mink_vacuum >= 2, "Minkowski should have vacuum axioms");
    assert!(schw_vacuum >= 2, "Schwarzschild should have vacuum axioms");
}

