//! Per-session chat publication markers.

use super::*;

impl Store {
    /// Mark that this session has successfully published through an explicit
    /// channel command. Resolves aliases first and keeps the earliest non-zero
    /// marker stable.
    pub fn mark_session_explicit_chat_published(&self, id: &str, at: u64) -> Result<()> {
        let Some(canonical) = self.resolve_canonical_id(id)? else {
            return Ok(());
        };
        self.conn.execute(
            "UPDATE sessions
             SET explicit_chat_published_at=CASE
                 WHEN explicit_chat_published_at=0 THEN ?2
                 ELSE explicit_chat_published_at
             END
             WHERE session_id=?1",
            params![canonical, at],
        )?;
        Ok(())
    }
}
