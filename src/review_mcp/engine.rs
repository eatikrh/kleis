//! Review Engine — loads a coding standards policy and checks source code
//!
//! Wraps the Kleis evaluator with code-review-specific logic.
//! The policy file defines `check_*` functions that accept source code
//! as a string and return "pass" or "fail: <reason>".

use crate::ast::Expression;
use crate::evaluator::{AssertResult, Evaluator};
use crate::kleis_ast::{Program, StructureMember, TopLevel};
use crate::kleis_parser::{parse_kleis_program_with_file, KleisParser};
use crate::pretty_print::PrettyPrinter;
use crate::solvers::backend::Witness;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Result of checking code against a single rule
#[derive(Debug, Clone)]
pub struct RuleVerdict {
    pub rule_name: String,
    pub passed: bool,
    pub message: String,
}

/// Result of checking code against all rules
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub passed: bool,
    pub verdicts: Vec<RuleVerdict>,
    pub summary: String,
}

/// A loaded coding standard rule
#[derive(Debug, Clone)]
pub struct ReviewRule {
    pub name: String,
    pub kind: ReviewRuleKind,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum ReviewRuleKind {
    CheckFunction,
    Axiom,
    Helper,
}

/// Result of evaluating a Kleis expression
#[derive(Debug, Clone)]
pub struct EvalResult {
    pub value: Option<String>,
    pub verified: Option<bool>,
    pub witness: Option<Witness>,
    pub error: Option<String>,
}

/// The Kleis Review Engine
///
/// Loads a coding standards policy and checks source code against it.
/// Each `check_*` function in the policy receives source code as a string
/// and returns "pass" or "fail: <reason>".
pub struct ReviewEngine {
    evaluator: Evaluator,
    rules: Vec<ReviewRule>,
    policy_file: PathBuf,
}

impl ReviewEngine {
    /// Load a review policy from a `.kleis` file
    pub fn load(policy_path: &PathBuf) -> Result<Self, String> {
        let source = std::fs::read_to_string(policy_path).map_err(|e| {
            format!(
                "Cannot read review policy '{}': {}",
                policy_path.display(),
                e
            )
        })?;

        let canonical = policy_path.canonicalize().map_err(|e| {
            format!(
                "Cannot resolve policy path '{}': {}",
                policy_path.display(),
                e
            )
        })?;
        let file_path_str = canonical.to_string_lossy().to_string();

        let program = parse_kleis_program_with_file(&source, &file_path_str)
            .map_err(|e| format!("Review policy parse error: {}", e.message))?;

        let mut evaluator = Evaluator::new();
        let mut loaded_files = HashSet::new();
        loaded_files.insert(canonical.clone());
        load_imports_recursive(&program, &canonical, &mut evaluator, &mut loaded_files)?;
        evaluator.load_program_with_file(&program, Some(canonical))?;

        let rules = Self::extract_rules(&program);

        eprintln!(
            "[kleis-review] Loaded policy: {} ({} rules, {} functions)",
            policy_path.display(),
            rules.len(),
            evaluator.list_functions().len()
        );

        Ok(Self {
            evaluator,
            rules,
            policy_file: policy_path.clone(),
        })
    }

