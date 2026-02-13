use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::ProviderKind;
use crate::paths;

const TTL_SECONDS: i64 = 60 * 60 * 24;

#[derive(Debug, Serialize, Deserialize, Default)]
struct MetaCache {
    #[serde(rename = "lastUsingModel")]
    last_using_model: Option<String>,
    #[serde(rename = "lastFetchedModelDateTime", alias = "lastUpdatedTime")]
    last_fetched_model_datetime: Option<i64>,
    #[serde(default)]
    models: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelItem {
    id: String,
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

#[derive(Debug, Deserialize)]
struct ClaudeModelsResponse {
    #[serde(default)]
    data: Vec<ClaudeModelItem>,
}

#[derive(Debug, Deserialize)]
struct ClaudeModelItem {
    id: String,
}

pub fn get_last_using_model(provider: ProviderKind) -> Result<Option<String>> {
    let meta = read_meta()?;
    let prefix = format!("{}:", provider.as_str());
    let model = meta
        .last_using_model
        .and_then(|value| value.strip_prefix(&prefix).map(|item| item.to_string()));
    Ok(model)
}

pub fn set_last_using_model(provider: ProviderKind, model: &str) -> Result<()> {
    let mut meta = read_meta()?;
    meta.last_using_model = Some(format!("{}:{}", provider.as_str(), model));
    write_meta(&meta)?;
    Ok(())
}

pub async fn get_models(provider: ProviderKind, key: Option<&str>) -> Result<Vec<String>> {
    let mut meta = read_meta()?;
    let prefix = provider_prefix(provider);
    let cached = models_for_provider(&meta.models, prefix);

    if !cached.is_empty() && !is_expired(&meta) {
        return Ok(cached);
    }
    if key.is_none() && !cached.is_empty() {
        return Ok(cached);
    }

    let key = key.ok_or_else(|| anyhow!("API key is required to fetch models"))?;
    let fetched = fetch_models(provider, key).await?;
    update_provider_models(&mut meta.models, prefix, &fetched);
    meta.last_fetched_model_datetime = Some(now_unix());
    write_meta(&meta)?;
    Ok(models_for_provider(&meta.models, prefix))
}

async fn fetch_models(provider: ProviderKind, key: &str) -> Result<Vec<String>> {
    match provider {
        ProviderKind::OpenAI => fetch_models_openai(key).await,
        ProviderKind::Gemini => fetch_models_gemini(key).await,
        ProviderKind::Claude => fetch_models_claude(key).await,
    }
}

async fn fetch_models_openai(key: &str) -> Result<Vec<String>> {
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

async fn fetch_models_gemini(key: &str) -> Result<Vec<String>> {
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

async fn fetch_models_claude(key: &str) -> Result<Vec<String>> {
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

fn compact_error(body: &str) -> String {
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

fn provider_prefix(provider: ProviderKind) -> &'static str {
    provider.as_str()
}

fn models_for_provider(models: &[String], prefix: &str) -> Vec<String> {
    let needle = format!("{}:", prefix);
    models
        .iter()
        .filter(|entry| entry.starts_with(&needle))
        .cloned()
        .collect::<Vec<_>>()
}

fn update_provider_models(models: &mut Vec<String>, prefix: &str, fetched_models: &[String]) {
    let needle = format!("{}:", prefix);
    models.retain(|entry| !entry.starts_with(&needle));
    models.extend(
        fetched_models
            .iter()
            .map(|model| format!("{}:{}", prefix, model)),
    );
    models.sort();
    models.dedup();
}

fn read_meta() -> Result<MetaCache> {
    let path = meta_path();
    if !path.exists() {
        return Ok(MetaCache::default());
    }
    let content = fs::read_to_string(path).with_context(|| "failed to read meta cache")?;
    let meta: MetaCache =
        serde_json::from_str(&content).with_context(|| "failed to parse meta cache JSON")?;
    Ok(meta)
}

fn write_meta(meta: &MetaCache) -> Result<()> {
    let path = meta_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| "failed to create cache directory")?;
    }
    let content = serde_json::to_string_pretty(meta)?;
    fs::write(path, content).with_context(|| "failed to write meta cache")?;
    Ok(())
}

fn meta_path() -> std::path::PathBuf {
    paths::cache_dir().join("meta.json")
}

fn is_expired(meta: &MetaCache) -> bool {
    let now = now_unix();
    match meta.last_fetched_model_datetime {
        Some(last) => now.saturating_sub(last) > TTL_SECONDS,
        None => true,
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
