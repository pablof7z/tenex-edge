use super::*;

impl Store {
    /// Advance the awareness cursor only if the caller still owns the value it read.
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
