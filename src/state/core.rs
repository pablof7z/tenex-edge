use super::*;

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path).with_context(|| format!("opening {}", path.display()))?;
        // WAL stopgap (M1 daemon migration, stage 1): until the per-machine daemon
        // is the sole writer, many processes (per-session engines + every CLI
        // invocation) share this file. WAL + a busy timeout + relaxed sync is the
        // bandage that stops the multi-writer corruption we recovered from. It
        // stays harmless (and a touch faster) once the daemon owns the db.
        //   journal_mode=WAL   readers don't block the writer; one writer at a time
        //   busy_timeout=5000  block up to 5s on a held lock instead of erroring
        //   synchronous=NORMAL safe under WAL; fsync only at checkpoints
        // No foreign_keys pragma: the schema declares no FK constraints.
        conn.pragma_update(None, "journal_mode", "WAL").ok();
        conn.pragma_update(None, "synchronous", "NORMAL").ok();
        conn.busy_timeout(std::time::Duration::from_secs(5)).ok();
        conn.execute_batch(SCHEMA).context("creating schema")?;
        // Migrations (ignore if column already present).
        // NIP-29 subgroup hierarchy columns on project_meta (issue #3).
        let _ = conn.execute(
            "ALTER TABLE project_meta ADD COLUMN name TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE project_meta ADD COLUMN parent TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE sessions ADD COLUMN last_seen INTEGER NOT NULL DEFAULT 0",
            [],
        );
        // Issue #6: mark owned groups that are per-session rooms (vs project /
        // task groups), so only the owning session auto-renames them to its
        // distilled title.
        let _ = conn.execute(
            "ALTER TABLE owned_groups ADD COLUMN is_session_room INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE owned_groups ADD COLUMN owns_group INTEGER NOT NULL DEFAULT 1",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE owned_groups ADD COLUMN room_parent TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN transcript_path TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE peer_sessions ADD COLUMN first_seen INTEGER NOT NULL DEFAULT 0",
            [],
        );
        // §8e: project-relative cwd, on own sessions and the peer directory.
        let _ = conn.execute(
            "ALTER TABLE sessions ADD COLUMN rel_cwd TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE sessions ADD COLUMN channel TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE peer_sessions ADD COLUMN rel_cwd TEXT NOT NULL DEFAULT ''",
            [],
        );
        // Snapshot of the last assistant text at the beginning of each turn.
        // Used by rpc_turn_end to poll until a *new* response appears in the
        // transcript (Claude Code writes the transcript after the stop hook fires).
        let _ = conn.execute(
            "ALTER TABLE sessions ADD COLUMN last_assistant_text_at_turn_start TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE chat_inbox ADD COLUMN notified_at INTEGER NOT NULL DEFAULT 0",
            [],
        );
        // Session-state rearchitecture: the legacy `agent_status` / `session_status`
        // tables are replaced wholesale by the canonical `session_state` +
        // `peer_session_state` aggregate. No backwards compatibility — drop them so
        // a stale schema can't be read by accident.
        let _ = conn.execute("DROP TABLE IF EXISTS agent_status", []);
        let _ = conn.execute("DROP TABLE IF EXISTS session_status", []);
        // Issue #5 §4: peer_session_state PK changed from (pubkey, project,
        // native_session_id) to (pubkey, project). No backwards compat — drop and
        // recreate when the old native_session_id column is still present.
        {
            let has_old: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('peer_session_state') WHERE name='native_session_id'",
                    [],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if has_old {
                conn.execute_batch(
                    "DROP TABLE IF EXISTS peer_session_state;
                     DROP INDEX IF EXISTS idx_peer_session_state_project_seen;
                     CREATE TABLE IF NOT EXISTS peer_session_state (
                         pubkey            TEXT NOT NULL,
                         project           TEXT NOT NULL,
                         agent_slug        TEXT NOT NULL DEFAULT '',
                         host              TEXT NOT NULL DEFAULT '',
                         rel_cwd           TEXT NOT NULL DEFAULT '',
                         title             TEXT NOT NULL DEFAULT '',
                         activity          TEXT NOT NULL DEFAULT '',
                         busy              INTEGER NOT NULL DEFAULT 0,
                         last_seen         INTEGER NOT NULL DEFAULT 0,
                         state_version     INTEGER NOT NULL DEFAULT 0,
                         lifecycle         TEXT NOT NULL DEFAULT 'active',
                         first_seen        INTEGER NOT NULL DEFAULT 0,
                         updated_at        INTEGER NOT NULL DEFAULT 0,
                         PRIMARY KEY (pubkey, project)
                     );
                     CREATE INDEX IF NOT EXISTS idx_peer_session_state_project_seen
                         ON peer_session_state(project, last_seen);",
                )
                .ok();
            }
        }
        // Harness-native resume token (e.g. the id `claude --resume <id>` /
        // `codex resume <id>` / `opencode --session <id>` wants). For claude-code
        // and codex this equals `session_id` (they assign their own id, which we
        // adopt); for opencode it is the `ses_*` id the plugin forwards, distinct
        // from our synthetic `te-*` identity. Empty = not resumable.
        let _ = conn.execute(
            "ALTER TABLE sessions ADD COLUMN resume_id TEXT NOT NULL DEFAULT ''",
            [],
        );
        // Mid-turn delta cursor: timestamp of the last PostToolUse `turn_check`
        // for this session. Lets each in-turn check report only sibling-session
        // changes since the previous check (gated by deltas), instead of re-
        // emitting the whole roster on every tool call. Reset to 0 at turn start.
        let _ = conn.execute(
            "ALTER TABLE turn_state ADD COLUMN last_check_at INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE profiles ADD COLUMN is_backend INTEGER NOT NULL DEFAULT 0",
            [],
        );
        Ok(Self { conn })
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// `PRAGMA integrity_check` → "ok" on a healthy db, else the first problem
    /// line. Used by the concurrency/corruption test to assert no corruption.
    pub fn integrity_check(&self) -> Result<String> {
        Ok(self
            .conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))?)
    }
}
