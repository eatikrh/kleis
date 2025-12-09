// Test if we can actually EXECUTE self-hosted functions

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis_program;
use kleis::ast::Expression;

#[test]
fn test_execute_maybe_add_matrices() {
    let mut eval = Evaluator::new();
    
    // Load stdlib types
    let types_code = include_str!("../stdlib/types.kleis");
    let program = parse_kleis_program(types_code).unwrap();
    eval.load_program(&program).unwrap();
    
    println!("✅ Stdlib functions loaded into evaluator");
    
    // Define maybeAddMatrices function
    let func_code = r#"
        define maybeAddMatrices(optA, optB) = match optA {
          None => None
          | Some(a) => match optB {
              None => None
              | Some(b) => Some(a + b)
            }
        }
    "#;
    let func_program = parse_kleis_program(func_code).unwrap();
    eval.load_program(&func_program).unwrap();
    println!("✅ maybeAddMatrices function defined");
    
    // Now call maybeAddMatrices
    let m1 = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            Expression::List(vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
                Expression::Const("3".to_string()),
                Expression::Const("4".to_string()),
            ]),
        ],
    };
    
    let m2 = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            Expression::List(vec![
                Expression::Const("5".to_string()),
                Expression::Const("6".to_string()),
                Expression::Const("7".to_string()),
                Expression::Const("8".to_string()),
            ]),
        ],
    };
    
    let some_m1 = Expression::Operation {
        name: "Some".to_string(),
        args: vec![m1],
    };
    
    let some_m2 = Expression::Operation {
        name: "Some".to_string(),
        args: vec![m2],
    };
    
    // Call: maybeAddMatrices(Some(M1), Some(M2))
    let result = eval.apply_function("maybeAddMatrices", vec![some_m1, some_m2]);
    
    match &result {
        Ok(expr) => {
            println!("\n=== RESULT ===");
            println!("{:?}", expr);
            
            // Check if it's symbolic or actually computed
            match expr {
                Expression::Match { .. } => {
                    println!("\n⚠️ Result is symbolic Match expression (not executed)");
                    println!("Evaluator doesn't execute pattern matching yet");
                }
                Expression::Operation { name, .. } if name == "Some" => {
                    println!("\n✅ Result is Some(...) - pattern matching executed!");
                }
                _ => {
                    println!("\n❓ Unexpected result type");
                }
            }
        }
        Err(e) => {
            println!("\n❌ ERROR: {}", e);
        }
    }
}

#[test]
fn test_can_we_execute_head() {
    let mut eval = Evaluator::new();
    
    // Load stdlib
    let types_code = include_str!("../stdlib/types.kleis");
    let program = parse_kleis_program(types_code).unwrap();
    eval.load_program(&program).unwrap();
    
    // Create a list: Cons(42, Nil)
    let list = Expression::Operation {
        name: "Cons".to_string(),
        args: vec![
            Expression::Const("42".to_string()),
            Expression::Operation {
                name: "Nil".to_string(),
                args: vec![],
            },
        ],
    };
    
    // Call head(list)
    let result = eval.apply_function("head", vec![list]);
    
    match &result {
        Ok(expr) => {
            println!("\n=== head(Cons(42, Nil)) Result ===");
            println!("{:#?}", expr);
            
            match expr {
                Expression::Match { .. } => {
                    println!("\n⚠️ Returns symbolic Match (not executed)");
                }
                Expression::Operation { name, .. } if name == "Some" => {
                    println!("\n✅ Returns Some(42) - executed!");
                }
                _ => {}
            }
        }
        Err(e) => {
            println!("\n❌ ERROR: {}", e);
        }
    }
}

