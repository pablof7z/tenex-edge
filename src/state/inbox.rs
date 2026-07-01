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

    /// All pending inbound rows for a session, oldest-first, WITHOUT consuming
    /// them — a read-only peek for callers that only display or warm caches
    /// (statusline, `who`, profile warm-up, the doorbell's "has pending?"
    /// filter). Delivery paths must use [`Store::claim_pending_for_session`]
    /// instead, so the rows are claimed atomically (resolves the id first).
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

    /// Atomically claim every pending row for a session: flip each to
    /// `delivered` AND return it in a single statement. The FIRST caller — the
    /// tmux paste path or a hook — wins; any concurrent caller gets an empty
    /// vec. This atomicity IS the dedup: a message can only be injected once,
    /// with no separate "notified" flag or external gate. Rows come back
    /// oldest-first (RETURNING order is unspecified, so we sort). Resolves the
    /// id first.
    pub fn claim_pending_for_session(
        &self,
        target_session: &str,
        now: u64,
    ) -> Result<Vec<InboxRow>> {
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        let mut stmt = self.conn.prepare(&format!(
            "UPDATE inbox SET state='delivered', delivered_at=?2
             WHERE target_session=?1 AND state='pending'
             RETURNING {COLS}"
        ))?;
        let rows = stmt.query_map(params![target, now], row_to_inbox)?;
        let mut out = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        out.sort_by_key(|r| r.created_at);
        Ok(out)
    }

    /// Roll claimed rows back to `pending` so they are retried rather than lost.
    /// Used only when a tmux paste fails AFTER the atomic claim (e.g. the pane
    /// died between liveness check and paste) — without this, an atomically
    /// claimed-but-undelivered message would silently vanish. Resolves first.
    pub fn reenqueue_pending(&self, event_ids: &[String], target_session: &str) -> Result<()> {
        let target = self
            .resolve_canonical_id(target_session)?
            .unwrap_or_else(|| target_session.to_string());
        for id in event_ids {
            self.conn.execute(
                "UPDATE inbox SET state='pending', delivered_at=0
                 WHERE event_id=?1 AND target_session=?2",
                params![id, target],
            )?;
        }
        Ok(())
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
