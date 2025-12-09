// Detailed debugging for head function

use kleis::kleis_parser::parse_kleis_program;
use kleis::type_checker::TypeChecker;

#[test]
fn test_head_with_detailed_output() {
    let mut checker = TypeChecker::new();

    // Load data types
    let types_code = r#"
        data Option(T) = None | Some(value: T)
        data List(T) = Nil | Cons(head: T, tail: List(T))
    "#;
    checker.load_data_types(types_code).unwrap();
    println!("✅ Data types loaded");

    // Parse head function
    let func_code = r#"
        define head(list) = match list {
          Nil => None
          | Cons(h, _) => Some(h)
        }
    "#;

    println!("\n=== Parsing head function ===");
    let program = parse_kleis_program(func_code).unwrap();

    for item in &program.items {
        if let kleis::kleis_ast::TopLevel::FunctionDef(func_def) = item {
            println!("Function name: {}", func_def.name);
            println!("Parameters: {:?}", func_def.params);
            println!("Body: {:?}", func_def.body);
        }
    }

    println!("\n=== Type checking head function ===");
    let result = checker.load_kleis(func_code);

    match &result {
        Ok(()) => println!("✅ SUCCESS!"),
        Err(e) => {
            println!("\n❌ ERROR:");
            println!("{}", e);
            println!("\nOccurs check suggests we created: α = Option(α)");
            println!("This shouldn't happen - need to track type parameters properly");
        }
    }
}
