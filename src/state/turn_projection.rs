use super::*;

impl Store {
    /// Apply the Trellis-derived local turn projection. The graph decides the
    /// values; this method only writes them to the canonical row.
    pub fn apply_turn_projection(
        &self,
        id: &str,
        working: bool,
        turn_started_at: u64,
        transcript_ref: Option<&str>,
    ) -> Result<()> {
        let Some(canonical) = self.resolve_canonical_id(id)? else {
            return Ok(());
        };
        if let Some(transcript) = transcript_ref {
            self.conn.execute(
                "UPDATE sessions
                 SET working=?2, turn_started_at=?3, transcript_path=?4
                 WHERE session_id=?1",
                params![canonical, working as i64, turn_started_at, transcript],
            )?;
        } else {
            self.conn.execute(
                "UPDATE sessions
                 SET working=?2, turn_started_at=?3
                 WHERE session_id=?1",
                params![canonical, working as i64, turn_started_at],
            )?;
        }
        Ok(())
    }
}
