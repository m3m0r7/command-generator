mod default;

use anyhow::Result;

use crate::model::ProviderKind;
use crate::session::SessionRecord;

pub use default::DefaultRuntimeResolver;

pub trait RuntimeResolver: Send + Sync {
    fn resolve_provider_for_model_listing(
        &self,
        model_arg: Option<&str>,
        resumed_session: Option<&SessionRecord>,
    ) -> Result<ProviderKind>;

    fn resolve_provider(
        &self,
        model_arg: Option<&str>,
        key_arg: Option<&str>,
        resumed_session: Option<&SessionRecord>,
    ) -> Result<ProviderKind>;

    fn resolve_model_name(
        &self,
        provider: ProviderKind,
        model_arg: Option<&str>,
        resumed_session: Option<&SessionRecord>,
    ) -> Result<String>;
}

pub fn default_runtime_resolver() -> Box<dyn RuntimeResolver> {
    Box::new(DefaultRuntimeResolver)
}
