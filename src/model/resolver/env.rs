use super::super::provider_kind::ProviderKind;

pub(super) fn get_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn provider_from_model_name(model: &str) -> Option<ProviderKind> {
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
