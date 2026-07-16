use super::*;

impl Store {
    /// Persist the caller-declared title used by local and relay status views.
    pub fn set_session_title(&self, pubkey: &str, title: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET title=?2 WHERE pubkey=?1",
            params![pubkey, title],
        )?;
        Ok(())
    }
}
