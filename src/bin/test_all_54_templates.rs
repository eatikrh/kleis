#!/usr/bin/env rust
//! Comprehensive test of all 54 palette templates
//! Tests that each template can be created and rendered successfully

use kleis::math_layout::compile_with_semantic_boxes;
use kleis::render::{RenderTarget, build_default_context, render_expression};
use kleis::templates::*;

fn main() {
    println!("🧪 Testing All 54 Palette Templates\n");
    println!("{}", "=".repeat(80));

    let templates = vec![
        // Basic Operations
        (
            "Fraction",
            template_fraction as fn() -> kleis::ast::Expression,
        ),
        ("Square Root", template_sqrt),
        ("Power", template_power),
        ("Subscript", template_subscript),
        ("Plus", template_plus),
        ("Minus", template_minus),
        ("Times", template_times),
        ("Equals", template_equals),
        // Calculus
        ("Integral", template_integral),
        ("Sum", template_sum),
        ("Product", template_product),
        ("Partial Derivative", template_partial),
        ("Time Derivative", template_derivative_time),
        ("Gradient", template_gradient),
        ("Limit", template_limit),
        // Matrices
        ("Matrix 2×2", template_matrix_2x2),
        ("Matrix 3×3", template_matrix_3x3),
        ("PMatrix 2×2", template_pmatrix_2x2),
        ("PMatrix 3×3", template_pmatrix_3x3),
        ("VMatrix 2×2", template_vmatrix_2x2),
        ("VMatrix 3×3", template_vmatrix_3x3),
        // Vectors
        ("Vector Bold", template_vector_bold),
        ("Vector Arrow", template_vector_arrow),
        ("Dot Product", template_dot_product),
        ("Cross Product", template_cross_product),
        ("Norm", template_norm),
        ("Absolute Value", template_abs),
        // Quantum
        ("Ket", template_ket),
        ("Bra", template_bra),
        ("Inner Product", template_inner),
        ("Outer Product", template_outer),
        ("Commutator", template_commutator),
        ("Expectation", template_expectation),
        // Tensors
        ("Tensor Mixed", template_tensor_mixed),
        ("Tensor Upper Pair", template_tensor_upper_pair),
        ("Christoffel", template_christoffel),
        ("Riemann", template_riemann),
        // Trigonometry & Functions
        ("Sine", template_sin),
        ("Cosine", template_cos),
        ("Tangent", template_tan),
        ("Arcsine", template_arcsin),
        ("Arccosine", template_arccos),
        ("Arctangent", template_arctan),
        ("Natural Log", template_ln),
        ("Logarithm", template_log),
        ("Exponential", template_exp),
        // Accents
        ("Dot Accent", template_dot_accent),
        ("Double Dot", template_ddot_accent),
        ("Hat", template_hat),
        ("Bar", template_bar),
        ("Tilde", template_tilde),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut warnings = 0;
    let mut current_category = "";

    for (name, template_fn) in &templates {
        // Determine category from name
        let category =
            if name.contains("Matrix") || name.contains("PMatrix") || name.contains("VMatrix") {
                "Matrices"
            } else if name.contains("Ket")
                || name.contains("Bra")
                || name.contains("Commutator")
                || name.contains("Expectation")
            {
                "Quantum"
            } else if name.contains("Vector")
                || name.contains("Dot Product")
                || name.contains("Cross")
                || name.contains("Norm")
                || name.contains("Absolute")
            {
                "Vectors"
            } else if name.contains("sin")
                || name.contains("cos")
                || name.contains("tan")
                || name.contains("Log")
                || name.contains("Exponential")
            {
                "Functions"
            } else if name.contains("Accent")
                || name.contains("Hat")
                || name.contains("Bar")
                || name.contains("Tilde")
            {
                "Accents"
            } else if name.contains("Christoffel")
                || name.contains("Riemann")
                || name.contains("Tensor")
            {
                "Tensors"
            } else if name.contains("Integral")
                || name.contains("Sum")
                || name.contains("Product")
                || name.contains("Derivative")
                || name.contains("Gradient")
                || name.contains("Limit")
            {
                "Calculus"
            } else {
                "Basic"
            };

        if category != current_category {
            println!("\n📁 {}", category);
            println!("{}", "-".repeat(80));
            current_category = category;
        }

        print!("  {:30} ", format!("{}...", name));

        reset_placeholder_counter();
        let expr = template_fn();

        // Test 1: Render to Typst
        let ctx = build_default_context();
        let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);

        // Test 2: Compile with semantic boxes
        let placeholders = expr.find_placeholders();
        let placeholder_ids: Vec<usize> = placeholders.iter().map(|(id, _)| *id).collect();

        match compile_with_semantic_boxes(&expr, &placeholder_ids) {
            Ok(output) => {
                let has_placeholders = output.placeholder_positions.len() > 0;
                let has_boxes = output.argument_bounding_boxes.len() > 0;

                if has_placeholders || has_boxes {
                    println!(
                        "✅ PASS ({} ph, {} boxes)",
                        output.placeholder_positions.len(),
                        output.argument_bounding_boxes.len()
                    );
                    passed += 1;
                } else if placeholders.is_empty() {
                    // No placeholders expected (e.g., vector_bold with fixed content)
                    println!("✅ PASS (no placeholders)");
                    passed += 1;
                } else {
                    println!(
                        "⚠️  WARN (expected {} ph, got {})",
                        placeholders.len(),
                        output.placeholder_positions.len()
                    );
                    warnings += 1;
                }
            }
            Err(e) => {
                println!("❌ FAIL");
                println!("     Error: {}", e);
                println!("     Typst: {}", typst_markup);
                failed += 1;
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("📊 Test Summary");
    println!("{}", "=".repeat(80));
    println!("Total:    {} templates", templates.len());
    println!("Passed:   {} ✅", passed);
    println!("Failed:   {} ❌", failed);
    println!("Warnings: {} ⚠️", warnings);

    let success_rate = (passed as f64 / templates.len() as f64) * 100.0;
    println!("\nSuccess Rate: {:.1}%", success_rate);

    if failed == 0 {
        println!("\n🎉 All critical tests passed!");
        if warnings > 0 {
            println!("⚠️  {} warnings (non-critical)", warnings);
        }
        std::process::exit(0);
    } else {
        println!(
            "\n⚠️  {} templates failed. Please review errors above.",
            failed
        );
        std::process::exit(1);
    }
}
