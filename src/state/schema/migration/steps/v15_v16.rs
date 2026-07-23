use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

pub(super) fn migrate(conn: &mut Connection, _path: &Path) -> Result<()> {
    super::require_shape(
        conn,
        15,
        "sessions",
        &[
            "pubkey",
            "runtime_generation",
            "explicit_chat_published_at",
            "transcript_path",
        ],
        &[],
    )?;
    let tx = conn.transaction().context("starting schema-15 migration")?;
    tx.execute_batch(
        "ALTER TABLE sessions DROP COLUMN explicit_chat_published_at;
         ALTER TABLE sessions DROP COLUMN transcript_path;",
    )?;
    tx.pragma_update(None, "user_version", 16)?;
    tx.commit().context("committing schema-15 migration")
}
