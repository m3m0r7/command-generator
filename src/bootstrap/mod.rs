mod model_list;
mod resolver;
mod runtime_setup;

use anyhow::Result;

use crate::cli::Cli;
use crate::llm::LlmClient;
use crate::session::{self, SessionRecord};

pub struct BootstrappedRuntime {
    pub llm: LlmClient,
    pub session: SessionRecord,
}

pub async fn bootstrap(cli: &Cli) -> Result<Option<BootstrappedRuntime>> {
    let resolver = resolver::default_runtime_resolver();
    let resumed_session = match cli.resume.as_deref() {
        Some(uuid) => Some(session::load_session(uuid)?),
        None => None,
    };

    if model_list::try_print_model_list(cli, resumed_session.as_ref(), resolver.as_ref()).await? {
        return Ok(None);
    }

    let runtime = runtime_setup::prepare_runtime(cli, resumed_session, resolver.as_ref())?;
    Ok(Some(runtime))
}
