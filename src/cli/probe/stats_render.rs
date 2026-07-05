use serde_json::Value;

/// The `probe stats` table: one row per surface with the suppression evidence.
pub(super) fn render_stats(v: &Value) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let since = v.get("since").and_then(Value::as_i64).unwrap_or(0);
    let _ = writeln!(out, "probe stats  (since={since})\n");
    let _ = writeln!(
        out,
        "{:<14} {:>7} {:>6} {:>6} {:>7} {:>7} {:>7} {:>6} {:>7}",
        "surface", "commits", "noop", "supp", "replace", "refresh", "open", "close", "live",
    );
    let empty = Vec::new();
    let surfaces = v
        .get("surfaces")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    for r in surfaces {
        let s = |k| r.get(k).and_then(Value::as_str).unwrap_or("");
        let n = |k| r.get(k).and_then(Value::as_i64).unwrap_or(0);
        let _ = writeln!(
            out,
            "{:<14} {:>7} {:>6} {:>6} {:>7} {:>7} {:>7} {:>6} {:>7}",
            s("surface"),
            n("commits"),
            n("noop"),
            n("suppressed_count_sum"),
            n("replace_count"),
            n("refresh_count"),
            n("open_count"),
            n("close_count"),
            live_cell(r),
        );
        if n("hook_unchanged_frames") > 0 {
            let _ = writeln!(
                out,
                "  hook unchanged frames: {}",
                n("hook_unchanged_frames")
            );
        }
        let _ = writeln!(
            out,
            "  duration: {}  graph-nodes: {}  graph-resources: {}",
            histogram(r, "duration_histogram"),
            histogram(r, "graph_nodes_histogram"),
            histogram(r, "graph_resources_histogram"),
        );
    }
    out
}

fn live_cell(r: &Value) -> String {
    let balance = r
        .get("live_resource_balance")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let live = r
        .get("latest_graph_resources")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if r.get("resource_drift").and_then(Value::as_bool) == Some(true) {
        format!("{balance}!={live}")
    } else {
        live.to_string()
    }
}

fn histogram(r: &Value, key: &str) -> String {
    let Some(items) = r.get(key).and_then(Value::as_array) else {
        return "-".into();
    };
    let parts = items
        .iter()
        .filter_map(|item| {
            Some(format!(
                "{}:{}",
                item.get("bucket")?.as_str()?,
                item.get("count")?.as_i64()?
            ))
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "-".into()
    } else {
        parts.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stats_render_tabulates_each_surface() {
        let text = render_stats(&json!({
            "verb": "stats",
            "since": 0,
            "surfaces": [
                { "surface": "status", "commits": 3, "noop": 1,
                  "suppressed_count_sum": 1, "latest_graph_resources": 2,
                  "open_count": 1, "close_count": 0, "replace_count": 1,
                  "refresh_count": 1, "live_resource_balance": 1,
                  "resource_drift": false, "hook_unchanged_frames": 0,
                  "duration_histogram": [{"bucket":"<1ms","count":3}],
                  "graph_nodes_histogram": [{"bucket":"1-9","count":3}],
                  "graph_resources_histogram": [{"bucket":"1-9","count":3}] },
                { "surface": "subscriptions", "commits": 1, "noop": 0,
                  "suppressed_count_sum": 0, "latest_graph_resources": 1,
                  "open_count": 1, "close_count": 0, "replace_count": 0,
                  "refresh_count": 0, "live_resource_balance": 1,
                  "resource_drift": false, "hook_unchanged_frames": 0,
                  "duration_histogram": [{"bucket":"<1ms","count":1}],
                  "graph_nodes_histogram": [{"bucket":"1-9","count":1}],
                  "graph_resources_histogram": [{"bucket":"1-9","count":1}] },
            ],
        }));
        assert!(text.contains("probe stats"));
        assert!(text.contains("status"));
        assert!(text.contains("subscriptions"));
        assert!(text.contains("duration: <1ms:3"));
    }
}
