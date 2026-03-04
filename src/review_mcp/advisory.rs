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

    #[derive(Debug, Clone)]
    pub struct AdvisoryConfig {
        pub api_key: String,
        pub endpoint: String,
        pub model: String,
    }

    #[derive(Debug, Clone)]
    pub struct Advisory {
        pub check: String,
        pub severity: String,
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
        severity: Option<String>,
        reason: Option<String>,
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
            })
        }
    }

    fn build_system_prompt(language: &str) -> String {
        format!(
            "You are a {language} code reviewer. Review the provided code for:\n\
             - Potential bugs or logic errors\n\
             - Idiomatic {language} improvements\n\
             - Naming and readability\n\
             - Performance concerns\n\n\
             IMPORTANT: A formal static analysis tool has already flagged some issues.\n\
             Those findings will be listed under \"Already flagged by formal checks\".\n\
             Do NOT repeat those. Only report NEW findings that the formal tool missed.\n\n\
             Be concise. Return ONLY a JSON array of findings. Each finding has:\n\
             - \"check\": short name (e.g. \"unnecessary-clone\")\n\
             - \"severity\": \"warning\" or \"info\"\n\
             - \"reason\": one-sentence explanation\n\n\
             If the code looks good (beyond what was already flagged), return an empty array: []"
        )
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
    pub async fn get_advisories(
        config: &AdvisoryConfig,
        source: &str,
        file_path: &str,
        language: &str,
        formal_messages: &[String],
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

        let already_flagged = format_formal_findings(formal_messages);
        let user_content = format!(
            "Review this {language} file ({file_path}):\n\n\
             Already flagged by formal checks:\n{already_flagged}\n\n\
             ```{fence_tag}\n{source}\n```"
        );

        let request = ChatRequest {
            model: config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: build_system_prompt(language),
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
            .map(|item| Advisory {
                check: item.check.unwrap_or_else(|| "unnamed".to_string()),
                severity: item.severity.unwrap_or_else(|| "info".to_string()),
                reason: item.reason.unwrap_or_else(|| "no reason given".to_string()),
            })
            .collect())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_clean_json() {
            let input = r#"[{"check":"unnecessary-clone","severity":"warning","reason":"Clone is not needed here."}]"#;
            let result = parse_advisories(input).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].check, "unnecessary-clone");
            assert_eq!(result[0].severity, "warning");
        }

        #[test]
        fn parse_markdown_fenced_json() {
            let input = "```json\n[{\"check\":\"use-iter\",\"severity\":\"info\",\"reason\":\"Use iterator.\"}]\n```";
            let result = parse_advisories(input).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].check, "use-iter");
        }

        #[test]
        fn parse_empty_array() {
            let result = parse_advisories("[]").unwrap();
            assert!(result.is_empty());
        }
    }
}

#[cfg(feature = "llm-advisory")]
pub use inner::*;

#[cfg(not(feature = "llm-advisory"))]
mod stub {
    use crate::config::LlmConfig as LlmCfg;

    #[derive(Debug, Clone)]
    pub struct AdvisoryConfig;

    #[derive(Debug, Clone)]
    pub struct Advisory {
        pub check: String,
        pub severity: String,
        pub reason: String,
    }

    impl AdvisoryConfig {
        pub fn from_config(_llm: &LlmCfg) -> Option<Self> {
            None
        }
    }

    pub async fn get_advisories(
        _config: &AdvisoryConfig,
        _source: &str,
        _file_path: &str,
        _formal_messages: &[String],
    ) -> Result<Vec<Advisory>, String> {
        Err("LLM advisory not available (compiled without llm-advisory feature)".to_string())
    }
}

#[cfg(not(feature = "llm-advisory"))]
pub use stub::*;
