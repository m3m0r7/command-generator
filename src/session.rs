use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::model::ProviderKind;
use crate::paths;
use crate::validation::ValidationReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTurn {
    pub timestamp: i64,
    pub user_input: String,
    pub command: String,
    #[serde(default)]
    pub reason: String,
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
        validation: ValidationReport,
    ) {
        let now = now_unix();
        self.updated_at = now;
        self.turns.push(SessionTurn {
            timestamp: now,
            user_input: user_input.into(),
            command: command.into(),
            reason: reason.into(),
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

pub fn load_session(uuid: &str) -> Result<SessionRecord> {
    let path = session_path(uuid);
    if !path.exists() {
        return Err(anyhow!("session '{}' not found", uuid));
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read session file: {}", path.display()))?;
    let session: SessionRecord = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse session JSON: {}", path.display()))?;
    Ok(session)
}

pub fn save_session(session: &SessionRecord) -> Result<()> {
    fs::create_dir_all(paths::sessions_dir())?;
    let path = session_path(&session.uuid);
    let content = serde_json::to_string_pretty(session)?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write session file: {}", path.display()))?;
    Ok(())
}

pub fn list_recent_commands(limit: usize) -> Result<Vec<String>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let dir = paths::sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut items: Vec<(i64, String)> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(session) = serde_json::from_str::<SessionRecord>(&content) else {
            continue;
        };
        for turn in session.turns {
            if !turn.command.trim().is_empty() {
                items.push((turn.timestamp, turn.command));
            }
        }
    }

    items.sort_by(|a, b| b.0.cmp(&a.0));

    let mut deduped = Vec::new();
    let mut seen = HashSet::new();
    for (_, command) in items {
        if seen.insert(command.clone()) {
            deduped.push(command);
            if deduped.len() >= limit {
                break;
            }
        }
    }
    Ok(deduped)
}

fn session_path(uuid: &str) -> std::path::PathBuf {
    paths::sessions_dir().join(format!("{}.json", uuid))
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
