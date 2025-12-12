#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Tests for user-defined parametric types in operation signatures
///!
///! This tests the fix for SignatureInterpreter to support:
///! - Simple user types (0-arity): Currency, Bool
///! - Parametric types (1-arity): Option(T), Vector(n)
///! - Multi-parameter types (2-arity): Matrix(m,n), Result(T,E)
///! - High-arity types (3+): Tensor3D(i,j,k), Tensor4D(i,j,k,l)
///! - Mixed parameter kinds: Nat, Type, String
///!
///! Before the fix: All user types defaulted to Scalar (wrong!)
///! After the fix: Types are looked up in DataTypeRegistry (correct!)
use kleis::data_registry::DataTypeRegistry;
use kleis::kleis_ast::{DataDef, DataVariant, TypeParam};
use kleis::kleis_parser::parse_kleis_program;
use kleis::signature_interpreter::SignatureInterpreter;
use kleis::structure_registry::StructureRegistry;
use kleis::type_inference::Type;

/// Helper: Create a simple data type (0-arity)
fn make_simple_type(name: &str, variants: Vec<&str>) -> DataDef {
    DataDef {
        name: name.to_string(),
        type_params: vec![],
        variants: variants
            .iter()
            .map(|v| DataVariant {
                name: v.to_string(),
                fields: vec![],
            })
            .collect(),
    }
}

/// Helper: Create a parametric type with Nat parameters
fn make_nat_parametric_type(name: &str, param_names: Vec<&str>) -> DataDef {
    DataDef {
        name: name.to_string(),
        type_params: param_names
            .iter()
            .map(|p| TypeParam {
                name: p.to_string(),
                kind: Some("Nat".to_string()),
            })
            .collect(),
        variants: vec![DataVariant {
            name: name.to_string(),
            fields: vec![],
        }],
    }
}

/// Helper: Create a parametric type with Type parameters
fn make_type_parametric_type(name: &str, param_names: Vec<&str>) -> DataDef {
    DataDef {
        name: name.to_string(),
        type_params: param_names
            .iter()
            .map(|p| TypeParam {
                name: p.to_string(),
                kind: Some("Type".to_string()),
            })
            .collect(),
        variants: vec![DataVariant {
            name: name.to_string(),
            fields: vec![],
        }],
    }
}

#[test]
fn test_simple_user_type_in_signature() {
    // Test: Simple (0-arity) user-defined type
    // data Currency = USD | EUR | GBP
    // structure Tradeable(C) {
    //     operation rate : C → ℝ
    // }

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_simple_type("Currency", vec!["USD", "EUR", "GBP"]))
        .unwrap();

    let code = r#"
        structure Tradeable(C) {
            operation rate : C → ℝ
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Manually bind C to Currency (would normally happen during unification)
    // For now we'll just test that Currency is recognized

    // Create a Currency type
    let currency_type = Type::Data {
        type_name: "Currency".to_string(),
        constructor: "USD".to_string(),
        args: vec![],
    };

    let result = interp.interpret_signature(structure, "rate", &[currency_type]);

    // Should return ℝ (Scalar)
    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());
}

#[test]
fn test_parametric_1_arity_nat() {
    // Test: 1-arity parametric type with Nat parameter
    // data Vector(n: Nat) = Vector(...)
    // structure VectorOps(n: Nat) {
    //     operation magnitude : Vector(n) → ℝ
    // }

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_nat_parametric_type("Vector", vec!["n"]))
        .unwrap();

    let code = r#"
        structure VectorOps(n: Nat) {
            operation magnitude : Vector(n) → ℝ
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Bind n to 3
    interp.bindings.insert("n".to_string(), 3);

    // Create Vector(3) type
    let vector_type = Type::Data {
        type_name: "Vector".to_string(),
        constructor: "Vector".to_string(),
        args: vec![Type::NatValue(3)],
    };

    let result = interp.interpret_signature(structure, "magnitude", &[vector_type]);

    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());
}

