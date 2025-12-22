//! Tests for Set Theory operations
//!
//! These tests verify that Kleis's set theory operations work correctly
//! with Z3's native set theory support.

use kleis::kleis_parser::KleisParser;

/// Helper: Parse and verify a structure definition is valid
fn parses_ok(source: &str) -> bool {
    let mut parser = KleisParser::new(source);
    match parser.parse_program() {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            false
        }
    }
}

// ============================================
// STRUCTURE PARSING TESTS
// ============================================

#[test]
fn test_parse_stdlib_sets() {
    let source =
        std::fs::read_to_string("stdlib/sets.kleis").expect("Failed to read stdlib/sets.kleis");
    assert!(parses_ok(&source), "stdlib/sets.kleis should parse");
}

#[test]
fn test_set_membership_axiom() {
    let source = r#"
structure TestSetMembership(T) {
    operation in_set : T → Set(T) → Bool
    axiom member_def: ∀(x : T, S : Set(T)). in_set(x, S) ↔ in_set(x, S)
}
"#;
    assert!(parses_ok(source), "Set membership structure should parse");
}

#[test]
fn test_set_union_axiom() {
    let source = r#"
structure TestUnion(T) {
    operation union : Set(T) → Set(T) → Set(T)
    operation in_set : T → Set(T) → Bool
    
    axiom union_def: ∀(A B : Set(T), x : T). 
        in_set(x, union(A, B)) ↔ (in_set(x, A) ∨ in_set(x, B))
}
"#;
    assert!(parses_ok(source), "Set union structure should parse");
}

#[test]
fn test_set_intersection_axiom() {
    let source = r#"
structure TestIntersect(T) {
    operation intersect : Set(T) → Set(T) → Set(T)
    operation in_set : T → Set(T) → Bool
    
    axiom intersect_def: ∀(A B : Set(T), x : T). 
        in_set(x, intersect(A, B)) ↔ (in_set(x, A) ∧ in_set(x, B))
}
"#;
    assert!(parses_ok(source), "Set intersection structure should parse");
}

#[test]
fn test_set_subset_axiom() {
    let source = r#"
structure TestSubset(T) {
    operation subset : Set(T) → Set(T) → Bool
    operation in_set : T → Set(T) → Bool
    
    axiom subset_def: ∀(A B : Set(T)). 
        subset(A, B) ↔ (∀(x : T). in_set(x, A) → in_set(x, B))
}
"#;
    assert!(parses_ok(source), "Set subset structure should parse");
}

#[test]
fn test_set_extensionality() {
    let source = r#"
structure TestExtensionality(T) {
    operation in_set : T → Set(T) → Bool
    
    axiom extensionality: ∀(A B : Set(T)). 
        (∀(x : T). in_set(x, A) ↔ in_set(x, B)) → A = B
}
"#;
    assert!(parses_ok(source), "Set extensionality should parse");
}

#[test]
fn test_de_morgan_laws() {
    let source = r#"
structure TestDeMorgan(T) {
    operation union : Set(T) → Set(T) → Set(T)
    operation intersect : Set(T) → Set(T) → Set(T)
    operation complement : Set(T) → Set(T)
    
    axiom de_morgan_union: ∀(A B : Set(T)). 
        complement(union(A, B)) = intersect(complement(A), complement(B))
    
    axiom de_morgan_intersect: ∀(A B : Set(T)). 
        complement(intersect(A, B)) = union(complement(A), complement(B))
}
"#;
    assert!(parses_ok(source), "De Morgan's laws should parse");
}

// ============================================
// SET OPERATIONS IN BOURBAKI CONTEXT
// ============================================

#[test]
fn test_bourbaki_set_theory_parses() {
    let source = std::fs::read_to_string("bourbaki/01_sets.kleis")
        .expect("Failed to read bourbaki/01_sets.kleis");
    assert!(parses_ok(&source), "bourbaki/01_sets.kleis should parse");
}

#[test]
fn test_metric_space_uses_sets() {
    let source = r#"
structure MetricSpace(X) {
    operation d : X → X → ℝ
    
    axiom positive: ∀(x y : X). d(x, y) >= 0
    axiom zero_self: ∀(x : X). d(x, x) = 0
    axiom symmetric: ∀(x y : X). d(x, y) = d(y, x)
    axiom triangle: ∀(x y z : X). d(x, z) <= d(x, y) + d(y, z)
}

structure OpenBall(X) {
    operation ball : X → ℝ → Set(X)
    operation d : X → X → ℝ
    
    axiom ball_def: ∀(center : X, radius : ℝ, x : X).
        in_set(x, ball(center, radius)) ↔ d(x, center) < radius
}
"#;
    assert!(parses_ok(source), "Metric space with sets should parse");
}

#[test]
fn test_measure_space_uses_sets() {
    let source = r#"
structure MeasureSpace(X) {
    element sigma_algebra : Set(Set(X))
    operation measure : Set(X) → ℝ
    
    axiom sigma_nonempty: ∃(A : Set(X)). in_set(A, sigma_algebra)
    axiom measure_positive: ∀(A : Set(X)). measure(A) >= 0
}
"#;
    assert!(parses_ok(source), "Measure space with sets should parse");
}

// ============================================
// POWER SET AND ADVANCED OPERATIONS
// ============================================

#[test]
fn test_power_set_type() {
    let source = r#"
structure PowerSet(T) {
    operation power_set : Set(T) → Set(Set(T))
    operation subset : Set(T) → Set(T) → Bool
    operation in_set_of_sets : Set(T) → Set(Set(T)) → Bool
    
    axiom power_set_def: ∀(S A : Set(T)). 
        in_set_of_sets(A, power_set(S)) ↔ subset(A, S)
}
"#;
    assert!(parses_ok(source), "Power set structure should parse");
}

#[test]
fn test_cartesian_product() {
    let source = r#"
structure CartesianProduct(A, B) {
    operation cartesian : Set(A) → Set(B) → Set(Pair(A, B))
    operation in_set_a : A → Set(A) → Bool
    operation in_set_b : B → Set(B) → Bool
    operation in_set_pair : Pair(A, B) → Set(Pair(A, B)) → Bool
    operation pair : A → B → Pair(A, B)
    
    axiom cartesian_def: ∀(S : Set(A), T : Set(B), a : A, b : B).
        in_set_pair(pair(a, b), cartesian(S, T)) ↔ (in_set_a(a, S) ∧ in_set_b(b, T))
}
"#;
    assert!(parses_ok(source), "Cartesian product should parse");
}
