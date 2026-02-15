use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::json;

use super::shared::{map_tool_output, parse_arguments};
use crate::llm::api_error::extract_api_error;
use crate::llm::parse::parse_candidate_text;
use crate::llm::tools::openai_tools;
use crate::llm::{
    COMMAND_TOOL_NAME, LlmClient, LlmOutput, QUESTION_TOOL_NAME, TEXT_QUESTION_TOOL_NAME,
};

impl LlmClient {
    pub(crate) async fn call_openai(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmOutput> {
        let base = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let url = format!("{}/chat/completions", base.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "tools": openai_tools(),
            "tool_choice": "required",
            "temperature": 0.2
        });

        let response = self
            .http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let payload = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "OpenAI API error ({}): {}",
                status,
                extract_api_error(&payload)
            ));
        }

        let parsed: OpenAIResponse = serde_json::from_str(&payload)
            .with_context(|| "failed to parse OpenAI response JSON")?;

        if let Some(calls) = parsed
            .choices
            .first()
            .and_then(|choice| choice.message.tool_calls.as_ref())
        {
            for call in calls {
                if !is_supported_tool(&call.function.name) {
                    continue;
                }
                let args_value = parse_arguments(&call.function.arguments, "OpenAI")?;
                return map_tool_output("OpenAI", &call.function.name, args_value);
            }
            return Err(anyhow!("no supported tool call returned from OpenAI"));
        }

        let content = parsed
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_deref())
            .ok_or_else(|| anyhow!("no tool call or message content returned from OpenAI"))?;
        parse_candidate_text(content)
    }
}

fn is_supported_tool(name: &str) -> bool {
    matches!(
        name,
        COMMAND_TOOL_NAME | QUESTION_TOOL_NAME | TEXT_QUESTION_TOOL_NAME
    )
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    function: OpenAIFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}
