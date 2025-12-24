//! Kleis Context - Shared State for LSP, REPL, and Debugger
//!
//! This module provides a unified context that is shared between all
//! Kleis interfaces (LSP, REPL, Debugger). This enables:
//!
//! - **Consistent state**: All interfaces see the same parsed program
//! - **Shared type information**: Hover in LSP uses same types as REPL
//! - **Live debugging**: Debugger can inspect evaluator state
//! - **Incremental updates**: Parse once, use everywhere
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │              KleisContext                    │
//! │  ├─ documents: HashMap<PathBuf, Document>   │
//! │  ├─ type_checker: TypeChecker               │
//! │  ├─ evaluator: Evaluator                    │
//! │  └─ solver: Option<Z3Backend>               │
//! ├─────────────────────────────────────────────┤
//! │  LSP Server  │  REPL  │  DAP Debugger       │
//! │  (hover,     │ (:eval │  (breakpoints,      │
//! │   diags)     │  :type)│   variables)        │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::{Arc, RwLock};
//! use kleis::context::KleisContext;
//!
//! // Create shared context
//! let ctx = Arc::new(RwLock::new(KleisContext::new()));
//!
//! // Pass to different interfaces
//! lsp::run(ctx.clone());
//! repl::run(ctx.clone());
//! dap::run(ctx.clone());
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::evaluator::Evaluator;
use crate::kleis_ast::Program;
use crate::type_checker::TypeChecker;

/// A parsed and analyzed document
pub struct Document {
    /// Original source code
    pub source: String,
    /// Parsed AST (None if parsing failed)
    pub program: Option<Program>,
    /// Parse/type errors for this document
    pub diagnostics: Vec<Diagnostic>,
    /// Import paths found in this document
    pub imports: Vec<String>,
    /// Whether the document has been modified since last parse
    pub dirty: bool,
}

/// A diagnostic message (error, warning, info)
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Byte offset of start
    pub start: usize,
    /// Byte offset of end
    pub end: usize,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Human-readable message
    pub message: String,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// The main Kleis context - shared state for all interfaces
///
/// This struct holds all the state that needs to be shared between
/// LSP, REPL, and Debugger. It is designed to be wrapped in
/// `Arc<RwLock<KleisContext>>` for thread-safe sharing.
pub struct KleisContext {
    /// Open documents indexed by file path
    pub documents: HashMap<PathBuf, Document>,

    /// Type checker for type inference and checking
    pub type_checker: TypeChecker,

    /// Evaluator for concrete execution
    pub evaluator: Evaluator,

    /// Whether Z3 solver is enabled (solver itself is created per-use due to lifetime)
    #[cfg(feature = "axiom-verification")]
    pub solver_enabled: bool,

    /// Current working directory
    pub cwd: PathBuf,

    /// Debug state (if debugging is active)
    pub debug_state: Option<DebugState>,
}

/// State for an active debug session
#[derive(Debug)]
pub struct DebugState {
    /// The file being debugged
    pub program_path: PathBuf,
    /// Current execution state
    pub execution_state: ExecutionState,
    /// Breakpoints by file
    pub breakpoints: HashMap<PathBuf, Vec<Breakpoint>>,
    /// Current stack frames
    pub stack_frames: Vec<StackFrame>,
}

/// Execution state during debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// Not started
    NotStarted,
    /// Running (not stopped at breakpoint)
    Running,
    /// Stopped at a breakpoint or step
    Stopped,
    /// Execution completed
    Terminated,
}

/// A breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Line number (1-indexed)
    pub line: u32,
    /// Optional condition expression
    pub condition: Option<String>,
    /// Whether the breakpoint is verified/valid
    pub verified: bool,
}

/// A stack frame during debugging
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Frame ID
    pub id: u32,
    /// Function name
    pub name: String,
    /// Source file
    pub source: Option<PathBuf>,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}

