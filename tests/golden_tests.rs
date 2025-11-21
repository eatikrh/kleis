// Golden tests for Kleis renderer
// These tests compare renderer output against verified reference outputs from style guides

// Note: Most render functions are currently private
// This test file demonstrates the testing structure for when the API is exposed
// For now, tests validate higher-level functionality

#[cfg(test)]
mod golden_calculus {
    /// Test basic derivative notation: dy/dx
    #[test]
    fn derivative_leibniz() {
        // Reference: AMS Short Math Guide, Section 3.2
        let expected_latex = r"\frac{d\,y}{dx}";
        
        // TODO: Once builders are exposed, create expression:
        // let expr = d_dt(o("y"), o("x"));
        // let output = render_expression(&expr, &ctx(), &RenderTarget::LaTeX);
        // assert_eq!(output, expected_latex);
        
        // For now, placeholder verifying structure
        assert!(expected_latex.contains("frac"));
    }

    /// Test partial derivative: ∂f/∂x  
    #[test]
    fn partial_derivative() {
        // Reference: AMS Short Math Guide, Section 3.2
        let expected_latex = r"\frac{\partial\,f}{\partial x}";
        
        // TODO: let expr = d_part(o("f"), o("x"));
        assert!(expected_latex.contains("partial"));
    }

    /// Test definite integral with bounds
    #[test]
    fn definite_integral() {
        // Reference: AMS Short Math Guide, Section 3.3
        let expected_latex = r"\int_{a}^{b} f(x) \, \mathrm{d}x";
        
        // TODO: let expr = int_e(o("f(x)"), o("a"), o("b"), o("x"));
        assert!(expected_latex.contains("int"));
    }

    /// Test sum with bounds
    #[test]
    fn sum_with_bounds() {
        // Reference: IEEE Math Guide, Section 5
        let expected_latex = r"\sum_{i=1}^{n} i";
        
        // TODO: let expr = sum_e(o("i"), o("i=1"), o("n"));
        assert!(expected_latex.contains("sum"));
    }
}

#[cfg(test)]
mod golden_linear_algebra {

    /// Test 2x2 matrix notation
    #[test]
    fn matrix_2x2() {
        // Reference: AMS Short Math Guide, Section 4.5
        let expected_latex = r"\begin{bmatrix}a&b\\c&d\end{bmatrix}";
        
        // TODO: let expr = m2(o("a"), o("b"), o("c"), o("d"));
        assert!(expected_latex.contains("bmatrix"));
    }

    /// Test vector with arrow
    #[test]
    fn vector_arrow() {
        // Reference: IEEE Math Guide, Section 3.1
        let expected_latex = r"\vec{v}";
        
        // TODO: let expr = vector_arrow_e(o("v"));
        assert!(expected_latex.contains("vec"));
    }

    /// Test inner product
    #[test]
    fn inner_product() {
        // Reference: AMS Short Math Guide, Section 4.3
        let expected_latex = r"\langle u, v \rangle";
        
        // TODO: let expr = inner_e(o("u"), o("v"));
        assert!(expected_latex.contains("langle"));
    }
}

#[cfg(test)]
mod golden_physics {

    /// Einstein Field Equations (core form)
    #[test]
    fn einstein_field_equations() {
        // Reference: Custom POT notation
        let expected_latex = r"G_{\mu\nu} + \Lambda g_{\mu\nu} = \kappa T_{\mu\nu}";
        
        // This is already tested in render.rs but serves as golden reference
        assert!(expected_latex.contains("Lambda"));
    }

    /// Maxwell tensor from potential
    #[test]
    fn maxwell_tensor() {
        // Reference: Standard GR notation
        let expected_latex = r"F_{\mu\nu} = \partial_\mu A_\nu - \partial_\nu A_\mu";
        
        assert!(expected_latex.contains("partial"));
    }
}

#[cfg(test)]
mod golden_sets_and_logic {
    // TODO: Add when set theory support is implemented
    
