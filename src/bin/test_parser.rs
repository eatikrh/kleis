use kleis::parser::parse_latex;

fn main() {
    println!("🧪 Testing LaTeX Parser\n");
    println!("{}", "=".repeat(60));

    let tests = vec![
        ("Simple fraction", r"\frac{1}{2}"),
        ("Square root", r"\sqrt{x}"),
        ("Greek letter", r"\alpha"),
        ("Trig function", r"\sin{x}"),
        ("Subscript", r"x_{0}"),
        ("Superscript", r"x^{2}"),
        ("Addition", r"a + b"),
        ("Simple matrix 2x2", r"\begin{bmatrix}a&b\\c&d\end{bmatrix}"),
        (
            "Complex HJB matrix",
            r"\begin{bmatrix}\frac{\partial\,V}{\partial x} + \min_{{u}} \left\{ \frac{\partial\,V}{\partial x} \cdot F(x, u) + C(x, u) \right\} &0\\0&a\_{22}\end{bmatrix}",
        ),
    ];

    for (name, latex) in tests {
        println!("\n📝 Test: {}", name);
        println!("   Input: {}", latex);
        match parse_latex(latex) {
            Ok(expr) => println!("   ✅ Parsed: {:?}", expr),
            Err(e) => println!("   ❌ Error: {}", e),
        }
    }

    println!("\n{}", "=".repeat(60));
}
