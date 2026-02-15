use anyhow::Result;

use crate::bootstrap::resolver::RuntimeResolver;
use crate::model::{self, ProviderKind, ProviderSelection};
use crate::session::SessionRecord;

pub struct DefaultRuntimeResolver;

impl RuntimeResolver for DefaultRuntimeResolver {
    fn resolve_provider_for_model_listing(
        &self,
        model_arg: Option<&str>,
        resumed_session: Option<&SessionRecord>,
    ) -> Result<ProviderKind> {
        if let Some(model_arg) = model_arg {
            let selection = model::resolve_provider_selection(Some(model_arg), None, true)?;
            return Ok(selection.provider);
        }
        if let Some(session) = resumed_session
            && let Some(provider) = model::provider_from_name(&session.provider)
        {
            return Ok(provider);
        }
        let selection = model::resolve_provider_selection(None, None, true)?;
        Ok(selection.provider)
    }

    fn resolve_provider(
        &self,
        model_arg: Option<&str>,
        key_arg: Option<&str>,
        resumed_session: Option<&SessionRecord>,
    ) -> Result<ProviderKind> {
        if let Some(model_arg) = model_arg {
            let selection = model::resolve_provider_selection(Some(model_arg), key_arg, false)?;
            return Ok(selection.provider);
        }
        if let Some(session) = resumed_session
            && let Some(provider) = model::provider_from_name(&session.provider)
        {
            return Ok(provider);
        }
        let selection = model::resolve_provider_selection(None, key_arg, false)?;
        Ok(selection.provider)
    }

    fn resolve_model_name(
        &self,
        provider: ProviderKind,
        model_arg: Option<&str>,
        resumed_session: Option<&SessionRecord>,
    ) -> Result<String> {
        if let Some(model_arg) = model_arg {
            let ProviderSelection {
                requested_model, ..
            } = model::resolve_provider_selection(Some(model_arg), None, true)?;
            return Ok(
                requested_model.unwrap_or_else(|| model::default_model(provider).to_string())
            );
        }

        if let Some(session) = resumed_session {
            let value = session.model.trim();
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }

        if let Some(last) = crate::meta::get_last_using_model(provider)? {
            return Ok(last);
        }
        Ok(model::default_model(provider).to_string())
    }
}
