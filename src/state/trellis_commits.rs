//! `trellis_commits` — the all-commit ledger (frontier design §4.1).
//!
//! Receipts (`state::receipts`) exist only for EFFECTFUL commits, so the value
//! evidence — suppressed publishes, no-op recomputes, unchanged hook frames — is
//! invisible. This store records EVERY transaction, effectful or not, so
//! `probe stats` can quantify "N commits, M suppressed, K no-ops" for a surface.
//!
//! Like `receipts`, this module is pure: no Trellis types cross it (the caller
//! flattens a `TransactionResult` into label arrays + counts via
//! `reconcile::labels::CommitFacts` before recording), and it never reads the
//! clock — the caller passes `created_at`. `transaction_id`/`revision` are stored
//! as `INTEGER` (Trellis's monotonic `u64` counters fit an `i64` column).

use super::*;

mod stats;
#[cfg(test)]
mod tests;

const COLS: &str = "id, surface, transaction_id, revision, mode, trigger_kind, trigger_ref, \
     changed_inputs_json, changed_derived_json, changed_collections_json, \
     resource_commands_json, output_frames_json, command_count, output_count, \
     effect_count, suppressed_count, noop, oracle_status, oracle_error, \
     duration_us, graph_nodes, graph_resources, created_at";

/// One persisted all-commit ledger row, flattened to plain fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRow {
    pub id: i64,
    pub surface: String,
    pub transaction_id: i64,
    pub revision: i64,
    pub mode: String,
    pub trigger_kind: String,
    pub trigger_ref: String,
    pub changed_inputs_json: String,
    pub changed_derived_json: String,
    pub changed_collections_json: String,
    pub resource_commands_json: String,
    pub output_frames_json: String,
    pub command_count: i64,
    pub output_count: i64,
    pub effect_count: i64,
    pub suppressed_count: i64,
    pub noop: i64,
    pub oracle_status: Option<String>,
    pub oracle_error: Option<String>,
    pub duration_us: i64,
    pub graph_nodes: i64,
    pub graph_resources: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCommit {
    pub surface: String,
    pub transaction_id: i64,
    pub revision: i64,
    pub mode: String,
    pub trigger_kind: String,
    pub trigger_ref: String,
    pub changed_inputs_json: String,
    pub changed_derived_json: String,
    pub changed_collections_json: String,
    pub resource_commands_json: String,
    pub output_frames_json: String,
    pub command_count: i64,
    pub output_count: i64,
    pub effect_count: i64,
    pub suppressed_count: i64,
    pub noop: i64,
    pub oracle_status: Option<String>,
    pub oracle_error: Option<String>,
    pub duration_us: i64,
    pub graph_nodes: i64,
    pub graph_resources: i64,
    pub created_at: i64,
}

/// Aggregate value evidence for a surface over a window, powering `probe stats`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitStats {
    pub commits: i64,
    pub effectful: i64,
    pub noop: i64,
    pub command_count_sum: i64,
    pub output_count_sum: i64,
    pub effect_count_sum: i64,
    pub suppressed_count_sum: i64,
    pub duration_us_sum: i64,
    pub max_graph_nodes: i64,
    pub max_graph_resources: i64,
    pub latest_graph_resources: i64,
    pub open_count: i64,
    pub close_count: i64,
    pub replace_count: i64,
    pub refresh_count: i64,
    pub live_resource_balance: i64,
    pub resource_drift: bool,
    pub hook_unchanged_frames: i64,
    pub duration_histogram: Vec<HistogramBucket>,
    pub graph_nodes_histogram: Vec<HistogramBucket>,
    pub graph_resources_histogram: Vec<HistogramBucket>,
    pub latest_oracle_status: Option<String>,
    pub latest_oracle_error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistogramBucket {
    pub bucket: String,
    pub count: i64,
}

fn row_to_commit(row: &rusqlite::Row) -> rusqlite::Result<CommitRow> {
    Ok(CommitRow {
        id: row.get(0)?,
        surface: row.get(1)?,
        transaction_id: row.get(2)?,
        revision: row.get(3)?,
        mode: row.get(4)?,
        trigger_kind: row.get(5)?,
        trigger_ref: row.get(6)?,
        changed_inputs_json: row.get(7)?,
        changed_derived_json: row.get(8)?,
        changed_collections_json: row.get(9)?,
        resource_commands_json: row.get(10)?,
        output_frames_json: row.get(11)?,
        command_count: row.get(12)?,
        output_count: row.get(13)?,
        effect_count: row.get(14)?,
        suppressed_count: row.get(15)?,
        noop: row.get(16)?,
        oracle_status: row.get(17)?,
        oracle_error: row.get(18)?,
        duration_us: row.get(19)?,
        graph_nodes: row.get(20)?,
        graph_resources: row.get(21)?,
        created_at: row.get(22)?,
    })
}

impl Store {
    /// Record one flattened commit. Returns the assigned `id`.
    pub fn record_commit(&self, row: &NewCommit) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO trellis_commits
                 (surface, transaction_id, revision, mode, trigger_kind, trigger_ref,
                  changed_inputs_json, changed_derived_json, changed_collections_json,
                  resource_commands_json, output_frames_json,
                  command_count, output_count, effect_count, suppressed_count, noop,
                  oracle_status, oracle_error, duration_us, graph_nodes, graph_resources, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                     ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                row.surface,
                row.transaction_id,
                row.revision,
                row.mode,
                row.trigger_kind,
                row.trigger_ref,
                row.changed_inputs_json,
                row.changed_derived_json,
                row.changed_collections_json,
                row.resource_commands_json,
                row.output_frames_json,
                row.command_count,
                row.output_count,
                row.effect_count,
                row.suppressed_count,
                row.noop,
                row.oracle_status,
                row.oracle_error,
                row.duration_us,
                row.graph_nodes,
                row.graph_resources,
                row.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Most recent commits for a surface, newest first, capped at `limit`.
    pub fn latest_commits_for_surface(&self, surface: &str, limit: u32) -> Result<Vec<CommitRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM trellis_commits
             WHERE surface=?1
             ORDER BY created_at DESC, id DESC LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![surface, limit], row_to_commit)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Aggregate value evidence for `surface` over commits with
    /// `created_at >= since`. Pure over the ledger — the proof `probe stats` works.
    pub fn commit_stats(&self, surface: &str, since: i64) -> Result<CommitStats> {
        stats::commit_stats(self, surface, since)
    }

    /// Stamp the newest ledger row for `surface` with a sampled oracle result.
    pub fn record_oracle_sample(
        &self,
        surface: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<usize> {
        Ok(self.conn.execute(
            "UPDATE trellis_commits
             SET oracle_status=?2, oracle_error=?3
             WHERE id=(
                 SELECT id FROM trellis_commits
                 WHERE surface=?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1
             )",
            params![surface, status, error],
        )?)
    }
}
