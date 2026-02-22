//! Kleis Policy Engine for MCP
//!
//! Loads a `.kleis` policy file and evaluates agent actions against it.
//! Exposes the full Kleis reasoning engine to MCP clients:
//!
//! - **Policy checks**: `check_*` functions return "allow"/"deny"
//! - **Preconditions**: `before_*` functions return required pre-steps
//! - **Schema introspection**: structures, data types, axioms, functions
//! - **Expression evaluation & verification**: evaluate any Kleis expression;
//!   propositions (∀, ∃, →) are automatically routed through the evaluator's
//!   `assert()` pipeline which uses Z3 when needed — no separate verification tool
//!
//! ## Policy File Convention
//!
//! ```kleis
//! define check_file_delete(path) = ...
//! define check_run_command(cmd) = ...
//! define before_git_push(branch, force) = "cargo test"
//! ```
//!
//! The MCP server maps action types to these functions and evaluates them.

use crate::ast::Expression;
use crate::evaluator::{AssertResult, Evaluator};
use crate::kleis_ast::{Program, StructureMember, TopLevel};
use crate::kleis_parser::{parse_kleis_program, KleisParser};
use crate::pretty_print::PrettyPrinter;
use crate::solvers::backend::Witness;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Policy decision
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub rule_name: Option<String>,
    pub reason: String,
    /// Preconditions: steps the agent should perform before the action.
    /// Evaluated from `before_*` functions in the policy. Empty if none.
    pub preconditions: Vec<String>,
}

/// A loaded policy rule (for listing/explaining)
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub name: String,
    pub kind: RuleKind,
    /// The source text of the rule (for explain_rule)
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum RuleKind {
    CheckFunction,
    Precondition,
    Axiom,
    Define,
}

/// The Kleis Policy Engine
///
/// Wraps the Kleis evaluator and policy metadata.
/// Provides two tiers of capability:
///
/// 1. **Policy checks** — `check_action()` for allow/deny decisions
/// 2. **Expression evaluation** — `evaluate_expression()` for any Kleis expression
///    or proposition (Z3 verification handled by the evaluator's assert pipeline)
pub struct PolicyEngine {
    evaluator: Evaluator,
    rules: Vec<PolicyRule>,
    policy_file: PathBuf,
}

