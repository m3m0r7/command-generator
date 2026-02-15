use anyhow::Result;

use crate::bootstrap::resolver::RuntimeResolver;
use crate::cli::Cli;
use crate::meta;
use crate::model;
use crate::session::SessionRecord;

pub async fn try_print_model_list(
    cli: &Cli,
    resumed_session: Option<&SessionRecord>,
    resolver: &dyn RuntimeResolver,
) -> Result<bool> {
    if !cli.show_models_list {
        return Ok(false);
    }

    let provider =
        resolver.resolve_provider_for_model_listing(cli.model.as_deref(), resumed_session)?;
    let key = cli
        .key
        .clone()
        .or_else(|| model::resolve_key(provider, None).ok());
    let models = meta::get_models(provider, key.as_deref()).await?;
    for model in models {
        println!("{model}");
    }
    Ok(true)
}
