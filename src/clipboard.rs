use anyhow::{Result, anyhow};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_text(text: &str) -> Result<()> {
    if cfg!(target_os = "macos") {
        return run_copy_command("pbcopy", &[], text);
    }
    if cfg!(target_os = "windows") {
        return run_copy_command("clip", &[], text);
    }

    run_copy_command("wl-copy", &[], text)
        .or_else(|_| run_copy_command("xclip", &["-selection", "clipboard"], text))
        .or_else(|_| run_copy_command("xsel", &["--clipboard", "--input"], text))
        .map_err(|_| anyhow!("no clipboard command available (tried wl-copy, xclip, xsel)"))
}

fn run_copy_command(cmd: &str, args: &[&str], text: &str) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())?;
    }
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("clipboard command '{}' failed", cmd))
    }
}
