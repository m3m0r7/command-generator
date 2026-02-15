mod env;
mod key;
mod selection;
mod types;

use anyhow::Result;

use super::provider_kind::ProviderKind;

pub use types::ProviderSelection;

pub fn resolve_provider_selection(
    model_arg: Option<&str>,
    override_key: Option<&str>,
    allow_no_key: bool,
) -> Result<ProviderSelection> {
    selection::resolve_provider_selection_internal(model_arg, override_key, allow_no_key)
}

pub fn resolve_key(provider: ProviderKind, override_key: Option<&str>) -> Result<String> {
    key::resolve_key_internal(provider, override_key)
}
