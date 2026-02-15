mod command_handler;
mod committer;
pub mod gateway;
pub mod guards;
mod orchestrator;
pub mod prompt_context;
mod question_handler;
pub mod state;
mod types;

use anyhow::Result;

use crate::cli::Cli;
use crate::command_validation::CommandValidator;
use crate::postprocess::CommandPostProcessor;
use crate::prompter::ClarificationPrompter;
use crate::session::SessionRecord;

pub use types::HandleResult;

pub struct RequestEngine<'a> {
    cli: &'a Cli,
    gateway: &'a dyn gateway::GenerationGateway,
    post_processor: Box<dyn CommandPostProcessor>,
    validator: Box<dyn CommandValidator>,
}

impl<'a> RequestEngine<'a> {
    pub fn new(
        cli: &'a Cli,
        gateway: &'a dyn gateway::GenerationGateway,
        post_processor: Box<dyn CommandPostProcessor>,
        validator: Box<dyn CommandValidator>,
    ) -> Self {
        Self {
            cli,
            gateway,
            post_processor,
            validator,
        }
    }

    pub async fn generate(
        &self,
        user_input: &str,
        session: &mut SessionRecord,
        prompter: Option<&mut dyn ClarificationPrompter>,
    ) -> Result<HandleResult> {
        orchestrator::run(
            orchestrator::EngineDeps {
                cli: self.cli,
                gateway: self.gateway,
                post_processor: self.post_processor.as_ref(),
                validator: self.validator.as_ref(),
            },
            user_input,
            session,
            prompter,
        )
        .await
    }
}
