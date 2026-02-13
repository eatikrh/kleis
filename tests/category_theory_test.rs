//! Tests for stdlib/category_theory.kleis

use kleis::kleis_parser::parse_kleis_program;

#[test]
fn test_category_theory_parses() {
    let source = std::fs::read_to_string("stdlib/category_theory.kleis")
        .expect("Failed to read stdlib/category_theory.kleis");
    parse_kleis_program(&source).expect("Failed to parse stdlib/category_theory.kleis");
}
