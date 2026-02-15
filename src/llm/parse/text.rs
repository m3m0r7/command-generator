use anyhow::{Context, Result, anyhow};
use serde_json::Value;

use crate::llm::{CommandCandidate, LlmOutput};

use super::normalize::command_from_value;

pub(super) fn parse_candidate_text(raw: &str) -> Result<LlmOutput> {
    let text = strip_fences(raw.trim());
    if let Ok(candidate) = serde_json::from_str::<CommandCandidate>(&text) {
        return command_from_value(serde_json::to_value(candidate)?).map(LlmOutput::Command);
    }

    if let Some(fragment) = extract_first_json_object(&text) {
        let parsed: Value =
            serde_json::from_str(&fragment).with_context(|| "failed to parse JSON fragment")?;
        return command_from_value(parsed).map(LlmOutput::Command);
    }

    Err(anyhow!("LLM output does not contain valid command data"))
}

fn strip_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    if !(trimmed.starts_with("```") && trimmed.ends_with("```")) {
        return trimmed.to_string();
    }

    let mut lines = trimmed.lines();
    let _ = lines.next();
    let mut out = String::new();
    for line in lines {
        if line.trim() == "```" {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

fn extract_first_json_object(raw: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == '{' {
            if depth == 0 {
                start = Some(idx);
            }
            depth += 1;
            continue;
        }
        if ch == '}' {
            if depth == 0 {
                continue;
            }
            depth -= 1;
            if depth == 0
                && let Some(begin) = start
            {
                return Some(raw[begin..=idx].to_string());
            }
        }
    }
    None
}
