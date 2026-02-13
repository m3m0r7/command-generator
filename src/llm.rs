use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::model::ProviderKind;

pub const COMMAND_TOOL_NAME: &str = "deliver_command";
pub const QUESTION_TOOL_NAME: &str = "ask_yes_no_question";
pub const TEXT_QUESTION_TOOL_NAME: &str = "ask_text_question";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCandidate {
    pub command: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub question: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum LlmOutput {
    Command(CommandCandidate),
    QuestionYesNo(ClarificationQuestion),
    QuestionText(ClarificationQuestion),
}

#[derive(Debug, Clone)]
pub struct LlmClient {
    provider: ProviderKind,
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(
        provider: ProviderKind,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            api_key: api_key.into(),
            model: model.into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }

    pub async fn generate_output(&self, prompt: &str) -> Result<LlmOutput> {
        match self.provider {
            ProviderKind::OpenAI => self.call_openai(prompt).await,
            ProviderKind::Gemini => self.call_gemini(prompt).await,
            ProviderKind::Claude => self.call_claude(prompt).await,
        }
    }

    async fn call_openai(&self, prompt: &str) -> Result<LlmOutput> {
        let base = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let url = format!("{}/chat/completions", base.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "Use one of the provided function tools for every response."
                },
                {
                    "role": "user",
                    "content": prompt
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
                if call.function.name == COMMAND_TOOL_NAME {
                    let args_value: Value = serde_json::from_str(&call.function.arguments)
                        .with_context(|| "failed to parse OpenAI command tool arguments")?;
                    return command_from_value(args_value).map(LlmOutput::Command);
                }
                if call.function.name == QUESTION_TOOL_NAME {
                    let args_value: Value = serde_json::from_str(&call.function.arguments)
                        .with_context(|| "failed to parse OpenAI question tool arguments")?;
                    return question_from_value(args_value).map(LlmOutput::QuestionYesNo);
                }
                if call.function.name == TEXT_QUESTION_TOOL_NAME {
                    let args_value: Value = serde_json::from_str(&call.function.arguments)
                        .with_context(|| "failed to parse OpenAI text question tool arguments")?;
                    return question_from_value(args_value).map(LlmOutput::QuestionText);
                }
            }
            return Err(anyhow!("no supported tool call returned from OpenAI"));
        }

        let content = parsed
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_deref())
            .ok_or_else(|| anyhow!("no tool call or message content returned from OpenAI"))?;
        parse_candidate_text(content).map(LlmOutput::Command)
    }

    async fn call_gemini(&self, prompt: &str) -> Result<LlmOutput> {
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
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        {"text": prompt}
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
            if function_call.name == COMMAND_TOOL_NAME {
                return command_from_value(args_value).map(LlmOutput::Command);
            }
            if function_call.name == QUESTION_TOOL_NAME {
                return question_from_value(args_value).map(LlmOutput::QuestionYesNo);
            }
            if function_call.name == TEXT_QUESTION_TOOL_NAME {
                return question_from_value(args_value).map(LlmOutput::QuestionText);
            }
            return Err(anyhow!(
                "unsupported function call returned from Gemini: {}",
                function_call.name
            ));
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
        parse_candidate_text(content).map(LlmOutput::Command)
    }

    async fn call_claude(&self, prompt: &str) -> Result<LlmOutput> {
        let base = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());
        let version =
            std::env::var("ANTHROPIC_API_VERSION").unwrap_or_else(|_| "2023-06-01".to_string());
        let url = format!("{}/messages", base.trim_end_matches('/'));

        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "temperature": 0.2,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
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
            match block.name.as_deref() {
                Some(COMMAND_TOOL_NAME) => {
                    return command_from_value(input).map(LlmOutput::Command);
                }
                Some(QUESTION_TOOL_NAME) => {
                    return question_from_value(input).map(LlmOutput::QuestionYesNo);
                }
                Some(TEXT_QUESTION_TOOL_NAME) => {
                    return question_from_value(input).map(LlmOutput::QuestionText);
                }
                Some(other) => {
                    return Err(anyhow!(
                        "unsupported tool call returned from Claude: {}",
                        other
                    ));
                }
                None => return Err(anyhow!("tool_use block from Claude missing name")),
            }
        }

        let content = parsed
            .content
            .iter()
            .find_map(|block| block.text.as_deref())
            .ok_or_else(|| anyhow!("no tool call or text block returned from Claude"))?;
        parse_candidate_text(content).map(LlmOutput::Command)
    }
}

