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

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
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
    /// Import paths found in this document (resolved to absolute paths)
    pub imports: HashSet<PathBuf>,
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
    ///
    /// This method:
    /// 1. Parses the source code
    /// 2. Extracts and resolves imports
    /// 3. Triggers cascading invalidation of dependent documents
    pub fn set_document_content(&mut self, path: PathBuf, source: String) {
        use crate::kleis_parser::parse_kleis_program_with_file;

        // Canonicalize path for consistency with VS Code
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());

        // Mark this document and all dependents as dirty
        self.invalidate_dependents(&canonical);

        let mut diagnostics = Vec::new();
        let mut program = None;

        // Parse with canonicalized file path for VS Code debugging support
        let file_path_str = canonical.to_string_lossy().to_string();
        match parse_kleis_program_with_file(&source, &file_path_str) {
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

        // Extract and resolve imports to absolute paths
        let imports: HashSet<PathBuf> = if let Some(ref p) = program {
            p.items
                .iter()
                .filter_map(|item| {
                    if let crate::kleis_ast::TopLevel::Import(import_path) = item {
                        self.resolve_import_path(import_path, &canonical)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            HashSet::new()
        };

        let doc = Document {
            source,
            program,
            diagnostics,
            imports,
            dirty: false,
        };

        self.documents.insert(canonical, doc);
    }

    /// Resolve an import path relative to the importing file
    ///
    /// Handles:
    /// - Relative paths (./foo.kleis, ../bar.kleis)
    /// - stdlib/ paths (checks KLEIS_ROOT first, then searches common locations)
    /// - Absolute paths (passed through)
    fn resolve_import_path(&self, import_path: &str, from_file: &Path) -> Option<PathBuf> {
        // Handle stdlib imports specially
        if import_path.starts_with("stdlib/") {
            // First, check KLEIS_ROOT environment variable
            if let Ok(kleis_root) = std::env::var("KLEIS_ROOT") {
                let candidate = PathBuf::from(&kleis_root).join(import_path);
                if candidate.exists() {
                    return candidate.canonicalize().ok();
                }
            }

            // Try relative to current working directory
            let stdlib_path = self.cwd.join(import_path);
            if stdlib_path.exists() {
                return stdlib_path.canonicalize().ok();
            }

            // Try relative to project root (common pattern)
            if let Some(parent) = from_file.parent() {
                // Walk up looking for stdlib directory
                let mut dir = parent.to_path_buf();
                for _ in 0..10 {
                    // Max 10 levels up
                    let candidate = dir.join(import_path);
                    if candidate.exists() {
                        return candidate.canonicalize().ok();
                    }
                    if let Some(p) = dir.parent() {
                        dir = p.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
            return None;
        }

        // Handle relative paths
        if let Some(parent) = from_file.parent() {
            let resolved = parent.join(import_path);
            if resolved.exists() {
                return resolved.canonicalize().ok();
            }
            // Try with .kleis extension
            let with_ext = parent.join(format!("{}.kleis", import_path));
            if with_ext.exists() {
                return with_ext.canonicalize().ok();
            }
        }

        // Try as absolute path
        let abs_path = PathBuf::from(import_path);
        if abs_path.exists() {
            return abs_path.canonicalize().ok();
        }

        None
    }

    /// Invalidate a document and all documents that depend on it
    ///
    /// Iterates over all documents to find those that import the changed file.
    /// Cascades upward: if A imports B and B is dirty, A becomes dirty too.
    fn invalidate_dependents(&mut self, path: &PathBuf) {
        // Mark the changed document as dirty
        if let Some(doc) = self.documents.get_mut(path) {
            doc.dirty = true;
        }

        // Keep iterating until no new documents are marked dirty
        loop {
            let mut newly_dirtied = false;

            // Collect paths of dirty documents
            let dirty_paths: HashSet<PathBuf> = self
                .documents
                .iter()
                .filter(|(_, doc)| doc.dirty)
                .map(|(p, _)| p.clone())
                .collect();

            // For each non-dirty document, check if it imports any dirty document
            for (doc_path, doc) in self.documents.iter_mut() {
                if doc.dirty {
                    continue;
                }

                // If this document imports any dirty file, mark it dirty
                if doc.imports.iter().any(|imp| dirty_paths.contains(imp)) {
                    doc.dirty = true;
                    newly_dirtied = true;
                }

                // Also mark dirty if the document path itself is the changed one
                if doc_path == path {
                    doc.dirty = true;
                    newly_dirtied = true;
                }
            }

            if !newly_dirtied {
                break;
            }
        }
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

    /// Get the AST for a file, loading and parsing if necessary
    ///
    /// This is the main entry point for DAP to get ASTs.
    /// It reuses cached ASTs from LSP parsing when available.
    pub fn get_or_load_ast(&mut self, path: &Path) -> Result<&Program, String> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Check if we need to (re)parse
        let needs_parse = match self.documents.get(&canonical) {
            None => true,
            Some(doc) => doc.dirty || doc.program.is_none(),
        };

        if needs_parse {
            let source = std::fs::read_to_string(&canonical)
                .map_err(|e| format!("Failed to read {}: {}", canonical.display(), e))?;
            self.set_document_content(canonical.clone(), source);
        }

        // Now get the AST
        self.documents
            .get(&canonical)
            .and_then(|doc| doc.program.as_ref())
            .ok_or_else(|| format!("Failed to parse {}", canonical.display()))
    }

    /// Get all cached ASTs for a program and its imports
    ///
    /// Returns a list of (path, program) pairs in dependency order (leaves first).
    /// This is useful for the debugger to know all files that might have breakpoints.
    pub fn get_all_program_asts(
        &mut self,
        main_path: &Path,
    ) -> Result<Vec<(PathBuf, Program)>, String> {
        let canonical = main_path
            .canonicalize()
            .unwrap_or_else(|_| main_path.to_path_buf());

        // First, ensure the main file is parsed
        self.get_or_load_ast(&canonical)?;

        // Collect all transitively imported files
        let mut to_visit: Vec<PathBuf> = vec![canonical.clone()];
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut result: Vec<(PathBuf, Program)> = Vec::new();

        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Ensure this file is parsed
            if let Err(e) = self.get_or_load_ast(&current) {
                // Log but don't fail - partial loading is OK
                eprintln!("Warning: {}", e);
                continue;
            }

            if let Some(doc) = self.documents.get(&current) {
                if let Some(ref program) = doc.program {
                    // Add imports to visit queue (iterate over HashSet)
                    for import_path in doc.imports.iter() {
                        if !visited.contains(import_path) {
                            to_visit.push(import_path.clone());
                        }
                    }

                    result.push((current.clone(), program.clone()));
                }
            }
        }

        // Reverse to get dependency order (leaves first)
        result.reverse();
        Ok(result)
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
#[allow(clippy::arc_with_non_send_sync)] // Context is used single-threaded for now
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

    #[test]
    fn test_imports_tracking() {
        let mut ctx = KleisContext::new();

        // Create a file with an import statement
        // Note: The import won't resolve (file doesn't exist), but it will be parsed
        let source = r#"
import "helper.kleis"
define x = 1
"#;
        ctx.set_document_content(PathBuf::from("/test/main.kleis"), source.to_string());

        let doc = ctx
            .get_document(&PathBuf::from("/test/main.kleis"))
            .unwrap();
        assert!(doc.program.is_some());
        // Import is extracted even if file doesn't exist (just won't resolve)
        // The imports set will be empty since the file doesn't exist on disk
        // This is correct behavior - we only track resolved imports
    }

    #[test]
    fn test_dirty_flag() {
        let mut ctx = KleisContext::new();
        let path = PathBuf::from("test.kleis");

        // Initial document is not dirty
        ctx.set_document_content(path.clone(), "define x = 1".to_string());
        assert!(!ctx.get_document(&path).unwrap().dirty);

        // Manually mark as dirty
        if let Some(doc) = ctx.get_document_mut(&path) {
            doc.dirty = true;
        }
        assert!(ctx.get_document(&path).unwrap().dirty);

        // Re-setting content clears dirty flag
        ctx.set_document_content(path.clone(), "define x = 2".to_string());
        assert!(!ctx.get_document(&path).unwrap().dirty);
    }

    #[test]
    fn test_cascade_invalidation() {
        let mut ctx = KleisContext::new();

        // Create two documents where A imports B
        let path_a = PathBuf::from("/test/a.kleis");
        let path_b = PathBuf::from("/test/b.kleis");

        ctx.set_document_content(path_b.clone(), "define helper = 1".to_string());

        // Manually set up the import relationship (since files don't exist on disk)
        ctx.set_document_content(path_a.clone(), "define main = 2".to_string());
        if let Some(doc_a) = ctx.get_document_mut(&path_a) {
            doc_a.imports.insert(path_b.clone());
        }

        // Both should be clean
        assert!(!ctx.get_document(&path_a).unwrap().dirty);
        assert!(!ctx.get_document(&path_b).unwrap().dirty);

        // Now invalidate B (simulating an edit)
        ctx.invalidate_dependents(&path_b);

        // B should be dirty
        assert!(ctx.get_document(&path_b).unwrap().dirty);

        // A should also be dirty (because it imports B)
        assert!(ctx.get_document(&path_a).unwrap().dirty);
    }

    #[test]
    fn test_document_with_imports_hashset() {
        let mut ctx = KleisContext::new();
        let path = PathBuf::from("test.kleis");

        ctx.set_document_content(path.clone(), "define x = 1".to_string());

        let doc = ctx.get_document(&path).unwrap();
        // Imports should be an empty HashSet for a file with no imports
        assert!(doc.imports.is_empty());
    }
}
