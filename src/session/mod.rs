mod recent;
mod record;
mod store;
mod time;

use anyhow::Result;

pub use record::{SessionRecord, SessionTurn};

pub fn load_session(uuid: &str) -> Result<SessionRecord> {
    store::load_session(uuid)
}

pub fn save_session(session: &SessionRecord) -> Result<()> {
    store::save_session(session)
}

pub fn list_recent_commands(limit: usize) -> Result<Vec<String>> {
    recent::list_recent_commands(limit)
}
