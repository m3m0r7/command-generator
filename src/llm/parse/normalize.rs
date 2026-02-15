use anyhow::{Context, Result, anyhow};
use serde_json::Value;

use crate::llm::{ClarificationQuestion, CommandCandidate};

pub(super) fn command_from_value(value: Value) -> Result<CommandCandidate> {
    let mut candidate: CommandCandidate =
        serde_json::from_value(value).with_context(|| "failed to parse command tool arguments")?;
    candidate.command = candidate.command.trim().to_string();
    candidate.reason = candidate.reason.trim().to_string();
    for item in &mut candidate.explanations {
        item.kind = item.kind.trim().to_string();
        item.value = item.value.trim().to_string();
        item.explanation = item.explanation.trim().to_string();
    }
    candidate.explanations.retain(|item| {
        !item.kind.is_empty() && !item.value.is_empty() && !item.explanation.is_empty()
    });
    if candidate.command.is_empty() {
        return Err(anyhow!("generated command is empty"));
    }
    Ok(candidate)
}

pub(super) fn question_from_value(value: Value) -> Result<ClarificationQuestion> {
    let mut question: ClarificationQuestion =
        serde_json::from_value(value).with_context(|| "failed to parse question tool arguments")?;
    question.question = question.question.trim().to_string();
    question.reason = question.reason.trim().to_string();
    if question.question.is_empty() {
        return Err(anyhow!("clarification question is empty"));
    }
    Ok(question)
}
