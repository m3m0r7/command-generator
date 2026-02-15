mod api_error;
mod parse;
mod providers;
mod tools;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::ProviderKind;

pub const COMMAND_TOOL_NAME: &str = "deliver_command";
pub const QUESTION_TOOL_NAME: &str = "ask_yes_no_question";
pub const TEXT_QUESTION_TOOL_NAME: &str = "ask_text_question";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExplanationItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCandidate {
    pub command: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub explanations: Vec<CommandExplanationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub question: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum LlmOutput {
    Command(CommandCandidate),
    QuestionYesNo(ClarificationQuestion),
    QuestionText(ClarificationQuestion),
}

#[derive(Debug, Clone)]
pub struct LlmClient {
    provider: ProviderKind,
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(
        provider: ProviderKind,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            api_key: api_key.into(),
            model: model.into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }

    pub async fn generate_output(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmOutput> {
        match self.provider {
            ProviderKind::OpenAI => self.call_openai(system_prompt, user_prompt).await,
            ProviderKind::Gemini => self.call_gemini(system_prompt, user_prompt).await,
            ProviderKind::Claude => self.call_claude(system_prompt, user_prompt).await,
        }
    }
}
