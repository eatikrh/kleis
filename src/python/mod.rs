//! Python language support for Kleis code review.
//!
//! Contains a hand-written line scanner that parses Python source into
//! Kleis AST (Expression trees), enabling structural review rules.

mod scanner;

pub use scanner::scan_python;
