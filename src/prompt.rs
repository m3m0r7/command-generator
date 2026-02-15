use anyhow::{Context, Result};
use serde::Serialize;
use tera::{Context as TeraContext, Tera};

#[derive(Debug, Clone, Serialize)]
pub struct PromptTurn {
    pub user_input: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptClarification {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptInput {
    pub os: String,
    pub shell: String,
    pub session_uuid: String,
    pub model: String,
    pub command_tool_name: String,
    pub question_tool_name: String,
    pub text_question_tool_name: String,
    pub user_input: String,
    pub shell_history: Vec<String>,
    pub generated_history: Vec<String>,
    pub turns: Vec<PromptTurn>,
    pub clarifications: Vec<PromptClarification>,
    pub feedback: Option<String>,
    pub explanation_mode: bool,
}

pub struct RenderedPrompt {
    pub system: String,
    pub user: String,
}

const SYSTEM_PROMPT_TEMPLATE: &str = include_str!("prompts/system_prompt.tera");
const USER_PROMPT_TEMPLATE: &str = include_str!("prompts/user_prompt.tera");

pub fn render(input: &PromptInput) -> Result<RenderedPrompt> {
    let mut context = TeraContext::new();
    context.insert("command_tool_name", &input.command_tool_name);
    context.insert("question_tool_name", &input.question_tool_name);
    context.insert("text_question_tool_name", &input.text_question_tool_name);
    context.insert("explanation_mode", &input.explanation_mode);

    let system = Tera::one_off(SYSTEM_PROMPT_TEMPLATE, &context, false)
        .with_context(|| "failed to render system prompt")?;

    context.insert("os", &input.os);
    context.insert("shell", &input.shell);
    context.insert("session_uuid", &input.session_uuid);
    context.insert("model", &input.model);
    context.insert("text_question_tool_name", &input.text_question_tool_name);
    context.insert("user_input", &input.user_input);
    context.insert("shell_history", &input.shell_history);
    context.insert("generated_history", &input.generated_history);
    context.insert("turns", &input.turns);
    context.insert("clarifications", &input.clarifications);
    context.insert("feedback", &input.feedback);
    context.insert("explanation_mode", &input.explanation_mode);

    let user = Tera::one_off(USER_PROMPT_TEMPLATE, &context, false)
        .with_context(|| "failed to render user prompt")?;

    Ok(RenderedPrompt { system, user })
}
