//! Operational ledgers that explain local attempts and reconciler decisions.

pub(super) const OPERATIONAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS channel_readiness_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT, channel_h TEXT NOT NULL,
    expect_member TEXT NOT NULL DEFAULT '', parent_hint TEXT, name TEXT,
    source TEXT NOT NULL DEFAULT '', outcome TEXT NOT NULL DEFAULT '',
    reason TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_channel_readiness_attempts_channel
    ON channel_readiness_attempts(channel_h, created_at);

CREATE TABLE IF NOT EXISTS receipts (
    id INTEGER PRIMARY KEY AUTOINCREMENT, surface TEXT NOT NULL,
    transaction_id INTEGER NOT NULL, revision INTEGER NOT NULL,
    changed_summary TEXT NOT NULL, commands TEXT NOT NULL,
    artifact_ref TEXT, created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_receipts_surface ON receipts(surface, created_at);
CREATE INDEX IF NOT EXISTS idx_receipts_artifact_ref ON receipts(artifact_ref);
"#;

pub(in crate::state::schema) const NATIVE_TURN_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS native_turn_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey TEXT NOT NULL,
    runtime_generation INTEGER NOT NULL CHECK (runtime_generation > 0),
    delivery_kind TEXT NOT NULL
        CHECK (delivery_kind IN ('inbox_event', 'spawn_prompt')),
    delivery_event_id TEXT NOT NULL,
    native_thread_id TEXT NOT NULL CHECK (native_thread_id <> ''),
    native_turn_id TEXT NOT NULL DEFAULT '',
    outcome TEXT NOT NULL CHECK (outcome IN (
        'started', 'completed', 'failed', 'interrupted',
        'rejected_before_start', 'child_exited', 'unknown_reconciled'
    )),
    error_message TEXT NOT NULL DEFAULT '',
    error_details TEXT NOT NULL DEFAULT '',
    started_at INTEGER NOT NULL CHECK (started_at > 0),
    finished_at INTEGER NOT NULL DEFAULT 0,
    CHECK (
        (delivery_kind='inbox_event' AND delivery_event_id<>'')
        OR (delivery_kind='spawn_prompt' AND delivery_event_id='')
    ),
    CHECK (
        (outcome='started' AND finished_at=0)
        OR (outcome<>'started' AND finished_at>0)
    )
);
CREATE INDEX IF NOT EXISTS idx_native_turn_attempts_session
    ON native_turn_attempts(pubkey, runtime_generation, id DESC);
CREATE INDEX IF NOT EXISTS idx_native_turn_attempts_finished
    ON native_turn_attempts(finished_at);
"#;
