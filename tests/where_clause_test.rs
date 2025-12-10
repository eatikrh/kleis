///! Tests for where clause parsing in implements blocks
///!
///! Tests Phase 3.1: Parse generic constraints on implementations
///!
///! Syntax:
///! ```kleis
///! implements StructureName(TypeArgs) where Constraint(T) {
///!     ...
///! }
///! ```

use kleis::kleis_ast::{ImplMember, ImplementsDef, TypeExpr};
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_where_clause_single_constraint() {
    // Test: implements Matrix(...) where Semiring(T)
    let code = r#"
        implements MatrixMultipliable(m, n, p, T) where Semiring(T) {
            operation multiply = builtin_matrix_multiply
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let impl_def = result.unwrap();
    assert_eq!(impl_def.structure_name, "MatrixMultipliable");
    assert_eq!(impl_def.type_args.len(), 4);

    // Check where clause
    assert!(
        impl_def.where_clause.is_some(),
        "Where clause should be present"
    );

    let constraints = impl_def.where_clause.unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].structure_name, "Semiring");
    assert_eq!(constraints[0].type_args.len(), 1);

    // The type arg could be Var("T") or Named("T") depending on parsing
    match &constraints[0].type_args[0] {
        TypeExpr::Var(name) => assert_eq!(name, "T"),
        TypeExpr::Named(name) => assert_eq!(name, "T"),
        other => panic!("Expected type variable or named type T, got: {:?}", other),
    }

    println!("✅ Single where constraint parsed correctly");
}

#[test]
fn test_parse_where_clause_multiple_constraints() {
    // Test: implements Foo(T) where Semiring(T), Ord(T), Show(T)
    let code = r#"
        implements SortableRing(T) where Semiring(T), Ord(T) {
            operation sort = builtin_sort
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let impl_def = result.unwrap();

    // Check where clause
    assert!(
        impl_def.where_clause.is_some(),
        "Where clause should be present"
    );

    let constraints = impl_def.where_clause.unwrap();
    assert_eq!(constraints.len(), 2, "Should have 2 constraints");

    // First constraint
    assert_eq!(constraints[0].structure_name, "Semiring");
    assert_eq!(constraints[0].type_args.len(), 1);

    // Second constraint
    assert_eq!(constraints[1].structure_name, "Ord");
    assert_eq!(constraints[1].type_args.len(), 1);

    println!("✅ Multiple where constraints parsed correctly");
}

#[test]
fn test_parse_where_clause_with_complex_types() {
    // Test: where constraint with parametric types
    let code = r#"
        implements Functor(F) where Applicative(F) {
            operation fmap = builtin_fmap
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let impl_def = result.unwrap();
    assert!(impl_def.where_clause.is_some());

    let constraints = impl_def.where_clause.unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].structure_name, "Applicative");

    println!("✅ Where clause with parametric types parsed");
}

