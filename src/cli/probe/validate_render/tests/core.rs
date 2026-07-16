use super::*;
use serde_json::json;

#[test]
fn validate_render_lists_checks_and_limitations() {
    let v = json!({
        "verb": "validate",
        "target": "status:s1",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"oracle","status":"passed","summary":"all green"},
            {"name":"seams","status":"not_proven","summary":"coverage 85%"},
            {"name":"simulate","status":"passed","summary":"would publish"}
        ],
        "limitations": ["host effects not fully covered"],
        "why": {
            "handle":"status:s1",
            "kind":"status",
            "found":true,
            "resource_key":"status/s1",
            "last_kind":"Replace",
            "cause":"planner",
            "input_causes":["status/s1/title"]
        },
        "simulate": {
            "surface":"status",
            "fact":{"kind":"StatusDrive"},
            "would_effect":true,
            "would_publish":true,
            "commands":[{"kind":30315,"op":"replace","resource":"status/s1"}],
            "changed":["status/s1/title"]
        },
        "acid": {
            "handle":"status:s1",
            "surface":"status",
            "cause":"status/s1/title",
            "necessary":true,
            "unrelated_stable":true,
            "ok":true,
            "original_hash":"sha256:o",
            "removed_hash":"sha256:r",
            "unrelated_hash":"sha256:o"
        },
        "explain": {
            "receipts": [{
                "surface":"status",
                "transaction_id":5,
                "revision":2,
                "artifact_ref":"evt-1"
            }]
        }
    });
    let text = render_validate(&v);
    assert!(text.contains("validate status:s1"));
    assert!(text.contains("oracle"));
    assert!(text.contains("not_proven"));
    assert!(text.contains("host effects not fully covered"));
    assert!(text.contains("why status:s1"));
    assert!(text.contains("simulate status"));
    assert!(text.contains("acid status:s1"));
    assert!(text.contains("status/s1/title"));
    assert!(text.contains("[status] txn 5 rev 2 -> evt-1"));
}

#[test]
fn validate_render_lists_unowned_fact_evidence() {
    let v = json!({
        "verb": "validate",
        "target": null,
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"fact","status":"not_proven","summary":"ClockTick has no validating Trellis surface yet"}
        ],
        "limitations": ["clock ticks still feed several imperative loops"],
        "fact_evidence": {
            "kind": "ClockTick",
            "supported": false,
            "frontier": "timekeeping",
            "reason": "clock ticks still feed several imperative loops"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("fact evidence"));
    assert!(text.contains("ClockTick: not proven (timekeeping)"));
    assert!(text.contains("clock ticks still feed several imperative loops"));
}

#[test]
fn validate_render_lists_invalid_fact_evidence() {
    let v = json!({
        "verb": "validate",
        "target": null,
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"fact","status":"failed","summary":"fact is not a valid InputFact"}
        ],
        "limitations": ["probe: invalid fact: unknown variant `Bogus`"],
        "fact_evidence": {
            "kind": "InvalidInputFact",
            "supported": false,
            "valid": false,
            "frontier": "input_decode",
            "summary": "fact is not a valid InputFact",
            "reason": "probe: invalid fact: unknown variant `Bogus`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("fact evidence"));
    assert!(text.contains("InvalidInputFact: invalid (input_decode)"));
    assert!(text.contains("unknown variant `Bogus`"));
}

#[test]
fn validate_render_lists_parameter_evidence() {
    let v = json!({
        "verb": "validate",
        "target": null,
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"input","status":"failed","summary":"invalid validate parameter(s): target, since"}
        ],
        "parameter_evidence": [
            {
                "parameter": "target",
                "kind": "invalid_parameter",
                "valid": false,
                "summary": "parameter `target` must be a string",
                "reason": "validate parameter `target` must be a string"
            },
            {
                "parameter": "since",
                "kind": "invalid_parameter",
                "valid": false,
                "summary": "parameter `since` must be an integer unix-millis stamp",
                "reason": "validate parameter `since` must be an integer unix-millis stamp"
            }
        ]
    });

    let text = render_validate(&v);

    assert!(text.contains("parameter evidence"));
    assert!(text.contains("target: invalid"));
    assert!(text.contains("since: invalid"));
    assert!(text.contains("validate parameter `target` must be a string"));
}

#[test]
fn validate_render_lists_error_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "status:s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"resource_accounting","status":"failed","summary":"no such table: trellis_commits"},
            {"name":"simulate","status":"not_proven","summary":"hook graph missing"},
            {"name":"replay","status":"failed","summary":"capsule id must be an integer"}
        ],
        "stats_error": "no such table: trellis_commits",
        "simulate_error": "hook graph missing",
        "replay_error": "probe replay: capsule id must be an integer"
    });

    let text = render_validate(&v);

    assert!(text.contains("error evidence"));
    assert!(text.contains("stats: no such table: trellis_commits"));
    assert!(text.contains("simulate: hook graph missing"));
    assert!(text.contains("replay: probe replay: capsule id must be an integer"));
}

#[test]
fn validate_render_lists_cause_label_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "subscriptions/daemon/channels",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"cause_label","status":"passed","summary":"cause label `subscriptions/daemon/channels` belongs to subscriptions"}
        ],
        "cause_label_evidence": {
            "target": "subscriptions/daemon/channels",
            "label": "subscriptions/daemon/channels",
            "surface": "subscriptions",
            "kind": "cause_label",
            "supported": true,
            "reason": "subscription cause labels identify Trellis inputs or planner collections, not individual relay resources"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("cause label evidence"));
    assert!(text.contains("subscriptions/daemon/channels: subscriptions"));
    assert!(text.contains("not individual relay resources"));
}
