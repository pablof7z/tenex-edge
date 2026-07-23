use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

pub(super) fn migrate(conn: &mut Connection, _path: &Path) -> Result<()> {
    let profile_columns = table_columns(conn, "relay_profiles")?;
    let profile_exists = !profile_columns.is_empty();
    let add_profile_columns = if profile_exists {
        super::require_shape(
            conn,
            16,
            "relay_profiles",
            &["pubkey", "host", "is_backend", "updated_at"],
            &[],
        )?;
        match (
            profile_columns.contains("agents_json"),
            profile_columns.contains("workspaces_json"),
        ) {
            (false, false) => true,
            (true, true) => false,
            _ => anyhow::bail!(
                "schema 16 table `relay_profiles` has a partial host-profile snapshot shape"
            ),
        }
    } else {
        false
    };
    let roster_exists = table_exists(conn, "relay_agent_roster")?;
    if roster_exists {
        super::require_shape(
            conn,
            16,
            "relay_agent_roster",
            &["backend_pubkey", "agent_slug", "channel_h", "use_criteria"],
            &[],
        )?;
    }
    let tx = conn.transaction().context("starting schema-16 migration")?;
    if add_profile_columns {
        tx.execute_batch(
            r#"
        ALTER TABLE relay_profiles
            ADD COLUMN agents_json TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE relay_profiles
            ADD COLUMN workspaces_json TEXT NOT NULL DEFAULT '[]';
        "#,
        )?;
    }
    if profile_exists {
        tx.execute("DELETE FROM relay_profiles WHERE is_backend=1", [])?;
    }
    if roster_exists {
        tx.execute("DROP TABLE relay_agent_roster", [])?;
    }
    tx.pragma_update(None, "user_version", 17)?;
    tx.commit().context("committing schema-16 migration")
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [table],
        |row| row.get(0),
    )?)
}

fn table_columns(conn: &Connection, table: &str) -> Result<std::collections::BTreeSet<String>> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(columns)
}
