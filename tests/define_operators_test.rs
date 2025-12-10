///! Tests for define with operator names
///!
///! This enables defining operations with operator syntax:
///! ```kleis
///! define (-)(x, y) = x + negate(y)
///! define (×)(x, y) = times(x, y)
///! ```

use kleis::kleis_parser::KleisParser;

#[test]
fn test_define_with_operator_name() {
    // Test: define (-)(x, y) = x + negate(y)
    let code = "define (-)(x, y) = plus(x, negate(y))";
    
    let mut parser = KleisParser::new(code);
    let result = parser.parse_function_def();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let func_def = result.unwrap();
    assert_eq!(func_def.name, "-");
    assert_eq!(func_def.params.len(), 2);
    assert_eq!(func_def.params[0], "x");
    assert_eq!(func_def.params[1], "y");
    
    println!("✅ define (-)(x, y) works!");
}

#[test]
fn test_define_multiplication_operator() {
    // Test: define (×)(x, y) = times(x, y)
    let code = "define (×)(x, y) = times(x, y)";
    
    let mut parser = KleisParser::new(code);
    let result = parser.parse_function_def();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let func_def = result.unwrap();
    assert_eq!(func_def.name, "×");
    assert_eq!(func_def.params.len(), 2);
    
    println!("✅ define (×)(x, y) works!");
}

#[test]
fn test_define_various_operators() {
    // Test: Various operator symbols work
    let operators = vec![
        ("define (+)(x, y) = plus(x, y)", "+"),
        ("define (-)(x, y) = minus(x, y)", "-"),  // ASCII hyphen, not unicode minus
        ("define (×)(x, y) = times(x, y)", "×"),
        ("define (⊗)(x, y) = tensor_product(x, y)", "⊗"),
        ("define (∘)(f, g) = compose(f, g)", "∘"),
    ];
    
    for (code, expected_name) in operators {
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def();
        
        assert!(result.is_ok(), "Failed to parse {}: {:?}", code, result.err());
        
        let func_def = result.unwrap();
        assert_eq!(func_def.name, expected_name);
        
        println!("✅ {} works!", code);
    }
}

#[test]
fn test_regular_define_still_works() {
    // Test: Regular identifier names still work
    let code = "define subtract(x, y) = x + negate(y)";
    
    let mut parser = KleisParser::new(code);
    let result = parser.parse_function_def();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let func_def = result.unwrap();
    assert_eq!(func_def.name, "subtract");
    
    println!("✅ Regular define names still work");
}

#[test]
fn test_define_operator_in_program() {
    // Test: define with operator in full program context
    let code = r#"
        structure Ring(R) {
            operation plus : R → R → R
            operation negate : R → R
        }
        
        define (-)(x, y) = plus(x, negate(y))
    "#;
    
    let mut parser = KleisParser::new(code);
    let result = parser.parse_program();
    
    assert!(result.is_ok(), "Failed to parse program: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.items.len(), 2); // Structure + FunctionDef
    
    // Check the function definition
    match &program.items[1] {
        kleis::kleis_ast::TopLevel::FunctionDef(func_def) => {
            assert_eq!(func_def.name, "-");
            println!("✅ define with operator name in full program works!");
            println!("   This is exactly what prelude.kleis needs!");
        }
        _ => panic!("Expected FunctionDef"),
    }
}

#[test]
fn test_real_world_subtraction_definition() {
    // Test: Real example from prelude.kleis
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R) {
                operation plus : R → R → R
                operation negate : R → R
                operation zero : R
            }
        }
        
        define (-)(x, y) = plus(x, negate(y))
    "#;
    
    let mut parser = KleisParser::new(code);
    let result = parser.parse_program();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    println!("✅ Real-world Ring with defined (-) operator works!");
    println!("   This is exactly the prelude.kleis pattern!");
}

