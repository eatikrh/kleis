use kleis::parser::parse_latex;

fn main() {
    println!("🧪 Testing Top 5 Parser Additions\n");

    // Test 1: Anticommutator
    println!("1. Anticommutator \\{{A, B\\}}");
    match parse_latex(r"\{A, B\}") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }

    // Test 2: Unary minus
    println!("2. Unary minus -x");
    match parse_latex("-x") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }

    // Test 3: Implicit multiplication
    println!("3. Implicit multiplication 2x");
    match parse_latex("2x") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }

    println!("4. Implicit multiplication ab");
    match parse_latex("ab") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }

    // Test 4: Function calls
    println!("5. Function call f(x)");
    match parse_latex("f(x)") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }

    println!("6. Function call F(x, y)");
    match parse_latex("F(x, y)") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }

    // Test 5: Box operator
    println!("7. Box operator \\Box");
    match parse_latex(r"\Box") {
        Ok(expr) => println!("   ✅ {:?}\n", expr),
        Err(e) => println!("   ❌ {}\n", e),
    }
}
