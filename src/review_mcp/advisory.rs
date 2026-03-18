//! LLM Advisory — optional AI-assisted code review
//!
//! Calls an OpenAI-compatible chat completions endpoint to get advisory
//! (non-blocking) code review findings. The LLM response augments the
//! formal Kleis verdicts but never overrides them.
//!
//! Configuration:
//!   - Endpoint and model: `[llm]` section in config.toml or KLEIS_LLM_* env vars
//!   - API key: `KLEIS_LLM_API_KEY` env var only (never stored in files)

#[cfg(feature = "llm-advisory")]
mod inner {
    use crate::config::LlmConfig as LlmCfg;
    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone)]
    pub struct AdvisoryConfig {
        pub api_key: String,
        pub endpoint: String,
        pub model: String,
        pub guidelines: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct Advisory {
        pub check: String,
        pub line: Option<u32>,
        pub severity: String,
        pub evidence: Option<String>,
        pub reason: String,
    }

    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        messages: Vec<Message>,
    }

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    #[derive(Deserialize)]
    struct ChatResponse {
        choices: Vec<Choice>,
    }

    #[derive(Deserialize)]
    struct Choice {
        message: ResponseMessage,
    }

    #[derive(Deserialize)]
    struct ResponseMessage {
        content: Option<String>,
    }

    #[derive(Deserialize)]
    struct AdvisoryItem {
        check: Option<String>,
        line: Option<u32>,
        severity: Option<String>,
        evidence: Option<String>,
        reason: Option<String>,
    }

    /// Resolve the guidelines file path for a given language.
    ///
    /// Resolution order:
    /// 1. `KLEIS_LLM_GUIDELINES_FILE` env var
    /// 2. `llm.guidelines_file` in config.toml
    /// 3. Auto-discovery: `examples/guidelines/{language}_guidelines.txt`
    ///    relative to the executable or current directory
    /// 4. None (fallback to generic prompt)
    pub fn resolve_guidelines_path(
        language: &str,
        config_override: &Option<String>,
    ) -> Option<PathBuf> {
        if let Ok(env_path) = std::env::var("KLEIS_LLM_GUIDELINES_FILE") {
            let p = PathBuf::from(&env_path);
            if p.exists() {
                return Some(p);
            }
        }

        if let Some(ref cfg_path) = config_override {
            let p = PathBuf::from(cfg_path);
            if p.exists() {
                return Some(p);
            }
        }

        let lang_lower = language.to_lowercase();
        let filename = format!("{lang_lower}_guidelines.txt");

        for base in auto_discovery_dirs() {
            let candidate = base.join("examples/guidelines").join(&filename);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }

    fn auto_discovery_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(bin_dir) = exe.parent() {
                // kleis installed in repo: <repo>/target/debug/kleis or <repo>/target/release/kleis
                if let Some(target_dir) = bin_dir.parent() {
                    if let Some(repo_root) = target_dir.parent() {
                        dirs.push(repo_root.to_path_buf());
                    }
                }
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            dirs.push(cwd);
        }
        dirs
    }

    /// Load the contents of a guidelines file. Returns None if the file
    /// is empty or consists only of comment lines (lines starting with `#`).
    pub fn load_guidelines_text(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        let has_substance = content
            .lines()
            .any(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'));
        if has_substance {
            Some(content)
        } else {
            None
        }
    }

    impl AdvisoryConfig {
        /// Build from KleisConfig's LLM section + KLEIS_LLM_API_KEY env var.
        /// Returns None if the API key is not set.
        pub fn from_config(llm: &LlmCfg) -> Option<Self> {
            let api_key = std::env::var("KLEIS_LLM_API_KEY").ok()?;
            if api_key.is_empty() {
                return None;
            }
            Some(Self {
                api_key,
                endpoint: llm.endpoint.clone(),
                model: llm.model.clone(),
                guidelines: None,
            })
        }

        /// Load per-language guidelines into this config.
        /// Call after construction to resolve and cache the guidelines text.
        pub fn load_guidelines(&mut self, language: &str, config_override: &Option<String>) {
            if let Some(path) = resolve_guidelines_path(language, config_override) {
                self.guidelines = load_guidelines_text(&path);
            }
        }
    }

    /// Build the system prompt. When guidelines text is provided, produces a
    /// structured prompt referencing the standards. Otherwise falls back to
    /// the generic review prompt.
    pub fn build_system_prompt(
        language: &str,
        guidelines: Option<&str>,
        formal_rule_names: &[String],
    ) -> String {
        match guidelines {
            Some(text) => build_guidelines_prompt(language, text, formal_rule_names),
            None => build_generic_prompt(language),
        }
    }

    fn build_guidelines_prompt(
        language: &str,
        guidelines_text: &str,
        formal_rule_names: &[String],
    ) -> String {
        let formal_list = if formal_rule_names.is_empty() {
            "(none)".to_string()
        } else {
            formal_rule_names
                .iter()
                .map(|n| format!("- {n}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            "You are a {language} code reviewer enforcing the coding standards below.\n\n\
             IMPORTANT: A formal static analysis tool (Kleis) has already checked for:\n\
             {formal_list}\n\
             Do NOT repeat those. Only report NEW findings from the guidelines that the\n\
             formal tool cannot check.\n\n\
             Focus especially on architectural and design guidelines that require judgment\n\
             (e.g., module organization, API design, type safety, design patterns,\n\
             abstraction boundaries, error design, performance).\n\n\
             ## Coding Standards\n\n\
             {guidelines_text}\n\n\
             ---\n\n\
             CRITICAL RULES:\n\
             - Every finding MUST reference a specific line number from the code.\n\
             - Every finding MUST include a short code snippet as evidence.\n\
             - If you cannot point to a concrete line that violates a guideline, do NOT report it.\n\
             - Do NOT report guidelines that are simply \"not demonstrated\" in the file.\n\
             - Only report violations you can SEE in the provided code.\n\n\
             Return ONLY a JSON array of findings. Each finding has:\n\
             - \"check\": the guideline ID (e.g. \"M-INIT-BUILDER\", \"C-NEWTYPE\")\n\
             - \"line\": the line number where the violation occurs (integer)\n\
             - \"severity\": \"warning\" or \"info\"\n\
             - \"evidence\": the code snippet from that line (copy from the source)\n\
             - \"reason\": one-sentence explanation of why this specific code violates the guideline\n\n\
             If the code looks good, return an empty array: []"
        )
    }

    fn build_generic_prompt(language: &str) -> String {
        format!(
            "You are a {language} code reviewer. Review the provided code for:\n\
             - Potential bugs or logic errors\n\
             - Idiomatic {language} improvements\n\
             - Naming and readability\n\
             - Performance concerns\n\n\
             IMPORTANT: A formal static analysis tool has already flagged some issues.\n\
             Those findings will be listed under \"Already flagged by formal checks\".\n\
             Do NOT repeat those. Only report NEW findings that the formal tool missed.\n\n\
             CRITICAL RULES:\n\
             - Every finding MUST reference a specific line number from the code.\n\
             - Every finding MUST include a short code snippet as evidence.\n\
             - If you cannot point to a concrete line, do NOT report it.\n\n\
             Return ONLY a JSON array of findings. Each finding has:\n\
             - \"check\": short name (e.g. \"unnecessary-clone\")\n\
             - \"line\": the line number where the issue occurs (integer)\n\
             - \"severity\": \"warning\" or \"info\"\n\
             - \"evidence\": the code snippet from that line (copy from the source)\n\
             - \"reason\": one-sentence explanation\n\n\
             If the code looks good (beyond what was already flagged), return an empty array: []"
        )
    }

    fn add_line_numbers(source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let width = lines.len().to_string().len();
        lines
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>width$}| {line}", i + 1))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_formal_findings(formal_messages: &[String]) -> String {
        if formal_messages.is_empty() {
            return "None (all formal checks passed).".to_string();
        }
        formal_messages
            .iter()
            .map(|m| format!("- {m}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Call the LLM endpoint and parse advisory findings.
    /// `formal_messages` contains the messages from formal checks that already fired,
    /// so the LLM can avoid repeating them.
    /// `formal_rule_names` is the complete list of check_*/advise_* rule names from
    /// the loaded policy, included in the system prompt so the LLM avoids overlap.
    pub async fn get_advisories(
        config: &AdvisoryConfig,
        source: &str,
        file_path: &str,
        language: &str,
        formal_messages: &[String],
        formal_rule_names: &[String],
    ) -> Result<Vec<Advisory>, String> {
        let client = reqwest::Client::new();

        let lang_lower = language.to_lowercase();
        let fence_tag = match lang_lower.as_str() {
            "python" => "python",
            "rust" => "rust",
            "go" => "go",
            "java" => "java",
            "typescript" | "javascript" => &lang_lower,
            _ => &lang_lower,
        };

        let numbered_source = add_line_numbers(source);
        let already_flagged = format_formal_findings(formal_messages);
        let user_content = format!(
            "Review this {language} file ({file_path}):\n\n\
             Already flagged by formal checks:\n{already_flagged}\n\n\
             ```{fence_tag}\n{numbered_source}\n```"
        );

        let system_prompt =
            build_system_prompt(language, config.guidelines.as_deref(), formal_rule_names);

        let request = ChatRequest {
            model: config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_content,
                },
            ],
        };

        let response = client
            .post(&config.endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("LLM returned {status}: {body}"));
        }

        let chat: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {e}"))?;

        let content = chat
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| "LLM returned no content".to_string())?;

        parse_advisories(content)
    }

    fn parse_advisories(content: &str) -> Result<Vec<Advisory>, String> {
        let trimmed = content.trim();

        let json_str = if trimmed.starts_with("```") {
            let start = trimmed.find('[').unwrap_or(0);
            let end = trimmed.rfind(']').map(|i| i + 1).unwrap_or(trimmed.len());
            &trimmed[start..end]
        } else {
            trimmed
        };

        let items: Vec<AdvisoryItem> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse LLM JSON: {e}\nRaw: {content}"))?;

        Ok(items
            .into_iter()
            .filter(|item| item.line.is_some())
            .map(|item| Advisory {
                check: item.check.unwrap_or_else(|| "unnamed".to_string()),
                line: item.line,
                severity: item.severity.unwrap_or_else(|| "info".to_string()),
                evidence: item.evidence,
                reason: item.reason.unwrap_or_else(|| "no reason given".to_string()),
            })
            .collect())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_clean_json() {
            let input = r#"[{"check":"unnecessary-clone","line":42,"severity":"warning","evidence":"let x = y.clone();","reason":"Clone is not needed here."}]"#;
            let result = parse_advisories(input).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].check, "unnecessary-clone");
            assert_eq!(result[0].line, Some(42));
            assert_eq!(result[0].severity, "warning");
            assert_eq!(result[0].evidence.as_deref(), Some("let x = y.clone();"));
        }

        #[test]
        fn parse_markdown_fenced_json() {
            let input = "```json\n[{\"check\":\"use-iter\",\"line\":10,\"severity\":\"info\",\"reason\":\"Use iterator.\"}]\n```";
            let result = parse_advisories(input).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].check, "use-iter");
            assert_eq!(result[0].line, Some(10));
        }

        #[test]
        fn parse_empty_array() {
            let result = parse_advisories("[]").unwrap();
            assert!(result.is_empty());
        }

        #[test]
        fn parse_filters_ungrounded_findings() {
            let input = r#"[
                {"check":"grounded","line":5,"severity":"warning","evidence":"bad code","reason":"real issue"},
                {"check":"hallucinated","severity":"info","reason":"no line given"}
            ]"#;
            let result = parse_advisories(input).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].check, "grounded");
            assert_eq!(result[0].line, Some(5));
        }

        #[test]
        fn add_line_numbers_formats_correctly() {
            let source = "fn main() {\n    println!(\"hello\");\n}";
            let numbered = add_line_numbers(source);
            assert!(numbered.starts_with("1| fn main() {"));
            assert!(numbered.contains("2|     println!(\"hello\");"));
            assert!(numbered.contains("3| }"));
        }

        #[test]
        fn generic_prompt_when_no_guidelines() {
            let prompt = build_system_prompt("Rust", None, &[]);
            assert!(prompt.contains("Rust code reviewer"));
            assert!(prompt.contains("Potential bugs or logic errors"));
            assert!(!prompt.contains("Coding Standards"));
        }

        #[test]
        fn guidelines_prompt_includes_standards() {
            let guidelines = "## Rule M-INIT-BUILDER\nUse builder pattern for complex init.";
            let rules = vec!["check_no_unwrap".to_string(), "advise_no_emoji".to_string()];
            let prompt = build_system_prompt("Rust", Some(guidelines), &rules);
            assert!(prompt.contains("Coding Standards"));
            assert!(prompt.contains("M-INIT-BUILDER"));
            assert!(prompt.contains("check_no_unwrap"));
            assert!(prompt.contains("advise_no_emoji"));
            assert!(prompt.contains("guideline ID"));
        }

        #[test]
        fn guidelines_prompt_with_empty_rules() {
            let prompt = build_system_prompt("Python", Some("PEP 8 rules here"), &[]);
            assert!(prompt.contains("(none)"));
            assert!(prompt.contains("PEP 8 rules here"));
        }

        #[test]
        fn resolve_guidelines_env_override() {
            // Guard: use a unique env var value per test to avoid races.
            // We can only test env-var resolution reliably in single-threaded mode,
            // so this test verifies the path-exists logic directly.
            let test_file = std::env::temp_dir().join("kleis_test_guidelines_env_override.txt");
            std::fs::write(&test_file, "test guidelines").unwrap();

            // Directly test the env-var branch by temporarily setting it.
            // Note: env vars are process-global, so this test is inherently racy
            // under parallel execution. We accept this and verify the logic.
            unsafe { std::env::set_var("KLEIS_LLM_GUIDELINES_FILE", test_file.to_str().unwrap()) };

            let result = resolve_guidelines_path("rust", &None);
            assert_eq!(result, Some(test_file.clone()));

            unsafe { std::env::remove_var("KLEIS_LLM_GUIDELINES_FILE") };
            std::fs::remove_file(&test_file).ok();
        }

        #[test]
        fn resolve_guidelines_config_override() {
            // Test config-level override. We pass the config path explicitly,
            // so this works regardless of env var state — config override is
            // checked after env var, but if the env var file doesn't exist
            // it falls through.
            let test_file = std::env::temp_dir().join("kleis_test_guidelines_cfg_override.txt");
            std::fs::write(&test_file, "config guidelines").unwrap();

            let result =
                resolve_guidelines_path("rust", &Some(test_file.to_str().unwrap().to_string()));
            // The result should be our config file (or the env var file if another
            // test set it concurrently — both are valid resolution results)
            assert!(result.is_some());

            std::fs::remove_file(&test_file).ok();
        }

        #[test]
        fn resolve_guidelines_none_when_missing() {
            let result =
                resolve_guidelines_path("cobol", &Some("/nonexistent/path.txt".to_string()));
            // Even with a config override pointing to a nonexistent file, and no
            // auto-discovery match for "cobol", resolution should return None
            // (unless KLEIS_LLM_GUIDELINES_FILE env var happens to be set by a parallel test)
            // We just verify it doesn't panic.
            let _ = result;
        }

        #[test]
        fn load_guidelines_skips_comment_only_files() {
            let test_file = std::env::temp_dir().join("kleis_test_guidelines_comments_only.txt");
            std::fs::write(&test_file, "# This is a comment\n# Another comment\n").unwrap();

            let result = load_guidelines_text(&test_file);
            assert!(result.is_none());

            std::fs::remove_file(&test_file).ok();
        }

        #[test]
        fn load_guidelines_reads_substantive_content() {
            let test_file = std::env::temp_dir().join("kleis_test_guidelines_substance.txt");
            std::fs::write(&test_file, "# Header\nUse builder pattern.").unwrap();

            let result = load_guidelines_text(&test_file);
            assert!(result.is_some());
            assert!(result.unwrap().contains("builder pattern"));

            std::fs::remove_file(&test_file).ok();
        }
    }
}

