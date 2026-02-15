use anyhow::Result;

use crate::bootstrap::resolver::RuntimeResolver;
use crate::cli::Cli;
use crate::llm::LlmClient;
use crate::meta;
use crate::model;
use crate::session::{self, SessionRecord};

use super::BootstrappedRuntime;

pub fn prepare_runtime(
    cli: &Cli,
    resumed_session: Option<SessionRecord>,
    resolver: &dyn RuntimeResolver,
) -> Result<BootstrappedRuntime> {
    let provider = resolver.resolve_provider(
        cli.model.as_deref(),
        cli.key.as_deref(),
        resumed_session.as_ref(),
    )?;
    let model_name =
        resolver.resolve_model_name(provider, cli.model.as_deref(), resumed_session.as_ref())?;
    let api_key = model::resolve_key(provider, cli.key.as_deref())?;

    meta::set_last_using_model(provider, &model_name)?;

    let llm = LlmClient::new(provider, api_key, model_name.clone());
    let mut active_session =
        resumed_session.unwrap_or_else(|| SessionRecord::new(provider, &model_name));
    active_session.provider = provider.as_str().to_string();
    active_session.model = model_name.clone();
    session::save_session(&active_session)?;
    eprintln!(
        "Session UUID: {} (resume with: command-generator --resume {})",
        active_session.uuid, active_session.uuid
    );

    Ok(BootstrappedRuntime {
        llm,
        session: active_session,
    })
}
