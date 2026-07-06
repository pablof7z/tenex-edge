use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_coverage_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "coverage",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"validation_coverage","status":"passed","summary":"validator maps 21/21 live durable table(s)"}
        ],
        "coverage_evidence": {
            "coverage_ok": true,
            "table_count": 21,
            "covered_table_count": 21,
            "direct_table_count": 20,
            "aggregate_table_count": 1,
            "uncovered_tables": [],
            "durable_tables": [
                {"table":"channel_readiness_attempts","mode":"direct","targets":"readiness_attempt:<id>","proves":"provider readiness decisions"},
                {"table":"trellis_replay_capsules","mode":"direct","targets":"capsule:<id>","proves":"captured replay scripts"}
            ],
            "surfaces": [
                {"surface":"status","mode":"authoritative","targets":"status:<session>"},
                {"surface":"session_start","mode":"advisory","targets":"session_start:<session>"}
            ],
            "reason": "every live durable application table has a declared validation target family"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("validation coverage evidence"));
    assert!(text.contains("tables=21/21 direct=20 aggregate=1 uncovered=0"));
    assert!(text.contains("channel_readiness_attempts [direct] -> readiness_attempt:<id>"));
    assert!(text.contains("surfaces: status=authoritative, session_start=advisory"));
    assert!(text.contains("every live durable application table"));
}

#[test]
fn validate_render_lists_table_coverage_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "table:messages",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"table_coverage","status":"passed","summary":"table `messages` has 12 row(s) and maps to `message:<id> | event:<id>`"}
        ],
        "coverage_evidence": {
            "kind": "validation_table",
            "table": "messages",
            "present": true,
            "covered": true,
            "row_count": 12,
            "column_count": 4,
            "columns": ["message_id", "thread_id", "channel_h", "body"],
            "mode": "direct",
            "targets": "message:<id> | event:<id>",
            "proves": "canonical chat rows",
            "sample_targets": [
                {"target":"message:event-123", "also": "event:event-123", "row": {"message_id":"event-123"}}
            ],
            "reason": "use `message:<id> | event:<id>` to validate rows from `messages`; this table proves canonical chat rows"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("table coverage evidence"));
    assert!(text.contains("table=messages present=true covered=true rows=12 columns=4"));
    assert!(text.contains("mode=direct targets=message:<id> | event:<id>"));
    assert!(text.contains("columns: message_id, thread_id, channel_h, body"));
    assert!(text.contains("sample: message:event-123 (also event:event-123)"));
}

#[test]
fn validate_render_lists_lookup_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "lookup:event-123",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"lookup","status":"passed","summary":"lookup `event-123` matched durable validation handle(s)"}
        ],
        "coverage_evidence": {
            "kind": "validation_lookup",
            "needle": "event-123",
            "found": true,
            "match_count": 2,
            "matches": [
                {"table":"messages", "target":"message:event-123", "also":"event:event-123"},
                {"table":"message_recipients", "target":"recipient:event-123:pk", "also":"message:event-123"}
            ],
            "reason": "matches are concrete validation handles; run any target to inspect that row"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("lookup evidence"));
    assert!(text.contains("needle=event-123 matches=2"));
    assert!(text.contains("messages -> message:event-123 (also event:event-123)"));
    assert!(text.contains("message_recipients -> recipient:event-123:pk"));
}