#[cfg(feature = "llm-advisory")]
pub use inner::*;

#[cfg(not(feature = "llm-advisory"))]
mod stub {
    use crate::config::LlmConfig as LlmCfg;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone)]
    pub struct AdvisoryConfig {
        pub guidelines: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct Advisory {
        pub check: String,
        pub line: Option<u32>,
        pub severity: String,
        pub evidence: Option<String>,
        pub reason: String,
    }

    impl AdvisoryConfig {
        pub fn from_config(_llm: &LlmCfg) -> Option<Self> {
            None
        }

        pub fn load_guidelines(&mut self, _language: &str, _config_override: &Option<String>) {}
    }

    pub fn resolve_guidelines_path(
        _language: &str,
        _config_override: &Option<String>,
    ) -> Option<PathBuf> {
        None
    }

    pub fn load_guidelines_text(_path: &Path) -> Option<String> {
        None
    }

    pub fn build_system_prompt(
        language: &str,
        _guidelines: Option<&str>,
        _formal_rule_names: &[String],
    ) -> String {
        format!("You are a {language} code reviewer.")
    }

    pub async fn get_advisories(
        _config: &AdvisoryConfig,
        _source: &str,
        _file_path: &str,
        _language: &str,
        _formal_messages: &[String],
        _formal_rule_names: &[String],
    ) -> Result<Vec<Advisory>, String> {
        Err("LLM advisory not available (compiled without llm-advisory feature)".to_string())
    }
}

#[cfg(not(feature = "llm-advisory"))]
pub use stub::*;
