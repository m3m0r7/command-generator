use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationReport {
    pub syntax_ok: bool,
    pub shell: String,
    #[serde(default)]
    pub missing_binaries: Vec<String>,
    #[serde(default)]
    pub checked_binaries: Vec<String>,
    #[serde(default)]
    pub alias_conflicts: Vec<String>,
    #[serde(default)]
    pub placeholder_tokens: Vec<String>,
    #[serde(default)]
    pub runtime_checked: bool,
    #[serde(default = "default_runtime_ok")]
    pub runtime_ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_note: Option<String>,
}

fn default_runtime_ok() -> bool {
    true
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.syntax_ok
            && self.missing_binaries.is_empty()
            && self.alias_conflicts.is_empty()
            && self.placeholder_tokens.is_empty()
            && (!self.runtime_checked || self.runtime_ok)
    }

    pub fn to_feedback_text(&self) -> String {
        let mut reasons = Vec::new();
        if !self.syntax_ok {
            reasons.push(format!("shell syntax check failed by {}", self.shell));
        }
        if !self.missing_binaries.is_empty() {
            reasons.push(format!(
                "unresolved commands: {}",
                self.missing_binaries.join(", ")
            ));
        }
        if !self.alias_conflicts.is_empty() {
            reasons.push(format!(
                "alias command(s) detected: {} (prefix with builtin or \\\\)",
                self.alias_conflicts.join(", ")
            ));
        }
        if !self.placeholder_tokens.is_empty() {
            reasons.push(format!(
                "placeholder tokens are not allowed: {}",
                self.placeholder_tokens.join(", ")
            ));
        }
        if self.runtime_checked && !self.runtime_ok {
            if let Some(note) = &self.runtime_note {
                reasons.push(format!("runtime validation failed: {}", note));
            } else {
                reasons.push("runtime validation failed".to_string());
            }
        }
        if reasons.is_empty() {
            "validation failed for an unknown reason".to_string()
        } else {
            reasons.join("; ")
        }
    }
}

