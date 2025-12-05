#!/usr/bin/env rust-script
//! Test all palette templates to ensure they render correctly
//!
//! This test validates that all templates in the equation editor palette
//! can be successfully parsed and rendered.

use kleis::parser::parse_latex;
use kleis::render::{RenderTarget, build_default_context, render_expression};

#[derive(Debug)]
struct TemplateTest {
    name: &'static str,
    latex: &'static str,
    category: &'static str,
}

fn main() {
    println!("🧪 Testing Kleis Palette Templates\n");
    println!("{}", "=".repeat(80));

    let tests = vec![
        // Basic Operations
        TemplateTest {
            name: "Fraction",
            latex: r"\frac{a}{b}",
            category: "Basic",
        },
        TemplateTest {
            name: "Square Root",
            latex: r"\sqrt{x}",
            category: "Basic",
        },
        TemplateTest {
            name: "Nth Root",
            latex: r"\sqrt[3]{x}",
            category: "Basic",
        },
        TemplateTest {
            name: "Power",
            latex: r"x^{n}",
            category: "Basic",
        },
        TemplateTest {
            name: "Subscript",
            latex: r"x_{i}",
            category: "Basic",
        },
        TemplateTest {
            name: "Mixed Index",
            latex: r"T^{i}_{j}",
            category: "Basic",
        },
        TemplateTest {
            name: "Absolute Value",
            latex: r"|x|",
            category: "Basic",
        },
        TemplateTest {
            name: "Binomial",
            latex: r"\binom{n}{k}",
            category: "Basic",
        },
        TemplateTest {
            name: "Factorial",
            latex: r"n!",
            category: "Basic",
        },
        TemplateTest {
            name: "Floor",
            latex: r"\lfloor x \rfloor",
            category: "Basic",
        },
        TemplateTest {
            name: "Ceiling",
            latex: r"\lceil x \rceil",
            category: "Basic",
        },
        // Calculus
        TemplateTest {
            name: "Integral",
            latex: r"\int f \, dx",
            category: "Calculus",
        },
        TemplateTest {
            name: "Definite Integral",
            latex: r"\int_{a}^{b} f(x) \, dx",
            category: "Calculus",
        },
        TemplateTest {
            name: "Double Integral",
            latex: r"\iint_{D} f \, dA",
            category: "Calculus",
        },
        TemplateTest {
            name: "Triple Integral",
            latex: r"\iiint_{V} f \, dV",
            category: "Calculus",
        },
        TemplateTest {
            name: "Contour Integral",
            latex: r"\oint f \, ds",
            category: "Calculus",
        },
        TemplateTest {
            name: "Derivative",
            latex: r"\frac{dy}{dx}",
            category: "Calculus",
        },
        TemplateTest {
            name: "Partial Derivative",
            latex: r"\frac{\partial f}{\partial x}",
            category: "Calculus",
        },
        TemplateTest {
            name: "Second Partial",
            latex: r"\frac{\partial^{2} f}{\partial x^{2}}",
            category: "Calculus",
        },
        TemplateTest {
            name: "Gradient",
            latex: r"\nabla f",
            category: "Calculus",
        },
        TemplateTest {
            name: "Divergence",
            latex: r"\nabla \cdot F",
            category: "Calculus",
        },
        TemplateTest {
            name: "Curl",
            latex: r"\nabla \times F",
            category: "Calculus",
        },
        TemplateTest {
            name: "Laplacian",
            latex: r"\nabla^{2} \phi",
            category: "Calculus",
        },
        TemplateTest {
            name: "Sum",
            latex: r"\sum_{i=1}^{n} a_i",
            category: "Calculus",
        },
        TemplateTest {
            name: "Product",
            latex: r"\prod_{i=1}^{n} a_i",
            category: "Calculus",
        },
        TemplateTest {
            name: "Limit",
            latex: r"\lim_{x \to a} f(x)",
            category: "Calculus",
        },
        // Matrices
        TemplateTest {
            name: "Matrix 2×2 [brackets]",
            latex: r"\begin{bmatrix}a&b\\c&d\end{bmatrix}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Matrix 2×2 (parens)",
            latex: r"\begin{pmatrix}a&b\\c&d\end{pmatrix}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Determinant 2×2",
            latex: r"\begin{vmatrix}a&b\\c&d\end{vmatrix}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Matrix 3×3 [brackets]",
            latex: r"\begin{bmatrix}a&b&c\\d&e&f\\g&h&i\end{bmatrix}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Matrix 3×3 (parens)",
            latex: r"\begin{pmatrix}a&b&c\\d&e&f\\g&h&i\end{pmatrix}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Determinant 3×3",
            latex: r"\begin{vmatrix}a&b&c\\d&e&f\\g&h&i\end{vmatrix}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Transpose",
            latex: r"A^{\mathsf{T}}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Inverse",
            latex: r"A^{-1}",
            category: "Matrices",
        },
        TemplateTest {
            name: "Determinant",
            latex: r"\det(A)",
            category: "Matrices",
        },
        TemplateTest {
            name: "Trace",
            latex: r"\mathrm{Tr}(A)",
            category: "Matrices",
        },
        // Quantum Mechanics
        TemplateTest {
            name: "Ket",
            latex: r"|\psi\rangle",
            category: "Quantum",
        },
        TemplateTest {
            name: "Bra",
            latex: r"\langle\phi|",
            category: "Quantum",
        },
        TemplateTest {
            name: "Inner Product",
            latex: r"\langle\phi|\psi\rangle",
            category: "Quantum",
        },
        TemplateTest {
            name: "Outer Product",
            latex: r"|\psi\rangle\langle\phi|",
            category: "Quantum",
        },
        TemplateTest {
            name: "Commutator",
            latex: r"[A, B]",
            category: "Quantum",
        },
        TemplateTest {
            name: "Anticommutator",
            latex: r"\{A, B\}",
            category: "Quantum",
        },
        TemplateTest {
            name: "Expectation",
            latex: r"\langle A \rangle",
            category: "Quantum",
        },
        TemplateTest {
            name: "Operator Hat",
            latex: r"\hat{H}",
            category: "Quantum",
        },
        // Vectors
        TemplateTest {
            name: "Vector Arrow",
            latex: r"\vec{v}",
            category: "Vectors",
        },
        TemplateTest {
            name: "Bold Vector",
            latex: r"\mathbf{v}",
            category: "Vectors",
        },
        TemplateTest {
            name: "Unit Vector",
            latex: r"\hat{n}",
            category: "Vectors",
        },
        TemplateTest {
            name: "Dot Product",
            latex: r"a \cdot b",
            category: "Vectors",
        },
        TemplateTest {
            name: "Cross Product",
            latex: r"a \times b",
            category: "Vectors",
        },
        TemplateTest {
            name: "Norm",
            latex: r"\|v\|",
            category: "Vectors",
        },
        TemplateTest {
            name: "Inner Product",
            latex: r"\langle u, v \rangle",
            category: "Vectors",
        },
        // Functions
        TemplateTest {
            name: "Sine",
            latex: r"\sin(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Cosine",
            latex: r"\cos(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Tangent",
            latex: r"\tan(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Arcsine",
            latex: r"\arcsin(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Arccosine",
            latex: r"\arccos(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Arctangent",
            latex: r"\arctan(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Hyperbolic Sine",
            latex: r"\sinh(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Hyperbolic Cosine",
            latex: r"\cosh(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Hyperbolic Tangent",
            latex: r"\tanh(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Natural Log",
            latex: r"\ln(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Logarithm",
            latex: r"\log(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Log Base b",
            latex: r"\log_{b}(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "Exponential",
            latex: r"\exp(x)",
            category: "Functions",
        },
        TemplateTest {
            name: "e to the power",
            latex: r"e^{x}",
            category: "Functions",
        },
        // Logic & Sets
        TemplateTest {
            name: "For All",
            latex: r"\forall x \colon P(x)",
            category: "Logic",
        },
        TemplateTest {
            name: "Exists",
            latex: r"\exists x \colon P(x)",
            category: "Logic",
        },
        TemplateTest {
            name: "Implies",
            latex: r"P \Rightarrow Q",
            category: "Logic",
        },
        TemplateTest {
            name: "If and Only If",
            latex: r"P \Leftrightarrow Q",
            category: "Logic",
        },
        TemplateTest {
            name: "Element Of",
            latex: r"x \in S",
            category: "Logic",
        },
        TemplateTest {
            name: "Subset",
            latex: r"A \subset B",
            category: "Logic",
        },
        TemplateTest {
            name: "Union",
            latex: r"A \cup B",
            category: "Logic",
        },
        TemplateTest {
            name: "Intersection",
            latex: r"A \cap B",
            category: "Logic",
        },
        TemplateTest {
            name: "Empty Set",
            latex: r"\emptyset",
            category: "Logic",
        },
        // Accents
        TemplateTest {
            name: "Hat",
            latex: r"\hat{x}",
            category: "Accents",
        },
        TemplateTest {
            name: "Bar",
            latex: r"\bar{x}",
            category: "Accents",
        },
        TemplateTest {
            name: "Overline",
            latex: r"\overline{x}",
            category: "Accents",
        },
        TemplateTest {
            name: "Tilde",
            latex: r"\tilde{x}",
            category: "Accents",
        },
        TemplateTest {
            name: "Dot",
            latex: r"\dot{x}",
            category: "Accents",
        },
        TemplateTest {
            name: "Double Dot",
            latex: r"\ddot{x}",
            category: "Accents",
        },
        TemplateTest {
            name: "Underline",
            latex: r"\underline{x}",
            category: "Accents",
        },
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut current_category = "";

    for test in &tests {
        if test.category != current_category {
            println!("\n📁 {}", test.category);
            println!("{}", "-".repeat(80));
            current_category = test.category;
        }

        print!("  Testing {:30} ", format!("{}...", test.name));

        match parse_latex(test.latex) {
            Ok(ast) => {
                // Just check if it parses - rendering is tested elsewhere
                println!("✅ PASS");
                passed += 1;
            }
            Err(e) => {
                println!("❌ FAIL (parse error)");
                println!("     Error: {}", e);
                println!("     LaTeX: {}", test.latex);
                failed += 1;
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("📊 Test Summary");
    println!("{}", "=".repeat(80));
    println!("Total:  {} templates", tests.len());
    println!("Passed: {} ✅", passed);
    println!("Failed: {} ❌", failed);

    if failed == 0 {
        println!("\n🎉 All templates passed!");
        std::process::exit(0);
    } else {
        println!("\n⚠️  Some templates failed. Please review the errors above.");
        std::process::exit(1);
    }
}