fn openai_tools() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": COMMAND_TOOL_NAME,
                "description": "Return a single shell command for the user request.",
                "parameters": command_tool_schema()
            }
        },
        {
            "type": "function",
            "function": {
                "name": QUESTION_TOOL_NAME,
                "description": "Ask a required yes/no clarification question before generating a command.",
                "parameters": question_tool_schema()
            }
        },
        {
            "type": "function",
            "function": {
                "name": TEXT_QUESTION_TOOL_NAME,
                "description": "Ask a required free-text clarification question before generating a command.",
                "parameters": text_question_tool_schema()
            }
        }
    ])
}

fn gemini_function_declarations() -> Value {
    json!([
        {
            "name": COMMAND_TOOL_NAME,
            "description": "Return a single shell command for the user request.",
            "parameters": command_tool_schema()
        },
        {
            "name": QUESTION_TOOL_NAME,
            "description": "Ask a required yes/no clarification question before generating a command.",
            "parameters": question_tool_schema()
        },
        {
            "name": TEXT_QUESTION_TOOL_NAME,
            "description": "Ask a required free-text clarification question before generating a command.",
            "parameters": text_question_tool_schema()
        }
    ])
}

fn claude_tools() -> Value {
    json!([
        {
            "name": COMMAND_TOOL_NAME,
            "description": "Return a single shell command for the user request.",
            "input_schema": command_tool_schema()
        },
        {
            "name": QUESTION_TOOL_NAME,
            "description": "Ask a required yes/no clarification question before generating a command.",
            "input_schema": question_tool_schema()
        },
        {
            "name": TEXT_QUESTION_TOOL_NAME,
            "description": "Ask a required free-text clarification question before generating a command.",
            "input_schema": text_question_tool_schema()
        }
    ])
}

fn command_tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "command": {
                "type": "string",
                "description": "Single shell command line."
            },
            "reason": {
                "type": "string",
                "description": "Short reason for the chosen command."
            }
        },
        "required": ["command", "reason"]
    })
}

fn question_tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "question": {
                "type": "string",
                "description": "One clear yes/no question."
            },
            "reason": {
                "type": "string",
                "description": "Short reason for asking this clarification."
            }
        },
        "required": ["question", "reason"]
    })
}

fn text_question_tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "question": {
                "type": "string",
                "description": "One clear question to collect a concrete value from user."
            },
            "reason": {
                "type": "string",
                "description": "Short reason for asking this clarification."
            }
        },
        "required": ["question", "reason"]
    })
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

fn command_from_value(value: Value) -> Result<CommandCandidate> {
    let mut candidate: CommandCandidate =
        serde_json::from_value(value).with_context(|| "failed to parse command tool arguments")?;
    candidate.command = candidate.command.trim().to_string();
    candidate.reason = candidate.reason.trim().to_string();
    if candidate.command.is_empty() {
        return Err(anyhow!("generated command is empty"));
    }
    Ok(candidate)
}

fn question_from_value(value: Value) -> Result<ClarificationQuestion> {
    let mut question: ClarificationQuestion =
        serde_json::from_value(value).with_context(|| "failed to parse question tool arguments")?;
    question.question = question.question.trim().to_string();
    question.reason = question.reason.trim().to_string();
    if question.question.is_empty() {
        return Err(anyhow!("clarification question is empty"));
    }
    Ok(question)
}

fn parse_candidate_text(raw: &str) -> Result<CommandCandidate> {
    let text = strip_fences(raw.trim());
    if let Ok(candidate) = serde_json::from_str::<CommandCandidate>(&text) {
        return command_from_value(serde_json::to_value(candidate)?);
    }

    if let Some(fragment) = extract_first_json_object(&text) {
        let parsed: Value =
            serde_json::from_str(&fragment).with_context(|| "failed to parse JSON fragment")?;
        return command_from_value(parsed);
    }

    Err(anyhow!("LLM output does not contain valid command data"))
}

