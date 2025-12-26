//! Kleis Language Server Protocol (LSP) Implementation
//!
//! This module provides IDE support for Kleis via the Language Server Protocol:
//! - Real-time diagnostics (parse errors, type errors)
//! - Hover information (type signatures from imports)
//! - Go to definition
//! - Document symbols
//! - Context-aware completions (based on imports)
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Run LSP over stdio (for VS Code integration)
//! kleis::lsp::run_lsp()?;
//!
//! // Or with shared context
//! let ctx = kleis::context::create_shared_context();
//! kleis::lsp::run_lsp_with_context(ctx)?;
//! ```

mod server;

pub use server::KleisLanguageServer;

use crate::context::SharedContext;

/// Run the LSP server over stdio
pub fn run_lsp() -> Result<(), Box<dyn std::error::Error>> {
    server::run_stdio()
}

/// Run the LSP server with shared context
pub fn run_lsp_with_context(ctx: SharedContext) -> Result<(), Box<dyn std::error::Error>> {
    server::run_stdio_with_context(ctx)
}

