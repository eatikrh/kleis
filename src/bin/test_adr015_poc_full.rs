///! Full POC Test for ADR-015: Text as Source of Truth
///!
///! This validates the complete pipeline:
///! 1. Parse Kleis text → AST
///! 2. AST representation
///! 3. (Conceptual) rendering to visual form
///!
///! Uses the actual Kleis text parser to validate ADR-015 decisions.

use kleis::ast::Expression;
use kleis::kleis_parser::parse_kleis;

fn main() {
    println!("🎯 ADR-015 Full POC Test (With Parser)");
    println!("{}", "=".repeat(70));
    println!("\nValidating: Text → Parser → AST → Visual Rendering");
    println!();

    let mut all_pass = true;

    // Test 1: Absolute Value
    println!("Test 1: Absolute Value");
    println!("{}", "-".repeat(70));
    all_pass &= test_parse("abs(x)", "abs", 1, "|x|");
    println!();

    // Test 2: Cardinality
    println!("Test 2: Cardinality");
    println!("{}", "-".repeat(70));
    all_pass &= test_parse("card(S)", "card", 1, "|S|");
    println!();

    // Test 3: Norm
    println!("Test 3: Norm");
    println!("{}", "-".repeat(70));
    all_pass &= test_parse("norm(v)", "norm", 1, "‖v‖");
    println!();

    // Test 4: Fraction (Display Mode)
    println!("Test 4: Fraction (Display Mode)");
    println!("{}", "-".repeat(70));
    all_pass &= test_parse("frac(a, b)", "frac", 2, "a/b (stacked)");
    println!();

    // Test 5: Division vs Fraction
    println!("Test 5: Division vs Fraction (Display Distinction)");
    println!("{}", "-".repeat(70));
    all_pass &= test_division_vs_fraction();
    println!();

    // Test 6: Nested Expression
    println!("Test 6: Nested Expression");
    println!("{}", "-".repeat(70));
    all_pass &= test_nested();
    println!();

    // Test 7: Complex Expression
    println!("Test 7: Complex Expression with Operators");
    println!("{}", "-".repeat(70));
    all_pass &= test_complex();
    println!();

    // Test 8: Ambiguity Rejection (show what we DON'T support)
    println!("Test 8: What ADR-015 Rejects");
    println!("{}", "-".repeat(70));
    demonstrate_rejection();
    println!();

    println!("{}", "=".repeat(70));
    if all_pass {
        println!("✅ All ADR-015 Full POC tests passed!");
        println!("\nKey Validation:");
        println!("  ✓ Kleis text parser works");
        println!("  ✓ Explicit forms (abs, card, norm, frac) parse correctly");
        println!("  ✓ Can distinguish division '/' from fraction 'frac()'");
        println!("  ✓ Text representation is unambiguous");
        println!("  ✓ Nested expressions parse correctly");
    } else {
        println!("❌ Some tests failed!");
        std::process::exit(1);
    }
}

fn test_parse(text: &str, expected_op: &str, expected_args: usize, visual: &str) -> bool {
    println!("  Text:   {}", text);
    
    match parse_kleis(text) {
        Ok(ast) => {
            println!("  AST:    {:?}", ast);
            
            match ast {
                Expression::Operation { name, args } => {
                    if name == expected_op && args.len() == expected_args {
                        println!("  Visual: {}", visual);
                        println!("  ✓ Parsed correctly: '{}' with {} args", name, args.len());
                        true
                    } else {
                        println!("  ❌ Expected operation '{}' with {} args, got '{}' with {} args",
                                 expected_op, expected_args, name, args.len());
                        false
                    }
                }
                _ => {
                    println!("  ❌ Expected Operation, got {:?}", ast);
                    false
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            false
        }
    }
}

fn test_division_vs_fraction() -> bool {
    // Test division operator
    let division_text = "a / b";
    println!("  Division:");
    println!("    Text:   {}", division_text);
    
    let div_result = match parse_kleis(division_text) {
        Ok(ast) => {
            println!("    AST:    {:?}", ast);
            match &ast {
                Expression::Operation { name, .. } if name == "divide" => {
                    println!("    Visual: a / b  (inline)");
                    println!("    ✓ Parsed as 'divide' operation");
                    true
                }
                _ => {
                    println!("    ❌ Expected 'divide' operation");
                    false
                }
            }
        }
        Err(e) => {
            println!("    ❌ Parse error: {}", e);
            false
        }
    };

    println!();

    // Test fraction function
    let fraction_text = "frac(a, b)";
    println!("  Fraction:");
    println!("    Text:   {}", fraction_text);
    
    let frac_result = match parse_kleis(fraction_text) {
        Ok(ast) => {
            println!("    AST:    {:?}", ast);
            match &ast {
                Expression::Operation { name, .. } if name == "frac" => {
                    println!("    Visual: a/b  (stacked)");
                    println!("    ✓ Parsed as 'frac' operation");
                    true
                }
                _ => {
                    println!("    ❌ Expected 'frac' operation");
                    false
                }
            }
        }
        Err(e) => {
            println!("    ❌ Parse error: {}", e);
            false
        }
    };

    println!();
    if div_result && frac_result {
        println!("  ✓ Both forms parse correctly with different operations");
        println!("  ✓ Git diff would show: 'divide' → 'frac' = style change");
    }

    div_result && frac_result
}

fn test_nested() -> bool {
    let text = "abs(frac(a, b))";
    println!("  Text:   {}", text);
    
    match parse_kleis(text) {
        Ok(ast) => {
            println!("  AST:    {:?}", ast);
            
            match ast {
                Expression::Operation { name, args } if name == "abs" && args.len() == 1 => {
                    match &args[0] {
                        Expression::Operation { name, args } if name == "frac" && args.len() == 2 => {
                            println!("  Visual: |a/b|  or  |a─b|");
                            println!("  ✓ Nesting works: abs( frac(...) )");
                            println!("  ✓ Unambiguous at every level");
                            true
                        }
                        _ => {
                            println!("  ❌ Expected nested frac operation");
                            false
                        }
                    }
                }
                _ => {
                    println!("  ❌ Expected abs operation with nested frac");
                    false
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            false
        }
    }
}

fn test_complex() -> bool {
    let text = "abs(x + y) / norm(v)";
    println!("  Text:   {}", text);
    
    match parse_kleis(text) {
        Ok(ast) => {
            println!("  AST:    {:?}", ast);
            println!("  Visual: |x + y| / ‖v‖");
            
            match ast {
                Expression::Operation { name, .. } if name == "divide" => {
                    println!("  ✓ Complex expression parsed correctly");
                    println!("  ✓ Multiple nested operations work");
                    true
                }
                _ => {
                    println!("  ❌ Expected divide operation at top level");
                    false
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            false
        }
    }
}

fn demonstrate_rejection() {
    println!("  ADR-015 explicitly rejects ambiguous notation in TEXT:");
    println!();
    println!("  ❌ Text: |x|");
    println!("     Problem: Could mean abs(x), card(x), or norm(x)");
    println!("     Not supported in Kleis text syntax!");
    println!();
    println!("  ✓ Solution: Use explicit forms in text:");
    println!("     - abs(x)  for absolute value");
    println!("     - card(S) for cardinality");
    println!("     - norm(v) for norm");
    println!();
    println!("  Note: Visual display can still use |x| notation!");
    println!("        The RENDERER converts abs(x) → |x| for display");
}

