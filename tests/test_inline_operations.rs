use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_implements_with_inline_negate() {
    let code = r#"
        implements Field(ℝ) {
            operation negate(x) = -x
        }
    "#;

    println!("\n🔍 Testing implements with inline negate...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    match result {
        Ok(impl_def) => {
            println!("✅ Parsed successfully!");
            println!("   Structure: {}", impl_def.structure_name);
            println!("   Members: {}", impl_def.members.len());
        }
        Err(e) => {
            println!("❌ Error: {}", e.message);
            println!("   Position: {}", e.position);
            panic!("Failed: {}", e.message);
        }
    }
}

#[test]
fn test_parse_multiple_inline_operations() {
    let code = r#"
        implements Field(ℝ) {
            operation negate(x) = -x
            operation inverse(x) = divide(1, x)
        }
    "#;

    println!("\n🔍 Testing multiple inline operations...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    match result {
        Ok(impl_def) => {
            println!("✅ Parsed successfully!");
            println!("   Members: {}", impl_def.members.len());
        }
        Err(e) => {
            println!("❌ Error: {}", e.message);
            println!("   Position: {}", e.position);

            if e.position < code.len() {
                let lines: Vec<&str> = code.lines().collect();
                let line_num = code[..e.position].chars().filter(|&c| c == '\n').count();
                println!(
                    "   Line {}: {}",
                    line_num + 1,
                    lines.get(line_num).unwrap_or(&"")
                );
            }

            panic!("Failed: {}", e.message);
        }
    }
}
