use anyhow::Result;

use crate::cli::Cli;
use crate::llm::CommandExplanationItem;
use crate::request_engine::types::HandleResult;
use crate::session::{self, SessionRecord};
use crate::validation::ValidationReport;

pub trait CommandCommitter: Send + Sync {
    fn commit(
        &self,
        user_input: &str,
        session: &mut SessionRecord,
        command: String,
        reason: String,
        explanations: Vec<CommandExplanationItem>,
        report: ValidationReport,
    ) -> Result<HandleResult>;
}

pub struct SessionCommandCommitter<'a> {
    cli: &'a Cli,
}

impl<'a> SessionCommandCommitter<'a> {
    pub fn new(cli: &'a Cli) -> Self {
        Self { cli }
    }
}

impl CommandCommitter for SessionCommandCommitter<'_> {
    fn commit(
        &self,
        user_input: &str,
        session: &mut SessionRecord,
        command: String,
        reason: String,
        explanations: Vec<CommandExplanationItem>,
        report: ValidationReport,
    ) -> Result<HandleResult> {
        if self.cli.copy
            && let Err(err) = crate::clipboard::copy_text(&command)
        {
            eprintln!("warning: failed to copy command: {err}");
        }
        session.push_turn(
            user_input,
            command.clone(),
            reason,
            explanations.clone(),
            report,
        );
        session::save_session(session)?;
        let explanations = if self.cli.explanation {
            explanations
        } else {
            Vec::new()
        };
        Ok(HandleResult {
            command,
            explanations,
        })
    }
}
