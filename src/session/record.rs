use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::now_unix;
use crate::llm::CommandExplanationItem;
use crate::model::ProviderKind;
use crate::validation::ValidationReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTurn {
    pub timestamp: i64,
    pub user_input: String,
    pub command: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub explanations: Vec<CommandExplanationItem>,
    pub validation: ValidationReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub uuid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub turns: Vec<SessionTurn>,
}

impl SessionRecord {
    pub fn new(provider: ProviderKind, model: impl Into<String>) -> Self {
        let now = now_unix();
        Self {
            uuid: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            provider: provider.as_str().to_string(),
            model: model.into(),
            turns: Vec::new(),
        }
    }

    pub fn push_turn(
        &mut self,
        user_input: impl Into<String>,
        command: impl Into<String>,
        reason: impl Into<String>,
        explanations: Vec<CommandExplanationItem>,
        validation: ValidationReport,
    ) {
        let now = now_unix();
        self.updated_at = now;
        self.turns.push(SessionTurn {
            timestamp: now,
            user_input: user_input.into(),
            command: command.into(),
            reason: reason.into(),
            explanations,
            validation,
        });
    }

    pub fn recent_turns(&self, limit: usize) -> Vec<SessionTurn> {
        if limit == 0 {
            return Vec::new();
        }
        let mut turns = self.turns.clone();
        if turns.len() > limit {
            let start = turns.len().saturating_sub(limit);
            turns = turns[start..].to_vec();
        }
        turns
    }
}
