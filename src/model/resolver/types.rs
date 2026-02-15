use super::super::provider_kind::ProviderKind;

#[derive(Debug, Clone)]
pub struct ProviderSelection {
    pub provider: ProviderKind,
    pub requested_model: Option<String>,
}
