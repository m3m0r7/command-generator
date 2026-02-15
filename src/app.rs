use anyhow::Result;

use crate::bootstrap;
use crate::cli::Cli;
use crate::interactive;
use crate::output;
use crate::postprocess;
use crate::request_engine::RequestEngine;
use crate::{command_validation, paths};

pub async fn run(cli: Cli) -> Result<()> {
    paths::ensure_dirs()?;

    let Some(mut runtime) = bootstrap::bootstrap(&cli).await? else {
        return Ok(());
    };

    let engine = RequestEngine::new(
        &cli,
        &runtime.llm,
        postprocess::default_post_processor(),
        command_validation::default_command_validator(),
    );

    if let Some(request) = cli.once.as_deref() {
        let result = engine.generate(request, &mut runtime.session, None).await?;
        output::print_generated_result(&result, cli.explanation)?;
        return Ok(());
    }

    interactive::run_interactive(&cli, &engine, &mut runtime.session).await
}