#[test]
fn test_parametric_2_arity_nat() {
    // Test: 2-arity parametric type with Nat parameters
    // data Matrix(m: Nat, n: Nat) = Matrix(...)
    // structure MatrixOps(m: Nat, n: Nat) {
    //     operation trace : Matrix(m, m) → ℝ
    // }

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_nat_parametric_type("Matrix", vec!["m", "n"]))
        .unwrap();

    let code = r#"
        structure SquareMatrix(n: Nat) {
            operation trace : Matrix(n, n) → ℝ
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Bind n to 4
    interp.bindings.insert("n".to_string(), 4);

    // Create Matrix(4, 4) type
    let matrix_type = Type::Data {
        type_name: "Matrix".to_string(),
        constructor: "Matrix".to_string(),
        args: vec![Type::NatValue(4), Type::NatValue(4)],
    };

    let result = interp.interpret_signature(structure, "trace", &[matrix_type]);

    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());
}

#[test]
fn test_parametric_3_arity_nat() {
    // Test: 3-arity parametric type with Nat parameters
    // This is the KEY test from NEXT_SESSION_TASK.md!
    // data Tensor3D(i: Nat, j: Nat, k: Nat) = Tensor3D(...)
    // structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
    //     operation sum : Tensor3D(i, j, k) → ℝ
    // }

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_nat_parametric_type("Tensor3D", vec!["i", "j", "k"]))
        .unwrap();

    let code = r#"
        structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
            operation sum : Tensor3D(i, j, k) → ℝ
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Bind i=10, j=20, k=30
    interp.bindings.insert("i".to_string(), 10);
    interp.bindings.insert("j".to_string(), 20);
    interp.bindings.insert("k".to_string(), 30);

    // Create Tensor3D(10, 20, 30) type
    let tensor_type = Type::Data {
        type_name: "Tensor3D".to_string(),
        constructor: "Tensor3D".to_string(),
        args: vec![Type::NatValue(10), Type::NatValue(20), Type::NatValue(30)],
    };

    let result = interp.interpret_signature(structure, "sum", &[tensor_type]);

    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());
}

#[test]
fn test_parametric_4_arity_nat() {
    // Test: 4-arity (and beyond works the same way!)
    // data Tensor4D(i: Nat, j: Nat, k: Nat, l: Nat) = Tensor4D(...)

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_nat_parametric_type(
            "Tensor4D",
            vec!["i", "j", "k", "l"],
        ))
        .unwrap();

    let code = r#"
        structure Tensor4DOps(i: Nat, j: Nat, k: Nat, l: Nat) {
            operation flatten : Tensor4D(i, j, k, l) → ℝ
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Bind i=2, j=3, k=4, l=5
    interp.bindings.insert("i".to_string(), 2);
    interp.bindings.insert("j".to_string(), 3);
    interp.bindings.insert("k".to_string(), 4);
    interp.bindings.insert("l".to_string(), 5);

    // Create Tensor4D(2, 3, 4, 5) type
    let tensor_type = Type::Data {
        type_name: "Tensor4D".to_string(),
        constructor: "Tensor4D".to_string(),
        args: vec![
            Type::NatValue(2),
            Type::NatValue(3),
            Type::NatValue(4),
            Type::NatValue(5),
        ],
    };

    let result = interp.interpret_signature(structure, "flatten", &[tensor_type]);

    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());
}

#[test]
fn test_arity_validation() {
    // Test: Arity mismatch is caught when interpreting type expression
    // Define Tensor3D with 3 params, but try to interpret it with 2
    // This directly tests interpret_type_expr arity validation

    use kleis::kleis_ast::TypeExpr;

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_nat_parametric_type("Tensor3D", vec!["i", "j", "k"]))
        .unwrap();

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Bind i=10, j=20
    interp.bindings.insert("i".to_string(), 10);
    interp.bindings.insert("j".to_string(), 20);

    // Try to interpret Tensor3D(i, j) - should fail because it needs 3 params!
    let bad_type_expr = TypeExpr::Parametric(
        "Tensor3D".to_string(),
        vec![
            TypeExpr::Named("i".to_string()),
            TypeExpr::Named("j".to_string()),
        ],
    );

    let result = interp.interpret_type_expr(&bad_type_expr);

    // Should error about arity mismatch
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("expects 3 parameters") && err.contains("got 2"),
        "Error was: {}",
        err
    );
}

