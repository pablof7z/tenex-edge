use super::*;

impl Store {
    /// Apply the canonical local turn projection.
    pub fn apply_turn_projection(
        &self,
        pubkey: &str,
        working: bool,
        turn_started_at: u64,
        transcript_ref: Option<&str>,
    ) -> Result<()> {
        if let Some(transcript) = transcript_ref {
            self.conn.execute(
                "UPDATE sessions
                 SET working=?2, turn_started_at=?3, transcript_path=?4
                 WHERE pubkey=?1",
                params![pubkey, working as i64, turn_started_at, transcript],
            )?;
        } else {
            self.conn.execute(
                "UPDATE sessions
                 SET working=?2, turn_started_at=?3
                 WHERE pubkey=?1",
                params![pubkey, working as i64, turn_started_at],
            )?;
        }
        Ok(())
    }

    /// Atomically advance the awareness cursor if the caller still observes the
    /// canonical value it read. Returns false when another hook already advanced
    /// it or the session row disappeared.
    pub fn advance_cursor_if_current(
        &self,
        pubkey: &str,
        expected: u64,
        seen_cursor: u64,
    ) -> Result<bool> {
        Ok(self.conn.execute(
            "UPDATE sessions SET seen_cursor=?3 WHERE pubkey=?1 AND seen_cursor=?2",
            params![pubkey, expected, seen_cursor],
        )? == 1)
    }
}