pub fn validate_command(command: &str) -> Result<ValidationReport> {
    let shell = std::env::var("SHELL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "sh".to_string());

    let syntax_ok = syntax_check(&shell, command)?;
    let heads = collect_command_heads(command);
    let mut checked = Vec::new();
    let mut missing = Vec::new();
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    for head in &heads {
        let lowered = head.name.to_lowercase();
        if !seen.insert(lowered) {
            continue;
        }
        checked.push(head.name.clone());
        if !command_exists(&shell, &head.name)? {
            missing.push(head.name.clone());
            continue;
        }
        if !head.prefixed_builtin
            && !head.prefixed_command
            && !head.prefixed_backslash
            && is_alias(&shell, &head.name)?
        {
            aliases.push(head.name.clone());
        }
    }

    let mut report = ValidationReport {
        syntax_ok,
        shell: shell.clone(),
        missing_binaries: missing,
        checked_binaries: checked,
        alias_conflicts: aliases,
        placeholder_tokens: find_placeholder_tokens(command),
        runtime_checked: false,
        runtime_ok: true,
        runtime_note: None,
    };

    if report.syntax_ok
        && report.missing_binaries.is_empty()
        && report.alias_conflicts.is_empty()
        && report.placeholder_tokens.is_empty()
    {
        if can_runtime_check(command, &heads) {
            let runtime = runtime_check(&shell, command)?;
            report.runtime_checked = true;
            report.runtime_ok = runtime.ok;
            report.runtime_note = runtime.note;
        } else {
            report.runtime_note = Some(
                "runtime check skipped because command may be stateful or long-running".to_string(),
            );
        }
    }

    Ok(report)
}

fn syntax_check(shell: &str, command: &str) -> Result<bool> {
    if command.trim().is_empty() {
        return Ok(false);
    }
    let status = Command::new(shell)
        .arg("-n")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(status.success())
}

fn command_exists(shell: &str, head: &str) -> Result<bool> {
    if which::which(head).is_ok() {
        return Ok(true);
    }
    let snippet = format!("command -v -- {} >/dev/null 2>&1", shell_escape(head));
    let status = Command::new(shell)
        .arg("-c")
        .arg(snippet)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(status.success())
}

fn is_alias(shell: &str, head: &str) -> Result<bool> {
    let snippet = format!("alias {} >/dev/null 2>&1", shell_escape(head));
    let status_interactive = Command::new(shell)
        .arg("-ic")
        .arg(&snippet)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if let Ok(status) = status_interactive
        && status.success()
    {
        return Ok(true);
    }

    let status = Command::new(shell)
        .arg("-c")
        .arg(snippet)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(status.success())
}

#[derive(Debug)]
struct RuntimeCheck {
    ok: bool,
    note: Option<String>,
}

fn runtime_check(shell: &str, command: &str) -> Result<RuntimeCheck> {
    let temp_root = std::env::temp_dir().join("command-generator-validation");
    fs::create_dir_all(&temp_root)?;
    let mut file_path = temp_root.join(format!("{}.sh", uuid::Uuid::new_v4()));
    if file_path.extension().is_none() {
        file_path.set_extension("sh");
    }
    fs::write(&file_path, format!("{}\n", command)).with_context(|| {
        format!(
            "failed to write runtime validation script: {}",
            file_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&file_path, perms)?;
    }

    let output = run_with_timeout(shell, &file_path, 2)?;
    let _ = fs::remove_file(&file_path);

    if output.status.success() {
        return Ok(RuntimeCheck {
            ok: true,
            note: Some("runtime check passed".to_string()),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let mut note = format!("exit status {:?}", output.status.code());
    if !stderr.is_empty() {
        let snippet = stderr.lines().take(3).collect::<Vec<_>>().join(" | ");
        note.push_str(&format!(", stderr: {}", snippet));
    }
    Ok(RuntimeCheck {
        ok: false,
        note: Some(note),
    })
}

fn run_with_timeout(
    shell: &str,
    script_path: &Path,
    timeout_seconds: u64,
) -> Result<std::process::Output> {
    let current_dir = script_path
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(std::env::temp_dir);

    if let Ok(timeout_bin) = which::which("timeout") {
        let output = Command::new(timeout_bin)
            .arg(timeout_seconds.to_string())
            .arg(shell)
            .arg(script_path)
            .current_dir(&current_dir)
            .stdin(Stdio::null())
            .output()?;
        return Ok(output);
    }
    if let Ok(timeout_bin) = which::which("gtimeout") {
        let output = Command::new(timeout_bin)
            .arg(timeout_seconds.to_string())
            .arg(shell)
            .arg(script_path)
            .current_dir(&current_dir)
            .stdin(Stdio::null())
            .output()?;
        return Ok(output);
    }

    let output = Command::new(shell)
        .arg(script_path)
        .current_dir(&current_dir)
        .stdin(Stdio::null())
        .output()?;
    Ok(output)
}

fn can_runtime_check(command: &str, heads: &[CommandHead]) -> bool {
    if command.trim().is_empty() || command.len() > 400 {
        return false;
    }
    let lowered = command.to_lowercase();
    if lowered.contains("$(")
        || lowered.contains("`")
        || lowered.contains(">>")
        || lowered.contains("<<")
        || lowered.contains(">|")
        || lowered.contains("<(")
        || lowered.contains(">(")
    {
        return false;
    }

    let risky_heads = [
        "rm", "mv", "cp", "dd", "mkfs", "reboot", "shutdown", "halt", "poweroff", "kill", "pkill",
        "killall", "chown", "chmod", "chgrp", "ln", "sudo", "tee", "git", "docker", "kubectl",
        "curl", "wget", "scp", "rsync",
    ];

    if heads.iter().any(|head| {
        let lowered = head.name.to_lowercase();
        risky_heads.iter().any(|risky| lowered == *risky)
    }) {
        return false;
    }

    let runtime_safe_heads = [
        "pwd", "ls", "echo", "print", "printf", "whoami", "uname", "id", "env", "which", "command",
        "dirname", "basename", "date", "true", "false", "realpath",
    ];

    heads.iter().all(|head| {
        let lowered = head.name.to_lowercase();
        runtime_safe_heads.iter().any(|safe| lowered == *safe)
    })
}

fn shell_escape(raw: &str) -> String {
    if raw.is_empty() {
        return "''".to_string();
    }
    let escaped = raw.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

fn find_placeholder_tokens(command: &str) -> Vec<String> {
    let banned_words = ["YOUR_VALUE", "REPLACE_ME", "INSERT_VALUE", "PLACEHOLDER"];
    let mut found = Vec::new();
    let lower = command.to_lowercase();
    for word in banned_words {
        if lower.contains(&word.to_lowercase()) {
            found.push(word.to_string());
        }
    }

    for token in shell_words::split(command).unwrap_or_default() {
        if token.len() >= 3
            && token.starts_with('<')
            && token.ends_with('>')
            && !token.contains('/')
            && !token.contains('\\')
        {
            found.push(token);
        }
    }
    found.sort();
    found.dedup();
    found
}

#[derive(Debug, Clone)]
struct CommandHead {
    name: String,
    prefixed_builtin: bool,
    prefixed_command: bool,
    prefixed_backslash: bool,
}

#[derive(Debug, Clone)]
struct SegmentToken {
    raw: String,
    cooked: String,
}

fn collect_command_heads(command: &str) -> Vec<CommandHead> {
    split_segments(command)
        .into_iter()
        .filter_map(|segment| extract_head_command(&segment))
        .collect::<Vec<_>>()
}

fn split_segments(command: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' && !in_single {
            current.push(ch);
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            current.push(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            current.push(ch);
            continue;
        }

        if !in_single && !in_double {
            if ch == ';' {
                push_segment(&mut segments, &mut current);
                continue;
            }
            if ch == '|' {
                if chars.peek() == Some(&'|') {
                    let _ = chars.next();
                }
                push_segment(&mut segments, &mut current);
                continue;
            }
            if ch == '&' && chars.peek() == Some(&'&') {
                let _ = chars.next();
                push_segment(&mut segments, &mut current);
                continue;
            }
        }
        current.push(ch);
    }
    push_segment(&mut segments, &mut current);
    segments
}

fn push_segment(segments: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }
    current.clear();
}

fn extract_head_command(segment: &str) -> Option<CommandHead> {
    let tokens = tokenize_segment(segment);
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
        });
    }
    None
}

fn tokenize_segment(segment: &str) -> Vec<SegmentToken> {
    let mut tokens = Vec::new();
    let mut raw = String::new();
    let mut cooked = String::new();
    let mut in_token = false;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    let push_token = |tokens: &mut Vec<SegmentToken>,
                      raw: &mut String,
                      cooked: &mut String,
                      in_token: &mut bool| {
        if !*in_token {
            return;
        }
        tokens.push(SegmentToken {
            raw: raw.clone(),
            cooked: cooked.clone(),
        });
        raw.clear();
        cooked.clear();
        *in_token = false;
    };

    for ch in segment.chars() {
        if !in_token {
            if ch.is_whitespace() {
                continue;
            }
            in_token = true;
        }

        if escaped {
            raw.push(ch);
            cooked.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' && !in_single {
            raw.push(ch);
            escaped = true;
            continue;
        }

        if ch == '\'' && !in_double {
            raw.push(ch);
            in_single = !in_single;
            continue;
        }

        if ch == '"' && !in_single {
            raw.push(ch);
            in_double = !in_double;
            continue;
        }

        if !in_single && !in_double && ch.is_whitespace() {
            push_token(&mut tokens, &mut raw, &mut cooked, &mut in_token);
            continue;
        }

        raw.push(ch);
        cooked.push(ch);
    }

    if in_token {
        tokens.push(SegmentToken { raw, cooked });
    }

    tokens
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_compound_command() {
        let heads = collect_command_heads("pwd || true; ls -la | grep src");
        let names = heads.into_iter().map(|head| head.name).collect::<Vec<_>>();
        assert_eq!(names, vec!["pwd", "true", "ls", "grep"]);
    }

    #[test]
    fn handles_no_space_pipeline() {
        let heads = collect_command_heads("cat Cargo.toml|grep name");
        let names = heads.into_iter().map(|head| head.name).collect::<Vec<_>>();
        assert_eq!(names, vec!["cat", "grep"]);
    }

    #[test]
    fn skips_env_assignment() {
        let heads = collect_command_heads("FOO=bar env ls");
        let names = heads.into_iter().map(|head| head.name).collect::<Vec<_>>();
        assert_eq!(names, vec!["ls"]);
    }

    #[test]
    fn allows_runtime_for_simple_readonly_command() {
        let heads = collect_command_heads("pwd");
        assert!(can_runtime_check("pwd", &heads));
    }

    #[test]
    fn skips_runtime_for_risky_command() {
        let heads = collect_command_heads("rm -rf /tmp/foo");
        assert!(!can_runtime_check("rm -rf /tmp/foo", &heads));
    }

    #[test]
    fn detects_placeholder_tokens() {
        let tokens = find_placeholder_tokens("echo <STRING>");
        assert_eq!(tokens, vec!["<STRING>"]);
    }

    #[test]
    fn detects_builtin_prefix() {
        let head = extract_head_command("builtin test -f Cargo.toml").unwrap();
        assert_eq!(head.name, "test");
        assert!(head.prefixed_builtin);
    }

    #[test]
    fn detects_command_prefix() {
        let head = extract_head_command("command ls -la").unwrap();
        assert_eq!(head.name, "ls");
        assert!(head.prefixed_command);
    }

    #[test]
    fn detects_backslash_prefix() {
        let head = extract_head_command("\\ls -la").unwrap();
        assert_eq!(head.name, "ls");
        assert!(head.prefixed_backslash);
    }
}
