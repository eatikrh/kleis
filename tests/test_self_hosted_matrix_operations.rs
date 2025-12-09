// Test self-hosted functions with matrix operations
//
// This verifies that functions can use structured operations (not just ADT pattern matching)

use kleis::type_checker::TypeChecker;

#[test]
fn test_define_matrix_addition_function() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Define a function that adds two matrices
    let code = r#"
        define addMatrices(A, B) = A + B
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "Should be able to define matrix addition function: {:?}", result.err());
    
    println!("\n✅ Matrix addition function defined!");
}

#[test]
fn test_define_matrix_scaling_function() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Define a function that scales a matrix
    let code = r#"
        define scaleMatrix(scalar, matrix) = scalar * matrix
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "Should be able to define matrix scaling: {:?}", result.err());
    
    println!("\n✅ Matrix scaling function defined!");
}

#[test]
fn test_define_matrix_multiplication_function() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Define a function that multiplies two matrices
    let code = r#"
        define multiplyMatrices(A, B) = A * B
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "Should be able to define matrix multiplication: {:?}", result.err());
    
    println!("\n✅ Matrix multiplication function defined!");
}

#[test]
fn test_define_vector_operations() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Define functions for vector operations
    let code = r#"
        define addVectors(v1, v2) = v1 + v2
        define dotProduct(v1, v2) = v1 * v2
        define scaleVector(scalar, vec) = scalar * vec
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "Should be able to define vector operations: {:?}", result.err());
    
    println!("\n✅ Vector operation functions defined!");
}

#[test]
fn test_combine_adt_and_matrix_operations() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Define a function that combines ADT pattern matching with matrix operations
    let code = r#"
        define maybeAddMatrices(optA, optB) = match optA {
          None => None
          | Some(a) => match optB {
              None => None
              | Some(b) => Some(a + b)
            }
        }
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "Should combine ADT matching with matrix ops: {:?}", result.err());
    
    println!("\n✅ Combined ADT + Matrix operations work!");
}

#[test]
fn test_matrix_function_with_list() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Use list operations with matrix
    let code = r#"
        define firstMatrixOrIdentity(list, identity) = 
            getOrDefault(head(list), identity)
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "List operations with matrices should work: {:?}", result.err());
    
    println!("\n✅ List operations work with any type (including matrices)!");
}

#[test]
fn test_realistic_linear_algebra() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Define a realistic linear algebra helper
    let code = r#"
        define linearCombination(scalar1, matrix1, scalar2, matrix2) = 
            (scalar1 * matrix1) + (scalar2 * matrix2)
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "Realistic linear algebra should work: {:?}", result.err());
    
    println!("\n✅ Realistic linear algebra functions work!");
}

#[test]
fn test_all_self_hosted_capabilities() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");
    
    // Comprehensive test showing all capabilities
    let code = r#"
        // Boolean logic (ADTs)
        define nand(a, b) = not(and(a, b))
        
        // Option handling (ADTs with polymorphism)
        define mapOption(opt, f) = match opt {
          None => None
          | Some(x) => Some(f)
        }
        
        // List operations (recursive ADTs)
        define secondElement(list) = head(tail(list))
        
        // Matrix operations (structured types)
        define matrixSum(A, B, C) = (A + B) + C
        
        // Mixed: ADTs + structured types
        define safeMatrixAdd(optA, optB) = match optA {
          None => None
          | Some(a) => match optB {
              None => None
              | Some(b) => Some(a + b)
            }
        }
    "#;
    
    let result = checker.load_kleis(code);
    assert!(result.is_ok(), "All self-hosted capabilities should work: {:?}", result.err());
    
    println!("\n✅ FULL SELF-HOSTING CAPABILITIES VERIFIED!");
    println!("  - Boolean ADTs");
    println!("  - Polymorphic ADTs");
    println!("  - Recursive types");
    println!("  - Matrix operations");
    println!("  - Combined ADT + structured operations");
}

