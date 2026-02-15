use anyhow::Result;

use crate::postprocess::CommandPostProcessor;

pub struct AndOrPrecedenceStage;

impl CommandPostProcessor for AndOrPrecedenceStage {
    fn process(&self, _shell: &str, command: String) -> Result<String> {
        Ok(normalize_and_or_precedence(&command))
    }
}

fn normalize_and_or_precedence(command: &str) -> String {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return command.to_string();
    }
    let first_or = find_top_level_op(trimmed, "||");
    if first_or.is_none() {
        return trimmed.to_string();
    }
    let first_or = first_or.unwrap_or(0);
    let left = trimmed[..first_or].trim();
    if left.is_empty() {
        return trimmed.to_string();
    }
    if find_top_level_op(left, "&&").is_none() {
        return trimmed.to_string();
    }
    if is_wrapped_with_parens(left) {
        return trimmed.to_string();
    }
    format!("({}) {}", left, trimmed[first_or..].trim_start())
}

fn find_top_level_op(input: &str, op: &str) -> Option<usize> {
    let chars = input.char_indices().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut depth = 0i32;
    while i < chars.len() {
        let (idx, ch) = chars[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if ch == '\\' && !in_single {
            escaped = true;
            i += 1;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if !in_single && !in_double {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' && depth > 0 {
                depth -= 1;
            }
            if depth == 0 && input[idx..].starts_with(op) {
                return Some(idx);
            }
        }
        i += 1;
    }
    None
}

fn is_wrapped_with_parens(input: &str) -> bool {
    let trimmed = input.trim();
    if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
        return false;
    }
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    for (idx, ch) in trimmed.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && !in_single {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if in_single || in_double {
            continue;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 && idx != trimmed.len() - 1 {
                return false;
            }
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_mixed_and_or_precedence() {
        assert_eq!(
            normalize_and_or_precedence("test -d src && pwd || echo no"),
            "(test -d src && pwd) || echo no"
        );
    }

    #[test]
    fn keeps_existing_parenthesized_precedence() {
        assert_eq!(
            normalize_and_or_precedence("(test -d src && pwd) || echo no"),
            "(test -d src && pwd) || echo no"
        );
    }
}