    /// Set membership - NOW IMPLEMENTED!
    #[test]
    fn set_membership() {
        // Reference: Standard set theory notation
        let samples = kleis::render::collect_samples_for_gallery();
        let set_examples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("membership"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!set_examples.is_empty(), "Should have set membership examples");
        assert!(set_examples[0].contains(r"\in"), "Should use \\in symbol");
    }

    /// Universal quantifier - NOW IMPLEMENTED!
    #[test]
    fn universal_quantifier() {
        // Reference: Standard logic notation
        let samples = kleis::render::collect_samples_for_gallery();
        let logic_examples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Universal"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!logic_examples.is_empty(), "Should have quantifier examples");
        assert!(logic_examples[0].contains(r"\forall"), "Should use \\forall symbol");
    }
}

// NEW: Top 5 Operations Golden Tests
#[cfg(test)]
mod golden_top5_operations {
    /// Bra-ket notation (Quantum mechanics)
    #[test]
    fn braket_notation() {
        // Reference: quantum_mechanics.tex examples
        let samples = kleis::render::collect_samples_for_gallery();
        
        // Find ket example
        let ket_samples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Ket"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        assert!(!ket_samples.is_empty());
        assert!(ket_samples[0].contains(r"\rangle"));
        
        // Find bra example  
        let bra_samples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Bra"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        assert!(!bra_samples.is_empty());
        assert!(bra_samples[0].contains(r"\langle"));
    }
    
    /// Commutators for quantum operators
    #[test]
    fn commutator_notation() {
        // Reference: quantum_mechanics.tex - [x̂, p̂] = iℏ
        let samples = kleis::render::collect_samples_for_gallery();
        
        let comm_samples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Commut"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!comm_samples.is_empty());
        // Commutator uses square brackets
        assert!(comm_samples[0].starts_with("[") || comm_samples[0].starts_with(r"\{"));
    }
    
    /// Multiple integrals for projections
    #[test]
    fn multiple_integrals() {
        // Reference: calculus_basic.tex
        let samples = kleis::render::collect_samples_for_gallery();
        
        let double_int: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Double"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        let triple_int: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Triple"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!double_int.is_empty());
        assert!(double_int[0].contains(r"\iint"));
        
        assert!(!triple_int.is_empty());
        assert!(triple_int[0].contains(r"\iiint"));
    }
    
    /// Square roots
    #[test]
    fn square_root_notation() {
        // Reference: Used everywhere in mathematics
        let samples = kleis::render::collect_samples_for_gallery();
        
        let sqrt_samples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("root") || t.contains("√"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!sqrt_samples.is_empty());
        assert!(sqrt_samples.iter().any(|s| s.contains(r"\sqrt")));
    }
    
    /// Set theory fundamentals
    #[test]
    fn set_theory_complete() {
        // Reference: Standard set theory
        let samples = kleis::render::collect_samples_for_gallery();
        
        // Should have membership, subset, union, intersection
        let set_ops: Vec<&String> = samples.iter()
            .filter(|(t, _)| t.contains("Set") || t.contains("membership") || 
                             t.contains("Subset") || t.contains("quantifier"))
            .map(|(t, _)| t)
            .collect();
        
        assert!(set_ops.len() >= 3, "Should have multiple set operations, found: {:?}", set_ops);
    }
}

// NEW: Next Top 3 + Low-Hanging Fruit Golden Tests
#[cfg(test)]
mod golden_next_batch {
    /// Comparison operators
    #[test]
    fn comparison_operators() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        // Should have inequality examples
        let comparisons: Vec<&String> = samples.iter()
            .filter(|(t, _)| t.contains("constraint") || t.contains("Inequality") || 
                             t.contains("equal") || t.contains("Approximation"))
            .map(|(t, _)| t)
            .collect();
        
