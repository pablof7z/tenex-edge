use super::{CommitStats, HistogramBucket, Store};
use anyhow::Result;
use rusqlite::params;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug)]
struct StatsRow {
    transaction_id: i64,
    revision: i64,
    resource_commands_json: String,
    output_frames_json: String,
    command_count: i64,
    output_count: i64,
    effect_count: i64,
    suppressed_count: i64,
    noop: i64,
    oracle_status: Option<String>,
    oracle_error: Option<String>,
    duration_us: i64,
    graph_nodes: i64,
    graph_resources: i64,
}

#[derive(Deserialize)]
struct KindOnly {
    kind: Option<String>,
}

#[derive(Default)]
struct CommandCounts {
    open: i64,
    close: i64,
    replace: i64,
    refresh: i64,
}

pub(super) fn commit_stats(store: &Store, surface: &str, since: i64) -> Result<CommitStats> {
    let mut stmt = store.conn.prepare(
        "SELECT transaction_id, revision, resource_commands_json, output_frames_json, command_count,
                output_count, effect_count, suppressed_count, noop, oracle_status,
                oracle_error, duration_us, graph_nodes, graph_resources
         FROM trellis_commits
         WHERE surface=?1 AND created_at >= ?2
         ORDER BY created_at ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![surface, since], |r| {
        Ok(StatsRow {
            transaction_id: r.get(0)?,
            revision: r.get(1)?,
            resource_commands_json: r.get(2)?,
            output_frames_json: r.get(3)?,
            command_count: r.get(4)?,
            output_count: r.get(5)?,
            effect_count: r.get(6)?,
            suppressed_count: r.get(7)?,
            noop: r.get(8)?,
            oracle_status: r.get(9)?,
            oracle_error: r.get(10)?,
            duration_us: r.get(11)?,
            graph_nodes: r.get(12)?,
            graph_resources: r.get(13)?,
        })
    })?;

    let mut out = CommitStats::default();
    let mut durations = BTreeMap::new();
    let mut graph_nodes = BTreeMap::new();
    let mut graph_resources = BTreeMap::new();
    let mut last_transaction_id = None;
    let mut last_revision = None;
    let mut epoch_balance = None;

    for row in rows {
        let row = row?;
        let counts = count_command_kinds(&row.resource_commands_json);
        let delta = counts.open - counts.close;
        let epoch_restarted = last_transaction_id.is_some_and(|prev| row.transaction_id < prev)
            || last_revision.is_some_and(|prev| row.revision < prev);
        if epoch_balance.is_none() || epoch_restarted {
            epoch_balance = Some(row.graph_resources - delta);
        }
        out.commits += 1;
        out.effectful += i64::from(row.noop == 0);
        out.noop += row.noop;
        out.command_count_sum += row.command_count;
        out.output_count_sum += row.output_count;
        out.effect_count_sum += row.effect_count;
        out.suppressed_count_sum += row.suppressed_count;
        out.duration_us_sum += row.duration_us;
        out.max_graph_nodes = out.max_graph_nodes.max(row.graph_nodes);
        out.max_graph_resources = out.max_graph_resources.max(row.graph_resources);
        out.latest_graph_resources = row.graph_resources;
        if let Some(status) = row.oracle_status {
            out.latest_oracle_status = Some(status);
            out.latest_oracle_error = row.oracle_error;
        }

        out.open_count += counts.open;
        out.close_count += counts.close;
        out.replace_count += counts.replace;
        out.refresh_count += counts.refresh;
        epoch_balance = epoch_balance.map(|balance| balance + delta);
        last_transaction_id = Some(row.transaction_id);
        last_revision = Some(row.revision);
        if surface == "hook_context" {
            out.hook_unchanged_frames += count_kind(&row.output_frames_json, "unchanged");
        }
        bump(&mut durations, duration_bucket(row.duration_us));
        bump(&mut graph_nodes, size_bucket(row.graph_nodes));
        bump(&mut graph_resources, size_bucket(row.graph_resources));
    }

    out.live_resource_balance = epoch_balance.unwrap_or_default();
    out.resource_drift =
        surface == "subscriptions" && out.live_resource_balance != out.latest_graph_resources;
    out.duration_histogram = buckets(durations);
    out.graph_nodes_histogram = buckets(graph_nodes);
    out.graph_resources_histogram = buckets(graph_resources);
    Ok(out)
}

fn count_command_kinds(json: &str) -> CommandCounts {
    let mut out = CommandCounts::default();
    for kind in kinds(json) {
        match kind.as_str() {
            "open" => out.open += 1,
            "close" => out.close += 1,
            "replace" => out.replace += 1,
            "refresh" => out.refresh += 1,
            _ => {}
        }
    }
    out
}

fn count_kind(json: &str, target: &str) -> i64 {
    kinds(json).into_iter().filter(|k| k == target).count() as i64
}

fn kinds(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<KindOnly>>(json)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.kind)
        .collect()
}

fn bump(hist: &mut BTreeMap<&'static str, i64>, bucket: &'static str) {
    *hist.entry(bucket).or_default() += 1;
}

fn buckets(hist: BTreeMap<&'static str, i64>) -> Vec<HistogramBucket> {
    hist.into_iter()
        .map(|(bucket, count)| HistogramBucket {
            bucket: bucket.to_string(),
            count,
        })
        .collect()
}

fn duration_bucket(duration_us: i64) -> &'static str {
    match duration_us {
        i64::MIN..=999 => "<1ms",
        1_000..=9_999 => "1-9ms",
        10_000..=99_999 => "10-99ms",
        100_000..=999_999 => "100-999ms",
        _ => ">=1s",
    }
}

fn size_bucket(size: i64) -> &'static str {
    match size {
        i64::MIN..=0 => "0",
        1..=9 => "1-9",
        10..=99 => "10-99",
        100..=999 => "100-999",
        _ => ">=1000",
    }
}
