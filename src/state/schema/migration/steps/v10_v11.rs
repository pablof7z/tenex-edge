use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::require_shape;

pub(super) fn migrate(conn: &mut Connection, _path: &Path) -> Result<()> {
    require_shape(
        conn,
        10,
        "sessions",
        &["pubkey", "runtime_state", "work_state"],
        &["alive", "working"],
    )?;
    require_shape(
        conn,
        10,
        "inbox",
        &["event_id", "target_pubkey", "state"],
        &[],
    )?;
    let tx = conn.transaction().context("starting schema-10 migration")?;
    tx.execute_batch(
        r#"
        UPDATE inbox
           SET state='echo_consumed'
         WHERE state='injected'
           AND EXISTS (
               SELECT 1 FROM sessions session
                WHERE session.pubkey=inbox.target_pubkey
                  AND session.work_state='idle'
           );
        PRAGMA user_version = 11;
        "#,
    )?;
    tx.commit().context("committing schema-10 migration")
}
