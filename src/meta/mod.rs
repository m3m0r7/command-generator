mod cache;
mod fetch;

use anyhow::{Result, anyhow};

use crate::model::ProviderKind;

pub fn get_last_using_model(provider: ProviderKind) -> Result<Option<String>> {
    let meta = cache::read_meta()?;
    let prefix = format!("{}:", provider.as_str());
    let model = meta
        .last_using_model
        .and_then(|value| value.strip_prefix(&prefix).map(|item| item.to_string()));
    Ok(model)
}

pub fn set_last_using_model(provider: ProviderKind, model: &str) -> Result<()> {
    let mut meta = cache::read_meta()?;
    meta.last_using_model = Some(format!("{}:{}", provider.as_str(), model));
    cache::write_meta(&meta)?;
    Ok(())
}

pub async fn get_models(provider: ProviderKind, key: Option<&str>) -> Result<Vec<String>> {
    let mut meta = cache::read_meta()?;
    let prefix = provider.as_str();
    let cached = cache::models_for_provider(&meta.models, prefix);

    if !cached.is_empty() && !cache::is_expired(&meta) {
        return Ok(cached);
    }
    if key.is_none() && !cached.is_empty() {
        return Ok(cached);
    }

    let key = key.ok_or_else(|| anyhow!("API key is required to fetch models"))?;
    let fetched = fetch::fetch_models(provider, key).await?;
    cache::update_provider_models(&mut meta.models, prefix, &fetched);
    meta.last_fetched_model_datetime = Some(cache::now_unix());
    cache::write_meta(&meta)?;
    Ok(cache::models_for_provider(&meta.models, prefix))
}
