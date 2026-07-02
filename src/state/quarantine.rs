//! Quarantine for relay events that cannot be admitted yet.
//!
//! The primary use is inbound kind:9 chat observed before the channel's roster
//! snapshots have hydrated. Quarantined rows stay out of `relay_events` so the
//! startup backfill cannot promote unadmitted chat into accepted messages.

use super::*;

impl Store {
    pub fn quarantine_event(
        &self,
        ev: &RelayEvent,
        event_json: &str,
        reason: &str,
        quarantined_at: u64,
    ) -> Result<bool> {
        let n = self.conn.execute(
            "INSERT INTO relay_event_quarantine
                 (id, kind, pubkey, created_at, channel_h, event_json, reason, quarantined_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                 reason=excluded.reason,
                 quarantined_at=excluded.quarantined_at",
            params![
                ev.id,
                ev.kind as i64,
                ev.pubkey,
                ev.created_at,
                ev.channel_h,
                event_json,
                reason,
                quarantined_at
            ],
        )?;
        Ok(n > 0)
    }

    pub fn quarantined_chat_events_for_channel(
        &self,
        channel_h: &str,
    ) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_json FROM relay_event_quarantine
             WHERE channel_h=?1 AND kind=9
             ORDER BY created_at ASC, id ASC",
        )?;
        let rows = stmt.query_map(params![channel_h], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn remove_quarantined_event(&self, id: &str) -> Result<bool> {
        let n = self.conn.execute(
            "DELETE FROM relay_event_quarantine WHERE id=?1",
            params![id],
        )?;
        Ok(n > 0)
    }

    pub fn count_quarantined_events(&self, channel_h: &str) -> Result<u64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM relay_event_quarantine WHERE channel_h=?1",
            params![channel_h],
            |r| r.get::<_, i64>(0),
        )? as u64)
    }
}
