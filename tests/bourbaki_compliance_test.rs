//! Bourbaki Compliance Tests
//!
//! Tests that verify Kleis can express mathematical structures from
//! Bourbaki's *Éléments de mathématique*.
//!
//! Reference: docs/CAPABILITY_ASSESSMENT.md

use kleis::kleis_parser::parse_kleis_program;

/// Helper to check if a program parses successfully
fn parses_ok(source: &str) -> bool {
    parse_kleis_program(source).is_ok()
}

// =============================================================================
// Volume I: Theory of Sets
// =============================================================================

#[test]
fn test_set_membership() {
    let source = r#"
structure SetTheory {
    axiom membership: ∀(x : T, S : Set(T)). in_set(x, S) = in_set(x, S)
}
"#;
    assert!(parses_ok(source), "Should express set membership");
}

#[test]
fn test_set_extensionality() {
    let source = r#"
structure SetTheory {
    axiom extensionality: ∀(A B : Set(T)). 
        (∀(x : T). in_set(x, A) ↔ in_set(x, B)) → A = B
}
"#;
    assert!(parses_ok(source), "Should express set extensionality");
}

#[test]
fn test_subset_definition() {
    let source = r#"
structure SetTheory {
    axiom subset_def: ∀(A B : Set(T)). 
        subset(A, B) ↔ (∀(x : T). in_set(x, A) → in_set(x, B))
}
"#;
    assert!(parses_ok(source), "Should express subset definition");
}

#[test]
fn test_union_definition() {
    let source = r#"
structure SetTheory {
    axiom union_def: ∀(A B : Set(T), x : T). 
        in_set(x, union(A, B)) ↔ (in_set(x, A) ∨ in_set(x, B))
}
"#;
    assert!(parses_ok(source), "Should express union definition");
}

#[test]
fn test_intersection_definition() {
    let source = r#"
structure SetTheory {
    axiom intersect_def: ∀(A B : Set(T), x : T). 
        in_set(x, intersect(A, B)) ↔ (in_set(x, A) ∧ in_set(x, B))
}
"#;
    assert!(parses_ok(source), "Should express intersection definition");
}

#[test]
fn test_power_set_definition() {
    let source = r#"
structure SetTheory {
    axiom power_set_def: ∀(S A : Set(T)). 
        in_set(A, power_set(S)) ↔ subset(A, S)
}
"#;
    assert!(parses_ok(source), "Should express power set definition");
}

#[test]
fn test_empty_set() {
    let source = r#"
structure SetTheory {
    axiom empty_set: ∀(x : T). ¬in_set(x, empty_set)
}
"#;
    assert!(parses_ok(source), "Should express empty set");
}

// =============================================================================
// Volume II: Algebra
// =============================================================================

#[test]
fn test_group_axioms() {
    let source = r#"
structure Group(G) {
    operation (•) : G × G → G
    element e : G
    operation inv : G → G
    
    axiom associativity: ∀(a b c : G). (a • b) • c = a • (b • c)
    axiom identity: ∀(a : G). a • e = a ∧ e • a = a
    axiom inverse: ∀(a : G). a • inv(a) = e ∧ inv(a) • a = e
}
"#;
    assert!(parses_ok(source), "Should express group axioms");
}

#[test]
fn test_abelian_group() {
    let source = r#"
structure AbelianGroup(G) {
    operation (+) : G × G → G
    element zero : G
    operation neg : G → G
    
    axiom associativity: ∀(a b c : G). (a + b) + c = a + (b + c)
    axiom commutativity: ∀(a b : G). a + b = b + a
    axiom identity: ∀(a : G). a + zero = a
    axiom inverse: ∀(a : G). a + neg(a) = zero
}
"#;
    assert!(parses_ok(source), "Should express abelian group axioms");
}

#[test]
fn test_ring_axioms() {
    let source = r#"
structure Ring(R) {
    operation (+) : R × R → R
    operation (*) : R × R → R
    element zero : R
    element one : R
    operation neg : R → R
    
    axiom add_assoc: ∀(a b c : R). (a + b) + c = a + (b + c)
    axiom add_comm: ∀(a b : R). a + b = b + a
    axiom add_identity: ∀(a : R). a + zero = a
    axiom add_inverse: ∀(a : R). a + neg(a) = zero
    axiom mul_assoc: ∀(a b c : R). (a * b) * c = a * (b * c)
    axiom mul_identity: ∀(a : R). a * one = a ∧ one * a = a
    axiom distributive_left: ∀(a b c : R). a * (b + c) = a * b + a * c
    axiom distributive_right: ∀(a b c : R). (a + b) * c = a * c + b * c
}
"#;
    assert!(parses_ok(source), "Should express ring axioms");
}

