use anyhow::Result;

use crate::postprocess::CommandPostProcessor;
use crate::validation;

pub struct AliasPrefixStage;

impl CommandPostProcessor for AliasPrefixStage {
    fn process(&self, shell: &str, command: String) -> Result<String> {
        validation::normalize_alias_prefixes(shell, &command)
    }
}
