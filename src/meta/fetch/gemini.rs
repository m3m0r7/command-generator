use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use super::error::compact_error;

pub(super) async fn fetch_models_gemini(key: &str) -> Result<Vec<String>> {
    let base = std::env::var("GEMINI_BASE_URL")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());
    let endpoint = format!("{}/models", base.trim_end_matches('/'));
    let mut url = reqwest::Url::parse(&endpoint)
        .with_context(|| "failed to parse Gemini models endpoint URL")?;
    url.query_pairs_mut().append_pair("key", key);

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "Gemini API error ({}): {}",
            status,
            compact_error(&body)
        ));
    }

    let parsed: GeminiModelsResponse =
        serde_json::from_str(&body).with_context(|| "failed to parse Gemini models response")?;
    let mut ids = parsed
        .models
        .into_iter()
        .map(|item| item.name.trim().trim_start_matches("models/").to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

#[derive(Debug, Deserialize)]
struct GeminiModelsResponse {
    #[serde(default)]
    models: Vec<GeminiModelItem>,
}

#[derive(Debug, Deserialize)]
struct GeminiModelItem {
    name: String,
}
