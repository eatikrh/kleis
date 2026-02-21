//! Theory Engine — session file management and evaluator lifecycle
//!
//! The TheoryEngine wraps a Kleis Evaluator with file-based state management.
//! The session file (`session.kleis`) is the source of truth; the evaluator
//! is a disposable view rebuilt from files on each commit.

use crate::ast::Expression;
use crate::config::TheoryConfig;
use crate::evaluator::Evaluator;
use crate::kleis_ast::{Program, StructureMember, TopLevel};
use crate::kleis_parser::{parse_kleis_program, parse_kleis_program_with_file, KleisParser};
use crate::pretty_print::PrettyPrinter;
use crate::solvers::backend::Witness;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Result of evaluating a Kleis expression via the theory MCP.
#[derive(Debug, Clone)]
pub struct EvalResult {
    pub value: Option<String>,
    pub verified: Option<bool>,
    pub witness: Option<Witness>,
    pub error: Option<String>,
}

/// Result of submitting Kleis code to the theory.
#[derive(Debug, Clone)]
pub struct SubmitResult {
    pub accepted: bool,
    pub structures_added: Vec<String>,
    pub functions_added: Vec<String>,
    pub data_types_added: Vec<String>,
    pub error: Option<String>,
}

/// The Kleis Theory Engine
///
/// Manages an evolving theory session where agents can submit structures,
/// definitions, and data types. The file system is the source of truth;
/// the evaluator is rebuilt from files on each commit.
pub struct TheoryEngine {
    evaluator: Evaluator,
    session_file: PathBuf,
    scratch_file: PathBuf,
    save_dir: PathBuf,
    session_history: Vec<String>,
}

impl TheoryEngine {
    /// Create a new theory engine with default prelude loaded.
    pub fn new(config: &TheoryConfig) -> Result<Self, String> {
        let workspace_dir = PathBuf::from(&config.workspace_dir);
        let save_dir = PathBuf::from(&config.save_dir);

        std::fs::create_dir_all(&workspace_dir).map_err(|e| {
            format!(
                "Cannot create workspace dir '{}': {}",
                workspace_dir.display(),
                e
            )
        })?;
        std::fs::create_dir_all(&save_dir)
            .map_err(|e| format!("Cannot create save dir '{}': {}", save_dir.display(), e))?;

        let session_file = workspace_dir.join("session.kleis");
        let scratch_file = workspace_dir.join("scratch.kleis");

        let initial_content = "import \"stdlib/prelude.kleis\"\n";
        std::fs::write(&session_file, initial_content).map_err(|e| {
            format!(
                "Cannot write session file '{}': {}",
                session_file.display(),
                e
            )
        })?;

        let mut engine = Self {
            evaluator: Evaluator::new(),
            session_file,
            scratch_file,
            save_dir,
            session_history: Vec::new(),
        };

        engine.rebuild_evaluator()?;

        eprintln!(
            "[kleis-theory] Session started with prelude ({} functions loaded)",
            engine.evaluator.list_functions().len()
        );

        Ok(engine)
    }

    /// Rebuild the evaluator from the session file.
    ///
    /// Drops the current evaluator, creates a fresh one, parses the session
    /// file (which transitively loads all imports), and loads everything.
    fn rebuild_evaluator(&mut self) -> Result<(), String> {
        self.rebuild_evaluator_from(&self.session_file.clone())
    }

    /// Rebuild the evaluator from an arbitrary file (session or scratch).
    fn rebuild_evaluator_from(&mut self, file_path: &Path) -> Result<(), String> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Cannot read '{}': {}", file_path.display(), e))?;

