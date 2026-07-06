use super::super::report::{bool_at, str_at};
use crate::daemon::server::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn evidence(
    state: &Arc<DaemonState>,
    session_id: &str,
    session_channel: &Value,
) -> Value {
    let graphs = state
        .hook_contexts
        .lock()
        .expect("hook-context mutex poisoned");
    let Some(graph) = graphs.get(session_id) else {
        return json!({
            "graph_found": false,
            "resource_key": format!("hook/{session_id}/view"),
        });
    };
    let text = graph.current_text();
    let channel_h = str_at(session_channel, "channel_h");
    let channel_confirmed = bool_at(session_channel, "confirmed");
    let rendered_unconfirmed_channel = text
        .as_ref()
        .is_some_and(|text| !channel_confirmed && renders_channel_block(text, channel_h));
    let missing_channel_warning_rendered = text
        .as_ref()
        .is_some_and(|text| renders_missing_channel_warning(text, channel_h));
    let rendered_local_agents = text
        .as_ref()
        .is_some_and(|text| text.contains("<available-agents>"));
    let rendered_member_roster = text.as_ref().is_some_and(|text| text.contains("<members>"));
    let rendered_legacy_agents_roster = text.as_ref().is_some_and(|text| text.contains("<agents>"));
    json!({
        "graph_found": true,
        "resource_key": graph
            .view_label()
            .unwrap_or_else(|| format!("hook/{session_id}/view")),
        "revision": graph.revision(),
        "nodes": graph.graph_node_count(),
        "render_count": graph.render_count(),
        "emitted": text.is_some(),
        "text_bytes": text.as_ref().map(String::len).unwrap_or(0),
        "rendered_unconfirmed_channel": rendered_unconfirmed_channel,
        "missing_channel_warning_rendered": missing_channel_warning_rendered,
        "rendered_local_agents": rendered_local_agents,
        "rendered_member_roster": rendered_member_roster,
        "rendered_legacy_agents_roster": rendered_legacy_agents_roster,
        "local_agent_rows": text.as_ref().map(|text| count_marker(text, "<agent ref=\"@")).unwrap_or(0),
        "member_rows": text.as_ref().map(|text| count_marker(text, "<member ref=\"@")).unwrap_or(0),
        "input_labels": graph.input_labels(),
        "why_input_causes": graph.why_view_input_causes(),
    })
}

fn renders_channel_block(text: &str, channel_h: &str) -> bool {
    !channel_h.is_empty() && text.contains(&format!("<channel name=\"#{channel_h}\""))
}

fn renders_missing_channel_warning(text: &str, channel_h: &str) -> bool {
    !channel_h.is_empty()
        && text.contains(&format!("Fabric channel \"{channel_h}\" is unavailable"))
}

fn count_marker(text: &str, marker: &str) -> usize {
    text.match_indices(marker).count()
}
