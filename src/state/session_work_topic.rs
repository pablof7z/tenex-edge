use super::*;

impl Store {
    /// Persist the caller-declared broad title. `work_topic` is kept as the
    /// distillation-suppression marker; the public session title changes now.
    pub fn set_session_work_topic(&self, id: &str, topic: &str, set_at: u64) -> Result<()> {
        let Some(canonical) = self.resolve_canonical_id(id)? else {
            return Ok(());
        };
        self.conn.execute(
            "UPDATE sessions SET work_topic=?2, work_topic_set_at=?3, title=?2 WHERE session_id=?1",
            params![canonical, topic, set_at],
        )?;
        Ok(())
    }
}
