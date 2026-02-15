use anyhow::Result;

use crate::command_validation::CommandValidator;
use crate::llm::CommandCandidate;
use crate::postprocess::CommandPostProcessor;
use crate::request_engine::committer::CommandCommitter;
use crate::request_engine::guards::has_runtime_input_prompt;
use crate::request_engine::prompt_context::PromptStaticContext;
use crate::request_engine::state::RuntimeState;
use crate::request_engine::types::HandleResult;
use crate::session::SessionRecord;

pub struct CommandDeps<'a> {
    pub post_processor: &'a dyn CommandPostProcessor,
    pub validator: &'a dyn CommandValidator,
    pub committer: &'a dyn CommandCommitter,
}

pub struct CommandInput<'a> {
    pub user_input: &'a str,
    pub session: &'a mut SessionRecord,
    pub context: &'a PromptStaticContext,
    pub state: &'a mut RuntimeState,
    pub candidate: CommandCandidate,
    pub has_prompter: bool,
}

pub fn handle_command(
    deps: CommandDeps<'_>,
    input: CommandInput<'_>,
) -> Result<Option<HandleResult>> {
    let CommandInput {
        user_input,
        session,
        context,
        state,
        candidate,
        has_prompter,
    } = input;
    let CommandDeps {
        post_processor,
        validator,
        committer,
    } = deps;

    let CommandCandidate {
        command: raw_command,
        reason,
        explanations,
    } = candidate;
    let command = post_processor.process(&context.shell, raw_command)?;

    if has_prompter && state.clarifications_empty() && has_runtime_input_prompt(&command) {
        let reason = "Do not use runtime read prompts in the final command. Ask a text clarification question first via tool.".to_string();
        state.set_feedback_reason(reason);
        return Ok(None);
    }

    state.mark_command_attempt();
    let report = validator.validate(&command)?;
    if report.is_valid() {
        let result =
            committer.commit(user_input, session, command, reason, explanations, report)?;
        return Ok(Some(result));
    }

    state.set_feedback_reason(report.to_feedback_text());
    Ok(None)
}
