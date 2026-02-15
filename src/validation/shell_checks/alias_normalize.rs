use anyhow::Result;

use super::detect::is_alias;
use crate::validation::parser::{locate_head_token, split_segment_ranges, tokenize_segment};

pub(super) fn normalize_alias_prefixes(shell: &str, command: &str) -> Result<String> {
    let ranges = split_segment_ranges(command);
    if ranges.is_empty() {
        return Ok(command.to_string());
    }

    let mut out = String::new();
    let mut cursor = 0usize;
    for (start, end) in ranges {
        if cursor < start {
            out.push_str(&command[cursor..start]);
        }
        let raw_segment = &command[start..end];
        out.push_str(&normalize_alias_prefix_segment(shell, raw_segment)?);
        cursor = end;
    }
    if cursor < command.len() {
        out.push_str(&command[cursor..]);
    }
    Ok(out)
}

fn normalize_alias_prefix_segment(shell: &str, segment: &str) -> Result<String> {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return Ok(segment.to_string());
    }

    let leading = segment.len() - segment.trim_start().len();
    let trailing = segment.len() - segment.trim_end().len();
    let core_end = segment.len().saturating_sub(trailing);
    let core = &segment[leading..core_end];
    let tokens = tokenize_segment(core);
    let Some(head) = locate_head_token(&tokens) else {
        return Ok(segment.to_string());
    };
    if head.prefixed_builtin || head.prefixed_command || head.prefixed_backslash {
        return Ok(segment.to_string());
    }
    if !is_alias(shell, &head.name)? {
        return Ok(segment.to_string());
    }

    let prefix = if is_known_shell_builtin(&head.name) {
        "builtin "
    } else {
        "\\"
    };
    let Some(token) = tokens.get(head.token_index) else {
        return Ok(segment.to_string());
    };
    let mut normalized = String::new();
    normalized.push_str(&segment[..leading]);
    normalized.push_str(&core[..token.start]);
    normalized.push_str(prefix);
    normalized.push_str(&core[token.start..]);
    normalized.push_str(&segment[core_end..]);
    Ok(normalized)
}

fn is_known_shell_builtin(name: &str) -> bool {
    matches!(
        name,
        "alias"
            | "bg"
            | "bind"
            | "break"
            | "builtin"
            | "cd"
            | "command"
            | "continue"
            | "dirs"
            | "echo"
            | "eval"
            | "exec"
            | "exit"
            | "export"
            | "false"
            | "fc"
            | "fg"
            | "getopts"
            | "hash"
            | "history"
            | "jobs"
            | "kill"
            | "let"
            | "local"
            | "logout"
            | "popd"
            | "printf"
            | "pushd"
            | "pwd"
            | "read"
            | "readonly"
            | "return"
            | "set"
            | "shift"
            | "source"
            | "test"
            | "times"
            | "trap"
            | "true"
            | "type"
            | "typeset"
            | "ulimit"
            | "umask"
            | "unalias"
            | "unset"
            | "wait"
            | "."
            | "["
    )
}