    fn extract_rules(program: &Program) -> Vec<ReviewRule> {
        let mut rules = Vec::new();

        for item in &program.items {
            match item {
                TopLevel::FunctionDef(func) => {
                    let kind = if func.name.starts_with("check_") {
                        ReviewRuleKind::CheckFunction
                    } else {
                        ReviewRuleKind::Helper
                    };
                    rules.push(ReviewRule {
                        name: func.name.clone(),
                        kind,
                        description: format!(
                            "define {}({}) = ...",
                            func.name,
                            func.params.join(", ")
                        ),
                    });
                }
                TopLevel::StructureDef(structure) => {
                    for member in &structure.members {
                        if let StructureMember::Axiom { name, .. } = member {
                            rules.push(ReviewRule {
                                name: name.clone(),
                                kind: ReviewRuleKind::Axiom,
                                description: format!(
                                    "axiom {} (in structure {})",
                                    name, structure.name
                                ),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        rules
    }

    /// Check source code against all `check_*` rules in the policy.
    ///
    /// Each `check_*` function is called with the source code string.
    /// Returns "pass" or "fail: <reason>".
    pub fn check_code(&self, source: &str, _language: &str) -> ReviewResult {
        let check_functions: Vec<String> = self
            .evaluator
            .list_functions()
            .into_iter()
            .filter(|name| name.starts_with("check_"))
            .collect();

        let mut verdicts = Vec::new();
        let mut all_passed = true;

        for func_name in &check_functions {
            let call_expr = Expression::Operation {
                name: func_name.clone(),
                args: vec![Expression::String(source.to_string())],
                span: None,
            };

            let verdict = match self.evaluator.eval_concrete(&call_expr) {
                Ok(result) => {
                    let result_str = Self::expression_to_string(&result);
                    let passed = Self::is_pass(&result);

                    if !passed {
                        all_passed = false;
                    }

                    RuleVerdict {
                        rule_name: func_name.clone(),
                        passed,
                        message: result_str,
                    }
                }
                Err(e) => {
                    all_passed = false;
                    RuleVerdict {
                        rule_name: func_name.clone(),
                        passed: false,
                        message: format!("error: {}", e),
                    }
                }
            };

            verdicts.push(verdict);
        }

        let pass_count = verdicts.iter().filter(|v| v.passed).count();
        let fail_count = verdicts.len() - pass_count;

        let summary = if all_passed {
            format!("All {} checks passed", verdicts.len())
        } else {
            format!(
                "{} passed, {} failed (out of {} checks)",
                pass_count,
                fail_count,
                verdicts.len()
            )
        };

        ReviewResult {
            passed: all_passed,
            verdicts,
            summary,
        }
    }

    /// Check a file on disk against all `check_*` rules.
    ///
    /// Validates the path (non-empty, exists, is a file, is readable text)
    /// before delegating to `check_code`.
    pub fn check_file(&self, path: &str, language: &str) -> Result<ReviewResult, String> {
        if path.is_empty() {
            return Err("'path' argument is required but was empty".to_string());
        }

        let file_path = Path::new(path);

        if !file_path.exists() {
            return Err(format!("File not found: {}", path));
        }

        if file_path.is_dir() {
            return Err(format!("'{}' is a directory, not a file", path));
        }

        let source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Cannot read file '{}': {}", path, e))?;

        Ok(self.check_code(&source, language))
    }

    fn is_pass(result: &Expression) -> bool {
        match result {
            Expression::String(s) => {
                let lower = s.to_lowercase();
                lower == "pass" || lower == "ok" || lower == "true"
            }
            Expression::Const(val) => val == "1" || val.to_lowercase() == "true",
            Expression::Object(name) => {
                let lower = name.to_lowercase();
                lower == "pass" || lower == "true" || lower == "ok"
            }
            _ => false,
        }
    }

    /// List all review rules
    pub fn list_rules(&self) -> &[ReviewRule] {
        &self.rules
    }

    /// Explain a specific rule
    pub fn explain_rule(&self, rule_name: &str) -> Option<&ReviewRule> {
        self.rules.iter().find(|r| r.name == rule_name)
    }

    /// Get the policy file path
    pub fn policy_file(&self) -> &PathBuf {
        &self.policy_file
    }

    /// Get summary stats
    pub fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_rules".to_string(), self.rules.len());
        stats.insert(
            "check_functions".to_string(),
            self.rules
                .iter()
                .filter(|r| matches!(r.kind, ReviewRuleKind::CheckFunction))
                .count(),
        );
        stats.insert(
            "axioms".to_string(),
            self.rules
                .iter()
                .filter(|r| matches!(r.kind, ReviewRuleKind::Axiom))
                .count(),
        );
        stats.insert(
            "functions".to_string(),
            self.evaluator.list_functions().len(),
        );
        stats
    }

    /// Describe the full schema of loaded standards
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

                serde_json::json!({
                    "name": s.name,
                    "axioms": axioms,
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
                        "body": pp.format_expression(&closure.body),
                        "kleis": pp.format_function(name, closure),
                    })
                } else {
                    serde_json::json!({ "name": name })
                }
            })
            .collect();

        let check_fns: Vec<&Value> = functions
            .iter()
            .filter(|f| {
                f.get("name")
                    .and_then(|n| n.as_str())
                    .is_some_and(|n| n.starts_with("check_"))
            })
            .collect();

        let helper_fns: Vec<&Value> = functions
            .iter()
            .filter(|f| {
                f.get("name")
                    .and_then(|n| n.as_str())
                    .is_some_and(|n| !n.starts_with("check_"))
            })
            .collect();

        serde_json::json!({
            "policy_file": self.policy_file.display().to_string(),
            "structures": structures,
            "check_functions": check_fns,
            "helper_functions": helper_fns,
            "stats": {
                "structures": structures.len(),
                "functions": functions.len(),
                "check_functions": check_fns.len(),
            }
        })
    }

    /// Evaluate an arbitrary Kleis expression in the context of the review policy
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

        if Self::is_proposition(&expr) {
            return Self::proposition_result(&self.evaluator.verify_proposition(&expr));
        }

        match self.evaluator.eval_concrete(&expr) {
            Ok(result) => {
                if let Expression::Operation { ref name, .. } = result {
                    if let Some(structure_name) = self.find_owner_structure(name) {
                        if let Some(z3_result) = self
                            .evaluator
                            .verify_structure_operation(&expr, &structure_name)
                        {
                            return Self::proposition_result(&z3_result);
                        }
                        return EvalResult {
                            value: None,
                            verified: None,
                            witness: None,
                            error: Some(
                                "Z3 unavailable for structure operation verification".to_string(),
                            ),
                        };
                    }
                }
                EvalResult {
                    value: Some(Self::expression_to_string(&result)),
                    verified: None,
                    witness: None,
                    error: None,
                }
            }
            Err(e) => EvalResult {
                value: None,
                verified: None,
                witness: None,
                error: Some(format!("Evaluation error: {}", e)),
            },
        }
    }

    fn find_owner_structure(&self, name: &str) -> Option<String> {
        for structure in self.evaluator.get_structures() {
            for member in &structure.members {
                if let crate::kleis_ast::StructureMember::Operation { name: op_name, .. } = member {
                    if op_name == name {
                        return Some(structure.name.clone());
                    }
                }
            }
        }
        None
    }

    fn is_proposition(expr: &Expression) -> bool {
        match expr {
            Expression::Quantifier { .. } => true,
            Expression::Operation { name, .. }
                if name == "implies"
                    || name == "→"
                    || name == "⟹"
                    || name == "and"
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

    fn proposition_result(assert_result: &AssertResult) -> EvalResult {
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
            AssertResult::Failed { actual, .. } => EvalResult {
                value: Some(format!(
                    "false (got {})",
                    Self::expression_to_string(actual)
                )),
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
            AssertResult::InconsistentAxioms => EvalResult {
                value: None,
                verified: Some(false),
                witness: None,
                error: Some("AXIOM INCONSISTENCY: loaded axioms are contradictory".to_string()),
            },
        }
    }

    fn expression_to_string(expr: &Expression) -> String {
        match expr {
            Expression::String(s) => s.clone(),
            Expression::Const(s) => s.clone(),
            Expression::Object(s) => s.clone(),
            Expression::List(items) => {
                let strs: Vec<String> = items.iter().map(Self::expression_to_string).collect();
                format!("[{}]", strs.join(", "))
            }
            other => format!("{:?}", other),
        }
    }
}

/// Recursively load imports for a program.
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
                PathBuf::from(import_path_str)
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
            let fp = canonical.to_string_lossy().to_string();
            let import_program = parse_kleis_program_with_file(&source, &fp)
                .map_err(|e| format!("Parse error in '{}': {}", import_path_str, e))?;

            load_imports_recursive(&import_program, &canonical, evaluator, loaded_files)?;
            evaluator.load_program_with_file(&import_program, Some(canonical))?;
        }
    }

    Ok(())
}
