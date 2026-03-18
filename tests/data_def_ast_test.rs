#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
/// Tests for DataDef AST (ADR-021: Algebraic Data Types)
///
/// This test file verifies that the DataDef, DataVariant, and DataField
/// structures can be created and used correctly.
use kleis::kleis_ast::{DataDef, DataField, DataVariant, Program, TopLevel, TypeExpr, TypeParam};

#[test]
fn test_simple_data_def_no_params() {
    // data Bool = True | False
    let data_def = DataDef {
        name: "Bool".to_string(),
        type_params: vec![],
        variants: vec![
            DataVariant {
                name: "True".to_string(),
                fields: vec![],
            },
            DataVariant {
                name: "False".to_string(),
                fields: vec![],
            },
        ],
    };

    assert_eq!(data_def.name, "Bool");
    assert_eq!(data_def.type_params.len(), 0);
    assert_eq!(data_def.variants.len(), 2);
    assert_eq!(data_def.variants[0].name, "True");
    assert_eq!(data_def.variants[1].name, "False");
    assert!(data_def.variants[0].fields.is_empty());
    assert!(data_def.variants[1].fields.is_empty());
}

#[test]
fn test_parametric_data_def() {
    // data Option(T) = None | Some(T)
    let data_def = DataDef {
        name: "Option".to_string(),
        type_params: vec![TypeParam {
            name: "T".to_string(),
            kind: None,
        }],
        variants: vec![
            DataVariant {
                name: "None".to_string(),
                fields: vec![],
            },
            DataVariant {
                name: "Some".to_string(),
                fields: vec![DataField {
                    name: None, // Positional field
                    type_expr: TypeExpr::Var("T".to_string()),
                }],
            },
        ],
    };

    assert_eq!(data_def.name, "Option");
    assert_eq!(data_def.type_params.len(), 1);
    assert_eq!(data_def.type_params[0].name, "T");
    assert_eq!(data_def.variants.len(), 2);

    // Check None variant
    assert_eq!(data_def.variants[0].name, "None");
    assert!(data_def.variants[0].fields.is_empty());

    // Check Some variant
    assert_eq!(data_def.variants[1].name, "Some");
    assert_eq!(data_def.variants[1].fields.len(), 1);
    assert!(data_def.variants[1].fields[0].name.is_none()); // Positional
    assert_eq!(
        data_def.variants[1].fields[0].type_expr,
        TypeExpr::Var("T".to_string())
    );
}

#[test]
fn test_complex_data_def_with_named_fields() {
    // data Type = Scalar | Matrix(m: Nat, n: Nat)
    let data_def = DataDef {
        name: "Type".to_string(),
        type_params: vec![],
        variants: vec![
            DataVariant {
                name: "Scalar".to_string(),
                fields: vec![],
            },
            DataVariant {
                name: "Matrix".to_string(),
                fields: vec![
                    DataField {
                        name: Some("m".to_string()),
                        type_expr: TypeExpr::Named("Nat".to_string()),
                    },
                    DataField {
                        name: Some("n".to_string()),
                        type_expr: TypeExpr::Named("Nat".to_string()),
                    },
                ],
            },
        ],
    };

    assert_eq!(data_def.name, "Type");
    assert!(data_def.type_params.is_empty());
    assert_eq!(data_def.variants.len(), 2);

    // Check Scalar variant
    assert_eq!(data_def.variants[0].name, "Scalar");
    assert!(data_def.variants[0].fields.is_empty());

    // Check Matrix variant
    let matrix_variant = &data_def.variants[1];
    assert_eq!(matrix_variant.name, "Matrix");
    assert_eq!(matrix_variant.fields.len(), 2);

    // Check first field (m: Nat)
    assert_eq!(matrix_variant.fields[0].name, Some("m".to_string()));
    assert_eq!(
        matrix_variant.fields[0].type_expr,
        TypeExpr::Named("Nat".to_string())
    );

    // Check second field (n: Nat)
    assert_eq!(matrix_variant.fields[1].name, Some("n".to_string()));
    assert_eq!(
        matrix_variant.fields[1].type_expr,
        TypeExpr::Named("Nat".to_string())
    );
}

