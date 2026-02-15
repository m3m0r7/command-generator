use anyhow::Result;
use std::collections::HashSet;
use std::fs;

use super::record::SessionRecord;
use crate::paths;

pub(super) fn list_recent_commands(limit: usize) -> Result<Vec<String>> {
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
