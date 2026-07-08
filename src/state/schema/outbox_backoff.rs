//! Idempotent migration: add `outbox.next_attempt_at` to DBs created before the
//! publish-retry backoff (issue #295). Mirrors the `trellis_commits` /
//! `session_claims` `ensure_columns` pattern so an existing on-disk queue gains
//! the column without a schema-version bump.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::BTreeSet;

const COLUMNS: &[(&str, &str)] = &[("next_attempt_at", "INTEGER NOT NULL DEFAULT 0")];

pub(super) fn ensure_columns(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(outbox)")
        .context("reading outbox columns")?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<BTreeSet<_>>>()?;

    for (name, spec) in COLUMNS {
        if !existing.contains(*name) {
            conn.execute(&format!("ALTER TABLE outbox ADD COLUMN {name} {spec}"), [])
                .with_context(|| format!("adding outbox.{name}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_next_attempt_at_to_pre_backoff_outbox() {
        let conn = Connection::open_in_memory().unwrap();
        // an outbox table shaped like the pre-#295 schema (no next_attempt_at)
        conn.execute_batch(
            "CREATE TABLE outbox (
                local_id     INTEGER PRIMARY KEY AUTOINCREMENT,
                event_json   TEXT NOT NULL,
                state        TEXT NOT NULL DEFAULT 'pending',
                retries      INTEGER NOT NULL DEFAULT 0,
                last_error   TEXT,
                enqueued_at  INTEGER NOT NULL
            )",
        )
        .unwrap();

        ensure_columns(&conn).unwrap();
        // idempotent second run must not error
        ensure_columns(&conn).unwrap();

        let cols = conn
            .prepare("PRAGMA table_info(outbox)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<BTreeSet<_>>>()
            .unwrap();
        assert!(cols.contains("next_attempt_at"), "column not added");
    }
}
