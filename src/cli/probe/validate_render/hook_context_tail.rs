//! Hook context evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "hook context evidence");
    let graph = evidence.get("graph").unwrap_or(&Value::Null);
    let receipt = evidence.get("receipt").unwrap_or(&Value::Null);
    let graph_status = if bool_at(evidence, "graph_found") {
        "present"
    } else {
        "missing"
    };
    let receipt_status = if bool_at(evidence, "receipt_found") {
        "present"
    } else {
        "missing"
    };
    let _ = writeln!(
        out,
        "  - {}: graph={} receipt={} revision_match={}",
        str_at(evidence, "session_id"),
        graph_status,
        receipt_status,
        bool_at(evidence, "revision_matches_receipt")
    );
    if bool_at(evidence, "graph_found") {
        let _ = writeln!(
            out,
            "  - graph revision={} nodes={} renders={} emitted={} bytes={}",
            int_at(graph, "revision"),
            int_at(graph, "nodes"),
            int_at(graph, "render_count"),
            bool_at(graph, "emitted"),
            int_at(graph, "text_bytes")
        );
        render_strings(out, "causes", graph.get("why_input_causes"));
        let _ = writeln!(
            out,
            "  - roster local_agents={} legacy_agents={} members={} corroborated={} local_rows={} member_rows={}",
            bool_at(graph, "rendered_local_agents"),
            bool_at(graph, "rendered_legacy_agents_roster"),
            bool_at(graph, "rendered_member_roster"),
            bool_at(evidence, "member_roster_corroborated"),
            int_at(graph, "local_agent_rows"),
            int_at(graph, "member_rows")
        );
        if bool_at(graph, "rendered_unconfirmed_channel") {
            let _ = writeln!(out, "  - rendered_unconfirmed_channel=true");
        }
        if bool_at(graph, "missing_channel_warning_rendered") {
            let _ = writeln!(out, "  - missing_channel_warning_rendered=true");
        }
    }
    if let Some(channel) = evidence.get("session_channel").filter(|v| !v.is_null()) {
        let _ = writeln!(
            out,
            "  - session channel={} confirmed={} membership_snapshot={} members={} admins={}",
            str_at(channel, "channel_h"),
            bool_at(channel, "confirmed"),
            bool_at(channel, "membership_snapshot"),
            int_at(channel, "member_count"),
            int_at(channel, "admin_count")
        );
    }
    if bool_at(evidence, "receipt_found") {
        let _ = writeln!(
            out,
            "  - receipt id={} txn={} rev={} kind={} frame={} shape={} artifact={:?}",
            int_at(receipt, "id"),
            int_at(receipt, "transaction_id"),
            int_at(receipt, "revision"),
            str_at(receipt, "kind"),
            str_at(receipt, "frame"),
            str_at(receipt, "shape"),
            str_at(receipt, "artifact_ref")
        );
        render_strings(out, "receipt_causes", receipt.get("input_causes"));
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn render_strings(out: &mut String, label: &str, value: Option<&Value>) {
    let items = value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    if !items.is_empty() {
        let _ = writeln!(out, "  - {label}={}", items.join(","));
    }
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn int_at(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}
