use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};

use super::shared::map_tool_output;
use crate::llm::api_error::extract_api_error;
use crate::llm::parse::parse_candidate_text;
use crate::llm::tools::gemini_function_declarations;
use crate::llm::{
    COMMAND_TOOL_NAME, LlmClient, LlmOutput, QUESTION_TOOL_NAME, TEXT_QUESTION_TOOL_NAME,
};

impl LlmClient {
    pub(crate) async fn call_gemini(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmOutput> {
        let base = std::env::var("GEMINI_BASE_URL")
            .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let model_path = if self.model.starts_with("models/") {
            self.model.clone()
        } else {
            format!("models/{}", self.model)
        };
        let endpoint = format!(
            "{}/{}:generateContent",
            base.trim_end_matches('/'),
            model_path
        );
        let mut url = reqwest::Url::parse(&endpoint)
            .with_context(|| "failed to parse Gemini endpoint URL")?;
        url.query_pairs_mut().append_pair("key", &self.api_key);

        let body = json!({
            "system_instruction": {
                "parts": [
                    {"text": system_prompt}
                ]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        {"text": user_prompt}
                    ]
                }
            ],
            "tools": [
                {
                    "functionDeclarations": gemini_function_declarations()
                }
            ],
            "toolConfig": {
                "functionCallingConfig": {
                    "mode": "ANY",
                    "allowedFunctionNames": [COMMAND_TOOL_NAME, QUESTION_TOOL_NAME, TEXT_QUESTION_TOOL_NAME]
                }
            },
            "generationConfig": {
                "temperature": 0.2
            }
        });

        let response = self.http.post(url).json(&body).send().await?;
        let status = response.status();
        let payload = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "Gemini API error ({}): {}",
                status,
                extract_api_error(&payload)
            ));
        }

        let parsed: GeminiResponse = serde_json::from_str(&payload)
            .with_context(|| "failed to parse Gemini response JSON")?;

        if let Some(function_call) = parsed.candidates.first().and_then(|candidate| {
            candidate
                .content
                .parts
                .iter()
                .find_map(|part| part.function_call.clone())
        }) {
            let args_value = function_call.args.unwrap_or_else(|| json!({}));
            return map_tool_output("Gemini", &function_call.name, args_value);
        }

        let content = parsed
            .candidates
            .first()
            .and_then(|candidate| {
                candidate
                    .content
                    .parts
                    .iter()
                    .find_map(|part| part.text.as_deref())
            })
            .ok_or_else(|| anyhow!("no tool call or text candidate returned from Gemini"))?;
        parse_candidate_text(content)
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    #[serde(default)]
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: Option<String>,
    #[serde(rename = "functionCall")]
    function_call: Option<GeminiFunctionCall>,
}

#[derive(Debug, Deserialize, Clone)]
struct GeminiFunctionCall {
    name: String,
    #[serde(default)]
    args: Option<Value>,
}
