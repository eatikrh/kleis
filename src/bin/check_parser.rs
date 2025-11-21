use kleis::parser::parse_latex;
use std::time::{Duration, Instant};

fn timed_test(name: &str, latex: &str) {
    print!("Testing {}: ", name);
    let start = Instant::now();
    
    match parse_latex(latex) {
        Ok(expr) => {
            let elapsed = start.elapsed();
            println!("✅ OK in {:?} - {:?}", elapsed, expr);
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("❌ ERROR in {:?} - {}", elapsed, e);
        }
    }
}

fn main() {
    println!("🧪 Parser Test Runner with Timers\n");
    println!("{}", "=".repeat(70));
    
    // Test 1
    timed_test("Simple fraction", r"\frac{1}{2}");
    
    // Test 2
    timed_test("Square root", r"\sqrt{x}");
    
    // Test 3
    timed_test("Greek letter", r"\alpha");
    
    // Test 4
    timed_test("Trig function", r"\sin{x}");
    
    // Test 5
    timed_test("Subscript", r"x_{0}");
    
    // Test 6
    timed_test("Superscript", r"x^{2}");
    
    // Test 7
    timed_test("Addition", r"a + b");
    
    // Test 8
    timed_test("Multiplication", r"a * b");
    
    // Test 9
    timed_test("Complex expression", r"a + b * c");
    
    // Test 10 - THE DANGEROUS ONE
    println!("\n⚠️  Testing matrix (might hang)...");
    timed_test("Simple matrix", r"\begin{bmatrix}a&b\\c&d\end{bmatrix}");
    
    println!("\n{}", "=".repeat(70));
    println!("All tests completed!");
}

