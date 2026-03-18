//! Native Rust structural scanner for Kleis code review.
//!
//! A hand-written, zero-dependency scanner that parses Rust source into
//! Kleis AST (Expression trees), enabling fast structural review rules.
//! Replaces the Kleis-interpreted scanner in `rust_parser.kleis`.
//!
//! Grammar reference: IntelliJ Rust BNF (MIT license)
//! https://github.com/intellij-rust/intellij-rust/blob/master/src/main/grammars/RustParser.bnf

mod scanner;

pub use scanner::scan_rust;
