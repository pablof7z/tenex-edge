//! Session selection for explicit resume surfaces.

use super::sessions::{row_to_session, COLS};
use super::*;

impl Store {
    pub(crate) fn session_row(&self, session_id: &str) -> Result<Option<Session>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM sessions WHERE session_id=?1"),
                [session_id],
                row_to_session,
            )
            .optional()?)
    }

    /// Recent resumable per-session identities, newest first. Durable agents
    /// always start fresh and therefore never appear in this candidate set.
    pub fn list_resumable_sessions(&self, limit: u32) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM sessions
             WHERE NOT EXISTS (
                 SELECT 1 FROM durable_agent_sessions d
                 WHERE d.pubkey=sessions.agent_pubkey
             )
             ORDER BY created_at DESC LIMIT ?1"
        ))?;
        let rows = stmt.query_map(params![limit], row_to_session)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