#[test]
fn test_field_axioms() {
    let source = r#"
structure Field(F) {
    operation (+) : F × F → F
    operation (*) : F × F → F
    element zero : F
    element one : F
    operation neg : F → F
    operation inv : F → F
    
    axiom add_assoc: ∀(a b c : F). (a + b) + c = a + (b + c)
    axiom add_comm: ∀(a b : F). a + b = b + a
    axiom add_identity: ∀(a : F). a + zero = a
    axiom add_inverse: ∀(a : F). a + neg(a) = zero
    axiom mul_assoc: ∀(a b c : F). (a * b) * c = a * (b * c)
    axiom mul_comm: ∀(a b : F). a * b = b * a
    axiom mul_identity: ∀(a : F). a * one = a
    axiom mul_inverse: ∀(a : F) where a ≠ zero. a * inv(a) = one
    axiom distributive: ∀(a b c : F). a * (b + c) = a * b + a * c
    axiom nontrivial: zero ≠ one
}
"#;
    assert!(parses_ok(source), "Should express field axioms");
}

#[test]
fn test_vector_space_axioms() {
    let source = r#"
structure VectorSpace(V, F) {
    operation (+) : V × V → V
    operation (·) : F × V → V
    element zero : V
    
    axiom vec_add_assoc: ∀(u v w : V). (u + v) + w = u + (v + w)
    axiom vec_add_comm: ∀(u v : V). u + v = v + u
    axiom vec_add_identity: ∀(v : V). v + zero = v
    axiom scalar_assoc: ∀(a b : F, v : V). a · (b · v) = (a * b) · v
    axiom scalar_identity: ∀(v : V). one · v = v
    axiom distributive_scalar: ∀(a : F, u v : V). a · (u + v) = a · u + a · v
    axiom distributive_field: ∀(a b : F, v : V). (a + b) · v = a · v + b · v
}
"#;
    assert!(parses_ok(source), "Should express vector space axioms");
}

#[test]
fn test_module_axioms() {
    let source = r#"
structure Module(M, R) {
    operation (+) : M × M → M
    operation (·) : R × M → M
    element zero : M
    
    axiom mod_add_assoc: ∀(x y z : M). (x + y) + z = x + (y + z)
    axiom mod_add_comm: ∀(x y : M). x + y = y + x
    axiom mod_add_identity: ∀(x : M). x + zero = x
    axiom scalar_assoc: ∀(r s : R, x : M). r · (s · x) = (r * s) · x
    axiom scalar_identity: ∀(x : M). one · x = x
    axiom distributive_mod: ∀(r : R, x y : M). r · (x + y) = r · x + r · y
    axiom distributive_ring: ∀(r s : R, x : M). (r + s) · x = r · x + s · x
}
"#;
    assert!(parses_ok(source), "Should express module axioms");
}

// =============================================================================
// Volume III: General Topology
// =============================================================================

#[test]
fn test_topological_space() {
    let source = r#"
structure TopologicalSpace(X) {
    element tau : Set(Set(X))
    
    axiom empty_open: in_set(empty_set, tau)
    axiom full_open: in_set(X, tau)
    axiom union_closed: ∀(U V : Set(X)). 
        (in_set(U, tau) ∧ in_set(V, tau)) → in_set(union(U, V), tau)
    axiom intersection_closed: ∀(U V : Set(X)). 
        (in_set(U, tau) ∧ in_set(V, tau)) → in_set(intersect(U, V), tau)
}
"#;
    assert!(parses_ok(source), "Should express topological space axioms");
}

#[test]
fn test_continuous_function() {
    let source = r#"
structure Continuous(X, Y) {
    axiom preimage_open: ∀(f : X → Y, V : Set(Y)). 
        is_open(V) → is_open(preimage(f, V))
}
"#;
    assert!(
        parses_ok(source),
        "Should express continuous function definition"
    );
}

