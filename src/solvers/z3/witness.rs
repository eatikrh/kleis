//! Z3 Witness Extraction — Z3 Model → Kleis Witness
//!
//! **CRITICAL ABSTRACTION BOUNDARY**
//!
//! This module converts Z3's `Model` (which contains solver-internal types like
//! `Dynamic`, `FuncDecl`, `Int`, `Real`) into solver-independent `Witness` structs
//! containing Kleis `Expression` values.
//!
//! Z3 types MUST NOT escape this module. The output is always `Witness` with
//! `Expression` bindings that can be pretty-printed, fed back into evaluation,
//! or used in CEGAR loops.
//!
//! **Design Pattern:**
//! ```text
//! Z3Backend::verify_axiom()
//!    |
//! Z3 says Sat (counterexample exists)
//!    |
//! model_to_witness(model, quantifier_vars, converter, ...) ← THIS MODULE
//!    |
//! Witness { bindings: [(x, Const("0")), (y, Const("42"))], raw: "..." }
//!    ^
//! Consumers (MCP server, REPL, LSP) render as Kleis syntax
//! ```
//!
//! **Why a separate module?**
//! - Mirrors `converter.rs` (single values) vs `witness.rs` (entire models)
//! - A CVC5 backend would have its own `witness.rs` producing the same `Witness` struct
//! - Keeps the Z3 `Model` iteration logic isolated from the main backend

use crate::ast::Expression;
use crate::solvers::backend::{Witness, WitnessBinding};
use crate::solvers::result_converter::ResultConverter;
use crate::solvers::z3::converter::Z3ResultConverter;
use z3::ast::{Ast, Dynamic};
use z3::Model;

/// Extract a structured `Witness` from a Z3 model using tracked quantifier variables.
///
/// # Arguments
/// * `model` — Z3 model from a `Sat` result
/// * `quantifier_vars` — `(kleis_name, z3_dynamic)` pairs saved during `kleis_to_z3` translation
/// * `converter` — converts individual Z3 `Dynamic` values to Kleis `Expression`
/// * `declared_data_types` — maps data type names to Z3 `DatatypeSort`, used to
///   reverse-map Z3 constructor indices back to Kleis constructor names
///
/// # Strategy
///
/// 1. Capture the raw model string (for debugging / fallback)
/// 2. For each quantifier variable:
///    a. Call `model.eval(&z3_var, model_completion=true)` to get a concrete value
///    b. Check if it's a datatype constructor → reverse-map to Kleis constructor name
///    c. Otherwise use `Z3ResultConverter::to_expression()` for standard types
/// 3. If no quantifier vars are tracked, fall back to raw model string only
///
/// # Returns
/// A `Witness` with structured bindings (if extraction succeeds) or raw-only fallback.
pub fn model_to_witness(
    model: &Model,
    quantifier_vars: &[(String, Dynamic)],
    converter: &Z3ResultConverter,
    declared_data_types: &std::collections::HashMap<String, z3::DatatypeSort>,
) -> Witness {
    let raw = format!("{}", model);

    if quantifier_vars.is_empty() {
        return Witness::from_raw(raw);
    }

    let mut bindings = Vec::with_capacity(quantifier_vars.len());

    for (kleis_name, z3_var) in quantifier_vars {
        match extract_binding(model, kleis_name, z3_var, converter, declared_data_types) {
            Ok(binding) => bindings.push(binding),
            Err(_) => {
                // If we can't extract a binding for this variable,
                // include it as a raw fallback
                bindings.push(WitnessBinding {
                    name: kleis_name.clone(),
                    value: Expression::Const("<unknown: see raw model>".to_string()),
                });
            }
        }
    }

    Witness { bindings, raw }
}

/// Extract a single witness binding from the Z3 model.
///
/// Uses `model.eval()` with `model_completion=true` to get a concrete value,
/// then converts to a Kleis `Expression`.
fn extract_binding(
    model: &Model,
    kleis_name: &str,
    z3_var: &Dynamic,
    converter: &Z3ResultConverter,
    declared_data_types: &std::collections::HashMap<String, z3::DatatypeSort>,
) -> Result<WitnessBinding, String> {
    // Evaluate the Z3 variable in the model to get a concrete value
    let evaluated = model
        .eval(z3_var, true)
        .ok_or_else(|| format!("Model has no value for '{}'", kleis_name))?;

    // Try to reverse-map datatype constructors first
    if let Some(expr) = try_reverse_map_datatype(&evaluated, declared_data_types) {
        return Ok(WitnessBinding {
            name: kleis_name.to_string(),
            value: expr,
        });
    }

    // Standard conversion via Z3ResultConverter
    let value = converter.to_expression(&evaluated)?;

    Ok(WitnessBinding {
        name: kleis_name.to_string(),
        value,
    })
}