fn strip_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    if !(trimmed.starts_with("```") && trimmed.ends_with("```")) {
        return trimmed.to_string();
    }

    let mut lines = trimmed.lines();
    let _ = lines.next();
    let mut out = String::new();
    for line in lines {
        if line.trim() == "```" {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

fn extract_first_json_object(raw: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == '{' {
            if depth == 0 {
                start = Some(idx);
            }
            depth += 1;
            continue;
        }
        if ch == '}' {
            if depth == 0 {
                continue;
            }
            depth -= 1;
            if depth == 0
                && let Some(begin) = start
            {
                return Some(raw[begin..=idx].to_string());
            }
        }
    }
    None
}

fn extract_api_error(body: &str) -> String {
    #[derive(Debug, Deserialize)]
    struct OpenAIErrorEnvelope {
        error: Option<OpenAIError>,
    }
    #[derive(Debug, Deserialize)]
    struct OpenAIError {
        message: Option<String>,
        #[serde(rename = "type")]
        kind: Option<String>,
        code: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    struct GeminiErrorEnvelope {
        error: Option<GeminiError>,
    }
    #[derive(Debug, Deserialize)]
    struct GeminiError {
        message: Option<String>,
        status: Option<String>,
        code: Option<i64>,
    }
    #[derive(Debug, Deserialize)]
    struct ClaudeErrorEnvelope {
        error: Option<ClaudeError>,
    }
    #[derive(Debug, Deserialize)]
    struct ClaudeError {
        #[serde(rename = "type")]
        kind: Option<String>,
        message: Option<String>,
    }

    if let Ok(parsed) = serde_json::from_str::<OpenAIErrorEnvelope>(body)
        && let Some(err) = parsed.error
    {
        let message = err.message.unwrap_or_else(|| "unknown error".to_string());
        let kind = err.kind.unwrap_or_else(|| "unknown".to_string());
        let code = err.code.unwrap_or_else(|| "none".to_string());
        return format!("{} (type={}, code={})", message, kind, code);
    }
    if let Ok(parsed) = serde_json::from_str::<GeminiErrorEnvelope>(body)
        && let Some(err) = parsed.error
    {
        let message = err.message.unwrap_or_else(|| "unknown error".to_string());
        let status = err.status.unwrap_or_else(|| "unknown".to_string());
        let code = err
            .code
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string());
        return format!("{} (status={}, code={})", message, status, code);
    }
    if let Ok(parsed) = serde_json::from_str::<ClaudeErrorEnvelope>(body)
        && let Some(err) = parsed.error
    {
        let message = err.message.unwrap_or_else(|| "unknown error".to_string());
        let kind = err.kind.unwrap_or_else(|| "unknown".to_string());
        return format!("{} (type={})", message, kind);
    }
    body.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_json_as_command() {
        let parsed = parse_candidate_text(r#"{"command":"pwd","reason":"show cwd"}"#).unwrap();
        assert_eq!(parsed.command, "pwd");
    }

    #[test]
    fn parses_fenced_json_as_command() {
        let parsed =
            parse_candidate_text("```json\n{\"command\":\"ls\",\"reason\":\"list\"}\n```").unwrap();
        assert_eq!(parsed.command, "ls");
    }

    #[test]
    fn parses_command_tool_args() {
        let value = json!({"command":"pwd","reason":"cwd"});
        let parsed = command_from_value(value).unwrap();
        assert_eq!(parsed.command, "pwd");
    }

    #[test]
    fn parses_question_tool_args() {
        let value = json!({"question":"Use recursive search?","reason":"strategy"});
        let parsed = question_from_value(value).unwrap();
        assert_eq!(parsed.question, "Use recursive search?");
    }

    #[test]
    fn has_text_question_tool_constant() {
        assert_eq!(TEXT_QUESTION_TOOL_NAME, "ask_text_question");
    }
}
