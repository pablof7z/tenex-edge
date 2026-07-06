//! `project_roots` — local filesystem map (channel/project id -> abs path on THIS
//! machine). The one fact about a project that is genuinely machine-local.

use super::*;

/// Local filesystem binding for a fabric project/root channel on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRootBinding {
    pub channel_h: String,
    pub abs_path: String,
    pub updated_at: u64,
}

fn row_to_project_root(row: &rusqlite::Row) -> rusqlite::Result<ProjectRootBinding> {
    Ok(ProjectRootBinding {
        channel_h: row.get(0)?,
        abs_path: row.get(1)?,
        updated_at: row.get(2)?,
    })
}

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

    /// The local filesystem binding row for a channel/project on this machine.
    pub fn project_root_binding(&self, channel_h: &str) -> Result<Option<ProjectRootBinding>> {
        Ok(self
            .conn
            .query_row(
                "SELECT channel_h, abs_path, updated_at FROM project_roots WHERE channel_h=?1",
                params![channel_h],
                row_to_project_root,
            )
            .optional()?)
    }
}
