use anyhow::Result;

use crate::cli::Cli;
use crate::command_validation::CommandValidator;
use crate::llm::LlmOutput;
use crate::postprocess::CommandPostProcessor;
use crate::prompter::{ClarificationKind, ClarificationPrompter};
use crate::session::SessionRecord;

use super::command_handler;
use super::committer::SessionCommandCommitter;
use super::gateway::GenerationGateway;
use super::prompt_context::PromptStaticContext;
use super::question_handler;
use super::state::RuntimeState;
use super::types::HandleResult;

pub struct EngineDeps<'a> {
    pub cli: &'a Cli,
    pub gateway: &'a dyn GenerationGateway,
    pub post_processor: &'a dyn CommandPostProcessor,
    pub validator: &'a dyn CommandValidator,
}

pub async fn run(
    deps: EngineDeps<'_>,
    user_input: &str,
    session: &mut SessionRecord,
    mut prompter: Option<&mut dyn ClarificationPrompter>,
) -> Result<HandleResult> {
    let context =
        PromptStaticContext::new(deps.cli, deps.gateway.model_name(), user_input, session)?;
    let mut state = RuntimeState::new(deps.cli.max_attempts.max(1), 8);
    let committer = SessionCommandCommitter::new(deps.cli);

    while state.can_attempt_command() {
        let rendered = context.render(
            &session.uuid,
            state.clarifications().to_vec(),
            state.feedback().cloned(),
            deps.cli.explanation,
        )?;

        match deps
            .gateway
            .generate_output(&rendered.system, &rendered.user)
            .await?
        {
            LlmOutput::Command(candidate) => {
                if let Some(result) = command_handler::handle_command(
                    command_handler::CommandDeps {
                        post_processor: deps.post_processor,
                        validator: deps.validator,
                        committer: &committer,
                    },
                    command_handler::CommandInput {
                        user_input,
                        session,
                        context: &context,
                        state: &mut state,
                        candidate,
                        has_prompter: prompter.is_some(),
                    },
                )? {
                    return Ok(result);
                }
            }
            LlmOutput::QuestionYesNo(question) => {
                question_handler::handle_question(
                    ClarificationKind::YesNo,
                    question.question,
                    &mut prompter,
                    &mut state,
                )?;
            }
            LlmOutput::QuestionText(question) => {
                question_handler::handle_question(
                    ClarificationKind::Text,
                    question.question,
                    &mut prompter,
                    &mut state,
                )?;
            }
        }
    }

    Err(state.finish_error())
}
