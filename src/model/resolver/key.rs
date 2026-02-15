use anyhow::{Result, anyhow};

use super::super::provider_kind::ProviderKind;
use super::env::get_env;

pub(super) fn resolve_key_internal(
    provider: ProviderKind,
    override_key: Option<&str>,
) -> Result<String> {
    if let Some(key) = override_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    match provider {
        ProviderKind::OpenAI => get_env("OPENAI_API_KEY"),
        ProviderKind::Gemini => get_env("GEMINI_API_KEY").or_else(|| get_env("GOOGLE_API_KEY")),
        ProviderKind::Claude => get_env("ANTHROPIC_API_KEY"),
    }
    .ok_or_else(|| anyhow!("API key not found for provider '{}'", provider.as_str()))
}
