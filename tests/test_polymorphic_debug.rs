// Debug test to understand polymorphic function type checking

use kleis::kleis_parser::parse_kleis_program;
use kleis::type_checker::TypeChecker;

#[test]
fn test_isssome_step_by_step() {
    let mut checker = TypeChecker::new();

    // Load data types
    let types_code = r#"
        data Bool = True | False
        data Option(T) = None | Some(value: T)
    "#;
    checker.load_data_types(types_code).unwrap();

    println!("\n=== Step 1: Data types loaded ===");

    // Now try to load isSome
    let func_code = r#"
        define isSome(opt) = match opt {
          None => False
          | Some(_) => True
        }
    "#;

    println!("\n=== Step 2: Parsing function ===");
    let program = parse_kleis_program(func_code).unwrap();
    println!("Parsed: {:?}", program);

    println!("\n=== Step 3: Type checking function ===");
    let result = checker.load_kleis(func_code);

    match result {
        Ok(()) => println!("✅ SUCCESS! Function loaded!"),
        Err(e) => println!("❌ ERROR: {}", e),
    }
}

#[test]
fn test_simple_bool_match() {
    let mut checker = TypeChecker::new();

    // Load Bool
    checker.load_data_types("data Bool = True | False").unwrap();

    // Simple function that doesn't use polymorphism
    let func_code = r#"
        define not(b) = match b {
          True => False
          | False => True
        }
    "#;

    let result = checker.load_kleis(func_code);
    assert!(
        result.is_ok(),
        "Simple non-polymorphic function should work: {:?}",
        result.err()
    );
}
