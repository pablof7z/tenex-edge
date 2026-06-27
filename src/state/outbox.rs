use super::{
    row_to_session_state_offset, StatusOutboxDebugRow, StatusOutboxItem, Store,
    SESSION_STATE_COLS_PREFIXED,
};
use anyhow::Result;
use rusqlite::params;

impl Store {
    pub fn list_status_outbox_debug(&self, limit: u64) -> Result<Vec<StatusOutboxDebugRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT
               o.session_id,
               o.state_version,
               o.publish_state,
               o.retries,
               o.native_event_id,
               o.last_error,
               o.enqueued_at,
               COALESCE(s.agent_slug, ''),
               COALESCE(s.project, ''),
               COALESCE(s.title, ''),
               COALESCE(s.activity, ''),
               COALESCE(s.busy, 0)
             FROM status_outbox o
             LEFT JOIN session_state s ON s.session_id=o.session_id
             ORDER BY o.enqueued_at DESC, o.state_version DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit.min(i64::MAX as u64) as i64], |r| {
            Ok(StatusOutboxDebugRow {
                session_id: r.get(0)?,
                state_version: r.get(1)?,
                publish_state: r.get(2)?,
                retries: r.get(3)?,
                native_event_id: r.get(4)?,
                last_error: r.get(5)?,
                enqueued_at: r.get(6)?,
                agent_slug: r.get(7)?,
                project: r.get(8)?,
                title: r.get(9)?,
                activity: r.get(10)?,
                busy: r.get::<_, i64>(11)? != 0,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Enqueue an outbox row for the session's CURRENT `state_version`.
    pub(in crate::state) fn enqueue_status_outbox_current(
        &self,
        session_id: &str,
        ts: u64,
    ) -> Result<()> {
        let version: Option<i64> = self
            .conn
            .query_row(
                "SELECT state_version FROM session_state WHERE session_id=?1",
                params![session_id],
                |r| r.get(0),
            )
            .ok();
        if let Some(v) = version {
            self.enqueue_status_outbox(session_id, v, ts)?;
        }
        Ok(())
    }

    pub(in crate::state) fn enqueue_status_outbox(
        &self,
        session_id: &str,
        state_version: i64,
        ts: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO status_outbox
               (session_id, state_version, publish_state, retries, enqueued_at)
             VALUES (?1, ?2, 'pending', 0, ?3)",
            params![session_id, state_version, ts],
        )?;
        Ok(())
    }

    /// Pending publications joined to the CURRENT session snapshot, oldest first.
    /// The drainer publishes each via `Nip29Provider::set_status` and then
    /// calls `mark_status_published` / `mark_status_failed`.
    pub(crate) fn pending_status_outbox(&self, limit: u64) -> Result<Vec<StatusOutboxItem>> {
        let sql = format!(
            "SELECT o.session_id, o.state_version, o.retries, {cols}
             FROM status_outbox o
             JOIN session_state s ON s.session_id=o.session_id
             WHERE o.publish_state='pending'
             ORDER BY o.enqueued_at ASC, o.state_version ASC
             LIMIT ?1",
            cols = SESSION_STATE_COLS_PREFIXED
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let session_id: String = row.get(0)?;
            let state_version: i64 = row.get(1)?;
            let retries: i64 = row.get(2)?;
            // Snapshot columns start at index 3.
            let snapshot = row_to_session_state_offset(row, 3)?;
            Ok(StatusOutboxItem {
                session_id,
                state_version,
                retries,
                snapshot,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Mark a publication delivered, recording the native event id.
    pub(crate) fn mark_status_published(
        &self,
        session_id: &str,
        state_version: i64,
        native_event_id: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE status_outbox SET publish_state='published', native_event_id=?3, last_error=NULL
             WHERE session_id=?1 AND state_version=?2",
            params![session_id, state_version, native_event_id],
        )?;
        Ok(())
    }

    /// Record a failed publish attempt (increments retries, keeps it pending).
    pub(crate) fn mark_status_failed(
        &self,
        session_id: &str,
        state_version: i64,
        error: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE status_outbox SET retries=retries+1, last_error=?3
             WHERE session_id=?1 AND state_version=?2",
            params![session_id, state_version, error],
        )?;
        Ok(())
    }
}