#[test]
fn test_parametric_with_type_params() {
    // Test: Type parameters (not just Nat)
    // data Option(T: Type) = None | Some(T)
    // structure Optionable(T) {
    //     operation unwrap : Option(T) → T
    // }

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_type_parametric_type("Option", vec!["T"]))
        .unwrap();

    let code = r#"
        structure Optionable(T) {
            operation unwrap : Option(T) → T
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Create Option(ℝ) type
    let option_type = Type::Data {
        type_name: "Option".to_string(),
        constructor: "Option".to_string(),
        args: vec![Type::scalar()],
    };

    let result = interp.interpret_signature(structure, "unwrap", &[option_type]);

    assert!(result.is_ok());
    // Result should be T, which is Scalar
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());
}

#[test]
#[ignore = "Backward compatibility test for old Matrix format - being phased out"]
fn test_backward_compatibility_matrix() {
    // Test: Hardcoded Matrix still works (backward compatibility)
    // Even without registry entry, Matrix(m,n) should work
    // NOTE: Old format uses Type::matrix() helper which is deprecated

    let registry = DataTypeRegistry::new(); // Empty registry!

    let code = r#"
        structure Matrix(m: Nat, n: Nat) {
            operation transpose : Matrix(n, m)
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Bind m=2, n=3
    interp.bindings.insert("m".to_string(), 2);
    interp.bindings.insert("n".to_string(), 3);

    let matrix_type = Type::matrix(2, 3, Type::scalar());

    let result = interp.interpret_signature(structure, "transpose", &[matrix_type]);

    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::matrix(3, 2, Type::scalar())); // Transposed!
}

#[test]
fn test_zero_arity_from_registry() {
    // Test: 0-arity types work
    // data Unit = Unit

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_simple_type("Unit", vec!["Unit"]))
        .unwrap();

    let code = r#"
        structure Trivial {
            operation unit : Unit
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // No arguments needed for 0-arity
    let result = interp.interpret_signature(structure, "unit", &[]);

    assert!(result.is_ok());
    let result_type = result.unwrap();

    // Should be Unit type
    match result_type {
        Type::Data { type_name, .. } => {
            assert_eq!(type_name, "Unit");
        }
        _ => panic!("Expected Type::Data for Unit"),
    }
}

// =============================================================================
// String Parameter Tests
// =============================================================================

/// Helper: Create a type with String and Type parameters
fn make_string_type_parametric(
    name: &str,
    string_params: Vec<&str>,
    type_params: Vec<&str>,
) -> DataDef {
    let mut params = vec![];

    // Add string parameters
    for p in string_params {
        params.push(TypeParam {
            name: p.to_string(),
            kind: Some("String".to_string()),
        });
    }

    // Add type parameters
    for p in type_params {
        params.push(TypeParam {
            name: p.to_string(),
            kind: Some("Type".to_string()),
        });
    }

    DataDef {
        name: name.to_string(),
        type_params: params,
        variants: vec![DataVariant {
            name: name.to_string(),
            fields: vec![],
        }],
    }
}

#[test]
fn test_string_parameter_binding() {
    // Test: String parameter binding works
    // data Metric(unit: String, T) = Metric(...)
    // structure MetricOps(unit: String, T) {
    //     operation convert : Metric(unit, T) → ℝ
    // }

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_string_type_parametric(
            "Metric",
            vec!["unit"],
            vec!["T"],
        ))
        .unwrap();

    let code = r#"
        structure MetricOps(unit: String, T) {
            operation magnitude : Metric(unit, T) → ℝ
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Create Metric("m/s", ℝ) type
    let metric_type = Type::Data {
        type_name: "Metric".to_string(),
        constructor: "Metric".to_string(),
        args: vec![Type::StringValue("m/s".to_string()), Type::scalar()],
    };

    let result = interp.interpret_signature(structure, "magnitude", &[metric_type]);

    assert!(result.is_ok());
    let result_type = result.unwrap();
    assert_eq!(result_type, Type::scalar());

    // Verify string binding was recorded
    assert_eq!(interp.string_bindings.get("unit"), Some(&"m/s".to_string()));

    println!("✓ String parameter binding works: Metric(\"m/s\", ℝ)");
}

