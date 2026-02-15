mod normalize;
mod text;

use anyhow::Result;
use serde_json::Value;

use super::{ClarificationQuestion, CommandCandidate, LlmOutput};

pub(crate) fn command_from_value(value: Value) -> Result<CommandCandidate> {
    normalize::command_from_value(value)
}

pub(crate) fn question_from_value(value: Value) -> Result<ClarificationQuestion> {
    normalize::question_from_value(value)
}

pub(crate) fn parse_candidate_text(raw: &str) -> Result<LlmOutput> {
    text::parse_candidate_text(raw)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::llm::TEXT_QUESTION_TOOL_NAME;

    #[test]
    fn parses_plain_json_as_command() {
        let parsed = parse_candidate_text(r#"{"command":"pwd","reason":"show cwd"}"#).unwrap();
        let LlmOutput::Command(command) = parsed else {
            panic!("expected command output");
        };
        assert_eq!(command.command, "pwd");
    }

    #[test]
    fn parses_fenced_json_as_command() {
        let parsed =
            parse_candidate_text("```json\n{\"command\":\"ls\",\"reason\":\"list\"}\n```").unwrap();
        let LlmOutput::Command(command) = parsed else {
            panic!("expected command output");
        };
        assert_eq!(command.command, "ls");
    }

    #[test]
    fn parses_command_tool_args() {
        let value = json!({"command":"pwd","reason":"cwd","explanations":[{"type":"command","value":"pwd","explanation":"show cwd"}]});
        let parsed = command_from_value(value).unwrap();
        assert_eq!(parsed.command, "pwd");
        assert_eq!(parsed.explanations.len(), 1);
    }

    #[test]
    fn parses_question_tool_args() {
        let value = json!({"question":"Use recursive search?","reason":"strategy"});
        let parsed = question_from_value(value).unwrap();
        assert_eq!(parsed.question, "Use recursive search?");
    }

    #[test]
    fn has_text_question_tool_constant() {
        assert_eq!(TEXT_QUESTION_TOOL_NAME, "ask_text_question");
    }
}