        let file_path_str = file_path.to_string_lossy().to_string();
        let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_kleis_program_with_file(&source, &file_path_str)
        }));
        let program = match parse_result {
            Ok(Ok(prog)) => prog,
            Ok(Err(e)) => return Err(format!("Parse error: {}", e.message)),
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic".to_string()
                };
                return Err(format!("Parser panic: {}", msg));
            }
        };

        let canonical = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());

        // Z3 axiom verification can panic on unsupported operations (e.g.,
        // transcendental functions causing sort mismatches in Z3_mk_app).
        // Wrap the entire load path so the MCP server stays alive.
        let load_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut evaluator = Evaluator::new();
            let mut loaded_files = HashSet::new();
            loaded_files.insert(canonical.clone());
            load_imports_recursive(&program, &canonical, &mut evaluator, &mut loaded_files)?;
            evaluator.load_program_with_file(&program, Some(canonical))?;
            Ok(evaluator)
        }));

        match load_result {
            Ok(Ok(evaluator)) => {
                self.evaluator = evaluator;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic in Z3 backend".to_string()
                };
                Err(format!(
                    "Z3 backend error (axiom verification failed): {}",
                    msg
                ))
            }
        }
    }

    /// Submit Kleis source code to the theory (structure, define, or data).
    ///
    /// Appends the source to a scratch copy, rebuilds an evaluator from it,
    /// and if successful, updates the session file and keeps the new evaluator.
    pub fn submit_kleis(&mut self, kleis_source: &str) -> SubmitResult {
        let current_content = match std::fs::read_to_string(&self.session_file) {
            Ok(c) => c,
            Err(e) => {
                return SubmitResult {
                    accepted: false,
                    structures_added: vec![],
                    functions_added: vec![],
                    data_types_added: vec![],
                    error: Some(format!("Cannot read session file: {}", e)),
                };
            }
        };

        let scratch_content = format!("{}\n{}\n", current_content, kleis_source);

        if let Err(e) = std::fs::write(&self.scratch_file, &scratch_content) {
            return SubmitResult {
                accepted: false,
                structures_added: vec![],
                functions_added: vec![],
                data_types_added: vec![],
                error: Some(format!("Cannot write scratch file: {}", e)),
            };
        }

        // Clone scratch path to avoid borrow conflict with &mut self
        let scratch = self.scratch_file.clone();

        // Stage 1+2+3: Parse, load, verify on scratch evaluator
        match self.rebuild_evaluator_from(&scratch) {
            Ok(()) => {
                // Success — commit: update session file
                if let Err(e) = std::fs::write(&self.session_file, &scratch_content) {
                    return SubmitResult {
                        accepted: false,
                        structures_added: vec![],
                        functions_added: vec![],
                        data_types_added: vec![],
                        error: Some(format!("Cannot update session file: {}", e)),
                    };
                }

                let added = extract_names_from_source(kleis_source);
                self.session_history.push(kleis_source.to_string());

                let _ = std::fs::remove_file(&self.scratch_file);

                SubmitResult {
                    accepted: true,
                    structures_added: added.structures,
                    functions_added: added.functions,
                    data_types_added: added.data_types,
                    error: None,
                }
            }
            Err(e) => {
                // Failed — rollback: restore evaluator from session file
                let _ = self.rebuild_evaluator();
                let _ = std::fs::remove_file(&self.scratch_file);

                SubmitResult {
                    accepted: false,
                    structures_added: vec![],
                    functions_added: vec![],
                    data_types_added: vec![],
                    error: Some(e),
                }
            }
        }
    }

    /// Try submitting Kleis source without committing (dry run).
    ///
    /// Checks whether the source would be accepted without modifying
    /// the session file or the live evaluator.
    pub fn try_kleis(&self, kleis_source: &str) -> SubmitResult {
        let current_content = match std::fs::read_to_string(&self.session_file) {
            Ok(c) => c,
            Err(e) => {
                return SubmitResult {
                    accepted: false,
                    structures_added: vec![],
                    functions_added: vec![],
                    data_types_added: vec![],
                    error: Some(format!("Cannot read session file: {}", e)),
                };
            }
        };

        let scratch_content = format!("{}\n{}\n", current_content, kleis_source);

        if let Err(e) = std::fs::write(&self.scratch_file, &scratch_content) {
            return SubmitResult {
                accepted: false,
                structures_added: vec![],
                functions_added: vec![],
                data_types_added: vec![],
                error: Some(format!("Cannot write scratch file: {}", e)),
            };
        }

        let source = match std::fs::read_to_string(&self.scratch_file) {
            Ok(s) => s,
            Err(e) => {
                let _ = std::fs::remove_file(&self.scratch_file);
                return SubmitResult {
                    accepted: false,
                    structures_added: vec![],
                    functions_added: vec![],
                    data_types_added: vec![],
                    error: Some(format!("Cannot read scratch file: {}", e)),
                };
            }
        };

        let file_path_str = self.scratch_file.to_string_lossy().to_string();
        let result = match parse_kleis_program_with_file(&source, &file_path_str) {
            Ok(program) => {
                let mut eval = Evaluator::new();
                let mut loaded = HashSet::new();
                let canonical = self
                    .scratch_file
                    .canonicalize()
                    .unwrap_or_else(|_| self.scratch_file.clone());
                loaded.insert(canonical.clone());

                match load_imports_recursive(&program, &canonical, &mut eval, &mut loaded) {
                    Ok(()) => match eval.load_program_with_file(&program, Some(canonical)) {
                        Ok(()) => {
                            let added = extract_names_from_source(kleis_source);
                            SubmitResult {
                                accepted: true,
                                structures_added: added.structures,
                                functions_added: added.functions,
                                data_types_added: added.data_types,
                                error: None,
                            }
                        }
                        Err(e) => SubmitResult {
                            accepted: false,
                            structures_added: vec![],
                            functions_added: vec![],
                            data_types_added: vec![],
                            error: Some(format!("Load error: {}", e)),
                        },
                    },
                    Err(e) => SubmitResult {
                        accepted: false,
                        structures_added: vec![],
                        functions_added: vec![],
                        data_types_added: vec![],
                        error: Some(e),
                    },
                }
            }
            Err(e) => SubmitResult {
                accepted: false,
                structures_added: vec![],
                functions_added: vec![],
                data_types_added: vec![],
                error: Some(format!("Parse error: {}", e.message)),
            },
        };

        let _ = std::fs::remove_file(&self.scratch_file);
        result
    }

    /// Evaluate a Kleis expression or verify a proposition.
    pub fn evaluate_expression(&self, expr_str: &str) -> EvalResult {
        let mut parser = KleisParser::new(expr_str);
        let expr = match parser.parse_proposition() {
            Ok(e) => e,
            Err(e) => {
                return EvalResult {
                    value: None,
                    verified: None,
                    witness: None,
                    error: Some(format!("Parse error: {}", e.message)),
                };
            }
        };

        if is_proposition(&expr) {
            let verify_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.evaluator.verify_proposition(&expr)
            }));
            return match verify_result {
                Ok(r) => proposition_result(&r),
                Err(panic_info) => {
                    let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "Unknown panic in Z3 backend".to_string()
                    };
                    EvalResult {
                        value: None,
                        verified: None,
                        witness: None,
                        error: Some(format!("Z3 backend error: {}", msg)),
                    }
                }
            };
        }

        match self.evaluator.eval_concrete(&expr) {
            Ok(result) => EvalResult {
                value: Some(expression_to_string(&result)),
                verified: None,
                witness: None,
                error: None,
            },
            Err(e) => EvalResult {
                value: None,
                verified: None,
                witness: None,
                error: Some(format!("Evaluation error: {}", e)),
            },
        }
    }

    /// Describe the full loaded schema: structures, data types, functions, axioms.
    pub fn describe_schema(&self) -> Value {
        let pp = PrettyPrinter::new();

        let structures: Vec<Value> = self
            .evaluator
            .get_structures()
            .iter()
            .map(|s| {
                let axioms: Vec<Value> = s
                    .members
                    .iter()
                    .filter_map(|m| {
                        if let StructureMember::Axiom { name, proposition } = m {
                            Some(serde_json::json!({
                                "name": name,
                                "kleis": pp.format_expression(proposition),
                            }))
                        } else {
                            None
                        }
                    })
                    .collect();

                let operations: Vec<Value> = s
                    .members
                    .iter()
                    .filter_map(|m| {
                        if let StructureMember::Operation {
                            name,
                            type_signature,
                        } = m
                        {
                            Some(serde_json::json!({
                                "name": name,
                                "type": format!("{}", type_signature),
                            }))
                        } else {
                            None
                        }
                    })
                    .collect();

                let fields: Vec<Value> = s
                    .members
                    .iter()
                    .filter_map(|m| {
                        if let StructureMember::Field { name, type_expr } = m {
                            Some(serde_json::json!({
                                "name": name,
                                "type": format!("{}", type_expr),
                            }))
                        } else {
                            None
                        }
                    })
                    .collect();

                let type_params: Vec<String> = s
                    .type_params
                    .iter()
                    .map(|tp| {
                        if let Some(ref kind) = tp.kind {
                            format!("{} : {}", tp.name, kind)
                        } else {
                            tp.name.clone()
                        }
                    })
                    .collect();

                serde_json::json!({
                    "name": s.name,
                    "type_params": type_params,
                    "axioms": axioms,
                    "operations": operations,
                    "fields": fields,
                    "extends": s.extends_clause.as_ref().map(|e| format!("{}", e)),
                })
            })
            .collect();

        let data_types: Vec<Value> = self
            .evaluator
            .get_data_types()
            .iter()
            .map(|d| {
                let variants: Vec<Value> = d
                    .variants
                    .iter()
                    .map(|v| {
                        let fields: Vec<String> = v
                            .fields
                            .iter()
                            .map(|f| {
                                if let Some(ref name) = f.name {
                                    format!("{}: {}", name, f.type_expr)
                                } else {
                                    format!("{}", f.type_expr)
                                }
                            })
                            .collect();

                        if fields.is_empty() {
                            serde_json::json!({ "name": v.name })
                        } else {
                            serde_json::json!({ "name": v.name, "fields": fields })
                        }
                    })
                    .collect();

                let type_params: Vec<String> = d
                    .type_params
                    .iter()
                    .map(|tp| {
                        if let Some(ref kind) = tp.kind {
                            format!("{} : {}", tp.name, kind)
                        } else {
                            tp.name.clone()
                        }
                    })
                    .collect();

                serde_json::json!({
                    "name": d.name,
                    "type_params": type_params,
                    "variants": variants,
                })
            })
            .collect();

        let functions: Vec<Value> = self
            .evaluator
            .list_functions()
            .iter()
            .map(|name| {
                if let Some(closure) = self.evaluator.get_function(name) {
                    serde_json::json!({
                        "name": name,
                        "params": closure.params,
                        "kleis": pp.format_function(name, closure),
                    })
                } else {
                    serde_json::json!({ "name": name })
                }
            })
            .collect();

        let mut stats = HashMap::new();
        stats.insert("structures", structures.len());
        stats.insert("data_types", data_types.len());
        stats.insert("functions", functions.len());
        let axiom_count: usize = self
            .evaluator
            .get_structures()
            .iter()
            .map(|s| {
                s.members
                    .iter()
                    .filter(|m| matches!(m, StructureMember::Axiom { .. }))
                    .count()
            })
            .sum();
        stats.insert("axioms", axiom_count);

        serde_json::json!({
            "session_file": self.session_file.to_string_lossy(),
            "structures": structures,
            "data_types": data_types,
            "functions": functions,
            "session_history_count": self.session_history.len(),
            "stats": stats,
        })
    }

    /// Load a new theory session with the specified imports.
    ///
    /// Replaces the current session entirely: writes a new session.kleis
    /// with the specified imports, drops the old evaluator, rebuilds.
    pub fn load_theory(&mut self, imports: Vec<String>) -> Result<(), String> {
        let mut content = String::new();
        for import_path in &imports {
            content.push_str(&format!("import \"{}\"\n", import_path));
        }

        std::fs::write(&self.session_file, &content).map_err(|e| {
            format!(
                "Cannot write session file '{}': {}",
                self.session_file.display(),
                e
            )
        })?;

        self.session_history.clear();
        self.rebuild_evaluator()?;

        eprintln!(
            "[kleis-theory] New session with {} imports ({} functions loaded)",
            imports.len(),
            self.evaluator.list_functions().len()
        );

        Ok(())
    }

    /// Save the current session to a named file in the save directory.
    pub fn save_theory(&self, name: &str) -> Result<PathBuf, String> {
        let save_name = if name.ends_with(".kleis") {
            name.to_string()
        } else {
            format!("{}.kleis", name)
        };
        let save_path = self.save_dir.join(&save_name);

        let content = std::fs::read_to_string(&self.session_file)
            .map_err(|e| format!("Cannot read session file: {}", e))?;

        std::fs::write(&save_path, &content)
            .map_err(|e| format!("Cannot write '{}': {}", save_path.display(), e))?;

        eprintln!("[kleis-theory] Theory saved to {}", save_path.display());

        Ok(save_path)
    }

    /// Return the session history (what the agent has submitted, in order).
    pub fn list_session(&self) -> &[String] {
        &self.session_history
    }

    /// Get summary stats.
    pub fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert(
            "functions".to_string(),
            self.evaluator.list_functions().len(),
        );
        stats.insert(
            "structures".to_string(),
            self.evaluator.get_structures().len(),
        );
        stats.insert(
            "data_types".to_string(),
            self.evaluator.get_data_types().len(),
        );
        stats.insert("session_items".to_string(), self.session_history.len());
        stats
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Recursively load imports for a program (adapted from src/bin/kleis.rs).
fn load_imports_recursive(
    program: &Program,
    file_path: &Path,
    evaluator: &mut Evaluator,
    loaded_files: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    let base_dir = file_path.parent().unwrap_or(Path::new("."));

    for item in &program.items {
        if let TopLevel::Import(import_path_str) = item {
            let import_path = Path::new(import_path_str);
            let resolved = if import_path.is_absolute() {
                import_path.to_path_buf()
            } else if import_path_str.starts_with("stdlib/") {
                if let Ok(kleis_root) = std::env::var("KLEIS_ROOT") {
                    let candidate = PathBuf::from(&kleis_root).join(import_path_str);
                    if candidate.exists() {
                        candidate
                    } else {
                        PathBuf::from(import_path_str)
                    }
                } else {
                    PathBuf::from(import_path_str)
                }
            } else {
                base_dir.join(import_path)
            };

            let canonical = resolved
                .canonicalize()
                .map_err(|e| format!("Cannot resolve import '{}': {}", import_path_str, e))?;

            if loaded_files.contains(&canonical) {
                continue;
            }
            loaded_files.insert(canonical.clone());

            let source = std::fs::read_to_string(&canonical)
                .map_err(|e| format!("Cannot read import '{}': {}", import_path_str, e))?;
            let fpath_str = canonical.to_string_lossy().to_string();
            let import_program = parse_kleis_program_with_file(&source, &fpath_str)
                .map_err(|e| format!("Parse error in '{}': {}", import_path_str, e))?;

            load_imports_recursive(&import_program, &canonical, evaluator, loaded_files)?;
            evaluator.load_program_with_file(&import_program, Some(canonical))?;
        }
    }

    Ok(())
}

