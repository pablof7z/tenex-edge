//! `inbox` — the inbound routing ledger AND the idempotency record.
//!
//! One row per (inbound event, target local session). An event is "handled"
//! because a row exists; there is no separate processed-orchestration table. A
//! row starts `pending` (parked for the next hook) and becomes `delivered` once
//! injected into a live tmux.

use super::*;

const COLS: &str = "event_id, target_session, state, from_pubkey, channel_h, body, created_at, \
     delivered_at";

fn row_to_inbox(row: &rusqlite::Row) -> rusqlite::Result<InboxRow> {
    Ok(InboxRow {
        event_id: row.get(0)?,
        target_session: row.get(1)?,
        state: row.get(2)?,
        from_pubkey: row.get(3)?,
        channel_h: row.get(4)?,
        body: row.get(5)?,
        created_at: row.get(6)?,
        delivered_at: row.get(7)?,
    })
}

impl Store {
    /// Record an inbound event addressed to a local session. The target id is
    /// resolved to its canonical session first. Idempotent: a duplicate
    /// (event_id, target_session) is ignored. Returns `true` if newly enqueued.
    pub fn enqueue_inbox(
        &self,
        event_id: &str,
        target_session: &str,
        from_pubkey: &str,
        channel_h: &str,
        body: &str,
        created_at: u64,
    ) -> Result<bool> {
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        let n = self.conn.execute(
            "INSERT OR IGNORE INTO inbox
                 (event_id, target_session, state, from_pubkey, channel_h, body, created_at)
             VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?6)",
            params![event_id, target, from_pubkey, channel_h, body, created_at],
        )?;
        Ok(n > 0)
    }

    /// Mark a parked inbound event as delivered into a live session (resolves the
    /// target id first).
    pub fn mark_delivered(
        &self,
        event_id: &str,
        target_session: &str,
        delivered_at: u64,
    ) -> Result<()> {
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        self.conn.execute(
            "UPDATE inbox SET state='delivered', delivered_at=?3
             WHERE event_id=?1 AND target_session=?2",
            params![event_id, target, delivered_at],
        )?;
        Ok(())
    }

    /// All pending inbound rows for a session, oldest-first — what the next hook
    /// should render (resolves the id first).
    pub fn drain_pending_for_session(&self, target_session: &str) -> Result<Vec<InboxRow>> {
        // Fall back to the raw id when no canonical mapping exists — symmetric
        // with `enqueue_inbox`, which parks under the same raw id in that case.
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM inbox
             WHERE target_session=?1 AND state='pending' ORDER BY created_at ASC"
        ))?;
        let rows = stmt.query_map(params![target], row_to_inbox)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Delivered inbound rows for a session whose delivery is newer than `since`,
    /// oldest-first. Powers the statusline "recently delivered" peek (read-only,
    /// resolves the id first).
    pub fn recently_delivered_for_session(
        &self,
        target_session: &str,
        since: u64,
    ) -> Result<Vec<InboxRow>> {
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM inbox
             WHERE target_session=?1 AND state='delivered' AND delivered_at>=?2
             ORDER BY created_at ASC"
        ))?;
        let rows = stmt.query_map(params![target, since], row_to_inbox)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// True if this (event, target) pair has already been recorded — the
    /// idempotency check (resolves the id first).
    pub fn is_event_handled(&self, event_id: &str, target_session: &str) -> Result<bool> {
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM inbox WHERE event_id=?1 AND target_session=?2",
                params![event_id, target],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }
}
