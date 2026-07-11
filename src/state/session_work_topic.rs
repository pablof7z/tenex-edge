use super::*;

impl Store {
    /// Persist the caller-declared broad work topic. Alias resolution keeps this
    /// mutation pinned to the canonical session row rather than a harness id.
    pub fn set_session_work_topic(&self, id: &str, topic: &str, set_at: u64) -> Result<()> {
        let Some(canonical) = self.resolve_canonical_id(id)? else {
            return Ok(());
        };
        self.conn.execute(
            "UPDATE sessions SET work_topic=?2, work_topic_set_at=?3 WHERE session_id=?1",
            params![canonical, topic, set_at],
        )?;
        Ok(())
    }
}
