use super::*;
use serde_json::json;

mod alias;
mod channel;
mod commit;
mod coverage;
mod cursor;
mod event;
mod hook_context;
mod identity;
mod inbox;
mod joined;
mod llm;
mod membership;
mod outbox;
mod project_root;
mod quarantine;
mod readiness_attempt;
mod receipt;
mod recipient;
mod session;
mod session_consistency;
mod state;
mod status;
mod subscription;
mod turn;
mod txn;

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
            "input_causes":["status/s1/activity"]
        },
        "simulate": {
            "surface":"status",
            "fact":{"kind":"StatusDrive"},
            "would_effect":true,
            "would_publish":true,
            "commands":[{"kind":30315,"op":"replace","resource":"status/s1"}],
            "changed":["status/s1/activity"]
        },
        "acid": {
            "handle":"status:s1",
            "surface":"status",
            "cause":"status/s1/activity",
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
            }],
            "llm_call": {
                "provider":"ollama",
                "model":"glm",
                "window_hash":"sha256:w",
                "parsed_title":"T",
                "parsed_activity":"A"
            }
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
    assert!(text.contains("status/s1/activity"));
    assert!(text.contains("[status] txn 5 rev 2 -> evt-1"));
    assert!(text.contains("llm ollama / glm"));
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

#[test]
fn validate_render_lists_channel_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "channel:room",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"channel","status":"passed","summary":"channel `room` is materialized from relay kind:39000"}
        ],
        "limitations": ["channel provisioning is a host/provider side effect"],
        "channel_evidence": {
            "target": "channel:room",
            "channel_h": "room",
            "kind": "channel",
            "supported": true,
            "found": true,
            "human_name": "Room",
            "parent": "",
            "project_root": "room",
            "member_count": 2,
            "admin_count": 1,
            "membership_snapshot": true,
            "reason": "channel provisioning is a host/provider side effect"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("channel evidence"));
    assert!(text.contains("room: materialized"));
    assert!(text.contains("members=2 admins=1 membership_snapshot=true"));
    assert!(text.contains("host/provider side effect"));
}

#[test]
fn validate_render_lists_awareness_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "awareness:room",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"awareness","status":"passed","summary":"awareness for channel `room` has 2 live row(s)"}
        ],
        "limitations": [],
        "awareness_evidence": {
            "channel_h": "room",
            "channel_confirmed": true,
            "channel_name": "Room",
            "parent": "",
            "project_root": "room",
            "membership_snapshot": true,
            "member_count": 3,
            "admin_count": 1,
            "row_count": 2,
            "local_row_count": 1,
            "peer_row_count": 1,
            "fresh_row_count": 2,
            "spawnable_count": 4
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("awareness evidence"));
    assert!(text.contains("room: confirmed"));
    assert!(text.contains("members=3 admins=1 membership_snapshot=true"));
    assert!(text.contains("live_rows=2 local=1 peer=1 fresh=2 spawnable_local=4"));
}

#[test]
fn validate_render_lists_message_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "message:event-123",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"message","status":"passed","summary":"message `event-123` is accepted in channel `room`"}
        ],
        "limitations": ["message validation proves the local canonical chat read model"],
        "message_evidence": {
            "requested_id": "event-123",
            "message_id": "event-123",
            "found": true,
            "channel_h": "room",
            "channel_confirmed": true,
            "direction": "outbound",
            "sync_state": "accepted",
            "author_session": "sender-session",
            "native_event_id": "event-123",
            "recipient_count": 2,
            "delivered_recipient_count": 1,
            "pending_recipient_count": 1,
            "body_len": 23,
            "body_preview": "hello from the fabric",
            "reason": "message validation proves the local canonical chat read model"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("message evidence"));
    assert!(text.contains("event-123: accepted channel=room (confirmed)"));
    assert!(text.contains("direction=outbound author_session=\"sender-session\""));
    assert!(text.contains("recipients=2 delivered=1 pending=1"));
    assert!(text.contains("preview=\"hello from the fabric\""));
}

#[test]
fn validate_render_lists_session_start_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "session_start:s1",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"session_start_outcome","status":"failed","summary":"session_start `s1` failed at channel_ready"}
        ],
        "limitations": ["relay rejected event: timeout"],
        "session_start_evidence": {
            "session_id": "s1",
            "found": true,
            "action": "RecordFailed",
            "channel_h": "room",
            "reassert": false,
            "has_channel_ready_intent": true,
            "has_spawn_intent": true,
            "ensure_subscription": true,
            "replay_chat": false,
            "failure_stage": "channel_ready",
            "failure_error": "relay rejected event: timeout",
            "reason": "relay rejected event: timeout"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("session start evidence"));
    assert!(text.contains("s1: RecordFailed channel=room reassert=false"));
    assert!(text.contains("planned host effects: channel_ready, spawn, subscription"));
    assert!(text.contains("failed at channel_ready: relay rejected event: timeout"));
}

#[test]
fn validate_render_lists_session_watch_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "watch:s1",
        "verdict": "passed",
        "ok": true,
        "checks": [
            {"name":"session_watch_outcome","status":"passed","summary":"session_watch `s1` is open and pid 42 is alive"}
        ],
        "limitations": [],
        "session_watch_evidence": {
            "session_id": "s1",
            "graph_open": true,
            "session_row_found": true,
            "session_alive": true,
            "channel_h": "room",
            "agent_slug": "coder",
            "child_pid": 42,
            "process_alive": true,
            "last_seen": 100,
            "reason": ""
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("session watch evidence"));
    assert!(text.contains("s1: graph=open session_row=alive"));
    assert!(text.contains("channel=room agent=coder last_seen=100"));
    assert!(text.contains("pid=42 process_alive=true"));
}

#[test]
fn validate_render_lists_unknown_target_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "not-a-known-target",
        "verdict": "passed_with_limitations",
        "ok": true,
        "checks": [
            {"name":"target","status":"not_proven","summary":"target `not-a-known-target` is not a known validation target"}
        ],
        "limitations": ["target must be a surface, probe handle, explain handle, or `capsule:<id>`"],
        "target_evidence": {
            "target": "not-a-known-target",
            "supported": false,
            "kind": "unknown_target",
            "reason": "target must be a surface, probe handle, explain handle, or `capsule:<id>`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("target evidence"));
    assert!(text.contains("not-a-known-target: not proven"));
    assert!(text.contains("target must be a surface"));
}

#[test]
fn validate_render_marks_invalid_target_evidence() {
    let v = json!({
        "verb": "validate",
        "target": "status/",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"target","status":"failed","summary":"target `status/` is missing a status resource"}
        ],
        "target_evidence": {
            "target": "status/",
            "supported": false,
            "valid": false,
            "kind": "empty_handle",
            "reason": "target `status/` must include a non-empty status resource after `status/`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("target evidence"));
    assert!(text.contains("status/: invalid"));
    assert!(text.contains("non-empty status resource"));
}

#[test]
fn validate_render_avoids_double_colon_for_colon_suffixed_targets() {
    let v = json!({
        "verb": "validate",
        "target": "capsule:",
        "verdict": "failed",
        "ok": false,
        "checks": [
            {"name":"target","status":"failed","summary":"target `capsule:` is missing a replay capsule id"}
        ],
        "target_evidence": {
            "target": "capsule:",
            "supported": false,
            "valid": false,
            "kind": "empty_handle",
            "reason": "target `capsule:` must include a non-empty replay capsule id after `capsule:`"
        }
    });

    let text = render_validate(&v);

    assert!(text.contains("capsule: invalid"));
    assert!(!text.contains("capsule:: invalid"));
}
