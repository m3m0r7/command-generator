use serde::Deserialize;

pub(super) fn compact_error(body: &str) -> String {
    #[derive(Deserialize)]
    struct ApiErrorRoot {
        error: Option<ApiError>,
    }
    #[derive(Deserialize)]
    struct ApiError {
        message: Option<String>,
        #[serde(rename = "type")]
        kind: Option<String>,
        code: Option<String>,
    }

    if let Ok(payload) = serde_json::from_str::<ApiErrorRoot>(body)
        && let Some(err) = payload.error
    {
        let message = err.message.unwrap_or_else(|| "unknown error".to_string());
        let kind = err.kind.unwrap_or_else(|| "unknown".to_string());
        let code = err.code.unwrap_or_else(|| "none".to_string());
        return format!("{} (type={}, code={})", message, kind, code);
    }
    body.to_string()
}
