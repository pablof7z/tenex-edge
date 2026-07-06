//! `channel_readiness_attempts` records host/provider readiness decisions.
//!
//! Relay materialized rows remain the source of channel truth. These rows only
//! explain local attempts to make that truth exist or become usable.

use super::*;

const COLS: &str = "id, channel_h, expect_member, parent_hint, name, source, outcome, reason, \
                   created_at";

fn row_to_attempt(row: &rusqlite::Row) -> rusqlite::Result<ChannelReadinessAttempt> {
    Ok(ChannelReadinessAttempt {
        id: row.get(0)?,
        channel_h: row.get(1)?,
        expect_member: row.get(2)?,
        parent_hint: row.get(3)?,
        name: row.get(4)?,
        source: row.get(5)?,
        outcome: row.get(6)?,
        reason: row.get(7)?,
        created_at: row.get(8)?,
    })
}

impl Store {
    pub fn record_channel_readiness_attempt(
        &self,
        row: &NewChannelReadinessAttempt,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO channel_readiness_attempts
                 (channel_h, expect_member, parent_hint, name, source, outcome, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                row.channel_h,
                row.expect_member,
                row.parent_hint,
                row.name,
                row.source,
                row.outcome,
                row.reason,
                row.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn channel_readiness_attempts(
        &self,
        channel_h: &str,
        limit: u32,
    ) -> Result<Vec<ChannelReadinessAttempt>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM channel_readiness_attempts
             WHERE channel_h=?1
             ORDER BY created_at DESC, id DESC
             LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![channel_h, limit], row_to_attempt)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn channel_readiness_attempt(&self, id: i64) -> Result<Option<ChannelReadinessAttempt>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM channel_readiness_attempts WHERE id=?1"),
                params![id],
                row_to_attempt,
            )
            .optional()?)
    }
}