#[test]
fn test_hausdorff_space() {
    let source = r#"
structure HausdorffSpace(X) {
    axiom separation: ∀(x y : X). x ≠ y → 
        (∃(U V : Set(X)). is_open(U) ∧ is_open(V) ∧ 
            in_set(x, U) ∧ in_set(y, V) ∧ intersect(U, V) = empty_set)
}
"#;
    assert!(
        parses_ok(source),
        "Should express Hausdorff separation axiom"
    );
}

#[test]
fn test_compact_space() {
    let source = r#"
structure CompactSpace(X) {
    axiom finite_subcover: ∀(C : Set(Set(X))). 
        is_open_cover(C, X) → (∃(F : Set(Set(X))). is_finite(F) ∧ subset(F, C) ∧ is_cover(F, X))
}
"#;
    assert!(parses_ok(source), "Should express compactness");
}

#[test]
fn test_connected_space() {
    let source = r#"
structure ConnectedSpace(X) {
    axiom no_separation: ∀(U V : Set(X)). 
        (is_open(U) ∧ is_open(V) ∧ union(U, V) = X ∧ intersect(U, V) = empty_set) →
        (U = empty_set ∨ V = empty_set)
}
"#;
    assert!(parses_ok(source), "Should express connectedness");
}

// =============================================================================
// Volume IV: Functions of a Real Variable (Analysis)
// =============================================================================

#[test]
fn test_metric_space() {
    let source = r#"
structure MetricSpace(X) {
    operation d : X × X → ℝ
    
    axiom non_negative: ∀(x y : X). d(x, y) >= 0
    axiom identity: ∀(x y : X). d(x, y) = 0 ↔ x = y
    axiom symmetry: ∀(x y : X). d(x, y) = d(y, x)
    axiom triangle: ∀(x y z : X). d(x, z) <= d(x, y) + d(y, z)
}
"#;
    assert!(parses_ok(source), "Should express metric space axioms");
}

#[test]
fn test_epsilon_delta_limit() {
    let source = r#"
structure Limits {
    axiom limit_def: ∀(f : ℝ → ℝ, L a : ℝ). 
        has_limit(f, a, L) ↔ 
        (∀(ε : ℝ). ε > 0 → (∃(δ : ℝ). δ > 0 ∧ 
            (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε)))
}
"#;
    assert!(
        parses_ok(source),
        "Should express epsilon-delta limit definition"
    );
}

#[test]
fn test_continuity_at_point() {
    let source = r#"
structure Continuity {
    axiom continuous_at: ∀(f : ℝ → ℝ, a : ℝ). 
        is_continuous_at(f, a) ↔ 
        (∀(ε : ℝ). ε > 0 → (∃(δ : ℝ). δ > 0 ∧ 
            (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - f(a)) < ε)))
}
"#;
    assert!(parses_ok(source), "Should express continuity at a point");
}

#[test]
fn test_uniform_continuity() {
    let source = r#"
structure UniformContinuity {
    axiom uniform_continuous: ∀(f : ℝ → ℝ). 
        is_uniformly_continuous(f) ↔ 
        (∀(ε : ℝ). ε > 0 → (∃(δ : ℝ). δ > 0 ∧ 
            (∀(x y : ℝ). abs(x - y) < δ → abs(f(x) - f(y)) < ε)))
}
"#;
    assert!(parses_ok(source), "Should express uniform continuity");
}

#[test]
fn test_sequence_convergence() {
    let source = r#"
structure Sequences {
    axiom converges: ∀(a : ℕ → ℝ, L : ℝ). 
        converges_to(a, L) ↔ 
        (∀(ε : ℝ). ε > 0 → (∃(N : ℕ). ∀(n : ℕ). n > N → abs(a(n) - L) < ε))
}
"#;
    assert!(parses_ok(source), "Should express sequence convergence");
}

#[test]
fn test_cauchy_sequence() {
    let source = r#"
structure CauchySequences {
    axiom cauchy: ∀(a : ℕ → ℝ). 
        is_cauchy(a) ↔ 
        (∀(ε : ℝ). ε > 0 → (∃(N : ℕ). ∀(m n : ℕ). m > N ∧ n > N → abs(a(m) - a(n)) < ε))
}
"#;
    assert!(parses_ok(source), "Should express Cauchy sequence");
}

