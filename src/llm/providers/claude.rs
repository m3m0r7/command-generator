use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};

use super::shared::map_tool_output;
use crate::llm::api_error::extract_api_error;
use crate::llm::parse::parse_candidate_text;
use crate::llm::tools::claude_tools;
use crate::llm::{LlmClient, LlmOutput};

impl LlmClient {
    pub(crate) async fn call_claude(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmOutput> {
        let base = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());
        let version =
            std::env::var("ANTHROPIC_API_VERSION").unwrap_or_else(|_| "2023-06-01".to_string());
        let url = format!("{}/messages", base.trim_end_matches('/'));

        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "temperature": 0.2,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "tools": claude_tools(),
            "tool_choice": { "type": "any" }
        });

        let response = self
            .http
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", version)
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let payload = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "Claude API error ({}): {}",
                status,
                extract_api_error(&payload)
            ));
        }

        let parsed: ClaudeResponse = serde_json::from_str(&payload)
            .with_context(|| "failed to parse Claude response JSON")?;
        if let Some(block) = parsed.content.iter().find(|block| block.kind == "tool_use") {
            let input = block.input.clone().unwrap_or_else(|| json!({}));
            if let Some(name) = block.name.as_deref() {
                return map_tool_output("Claude", name, input);
            }
            return Err(anyhow!("tool_use block from Claude missing name"));
        }

        let content = parsed
            .content
            .iter()
            .find_map(|block| block.text.as_deref())
            .ok_or_else(|| anyhow!("no tool call or text block returned from Claude"))?;
        parse_candidate_text(content)
    }
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    #[serde(default)]
    content: Vec<ClaudeContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContentBlock {
    #[serde(rename = "type")]
    kind: String,
    name: Option<String>,
    text: Option<String>,
    input: Option<Value>,
}
