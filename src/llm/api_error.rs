use serde::Deserialize;

pub(crate) fn extract_api_error(body: &str) -> String {
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