#[test]
fn test_completeness() {
    let source = r#"
structure Complete {
    axiom completeness: ∀(a : ℕ → ℝ). 
        is_cauchy(a) → (∃(L : ℝ). converges_to(a, L))
}
"#;
    assert!(parses_ok(source), "Should express completeness");
}

// =============================================================================
// Volume V: Topological Vector Spaces
// =============================================================================

#[test]
fn test_normed_space() {
    let source = r#"
structure NormedSpace(V) {
    operation norm : V → ℝ
    
    axiom non_negative: ∀(v : V). norm(v) >= 0
    axiom zero_norm: ∀(v : V). norm(v) = 0 ↔ v = zero
    axiom scalar_mult: ∀(a : ℝ, v : V). norm(a · v) = abs(a) * norm(v)
    axiom triangle: ∀(u v : V). norm(u + v) <= norm(u) + norm(v)
}
"#;
    assert!(parses_ok(source), "Should express normed space axioms");
}

#[test]
fn test_inner_product_space() {
    let source = r#"
structure InnerProductSpace(V) {
    operation inner : V × V → ℝ
    
    axiom linearity_first: ∀(u v w : V, a : ℝ). 
        inner(a · u + v, w) = a * inner(u, w) + inner(v, w)
    axiom symmetry: ∀(u v : V). inner(u, v) = inner(v, u)
    axiom positive_definite: ∀(v : V). inner(v, v) >= 0
    axiom zero_inner: ∀(v : V). inner(v, v) = 0 ↔ v = zero
}
"#;
    assert!(
        parses_ok(source),
        "Should express inner product space axioms"
    );
}

#[test]
fn test_banach_space() {
    let source = r#"
structure BanachSpace(V) {
    axiom complete: ∀(seq : ℕ → V). 
        is_cauchy_norm(seq) → (∃(L : V). converges_to_norm(seq, L))
}
"#;
    assert!(
        parses_ok(source),
        "Should express Banach space (complete normed space)"
    );
}

#[test]
fn test_hilbert_space() {
    let source = r#"
structure HilbertSpace(H) {
    axiom complete: ∀(seq : ℕ → H). 
        is_cauchy_inner(seq) → (∃(L : H). converges_to_inner(seq, L))
}
"#;
    assert!(parses_ok(source), "Should express Hilbert space");
}

// =============================================================================
// Dependent Types (Partial support)
// =============================================================================

#[test]
fn test_vector_type() {
    let source = r#"
structure LinearAlgebra {
    axiom vector_add: ∀(u v : Vector(n, ℝ)). u + v = v + u
}
"#;
    assert!(
        parses_ok(source),
        "Should express vectors with dimension parameter"
    );
}

#[test]
fn test_matrix_type() {
    let source = r#"
structure Matrices {
    axiom matrix_add: ∀(A B : Matrix(m, n, ℝ)). A + B = B + A
}
"#;
    assert!(
        parses_ok(source),
        "Should express matrices with dimension parameters"
    );
}

#[test]
fn test_matrix_multiplication_typing() {
    let source = r#"
structure MatrixMul {
    axiom mul_compat: ∀(A : Matrix(m, n, ℝ), B : Matrix(n, p, ℝ)). 
        mul(A, B) = mul(A, B)
}
"#;
    assert!(
        parses_ok(source),
        "Should express matrix multiplication dimension compatibility"
    );
}

// =============================================================================
// Complex Analysis
// =============================================================================

#[test]
fn test_complex_numbers() {
    let source = r#"
structure ComplexNumbers {
    axiom field: ∀(z w : ℂ). z + w = w + z
    axiom imaginary: i * i = neg(one)
    axiom conjugate: ∀(z : ℂ). z * conj(z) = norm_sq(z)
}
"#;
    assert!(
        parses_ok(source),
        "Should express complex number properties"
    );
}

#[test]
fn test_holomorphic_function() {
    let source = r#"
structure Holomorphic {
    axiom cauchy_riemann: ∀(f : ℂ → ℂ, z : ℂ). 
        is_holomorphic(f, z) ↔ 
        (∃(f_prime : ℂ). 
            (∀(ε : ℝ). ε > 0 → (∃(δ : ℝ). δ > 0 ∧ 
                (∀(h : ℂ). abs(h) < δ ∧ h ≠ zero → 
                    abs((f(z + h) - f(z)) / h - f_prime) < ε))))
}
"#;
    assert!(
        parses_ok(source),
        "Should express holomorphic function definition"
    );
}
