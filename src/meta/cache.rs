use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;

const TTL_SECONDS: i64 = 60 * 60 * 24;

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct MetaCache {
    #[serde(rename = "lastUsingModel")]
    pub(crate) last_using_model: Option<String>,
    #[serde(rename = "lastFetchedModelDateTime", alias = "lastUpdatedTime")]
    pub(crate) last_fetched_model_datetime: Option<i64>,
    #[serde(default)]
    pub(crate) models: Vec<String>,
}

pub(crate) fn read_meta() -> Result<MetaCache> {
    let path = meta_path();
    if !path.exists() {
        return Ok(MetaCache::default());
    }
    let content = fs::read_to_string(path).with_context(|| "failed to read meta cache")?;
    let meta: MetaCache =
        serde_json::from_str(&content).with_context(|| "failed to parse meta cache JSON")?;
    Ok(meta)
}

pub(crate) fn write_meta(meta: &MetaCache) -> Result<()> {
    let path = meta_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| "failed to create cache directory")?;
    }
    let content = serde_json::to_string_pretty(meta)?;
    fs::write(path, content).with_context(|| "failed to write meta cache")?;
    Ok(())
}

pub(crate) fn models_for_provider(models: &[String], prefix: &str) -> Vec<String> {
    let needle = format!("{}:", prefix);
    models
        .iter()
        .filter(|entry| entry.starts_with(&needle))
        .cloned()
        .collect::<Vec<_>>()
}

pub(crate) fn update_provider_models(
    models: &mut Vec<String>,
    prefix: &str,
    fetched_models: &[String],
) {
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

pub(crate) fn is_expired(meta: &MetaCache) -> bool {
    let now = now_unix();
    match meta.last_fetched_model_datetime {
        Some(last) => now.saturating_sub(last) > TTL_SECONDS,
        None => true,
    }
}

pub(crate) fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn meta_path() -> std::path::PathBuf {
    paths::cache_dir().join("meta.json")
}
