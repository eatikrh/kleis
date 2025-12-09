///! Test List literal support

use kleis::kleis_parser::parse_kleis;
use kleis::type_inference::{Type, TypeInference};

#[test]
fn test_parse_list_literal() {
    let result = parse_kleis("[1, 2, 3]");
    assert!(result.is_ok(), "Failed to parse list literal: {:?}", result.err());
    
    println!("✓ Parsed [1, 2, 3]");
}

#[test]
fn test_parse_empty_list() {
    let result = parse_kleis("[]");
    assert!(result.is_ok(), "Failed to parse empty list: {:?}", result.err());
    
    println!("✓ Parsed []");
}

#[test]
fn test_list_type_inference() {
    let list_expr = parse_kleis("[1, 2, 3]").unwrap();
    let mut inference = TypeInference::new();
    
    let list_type = inference.infer(&list_expr, None).expect("Failed to infer list type");
    
    // Should be List(Scalar)
    if let Type::Data { constructor, args, .. } = &list_type {
        assert_eq!(constructor, "List");
        assert_eq!(args.len(), 1, "List should have 1 type parameter");
        println!("✓ Type of [1, 2, 3]: List(Scalar)");
    } else {
        panic!("Expected List type, got: {:?}", list_type);
    }
}

#[test]
fn test_matrix_with_list_new_format() {
    // NEW FORMAT: Matrix(2, 2, [a, b, c, d])
    let result = parse_kleis("Matrix(2, 2, [a, b, c, d])");
    assert!(result.is_ok(), "Failed to parse Matrix with List: {:?}", result.err());
    
    let expr = result.unwrap();
    println!("✓ Parsed Matrix(2, 2, [a, b, c, d])");
    
    // Verify it's an Operation with 3 args
    if let kleis::ast::Expression::Operation { name, args } = &expr {
        assert_eq!(name, "Matrix");
        assert_eq!(args.len(), 3, "New format should have 3 args");
        
        // Third arg should be a List
        assert!(matches!(args[2], kleis::ast::Expression::List(_)));
        println!("✓ Third argument is a List");
    } else {
        panic!("Expected Operation, got: {:?}", expr);
    }
}

#[test]
fn test_matrix_old_format_still_works() {
    // OLD FORMAT: Matrix(2, 2, a, b, c, d)
    let result = parse_kleis("Matrix(2, 2, a, b, c, d)");
    assert!(result.is_ok(), "Failed to parse old Matrix format: {:?}", result.err());
    
    let expr = result.unwrap();
    println!("✓ Parsed Matrix(2, 2, a, b, c, d) (old format)");
    
    // Verify it's an Operation with 6 args
    if let kleis::ast::Expression::Operation { name, args } = &expr {
        assert_eq!(name, "Matrix");
        assert_eq!(args.len(), 6, "Old format should have 6 args");
        println!("✓ Old format has 6 args (backwards compatible)");
    } else {
        panic!("Expected Operation, got: {:?}", expr);
    }
}