impl PolicyEngine {
    /// Load a policy from a `.kleis` file
    pub fn load(policy_path: &PathBuf) -> Result<Self, String> {
        let source = std::fs::read_to_string(policy_path)
            .map_err(|e| format!("Cannot read policy file '{}': {}", policy_path.display(), e))?;

        let program = parse_kleis_program(&source)
            .map_err(|e| format!("Policy parse error: {}", e.message))?;

        let mut evaluator = Evaluator::new();
        evaluator.load_program(&program)?;

        let rules = Self::extract_rules(&program);

        eprintln!(
            "[kleis-mcp] Loaded policy: {} ({} rules, {} functions)",
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

    /// Extract rule metadata from the parsed program
    fn extract_rules(program: &Program) -> Vec<PolicyRule> {
        let mut rules = Vec::new();

        for item in &program.items {
            match item {
                TopLevel::FunctionDef(func) => {
                    let kind = if func.name.starts_with("check_") {
                        RuleKind::CheckFunction
                    } else if func.name.starts_with("before_") {
                        RuleKind::Precondition
                    } else {
                        RuleKind::Define
                    };
                    rules.push(PolicyRule {
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
                    // Extract axioms from structures
                    for member in &structure.members {
                        if let StructureMember::Axiom { name, .. } = member {
                            rules.push(PolicyRule {
                                name: name.clone(),
                                kind: RuleKind::Axiom,
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

    /// Check whether an action is allowed by the policy.
    ///
    /// Evaluates the corresponding `check_*` function for allow/deny,
    /// and the `before_*` function for preconditions (if defined).
    pub fn check_action(&self, action: &Value) -> PolicyDecision {
        let action_type = action
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("unknown");

        // Map action type → (suffix, args)
        let (suffix, args) = match action_type {
            "file_delete" => {
                let path = action.get("path").and_then(|p| p.as_str()).unwrap_or("");
                (
                    "file_delete".to_string(),
                    vec![Expression::String(path.to_string())],
                )
            }
            "file_create" => {
                let path = action.get("path").and_then(|p| p.as_str()).unwrap_or("");
                (
                    "file_create".to_string(),
                    vec![Expression::String(path.to_string())],
                )
            }
            "file_edit" => {
                let path = action.get("path").and_then(|p| p.as_str()).unwrap_or("");
                (
                    "file_edit".to_string(),
                    vec![Expression::String(path.to_string())],
                )
            }
            "run_command" => {
                let cmd = action.get("command").and_then(|c| c.as_str()).unwrap_or("");
                (
                    "run_command".to_string(),
                    vec![Expression::String(cmd.to_string())],
                )
            }
            "git_push" => {
                let branch = action
                    .get("branch")
                    .and_then(|b| b.as_str())
                    .unwrap_or("main");
                let force = action
                    .get("force")
                    .map(|f| f.as_bool().unwrap_or_else(|| f.as_u64().unwrap_or(0) != 0))
                    .unwrap_or(false);
                (
                    "git_push".to_string(),
                    vec![
                        Expression::String(branch.to_string()),
                        Expression::Const(if force {
                            "1".to_string()
                        } else {
                            "0".to_string()
                        }),
                    ],
                )
            }
            "git_commit" => {
                let desc = action
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                (
                    "git_commit".to_string(),
                    vec![Expression::String(desc.to_string())],
                )
            }
            _ => {
                return PolicyDecision {
                    allowed: false,
                    rule_name: None,
                    reason: format!("Unknown action type: '{}'", action_type),
                    preconditions: vec![],
                };
            }
        };

        let check_name = format!("check_{}", suffix);
        let before_name = format!("before_{}", suffix);

        // Evaluate preconditions (before_* function)
        let preconditions = self.eval_preconditions(&before_name, &args);

        // Check if the policy defines the check function
        let available_functions = self.evaluator.list_functions();
        if !available_functions.contains(&check_name) {
            // No rule defined → default allow (open policy)
            return PolicyDecision {
                allowed: true,
                rule_name: None,
                reason: format!(
                    "No policy rule '{}' defined — action allowed by default",
                    check_name
                ),
                preconditions,
            };
        }

        // Build the function call expression
        let call_expr = Expression::Operation {
            name: check_name.clone(),
            args,
            span: None,
        };

        // Evaluate using eval_concrete for full reduction
        match self.evaluator.eval_concrete(&call_expr) {
            Ok(result) => {
                let allowed = Self::is_allowed(&result);

                PolicyDecision {
                    allowed,
                    rule_name: Some(check_name.clone()),
                    reason: if allowed {
                        format!("ALLOWED by {}", check_name)
                    } else {
                        format!("DENIED by {}", check_name)
                    },
                    preconditions,
                }
            }
            Err(e) => {
                // Evaluation error → deny (fail-closed)
                PolicyDecision {
                    allowed: false,
                    rule_name: Some(check_name.clone()),
                    reason: format!(
                        "DENIED — policy evaluation error in '{}': {}",
                        check_name, e
                    ),
                    preconditions,
                }
            }
        }
    }

    /// Evaluate the `before_*` function for an action, returning precondition
    /// steps. Returns an empty vec if no `before_*` function is defined or
    /// if it returns "none".
    fn eval_preconditions(&self, func_name: &str, args: &[Expression]) -> Vec<String> {
        let available_functions = self.evaluator.list_functions();
        if !available_functions.contains(&func_name.to_string()) {
            return vec![];
        }

        let call_expr = Expression::Operation {
            name: func_name.to_string(),
            args: args.to_vec(),
            span: None,
        };

        match self.evaluator.eval_concrete(&call_expr) {
            Ok(result) => Self::parse_preconditions(&result),
            Err(_) => vec![],
        }
    }

    /// Parse a precondition result expression into a list of steps.
    ///
    /// Convention:
    /// - "none" → empty (no preconditions)
    /// - "cargo test" → ["cargo test"]
    /// - "cargo test && cargo clippy" → ["cargo test", "cargo clippy"]
    fn parse_preconditions(result: &Expression) -> Vec<String> {
        let text = match result {
            Expression::String(s) => s.clone(),
            Expression::Object(s) => s.clone(),
            Expression::Const(s) => s.clone(),
            _ => return vec![],
        };

        let lower = text.to_lowercase();
        if lower == "none" || lower.is_empty() {
            return vec![];
        }

        text.split("&&")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Check if an evaluation result means "allowed"
    ///
    /// The Kleis evaluator fully reduces expressions — `eval_concrete` handles
    /// conditionals, `=` comparisons, `hasPrefix`, `contains`, etc. So by the
    /// time we get here, the result is a terminal value (String, Const, or Object).
    ///
    /// Recognizes:
    /// - String: "allow", "allowed", "true", "yes"
    /// - Const: "1", "true"
    /// - Object: "true", "allow"
    fn is_allowed(result: &Expression) -> bool {
        match result {
            Expression::String(s) => {
                let lower = s.to_lowercase();
                lower == "allow" || lower == "allowed" || lower == "true" || lower == "yes"
            }
            Expression::Const(val) => val == "1" || val.to_lowercase() == "true",
            Expression::Object(name) => {
                let lower = name.to_lowercase();
                lower == "true" || lower == "allow" || lower == "allowed"
            }
            _ => false,
        }
    }

    /// List all policy rules
    pub fn list_rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    /// Explain a specific rule
    pub fn explain_rule(&self, rule_name: &str) -> Option<&PolicyRule> {
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
                .filter(|r| matches!(r.kind, RuleKind::CheckFunction))
                .count(),
        );
        stats.insert(
            "preconditions".to_string(),
            self.rules
                .iter()
                .filter(|r| matches!(r.kind, RuleKind::Precondition))
                .count(),
        );
        stats.insert(
            "axioms".to_string(),
            self.rules
                .iter()
                .filter(|r| matches!(r.kind, RuleKind::Axiom))
                .count(),
        );
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
        stats
    }

    // ========================================================================
    // Schema Introspection — expose Kleis structures to the agent
    // ========================================================================

    /// Describe the full schema: structures, data types, axioms, functions.
    ///
    /// No Z3 needed — this is pure AST introspection.
    /// Returns a structured JSON value that the agent can use to understand
    /// the domain vocabulary and formulate its own queries.
    pub fn describe_schema(&self) -> Value {
        let pp = PrettyPrinter::new();

        // ---- Structures ----
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

        // ---- Data types ----
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

        // ---- Functions (with bodies rendered as Kleis syntax) ----
        let functions: Vec<Value> = self
            .evaluator
            .list_functions()
            .iter()
            .map(|name| {
                if let Some(closure) = self.evaluator.get_function(name) {
                    let body_kleis = pp.format_expression(&closure.body);
                    serde_json::json!({
                        "name": name,
                        "params": closure.params,
                        "body": body_kleis,
                        "kleis": pp.format_function(name, closure),
                    })
                } else {
                    serde_json::json!({ "name": name })
                }
            })
            .collect();

        // Separate functions by role for agent readability
        let check_fns: Vec<&Value> = functions
            .iter()
            .filter(|f| {
                f.get("name")
                    .and_then(|n| n.as_str())
                    .is_some_and(|n| n.starts_with("check_"))
            })
            .collect();
        let before_fns: Vec<&Value> = functions
            .iter()
            .filter(|f| {
                f.get("name")
                    .and_then(|n| n.as_str())
                    .is_some_and(|n| n.starts_with("before_"))
            })
            .collect();
        let helper_fns: Vec<&Value> = functions
            .iter()
            .filter(|f| {
                f.get("name")
                    .and_then(|n| n.as_str())
                    .is_some_and(|n| !n.starts_with("check_") && !n.starts_with("before_"))
            })
            .collect();

        // ---- Synthesize verifiable propositions ----
        //
        // Give the agent concrete examples of propositions it can send to
        // `evaluate` for Z3 verification.  These are inferred from the
        // schema: for every check_* function we generate a "boundary"
        // proposition (∀ over its params, result = "deny" or "allow").
        let verifiable_propositions = Self::synthesize_propositions(&functions, &structures);

        serde_json::json!({
            "policy_file": self.policy_file.display().to_string(),
            "structures": structures,
            "data_types": data_types,
            "check_functions": check_fns,
            "precondition_functions": before_fns,
            "helper_functions": helper_fns,
            "verifiable_propositions": verifiable_propositions,
            "stats": {
                "structures": structures.len(),
                "data_types": data_types.len(),
                "functions": functions.len(),
                "axioms": self.rules.iter().filter(|r| matches!(r.kind, RuleKind::Axiom)).count(),
            }
        })
    }

    /// Synthesize example propositions the agent can send to `evaluate`
    /// for Z3 verification.
    ///
    /// Inferred from the loaded schema — the agent doesn't have to invent
    /// Kleis syntax from scratch.
    fn synthesize_propositions(functions: &[Value], structures: &[Value]) -> Vec<Value> {
        let mut props: Vec<Value> = Vec::new();

        // ---- From check_* functions ----
        for f in functions {
            let name = match f.get("name").and_then(|n| n.as_str()) {
                Some(n) if n.starts_with("check_") => n,
                _ => continue,
            };
            let params = match f.get("params").and_then(|p| p.as_array()) {
                Some(p) => p,
                None => continue,
            };

            // Build typed parameter list for ∀(...)
            let typed_params: Vec<String> = params
                .iter()
                .filter_map(|p| p.as_str())
                .map(|p| format!("{} : String", p))
                .collect();

            if typed_params.is_empty() {
                continue;
            }

            let param_names: Vec<&str> = params.iter().filter_map(|p| p.as_str()).collect();
            let call = format!("{}({})", name, param_names.join(", "));

            // Generic proposition: ∀(params). f(params) = "allow" or "deny"
            let forall = format!("∀({})", typed_params.join(", "));
            props.push(serde_json::json!({
                "kleis": format!("{forall}. {call} = \"allow\""),
                "description": format!("Is {name} always allowed?"),
                "hint": "verify",
            }));
            props.push(serde_json::json!({
                "kleis": format!("{forall}. {call} = \"deny\""),
                "description": format!("Is {name} always denied?"),
                "hint": "verify",
            }));

            // Concrete spot-check with first param as a sample value
            let sample_args: Vec<String> = param_names
                .iter()
                .map(|p| {
                    if *p == "force" {
                        "1".to_string()
                    } else {
                        format!("\"test_{}\"", p)
                    }
                })
                .collect();
            props.push(serde_json::json!({
                "kleis": format!("{}({})", name, sample_args.join(", ")),
                "description": format!("Evaluate {name} with sample inputs"),
                "hint": "evaluate",
            }));
        }

        // ---- Regex examples (teach the agent regex syntax) ----
        props.push(serde_json::json!({
            "kleis": "isAscii(\"hello world\")",
            "description": "Check if a concrete string is ASCII printable",
            "hint": "evaluate",
        }));
        props.push(serde_json::json!({
            "kleis": "matches(\"foo42\", re_concat(re_literal(\"foo\"), re_plus(re_range(\"0\", \"9\"))))",
            "description": "Check if \"foo42\" matches pattern foo[0-9]+",
            "hint": "evaluate",
        }));
        props.push(serde_json::json!({
            "kleis": "∀(s : String). implies(isDigits(s), isAlphaNum(s))",
            "description": "Prove: all-digit strings are also alphanumeric",
            "hint": "verify",
        }));
        props.push(serde_json::json!({
            "kleis": "∀(s : String). implies(isAlphaNum(s), isAscii(s))",
            "description": "Prove: all alphanumeric strings are ASCII printable",
            "hint": "verify",
        }));

        // ---- From structure axioms ----
        for s in structures {
            let s_name = s.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            if let Some(axioms) = s.get("axioms").and_then(|a| a.as_array()) {
                for ax in axioms {
                    let ax_name = ax.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let ax_kleis = ax.get("kleis").and_then(|k| k.as_str()).unwrap_or("?");
                    props.push(serde_json::json!({
                        "kleis": ax_kleis,
                        "description": format!("Axiom {}.{} — verify with Z3", s_name, ax_name),
                        "hint": "verify",
                    }));
                }
            }
        }

        props
    }

    // ========================================================================
    // Expression Evaluation — general-purpose Kleis evaluation
    // ========================================================================

    /// Evaluate an arbitrary Kleis expression string.
    ///
    /// The expression is parsed and evaluated in the context of the loaded
    /// policy (all `define` functions are available). This generalizes
    /// `check_action` — the agent can call any Kleis function or verify
    /// any proposition.
    ///
    /// For propositions (quantified expressions, equalities, boolean claims):
    /// uses the evaluator's `verify_proposition()` which tries concrete
    /// evaluation first, then falls back to Z3 — the same pipeline that
    /// Kleis `assert()` uses inside example blocks.
    ///
    /// # Examples
    /// ```ignore
    /// // Concrete evaluation
    /// engine.evaluate_expression("check_file_delete(\"src/main.rs\")")
    /// // → EvalResult { value: "deny", .. }
    ///
    /// // Proposition verification (uses Z3)
    /// engine.evaluate_expression("∀(b : String). check_git_push(b, 1) = \"deny\"")
    /// // → EvalResult { verified: Some(true), .. }
    /// ```
    pub fn evaluate_expression(&self, expr_str: &str) -> EvalResult {
        // Parse the expression using the Kleis parser
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

        // If the expression is a proposition (quantifier, or looks like a
        // boolean claim), use the evaluator's assert pipeline which
        // automatically routes to Z3 when needed.
        if Self::is_proposition(&expr) {
            return Self::proposition_result(&self.evaluator.verify_proposition(&expr));
        }

        // Otherwise, evaluate concretely
        match self.evaluator.eval_concrete(&expr) {
            Ok(result) => EvalResult {
                value: Some(Self::expression_to_string(&result)),
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

    /// Check if an expression looks like a proposition (should be verified,
    /// not just evaluated).
    fn is_proposition(expr: &Expression) -> bool {
        match expr {
            // Quantified: ∀(...). ... or ∃(...). ...
            Expression::Quantifier { .. } => true,
            // Top-level implication: a → b
            Expression::Operation { name, .. }
                if name == "implies" || name == "→" || name == "⟹" =>
            {
                true
            }
            // Top-level logical connective with equality inside
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

    /// Convert an AssertResult into an EvalResult for the MCP response.
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
            AssertResult::Failed {
                expected: _,
                actual,
            } => EvalResult {
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
                error: Some(
                    "AXIOM INCONSISTENCY: loaded axioms are contradictory — \
                     all assertions would be vacuously true"
                        .to_string(),
                ),
            },
        }
    }

    /// Convert an Expression to a human-readable string
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

/// Result of evaluating a Kleis expression via MCP.
///
/// Covers both concrete evaluation and proposition verification:
/// - For concrete expressions: `value` is set, `verified` is None
/// - For propositions: `verified` is set (true/false), `witness` on disproof
/// - On error: `error` is set
#[derive(Debug, Clone)]
pub struct EvalResult {
    /// The evaluated value (for concrete expressions or verdict summary)
    pub value: Option<String>,
    /// Whether the proposition was verified (None for non-propositions)
    pub verified: Option<bool>,
    /// Structured Z3 witness (if proposition was disproved).
    /// Contains Kleis expression bindings for each quantified variable.
    pub witness: Option<Witness>,
    /// Error message (parse error, evaluation error, etc.)
    pub error: Option<String>,
}