#[test]
fn test_program_data_types_helper() {
    // Create a program with mixed top-level items
    let mut program = Program::new();

    // Add a data definition
    program.add_item(TopLevel::DataDef(DataDef {
        name: "Bool".to_string(),
        type_params: vec![],
        variants: vec![
            DataVariant {
                name: "True".to_string(),
                fields: vec![],
            },
            DataVariant {
                name: "False".to_string(),
                fields: vec![],
            },
        ],
    }));

    // Add another data definition
    program.add_item(TopLevel::DataDef(DataDef {
        name: "Option".to_string(),
        type_params: vec![TypeParam {
            name: "T".to_string(),
            kind: None,
        }],
        variants: vec![
            DataVariant {
                name: "None".to_string(),
                fields: vec![],
            },
            DataVariant {
                name: "Some".to_string(),
                fields: vec![DataField {
                    name: None,
                    type_expr: TypeExpr::Var("T".to_string()),
                }],
            },
        ],
    }));

    // Test the data_types() helper
    let data_types = program.data_types();
    assert_eq!(data_types.len(), 2);
    assert_eq!(data_types[0].name, "Bool");
    assert_eq!(data_types[1].name, "Option");
}

#[test]
fn test_multi_param_data_def() {
    // data Result(T, E) = Ok(value: T) | Err(error: E)
    let data_def = DataDef {
        name: "Result".to_string(),
        type_params: vec![
            TypeParam {
                name: "T".to_string(),
                kind: None,
            },
            TypeParam {
                name: "E".to_string(),
                kind: None,
            },
        ],
        variants: vec![
            DataVariant {
                name: "Ok".to_string(),
                fields: vec![DataField {
                    name: Some("value".to_string()),
                    type_expr: TypeExpr::Var("T".to_string()),
                }],
            },
            DataVariant {
                name: "Err".to_string(),
                fields: vec![DataField {
                    name: Some("error".to_string()),
                    type_expr: TypeExpr::Var("E".to_string()),
                }],
            },
        ],
    };

    assert_eq!(data_def.name, "Result");
    assert_eq!(data_def.type_params.len(), 2);
    assert_eq!(data_def.type_params[0].name, "T");
    assert_eq!(data_def.type_params[1].name, "E");
    assert_eq!(data_def.variants.len(), 2);

    // Check Ok variant
    let ok_variant = &data_def.variants[0];
    assert_eq!(ok_variant.name, "Ok");
    assert_eq!(ok_variant.fields.len(), 1);
    assert_eq!(ok_variant.fields[0].name, Some("value".to_string()));
    assert_eq!(
        ok_variant.fields[0].type_expr,
        TypeExpr::Var("T".to_string())
    );

    // Check Err variant
    let err_variant = &data_def.variants[1];
    assert_eq!(err_variant.name, "Err");
    assert_eq!(err_variant.fields.len(), 1);
    assert_eq!(err_variant.fields[0].name, Some("error".to_string()));
    assert_eq!(
        err_variant.fields[0].type_expr,
        TypeExpr::Var("E".to_string())
    );
}

#[test]
fn test_data_variant_with_multiple_fields() {
    // data Point = Point(x: Nat, y: Nat, z: Nat)
    let data_def = DataDef {
        name: "Point".to_string(),
        type_params: vec![],
        variants: vec![DataVariant {
            name: "Point".to_string(),
            fields: vec![
                DataField {
                    name: Some("x".to_string()),
                    type_expr: TypeExpr::Named("Nat".to_string()),
                },
                DataField {
                    name: Some("y".to_string()),
                    type_expr: TypeExpr::Named("Nat".to_string()),
                },
                DataField {
                    name: Some("z".to_string()),
                    type_expr: TypeExpr::Named("Nat".to_string()),
                },
            ],
        }],
    };

    assert_eq!(data_def.name, "Point");
    assert_eq!(data_def.variants.len(), 1);

    let point_variant = &data_def.variants[0];
    assert_eq!(point_variant.name, "Point");
    assert_eq!(point_variant.fields.len(), 3);
    assert_eq!(point_variant.fields[0].name, Some("x".to_string()));
    assert_eq!(point_variant.fields[1].name, Some("y".to_string()));
    assert_eq!(point_variant.fields[2].name, Some("z".to_string()));
}

#[test]
fn test_clone_and_equality() {
    // Test that DataDef, DataVariant, and DataField can be cloned and compared
    let original = DataDef {
        name: "Test".to_string(),
        type_params: vec![],
        variants: vec![DataVariant {
            name: "Value".to_string(),
            fields: vec![DataField {
                name: Some("field".to_string()),
                type_expr: TypeExpr::Named("Nat".to_string()),
            }],
        }],
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);

    // Test that different values are not equal
    let different = DataDef {
        name: "Different".to_string(),
        type_params: vec![],
        variants: vec![],
    };
    assert_ne!(original, different);
}
