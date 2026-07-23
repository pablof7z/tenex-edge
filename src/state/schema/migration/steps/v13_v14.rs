use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

pub(super) fn migrate(conn: &mut Connection, _path: &Path) -> Result<()> {
    super::require_shape(
        conn,
        13,
        "sessions",
        &["pubkey", "runtime_generation", "state_changed_at"],
        &[],
    )?;
    let tx = conn.transaction().context("starting schema-13 migration")?;
    tx.execute_batch(super::super::super::ddl::operational::NATIVE_TURN_SCHEMA)?;
    tx.pragma_update(None, "user_version", 14)?;
    tx.commit().context("committing schema-13 migration")
}
