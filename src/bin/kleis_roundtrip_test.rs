//! Kleis Language Round-Trip Test
//!
//! Verifies that:
//! 1. Parse .kleis -> Program
//! 2. Pretty-print Program -> String
//! 3. Parse String -> Program2
//! 4. Programs are equivalent (same number of items, same structure)

use kleis::kleis_parser::parse_kleis_program;
use kleis::pretty_print::PrettyPrinter;

fn test_roundtrip(name: &str, source: &str) -> Result<(), String> {
    // Step 1: Parse original
    let program1 =
        parse_kleis_program(source).map_err(|e| format!("Initial parse failed: {}", e))?;

    // Step 2: Pretty-print
    let pp = PrettyPrinter::new();
    let printed = pp.format_program(&program1);

    // Step 3: Parse the pretty-printed output
    let program2 = parse_kleis_program(&printed)
        .map_err(|e| format!("Round-trip parse failed: {}\n\nPrinted:\n{}", e, printed))?;

    // Step 4: Compare structure counts
    let check = |what: &str, a: usize, b: usize| -> Result<(), String> {
        if a != b {
            Err(format!(
                "{}: count mismatch ({} vs {})\nOriginal:\n{}\n\nPrinted:\n{}",
                what, a, b, source, printed
            ))
        } else {
            Ok(())
        }
    };

    check(
        "structures",
        program1.structures().len(),
        program2.structures().len(),
    )?;
    check(
        "functions",
        program1.functions().len(),
        program2.functions().len(),
    )?;
    check(
        "data_types",
        program1.data_types().len(),
        program2.data_types().len(),
    )?;
    check(
        "implements",
        program1.implements().len(),
        program2.implements().len(),
    )?;
    check(
        "operations",
        program1.operations().len(),
        program2.operations().len(),
    )?;
    check(
        "type_aliases",
        program1.type_aliases().len(),
        program2.type_aliases().len(),
    )?;

    println!("✅ {} - round-trip OK", name);
    Ok(())
}

fn main() {
    println!("🔄 Kleis Language Round-Trip Test\n");
    println!("{}", "=".repeat(60));

    let mut passed = 0;
    let mut failed = 0;

    let tests: Vec<(&str, &str)> = vec![
        // Type aliases
        ("Simple type alias", r#"type Real = ℝ"#),
        ("Parametric type alias", r#"type Point = Vector(2, ℝ)"#),
        // Operation declarations
        ("Operation declaration", r#"operation add : ℝ → ℝ"#),
        // Function definitions
        ("Simple function", r#"define double(x) = x + x"#),
        (
            "Function with type annotation",
            r#"define square(x): ℝ = x * x"#,
        ),
        // Data types
        ("Simple data type", r#"data Bool = True | False"#),
        ("Parametric data type", r#"data Option(T) = None | Some(T)"#),
        // Structures
        (
            "Simple structure",
            r#"structure Monoid(M) {
    element identity : M
    operation op : M → M
}"#,
        ),
        // Implements
        (
            "Simple implements",
            r#"structure Numeric(N) {
    operation abs : N → N
}

implements Numeric(ℝ) {
    operation abs = builtin_abs
}"#,
        ),
        // Combined
        (
            "Type alias with function",
            r#"type Real = ℝ

define magnitude(x): Real = abs(x)"#,
        ),
        // Implements with inline operation
        (
            "Implements with inline",
            r#"structure Numeric(N) {
    operation negate : N → N
}

implements Numeric(ℝ) {
    operation negate(x) = -1 * x
}"#,
        ),
        // Implements with element
        (
            "Implements with element",
            r#"structure Ring(R) {
    element zero : R
    element one : R
}

implements Ring(ℝ) {
    element zero = 0
    element one = 1
}"#,
        ),
        // Structure with axiom
        (
            "Structure with axiom",
            r#"structure Group(G) {
    operation op : G → G
    axiom associative : ∀(a : G). ∀(b : G). ∀(c : G). op(op(a, b), c) = op(a, op(b, c))
}"#,
        ),
        // Simple function type
        ("Simple arrow type", r#"operation apply : ℝ → ℝ"#),
        // Parenthesized function type (Kleis v0.7 grammar: type ::= ... | "(" type ")")
        (
            "Parenthesized function type",
            r#"operation compose : (ℝ → ℝ) → (ℝ → ℝ) → (ℝ → ℝ)"#,
        ),
        // Multiple type aliases
        (
            "Multiple type aliases",
            r#"type R = ℝ
type C = ℂ
type N = ℕ"#,
        ),
        // Data type with multiple fields
        (
            "Data type complex variant",
            r#"data Result(T, E) = Ok(T) | Err(E)"#,
        ),
    ];

    for (name, source) in &tests {
        match test_roundtrip(name, source) {
            Ok(()) => passed += 1,
            Err(e) => {
                println!("❌ {} - FAILED", name);
                println!("   {}", e.lines().next().unwrap_or(&e));
                failed += 1;
            }
        }
    }

    println!();
    println!("{}", "=".repeat(60));
    println!("\n📊 Summary:");
    println!("   ✅ Passed: {}", passed);
    println!("   ❌ Failed: {}", failed);
    println!("   Total: {}", passed + failed);

    if failed == 0 {
        println!("\n🎉 Perfect round-trip fidelity!");
    } else {
        println!("\n⚠️  Some round-trips failed - investigate pretty printer or parser");
        std::process::exit(1);
    }
}
