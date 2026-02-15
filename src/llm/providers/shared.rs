use anyhow::{Context, Result, anyhow};
use serde_json::Value;

use crate::llm::parse::{command_from_value, question_from_value};
use crate::llm::{COMMAND_TOOL_NAME, LlmOutput, QUESTION_TOOL_NAME, TEXT_QUESTION_TOOL_NAME};

pub(super) fn map_tool_output(provider: &str, name: &str, args: Value) -> Result<LlmOutput> {
    match name {
        COMMAND_TOOL_NAME => command_from_value(args).map(LlmOutput::Command),
        QUESTION_TOOL_NAME => question_from_value(args).map(LlmOutput::QuestionYesNo),
        TEXT_QUESTION_TOOL_NAME => question_from_value(args).map(LlmOutput::QuestionText),
        other => Err(anyhow!(
            "unsupported tool call returned from {}: {}",
            provider,
            other
        )),
    }
}

pub(super) fn parse_arguments(arguments: &str, context: &str) -> Result<Value> {
    serde_json::from_str(arguments)
        .with_context(|| format!("failed to parse {} tool arguments", context))
}
