use anyhow::Result;
use clap::Parser;

use command_generator::app;
use command_generator::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    app::run(cli).await
}