impl KleisContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            type_checker: TypeChecker::new(),
            evaluator: Evaluator::new(),
            #[cfg(feature = "axiom-verification")]
            solver_enabled: false,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            debug_state: None,
        }
    }

    /// Create a new context with Z3 solver enabled
    #[cfg(feature = "axiom-verification")]
    pub fn with_solver() -> Self {
        let mut ctx = Self::new();
        ctx.solver_enabled = true;
        ctx
    }

    /// Open a document from a file path
    pub fn open_document(&mut self, path: PathBuf) -> Result<(), String> {
        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        self.set_document_content(path, source);
        Ok(())
    }

    /// Set document content (for LSP didOpen/didChange)
    pub fn set_document_content(&mut self, path: PathBuf, source: String) {
        use crate::kleis_parser::parse_kleis_program;

        let mut diagnostics = Vec::new();
        let mut program = None;

        // Parse the document
        match parse_kleis_program(&source) {
            Ok(p) => {
                program = Some(p);
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    start: e.position,
                    end: e.position + 1,
                    severity: DiagnosticSeverity::Error,
                    message: e.message,
                });
            }
        }

        // Extract imports
        let imports = if let Some(ref p) = program {
            p.items
                .iter()
                .filter_map(|item| {
                    if let crate::kleis_ast::TopLevel::Import(path) = item {
                        Some(path.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let doc = Document {
            source,
            program,
            diagnostics,
            imports,
            dirty: false,
        };

        self.documents.insert(path, doc);
    }

    /// Close a document
    pub fn close_document(&mut self, path: &PathBuf) {
        self.documents.remove(path);
    }

    /// Get a document by path
    pub fn get_document(&self, path: &PathBuf) -> Option<&Document> {
        self.documents.get(path)
    }

    /// Get a mutable document by path
    pub fn get_document_mut(&mut self, path: &PathBuf) -> Option<&mut Document> {
        self.documents.get_mut(path)
    }

    /// Load a program into the evaluator
    pub fn load_program(&mut self, path: &PathBuf) -> Result<(), String> {
        let doc = self
            .documents
            .get(path)
            .ok_or_else(|| format!("Document not open: {}", path.display()))?;

        let program = doc
            .program
            .as_ref()
            .ok_or_else(|| "Document has parse errors".to_string())?;

        self.evaluator.load_program(program)?;
        Ok(())
    }

    /// Start a debug session
    pub fn start_debug_session(&mut self, program_path: PathBuf) -> Result<(), String> {
        // Open and load the program if not already open
        if !self.documents.contains_key(&program_path) {
            self.open_document(program_path.clone())?;
        }
        self.load_program(&program_path)?;

        self.debug_state = Some(DebugState {
            program_path,
            execution_state: ExecutionState::NotStarted,
            breakpoints: HashMap::new(),
            stack_frames: Vec::new(),
        });

        Ok(())
    }

    /// End the current debug session
    pub fn end_debug_session(&mut self) {
        self.debug_state = None;
    }

    /// Set breakpoints for a file
    pub fn set_breakpoints(&mut self, path: PathBuf, breakpoints: Vec<Breakpoint>) {
        if let Some(ref mut debug_state) = self.debug_state {
            debug_state.breakpoints.insert(path, breakpoints);
        }
    }
}

impl Default for KleisContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared context
pub type SharedContext = Arc<RwLock<KleisContext>>;

/// Create a new shared context
pub fn create_shared_context() -> SharedContext {
    Arc::new(RwLock::new(KleisContext::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = KleisContext::new();
        assert!(ctx.documents.is_empty());
    }

    #[test]
    fn test_document_content() {
        let mut ctx = KleisContext::new();
        let path = PathBuf::from("test.kleis");
        // Use valid Kleis syntax: a define statement
        let source = "define answer = 42".to_string();

        ctx.set_document_content(path.clone(), source);

        let doc = ctx.get_document(&path).unwrap();
        assert_eq!(doc.source, "define answer = 42");
        assert!(doc.program.is_some());
        assert!(doc.diagnostics.is_empty());
    }

    #[test]
    fn test_shared_context() {
        let ctx = create_shared_context();

        // Can clone and share
        let ctx2 = ctx.clone();

        // Can write (use valid Kleis syntax)
        {
            let mut guard = ctx.write().unwrap();
            guard.set_document_content(PathBuf::from("test.kleis"), "define x = 1".to_string());
        }

        // Can read from clone
        {
            let guard = ctx2.read().unwrap();
            assert!(guard.get_document(&PathBuf::from("test.kleis")).is_some());
        }
    }
}
