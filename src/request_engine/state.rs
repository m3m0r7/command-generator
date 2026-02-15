use anyhow::{Error, Result, anyhow};
use std::collections::HashSet;

use crate::prompt::PromptClarification;

pub struct RuntimeState {
    clarifications: Vec<PromptClarification>,
    asked_questions: HashSet<String>,
    feedback: Option<String>,
    max_attempts: usize,
    max_questions: usize,
    question_count: usize,
    command_attempt_count: usize,
    last_error: Option<Error>,
}

impl RuntimeState {
    pub fn new(max_attempts: usize, max_questions: usize) -> Self {
        Self {
            clarifications: Vec::new(),
            asked_questions: HashSet::new(),
            feedback: None,
            max_attempts,
            max_questions,
            question_count: 0,
            command_attempt_count: 0,
            last_error: None,
        }
    }

    pub fn can_attempt_command(&self) -> bool {
        self.command_attempt_count < self.max_attempts
    }

    pub fn mark_command_attempt(&mut self) {
        self.command_attempt_count += 1;
    }

    pub fn ensure_question_capacity(&mut self) -> Result<()> {
        self.question_count += 1;
        if self.question_count > self.max_questions {
            return Err(anyhow!("too many clarification questions from model"));
        }
        Ok(())
    }

    pub fn register_question(&mut self, normalized: String, original: &str) -> Result<()> {
        if !self.asked_questions.insert(normalized) {
            return Err(anyhow!(
                "model asked duplicate clarification question: {}",
                original
            ));
        }
        Ok(())
    }

    pub fn push_clarification(&mut self, question: String, answer: String) {
        self.clarifications
            .push(PromptClarification { question, answer });
    }

    pub fn clarifications(&self) -> &[PromptClarification] {
        &self.clarifications
    }

    pub fn clarifications_empty(&self) -> bool {
        self.clarifications.is_empty()
    }

    pub fn feedback(&self) -> Option<&String> {
        self.feedback.as_ref()
    }

    pub fn set_feedback_reason(&mut self, reason: String) {
        self.feedback = Some(reason.clone());
        self.last_error = Some(anyhow!(reason));
    }

    pub fn clear_feedback(&mut self) {
        self.feedback = None;
    }

    pub fn finish_error(self) -> Error {
        self.last_error
            .unwrap_or_else(|| anyhow!("failed to generate a valid command"))
    }
}
