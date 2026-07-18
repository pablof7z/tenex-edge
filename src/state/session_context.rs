use super::*;

impl Store {
    pub fn set_session_transcript(&self, pubkey: &str, transcript_path: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET transcript_path=?2 WHERE pubkey=?1",
            params![pubkey, transcript_path],
        )?;
        Ok(())
    }

    pub fn set_session_channel(&self, pubkey: &str, channel_h: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET channel_h=?2 WHERE pubkey=?1",
            params![pubkey, channel_h],
        )?;
        if !channel_h.trim().is_empty() {
            self.grant_session_route(pubkey, channel_h, crate::util::now_secs())?;
        }
        Ok(())
    }

    pub fn set_session_context(
        &self,
        pubkey: &str,
        channel_h: &str,
        work_root: &str,
        readiness_parent: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET channel_h=?2, work_root=?3, readiness_parent=?4
             WHERE pubkey=?1",
            params![pubkey, channel_h, work_root, readiness_parent],
        )?;
        if !channel_h.trim().is_empty() {
            self.grant_session_route(pubkey, channel_h, crate::util::now_secs())?;
        }
        Ok(())
    }

    pub fn session_readiness_parent(&self, channel_h: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT readiness_parent FROM sessions
                 WHERE channel_h=?1 AND readiness_parent<>''
                 ORDER BY (runtime_state='running') DESC, created_at DESC LIMIT 1",
                [channel_h],
                |row| row.get::<_, String>(0),
            )
            .optional()?)
    }
}
