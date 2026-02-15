use anyhow::Result;

use crate::validation::{self, ValidationReport};

pub trait CommandValidator: Send + Sync {
    fn validate(&self, command: &str) -> Result<ValidationReport>;
}

pub struct DefaultCommandValidator;

impl CommandValidator for DefaultCommandValidator {
    fn validate(&self, command: &str) -> Result<ValidationReport> {
        validation::validate_command(command)
    }
}

pub fn default_command_validator() -> Box<dyn CommandValidator> {
    Box::new(DefaultCommandValidator)
}
