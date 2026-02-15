use crate::validation::types::{CommandHead, SegmentToken};

pub(super) fn locate_head_token(tokens: &[SegmentToken]) -> Option<CommandHead> {
    if tokens.is_empty() {
        return None;
    }

    let mut index = 0usize;
    while index < tokens.len() && looks_like_assignment(&tokens[index].cooked) {
        index += 1;
    }

    let mut prefixed_builtin = false;
    let mut prefixed_command = false;
    while index < tokens.len() {
        let raw = tokens[index].raw.trim();
        if raw.is_empty() {
            index += 1;
            continue;
        }
        let token = tokens[index]
            .cooked
            .trim()
            .trim_start_matches('(')
            .trim_start_matches('{')
            .trim()
            .to_string();
        if token.is_empty() {
            index += 1;
            continue;
        }
        let lowered = token.to_lowercase();
        if lowered == "builtin" {
            prefixed_builtin = true;
            index += 1;
            continue;
        }
        if lowered == "command" {
            prefixed_command = true;
            index += 1;
            continue;
        }
        if is_wrapper_command(&lowered) {
            index += 1;
            continue;
        }
        return Some(CommandHead {
            name: token,
            prefixed_builtin,
            prefixed_command,
            prefixed_backslash: tokens[index].raw.trim_start().starts_with('\\'),
            token_index: index,
        });
    }
    None
}

fn looks_like_assignment(token: &str) -> bool {
    if token.starts_with('-') || !token.contains('=') {
        return false;
    }
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    if name.is_empty() {
        return false;
    }
    name.chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn is_wrapper_command(token: &str) -> bool {
    matches!(token, "sudo" | "env" | "nohup" | "time" | "nice")
}