#[test]
fn test_parse_implements_without_where_clause() {
    // Test: Regular implements without where clause still works
    let code = r#"
        implements Numeric(ℝ) {
            operation abs = builtin_abs
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let impl_def = result.unwrap();
    assert_eq!(impl_def.structure_name, "Numeric");

    // Check where clause is None
    assert!(
        impl_def.where_clause.is_none(),
        "Where clause should be None when not present"
    );

    println!("✅ Implements without where clause still works");
}

#[test]
fn test_where_clause_with_multiple_type_args() {
    // Test: where Constraint(T, U)
    let code = r#"
        implements Bifunctor(F) where Functor(F, A), Functor(F, B) {
            operation bimap = builtin_bimap
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let impl_def = result.unwrap();
    assert!(impl_def.where_clause.is_some());

    let constraints = impl_def.where_clause.unwrap();
    assert_eq!(constraints.len(), 2);

    // First constraint should have 2 type args
    assert_eq!(constraints[0].type_args.len(), 2);
    // Second constraint should have 2 type args
    assert_eq!(constraints[1].type_args.len(), 2);

    println!("✅ Where clause with multiple type arguments parsed");
}

#[test]
fn test_real_world_matrix_multiply_with_where() {
    // Test: Real example from prelude.kleis
    let code = r#"
        implements MatrixMultipliable(m, n, p, T) where Semiring(T) {
            operation multiply = builtin_matrix_multiply
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_implements();

    assert!(result.is_ok(), "Failed to parse real-world example: {:?}", result.err());

    let impl_def = result.unwrap();
    assert_eq!(impl_def.structure_name, "MatrixMultipliable");
    assert_eq!(impl_def.type_args.len(), 4); // m, n, p, T

    // Verify where clause
    assert!(impl_def.where_clause.is_some());
    let constraints = impl_def.where_clause.unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].structure_name, "Semiring");

    // Verify member
    assert_eq!(impl_def.members.len(), 1);
    match &impl_def.members[0] {
        ImplMember::Operation { name, .. } => {
            assert_eq!(name, "multiply");
        }
        _ => panic!("Expected operation member"),
    }

    println!("✅ Real-world matrix multiply example parses correctly!");
    println!("   This is exactly what we need for prelude.kleis!");
}

#[test]
fn test_where_clause_preserves_whitespace_independence() {
    // Test: where clause parsing handles various whitespace
    let variations = vec![
        "implements Foo(T) where Bar(T) { operation x = y }",
        "implements Foo(T)where Bar(T){ operation x = y }",
        "implements Foo(T)  where  Bar(T)  { operation x = y }",
        "implements Foo(T)\n    where Bar(T)\n    { operation x = y }",
    ];

    for code in variations {
        let mut parser = KleisParser::new(code);
        let result = parser.parse_implements();

        assert!(
            result.is_ok(),
            "Failed to parse with whitespace variant: {:?}",
            result.err()
        );

        let impl_def = result.unwrap();
        assert!(impl_def.where_clause.is_some());
    }

    println!("✅ Where clause parsing is whitespace-independent");
}

#[test]
fn test_where_constraint_validation_success() {
    // Test: where clause referencing a known structure should succeed
    use kleis::type_checker::TypeChecker;

    let code = r#"
        structure Semiring(S) {
            operation plus : S → S → S
            operation times : S → S → S
        }
        
        structure MatrixMultipliable(m, n, p, T) {
            operation multiply : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
        }
        
        implements MatrixMultipliable(m, n, p, T) where Semiring(T) {
            operation multiply = builtin_matrix_multiply
        }
    "#;

    let mut checker = TypeChecker::new();
    let result = checker.load_kleis(code);

    assert!(
        result.is_ok(),
        "Should succeed with valid where constraint: {:?}",
        result.err()
    );

    println!("✅ Where constraint validation succeeds for known structure");
}

#[test]
fn test_where_constraint_validation_fails_unknown_structure() {
    // Test: where clause referencing unknown structure should fail
    use kleis::type_checker::TypeChecker;

    let code = r#"
        implements Foo(T) where UnknownStructure(T) {
            operation bar = builtin_bar
        }
    "#;

    let mut checker = TypeChecker::new();
    let result = checker.load_kleis(code);

    assert!(
        result.is_err(),
        "Should fail with unknown structure in where clause"
    );

    let error = result.unwrap_err();
    assert!(
        error.contains("Unknown structure") || error.contains("where"),
        "Error should mention unknown structure: {}",
        error
    );

    println!("✅ Where constraint validation fails for unknown structure");
}

#[test]
fn test_multiple_where_constraints_all_valid() {
    // Test: Multiple constraints, all referencing known structures
    use kleis::type_checker::TypeChecker;

    let code = r#"
        structure Semiring(S) {
            operation plus : S → S → S
        }
        
        structure Ord(O) {
            operation compare : O → O → O
        }
        
        structure SortableRing(T) {
            operation sort : List(T) → List(T)
        }
        
        implements SortableRing(T) where Semiring(T), Ord(T) {
            operation sort = builtin_sort
        }
    "#;

    let mut checker = TypeChecker::new();
    let result = checker.load_kleis(code);

    assert!(
        result.is_ok(),
        "Should succeed with all valid where constraints: {:?}",
        result.err()
    );

    println!("✅ Multiple where constraints validated successfully");
}

