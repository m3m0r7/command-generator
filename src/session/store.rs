use anyhow::{Context, Result, anyhow};
use std::fs;

use super::record::SessionRecord;
use crate::paths;

pub(super) fn load_session(uuid: &str) -> Result<SessionRecord> {
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

pub(super) fn save_session(session: &SessionRecord) -> Result<()> {
    fs::create_dir_all(paths::sessions_dir())?;
    let path = session_path(&session.uuid);
    let content = serde_json::to_string_pretty(session)?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write session file: {}", path.display()))?;
    Ok(())
}

pub(super) fn session_path(uuid: &str) -> std::path::PathBuf {
    paths::sessions_dir().join(format!("{}.json", uuid))
}
