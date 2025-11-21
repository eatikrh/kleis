use kleis::parser::parse_latex;

fn test_parse(name: &str, latex: &str) -> bool {
    match parse_latex(latex) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    println!("🔄 Parser Test: Can We Parse Common LaTeX?\n");
    println!("{}", "=".repeat(80));
    println!();
    
    let tests = vec![
        // Basic operations
        ("Fraction", r"\frac{1}{2}"),
        ("Square root", r"\sqrt{x}"),
        ("Power", r"x^{2}"),
        ("Subscript", r"x_{0}"),
        
        // Greek letters
        ("Greek alpha", r"\alpha"),
        ("Greek multiple", r"\alpha + \beta = \gamma"),
        
        // Matrices
        ("Matrix 2x2", r"\begin{bmatrix}a&b\\c&d\end{bmatrix}"),
        ("Matrix 3x3", r"\begin{bmatrix}1&2&3\\4&5&6\\7&8&9\end{bmatrix}"),
        
        // Quantum mechanics
        ("Ket vector", r"|\psi\rangle"),
        ("Bra vector", r"\langle\phi|"),
        ("Commutator", r"[A, B]"),
        ("Anticommutator", r"\{A, B\}"),
        
        // Set theory
        ("Set membership", r"x \in \mathbb{R}"),
        ("Subset", r"A \subseteq B"),
        ("Union", r"A \cup B"),
        
        // Functions
        ("Trig function", r"\sin{x}"),
        ("Function call", r"f(x, y)"),
        
        // Complex expressions
        ("Addition", r"a + b"),
        ("Multiplication", r"2m"),  // Implicit mult
        ("Unary minus", r"-\frac{1}{2}"),
        
        // Operators
        ("Equals", r"E = mc^{2}"),
        ("Inequality", r"x \leq y"),
    ];
    
    let mut success_count = 0;
    
    for (name, latex) in &tests {
        print!("{:30} ", name);
        if test_parse(name, latex) {
            println!("✅ PARSED");
            success_count += 1;
        } else {
            println!("❌ FAILED");
        }
    }
    
    println!();
    println!("{}", "=".repeat(80));
    println!("\n📊 Summary:");
    println!("   ✅ Successful parses: {}/{}", success_count, tests.len());
    
    let success_rate = (success_count as f64 / tests.len() as f64) * 100.0;
    println!("   Success rate: {:.1}%", success_rate);
    
    if success_count == tests.len() {
        println!("\n🎉 Perfect! All test cases parse successfully!");
    } else {
        println!("\n⚠️  Some expressions failed to parse.");
    }
}
