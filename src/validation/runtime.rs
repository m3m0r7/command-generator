use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use super::types::{CommandHead, RuntimeCheck};

pub(crate) fn runtime_check(shell: &str, command: &str) -> Result<RuntimeCheck> {
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

pub(crate) fn can_runtime_check(command: &str, heads: &[CommandHead]) -> bool {
    if command.trim().is_empty() || command.len() > 400 {
        return false;
    }
    let lowered = command.to_lowercase();
    if lowered.contains("$(")
        || lowered.contains('`')
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
