use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenAI,
    Gemini,
    Claude,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "openai",
            ProviderKind::Gemini => "gemini",
            ProviderKind::Claude => "claude",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderSelection {
    pub provider: ProviderKind,
    pub requested_model: Option<String>,
}

pub fn default_model(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::OpenAI => "gpt-5.2",
        ProviderKind::Gemini => "gemini-2.5-flash",
        ProviderKind::Claude => "claude-sonnet-4-5",
    }
}

pub fn provider_from_name(name: &str) -> Option<ProviderKind> {
    match name.trim().to_lowercase().as_str() {
        "openai" => Some(ProviderKind::OpenAI),
        "gemini" | "google" => Some(ProviderKind::Gemini),
        "claude" | "anthropic" => Some(ProviderKind::Claude),
        _ => None,
    }
}

pub fn resolve_provider_selection(
    model_arg: Option<&str>,
    override_key: Option<&str>,
    allow_no_key: bool,
) -> Result<ProviderSelection> {
    match model_arg {
        Some(model) => parse_model_arg(model, override_key, allow_no_key),
        None => default_provider_selection(override_key, allow_no_key),
    }
}

pub fn resolve_key(provider: ProviderKind, override_key: Option<&str>) -> Result<String> {
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

fn parse_model_arg(
    model_arg: &str,
    override_key: Option<&str>,
    allow_no_key: bool,
) -> Result<ProviderSelection> {
    let raw = model_arg.trim();
    if raw.is_empty() {
        return Err(anyhow!("model argument is empty"));
    }

    if let Some((provider_part, model_part)) = raw.split_once(':') {
        let provider = provider_from_name(provider_part)
            .ok_or_else(|| anyhow!("unknown provider '{}'", provider_part))?;
        let requested_model = if model_part.trim().is_empty() {
            None
        } else {
            Some(model_part.trim().to_string())
        };
        return Ok(ProviderSelection {
            provider,
            requested_model,
        });
    }

    if let Some(provider) = provider_from_name(raw) {
        return Ok(ProviderSelection {
            provider,
            requested_model: None,
        });
    }

    if let Some(provider) = provider_from_model_name(raw) {
        return Ok(ProviderSelection {
            provider,
            requested_model: Some(raw.to_string()),
        });
    }

    let provider = infer_default_provider(override_key, allow_no_key)?;
    Ok(ProviderSelection {
        provider,
        requested_model: Some(raw.to_string()),
    })
}

fn default_provider_selection(
    override_key: Option<&str>,
    allow_no_key: bool,
) -> Result<ProviderSelection> {
    let provider = infer_default_provider(override_key, allow_no_key)?;
    Ok(ProviderSelection {
        provider,
        requested_model: None,
    })
}

fn infer_default_provider(override_key: Option<&str>, allow_no_key: bool) -> Result<ProviderKind> {
    if get_env("OPENAI_API_KEY").is_some() {
        return Ok(ProviderKind::OpenAI);
    }
    if get_env("GEMINI_API_KEY").is_some() || get_env("GOOGLE_API_KEY").is_some() {
        return Ok(ProviderKind::Gemini);
    }
    if get_env("ANTHROPIC_API_KEY").is_some() {
        return Ok(ProviderKind::Claude);
    }

    if override_key.is_some() {
        return Ok(ProviderKind::OpenAI);
    }

    if allow_no_key {
        return Ok(ProviderKind::OpenAI);
    }

    Err(anyhow!(
        "no API key found (checked OPENAI_API_KEY, GEMINI_API_KEY/GOOGLE_API_KEY, ANTHROPIC_API_KEY, and --key)"
    ))
}

fn provider_from_model_name(model: &str) -> Option<ProviderKind> {
    let lowered = model.trim().to_lowercase();
    if lowered.is_empty() {
        return None;
    }
    if lowered.starts_with("gpt-")
        || lowered.starts_with("o1")
        || lowered.starts_with("o3")
        || lowered.starts_with("o4")
    {
        return Some(ProviderKind::OpenAI);
    }
    if lowered.starts_with("gemini") {
        return Some(ProviderKind::Gemini);
    }
    if lowered.starts_with("claude") {
        return Some(ProviderKind::Claude);
    }
    None
}

fn get_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_and_model_pair() {
        let selection = parse_model_arg("openai:gpt-5", None, true).unwrap();
        assert_eq!(selection.provider, ProviderKind::OpenAI);
        assert_eq!(selection.requested_model.as_deref(), Some("gpt-5"));
    }

    #[test]
    fn parses_provider_only() {
        let selection = parse_model_arg("openai", None, true).unwrap();
        assert_eq!(selection.provider, ProviderKind::OpenAI);
        assert!(selection.requested_model.is_none());
    }

    #[test]
    fn parses_gemini_provider_and_model_pair() {
        let selection = parse_model_arg("gemini:gemini-2.5-flash", None, true).unwrap();
        assert_eq!(selection.provider, ProviderKind::Gemini);
        assert_eq!(
            selection.requested_model.as_deref(),
            Some("gemini-2.5-flash")
        );
    }

    #[test]
    fn parses_claude_provider_and_model_pair() {
        let selection = parse_model_arg("claude:claude-sonnet-4-5", None, true).unwrap();
        assert_eq!(selection.provider, ProviderKind::Claude);
        assert_eq!(
            selection.requested_model.as_deref(),
            Some("claude-sonnet-4-5")
        );
    }

    #[test]
    fn treats_plain_model_as_default_provider() {
        let selection = parse_model_arg("gpt-5.2", Some("dummy"), true).unwrap();
        assert_eq!(selection.provider, ProviderKind::OpenAI);
        assert_eq!(selection.requested_model.as_deref(), Some("gpt-5.2"));
    }

    #[test]
    fn infers_provider_from_model_name() {
        let gemini = parse_model_arg("gemini-2.5-flash", None, true).unwrap();
        assert_eq!(gemini.provider, ProviderKind::Gemini);
        let claude = parse_model_arg("claude-sonnet-4-5", None, true).unwrap();
        assert_eq!(claude.provider, ProviderKind::Claude);
    }
}
