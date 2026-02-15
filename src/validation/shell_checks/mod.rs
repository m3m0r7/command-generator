mod alias_normalize;
mod detect;

use anyhow::Result;

pub(crate) fn syntax_check(shell: &str, command: &str) -> Result<bool> {
    detect::syntax_check(shell, command)
}

pub(crate) fn command_exists(shell: &str, head: &str) -> Result<bool> {
    detect::command_exists(shell, head)
}

pub(crate) fn is_alias(shell: &str, head: &str) -> Result<bool> {
    detect::is_alias(shell, head)
}

pub(crate) fn normalize_alias_prefixes(shell: &str, command: &str) -> Result<String> {
    alias_normalize::normalize_alias_prefixes(shell, command)
}
