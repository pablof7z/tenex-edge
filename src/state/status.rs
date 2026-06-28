//! `relay_status` — kind:30315 current activity, one row per (pubkey, channel).
//!
//! Liveness is freshness via NIP-40: a row is live only while `now <= expiration`.

use super::*;

fn row_to_status(row: &rusqlite::Row) -> rusqlite::Result<Status> {
    Ok(Status {
        pubkey: row.get(0)?,
        channel_h: row.get(1)?,
        slug: row.get(2)?,
        title: row.get(3)?,
        activity: row.get(4)?,
        busy: row.get::<_, i64>(5)? != 0,
        last_seen: row.get(6)?,
        updated_at: row.get(7)?,
        expiration: row.get(8)?,
    })
}

const COLS: &str =
    "pubkey, channel_h, slug, title, activity, busy, last_seen, updated_at, expiration";

impl Store {
    /// Materialize a kind:30315 status. Newer `updated_at` wins; a pubkey appears
    /// at most once per channel.
    pub fn upsert_status(&self, s: &Status) -> Result<()> {
        self.conn.execute(
            "INSERT INTO relay_status
                 (pubkey, channel_h, slug, title, activity, busy, last_seen, updated_at, expiration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(pubkey, channel_h) DO UPDATE SET
                 slug=excluded.slug, title=excluded.title, activity=excluded.activity,
                 busy=excluded.busy, last_seen=excluded.last_seen,
                 updated_at=excluded.updated_at, expiration=excluded.expiration
             WHERE excluded.updated_at >= relay_status.updated_at",
            params![
                s.pubkey,
                s.channel_h,
                s.slug,
                s.title,
                s.activity,
                s.busy as i64,
                s.last_seen,
                s.updated_at,
                s.expiration
            ],
        )?;
        Ok(())
    }

    /// Read what one agent is doing in one channel (regardless of liveness).
    pub fn get_status(&self, pubkey: &str, channel_h: &str) -> Result<Option<Status>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM relay_status WHERE pubkey=?1 AND channel_h=?2"),
                params![pubkey, channel_h],
                row_to_status,
            )
            .optional()?)
    }

    /// All currently-live statuses in a channel (`now <= expiration`), newest
    /// activity first. Expired rows (NIP-40) are excluded.
    pub fn live_status_for_channel(&self, channel_h: &str, now: u64) -> Result<Vec<Status>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM relay_status
             WHERE channel_h=?1 AND expiration >= ?2
             ORDER BY updated_at DESC"
        ))?;
        let rows = stmt.query_map(params![channel_h, now], row_to_status)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Distinct channels with any status update at or after `cursor` — the set of
    /// channels worth re-rendering for awareness deltas.
    pub fn active_channels_since(&self, cursor: u64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT channel_h FROM relay_status WHERE updated_at >= ?1 ORDER BY channel_h",
        )?;
        let rows = stmt.query_map(params![cursor], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