#[test]
fn test_string_parameter_mismatch_caught() {
    // Test: String parameter mismatches are caught
    // Metric("m/s", ℝ) + Metric("N", ℝ) should fail (unit mismatch!)

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_string_type_parametric(
            "Metric",
            vec!["unit"],
            vec!["T"],
        ))
        .unwrap();

    let code = r#"
        structure MetricAddable(unit: String, T) {
            operation add : Metric(unit, T) → Metric(unit, T) → Metric(unit, T)
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Try to add metrics with different units - should fail!
    let metric1 = Type::Data {
        type_name: "Metric".to_string(),
        constructor: "Metric".to_string(),
        args: vec![Type::StringValue("m/s".to_string()), Type::scalar()],
    };

    let metric2 = Type::Data {
        type_name: "Metric".to_string(),
        constructor: "Metric".to_string(),
        args: vec![Type::StringValue("N".to_string()), Type::scalar()],
    };

    let result = interp.interpret_signature(structure, "add", &[metric1, metric2]);

    // Should error about unit mismatch
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("String parameter") && err.contains("mismatch"),
        "Expected string mismatch error, got: {}",
        err
    );
    assert!(err.contains("m/s") && err.contains("N"));

    println!("✓ String parameter mismatch correctly caught");
}

#[test]
fn test_string_parameter_consistency() {
    // Test: Same string parameter used multiple times must be consistent
    // operation process : Metric(unit, T) → Metric(unit, T) → Metric(unit, T)
    // All three Metric types must have the same unit!

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_string_type_parametric(
            "Metric",
            vec!["unit"],
            vec!["T"],
        ))
        .unwrap();

    let code = r#"
        structure MetricOps(unit: String, T) {
            operation average : Metric(unit, T) → Metric(unit, T) → Metric(unit, T)
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Both arguments have same unit
    let metric1 = Type::Data {
        type_name: "Metric".to_string(),
        constructor: "Metric".to_string(),
        args: vec![Type::StringValue("kg".to_string()), Type::scalar()],
    };

    let metric2 = Type::Data {
        type_name: "Metric".to_string(),
        constructor: "Metric".to_string(),
        args: vec![Type::StringValue("kg".to_string()), Type::scalar()],
    };

    let result = interp.interpret_signature(structure, "average", &[metric1, metric2]);

    assert!(result.is_ok());
    let result_type = result.unwrap();

    // Result should also be Metric with unit="kg"
    match result_type {
        Type::Data {
            type_name, args, ..
        } => {
            assert_eq!(type_name, "Metric");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Type::StringValue("kg".to_string()));
        }
        _ => panic!("Expected Metric type, got {:?}", result_type),
    }

    println!("✓ String parameter consistency enforced");
}

