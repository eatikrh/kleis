///! Test parsing of operator symbols in operation declarations
///!
///! Tests Phase 1 Task 2: Operator Symbols

use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_operator_plus() {
    let input = "operation (+) : R → R → R";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_operation_decl();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let op_decl = result.unwrap();
    assert_eq!(op_decl.name, "+");
}

#[test]
fn test_parse_operator_times() {
    let input = "operation (×) : R → R → R";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_operation_decl();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let op_decl = result.unwrap();
    assert_eq!(op_decl.name, "×");
}

#[test]
fn test_parse_operator_dot() {
    let input = "operation (•) : M → M → M";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_operation_decl();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let op_decl = result.unwrap();
    assert_eq!(op_decl.name, "•");
}

#[test]
fn test_parse_multiple_operators() {
    // Test parsing multiple operators separately
    let input1 = "operation (+) : R → R → R";
    let mut parser1 = KleisParser::new(input1);
    let result1 = parser1.parse_operation_decl();
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().name, "+");
    
    let input2 = "operation (×) : R → R → R";
    let mut parser2 = KleisParser::new(input2);
    let result2 = parser2.parse_operation_decl();
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().name, "×");
    
    let input3 = "operation (-) : R → R → R";
    let mut parser3 = KleisParser::new(input3);
    let result3 = parser3.parse_operation_decl();
    assert!(result3.is_ok());
    assert_eq!(result3.unwrap().name, "-");
}

#[test]
fn test_operator_in_structure() {
    let input = r#"
        structure Ring(R) {
            operation (+) : R → R → R
            operation (×) : R → R → R
            operation (-) : R → R → R
        }
    "#;
    
    let mut parser = KleisParser::new(input);
    let result = parser.parse_structure();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let structure = result.unwrap();
    assert_eq!(structure.name, "Ring");
    assert_eq!(structure.members.len(), 3);
    
    // Check operation names
    match &structure.members[0] {
        kleis::kleis_ast::StructureMember::Operation { name, .. } => {
            assert_eq!(name, "+");
        }
        _ => panic!("Expected Operation"),
    }
    
    match &structure.members[1] {
        kleis::kleis_ast::StructureMember::Operation { name, .. } => {
            assert_eq!(name, "×");
        }
        _ => panic!("Expected Operation"),
    }
    
    match &structure.members[2] {
        kleis::kleis_ast::StructureMember::Operation { name, .. } => {
            assert_eq!(name, "-");
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_operator_with_axiom() {
    // Complete Ring structure with operator symbols and axioms
    let input = r#"
        structure Ring(R) {
            operation (+) : R → R → R
            operation (×) : R → R → R
            axiom commutativity: ∀(x y : R). plus(x, y)
        }
    "#;
    
    let mut parser = KleisParser::new(input);
    let result = parser.parse_structure();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let structure = result.unwrap();
    assert_eq!(structure.name, "Ring");
    assert_eq!(structure.members.len(), 3);
    
    // Verify we have two operations and one axiom
    let mut op_count = 0;
    let mut axiom_count = 0;
    
    for member in &structure.members {
        match member {
            kleis::kleis_ast::StructureMember::Operation { .. } => op_count += 1,
            kleis::kleis_ast::StructureMember::Axiom { .. } => axiom_count += 1,
            _ => {}
        }
    }
    
    assert_eq!(op_count, 2, "Expected 2 operations");
    assert_eq!(axiom_count, 1, "Expected 1 axiom");
}

#[test]
fn test_extended_operators() {
    // Test additional operator symbols
    let input = r#"
        structure Algebra(A) {
            operation (⊗) : A → A → A
            operation (⊕) : A → A → A
            operation (∘) : A → A → A
        }
    "#;
    
    let mut parser = KleisParser::new(input);
    let result = parser.parse_structure();
    
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let structure = result.unwrap();
    assert_eq!(structure.members.len(), 3);
    
    match &structure.members[0] {
        kleis::kleis_ast::StructureMember::Operation { name, .. } => {
            assert_eq!(name, "⊗", "Tensor product symbol");
        }
        _ => panic!("Expected Operation"),
    }
    
    match &structure.members[1] {
        kleis::kleis_ast::StructureMember::Operation { name, .. } => {
            assert_eq!(name, "⊕", "Direct sum symbol");
        }
        _ => panic!("Expected Operation"),
    }
    
    match &structure.members[2] {
        kleis::kleis_ast::StructureMember::Operation { name, .. } => {
            assert_eq!(name, "∘", "Composition symbol");
        }
        _ => panic!("Expected Operation"),
    }
}

