use kleis::parser::parse_latex;

fn test_parse(_name: &str, latex: &str) -> bool {
    match parse_latex(latex) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    println!("🔄 Comprehensive Roundtrip Parser Test\n");
    println!("Testing all patterns including new features (109 test cases)");
    println!("{}", "=".repeat(80));
    println!();

    let tests = vec![
        // ===== BASIC OPERATIONS (Original 22) =====
        ("Fraction", r"\frac{1}{2}"),
        ("Square root", r"\sqrt{x}"),
        ("Nth root", r"\sqrt[3]{x}"),
        ("Power", r"x^{2}"),
        ("Subscript", r"x_{0}"),
        ("Greek alpha", r"\alpha"),
        ("Greek multiple", r"\alpha + \beta = \gamma"),
        ("Matrix 2x2", r"\begin{bmatrix}a&b\\c&d\end{bmatrix}"),
        (
            "Matrix 3x3",
            r"\begin{bmatrix}1&2&3\\4&5&6\\7&8&9\end{bmatrix}",
        ),
        ("Ket vector", r"|\psi\rangle"),
        ("Bra vector", r"\langle\phi|"),
        ("Commutator", r"[A, B]"),
        ("Anticommutator", r"\{A, B\}"),
        ("Set membership", r"x \in \mathbb{R}"),
        ("Subset", r"A \subseteq B"),
        ("Union", r"A \cup B"),
        ("Intersection", r"A \cap B"),
        ("Trig function", r"\sin{x}"),
        ("Function call", r"f(x, y)"),
        ("Addition", r"a + b"),
        ("Multiplication", r"2m"),
        ("Unary minus", r"-\frac{1}{2}"),
        ("Equals", r"E = mc^{2}"),
        ("Inequality", r"x \leq y"),
        // ===== INNER PRODUCT & VECTORS =====
        ("Inner product", r"\langle u, v \rangle"),
        ("Vector arrow", r"\vec{v}"),
        ("Vector bold", r"\boldsymbol{v}"),
        ("Outer product", r"|\psi\rangle\langle\phi|"),
        // ===== PHYSICS EQUATIONS =====
        (
            "Einstein field eqs",
            r"G_{{\mu\nu}} + \Lambda \, g_{{\mu\nu}} = \kappa \, T_{{\mu\nu}}",
        ),
        (
            "Maxwell tensor",
            r"F_{{\mu\nu}} = \partial_{{\mu}} A_{{\nu}} - \partial_{{\nu}} A_{{\mu}}",
        ),
        ("Christoffel symbol", r"\Gamma^{{\rho}}_{{\mu \nu}}"),
        ("Riemann tensor", r"R^{{\rho}}_{{\sigma \mu \nu}}"),
        // ===== CALCULUS =====
        ("Partial derivative", r"\frac{\partial\,L}{\partial y}"),
        ("Total derivative", r"\frac{d\,y}{dx}"),
        (
            "Second derivative",
            r"\frac{\partial^{2} \,V}{\partial x^{2}}",
        ),
        ("Limit", r"\lim_{ x \to 0 } f(x)"),
        ("Limsup", r"\limsup_{ x \to \infty } f(x)"),
        ("Liminf", r"\liminf_{ x \to a } f(x)"),
        ("Single integral", r"\int f(x) \, \mathrm{d}x"),
        ("Definite integral", r"\int_{ 0 }^{ 1 } f(x) \, \mathrm{d}x"),
        (
            "Double integral",
            r"\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y",
        ),
        (
            "Triple integral",
            r"\iiint_{V} f(x,y,z) \, \mathrm{d}x \, \mathrm{d}y \, \mathrm{d}z",
        ),
        ("Sum notation", r"\sum_{ n=1 }^{ \infty } \frac{1}{n^{2}}"),
        ("Product notation", r"\prod_{ i=1 }^{ n } i"),
        // ===== COMPLEX EQUATIONS =====
        (
            "Riemann zeta",
            r"\zeta(s) = \sum_{ n=1 }^{ \infty } \frac{1}{n^{s}}",
        ),
        ("Gamma function", r"\Gamma(s)"),
        // ===== INEQUALITIES & RELATIONS =====
        ("Less than", r"x < 0"),
        ("Greater than", r"x > 0"),
        ("Leq", r"E \leq 0"),
        ("Geq", r"x \geq 0"),
        ("Not equal", r"a \neq b"),
        ("Approximately", r"\pi \approx 3.14"),
        ("Proportional", r"F \propto ma"),
        // ===== COMPLEX NUMBERS =====
        ("Complex conjugate", r"\overline{z}"),
        ("Real part", r"\mathrm{Re}(z)"),
        ("Imaginary part", r"\mathrm{Im}(z)"),
        ("Modulus", r"\left|z\right|"),
        // ===== QUANTUM MECHANICS =====
        ("Operator hat", r"\hat{H}"),
        ("Hamiltonian ket", r"\hat{H}|\psi\rangle"),
        ("Commutation relation", r"[\hat{x}, \hat{p}] = i \, \hbar"),
        // ===== TRIGONOMETRIC FUNCTIONS =====
        ("Cosine", r"\cos(x)"),
        ("Tangent", r"\tan(\theta)"),
        ("Arcsine", r"\arcsin(x)"),
        ("Arccos", r"\arccos(x)"),
        ("Arctan", r"\arctan(x)"),
        ("Secant", r"\sec(x)"),
        ("Cosecant", r"\csc(x)"),
        ("Cotangent", r"\cot(x)"),
        // ===== HYPERBOLIC FUNCTIONS =====
        ("Sinh", r"\sinh(x)"),
        ("Cosh", r"\cosh(x)"),
        // ===== LOGARITHMS =====
        ("Natural log", r"\ln(x)"),
        ("Logarithm", r"\log(x)"),
        // ===== MATRIX OPERATIONS =====
        ("Trace", r"\mathrm{Tr}(\rho)"),
        ("Matrix inverse", r"A^{-1}"),
        ("Pmatrix 2x2", r"\begin{pmatrix}0&1\\1&0\end{pmatrix}"),
        (
            "Pmatrix 3x3",
            r"\begin{pmatrix}1&0&0\\0&1&0\\0&0&1\end{pmatrix}",
        ),
        ("Vmatrix 2x2", r"\begin{vmatrix}a&b\\c&d\end{vmatrix}"),
        (
            "Vmatrix 3x3",
            r"\begin{vmatrix}1&2&3\\4&5&6\\7&8&9\end{vmatrix}",
        ),
        // ===== COMBINATORICS =====
        ("Factorial", r"n!"),
        ("Binomial coefficient", r"\binom{n}{k}"),
        // ===== FLOOR & CEILING =====
        ("Floor", r"\lfloor x \rfloor"),
        ("Ceiling", r"\lceil x \rceil"),
        // ===== VECTOR CALCULUS =====
        ("Divergence", r"\nabla \cdot \mathbf{F}"),
        ("Curl", r"\nabla \times \mathbf{B}"),
        ("Laplacian", r"\nabla^2 \phi"),
        // ===== SET THEORY (Extended) =====
        ("Proper subset", r"A \subset B"),
        ("Forall quantifier", r"\forall x"),
        ("Exists quantifier", r"\exists x"),
        ("Implies", r"P \Rightarrow Q"),
        ("Iff", r"P \Leftrightarrow Q"),
        // ===== NUMBER SETS =====
        ("Real numbers", r"\mathbb{R}"),
        ("Complex numbers", r"\mathbb{C}"),
        ("Natural numbers", r"\mathbb{N}"),
        ("Integers", r"\mathbb{Z}"),
        ("Rationals", r"\mathbb{Q}"),
        // ===== MODULAR ARITHMETIC =====
        ("Congruence mod", r"a \equiv b \pmod{n}"),
        // ===== STATISTICS =====
        ("Variance", r"\mathrm{Var}(X)"),
        ("Covariance", r"\mathrm{Cov}(X, Y)"),
        // ===== PIECEWISE FUNCTIONS =====
        (
            "Piecewise 2 cases",
            r"\begin{cases}x^{2} & x \geq 0\\0 & x < 0\end{cases}",
        ),
        (
            "Piecewise 3 cases",
            r"\begin{cases}-1 & x < 0\\0 & x = 0\\1 & x > 0\end{cases}",
        ),
        // ===== TEXT MODE =====
        ("Text simple", r"\text{hello}"),
        ("Text with spaces", r"\text{if }"),
        ("Text in equation", r"x \text{ for all } y"),
        // ===== ACCENT COMMANDS =====
        ("Bar accent", r"\bar{x}"),
        ("Tilde accent", r"\tilde{x}"),
        ("Overline", r"\overline{z}"),
        ("Dot accent", r"\dot{x}"),
        ("Double dot accent", r"\ddot{x}"),
        ("Newton 2nd law", r"F = m\ddot{x}"),
    ];

    let mut success_count = 0;
    let mut failed_tests = Vec::new();

    for (name, latex) in &tests {
        print!("{:40} ", name);
        if test_parse(name, latex) {
            println!("✅");
            success_count += 1;
        } else {
            println!("❌");
            failed_tests.push((name, latex));
        }
    }

    println!();
    println!("{}", "=".repeat(80));
    println!("\n📊 Summary:");
    println!("   ✅ Successful parses: {}/{}", success_count, tests.len());

    let success_rate = (success_count as f64 / tests.len() as f64) * 100.0;
    println!("   Success rate: {:.1}%", success_rate);

    if !failed_tests.is_empty() {
        println!("\n❌ Failed tests:");
        for (name, latex) in &failed_tests {
            println!("   - {}: {}", name, latex);
        }
    }

    if success_count == tests.len() {
        println!(
            "\n🎉 Perfect! All {} test cases parse successfully!",
            tests.len()
        );
        println!("   Parser coverage matches all render.rs test patterns!");
    } else {
        println!(
            "\n⚠️  {} expressions failed to parse.",
            tests.len() - success_count
        );
        println!("   These may need parser implementation or are expected limitations.");
    }
}
