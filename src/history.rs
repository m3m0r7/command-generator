use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub fn load_shell_history(limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for path in history_paths() {
        if !path.exists() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        entries.extend(content.lines().filter_map(parse_history_line));
    }

    if entries.is_empty() {
        return entries;
    }

    if entries.len() > limit {
        let start = entries.len().saturating_sub(limit);
        entries = entries[start..].to_vec();
    }

    entries.reverse();

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for entry in entries {
        if seen.insert(entry.clone()) {
            deduped.push(entry);
        }
    }
    deduped
}

fn history_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        let home = home.trim();
        if !home.is_empty() {
            paths.push(PathBuf::from(home).join(".zsh_history"));
            paths.push(PathBuf::from(home).join(".bash_history"));
        }
    }
    paths
}

fn parse_history_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    if let Some((_, command)) = trimmed.split_once(';')
        && trimmed.starts_with(": ")
    {
        let command = command.trim();
        if !command.is_empty() {
            return Some(command.to_string());
        }
    }
    Some(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zsh_history_format() {
        let line = ": 1730000000:0;pwd";
        assert_eq!(parse_history_line(line).as_deref(), Some("pwd"));
    }

    #[test]
    fn keeps_plain_line() {
        assert_eq!(parse_history_line("ls -la").as_deref(), Some("ls -la"));
    }
}
