use kleis::parser::parse_latex;

fn test(name: &str, latex: &str) -> bool {
    print!("{:40} ", name);
    match parse_latex(latex) {
        Ok(_) => {
            println!("✅");
            true
        }
        Err(e) => {
            println!("❌ {}", e);
            false
        }
    }
}

fn main() {
    println!("🧪 Testing Parser Against LaTeX Math Guide Examples\n");
    println!("{}", "=".repeat(80));
    
    let mut passed = 0;
    let mut total = 0;
    
    // Examples from calculus_basic.tex (AMS/IEEE style guides)
    total += 1; if test("Ordinary derivative", r"\frac{dy}{dx}") { passed += 1; }
    total += 1; if test("Partial derivative", r"\frac{\partial f}{\partial x}") { passed += 1; }
    total += 1; if test("Second derivative", r"\frac{d^2 y}{dx^2}") { passed += 1; }
    total += 1; if test("Indefinite integral", r"\int f(x) \, dx") { passed += 1; }
    total += 1; if test("Definite integral", r"\int_{a}^{b} f(x) \, dx") { passed += 1; }
    total += 1; if test("Double integral", r"\iint_{D} f(x,y) \, dx \, dy") { passed += 1; }
    total += 1; if test("Contour integral", r"\oint_{C} f(z) \, dz") { passed += 1; }
    total += 1; if test("Basic limit", r"\lim_{x \to a} f(x)") { passed += 1; }
    total += 1; if test("Finite sum", r"\sum_{i=1}^{n} i") { passed += 1; }
    total += 1; if test("Infinite sum", r"\sum_{n=1}^{\infty} \frac{1}{n^2}") { passed += 1; }
    total += 1; if test("Product", r"\prod_{i=1}^{n} i") { passed += 1; }
    
    // From physics_tensors.tex
    total += 1; if test("Einstein field eq", r"G_{\mu\nu} + \Lambda g_{\mu\nu}") { passed += 1; }
    total += 1; if test("Christoffel symbol", r"\Gamma^\rho_{\mu\nu}") { passed += 1; }
    total += 1; if test("Covariant derivative", r"\nabla_\mu V^\nu") { passed += 1; }
    total += 1; if test("Maxwell tensor", r"F_{\mu\nu} = \partial_\mu A_\nu - \partial_\nu A_\mu") { passed += 1; }
    
    // From quantum_mechanics.tex
    total += 1; if test("Ket vector", r"|\psi\rangle") { passed += 1; }
    total += 1; if test("Bra vector", r"\langle\phi|") { passed += 1; }
    total += 1; if test("Inner product", r"\langle\phi|\psi\rangle") { passed += 1; }
    total += 1; if test("Commutator", r"[\hat{L}_i, \hat{L}_j]") { passed += 1; }
    total += 1; if test("Hamiltonian", r"\hat{H} = \frac{\hat{p}^2}{2m} + V(\hat{x})") { passed += 1; }
    total += 1; if test("Pauli matrix", r"\begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}") { passed += 1; }
    
    println!("\n{}", "=".repeat(80));
    println!("📊 Results: {}/{} passed ({:.0}%)", passed, total, (passed as f32 / total as f32) * 100.0);
    println!();
    
    if passed == total {
        println!("🎉 PERFECT! Parser can handle all LaTeX Math Guide examples!");
    } else {
        println!("⚠️  Parser needs work on {} more patterns", total - passed);
    }
}

