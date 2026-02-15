use anyhow::Result;
use std::process::{Command, Stdio};

pub(super) fn syntax_check(shell: &str, command: &str) -> Result<bool> {
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

pub(super) fn command_exists(shell: &str, head: &str) -> Result<bool> {
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

pub(super) fn is_alias(shell: &str, head: &str) -> Result<bool> {
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

pub(super) fn shell_escape(raw: &str) -> String {
    if raw.is_empty() {
        return "''".to_string();
    }
    let escaped = raw.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}
