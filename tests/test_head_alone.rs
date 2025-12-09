// Test loading head function alone

use kleis::type_checker::TypeChecker;

#[test]
fn test_load_head_alone() {
    let mut checker = TypeChecker::new();

    // Load data types
    let types_code = r#"
        data Option(T) = None | Some(value: T)
        data List(T) = Nil | Cons(head: T, tail: List(T))
    "#;
    checker.load_data_types(types_code).unwrap();

    // Try to load head
    let func_code = r#"
        define head(list) = match list {
          Nil => None
          | Cons(h, _) => Some(h)
        }
    "#;

    let result = checker.load_kleis(func_code);

    match &result {
        Ok(()) => println!("✅ SUCCESS! head loaded!"),
        Err(e) => {
            println!("\n❌ ERROR:");
            println!("{}", e);
            println!("\nThis is the occurs check problem we need to fix");
        }
    }

    assert!(result.is_ok(), "head should load: {:?}", result.err());
}
