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
