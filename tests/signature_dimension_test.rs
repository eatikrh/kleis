///! Test that dimension constraints are enforced via signatures
///! This is the TRUE ADR-016 vision: constraints in signatures, not code!

use kleis::kleis_parser::parse_kleis_program;
use kleis::signature_interpreter::SignatureInterpreter;
use kleis::type_inference::Type;

#[test]
fn test_add_dimension_constraint_via_signature() {
    // Structure says: add : Matrix(m, n, T) → Matrix(m, n, T) → Matrix(m, n, T)
    // Both args must have SAME (m, n)!
    
    let code = r#"
        structure MatrixAddable(m: Nat, n: Nat, T) {
            operation add : Matrix(m, n, T)
        }
    "#;
    
    let program = parse_kleis_program(code).unwrap();
    let structure = program.structures()[0];
    
    // Test 1: Same dimensions - should work
    let mut interp1 = SignatureInterpreter::new();
    let args1 = vec![Type::Matrix(2, 3), Type::Matrix(2, 3)];
    
    // This should work because both are Matrix(2, 3)
    // Binds m=2, n=3 from first arg
    // Checks second arg matches Matrix(m=2, n=3) ✓
    let result1 = interp1.interpret_signature(structure, "add", &args1);
    
    match result1 {
        Ok(ty) => {
            assert_eq!(ty, Type::Matrix(2, 3));
            println!("✓ add(Matrix(2,3), Matrix(2,3)) → Matrix(2,3)");
        }
        Err(e) => panic!("Should have succeeded: {}", e),
    }
    
    // Test 2: Different dimensions - should fail
    let mut interp2 = SignatureInterpreter::new();
    let args2 = vec![Type::Matrix(2, 3), Type::Matrix(4, 5)];
    
    // This should FAIL because dimensions don't match
    // Binds m=2, n=3 from first arg
    // Tries to check second arg against Matrix(m=2, n=3)
    // Sees Matrix(4, 5) ≠ Matrix(2, 3) → ERROR!
    let result2 = interp2.interpret_signature(structure, "add", &args2);
    
    match result2 {
        Err(e) => {
            assert!(e.contains("Dimension constraint") || e.contains("mismatch"));
            println!("✓ add(Matrix(2,3), Matrix(4,5)) correctly rejected: {}", e);
        }
        Ok(_) => panic!("Should have failed on dimension mismatch!"),
    }
}

#[test]
fn test_multiply_dimension_constraint_via_signature() {
    // multiply : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
    // Inner dimension n must match!
    
    let code = r#"
        structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
            operation multiply : Matrix(m, p, T)
        }
    "#;
    
    let program = parse_kleis_program(code).unwrap();
    let structure = program.structures()[0];
    
    // Test: Compatible dimensions (2×3 × 3×4)
    let mut interp = SignatureInterpreter::new();
    let args = vec![Type::Matrix(2, 3), Type::Matrix(3, 4)];
    
    // Should work: m=2, n=3 from first, p=4 from second
    // Inner dimension n=3 matches!
    let result = interp.interpret_signature(structure, "multiply", &args);
    
    match result {
        Ok(ty) => {
            assert_eq!(ty, Type::Matrix(2, 4));
            println!("✓ multiply(Matrix(2,3), Matrix(3,4)) → Matrix(2,4)");
        }
        Err(e) => panic!("Should have succeeded: {}", e),
    }
}

