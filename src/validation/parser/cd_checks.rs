use std::collections::BTreeSet;
use std::path::Path;

use super::segments::split_segments;
use super::tokens::{locate_head_token, tokenize_segment};

pub(crate) fn find_invalid_cd_directories(command: &str) -> Vec<String> {
    let mut invalid = BTreeSet::new();
    let cwd = std::env::current_dir().ok();
    for segment in split_segments(command) {
        let tokens = tokenize_segment(&segment);
        let Some(head) = locate_head_token(&tokens) else {
            continue;
        };
        if head.name != "cd" {
            continue;
        }

        let arg_index = head.token_index + 1;
        let Some(path_token) = tokens.get(arg_index) else {
            continue;
        };
        let mut raw_path = path_token.cooked.trim().to_string();
        if raw_path.is_empty() || raw_path == "-" {
            continue;
        }
        if raw_path == "--"
            && let Some(next) = tokens.get(arg_index + 1)
        {
            raw_path = next.cooked.trim().to_string();
        }
        if raw_path.is_empty() || raw_path == "-" {
            continue;
        }
        if contains_dynamic_path(&raw_path) {
            continue;
        }
        if let Some(expanded) = expand_tilde_path(&raw_path) {
            raw_path = expanded;
        }

        let candidate = Path::new(&raw_path);
        let resolved = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else if let Some(base) = &cwd {
            base.join(candidate)
        } else {
            candidate.to_path_buf()
        };

        if !resolved.exists() || !resolved.is_dir() {
            invalid.insert(raw_path.clone());
        }
    }
    invalid.into_iter().collect::<Vec<_>>()
}

fn contains_dynamic_path(path: &str) -> bool {
    path.contains('$')
        || path.contains('*')
        || path.contains('?')
        || path.contains('[')
        || path.contains('{')
        || path.contains('`')
        || path.contains("$(")
}

fn expand_tilde_path(path: &str) -> Option<String> {
    if path == "~" {
        return dirs::home_dir().map(|home| home.display().to_string());
    }
    if let Some(suffix) = path.strip_prefix("~/") {
        return dirs::home_dir().map(|home| home.join(suffix).display().to_string());
    }
    None
}
