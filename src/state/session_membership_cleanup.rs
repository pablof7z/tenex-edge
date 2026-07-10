//! Session queries used by lifecycle membership cleanup.

use super::sessions::{row_to_session, COLS};
use super::*;

impl Store {
    /// Local sessions that may need lifecycle cleanup. Includes every alive row
    /// (so callers can check child-pid liveness) and dead rows whose last heartbeat
    /// is older than the membership grace window.
    pub fn list_membership_cleanup_candidates(&self, stale_before: u64) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM sessions
             WHERE alive=1 OR (last_seen > 0 AND last_seen < ?1)
             ORDER BY created_at DESC"
        ))?;
        let rows = stmt.query_map(params![stale_before], row_to_session)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
