use anyhow::{bail, Context, Result};
use rusqlite::{Connection, Transaction};
use std::path::Path;

pub(super) const SCHEMA_VERSION: u32 = 6;

/// Upgrade the only schema-5 layout that was deployed without a corresponding
/// version bump. This is a one-way conversion: legacy tables are removed in the
/// same transaction; the daemon never dual-reads them.
pub(super) fn migrate_legacy_v5(conn: &mut Connection, path: &Path, ddl: &str) -> Result<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != 5 || !table_exists(conn, "session_aliases")? {
        return Ok(());
    }
    let tx = conn
        .transaction()
        .with_context(|| format!("migrating {}", path.display()))?;
    migrate_v5(&tx, ddl)?;
    tx.commit().context("committing schema-5 migration")
}

fn migrate_v5(tx: &Transaction<'_>, ddl: &str) -> Result<()> {
    // Renaming a table retains its index names; release the names that schema 6
    // owns before recreating the canonical tables.
    for index in [
        "idx_sessions_alive",
        "idx_session_channels_channel",
        "idx_session_claims_expires",
        "idx_llm_calls_pubkey",
        "idx_llm_calls_window_hash",
    ] {
        tx.execute_batch(&format!("DROP INDEX IF EXISTS {index};"))?;
    }
    for table in [
        "relay_channels",
        "relay_channel_members",
        "relay_channel_member_sets",
        "relay_profiles",
        "relay_status",
        "relay_agent_roster",
        "relay_events",
        "relay_reactions",
        "relay_event_quarantine",
        "identities",
        "durable_agent_sessions",
    ] {
        tx.execute_batch(&format!("DROP TABLE IF EXISTS {table};"))?;
    }
    for table in [
        "sessions",
        "session_channels",
        "session_aliases",
        "session_claims",
        "llm_calls",
    ] {
        tx.execute_batch(&format!("ALTER TABLE {table} RENAME TO legacy_{table};"))?;
    }
    tx.execute_batch(ddl).context("creating schema-6 tables")?;
    tx.execute_batch(
        r#"
        INSERT OR REPLACE INTO sessions (
            pubkey, runtime_generation, agent_slug, channel_h, harness, child_pid,
            transcript_path, alive, created_at, last_seen, working, turn_started_at,
            last_distill_at, work_topic, work_topic_set_at, seen_cursor, title, activity,
            distill_fail_streak, distill_notice_at, explicit_chat_published_at
        ) SELECT agent_pubkey, 0, agent_slug, channel_h, harness, child_pid,
            transcript_path, alive, created_at, last_seen, working, turn_started_at,
            last_distill_at, work_topic, work_topic_set_at, seen_cursor, title, activity,
            distill_fail_streak, distill_notice_at, explicit_chat_published_at
          FROM legacy_sessions ORDER BY last_seen;

        INSERT OR IGNORE INTO session_channels (pubkey, channel_h, joined_at)
        SELECT s.agent_pubkey, c.channel_h, c.joined_at
          FROM legacy_session_channels c JOIN legacy_sessions s USING (session_id);

        INSERT OR IGNORE INTO session_locators (harness, locator_kind, locator_value, pubkey, created_at)
        SELECT a.harness,
               CASE a.external_id_kind
                   WHEN 'harness_session' THEN 'native_resume'
                   WHEN 'pty_session' THEN 'pty'
                   WHEN 'watch_pid' THEN 'pid'
                   ELSE 'acp'
               END,
               a.external_id, s.agent_pubkey, a.created_at
          FROM legacy_session_aliases a JOIN legacy_sessions s USING (session_id);

        INSERT OR REPLACE INTO session_claims (
            pubkey, agent_slug, channel_h, harness, last_active_at, expires_at,
            owner_backend_pubkey, owner_host
        ) SELECT pubkey, agent_slug, channel_h, harness, last_active_at, expires_at,
            owner_backend_pubkey, owner_host FROM legacy_session_claims;

        INSERT INTO llm_calls (
            pubkey, window_hash, provider, model, system_prompt, transcript_slice,
            raw_response, parsed_title, parsed_activity, created_at
        ) SELECT s.agent_pubkey, l.window_hash, l.provider, l.model, l.system_prompt,
            l.transcript_slice, l.raw_response, l.parsed_title, l.parsed_activity, l.created_at
          FROM legacy_llm_calls l JOIN legacy_sessions s USING (session_id);

        DROP TABLE legacy_sessions;
        DROP TABLE legacy_session_channels;
        DROP TABLE legacy_session_aliases;
        DROP TABLE legacy_session_claims;
        DROP TABLE legacy_llm_calls;
        PRAGMA user_version = 6;
        "#,
    )?;
    Ok(())
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [table],
        |row| row.get(0),
    )
    .context("checking schema table")
}

pub(super) fn stamp(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)
        .context("stamping schema version")
}

pub(super) fn check(conn: &Connection, path: &Path) -> Result<()> {
    let version: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .context("reading schema user_version")?;
    let has_tables = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type='table' AND name NOT LIKE 'sqlite_%'
            )",
            [],
            |row| row.get::<_, bool>(0),
        )
        .context("checking for existing schema tables")?;
    if version == 0 && has_tables {
        anyhow::bail!(
            "refusing to open {}: existing state.db has no schema version stamp; \
             move it aside or export non-rebuildable local state before rebuilding",
            path.display()
        );
    }
    if version != 0 && version != SCHEMA_VERSION {
        bail!(
            "refusing to open {}: schema version {version} is incompatible with expected {SCHEMA_VERSION}",
            path.display()
        );
    }
    Ok(())
}
