use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use super::error::compact_error;

pub(super) async fn fetch_models_openai(key: &str) -> Result<Vec<String>> {
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client.get(url).bearer_auth(key).send().await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "OpenAI API error ({}): {}",
            status,
            compact_error(&body)
        ));
    }

    let parsed: OpenAIModelsResponse =
        serde_json::from_str(&body).with_context(|| "failed to parse OpenAI models response")?;

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
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelItem {
    id: String,
}
