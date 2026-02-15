mod parser;
mod report;
mod runtime;
mod shell_checks;
mod types;
mod validate;

#[cfg(test)]
mod tests;

use anyhow::Result;

pub use report::ValidationReport;

pub fn validate_command(command: &str) -> Result<ValidationReport> {
    validate::validate_command_internal(command)
}

pub fn normalize_alias_prefixes(shell: &str, command: &str) -> Result<String> {
    shell_checks::normalize_alias_prefixes(shell, command)
}