#[test]
fn test_mixed_string_and_nat_parameters() {
    // Test: Types can have both String and Nat parameters
    // data LabeledMatrix(label: String, m: Nat, n: Nat, T) = LabeledMatrix(...)

    let mut registry = DataTypeRegistry::new();

    // Create type with String param first, then Nat params, then Type param
    let params = vec![
        TypeParam {
            name: "label".to_string(),
            kind: Some("String".to_string()),
        },
        TypeParam {
            name: "m".to_string(),
            kind: Some("Nat".to_string()),
        },
        TypeParam {
            name: "n".to_string(),
            kind: Some("Nat".to_string()),
        },
        TypeParam {
            name: "T".to_string(),
            kind: Some("Type".to_string()),
        },
    ];

    let data_def = DataDef {
        name: "LabeledMatrix".to_string(),
        type_params: params,
        variants: vec![DataVariant {
            name: "LabeledMatrix".to_string(),
            fields: vec![],
        }],
    };

    registry.register(data_def).unwrap();

    let code = r#"
        structure LabeledMatrixOps(label: String, m: Nat, n: Nat, T) {
            operation transpose : LabeledMatrix(label, m, n, T) → LabeledMatrix(label, n, m, T)
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    let mut interp = SignatureInterpreter::new(registry, StructureRegistry::new());

    // Create LabeledMatrix("velocity", 2, 3, ℝ)
    let labeled_matrix = Type::Data {
        type_name: "LabeledMatrix".to_string(),
        constructor: "LabeledMatrix".to_string(),
        args: vec![
            Type::StringValue("velocity".to_string()),
            Type::NatValue(2),
            Type::NatValue(3),
            Type::scalar(),
        ],
    };

    let result = interp.interpret_signature(structure, "transpose", &[labeled_matrix]);

    assert!(result.is_ok());
    let result_type = result.unwrap();

    // Result should be LabeledMatrix("velocity", 3, 2, ℝ) - dimensions flipped!
    match result_type {
        Type::Data {
            type_name, args, ..
        } => {
            assert_eq!(type_name, "LabeledMatrix");
            assert_eq!(args[0], Type::StringValue("velocity".to_string()));
            assert_eq!(args[1], Type::NatValue(3)); // Flipped!
            assert_eq!(args[2], Type::NatValue(2)); // Flipped!
            assert_eq!(args[3], Type::scalar());
        }
        _ => panic!("Expected LabeledMatrix type, got {:?}", result_type),
    }

    // Verify all bindings were recorded
    assert_eq!(
        interp.string_bindings.get("label"),
        Some(&"velocity".to_string())
    );
    assert_eq!(interp.bindings.get("m"), Some(&2));
    assert_eq!(interp.bindings.get("n"), Some(&3));

    println!("✓ Mixed String, Nat, and Type parameters work together");
}

#[test]
fn test_unit_safe_physics_calculations() {
    // Test: Unit-safe physics - THE KILLER FEATURE!
    // This prevents: adding velocity (m/s) to force (N) - physically nonsensical!

    let mut registry = DataTypeRegistry::new();
    registry
        .register(make_string_type_parametric(
            "Quantity",
            vec!["unit"],
            vec!["T"],
        ))
        .unwrap();

    let code = r#"
        structure QuantityAddable(unit: String, T) {
            operation add : Quantity(unit, T) → Quantity(unit, T) → Quantity(unit, T)
        }
    "#;

    let program = parse_kleis_program(code).unwrap();
    let structure = &program.structures()[0];

    // Case 1: Adding same units - SHOULD WORK
    let mut interp1 = SignatureInterpreter::new(registry.clone(), StructureRegistry::new());

    let velocity1 = Type::Data {
        type_name: "Quantity".to_string(),
        constructor: "Quantity".to_string(),
        args: vec![Type::StringValue("m/s".to_string()), Type::scalar()],
    };

    let velocity2 = Type::Data {
        type_name: "Quantity".to_string(),
        constructor: "Quantity".to_string(),
        args: vec![Type::StringValue("m/s".to_string()), Type::scalar()],
    };

    let result1 = interp1.interpret_signature(structure, "add", &[velocity1, velocity2]);
    assert!(result1.is_ok(), "Adding velocities should work!");
    println!("✓ velocity (m/s) + velocity (m/s) → velocity (m/s) ✅");

    // Case 2: Adding different units - SHOULD FAIL
    let mut interp2 = SignatureInterpreter::new(registry.clone(), StructureRegistry::new());

    let velocity = Type::Data {
        type_name: "Quantity".to_string(),
        constructor: "Quantity".to_string(),
        args: vec![Type::StringValue("m/s".to_string()), Type::scalar()],
    };

    let force = Type::Data {
        type_name: "Quantity".to_string(),
        constructor: "Quantity".to_string(),
        args: vec![Type::StringValue("N".to_string()), Type::scalar()],
    };

    let result2 = interp2.interpret_signature(structure, "add", &[velocity, force]);
    assert!(result2.is_err(), "Adding velocity to force should FAIL!");

    let err = result2.unwrap_err();
    assert!(err.contains("m/s") && err.contains("N"));
    println!("✓ velocity (m/s) + force (N) → ERROR ❌ (unit mismatch caught!)");

    // Case 3: Multiple physics units
    let mut interp3 = SignatureInterpreter::new(registry.clone(), StructureRegistry::new());

    let mass1 = Type::Data {
        type_name: "Quantity".to_string(),
        constructor: "Quantity".to_string(),
        args: vec![Type::StringValue("kg".to_string()), Type::scalar()],
    };

    let mass2 = Type::Data {
        type_name: "Quantity".to_string(),
        constructor: "Quantity".to_string(),
        args: vec![Type::StringValue("kg".to_string()), Type::scalar()],
    };

    let result3 = interp3.interpret_signature(structure, "add", &[mass1, mass2]);
    assert!(result3.is_ok(), "Adding masses should work!");
    println!("✓ mass (kg) + mass (kg) → mass (kg) ✅");

    println!("\n🎉 UNIT-SAFE PHYSICS: Type system prevents dimensional errors!");
}
