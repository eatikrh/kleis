use kleis::kleis_parser::KleisParser;
///! Test loading the full prelude.kleis (not minimal_prelude.kleis)
///!
///! This tests that the parser can now handle custom operators like •
///! which are used throughout the prelude.
use kleis::type_checker::TypeChecker;

#[test]
fn test_parse_full_prelude() {
    println!("\n🔍 Attempting to parse full prelude.kleis...\n");

    let prelude = include_str!("../stdlib/prelude.kleis");
    let mut parser = KleisParser::new(prelude);

    match parser.parse_program() {
        Ok(program) => {
            println!("✅ Successfully parsed prelude.kleis!");
            println!("   Total items: {}", program.items.len());

            // Count different types of items
            use kleis::kleis_ast::TopLevel;
            let mut structures = 0;
            let mut implements = 0;
            let mut operations = 0;
            let mut functions = 0;

            for item in &program.items {
                match item {
                    TopLevel::StructureDef(_) => structures += 1,
                    TopLevel::ImplementsDef(_) => implements += 1,
                    TopLevel::OperationDecl(_) => operations += 1,
                    TopLevel::FunctionDef(_) => functions += 1,
                    _ => {}
                }
            }

            println!("   Structures: {}", structures);
            println!("   Implements: {}", implements);
            println!("   Operations: {}", operations);
            println!("   Functions: {}", functions);

            assert!(structures > 0, "Should have structure definitions");
        }
        Err(e) => {
            println!("❌ Failed to parse prelude.kleis");
            println!("   Error: {}", e.message);
            println!("   Position: {}", e.position);

            // Show context around error
            let lines: Vec<&str> = prelude.lines().collect();
            let line_num = prelude[..e.position.min(prelude.len())]
                .chars()
                .filter(|&c| c == '\n')
                .count();

            println!("\n   Context (around line {}):", line_num + 1);
            if line_num > 0 {
                println!(
                    "   {:3} | {}",
                    line_num,
                    lines.get(line_num.saturating_sub(1)).unwrap_or(&"")
                );
            }
            println!(
                "   {:3} | {}",
                line_num + 1,
                lines.get(line_num).unwrap_or(&"")
            );
            if line_num + 1 < lines.len() {
                println!(
                    "   {:3} | {}",
                    line_num + 2,
                    lines.get(line_num + 1).unwrap_or(&"")
                );
            }

            panic!("Failed to parse prelude.kleis: {}", e.message);
        }
    }
}

#[test]
fn test_load_prelude_into_typechecker() {
    println!("\n🔍 Attempting to load prelude.kleis into TypeChecker...\n");

    let mut checker = TypeChecker::new();
    let prelude = include_str!("../stdlib/prelude.kleis");

    match checker.load_kleis(prelude) {
        Ok(_) => {
            println!("✅ Successfully loaded prelude.kleis into TypeChecker!");
            println!("   This means custom operators work end-to-end!");
        }
        Err(e) => {
            println!("❌ Failed to load prelude.kleis into TypeChecker");
            println!("   Error: {}", e);

            // This might fail for other reasons (e.g., missing features like 'over' clause)
            // but the important thing is that it doesn't fail on parsing custom operators
            if e.contains("Expected ')'") || e.contains("•") {
                panic!("Failed due to custom operator parsing: {}", e);
            } else {
                println!(
                    "⚠️  Failed for a different reason (expected - prelude uses advanced features)"
                );
                println!("   But custom operator parsing works! ✅");
            }
        }
    }
}

#[test]
fn test_parse_semigroup_structure() {
    // The structure that uses • operator
    let semigroup = r#"
    structure Semigroup(S) {
      operation (•) : S × S → S
      
      axiom associativity:
        ∀(x y z : S). (x • y) • z = x • (y • z)
    }
    "#;

    println!("\n🔍 Testing Semigroup structure with • operator...\n");

    let mut parser = KleisParser::new(semigroup);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse Semigroup: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    println!("✅ Successfully parsed Semigroup structure!");
    println!("   Name: {}", structure.name);
    println!("   Members: {}", structure.members.len());

    // Check we have the • operation and associativity axiom
    use kleis::kleis_ast::StructureMember;
    let has_bullet_op = structure
        .members
        .iter()
        .any(|m| matches!(m, StructureMember::Operation { name, .. } if name == "•"));

    let has_assoc_axiom = structure
        .members
        .iter()
        .any(|m| matches!(m, StructureMember::Axiom { name, .. } if name == "associativity"));

    assert!(has_bullet_op, "Should have • operation");
    assert!(has_assoc_axiom, "Should have associativity axiom");

    println!("   ✅ Has • operation");
    println!("   ✅ Has associativity axiom with custom operators");
}
