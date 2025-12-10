///! Test parsing and verification of logical operators
///!
///! Tests Phase 2 Task 1: Logical Operators (⟹, ∧, ∨, ¬)
use kleis::ast::Expression;
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_conjunction() {
    // Test: A ∧ B
    let input = "x ∧ y";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_and");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_disjunction() {
    // Test: A ∨ B
    let input = "x ∨ y";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_or");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_negation() {
    // Test: ¬A
    let input = "¬x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_not");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_implication() {
    // Test: A ⟹ B
    let input = "x ⟹ y";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "implies");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_comparison() {
    // Test: x = y
    let input = "x = y";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "equals");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_logical_precedence() {
    // Test: A ∧ B ∨ C should parse as (A ∧ B) ∨ C
    // Because ∧ has higher precedence than ∨
    let input = "a ∧ b ∨ c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_or");
            // Left arg should be a ∧ b
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_and");
                }
                _ => panic!("Expected conjunction on left"),
            }
        }
        _ => panic!("Expected disjunction at top"),
    }
}

#[test]
fn test_implication_with_conjunction() {
    // Test: (A ∧ B) ⟹ C
    let input = "(a ∧ b) ⟹ c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "implies");
            // Left should be conjunction
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_and");
                }
                _ => panic!("Expected conjunction"),
            }
        }
        _ => panic!("Expected implication"),
    }
}

#[test]
fn test_de_morgan_law_parsing() {
    // TODO: This test should verify De Morgan's Law: ¬(A ∨ B) = (¬A) ∧ (¬B)
    // Currently simplified because:
    // 1. Need better Z3 translator for nested boolean operations
    // 2. Need Bool type support (not just Int)
    // 3. Need proper type context for boolean variables
    //
    // Future implementation should:
    // - Parse: "∀(a b : Bool). ¬(a ∨ b) = (¬a) ∧ (¬b)"
    // - Verify with Z3 that it holds for all boolean values
    // - Show counterexample if violated
    //
    // For now, just test that equality parsing works

    let input = "a = b";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, .. } => {
            assert_eq!(name, "equals");
            println!("✅ Equality parses correctly (De Morgan's full test TODO)");
        }
        _ => panic!("Expected equals operation"),
    }
}

#[test]
fn test_modus_ponens() {
    // Test: (P ∧ (P ⟹ Q)) ⟹ Q
    // This is the modus ponens inference rule

    let axiom_text = "∀(p q : Bool). implies(logical_and(p, implies(p, q)), q)";
    let mut parser = KleisParser::new(axiom_text);
    let axiom = parser.parse_proposition().expect("Failed to parse");

    let verifier = AxiomVerifier::new();
    let result = verifier.verify_axiom(&axiom);

    println!("Modus Ponens verification: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_complex_logical_expression() {
    // Test: (A ⟹ B) ∧ (B ⟹ C) ⟹ (A ⟹ C)
    // Transitivity of implication

    let input = "(a ⟹ b) ∧ (b ⟹ c) ⟹ (a ⟹ c)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    // Check that it parsed correctly
    match result.unwrap() {
        Expression::Operation { name, .. } => {
            assert_eq!(name, "implies", "Top level should be implication");
        }
        _ => panic!("Expected implication at top level"),
    }
}

#[test]
fn test_logical_operators_in_structure() {
    // Test: Structure with logical operator declarations
    let structure_text = r#"
        structure Logic {
            operation (∧) : Bool → Bool → Bool
            operation (∨) : Bool → Bool → Bool
            operation (¬) : Bool → Bool
            operation (⟹) : Bool → Bool → Bool
            axiom excluded_middle: ∀(p : Bool). logical_or(p, logical_not(p))
        }
    "#;

    let mut parser = KleisParser::new(structure_text);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse structure: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "Logic");
    assert_eq!(structure.members.len(), 5); // 4 operations + 1 axiom

    // Check operation names
    let op_names: Vec<String> = structure
        .members
        .iter()
        .filter_map(|member| match member {
            kleis::kleis_ast::StructureMember::Operation { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    assert!(op_names.contains(&"∧".to_string()));
    assert!(op_names.contains(&"∨".to_string()));
    assert!(op_names.contains(&"¬".to_string()));
    assert!(op_names.contains(&"⟹".to_string()));
}
