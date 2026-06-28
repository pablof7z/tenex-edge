//! `project_roots` — local filesystem map (channel/project id -> abs path on THIS
//! machine). The one fact about a project that is genuinely machine-local.

use super::*;

impl Store {
    /// Record (or update) the absolute on-disk path for a channel/project.
    pub fn upsert_project_root(
        &self,
        channel_h: &str,
        abs_path: &str,
        updated_at: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO project_roots (channel_h, abs_path, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(channel_h) DO UPDATE SET
                 abs_path=excluded.abs_path, updated_at=excluded.updated_at",
            params![channel_h, abs_path, updated_at],
        )?;
        Ok(())
    }

    /// The absolute path for a channel/project on this machine, if known.
    pub fn project_root(&self, channel_h: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT abs_path FROM project_roots WHERE channel_h=?1",
                params![channel_h],
                |r| r.get::<_, String>(0),
            )
            .optional()?)
    }
}