/// Check if an expression looks like a proposition.
fn is_proposition(expr: &Expression) -> bool {
    match expr {
        Expression::Quantifier { .. } => true,
        Expression::Operation { name, .. } if name == "implies" || name == "→" || name == "⟹" => {
            true
        }
        Expression::Operation { name, .. }
            if name == "and"
                || name == "or"
                || name == "∧"
                || name == "∨"
                || name == "not"
                || name == "¬"
                || name == "iff"
                || name == "⟺" =>
        {
            true
        }
        _ => false,
    }
}

/// Convert an AssertResult into an EvalResult.
fn proposition_result(assert_result: &crate::evaluator::AssertResult) -> EvalResult {
    use crate::evaluator::AssertResult;
    match assert_result {
        AssertResult::Passed => EvalResult {
            value: Some("true".to_string()),
            verified: Some(true),
            witness: None,
            error: None,
        },
        AssertResult::Verified { witness } => EvalResult {
            value: Some("true (proved by Z3)".to_string()),
            verified: Some(true),
            witness: witness.clone(),
            error: None,
        },
        AssertResult::Failed {
            expected: _,
            actual,
        } => EvalResult {
            value: Some(format!("false (got {})", expression_to_string(actual))),
            verified: Some(false),
            witness: None,
            error: None,
        },
        AssertResult::Disproved { witness } => EvalResult {
            value: Some("false (disproved by Z3)".to_string()),
            verified: Some(false),
            witness: Some(witness.clone()),
            error: None,
        },
        AssertResult::Unknown(msg) => EvalResult {
            value: None,
            verified: None,
            witness: None,
            error: Some(format!("Unknown: {}", msg)),
        },
    }
}

/// Convert an Expression to a human-readable string.
fn expression_to_string(expr: &Expression) -> String {
    match expr {
        Expression::String(s) => s.clone(),
        Expression::Const(s) => s.clone(),
        Expression::Object(s) => s.clone(),
        Expression::List(items) => {
            let strs: Vec<String> = items.iter().map(expression_to_string).collect();
            format!("[{}]", strs.join(", "))
        }
        other => format!("{:?}", other),
    }
}

struct ExtractedNames {
    structures: Vec<String>,
    functions: Vec<String>,
    data_types: Vec<String>,
}

/// Extract structure/function/data type names from submitted Kleis source.
fn extract_names_from_source(kleis_source: &str) -> ExtractedNames {
    let mut names = ExtractedNames {
        structures: vec![],
        functions: vec![],
        data_types: vec![],
    };

    if let Ok(program) = parse_kleis_program(kleis_source) {
        for item in &program.items {
            match item {
                TopLevel::StructureDef(s) => names.structures.push(s.name.clone()),
                TopLevel::FunctionDef(f) => names.functions.push(f.name.clone()),
                TopLevel::DataDef(d) => names.data_types.push(d.name.clone()),
                _ => {}
            }
        }
    }

    names
}