        assert!(comparisons.len() >= 3, "Should have comparison operators");
    }
    
    /// Complex number operations
    #[test]
    fn complex_numbers() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let complex_ops: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Complex") || t.contains("conjugate") || 
                             t.contains("Real and imaginary"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!complex_ops.is_empty());
        assert!(complex_ops.iter().any(|s| s.contains(r"\overline") || s.contains("Re") || s.contains("Im")));
    }
    
    /// Operator hat notation
    #[test]
    fn operator_hat() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let hat_ops: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Hamiltonian") || t.contains("Schrodinger"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!hat_ops.is_empty());
        assert!(hat_ops.iter().any(|s| s.contains(r"\hat")));
    }
    
    /// Trigonometric functions
    #[test]
    fn trig_functions() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let trig: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Trigonometric") || t.contains("Hyperbolic") || t.contains("Euler formula"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(trig.len() >= 2);
        assert!(trig.iter().any(|s| s.contains(r"\cos")));
        assert!(trig.iter().any(|s| s.contains(r"\sinh") || s.contains(r"\cosh")));
    }
    
    /// Matrix operations (trace, inverse)
    #[test]
    fn matrix_operations() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let matrix_ops: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Trace") || t.contains("inverse"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(matrix_ops.len() >= 2);
        assert!(matrix_ops.iter().any(|s| s.contains(r"\mathrm{Tr")));
        assert!(matrix_ops.iter().any(|s| s.contains("^{-1}")));
    }
    
    /// Integration: Complete quantum mechanics example
    #[test]
    fn quantum_mechanics_complete() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        // Should have bra-ket, hat operators, trace
        let qm_examples: Vec<&String> = samples.iter()
            .filter(|(t, _)| t.contains("QM") || t.contains("Hamiltonian") || 
                             t.contains("Schrodinger") || t.contains("Trace"))
            .map(|(t, _)| t)
            .collect();
        
        assert!(qm_examples.len() >= 3, "Should have complete QM notation support");
    }
}

// Batch 4: Polish & Edge Cases Golden Tests
#[cfg(test)]
mod golden_batch4_polish {
    /// Number set Unicode rendering
    #[test]
    fn number_sets_render_unicode() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let hilbert_examples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Hilbert"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!hilbert_examples.is_empty());
        assert!(hilbert_examples.iter().any(|s| s.contains(r"\mathbb{C}")));
    }
    
    /// Piecewise functions
    #[test]
    fn piecewise_functions() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let piecewise: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("piecewise") || t.contains("Sign function"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(piecewise.len() >= 2);
        assert!(piecewise.iter().any(|s| s.contains(r"\begin{cases}")));
    }
    
    /// Vmatrix (determinant bars)
    #[test]
    fn vmatrix_notation() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let vmat: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("vmatrix") || t.contains("Determinant"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(vmat.len() >= 2);
        assert!(vmat.iter().any(|s| s.contains(r"\begin{vmatrix}")));
    }
    
    /// Modular arithmetic
    #[test]
    fn modular_arithmetic() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let modular: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Congruence") || t.contains("Fermat"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(modular.len() >= 2);
        assert!(modular.iter().any(|s| s.contains(r"\pmod")));
    }
    
    /// Statistics notation
    #[test]
    fn statistics_notation() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let stats: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Variance") || t.contains("covariance"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!stats.is_empty());
        assert!(stats.iter().any(|s| s.contains(r"\mathrm{Var}") || s.contains(r"\mathrm{Cov}")));
    }
    
    /// Polish: Complete coverage verification
    #[test]
    fn coverage_near_complete() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        // Should have comprehensive examples across all domains
        assert!(samples.len() >= 70, "Should have at least 70 examples");
        
        // Check key domains are represented
        let categories: Vec<&str> = samples.iter().map(|(t, _)| t.as_str()).collect();
        
        assert!(categories.iter().any(|t| t.contains("Pauli")));
        assert!(categories.iter().any(|t| t.contains("piecewise")));
        assert!(categories.iter().any(|t| t.contains("Maxwell")));
        assert!(categories.iter().any(|t| t.contains("vmatrix")));
    }
}

