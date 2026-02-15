use anyhow::Result;
use std::collections::HashSet;

use super::parser::{collect_command_heads, find_invalid_cd_directories, find_placeholder_tokens};
use super::report::ValidationReport;
use super::runtime::{can_runtime_check, runtime_check};
use super::shell_checks::{command_exists, is_alias, syntax_check};

pub(super) fn validate_command_internal(command: &str) -> Result<ValidationReport> {
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
    let invalid_directories = find_invalid_cd_directories(command);

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
        invalid_directories,
        placeholder_tokens: find_placeholder_tokens(command),
        runtime_checked: false,
        runtime_ok: true,
        runtime_note: None,
    };

    if report.syntax_ok
        && report.missing_binaries.is_empty()
        && report.alias_conflicts.is_empty()
        && report.invalid_directories.is_empty()
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
