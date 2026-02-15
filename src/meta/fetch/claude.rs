use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use super::error::compact_error;

pub(super) async fn fetch_models_claude(key: &str) -> Result<Vec<String>> {
    let base = std::env::var("ANTHROPIC_BASE_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());
    let version =
        std::env::var("ANTHROPIC_API_VERSION").unwrap_or_else(|_| "2023-06-01".to_string());
    let endpoint = format!("{}/models", base.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .get(endpoint)
        .header("x-api-key", key)
        .header("anthropic-version", version)
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "Claude API error ({}): {}",
            status,
            compact_error(&body)
        ));
    }

    let parsed: ClaudeModelsResponse =
        serde_json::from_str(&body).with_context(|| "failed to parse Claude models response")?;
    let mut ids = parsed
        .data
        .into_iter()
        .map(|item| item.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

#[derive(Debug, Deserialize)]
struct ClaudeModelsResponse {
    #[serde(default)]
    data: Vec<ClaudeModelItem>,
}

#[derive(Debug, Deserialize)]
struct ClaudeModelItem {
    id: String,
}