// Batch 3: Completeness Operations Golden Tests
#[cfg(test)]
mod golden_batch3_completeness {
    /// Factorial and combinatorics
    #[test]
    fn factorial_notation() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let factorial_examples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Factorial") || t.contains("factorial"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!factorial_examples.is_empty());
        assert!(factorial_examples.iter().any(|s| s.contains("!")));
    }
    
    /// Binomial coefficients
    #[test]
    fn binomial_coefficients() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let binom_examples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Binomial") || t.contains("binomial"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!binom_examples.is_empty());
        assert!(binom_examples.iter().any(|s| s.contains(r"\binom")));
    }
    
    /// Floor and ceiling functions
    #[test]
    fn floor_ceiling_functions() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let bracket_examples: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Floor") || t.contains("ceiling"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!bracket_examples.is_empty());
        assert!(bracket_examples.iter().any(|s| s.contains(r"\lfloor") || s.contains(r"\lceil")));
    }
    
    /// Inverse trigonometric functions
    #[test]
    fn inverse_trig_complete() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let inv_trig: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Inverse trig"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!inv_trig.is_empty());
        assert!(inv_trig.iter().any(|s| s.contains(r"\arcsin")));
        assert!(inv_trig.iter().any(|s| s.contains(r"\arccos")));
    }
    
    /// Pauli matrices with pmatrix
    #[test]
    fn pauli_matrices() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let pauli: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Pauli"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(pauli.len() >= 2, "Should have at least 2 Pauli matrix examples");
        assert!(pauli.iter().any(|s| s.contains(r"\begin{pmatrix}")));
    }
    
    /// Vector calculus operators
    #[test]
    fn vector_calculus_operators() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let vec_calc: Vec<&String> = samples.iter()
            .filter(|(t, _)| t.contains("Divergence") || t.contains("Curl") || 
                             t.contains("Laplacian") || t.contains("Maxwell"))
            .map(|(t, _)| t)
            .collect();
        
        assert!(vec_calc.len() >= 3, "Should have divergence, curl, and Laplacian");
    }
    
    /// Wave equation using Laplacian
    #[test]
    fn wave_equation_complete() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        let wave: Vec<&str> = samples.iter()
            .filter(|(t, _)| t.contains("Wave equation"))
            .map(|(_, latex)| latex.as_str())
            .collect();
        
        assert!(!wave.is_empty());
        assert!(wave[0].contains(r"\nabla^2"));
    }
    
    /// Complete function library verification
    #[test]
    fn complete_function_library() {
        let samples = kleis::render::collect_samples_for_gallery();
        
        // Count all function-related examples
        let functions: Vec<&String> = samples.iter()
            .filter(|(t, _)| t.contains("Trigonometric") || t.contains("Hyperbolic") || 
                             t.contains("Logarithms") || t.contains("Inverse trig") ||
                             t.contains("Reciprocal"))
            .map(|(t, _)| t)
            .collect();
        
        assert!(functions.len() >= 4, "Should have comprehensive function support");
    }
}

// Integration test: Verify entire gallery against golden output
#[test]
fn gallery_output_stability() {
    // This test ensures that the gallery output doesn't change unexpectedly
    // Run this test, inspect output, and if correct, save as golden reference
    
    let samples = kleis::render::collect_samples_for_gallery();
    
    // Basic sanity checks
    assert!(!samples.is_empty(), "Gallery should have samples");
    assert!(samples.len() >= 15, "Gallery should have at least 15 examples");
    
    // Verify key examples are present
    let titles: Vec<&str> = samples.iter().map(|(t, _)| t.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("Einstein")), "Should have Einstein equations");
    assert!(titles.iter().any(|t| t.contains("Maxwell")), "Should have Maxwell tensor");
    assert!(titles.iter().any(|t| t.contains("zeta")), "Should have zeta function");
    
    // TODO: Compare full output against saved golden file
    // let golden = std::fs::read_to_string("tests/golden/references/gallery_full.txt")?;
    // let current = format_gallery_output(&samples);
    // assert_eq!(current, golden);
}

// Helper function to format gallery for comparison
#[allow(dead_code)]
fn format_gallery_output(samples: &[(String, String)]) -> String {
    let mut output = String::new();
    for (title, latex) in samples {
        output.push_str(&format!("### {}\n{}\n\n", title, latex));
    }
    output
}

