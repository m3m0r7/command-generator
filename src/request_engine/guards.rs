pub fn has_runtime_input_prompt(command: &str) -> bool {
    let lowered = command.to_lowercase();
    lowered.contains("read ")
        || lowered.contains("read\t")
        || lowered.contains("vared ")
        || lowered.contains("vared\t")
}

pub fn normalize_question_text(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_question_text_for_duplicate_detection() {
        assert_eq!(
            normalize_question_text("  Use recursive  search? "),
            "use recursive search?"
        );
    }

    #[test]
    fn detects_runtime_input_prompt() {
        assert!(has_runtime_input_prompt("read -r x"));
        assert!(has_runtime_input_prompt("vared target"));
        assert!(!has_runtime_input_prompt("echo hello"));
    }
}
