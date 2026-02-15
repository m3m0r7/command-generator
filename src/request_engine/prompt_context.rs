use anyhow::Result;

use crate::cli::Cli;
use crate::history;
use crate::prompt::{PromptClarification, PromptInput, PromptTurn, RenderedPrompt};
use crate::session::{self, SessionRecord};

pub struct PromptStaticContext {
    os: String,
    pub shell: String,
    user_input: String,
    model: String,
    shell_history: Vec<String>,
    generated_history: Vec<String>,
    turns: Vec<PromptTurn>,
}

impl PromptStaticContext {
    pub fn new(
        cli: &Cli,
        model_name: &str,
        user_input: &str,
        session: &SessionRecord,
    ) -> Result<Self> {
        let shell_history = history::load_shell_history(cli.history_lines);
        let generated_history = session::list_recent_commands(cli.generated_history_lines)?;
        let turns = session
            .recent_turns(cli.context_turns)
            .into_iter()
            .map(|turn| PromptTurn {
                user_input: turn.user_input,
                command: turn.command,
            })
            .collect::<Vec<_>>();

        Ok(Self {
            os: std::env::consts::OS.to_string(),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string()),
            user_input: user_input.to_string(),
            model: model_name.to_string(),
            shell_history,
            generated_history,
            turns,
        })
    }

    pub fn render(
        &self,
        session_uuid: &str,
        clarifications: Vec<PromptClarification>,
        feedback: Option<String>,
        explanation_mode: bool,
    ) -> Result<RenderedPrompt> {
        crate::prompt::render(&PromptInput {
            os: self.os.clone(),
            shell: self.shell.clone(),
            session_uuid: session_uuid.to_string(),
            model: self.model.clone(),
            command_tool_name: crate::llm::COMMAND_TOOL_NAME.to_string(),
            question_tool_name: crate::llm::QUESTION_TOOL_NAME.to_string(),
            text_question_tool_name: crate::llm::TEXT_QUESTION_TOOL_NAME.to_string(),
            user_input: self.user_input.clone(),
            shell_history: self.shell_history.clone(),
            generated_history: self.generated_history.clone(),
            turns: self.turns.clone(),
            clarifications,
            feedback,
            explanation_mode,
        })
    }
}