/// Attempt to reverse-map a Z3 datatype value to a Kleis constructor expression.
///
/// Z3 represents algebraic data type values as constructor applications:
/// - Nullary: `Red` (just a constant)
/// - With fields: `Pair(3, 4)` (constructor applied to arguments)
///
/// The `declared_data_types` map lets us find the constructor name from Z3's
/// internal representation.
///
/// # Returns
/// `Some(Expression)` if the value is a recognized datatype constructor,
/// `None` otherwise (caller falls through to standard conversion).
fn try_reverse_map_datatype(
    value: &Dynamic,
    declared_data_types: &std::collections::HashMap<String, z3::DatatypeSort>,
) -> Option<Expression> {
    use z3::SortKind;

    // Only attempt for datatype sorts
    if value.sort_kind() != SortKind::Datatype {
        return None;
    }

    let value_sort_name = value.get_sort().to_string();

    // Find the matching declared data type
    for dt_sort in declared_data_types.values() {
        if dt_sort.sort.to_string() != value_sort_name {
            continue;
        }

        // Try each variant's tester to find which constructor this is
        for variant in &dt_sort.variants {
            let constructor_name = variant.constructor.name();

            // Check if this value was built by this constructor
            // by applying the tester function and simplifying
            let tester = &variant.tester;
            let test_result = tester.apply(&[value]).simplify();

            // If the tester evaluates to true, this is our constructor
            if let Some(bool_val) = test_result.as_bool() {
                if bool_val.as_bool() == Some(true) {
                    if variant.accessors.is_empty() {
                        // Nullary constructor: just the name
                        return Some(Expression::Object(constructor_name));
                    } else {
                        // Constructor with fields: extract each field
                        let mut args = Vec::new();
                        for accessor in &variant.accessors {
                            let field_val = accessor.apply(&[value]);
                            // Recursively try reverse-mapping nested datatypes
                            if let Some(nested) =
                                try_reverse_map_datatype(&field_val, declared_data_types)
                            {
                                args.push(nested);
                            } else {
                                // Standard conversion for primitive fields
                                let converter = Z3ResultConverter;
                                match converter.to_expression(&field_val) {
                                    Ok(expr) => args.push(expr),
                                    Err(_) => args.push(Expression::Const(field_val.to_string())),
                                }
                            }
                        }
                        return Some(Expression::operation(constructor_name, args));
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::ast::Int;
    use z3::Solver;

    #[test]
    fn test_model_to_witness_basic_int() {
        // Create a simple model: x = 42
        let solver = Solver::new();
        let x = Int::new_const("x");
        solver.assert(x.eq(&Int::from_i64(42)));

        assert_eq!(solver.check(), z3::SatResult::Sat);
        let model = solver.get_model().unwrap();

        let z3_x: Dynamic = x.into();
        let quantifier_vars = vec![("x".to_string(), z3_x)];
        let converter = Z3ResultConverter;

        let witness = model_to_witness(
            &model,
            &quantifier_vars,
            &converter,
            &std::collections::HashMap::new(),
        );

        assert!(witness.has_bindings());
        assert_eq!(witness.bindings.len(), 1);
        assert_eq!(witness.bindings[0].name, "x");
        assert_eq!(
            witness.bindings[0].value,
            Expression::Const("42".to_string())
        );

        // Display should produce "x = 42"
        assert_eq!(witness.to_string(), "x = 42");
    }

    #[test]
    fn test_model_to_witness_multiple_vars() {
        let solver = Solver::new();
        let x = Int::new_const("x");
        let y = Int::new_const("y");
        solver.assert(x.eq(&Int::from_i64(3)));
        solver.assert(y.eq(&Int::from_i64(4)));

        assert_eq!(solver.check(), z3::SatResult::Sat);
        let model = solver.get_model().unwrap();

        let z3_x: Dynamic = x.into();
        let z3_y: Dynamic = y.into();
        let quantifier_vars = vec![("x".to_string(), z3_x), ("y".to_string(), z3_y)];
        let converter = Z3ResultConverter;

        let witness = model_to_witness(
            &model,
            &quantifier_vars,
            &converter,
            &std::collections::HashMap::new(),
        );

        assert_eq!(witness.bindings.len(), 2);
        assert_eq!(witness.bindings[0].name, "x");
        assert_eq!(
            witness.bindings[0].value,
            Expression::Const("3".to_string())
        );
        assert_eq!(witness.bindings[1].name, "y");
        assert_eq!(
            witness.bindings[1].value,
            Expression::Const("4".to_string())
        );

        assert_eq!(witness.to_string(), "x = 3, y = 4");
    }

    #[test]
    fn test_model_to_witness_empty_vars_falls_back_to_raw() {
        let solver = Solver::new();
        let x = Int::new_const("x");
        solver.assert(x.eq(&Int::from_i64(7)));

        assert_eq!(solver.check(), z3::SatResult::Sat);
        let model = solver.get_model().unwrap();

        let converter = Z3ResultConverter;
        let witness = model_to_witness(&model, &[], &converter, &std::collections::HashMap::new());

        assert!(!witness.has_bindings());
        // Display falls back to raw Z3 model
        assert!(!witness.raw.is_empty());
    }

    #[test]
    fn test_model_to_witness_bool_var() {
        let solver = Solver::new();
        let b = z3::ast::Bool::new_const("flag");
        solver.assert(&b);

        assert_eq!(solver.check(), z3::SatResult::Sat);
        let model = solver.get_model().unwrap();

        let z3_b: Dynamic = b.into();
        let quantifier_vars = vec![("flag".to_string(), z3_b)];
        let converter = Z3ResultConverter;

        let witness = model_to_witness(
            &model,
            &quantifier_vars,
            &converter,
            &std::collections::HashMap::new(),
        );

        assert_eq!(witness.bindings.len(), 1);
        assert_eq!(witness.bindings[0].name, "flag");
        // Z3 Bool display: "true"
        assert_eq!(
            witness.bindings[0].value,
            Expression::Const("true".to_string())
        );
    }

    #[test]
    fn test_model_to_witness_counterexample() {
        // Verify ∀x. x * x > 0 — should fail at x = 0
        let solver = Solver::new();
        let x = Int::new_const("x");

        // Assert negation: ¬(x * x > 0) i.e. x * x <= 0
        let x_sq = Int::mul(&[&x, &x]);
        let zero = Int::from_i64(0);
        solver.assert(x_sq.le(&zero));

        assert_eq!(solver.check(), z3::SatResult::Sat);
        let model = solver.get_model().unwrap();

        let z3_x: Dynamic = x.into();
        let quantifier_vars = vec![("x".to_string(), z3_x)];
        let converter = Z3ResultConverter;

        let witness = model_to_witness(
            &model,
            &quantifier_vars,
            &converter,
            &std::collections::HashMap::new(),
        );

        assert_eq!(witness.bindings.len(), 1);
        assert_eq!(witness.bindings[0].name, "x");
        assert_eq!(
            witness.bindings[0].value,
            Expression::Const("0".to_string())
        );
        assert_eq!(witness.to_string(), "x = 0");
    }

    #[test]
    fn test_model_to_witness_with_datatype() {
        // Create a Color datatype: Red | Green | Blue
        let color_sort = z3::DatatypeBuilder::new("Color")
            .variant("Red", vec![])
            .variant("Green", vec![])
            .variant("Blue", vec![])
            .finish();

        let solver = Solver::new();

        // Assert c = Red (using the first constructor)
        let c = Dynamic::fresh_const("c", &color_sort.sort);
        let red = color_sort.variants[0].constructor.apply(&[]);
        solver.assert(c.eq(&red));

        assert_eq!(solver.check(), z3::SatResult::Sat);
        let model = solver.get_model().unwrap();

        let quantifier_vars = vec![("c".to_string(), c)];
        let converter = Z3ResultConverter;

        let mut dt_map = std::collections::HashMap::new();
        dt_map.insert("Color".to_string(), color_sort);

        let witness = model_to_witness(&model, &quantifier_vars, &converter, &dt_map);

        assert_eq!(witness.bindings.len(), 1);
        assert_eq!(witness.bindings[0].name, "c");
        // Should reverse-map to the Kleis constructor name "Red"
        assert_eq!(
            witness.bindings[0].value,
            Expression::Object("Red".to_string())
        );
        assert_eq!(witness.to_string(), "c = Red");
    }

    #[test]
    fn test_witness_display_format() {
        let witness = Witness {
            bindings: vec![
                WitnessBinding {
                    name: "x".to_string(),
                    value: Expression::Const("0".to_string()),
                },
                WitnessBinding {
                    name: "y".to_string(),
                    value: Expression::Const("42".to_string()),
                },
            ],
            raw: "x -> 0\ny -> 42".to_string(),
        };

        assert_eq!(witness.to_string(), "x = 0, y = 42");
    }

    #[test]
    fn test_witness_display_empty_falls_back_to_raw() {
        let witness = Witness::from_raw("some_func -> {\n  else -> 0\n}".to_string());
        assert_eq!(witness.to_string(), "some_func -> {\n  else -> 0\n}");
    }
}
