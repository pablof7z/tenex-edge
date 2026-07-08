//! `outbox` — the outbound publish queue.
//!
//! A signed event is enqueued before it hits the relay so it survives a crash
//! between the decision to publish and the relay ack. The drainer publishes
//! pending rows and marks each published or failed (bumping the retry count).

use super::*;

const COLS: &str = "local_id, event_json, state, retries, last_error, enqueued_at, next_attempt_at";

fn row_to_outbox(row: &rusqlite::Row) -> rusqlite::Result<OutboxRow> {
    Ok(OutboxRow {
        local_id: row.get(0)?,
        event_json: row.get(1)?,
        state: row.get(2)?,
        retries: row.get(3)?,
        last_error: row.get(4)?,
        enqueued_at: row.get(5)?,
        next_attempt_at: row.get(6)?,
    })
}

/// Delay (seconds) before a failed publish may be retried: exponential in the
/// row's `retries` (base 2s, ×2 per failure), capped at 60s, plus a small
/// deterministic per-row jitter so many rows failing at once against a wedged
/// relay don't re-fire in a synchronized burst. A wedged relay therefore sees at
/// most ~1 attempt/min/row instead of a per-`outbox_notify` storm (issue #295).
pub fn outbox_retry_delay_secs(retries: i64, local_id: i64) -> u64 {
    const BASE_SECS: u64 = 2;
    const CAP_SECS: u64 = 60;
    let exp = retries.clamp(0, 16) as u32;
    let base = BASE_SECS.saturating_mul(1u64 << exp).min(CAP_SECS);
    // jitter in [0, base/4], derived from local_id (no rng dependency).
    let jitter = (local_id as u64).wrapping_mul(2_654_435_761) % (base / 4 + 1);
    base + jitter
}

impl Store {
    /// Queue a signed event JSON for publishing. Returns its `local_id`.
    pub fn enqueue_outbox(&self, event_json: &str, enqueued_at: u64) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO outbox (event_json, state, enqueued_at) VALUES (?1, 'pending', ?2)",
            params![event_json, enqueued_at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Read-only peek at the next *due* pending rows to publish, oldest-first,
    /// capped at `limit`. A row is due when `next_attempt_at <= now`, so a row
    /// currently backing off after a failed publish is skipped (and, crucially,
    /// does not head-of-line-block newer due rows). Callers mark rows published
    /// or failed after the relay result. Pass `now` = current wall-clock seconds.
    pub fn peek_outbox(&self, limit: u32, now: u64) -> Result<Vec<OutboxRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM outbox
             WHERE state='pending' AND next_attempt_at <= ?2
             ORDER BY local_id ASC LIMIT ?1"
        ))?;
        // SQLite integers are i64; clamp so a u64::MAX "all-due" sentinel binds.
        let now_i64 = now.min(i64::MAX as u64) as i64;
        let rows = stmt.query_map(params![limit, now_i64], row_to_outbox)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Schedule the next retry of a still-pending outbox row: it will not be
    /// returned by [`peek_outbox`] again until `next_attempt_at`. No-op on the
    /// happy path (a successful publish moves the row out of 'pending').
    pub fn schedule_outbox_retry(&self, local_id: i64, next_attempt_at: u64) -> Result<()> {
        let next_i64 = next_attempt_at.min(i64::MAX as u64) as i64;
        self.conn.execute(
            "UPDATE outbox SET next_attempt_at=?2 WHERE local_id=?1",
            params![local_id, next_i64],
        )?;
        Ok(())
    }

    /// Fetch one outbound publish row by local id.
    pub fn get_outbox(&self, local_id: i64) -> Result<Option<OutboxRow>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM outbox WHERE local_id=?1"),
                params![local_id],
                row_to_outbox,
            )
            .optional()?)
    }

    /// Fetch outbound publish rows whose signed event JSON id starts with the
    /// supplied prefix, newest first. The outbox table stores raw signed JSON, so
    /// this parses candidate rows rather than trusting text search.
    pub fn outbox_by_event_id_prefix(&self, prefix: &str) -> Result<Vec<OutboxRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM outbox ORDER BY enqueued_at DESC, local_id DESC"
        ))?;
        let rows = stmt.query_map([], row_to_outbox)?;
        let mut matched = Vec::new();
        for row in rows {
            let row = row?;
            if event_json_id(&row.event_json).is_some_and(|id| id.starts_with(prefix)) {
                matched.push(row);
            }
        }
        Ok(matched)
    }

    /// Apply a Trellis-derived publish result to the durable queue row.
    pub fn apply_outbox_projection(
        &self,
        local_id: i64,
        state: &str,
        last_error: Option<&str>,
        bump_retries: bool,
    ) -> Result<()> {
        if bump_retries {
            self.conn.execute(
                "UPDATE outbox
                 SET state=?2, retries=retries+1, last_error=?3
                 WHERE local_id=?1",
                params![local_id, state, last_error],
            )?;
        } else {
            self.conn.execute(
                "UPDATE outbox SET state=?2, last_error=?3 WHERE local_id=?1",
                params![local_id, state, last_error],
            )?;
        }
        Ok(())
    }
}

fn event_json_id(event_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(event_json)
        .ok()
        .and_then(|v| {
            v.get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_outbox_reads_non_pending_rows() {
        let s = Store::open_memory().unwrap();
        let id = s.enqueue_outbox(r#"{"id":"ev1"}"#, 100).unwrap();
        s.apply_outbox_projection(id, "published", None, false)
            .unwrap();

        let row = s.get_outbox(id).unwrap().unwrap();
        assert_eq!(row.local_id, id);
        assert_eq!(row.state, "published");
        assert_eq!(row.enqueued_at, 100);
        assert!(s.get_outbox(id + 1).unwrap().is_none());
    }

    #[test]
    fn outbox_by_event_id_prefix_reads_signed_json_ids() {
        let s = Store::open_memory().unwrap();
        let first = s.enqueue_outbox(r#"{"id":"evt-123"}"#, 100).unwrap();
        s.enqueue_outbox(r#"{"id":"other"}"#, 101).unwrap();

        let rows = s.outbox_by_event_id_prefix("evt-").unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].local_id, first);
    }
}
